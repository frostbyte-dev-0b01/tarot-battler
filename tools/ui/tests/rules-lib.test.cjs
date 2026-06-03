// Tier 1 UI unit tests: pure rule/format/stat helpers (no DOM, no browser).
// Run from the repo root:  node --test tools/ui/tests/
const test = require("node:test");
const assert = require("node:assert/strict");
const R = require("../rules-lib.js");

test("wilsonInterval handles the empty case and brackets the rate", () => {
  assert.deepEqual(R.wilsonInterval(0, 0), [0, 0]);
  const [lo, hi] = R.wilsonInterval(5, 10);
  assert.ok(lo > 0 && lo < 0.5, `lo=${lo}`);
  assert.ok(hi > 0.5 && hi < 1, `hi=${hi}`);
  // A clean sweep (all wins) keeps a non-degenerate lower bound; the upper
  // bound clamps to 1.
  const [loAll, hiAll] = R.wilsonInterval(10, 10);
  assert.ok(loAll > 0 && loAll < 1, `loAll=${loAll}`);
  assert.equal(hiAll, 1);
});

test("formatPct rounds to a whole percent", () => {
  assert.equal(R.formatPct(0), "0%");
  assert.equal(R.formatPct(0.5), "50%");
  assert.equal(R.formatPct(0.333), "33%");
  assert.equal(R.formatPct(1), "100%");
});

test("clampValue and moveArrayItem", () => {
  assert.equal(R.clampValue(5, 0, 3), 3);
  assert.equal(R.clampValue(-1, 0, 3), 0);
  assert.equal(R.clampValue(2, 0, 3), 2);
  const arr = ["a", "b", "c"];
  R.moveArrayItem(arr, 0, 2);
  assert.deepEqual(arr, ["b", "c", "a"]);
  const arr2 = ["a", "b"];
  R.moveArrayItem(arr2, 0, 5); // out of range: no-op
  assert.deepEqual(arr2, ["a", "b"]);
});

test("getConditionValueType detects each value shape", () => {
  assert.equal(R.getConditionValueType({ value: "hp" }), "hp");
  assert.equal(R.getConditionValueType({ value: "mp" }), "mp");
  assert.equal(R.getConditionValueType({ value: { stat: "mgt" } }), "stat");
  assert.equal(R.getConditionValueType({ value: { status_stacks: "Omen" } }), "status_stacks");
  assert.equal(R.getConditionValueType({ value: { has_status: "Ward" } }), "status_stacks");
  assert.equal(R.getConditionValueType({ value: { condition_stacks: "Stunned" } }), "condition_stacks");
  assert.equal(R.getConditionValueType({}), "hp"); // default
});

test("isAlwaysCondition recognizes the canonical self HP >= 1 and nothing else", () => {
  assert.equal(R.isAlwaysCondition(R.createEmptyCondition()), true);
  assert.equal(R.isAlwaysCondition({ subject: "self", value: "hp", op: "gte", threshold: 1 }), true);
  assert.equal(R.isAlwaysCondition({ subject: "self", value: "hp", op: "lte", threshold: 50 }), false);
  assert.equal(R.isAlwaysCondition({ subject: "target", value: "hp", op: "gte", threshold: 1 }), false);
  assert.equal(R.isAlwaysCondition({ subject: "self", value: "mp", op: "gte", threshold: 1 }), false);
});

test("formatConditionPreview renders each value type", () => {
  assert.equal(R.formatConditionPreview({ subject: "self", value: "hp", op: "lte", threshold: 50 }), "HP ≤ 50%");
  assert.equal(R.formatConditionPreview({ subject: "self", value: "mp", op: "gte", threshold: 4 }), "MP ≥ 4");
  assert.equal(R.formatConditionPreview({ subject: "self", value: { stat: "mgt" }, op: "gte", threshold: 10 }), "MGT ≥ 10");
  assert.equal(R.formatConditionPreview({ subject: "self", value: { status_stacks: "Omen" }, op: "gte", threshold: 3 }), "Omen ≥ 3");
  assert.equal(R.formatConditionPreview({ subject: "self", value: "self_row", op: "gte", threshold: 1 }), "Column ≥ Middle");
  // Non-self subject gets a label prefix.
  assert.equal(R.formatConditionPreview({ subject: "any_ally", value: "hp", op: "lte", threshold: 30 }), "Any ally HP ≤ 30%");
  assert.equal(R.formatConditionPreview({ subject: "self", value: "focused_by_count", op: "gte", threshold: 2 }), "Enemies Targeting ≥ 2");
  // The canonical always condition.
  assert.equal(R.formatConditionPreview({ subject: "self", value: "hp", op: "gte", threshold: 1 }), "Always");
});

test("formatRulePreview joins by AND/OR and drops always-conditions", () => {
  const always = { subject: "self", value: "hp", op: "gte", threshold: 1 };
  assert.equal(R.formatRulePreview({ ability: "Smite", when: [] }), "Use Smite always");
  assert.equal(R.formatRulePreview({ ability: "Smite", when: [always] }), "Use Smite always");
  assert.equal(
    R.formatRulePreview({
      ability: "Charge",
      when: [{ subject: "self", value: "mp", op: "gte", threshold: 4 }, { subject: "self", value: "self_row", op: "gte", threshold: 1 }],
    }),
    "Use Charge if MP ≥ 4 and Column ≥ Middle",
  );
  assert.equal(
    R.formatRulePreview({
      ability: "Charge",
      match_any: true,
      when: [{ subject: "self", value: "mp", op: "gte", threshold: 4 }, { subject: "self", value: "hp", op: "lte", threshold: 30 }],
    }),
    "Use Charge if MP ≥ 4 or HP ≤ 30%",
  );
  // With match_any, an always-condition is kept (it changes the OR semantics).
  assert.equal(
    R.formatRulePreview({ ability: "Smite", match_any: true, when: [always, { subject: "self", value: "mp", op: "gte", threshold: 4 }] }),
    "Use Smite if Always or MP ≥ 4",
  );
});

test("formatRuleConditionClause is the why-clause without the ability prefix", () => {
  assert.equal(R.formatRuleConditionClause({ when: [] }), "always");
  assert.equal(
    R.formatRuleConditionClause({ when: [{ subject: "self", value: "hp", op: "lte", threshold: 50 }] }),
    "HP ≤ 50%",
  );
});

test("getAllowedRuleValueOptions respects subject scoping", () => {
  const selfVals = R.getAllowedRuleValueOptions("self").map((o) => o.value);
  assert.ok(selfVals.includes("use_count"), "self should offer cadence");
  assert.ok(selfVals.includes("focused_by_count"), "self should offer threat");
  const targetVals = R.getAllowedRuleValueOptions("target").map((o) => o.value);
  assert.ok(!targetVals.includes("focused_by_count"), "target should not offer focused_by_count");
  assert.ok(targetVals.includes("target_companion_count"));
  // Unknown subject falls back to self's list.
  assert.deepEqual(R.getAllowedRuleValueOptions("bogus").map((o) => o.value), selfVals);
});

test("renderRuleValueSelectOptions groups options and never drops the saved value", () => {
  const opts = R.getAllowedRuleValueOptions("self");
  const html = R.renderRuleValueSelectOptions(opts, "hp");
  assert.ok(html.includes("<optgroup"), "should render optgroups");
  assert.ok(/<option value="hp"[^>]*selected/.test(html), "saved value selected");
  // A saved value not in the allowed list is appended so it can't silently vanish.
  const html2 = R.renderRuleValueSelectOptions(R.getAllowedRuleValueOptions("target"), "use_count");
  assert.ok(/<option value="use_count"[^>]*selected/.test(html2), "out-of-list value still present");
});

test("escapeHtml escapes the dangerous characters", () => {
  assert.equal(R.escapeHtml(`<a href="x" o='y'>&`), "&lt;a href=&quot;x&quot; o=&#39;y&#39;&gt;&amp;");
});

test("beatKindLabel maps server tags to friendly labels", () => {
  assert.equal(R.beatKindLabel("banner"), "Banner");
  assert.equal(R.beatKindLabel("character"), "New Character");
  assert.equal(R.beatKindLabel("team_passive"), "Team Passive");
  assert.equal(R.beatKindLabel("swap"), "Swap");
  // Unknown kinds pass through as their raw string.
  assert.equal(R.beatKindLabel("mystery"), "mystery");
});

test("seasonClockLine reads day/beat as 1-based and clamps to the season length", () => {
  assert.equal(R.seasonClockLine({ day: 0, beats_revealed: 1 }), "Day 1 · Beat 1 of 8");
  assert.equal(R.seasonClockLine({ day: 2, beats_revealed: 3 }), "Day 3 · Beat 3 of 8");
  // beats_revealed beyond the schedule clamps; a fresh season shows Beat 1.
  assert.equal(R.seasonClockLine({ day: 0, beats_revealed: 0 }), "Day 1 · Beat 1 of 8");
  assert.equal(R.seasonClockLine({ day: 0, beats_revealed: 99 }, 8), "Day 1 · Beat 8 of 8");
  assert.equal(R.seasonClockLine(null), "");
});

test("tallyRecord counts wins/losses/draws for a player across both sides", () => {
  const results = [
    { player_a: "ada", player_b: "bo", winner: "a" },   // ada win
    { player_a: "bo", player_b: "ada", winner: "a" },   // ada loss (bo won)
    { player_a: "ada", player_b: "cy", winner: "draw" }, // draw
    { player_a: "bo", player_b: "cy", winner: "b" },     // not ada's match
  ];
  assert.deepEqual(R.tallyRecord("ada", results), { wins: 1, losses: 1, draws: 1 });
  assert.deepEqual(R.tallyRecord("nobody", results), { wins: 0, losses: 0, draws: 0 });
  assert.deepEqual(R.tallyRecord("ada", null), { wins: 0, losses: 0, draws: 0 });
});

test("formatStatBonuses orders stats and signs them, skipping zeros", () => {
  assert.equal(R.formatStatBonuses({ mgt: 2, vit: -2, arm: -1 }), "VIT -2, MGT +2, ARM -1");
  assert.equal(R.formatStatBonuses({ spd: 0, mag: 3 }), "MAG +3");
  assert.equal(R.formatStatBonuses(null), "");
});

test("describeDraftOffer resolves display names + tooltips per kind", () => {
  const catalogs = {
    archetypes: { the_emperor: { display_name: "The Emperor", cost: 3, passive_pool: ["Imperial Formation"] } },
    aspects: { aspect_of_ruin: { display_name: "Aspect of Ruin", description: "Trade durability for pressure.", stat_bonuses: { mgt: 2, vit: -2 } } },
    teamPassives: { Aegis: { description: "Resist one debuff." } },
    banners: { Rally: { description: "First turn comes sooner.", scope: "team" } },
    passiveDescriptions: { "Imperial Formation": "Buffs the front row." },
  };
  const character = R.describeDraftOffer("character", "the_emperor", catalogs);
  assert.equal(character.label, "The Emperor");
  assert.match(character.tooltip, /Cost 3/);
  assert.match(character.tooltip, /Imperial Formation — Buffs the front row\./);

  const item = R.describeDraftOffer("item", "aspect_of_ruin", catalogs);
  assert.equal(item.label, "Aspect of Ruin");
  assert.match(item.tooltip, /Trade durability/);
  assert.match(item.tooltip, /VIT -2, MGT \+2/);

  assert.equal(R.describeDraftOffer("team_passive", "Aegis", catalogs).label, "Aegis");
  assert.match(R.describeDraftOffer("team_passive", "Aegis", catalogs).tooltip, /Resist one debuff/);

  const banner = R.describeDraftOffer("banner", "Rally", catalogs);
  assert.match(banner.tooltip, /First turn comes sooner.*affects team/);

  // Unknown id falls back to the raw id with no tooltip.
  assert.deepEqual(R.describeDraftOffer("character", "mystery", catalogs), { label: "mystery", tooltip: "" });
});

test("teamSeasonValidity passes a team within the unlocked pool + budget", () => {
  const unlocked = { archetypes: ["the_emperor", "the_moon"], aspects: ["aspect_of_ruin"], teamPassives: ["Aegis"], banner: "Rally", budget: 12 };
  const team = {
    characters: [
      { id: "e", template_id: "the_emperor", aspect: "aspect_of_ruin" },
      { id: "m", template_id: "the_moon" },
    ],
    team_passives: ["Aegis"],
    commander: "e",
    banner: "Rally",
  };
  const v = R.teamSeasonValidity(team, unlocked, 9);
  assert.equal(v.valid, true);
  assert.deepEqual(v.reasons, []);
});

test("teamSeasonValidity flags each kind of violation", () => {
  const unlocked = { archetypes: ["the_emperor"], aspects: [], teamPassives: [], banner: "Rally", budget: 8 };
  const team = {
    characters: [
      { id: "e", template_id: "the_emperor" },
      { id: "h", template_id: "the_hermit", aspect: "aspect_of_ruin" }, // locked archetype + locked aspect
    ],
    team_passives: ["Aegis"], // locked passive
    commander: "ghost",        // not a team member
    banner: "Bulwark",         // not the drafted banner
  };
  const v = R.teamSeasonValidity(team, unlocked, 12); // over the 8 budget
  assert.equal(v.valid, false);
  const kinds = v.reasons.map((r) => r.kind).sort();
  assert.deepEqual(kinds, [
    "banner_mismatch",
    "commander_missing",
    "locked_archetype",
    "locked_aspect",
    "locked_passive",
    "over_budget",
  ]);
});

test("teamSeasonValidity wants a commander when a banner is set", () => {
  const unlocked = { archetypes: ["the_emperor"], aspects: [], teamPassives: [], banner: "Rally", budget: 10 };
  const team = { characters: [{ id: "e", template_id: "the_emperor" }], banner: "Rally" };
  const v = R.teamSeasonValidity(team, unlocked, 3);
  assert.equal(v.valid, false);
  assert.ok(v.reasons.some((r) => r.kind === "banner_needs_commander"));
});

test("teamSeasonValidity is invalid without a season context", () => {
  assert.equal(R.teamSeasonValidity({ characters: [] }, null, 0).valid, false);
});

test("seasonNextActions surfaces the open beat and team-submission state", () => {
  // Open, unclaimed beat + no team submitted → two pending items.
  let items = R.seasonNextActions({ openBeatIndex: 2, openBeatKind: "character", openBeatClaimed: false, teamSubmitted: false });
  assert.equal(items.length, 2);
  assert.equal(items[0].done, false);
  assert.match(items[0].text, /Beat 3 \(New Character\) is open/);
  assert.equal(items[1].done, false);
  assert.match(items[1].text, /Submit a team/);

  // Claimed beat + a legal submitted team → both done.
  items = R.seasonNextActions({ openBeatIndex: 0, openBeatKind: "banner", openBeatClaimed: true, teamSubmitted: true, teamLegal: true });
  assert.equal(items[0].done, true);
  assert.match(items[0].text, /Beat 1 \(Banner\) claimed/);
  assert.equal(items[1].done, true);
  assert.match(items[1].text, /Team submitted/);

  // Submitted but no longer legal → flagged pending.
  items = R.seasonNextActions({ teamSubmitted: true, teamLegal: false });
  assert.equal(items.length, 1); // no open beat
  assert.equal(items[0].done, false);
  assert.match(items[0].text, /no longer season-legal/);
});
