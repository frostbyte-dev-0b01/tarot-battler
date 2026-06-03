// Tier 2 UI smoke test: drives the static site in a headless browser and
// asserts the core flows work end to end (incl. regressions we've fixed).
//
// Usage (from repo root):
//   node tools/ui/smoke-test.cjs
//
// It serves the repo over http and launches Chromium via Playwright. Override
// discovery with env vars if your setup differs:
//   PW_MODULE   path to the playwright module (default: resolve "playwright",
//               then /opt/node22/lib/node_modules/playwright)
//   PW_CHROME   path to a Chromium/headless_shell binary (default: Playwright's
//               bundled browser; then a known headless_shell path)
//   SMOKE_PORT  http port (default 8099)
const { spawn } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

function resolvePlaywright() {
  const candidates = [process.env.PW_MODULE, "playwright", "/opt/node22/lib/node_modules/playwright"].filter(Boolean);
  for (const c of candidates) {
    try { return require(c); } catch { /* keep trying */ }
  }
  throw new Error("Could not resolve the 'playwright' module — set PW_MODULE.");
}

function resolveChrome() {
  const known = "/opt/pw-browsers/chromium_headless_shell-1194/chrome-linux/headless_shell";
  if (process.env.PW_CHROME && fs.existsSync(process.env.PW_CHROME)) return process.env.PW_CHROME;
  if (fs.existsSync(known)) return known;
  return undefined; // let Playwright use its bundled browser
}

const PORT = Number(process.env.SMOKE_PORT || 8099);
const URL = `http://localhost:${PORT}/tools/ui/index.html`;
const repoRoot = path.resolve(__dirname, "..", "..");

const results = [];
function check(name, cond, detail) {
  const ok = !!cond;
  results.push({ name, ok, detail: ok ? "" : (detail || "") });
  console.log(`${ok ? "PASS" : "FAIL"}  ${name}${ok ? "" : (detail ? "  — " + detail : "")}`);
}

async function main() {
  const { chromium } = resolvePlaywright();
  // Self-serve by default; set SMOKE_SERVE=0 to use an already-running server.
  let server = null;
  if (process.env.SMOKE_SERVE !== "0") {
    server = spawn("python3", ["-m", "http.server", String(PORT)], { cwd: repoRoot, stdio: "ignore" });
    await new Promise((r) => setTimeout(r, 1000));
  }

  const browser = await chromium.launch({
    executablePath: resolveChrome(),
    args: ["--no-sandbox", "--disable-dev-shm-usage", "--single-process"],
  });
  try {
    // ===== Scenario 1: the normal bundled roster (core flows + watch) =====
    const errors = [];
    const page = await browser.newPage();
    page.on("pageerror", (e) => errors.push("pageerror: " + e.message));
    page.on("console", (m) => { if (m.type() === "error") errors.push("console: " + m.text()); });

    await page.goto(URL, { waitUntil: "networkidle" });
    await page.waitForFunction(() => typeof window.runBattleWasm === "function", { timeout: 20000 });
    await page.waitForTimeout(600);
    check("WASM engine ready", true);

    // rules-lib globals are wired (function declarations are on window).
    const previewOk = await page.evaluate(() =>
      typeof window.formatRulePreview === "function" &&
      window.formatRulePreview({ ability: "Smite", when: [{ subject: "self", value: "mp", op: "gte", threshold: 4 }] }) === "Use Smite if MP ≥ 4");
    check("rules-lib helpers available in app", previewOk, "formatRulePreview missing/incorrect");

    // Replay viewer: load a bundled replay and confirm it renders.
    await page.evaluate(async () => {
      const txt = await (await fetch("./sample-data/replays/imperial_phalanx_defeats_omen_engine.json")).text();
      window.loadReplayFromText(txt.trim());
    });
    await page.waitForTimeout(400);
    const replayRendered = await page.evaluate(() => ({
      timeline: document.querySelectorAll("#timeline-list > *").length,
      board: document.querySelectorAll("#battle-board *").length,
    }));
    check("replay loads & renders", replayRendered.timeline > 0 && replayRendered.board > 0, JSON.stringify(replayRendered));

    // Rule attribution shows up somewhere in the timeline.
    const attribution = await page.evaluate(() => document.querySelector(".beat-rule") ? document.querySelector(".beat-rule").textContent.trim() : null);
    check("replay shows rule attribution", !!attribution, "no .beat-rule rendered");

    // Arena: run a small sim and confirm results render.
    await page.evaluate(() => { const el = [...document.querySelectorAll("button,a")].find((x) => /^arena/i.test(x.textContent.trim())); el && el.click(); });
    await page.waitForTimeout(300);
    await page.evaluate(() => { const rc = document.querySelector("#arena-run-count"); if (rc) { rc.value = "3"; rc.dispatchEvent(new Event("input", { bubbles: true })); } });
    await page.evaluate(() => { const b = document.querySelector('[data-arena-foe-all="1"]'); b && b.click(); });
    await page.waitForTimeout(150);
    await page.evaluate(() => document.querySelector("#arena-fight-button")?.click());
    await page.waitForTimeout(3500);
    const arena = await page.evaluate(() => ({
      record: document.querySelector("#arena-record")?.textContent?.trim() || "",
      rows: document.querySelectorAll("#arena-results [data-arena-replay]").length,
    }));
    check("arena produces a record", /win rate/i.test(arena.record), arena.record);
    check("arena lists matchups", arena.rows > 0, `rows=${arena.rows}`);

    // Watch Victory/Defeat keeps the replay loaded (regression for the clobber bug).
    const watch = await page.evaluate(() => {
      const btns = [...document.querySelectorAll('[data-arena-replay]')].filter((b) => !b.disabled);
      if (!btns.length) return { clicked: false };
      btns[0].click();
      return { clicked: true };
    });
    check("arena has a watchable matchup", watch.clicked, "no enabled watch button");
    if (watch.clicked) {
      await page.waitForTimeout(1200); // past the frame that used to clobber it
      const stillLoaded = await page.evaluate(() => {
        const ce = document.querySelector("#current-event-text")?.textContent || "";
        return !/no replay loaded/i.test(ce) && document.querySelectorAll("#timeline-list > *").length > 0;
      });
      check("Watch keeps the replay loaded", stillLoaded, "replay was cleared after watch");
    }

    check("no console/page errors", errors.length === 0, errors.join(" | "));

    // ===== Scenario 2: a roster with a stale team -> Arena error banner (PR #28) =====
    // Reuse the same page (a fresh page can crash single-process Chromium):
    // seed localStorage with two valid teams plus one referencing a removed
    // aspect, then reload so the app reads the stale roster.
    await page.evaluate(() => {
      localStorage.setItem("tarot:teams", JSON.stringify([
        { version: 2, name: "Valid Chariot", characters: [
          { id: "c", template_id: "the_chariot", display_name: "The Chariot", position: { row: 0, col: 0 }, passive: "Pursuit", actives: ["Charge", "Withdraw", "Breakthrough"], aspect: null, rules: [] }] },
        { version: 2, name: "Valid Justice", characters: [
          { id: "j", template_id: "justice", display_name: "Justice", position: { row: 0, col: 0 }, passive: "Sentence", actives: ["Condemn", "Verdict", "Rebuke"], aspect: null, rules: [] }] },
        { version: 2, name: "Stale Team", characters: [
          { id: "e", template_id: "the_emperor", display_name: "The Emperor", position: { row: 0, col: 0 }, passive: "Imperial Formation", actives: ["Taunt", "Command", "Sunder"], aspect: "vitality_charm", rules: [] }] },
      ]));
    });
    await page.reload({ waitUntil: "networkidle" });
    await page.waitForFunction(() => typeof window.runBattleWasm === "function", { timeout: 20000 });
    await page.waitForTimeout(400);
    await page.evaluate(() => { const el = [...document.querySelectorAll("button,a")].find((x) => /^arena/i.test(x.textContent.trim())); el && el.click(); });
    await page.waitForTimeout(300);
    await page.evaluate(() => { const rc = document.querySelector("#arena-run-count"); if (rc) { rc.value = "2"; rc.dispatchEvent(new Event("input", { bubbles: true })); } });
    await page.evaluate(() => { const b = document.querySelector('[data-arena-foe-all="1"]'); b && b.click(); });
    await page.waitForTimeout(150);
    await page.evaluate(() => document.querySelector("#arena-fight-button")?.click());
    await page.waitForTimeout(2500);
    const banner = await page.evaluate(() => document.querySelector(".arena-error-banner")?.textContent?.trim() || null);
    check("arena surfaces stale-team error banner", !!banner && /unknown aspect/i.test(banner), banner || "no banner");

    // ===== Scenario 3: Season tab against a stubbed /api server (PR #42) =====
    // Intercept every /api/** request with canned JSON so the Season workspace
    // can be exercised without running the Rust server.
    const sampleReplay = fs.readFileSync(
      path.join(repoRoot, "tools/ui/sample-data/replays/front_row_defeats_good_stats.json"),
      "utf8",
    );
    const schedule = [
      { index: 0, kind: "banner", budget_delta: 0 },
      { index: 1, kind: "item", budget_delta: 1 },
      { index: 2, kind: "character", budget_delta: 0 },
      { index: 3, kind: "team_passive", budget_delta: 1 },
      { index: 4, kind: "item", budget_delta: 1 },
      { index: 5, kind: "character", budget_delta: 1 },
      { index: 6, kind: "swap", budget_delta: 0 },
      { index: 7, kind: "item", budget_delta: 1 },
    ];
    const stubs = {
      version: { name: "tarot-server", version: "0.1.0" },
      join: { player: { id: "tester", name: "Tester", points: 10 }, season: { id: "season-1", name: "Season 1" } },
      season: {
        season: { id: "season-1", name: "Season 1", day: 1, beats_revealed: 3, seed: 1 },
        schedule,
        current_beat: 2,
        budget: 11,
      },
      standings: { standings: [
        { id: "tester", name: "Tester", points: 10 },
        { id: "rival", name: "Rival", points: 5 },
      ] },
      results: { results: [
        { id: "d0-tester-vs-rival", day: 0, player_a: "tester", player_b: "rival", winner: "a", seed: 1, replay_id: "d0-tester-vs-rival" },
      ] },
    };
    // The open beat (index 2) is unclaimed until the client claims it; the
    // /api/team stub allows only chariot/justice archetypes so we can exercise
    // both the success and the rejection paths.
    let beat2Claimed = null;
    const teamPoolAllow = ["the_chariot", "justice"];
    const draftPayload = () => ({
      player: "tester",
      current_beat: 2,
      budget: 11,
      unlocked: { archetypes: ["the_chariot", "justice", "the_emperor"], aspects: [], team_passives: ["Aegis"], banner: "Rally" },
      beats: [
        { index: 0, kind: "banner", budget_delta: 0, offers: ["Rally", "Bulwark"], claimed: "Rally", open: false },
        { index: 1, kind: "item", budget_delta: 1, offers: ["Sharp"], claimed: "Sharp", open: false },
        { index: 2, kind: "character", budget_delta: 0, offers: ["the_fool", "the_hermit"], claimed: beat2Claimed, open: true },
      ],
    });
    await page.route("**/api/**", (route) => {
      // NB: a local `const URL` (the page address) shadows the global URL
      // constructor in this file, so match on the raw request URL string.
      const u = route.request().url();
      const json = (obj) => route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(obj) });
      const fail = (msg) => route.fulfill({ status: 400, contentType: "application/json", body: JSON.stringify({ error: msg }) });
      if (u.includes("/api/version")) return json(stubs.version);
      if (u.includes("/api/join")) return json(stubs.join);
      if (u.includes("/api/season")) return json(stubs.season);
      if (u.includes("/api/standings")) return json(stubs.standings);
      if (u.includes("/api/stats")) return json({ stats: [
        { player: "tester", name: "Tester", points: 10, wins: 1, losses: 0, draws: 0 },
        { player: "rival", name: "Rival", points: 5, wins: 0, losses: 1, draws: 0 },
      ] });
      if (u.includes("/api/admin/run-finals")) return json({ matches: 3, victor: "tester", victor_name: "Tester", already_run: false });
      if (u.includes("/api/admin/reset-season")) return json({ season: { id: "season-2", name: "Season 2", day: 0, beats_revealed: 1, seed: 2 } });
      if (u.includes("/api/results")) return json(stubs.results);
      if (u.includes("/api/draft/claim")) {
        const body = JSON.parse(route.request().postData() || "{}");
        beat2Claimed = body.choice;
        return json(draftPayload());
      }
      if (u.includes("/api/draft")) return json(draftPayload());
      if (u.includes("/api/team")) {
        const body = JSON.parse(route.request().postData() || "{}");
        const ids = ((body.team && body.team.characters) || []).map((c) => c.template_id);
        const locked = ids.filter((id) => !teamPoolAllow.includes(id));
        if (locked.length) return fail(`team uses locked archetype '${locked[0]}' (not in this season's pool)`);
        return json({ ok: true, player: "tester" });
      }
      if (u.includes("/api/replays/")) return route.fulfill({ status: 200, contentType: "application/json", body: sampleReplay });
      return route.fulfill({ status: 404, contentType: "application/json", body: '{"error":"not found"}' });
    });

    // Open the Season tab → should reach the join form (no stored player).
    await page.evaluate(() => localStorage.removeItem("tarot:season:player"));
    await page.evaluate(() => { const el = [...document.querySelectorAll("button")].find((x) => /^season$/i.test(x.textContent.trim())); el && el.click(); });
    await page.waitForFunction(() => !!document.querySelector("#season-name"), { timeout: 5000 }).catch(() => {});
    const joinFormShown = await page.evaluate(() => !!document.querySelector('[data-season-action="join"]'));
    check("season tab shows join form when no player is stored", joinFormShown);

    // Join → dashboard with schedule + standings.
    await page.evaluate(() => { const i = document.querySelector("#season-name"); if (i) { i.value = "Tester"; } });
    await page.evaluate(() => document.querySelector('[data-season-action="join"]')?.click());
    await page.waitForFunction(() => document.querySelectorAll(".season-beat").length > 0, { timeout: 5000 }).catch(() => {});
    const dash = await page.evaluate(() => ({
      beats: document.querySelectorAll(".season-beat").length,
      standings: document.querySelectorAll(".season-standing-row").length,
      results: document.querySelectorAll(".season-result").length,
      clock: document.querySelector(".season-header .season-hint")?.textContent?.trim() || "",
    }));
    check("season dashboard renders schedule", dash.beats === 8, `beats=${dash.beats}`);
    check("season dashboard renders standings", dash.standings === 2, `rows=${dash.standings}`);
    check("season dashboard renders results", dash.results === 1, `results=${dash.results}`);
    check("season clock line is legible", /Day \d+ · Beat \d+ of 8/.test(dash.clock), dash.clock);

    // Stats panel renders per-player W/L/D.
    const statRows = await page.evaluate(() => document.querySelectorAll(".season-stat-row").length);
    check("season stats panel renders", statRows === 2, `rows=${statRows}`);

    // Finals (Victors round) → admin message names the victor.
    await page.evaluate(() => document.querySelector('[data-season-action="run-finals"]')?.click());
    await page.waitForFunction(() => /Victor/.test(document.querySelector(".season-admin ~ .season-ok, .season-ok")?.textContent || ""), { timeout: 5000 }).catch(() => {});
    const finalsMsg = await page.evaluate(() => [...document.querySelectorAll(".season-ok")].map((e) => e.textContent).join(" "));
    check("season finals announces a victor", /Victor: Tester/.test(finalsMsg), finalsMsg);

    // Draft UI: the open beat offers claimable options, by display name + blurb.
    const draftOffers = await page.evaluate(() => ({
      count: document.querySelectorAll('.season-open-beat .season-offer').length,
      names: [...document.querySelectorAll('.season-open-beat .season-offer-name')].map((e) => e.textContent.trim()),
      descs: document.querySelectorAll('.season-open-beat .season-offer-desc').length,
    }));
    check("season open beat shows claimable offers", draftOffers.count >= 2, `offers=${draftOffers.count}`);
    check("season draft offers use display names", draftOffers.names.includes("The Fool"), draftOffers.names.join(","));
    check("season draft offers show descriptions", draftOffers.descs >= 2, `descs=${draftOffers.descs}`);

    // Claim the first offer → it reflects as claimed (by display name).
    await page.evaluate(() => document.querySelector('.season-offer[data-season-action="claim"]')?.click());
    await page.waitForFunction(() => !!document.querySelector('.season-offer.is-claimed'), { timeout: 5000 }).catch(() => {});
    const claimed = await page.evaluate(() => document.querySelector('.season-offer.is-claimed')?.textContent?.trim() || "");
    check("season claim marks the chosen offer", /The Fool/.test(claimed), claimed);

    // Season builder: submitting a team within the pool succeeds.
    const builderReady = await page.evaluate(() => !!document.querySelector('[data-season-action="submit-team"]'));
    check("season team builder renders", builderReady);
    await page.evaluate(() => document.querySelector('[data-season-action="submit-team"]')?.click());
    await page.waitForFunction(() => !!document.querySelector('.season-builder-actions .season-ok'), { timeout: 5000 }).catch(() => {});
    const submitOk = await page.evaluate(() => document.querySelector('.season-builder-actions .season-ok')?.textContent?.trim() || "");
    check("season team submit succeeds within pool", /submitted/i.test(submitOk), submitOk);

    // Submitting a team that uses locked content is rejected with a message.
    await page.evaluate(() => {
      const sel = document.querySelector('select[data-season-action="select-team"]');
      if (sel) { sel.value = "Stale Team"; sel.dispatchEvent(new Event("change", { bubbles: true })); }
    });
    await page.waitForTimeout(300);
    await page.evaluate(() => document.querySelector('[data-season-action="submit-team"]')?.click());
    await page.waitForFunction(
      () => /locked archetype/i.test(document.querySelector('.season-builder-actions .season-error')?.textContent || ""),
      { timeout: 5000 },
    ).catch(() => {});
    const submitErr = await page.evaluate(() => document.querySelector('.season-builder-actions .season-error')?.textContent?.trim() || "");
    check("season team submit rejects locked content with a message", /locked archetype/i.test(submitErr), submitErr);

    // Legality badge: with a season context loaded, season-legal teams get a
    // seal in the Arena (the stale team, which uses a locked aspect, does not).
    await page.evaluate(() => { const el = [...document.querySelectorAll("button")].find((x) => /^arena$/i.test(x.textContent.trim())); el && el.click(); });
    await page.waitForFunction(() => document.querySelectorAll(".arena-foe").length > 0, { timeout: 5000 }).catch(() => {});
    const seals = await page.evaluate(() => ({
      total: document.querySelectorAll(".arena-foe .season-seal").length,
      staleSealed: [...document.querySelectorAll(".arena-foe")].some(
        (el) => /Stale Team/.test(el.textContent) && el.querySelector(".season-seal"),
      ),
    }));
    check("arena marks season-legal teams with a seal", seals.total === 2, `seals=${seals.total}`);
    check("arena leaves season-illegal teams unsealed", seals.staleSealed === false);

    // Phase 2/3: Season build mode in the Team builder.
    await page.evaluate(() => { const el = [...document.querySelectorAll("button")].find((x) => /^team$/i.test(x.textContent.trim())); el && el.click(); });
    await page.waitForFunction(() => !!document.querySelector('[data-builder-mode="season"]'), { timeout: 5000 }).catch(() => {});
    await page.evaluate(() => document.querySelector('[data-builder-mode="season"]')?.click());
    await page.waitForFunction(() => !!document.querySelector(".season-build-strip"), { timeout: 5000 }).catch(() => {});
    const seasonBuild = await page.evaluate(() => ({
      strip: !!document.querySelector(".season-build-strip"),
      budget: (document.querySelector(".budget-meter")?.textContent || "").replace(/\s+/g, " ").trim(),
    }));
    check("team builder enters Season build mode", seasonBuild.strip, JSON.stringify(seasonBuild));
    check("season build shows the season budget", /Season budget/.test(seasonBuild.budget) && /\/ 11/.test(seasonBuild.budget), seasonBuild.budget);

    // Load a pool-legal team → submit enabled → submitting from the builder works.
    await page.evaluate(() => { const sel = document.querySelector(".roster-select"); if (sel) { sel.value = "Valid Chariot"; sel.dispatchEvent(new Event("change", { bubbles: true })); } });
    await page.waitForFunction(() => !!document.querySelector(".season-build-ok"), { timeout: 5000 }).catch(() => {});
    const legal = await page.evaluate(() => ({
      ok: !!document.querySelector(".season-build-ok"),
      submitDisabled: document.querySelector('[data-team-action="submit-team-season"]')?.disabled,
    }));
    check("season build marks a pool-legal team legal", legal.ok && legal.submitDisabled === false, JSON.stringify(legal));
    await page.evaluate(() => document.querySelector('[data-team-action="submit-team-season"]')?.click());
    await page.waitForFunction(() => /Submitted/.test(document.querySelector(".season-build-strip .season-ok")?.textContent || ""), { timeout: 5000 }).catch(() => {});
    const builderSubmit = await page.evaluate(() => document.querySelector(".season-build-strip .season-ok")?.textContent?.trim() || "");
    check("season build submits a team from the Team tab", /Submitted/.test(builderSubmit), builderSubmit);

    // Load a pool-illegal team → flagged, submit disabled.
    await page.evaluate(() => { const sel = document.querySelector(".roster-select"); if (sel) { sel.value = "Stale Team"; sel.dispatchEvent(new Event("change", { bubbles: true })); } });
    await page.waitForFunction(() => !!document.querySelector(".season-build-bad"), { timeout: 5000 }).catch(() => {});
    const illegal = await page.evaluate(() => ({
      bad: !!document.querySelector(".season-build-bad"),
      submitDisabled: document.querySelector('[data-team-action="submit-team-season"]')?.disabled,
      reasons: document.querySelectorAll(".season-build-reasons li").length,
    }));
    check("season build flags an illegal team and disables submit", illegal.bad && illegal.submitDisabled === true && illegal.reasons >= 1, JSON.stringify(illegal));

    // Back to the Season tab for the Watch step.
    await page.evaluate(() => { const el = [...document.querySelectorAll("button")].find((x) => /^season$/i.test(x.textContent.trim())); el && el.click(); });
    await page.waitForFunction(() => document.querySelectorAll(".season-result").length > 0, { timeout: 5000 }).catch(() => {});

    // Watch a result → loads the replay into the existing viewer.
    await page.evaluate(() => document.querySelector('[data-season-action="watch"]')?.click());
    await page.waitForFunction(
      () => document.getElementById("replay-viewer")?.classList.contains("is-active") &&
            document.querySelectorAll("#battle-board .board-cell, #battle-board .grid-unit, #battle-board > *").length > 0,
      { timeout: 8000 },
    ).catch(() => {});
    const watched = await page.evaluate(() => ({
      replayActive: document.getElementById("replay-viewer")?.classList.contains("is-active"),
      board: document.querySelectorAll("#battle-board > *").length,
    }));
    check("season Watch opens the replay viewer", watched.replayActive && watched.board > 0, JSON.stringify(watched));
    await page.unroute("**/api/**");
  } finally {
    await browser.close();
    if (server) server.kill("SIGTERM");
  }

  const failed = results.filter((r) => !r.ok);
  console.log(`\n${results.length - failed.length}/${results.length} checks passed`);
  process.exit(failed.length ? 1 : 0);
}

main().catch((e) => { console.error("smoke test crashed:", e); process.exit(1); });
