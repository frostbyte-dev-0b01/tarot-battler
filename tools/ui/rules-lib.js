// Pure, DOM-free helpers shared by the UI and unit tests.
//
// This file is loaded as a plain <script> before app.js (so its top-level
// declarations are visible to app.js as globals) AND is require()-able from
// Node for `node --test` (via the module.exports guard at the bottom). Keep it
// free of any DOM or appState access so it stays testable in isolation.

const basicAttackActionName = "Basic Attack";

// Subjects a condition can read. Engine values are stable (serde); only the
// labels are player-facing.
const ruleSubjectOptions = [
  { value: "self", label: "Self" },
  {
    value: "companion",
    label: "Companion",
    hint: "Companions are the allies cardinally adjacent at battle start. The bond is fixed — it stays even if units move or a companion is elsewhere on the board.",
  },
  { value: "any_ally", label: "Any ally", hint: "True if any living ally (the whole team) matches." },
  { value: "lowest_ally", label: "Lowest-HP ally", hint: "Reads the living ally with the lowest current HP." },
  { value: "target", label: "Current target", hint: "The enemy this character is currently focusing." },
  { value: "any_enemy", label: "Any enemy", hint: "True if any living enemy matches." },
  { value: "lowest_enemy", label: "Lowest-HP enemy", hint: "Reads the living enemy with the lowest current HP." },
  { value: "world", label: "Game State" },
];
// Each value carries a `group` so the dropdown can be categorized.
const ruleValueTypeOptions = [
  { value: "hp", label: "HP %", group: "Vitals" },
  { value: "mp", label: "MP", group: "Vitals" },
  { value: "self_row", label: "Column", group: "Position" },
  { value: "focused_by_count", label: "Enemies Targeting", group: "Threat" },
  { value: "self_companion_count", label: "Companion Count", group: "Bonds" },
  { value: "target_companion_count", label: "Companion Count", group: "Bonds" },
  { value: "stat", label: "Stat", group: "Attributes" },
  { value: "status_stacks", label: "Status", group: "Effects" },
  { value: "condition_stacks", label: "Condition", group: "Effects" },
  { value: "use_count", label: "Use Count", group: "Cadence" },
  { value: "turns_since_use", label: "Turns Since Use", group: "Cadence" },
  { value: "tick_count", label: "Tick Count", group: "Battlefield" },
  { value: "ally_count", label: "Allies Alive", group: "Battlefield" },
  { value: "enemy_count", label: "Enemies Alive", group: "Battlefield" },
];
// Order within each subject also defines optgroup order in the dropdown.
// use_count / turns_since_use are actor-self ability cadence (the engine
// ignores their subject), so they live under both Self and Game State.
const ruleValueOptionsBySubject = {
  self: ["hp", "mp", "self_row", "focused_by_count", "stat", "status_stacks", "condition_stacks", "self_companion_count", "use_count", "turns_since_use"],
  companion: ["hp", "mp", "self_row", "focused_by_count", "stat", "status_stacks", "condition_stacks"],
  any_ally: ["hp", "mp", "self_row", "focused_by_count", "stat", "status_stacks", "condition_stacks"],
  lowest_ally: ["hp", "mp", "self_row", "focused_by_count", "stat", "status_stacks", "condition_stacks"],
  target: ["hp", "mp", "self_row", "stat", "status_stacks", "condition_stacks", "target_companion_count"],
  any_enemy: ["hp", "mp", "self_row", "stat", "status_stacks", "condition_stacks"],
  lowest_enemy: ["hp", "mp", "self_row", "stat", "status_stacks", "condition_stacks"],
  world: ["use_count", "turns_since_use", "tick_count", "ally_count", "enemy_count"],
};
const ruleOperatorOptions = [
  { value: "gte", label: "≥" },
  { value: "lte", label: "≤" },
  { value: "eq", label: "=" },
];
// Front/Middle/Back maps to the engine's depth index (position.row).
const columnOptions = [
  { value: 0, label: "Front" },
  { value: 1, label: "Middle" },
  { value: 2, label: "Back" },
];
const statFieldOptions = ["vit", "mgt", "mag", "arm", "res", "spd"];

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function isPlainObject(value) {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function clampValue(value, minValue, maxValue) {
  return Math.max(minValue, Math.min(maxValue, value));
}

function moveArrayItem(array, fromIndex, toIndex) {
  if (!Array.isArray(array) || toIndex < 0 || toIndex >= array.length) {
    return;
  }
  const [item] = array.splice(fromIndex, 1);
  array.splice(toIndex, 0, item);
}

// Wilson score interval for a binomial proportion (used by the Arena win-rate CI).
function wilsonInterval(wins, n) {
  if (n <= 0) {
    return [0, 0];
  }
  const z = 1.96;
  const z2 = z * z;
  const p = wins / n;
  const denom = 1 + z2 / n;
  const center = (p + z2 / (2 * n)) / denom;
  const margin = (z * Math.sqrt((p * (1 - p) + z2 / (4 * n)) / n)) / denom;
  return [Math.max(0, center - margin), Math.min(1, center + margin)];
}

function formatPct(value) {
  return `${Math.round(value * 100)}%`;
}

function createEmptyRule() {
  return {
    ability: basicAttackActionName,
    when: [],
  };
}

// A new condition starts as the canonical "always true" check (self HP >= 1).
// A living actor always satisfies it, so a freshly added condition is
// permissive until the player constrains it. Previews render it as "Always".
function createEmptyCondition() {
  return {
    subject: "self",
    value: "hp",
    op: "gte",
    threshold: 1,
  };
}

// The canonical always-true condition: self HP >= 1 (any living actor).
function isAlwaysCondition(condition) {
  if (!condition || (condition.subject ?? "self") !== "self") {
    return false;
  }
  const op = condition.op ?? condition.comparator ?? "gte";
  return getConditionValueType(condition) === "hp" && op === "gte" && Number(condition.threshold ?? 0) <= 1;
}

function getAllowedRuleValueOptions(subject) {
  const optionValues = ruleValueOptionsBySubject[subject] ?? ruleValueOptionsBySubject.self;
  return optionValues
    .map((value) => ruleValueTypeOptions.find((option) => option.value === value))
    .filter(Boolean);
}

// Render the value dropdown with <optgroup>s, preserving subject option order.
function renderRuleValueSelectOptions(allowedOptions, selectedValue) {
  // Safety net: always include the saved value even if it is not normally
  // offered for this subject, so the dropdown can never silently fall back to
  // a different option than the data actually holds.
  const options = [...allowedOptions];
  if (selectedValue && !options.some((option) => option.value === selectedValue)) {
    const known = ruleValueTypeOptions.find((option) => option.value === selectedValue);
    options.push(known ?? { value: selectedValue, label: selectedValue, group: "Other" });
  }
  const groupsInOrder = [];
  const byGroup = new Map();
  for (const option of options) {
    const group = option.group ?? "Other";
    if (!byGroup.has(group)) {
      byGroup.set(group, []);
      groupsInOrder.push(group);
    }
    byGroup.get(group).push(option);
  }
  return groupsInOrder
    .map((group) => {
      const optionsMarkup = byGroup
        .get(group)
        .map((option) => `<option value="${option.value}" ${selectedValue === option.value ? "selected" : ""}>${escapeHtml(option.label)}</option>`)
        .join("");
      return `<optgroup label="${escapeHtml(group)}">${optionsMarkup}</optgroup>`;
    })
    .join("");
}

function getDefaultRuleValueForSubject(subject) {
  const [firstOption] = getAllowedRuleValueOptions(subject);
  if (!firstOption) {
    return "hp";
  }
  return firstOption.value;
}

function normalizeConditionForSubject(condition) {
  const subject = condition.subject ?? "self";
  const allowedOptionValues = new Set(getAllowedRuleValueOptions(subject).map((option) => option.value));
  const currentValueType = getConditionValueType(condition);
  if (!allowedOptionValues.has(currentValueType)) {
    setConditionValueType(condition, getDefaultRuleValueForSubject(subject));
  }
}

function setConditionValueType(condition, valueType) {
  if (valueType === "stat") {
    condition.value = { stat: "vit" };
  } else if (valueType === "status_stacks") {
    condition.value = { status_stacks: "Empower:MGT" };
  } else if (valueType === "condition_stacks") {
    condition.value = { condition_stacks: "Stunned" };
  } else {
    condition.value = valueType;
  }
}

function getConditionValueType(condition) {
  const value = condition?.value;
  if (isPlainObject(value) && typeof value.stat === "string") {
    return "stat";
  }
  if (isPlainObject(value) && typeof value.has_status === "string") {
    return "status_stacks";
  }
  if (isPlainObject(value) && typeof value.status_stacks === "string") {
    return "status_stacks";
  }
  if (isPlainObject(value) && typeof value.has_condition === "string") {
    return "condition_stacks";
  }
  if (isPlainObject(value) && typeof value.condition_stacks === "string") {
    return "condition_stacks";
  }
  return String(value ?? "hp");
}

function formatConditionPreview(condition) {
  if (isAlwaysCondition(condition)) {
    return "Always";
  }
  const subject = condition.subject ?? "self";
  const subjectLabel = getRuleOptionLabel(ruleSubjectOptions, subject);
  const valueType = getConditionValueType(condition);
  const operatorLabel = getRuleOptionLabel(ruleOperatorOptions, condition.op ?? condition.comparator ?? "gte");
  const threshold = condition.threshold ?? 0;
  const prefix = subject === "self" || subject === "world" ? "" : `${subjectLabel} `;

  if (valueType === "stat") {
    return `${prefix}${String(condition.value?.stat ?? "vit").toUpperCase()} ${operatorLabel} ${threshold}`;
  }

  if (valueType === "status_stacks") {
    return `${prefix}${condition.value?.status_stacks ?? "Empower:MGT"} ${operatorLabel} ${threshold}`;
  }

  if (valueType === "condition_stacks") {
    return `${prefix}${condition.value?.condition_stacks ?? "Stunned"} ${operatorLabel} ${threshold}`;
  }

  if (valueType === "self_row") {
    const columnLabel = columnOptions.find((option) => option.value === Number(threshold))?.label ?? threshold;
    return `${prefix}Column ${operatorLabel} ${columnLabel}`;
  }

  if (valueType === "hp") {
    return `${prefix}HP ${operatorLabel} ${threshold}%`;
  }

  const contextualLabel = getContextualRuleValueLabel(subject, valueType);
  return `${prefix}${contextualLabel} ${operatorLabel} ${threshold}`;
}

function formatRulePreview(rule) {
  const abilityLabel = rule?.ability || "an ability";
  const conditions = Array.isArray(rule?.when) ? rule.when : [];
  // Always-true conditions add nothing under AND, so read them as "always".
  const meaningful = rule?.match_any === true
    ? conditions
    : conditions.filter((condition) => !isAlwaysCondition(condition));
  if (meaningful.length === 0) {
    return `Use ${abilityLabel} always`;
  }

  const joiner = rule?.match_any === true ? " or " : " and ";
  return `Use ${abilityLabel} if ${meaningful.map((condition) => formatConditionPreview(condition)).join(joiner)}`;
}

// Just the "why" clause of a rule (no "Use <ability>" prefix): "self HP ≤ 50%"
// or "always". Used for replay rule attribution.
function formatRuleConditionClause(rule) {
  const conditions = Array.isArray(rule?.when) ? rule.when : [];
  const meaningful = rule?.match_any === true
    ? conditions
    : conditions.filter((condition) => !isAlwaysCondition(condition));
  if (meaningful.length === 0) {
    return "always";
  }
  const joiner = rule?.match_any === true ? " or " : " and ";
  return meaningful.map((condition) => formatConditionPreview(condition)).join(joiner);
}

function getContextualRuleValueLabel(subject, valueType) {
  if (valueType === "self_row") {
    return "Column";
  }
  if (valueType === "self_companion_count" || valueType === "target_companion_count") {
    return "Companion Count";
  }
  return getRuleOptionLabel(ruleValueTypeOptions, valueType);
}

function getRuleOptionLabel(options, value) {
  return options.find((option) => option.value === value)?.label ?? String(value);
}

// ===== Season / draft schedule (pure presentation helpers) =====

// Player-facing label for a draft beat's kind (server sends snake_case tags).
const beatKindLabels = {
  banner: "Banner",
  item: "Item",
  character: "New Character",
  team_passive: "Team Passive",
  swap: "Swap",
};

function beatKindLabel(kind) {
  return beatKindLabels[kind] || String(kind || "");
}

// A short, legible clock line for a season, e.g. "Day 3 · Beat 2 of 8".
// `totalBeats` defaults to the full 8-beat season.
function seasonClockLine(season, totalBeats = 8) {
  if (!season || typeof season !== "object") return "";
  const day = Number(season.day || 0);
  const revealed = Math.max(0, Number(season.beats_revealed || 0));
  const current = Math.min(Math.max(revealed, 1), totalBeats);
  return `Day ${day + 1} · Beat ${current} of ${totalBeats}`;
}

// Describe a player's win/loss record from a list of stored match results.
// Pure: takes the player id and the results array, returns a {wins,losses,draws}.
function tallyRecord(playerId, results) {
  const tally = { wins: 0, losses: 0, draws: 0 };
  if (!Array.isArray(results)) return tally;
  for (const r of results) {
    const isA = r.player_a === playerId;
    const isB = r.player_b === playerId;
    if (!isA && !isB) continue;
    if (r.winner === "draw") tally.draws += 1;
    else if ((r.winner === "a" && isA) || (r.winner === "b" && isB)) tally.wins += 1;
    else tally.losses += 1;
  }
  return tally;
}

// Format a stat-bonus map (e.g. { mgt: 2, vit: -2 }) as "MGT +2, VIT -2".
// Keeps a stable stat order so the same item always reads the same way.
const STAT_ORDER = ["vit", "mgt", "mag", "arm", "res", "spd"];
function formatStatBonuses(bonuses) {
  if (!bonuses || typeof bonuses !== "object") return "";
  const keys = Object.keys(bonuses).sort(
    (a, b) => STAT_ORDER.indexOf(a) - STAT_ORDER.indexOf(b),
  );
  return keys
    .filter((k) => Number(bonuses[k]) !== 0)
    .map((k) => `${k.toUpperCase()} ${Number(bonuses[k]) > 0 ? "+" : ""}${bonuses[k]}`)
    .join(", ");
}

// Resolve a draft offer id into a player-facing { label, tooltip } using the
// loaded content catalogs. Pure: `catalogs` is plain data
// ({ archetypes, aspects, teamPassives, banners, passiveDescriptions }), so this
// is unit-testable and shared by the Season UI. Unknown ids fall back to the
// raw id with an empty tooltip.
function describeDraftOffer(kind, id, catalogs) {
  const c = catalogs || {};
  if (kind === "character" || kind === "swap") {
    const arch = (c.archetypes || {})[id];
    if (!arch) return { label: id, tooltip: "" };
    const cost = arch.cost != null ? `Cost ${arch.cost}` : "";
    const passive = Array.isArray(arch.passive_pool) ? arch.passive_pool[0] : null;
    const pdesc = passive ? (c.passiveDescriptions || {})[passive] : "";
    const passiveBit = passive ? `Passive: ${passive}${pdesc ? " — " + pdesc : ""}` : "";
    return {
      label: arch.display_name || id,
      tooltip: [cost, passiveBit].filter(Boolean).join(" · "),
    };
  }
  if (kind === "item") {
    const aspect = (c.aspects || {})[id];
    if (!aspect) return { label: id, tooltip: "" };
    const stats = formatStatBonuses(aspect.stat_bonuses);
    return {
      label: aspect.display_name || id,
      tooltip: [aspect.description || "", stats].filter(Boolean).join(" · "),
    };
  }
  if (kind === "team_passive") {
    const def = (c.teamPassives || {})[id];
    return { label: id, tooltip: (def && def.description) || "" };
  }
  if (kind === "banner") {
    const def = (c.banners || {})[id];
    const scope = def && def.scope ? ` (affects ${def.scope})` : "";
    return { label: id, tooltip: def ? `${def.description || ""}${scope}` : "" };
  }
  return { label: id, tooltip: "" };
}

// Node (test) export. In the browser `module` is undefined, so these stay
// plain top-level declarations shared with app.js as globals.
if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    basicAttackActionName,
    ruleSubjectOptions,
    ruleValueTypeOptions,
    ruleValueOptionsBySubject,
    ruleOperatorOptions,
    columnOptions,
    statFieldOptions,
    escapeHtml,
    isPlainObject,
    clampValue,
    moveArrayItem,
    wilsonInterval,
    formatPct,
    createEmptyRule,
    createEmptyCondition,
    isAlwaysCondition,
    getAllowedRuleValueOptions,
    renderRuleValueSelectOptions,
    getDefaultRuleValueForSubject,
    normalizeConditionForSubject,
    setConditionValueType,
    getConditionValueType,
    formatConditionPreview,
    formatRulePreview,
    formatRuleConditionClause,
    getContextualRuleValueLabel,
    getRuleOptionLabel,
    beatKindLabel,
    seasonClockLine,
    tallyRecord,
    formatStatBonuses,
    describeDraftOffer,
  };
}
