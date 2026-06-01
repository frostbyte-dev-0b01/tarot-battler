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
  } finally {
    await browser.close();
    if (server) server.kill("SIGTERM");
  }

  const failed = results.filter((r) => !r.ok);
  console.log(`\n${results.length - failed.length}/${results.length} checks passed`);
  process.exit(failed.length ? 1 : 0);
}

main().catch((e) => { console.error("smoke test crashed:", e); process.exit(1); });
