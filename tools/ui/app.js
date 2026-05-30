// Unified inline line-icon set (replaces emoji across the UI).
const UI_ICONS = {
  library: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="4" width="4" height="16" rx="1"/><rect x="10" y="4" width="4" height="16" rx="1"/><path d="m16.5 5 3.4 1 .1.4-3.2 13.4-3.4-1"/></svg>',
  export: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M12 4v10"/><path d="m8 10 4 4 4-4"/><path d="M5 18.5h14"/></svg>',
  edit: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M14.5 5.5 18.5 9.5 8 20H4v-4L14.5 5.5Z"/><path d="m13 7 4 4"/></svg>',
  trash: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M5 7h14"/><path d="M9 7V5h6v2"/><path d="M7 7l.7 12.5h8.6L17 7"/></svg>',
  add: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 6v12"/><path d="M6 12h12"/></svg>',
  passive: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3.5 19 6v5c0 4.4-3 7.6-7 9-4-1.4-7-4.6-7-9V6l7-2.5Z"/></svg>',
  active: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M11 3 5.5 13H11l-1 8 7.5-11H12l-1-7Z" fill="currentColor" stroke="none"/></svg>',
  aspect: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"><path d="m12 3.5 2.4 5.1 5.6.7-4.1 3.9 1.1 5.6L12 21.2l-5 2.6 1.1-5.6-4.1-3.9 5.6-.7L12 3.5Z" fill="currentColor" stroke="none"/></svg>',
};

function icon(name) {
  return `<span class="ui-icon" aria-hidden="true">${UI_ICONS[name] ?? ""}</span>`;
}

// Damage symbols: physical = sword (red), magical = spark/burst (blue). A hit's
// tier is shown as that many clustered glyphs (Light 1 … Ultimate 4).
const ATK_GLYPHS = {
  phys: '<svg viewBox="0 0 24 24" class="atk-glyph" aria-hidden="true"><path d="M19 4.5 18 9l-6.2 6.2-1.6 1.6-2-2 1.6-1.6L16 7l3-2.5Z" fill="currentColor"/><path d="m8.6 14.4 1 1L5.2 19.8l-1.4.2.2-1.4 4.6-4.2Z" fill="currentColor"/><path d="m6.4 17.6 1.4 1.4" stroke="currentColor" stroke-width="1.4" fill="none"/></svg>',
  mag: '<svg viewBox="0 0 24 24" class="atk-glyph" aria-hidden="true"><path d="M12 2.5 13.4 9 19 7.6 14.8 12 19 16.4 13.4 15 12 21.5 10.6 15 5 16.4 9.2 12 5 7.6 10.6 9 12 2.5Z" fill="currentColor"/></svg>',
};
const TIER_PIPS = { light: 1, medium: 2, heavy: 3, ultimate: 4 };
const TIER_NAMES = { 1: "Light", 2: "Medium", 3: "Heavy", 4: "Ultimate" };

function renderAtkChip(kind, tier) {
  const count = Math.max(1, Math.min(Number(tier) || 1, 4));
  const glyphs = ATK_GLYPHS[kind] ?? ATK_GLYPHS.phys;
  const cluster = glyphs.repeat(count);
  const label = `${TIER_NAMES[count] ?? "Light"} ${kind === "mag" ? "magical" : "physical"}`;
  return `<span class="atk-chip atk-${kind === "mag" ? "mag" : "phys"}" title="${escapeHtml(label)}" aria-label="${escapeHtml(label)}">${cluster}</span>`;
}

// Derive an ability's primary damage (kind + tier) from its primitives, so the
// symbols can never drift from the actual numbers. Returns null for abilities
// that deal no tiered damage.
function abilityPrimaryDamage(abilityName) {
  const def = appState.catalogs.abilityDefinitions?.[abilityName];
  if (!def || !Array.isArray(def.primitives)) {
    return null;
  }
  const hits = [];
  const visit = (list, depth) => {
    for (const prim of list) {
      if (!isPlainObject(prim)) continue;
      const kind = prim.kind;
      if (kind === "deal_physical_damage" || kind === "deal_physical_damage_current_target_and_companions") {
        hits.push({ kind: "phys", tier: TIER_PIPS[prim.power ?? prim.primary_power] ?? 1, depth });
      } else if (kind === "deal_magical_damage" || kind === "deal_magical_damage_current_target_and_companions") {
        hits.push({ kind: "mag", tier: TIER_PIPS[prim.power ?? prim.primary_power] ?? 1, depth });
      } else if (kind === "command_attack") {
        hits.push({ kind: "phys", tier: 1, depth });
      }
      for (const value of Object.values(prim)) {
        if (Array.isArray(value)) visit(value, depth + 1);
      }
    }
  };
  visit(def.primitives, 0);
  if (hits.length === 0) {
    return null;
  }
  // Prefer an unconditional (top-level) hit; otherwise the lowest tier is the
  // base (e.g. Condemn: Medium normally, Heavy only vs Omen).
  const topLevel = hits.filter((hit) => hit.depth === 0);
  const pool = topLevel.length ? topLevel : hits;
  return pool.reduce((best, hit) => (hit.tier < best.tier ? hit : best), pool[0]);
}

function renderAbilityDamageChip(abilityName) {
  const damage = abilityPrimaryDamage(abilityName);
  return damage ? renderAtkChip(damage.kind, damage.tier) : "";
}

// Render an ability description, turning {p:N}/{m:N} tokens into damage chips.
function formatAbilityDescriptionMarkup(description) {
  return escapeHtml(description ?? "").replace(/\{([pm]):(\d)\}/g, (_match, kindCode, tier) =>
    renderAtkChip(kindCode === "m" ? "mag" : "phys", Number(tier)),
  );
}

function loadoutTypeIcon(mode) {
  const key = mode === "passive" ? "passive" : mode === "aspect" ? "aspect" : "active";
  return `<span class="loadout-chip-icon loadout-chip-icon-${key}" aria-hidden="true">${UI_ICONS[key]}</span>`;
}

const tabButtons = document.querySelectorAll("[data-tab-target]");
const workspaces = document.querySelectorAll(".workspace");
const replayFileInput = document.querySelector("#replay-file-input");
const replayFileButton = document.querySelector("#replay-file-button");
const replayJsonInput = document.querySelector("#replay-json-input");
const replayDemoButton = document.querySelector("#replay-demo-button");
const replayRunButton = document.querySelector("#replay-run-button");
const sampleOpponentTeamPath = "./sample-data/teams/omen_engine.json";
const arenaFightButton = document.querySelector("#arena-fight-button");
const arenaResults = document.querySelector("#arena-results");
const arenaRecord = document.querySelector("#arena-record");
const gauntletManifestPath = "./sample-data/gauntlet.json";
const gauntletTeamsDir = "./sample-data/teams/";
const arenaReplayStore = new Map();
const replayStatsButton = document.querySelector("#replay-stats-button");
const statsOverlay = document.querySelector("#stats-overlay");
const statsCloseButton = document.querySelector("#stats-close-button");
const statsBody = document.querySelector("#stats-body");
const replayValidationOutput = document.querySelector("#replay-validation-output");
const latestReplayPaths = [
  "./sample-data/latest_replay.json",
  "./tools/ui/sample-data/latest_replay.json",
  "../sample-data/latest_replay.json",
];
const basicAttackActionName = "Basic Attack";
const archetypeCatalogPath = "../../battle_engine/src/data/archetypes.json";
const passiveCatalogPath = "../../battle_engine/src/data/passives.json";
const abilityCatalogPath = "../../battle_engine/src/data/abilities.json";
const aspectCatalogPath = "../../battle_engine/src/data/aspects.json";
const statusCatalogPath = "../../battle_engine/src/data/statuses.json";
const conditionCatalog = ["Stunned", "Marked", "Severed"];
const ruleStatusCatalog = ["Omen", "Restoration", "Ward", "Empower:MGT", "Empower:MAG", "Empower:ARM", "Empower:RES", "Weaken:MGT", "Weaken:MAG", "Weaken:ARM", "Weaken:RES"];
const TEAM_SLOT_POSITIONS = [
  { row: 0, col: 0 },
  { row: 0, col: 2 },
  { row: 1, col: 1 },
  { row: 2, col: 1 },
];
// Team-building point budget. Must match TEAM_BUDGET in battle_engine/src/loader.rs.
const TEAM_BUDGET = 14;
const ruleSubjectOptions = [
  { value: "self", label: "Self" },
  { value: "target", label: "Target" },
  { value: "companion", label: "Companion" },
  { value: "world", label: "Game State" },
];
const ruleValueTypeOptions = [
  { value: "hp", label: "HP" },
  { value: "mp", label: "MP" },
  { value: "self_row", label: "Row" },
  { value: "self_companion_count", label: "Companion Count" },
  { value: "target_companion_count", label: "Companion Count" },
  { value: "use_count", label: "Use Count" },
  { value: "turns_since_use", label: "Turns Since Use" },
  { value: "tick_count", label: "Tick Count" },
  { value: "ally_count", label: "Allies Alive" },
  { value: "enemy_count", label: "Enemies Alive" },
  { value: "stat", label: "Stat" },
  { value: "status_stacks", label: "Status" },
  { value: "condition_stacks", label: "Condition" },
];
const ruleValueOptionsBySubject = {
  self: ["hp", "mp", "self_row", "stat", "status_stacks", "condition_stacks", "self_companion_count"],
  target: ["hp", "mp", "self_row", "stat", "status_stacks", "condition_stacks", "target_companion_count"],
  companion: ["hp", "mp", "self_row", "stat", "status_stacks", "condition_stacks"],
  world: ["use_count", "turns_since_use", "tick_count", "ally_count", "enemy_count"],
};
const ruleOperatorOptions = [
  { value: "gte", label: ">=" },
  { value: "lte", label: "<=" },
];
const statFieldOptions = ["vit", "mgt", "mag", "arm", "res", "spd"];
const teamEditorConfig = {
  fileInput: document.querySelector("#team-file-input"),
  jsonInput: document.querySelector("#team-json-input"),
  loadButton: document.querySelector("#team-load-button"),
  copyButton: document.querySelector("#team-copy-button"),
  downloadButton: document.querySelector("#team-download-button"),
  validationOutput: document.querySelector("#team-validation-output"),
  editor: document.querySelector("#team-editor"),
};
const characterEditor = document.querySelector("#character-editor");
// Builder DOM event handlers are delegated across both the Team tab
// (#team-editor) and the Character tab (#character-editor) containers.
const builderRoots = () => [teamEditorConfig.editor, characterEditor].filter(Boolean);
const replaySidebar = document.querySelector("#replay-sidebar");
const replaySidebarCollapse = document.querySelector("#replay-sidebar-collapse");
const replaySidebarExpand = document.querySelector("#replay-sidebar-expand");
const replaySideButtons = document.querySelectorAll("[data-replay-side]");
const replayPreviousButton = document.querySelector("#replay-previous-button");
const replayPlayButton = document.querySelector("#replay-play-button");
const replayPauseButton = document.querySelector("#replay-pause-button");
const replayNextButton = document.querySelector("#replay-next-button");
const replayRestartButton = document.querySelector("#replay-restart-button");
const replayEventSlider = document.querySelector("#replay-event-slider");
const replayEventLabel = document.querySelector("#replay-event-label");
const replayTickDisplay = document.querySelector("#replay-tick-display");
const replaySpeedButtons = document.querySelectorAll(".speed-button");
const currentEventTick = document.querySelector("#current-event-tick");
const currentEventIndex = document.querySelector("#current-event-index");
const currentEventText = document.querySelector("#current-event-text");
const logModeButtons = document.querySelectorAll("[data-log-mode]");
const logFocusButton = document.querySelector("[data-log-focus]");
const timelineList = document.querySelector("#timeline-list");
const inspectorPanel = document.querySelector("#inspector-panel");
const battleBoard = document.querySelector("#battle-board");
const boardFx = document.querySelector("#board-fx");
const boardPopups = document.querySelector("#board-popups");
const appState = {
  replay: null,
  selectedEventIndex: -1,
  selectedCharacterId: null,
  playbackTimerId: null,
  playbackSpeed: 1,
  logMode: "story",
  logFocus: false,
  beats: [],
  beatsReplay: null,
  replaySidebarTab: "detail",
  replaySidebarCollapsed: false,
  teamConfig: null,
  characterLibrary: [],
  teamRoster: [],
  activeTeamName: null,
  arenaMode: "test",
  arenaAttackerName: null,
  arenaResultView: "winrate",
  lastRoundRobin: null,
  selectedTeamCharacterIndex: 0,
  teamDetailTab: "design",
  teamDesignRightPane: "loadout",
  teamBrowserMode: "active",
  teamBrowserSlotIndex: 0,
  expandedRuleIndex: null,
  catalogs: {
    archetypes: {},
    archetypeIds: [],
    passives: [],
    abilities: [],
    aspects: [],
    statuses: [],
    conditions: [...conditionCatalog],
    passiveDescriptions: {},
    abilityDefinitions: {},
    abilityDescriptions: {},
    aspectDescriptions: {},
    aspectDefinitions: {},
  },
};
const metadataFields = {
  seed: document.querySelector('[data-meta-field="seed"]'),
  winner: document.querySelector('[data-meta-field="winner"]'),
  tick_count: document.querySelector('[data-meta-field="tick_count"]'),
  team_a: document.querySelector('[data-meta-field="team_a"]'),
  team_b: document.querySelector('[data-meta-field="team_b"]'),
};

const demoTeam = {
  version: 2,
  name: "Imperial Phalanx",
  characters: [
      {
        id: "the_emperor",
        template_id: "the_emperor",
        display_name: "The Emperor",
        position: { row: 0, col: 0 },
        passive: "Imperial Formation",
        actives: ["Hold the Line", "Command", "Taunt"],
        aspect: "aspect_of_grace",
        rules: [
          {
            ability: "Hold the Line",
            when: [
              { subject: "self", value: "mp", op: "gte", threshold: 5 },
              { subject: "self", value: "use_count", op: "lte", threshold: 0 },
            ],
          },
          {
            ability: "Taunt",
            when: [
              { subject: "self", value: "mp", op: "gte", threshold: 4 },
              { subject: "self", value: "use_count", op: "lte", threshold: 0 },
            ],
          },
        ],
      },
      {
        id: "the_hierophant",
        template_id: "the_hierophant",
        display_name: "The Hierophant",
        position: { row: 0, col: 2 },
        passive: "Sanctuary",
        actives: ["Smite", "Consecrate", "Blessing"],
        aspect: null,
        rules: [
          {
            ability: "Blessing",
            when: [
              { subject: "self", value: "mp", op: "gte", threshold: 4 },
              { subject: "companion", value: "mp", op: "lte", threshold: 2 },
            ],
          },
          {
            ability: "Consecrate",
            when: [
              { subject: "self", value: "mp", op: "gte", threshold: 5 },
              { subject: "target", value: "target_companion_count", op: "gte", threshold: 1 },
            ],
          },
          {
            ability: "Smite",
            when: [],
          },
        ],
      },
      {
        id: "the_chariot",
        template_id: "the_chariot",
        display_name: "The Chariot",
        position: { row: 1, col: 1 },
        passive: "Pursuit",
        actives: ["Charge", "Withdraw", "Breakthrough"],
        aspect: null,
        rules: [
          {
            ability: "Breakthrough",
            when: [{ subject: "self", value: { status_stacks: "Empower:MGT" }, op: "gte", threshold: 3 }],
          },
          {
            ability: "Charge",
            when: [
              { subject: "self", value: "mp", op: "gte", threshold: 4 },
              { subject: "self", value: "self_row", op: "gte", threshold: 1 },
            ],
          },
          {
            ability: "Withdraw",
            when: [{ subject: "self", value: "hp", op: "lte", threshold: 16 }],
          },
        ],
      },
    ],
};

for (const button of tabButtons) {
  button.addEventListener("click", () => {
    setActiveWorkspace(button.dataset.tabTarget);
  });
}

// Keyboard: digit keys jump between workspaces (1 Team … 4 Replay).
const railShortcutTargets = {
  Digit1: "team-builder",
  Digit2: "character-builder",
  Digit3: "arena",
  Digit4: "replay-viewer",
};
window.addEventListener("keydown", (event) => {
  if (shouldIgnoreGlobalKeydown(event) || event.metaKey || event.ctrlKey || event.altKey) {
    return;
  }
  const target = railShortcutTargets[event.code];
  if (target) {
    event.preventDefault();
    setActiveWorkspace(target);
  }
});

function setActiveWorkspace(targetId) {
  for (const workspace of workspaces) {
    workspace.classList.toggle("is-active", workspace.id === targetId);
  }

  for (const tabButton of tabButtons) {
    tabButton.classList.toggle("is-active", tabButton.dataset.tabTarget === targetId);
  }

  if (targetId === "replay-viewer" && !appState.replay) {
    void loadLatestReplay();
  }
  if (targetId === "arena") {
    renderArena();
  }
}

// ===== Replay sidebar: Detail / Log segmented toggle + collapse =====
function setReplaySidebarTab(tab) {
  appState.replaySidebarTab = tab === "log" ? "log" : "detail";
  appState.replaySidebarCollapsed = false;
  renderReplaySidebar();
}

function renderReplaySidebar() {
  const tab = appState.replaySidebarTab;
  const collapsed = appState.replaySidebarCollapsed;
  replaySidebar?.classList.toggle("is-collapsed", collapsed);
  replaySidebarExpand && (replaySidebarExpand.hidden = !collapsed);
  for (const button of replaySideButtons) {
    button.classList.toggle("is-active", button.dataset.replaySide === tab);
  }
  for (const pane of document.querySelectorAll("[data-replay-pane]")) {
    pane.classList.toggle("is-active", pane.dataset.replayPane === tab);
  }
}

for (const button of replaySideButtons) {
  button.addEventListener("click", () => setReplaySidebarTab(button.dataset.replaySide));
}
replaySidebarCollapse?.addEventListener("click", () => {
  appState.replaySidebarCollapsed = true;
  renderReplaySidebar();
});
replaySidebarExpand?.addEventListener("click", () => {
  appState.replaySidebarCollapsed = false;
  renderReplaySidebar();
});
renderReplaySidebar();

function scrollSelectedTimelineEventIntoView() {
  window.requestAnimationFrame(() => {
    const selected = timelineList?.querySelector(".timeline-beat.is-active");
    selected?.scrollIntoView({ block: "nearest" });
  });
}

function setLogMode(mode) {
  appState.logMode = mode === "detailed" ? "detailed" : "story";
  for (const button of logModeButtons) {
    button.classList.toggle("is-active", button.dataset.logMode === appState.logMode);
  }
  renderTimeline();
}

for (const button of logModeButtons) {
  button.addEventListener("click", () => setLogMode(button.dataset.logMode));
}

logFocusButton?.addEventListener("click", () => {
  if (logFocusButton.disabled) {
    return;
  }
  appState.logFocus = !appState.logFocus;
  logFocusButton.classList.toggle("is-active", appState.logFocus);
  renderTimeline();
});

function loadReplayFromText(sourceText) {
  if (!sourceText) {
    renderReplayValidation({
      ok: false,
      errors: ["Replay JSON input is empty."],
    });
    resetMetadata();
    return false;
  }

  try {
    const parsedReplay = JSON.parse(sourceText);
    const validation = validateReplay(parsedReplay);
    renderReplayValidation(validation);
    if (validation.ok) {
      appState.replay = parsedReplay;
      appState.selectedEventIndex = -1;
      appState.selectedCharacterId = null;
      renderReplayMetadata(parsedReplay);
      renderCurrentReplay();
      renderPlaybackControls();
      return true;
    } else {
      appState.replay = null;
      appState.selectedEventIndex = -1;
      appState.selectedCharacterId = null;
      stopPlayback();
      resetMetadata();
      resetBoards();
      renderPlaybackControls();
      renderInspector(null);
      return false;
    }
  } catch (error) {
    renderReplayValidation({
      ok: false,
      errors: [`Could not parse replay JSON: ${error.message}`],
    });
    appState.replay = null;
    appState.selectedEventIndex = -1;
    appState.selectedCharacterId = null;
    stopPlayback();
    resetMetadata();
    resetBoards();
    renderPlaybackControls();
    renderInspector(null);
    return false;
  }
}

replayDemoButton.addEventListener("click", () => {
  setActiveWorkspace("replay-viewer");
  void loadLatestReplay();
});

replayRunButton?.addEventListener("click", () => {
  setActiveWorkspace("replay-viewer");
  void runBattleInBrowser();
});

async function runBattleInBrowser() {
  if (typeof window.tarotEngineReady === "undefined") {
    renderReplayValidation({
      ok: false,
      errors: ["Battle engine module is not present. Rebuild it with tools/ui/build-engine.sh."],
    });
    return;
  }

  const ready = await window.tarotEngineReady;
  if (!ready || typeof window.runBattleWasm !== "function") {
    renderReplayValidation({
      ok: false,
      errors: ["Battle engine failed to load. Rebuild it with tools/ui/build-engine.sh and reload."],
    });
    return;
  }

  if (!appState.teamConfig) {
    renderReplayValidation({
      ok: false,
      errors: ["Load or build a valid team before running a battle."],
    });
    return;
  }

  let opponentJson;
  try {
    const response = await fetch(sampleOpponentTeamPath, { cache: "no-store" });
    if (!response.ok) {
      throw new Error(`request failed with ${response.status}`);
    }
    opponentJson = await response.text();
  } catch (error) {
    renderReplayValidation({
      ok: false,
      errors: [`Could not load the sample opponent team: ${error.message}`],
    });
    return;
  }

  let resultJson;
  try {
    const teamAJson = JSON.stringify(appState.teamConfig);
    resultJson = window.runBattleWasm(teamAJson, opponentJson, 42);
  } catch (error) {
    renderReplayValidation({
      ok: false,
      errors: [`Battle engine error: ${error.message}`],
    });
    return;
  }

  let parsed;
  try {
    parsed = JSON.parse(resultJson);
  } catch (error) {
    renderReplayValidation({
      ok: false,
      errors: [`Battle engine returned invalid JSON: ${error.message}`],
    });
    return;
  }

  if (parsed && typeof parsed.error === "string") {
    renderReplayValidation({
      ok: false,
      errors: [`Battle could not run: ${parsed.error}`],
    });
    return;
  }

  replayJsonInput.value = resultJson;
  loadReplayFromText(resultJson.trim());
}

async function ensureBattleEngineReady() {
  if (typeof window.tarotEngineReady === "undefined") {
    return false;
  }
  const ready = await window.tarotEngineReady;
  return ready && typeof window.runBattleWasm === "function";
}

arenaFightButton?.addEventListener("click", () => {
  if (appState.arenaMode === "roundrobin") {
    void runRoundRobin();
  } else {
    void runArenaSimulation();
  }
});

for (const modeBtn of document.querySelectorAll("[data-arena-mode]")) {
  modeBtn.addEventListener("click", () => {
    appState.arenaMode = modeBtn.dataset.arenaMode === "roundrobin" ? "roundrobin" : "test";
    for (const btn of document.querySelectorAll("[data-arena-mode]")) {
      btn.classList.toggle("is-active", btn.dataset.arenaMode === appState.arenaMode);
    }
    const sub = document.querySelector("#arena-sub");
    if (sub) {
      sub.textContent = appState.arenaMode === "roundrobin"
        ? "Tick the teams to include; every selected team fights every other across many seeded battles. Results show a head-to-head matrix and ranked standings."
        : "Simulate a team from your roster against the others. Each matchup runs many battles across varied seeds and reports a win rate with a 95% confidence interval.";
    }
    arenaRecord.textContent = "";
    arenaResults.innerHTML = '<div class="board-empty-state">Pick teams and press Simulate.</div>';
    renderArena();
  });
}

document.querySelector("#arena-attacker")?.addEventListener("change", (event) => {
  const select = event.target.closest?.("#arena-attacker-select");
  if (select) {
    appState.arenaAttackerName = select.value;
    renderArenaFoes();
  }
});

document.querySelector("#arena")?.addEventListener("click", (event) => {
  const toggle = event.target.closest?.("[data-arena-view]");
  if (!toggle) {
    return;
  }
  appState.arenaResultView = toggle.dataset.arenaView === "wl" ? "wl" : "winrate";
  if (appState.lastRoundRobin) {
    renderRoundRobinResults(appState.lastRoundRobin.names, appState.lastRoundRobin.matrix, appState.lastRoundRobin.runs);
  }
});

const arenaFoesEl = document.querySelector("#arena-foes");
arenaFoesEl?.addEventListener("change", (event) => {
  const checkbox = event.target.closest?.("[data-arena-foe]");
  if (!checkbox) {
    return;
  }
  appState.arenaSelectedFoes = appState.arenaSelectedFoes ?? new Set();
  if (checkbox.checked) {
    appState.arenaSelectedFoes.add(checkbox.dataset.arenaFoe);
  } else {
    appState.arenaSelectedFoes.delete(checkbox.dataset.arenaFoe);
  }
});
arenaFoesEl?.addEventListener("click", (event) => {
  const toggle = event.target.closest?.("[data-arena-foe-all]");
  if (!toggle) {
    return;
  }
  appState.arenaSelectedFoes = new Set(
    toggle.dataset.arenaFoeAll === "1" ? appState.teamRoster.map(teamDisplayName) : [],
  );
  renderArenaFoes();
});

function renderArena() {
  renderArenaAttacker();
  renderArenaFoes();
}

function arenaAttackerTeam() {
  const roster = appState.teamRoster ?? [];
  return roster.find((team) => teamDisplayName(team) === appState.arenaAttackerName) ?? null;
}

function renderArenaAttacker() {
  const el = document.querySelector("#arena-attacker");
  if (!el) {
    return;
  }
  if (appState.arenaMode === "roundrobin") {
    el.hidden = true;
    el.innerHTML = "";
    return;
  }
  el.hidden = false;
  const roster = appState.teamRoster ?? [];
  if (roster.length === 0) {
    el.innerHTML = '<span class="arena-attacker-label">Team under test</span><span class="arena-attacker-sub">Save a team in the Team Builder first.</span>';
    return;
  }
  // Default the team under test to the active/edited team, else the first roster team.
  if (!appState.arenaAttackerName || !roster.some((team) => teamDisplayName(team) === appState.arenaAttackerName)) {
    appState.arenaAttackerName =
      appState.activeTeamName && roster.some((team) => teamDisplayName(team) === appState.activeTeamName)
        ? appState.activeTeamName
        : teamDisplayName(roster[0]);
  }
  const options = roster
    .map((team) => {
      const name = teamDisplayName(team);
      return `<option value="${escapeHtml(name)}" ${name === appState.arenaAttackerName ? "selected" : ""}>${escapeHtml(name)}</option>`;
    })
    .join("");
  el.innerHTML = `
    <label class="arena-attacker-pick">
      <span class="arena-attacker-label">Team under test</span>
      <select id="arena-attacker-select">${options}</select>
    </label>`;
}

function renderArenaFoes() {
  const el = document.querySelector("#arena-foes");
  if (!el) {
    return;
  }
  const roundRobin = appState.arenaMode === "roundrobin";
  const attackerName = roundRobin ? null : appState.arenaAttackerName;
  const foes = appState.teamRoster ?? [];
  if (foes.length === 0) {
    el.innerHTML = '<div class="board-empty-state">Your roster is empty. Save teams in the Team Builder to fight them here.</div>';
    return;
  }
  if (!appState.arenaSelectedFoes) {
    appState.arenaSelectedFoes = new Set(foes.map(teamDisplayName).filter((name) => name !== attackerName));
  }
  const items = foes
    .map((team) => {
      const name = teamDisplayName(team);
      const checked = appState.arenaSelectedFoes.has(name) ? "checked" : "";
      const marker = !roundRobin && name === attackerName ? '<span class="arena-foe-self">team under test</span>' : "";
      return `<label class="arena-foe"><input type="checkbox" data-arena-foe="${escapeHtml(name)}" ${checked}><span>${escapeHtml(name)}</span>${marker}</label>`;
    })
    .join("");
  el.innerHTML = `
    <div class="arena-foes-head">
      <span>${roundRobin ? "Teams to include" : "Opponents (from your roster)"}</span>
      <span class="arena-foes-tools">
        <button type="button" class="button-quiet" data-arena-foe-all="1">All</button>
        <button type="button" class="button-quiet" data-arena-foe-all="0">None</button>
      </span>
    </div>
    <div class="arena-foe-list">${items}</div>`;
}

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

async function runArenaSimulation() {
  if (!(await ensureBattleEngineReady())) {
    renderArenaMessage("Battle engine failed to load. Rebuild it with tools/ui/build-engine.sh and reload.");
    return;
  }
  const attacker = arenaAttackerTeam();
  if (!attacker) {
    renderArenaMessage("Choose a team to test (save one in the Team Builder first).");
    return;
  }

  const attackerName = teamDisplayName(attacker);
  const teamAJson = JSON.stringify(attacker);
  const selected = appState.teamRoster.filter(
    (team) => appState.arenaSelectedFoes?.has(teamDisplayName(team)) && teamDisplayName(team) !== attackerName,
  );
  if (selected.length === 0) {
    renderArenaMessage("Select at least one opponent from your roster (you can't fight your own team).");
    return;
  }

  const runs = clampValue(Math.round(Number(document.querySelector("#arena-run-count")?.value) || 1), 1, 2000);
  const progress = document.querySelector("#arena-progress");
  arenaFightButton.disabled = true;
  arenaRecord.textContent = "";
  arenaResults.innerHTML = "";
  arenaReplayStore.clear();
  if (progress) {
    progress.hidden = false;
  }

  const rows = [];
  const total = selected.length * runs;
  let done = 0;
  for (const foe of selected) {
    const foeName = teamDisplayName(foe);
    const teamBJson = JSON.stringify(foe);
    const row = { name: foeName, wins: 0, losses: 0, draws: 0, errors: 0, runs };
    for (let seed = 0; seed < runs; seed += 1) {
      try {
        const resultJson = window.runBattleWasm(teamAJson, teamBJson, seed);
        const parsed = JSON.parse(resultJson);
        if (parsed && typeof parsed.error === "string") {
          row.errors += 1;
        } else {
          if (parsed.winner === "team_a") {
            row.wins += 1;
          } else if (parsed.winner === "team_b") {
            row.losses += 1;
          } else {
            row.draws += 1;
          }
          if (seed === 0) {
            arenaReplayStore.set(foeName, resultJson);
          }
        }
      } catch {
        row.errors += 1;
      }
      done += 1;
      if (done % 20 === 0) {
        if (progress) {
          progress.textContent = `Simulating… ${done} / ${total} battles`;
        }
        await new Promise((resolve) => setTimeout(resolve, 0));
      }
    }
    rows.push(row);
  }

  if (progress) {
    progress.hidden = true;
  }
  arenaFightButton.disabled = false;
  renderArenaResults(rows, runs);
}

function renderArenaMessage(message) {
  arenaRecord.textContent = "";
  arenaResults.innerHTML = `<div class="board-empty-state">${escapeHtml(message)}</div>`;
}

function formatPct(value) {
  return `${Math.round(value * 100)}%`;
}

function renderArenaResults(rows, runs) {
  const totalWins = rows.reduce((sum, row) => sum + row.wins, 0);
  const totalLosses = rows.reduce((sum, row) => sum + row.losses, 0);
  const totalDraws = rows.reduce((sum, row) => sum + row.draws, 0);
  const decisive = totalWins + totalLosses;
  const [lo, hi] = wilsonInterval(totalWins, Math.max(decisive, 1));
  const overall = decisive ? totalWins / decisive : 0;
  arenaRecord.innerHTML = `<span class="arena-record-line">Overall <strong>${totalWins}W – ${totalLosses}L${totalDraws ? ` – ${totalDraws}D` : ""}</strong> · win rate <strong>${formatPct(overall)}</strong> <span class="arena-ci">[${formatPct(lo)}–${formatPct(hi)}]</span> over ${runs} battles each</span>`;

  const body = rows
    .map((row) => {
      const rowDecisive = row.wins + row.losses;
      const [rlo, rhi] = wilsonInterval(row.wins, Math.max(rowDecisive, 1));
      const pct = rowDecisive ? row.wins / rowDecisive : 0;
      const cls = pct >= 0.55 ? "arena-badge-win" : pct <= 0.45 ? "arena-badge-loss" : "arena-badge-draw";
      const ciText = rowDecisive ? `[${formatPct(rlo)}–${formatPct(rhi)}]` : "—";
      const wld = `${row.wins}–${row.losses}${row.draws ? `–${row.draws}D` : ""}${row.errors ? ` · ${row.errors} err` : ""}`;
      const action = arenaReplayStore.has(row.name)
        ? `<button type="button" class="arena-view-button" data-arena-replay="${escapeHtml(row.name)}">View replay</button>`
        : "";
      return `
        <tr>
          <td class="arena-opp-name">${escapeHtml(row.name)}</td>
          <td><span class="arena-badge ${cls}">${formatPct(pct)}</span></td>
          <td class="arena-detail arena-ci">${escapeHtml(ciText)}</td>
          <td class="arena-detail">${escapeHtml(wld)}</td>
          <td>${action}</td>
        </tr>`;
    })
    .join("");

  arenaResults.innerHTML = `
    <table class="arena-table">
      <thead>
        <tr><th>Opponent</th><th>Win %</th><th>95% CI</th><th>W–L</th><th></th></tr>
      </thead>
      <tbody>${body}</tbody>
    </table>`;

  for (const button of arenaResults.querySelectorAll("[data-arena-replay]")) {
    button.addEventListener("click", () => {
      const replay = arenaReplayStore.get(button.dataset.arenaReplay);
      if (!replay) {
        return;
      }
      replayJsonInput.value = replay;
      setActiveWorkspace("replay-viewer");
      loadReplayFromText(replay.trim());
    });
  }
}

async function runRoundRobin() {
  if (!(await ensureBattleEngineReady())) {
    renderArenaMessage("Battle engine failed to load. Rebuild it with tools/ui/build-engine.sh and reload.");
    return;
  }
  const teams = (appState.teamRoster ?? []).filter((team) => appState.arenaSelectedFoes?.has(teamDisplayName(team)));
  if (teams.length < 2) {
    renderArenaMessage("Tick at least two teams to run a round robin.");
    return;
  }

  const runs = clampValue(Math.round(Number(document.querySelector("#arena-run-count")?.value) || 1), 1, 2000);
  const names = teams.map(teamDisplayName);
  const json = teams.map((team) => JSON.stringify(team));
  const n = teams.length;
  const matrix = Array.from({ length: n }, () =>
    Array.from({ length: n }, () => ({ wins: 0, losses: 0, draws: 0 })),
  );

  const progress = document.querySelector("#arena-progress");
  arenaFightButton.disabled = true;
  arenaRecord.textContent = "";
  arenaResults.innerHTML = "";
  if (progress) {
    progress.hidden = false;
  }

  const total = ((n * (n - 1)) / 2) * runs;
  let done = 0;
  for (let i = 0; i < n; i += 1) {
    for (let j = i + 1; j < n; j += 1) {
      for (let seed = 0; seed < runs; seed += 1) {
        // Alternate which side is team A across seeds to neutralize first-mover bias.
        const iHome = seed % 2 === 0;
        const teamA = iHome ? json[i] : json[j];
        const teamB = iHome ? json[j] : json[i];
        try {
          const parsed = JSON.parse(window.runBattleWasm(teamA, teamB, seed));
          if (!parsed || typeof parsed.error === "string") {
            matrix[i][j].draws += 1;
            matrix[j][i].draws += 1;
          } else {
            const iWon = (parsed.winner === "team_a") === iHome && parsed.winner !== "draw";
            const jWon = (parsed.winner === "team_b") === iHome && parsed.winner !== "draw";
            if (parsed.winner === "draw") {
              matrix[i][j].draws += 1;
              matrix[j][i].draws += 1;
            } else if (iWon) {
              matrix[i][j].wins += 1;
              matrix[j][i].losses += 1;
            } else if (jWon) {
              matrix[i][j].losses += 1;
              matrix[j][i].wins += 1;
            }
          }
        } catch {
          matrix[i][j].draws += 1;
          matrix[j][i].draws += 1;
        }
        done += 1;
        if (done % 20 === 0) {
          if (progress) {
            progress.textContent = `Simulating… ${done} / ${total} battles`;
          }
          await new Promise((resolve) => setTimeout(resolve, 0));
        }
      }
    }
  }

  if (progress) {
    progress.hidden = true;
  }
  arenaFightButton.disabled = false;
  appState.lastRoundRobin = { names, matrix, runs };
  renderRoundRobinResults(names, matrix, runs);
}

function recordWinRate(rec) {
  const decisive = rec.wins + rec.losses;
  return decisive ? rec.wins / decisive : 0;
}

function formatRecordCell(rec, view) {
  if (rec.wins + rec.losses + rec.draws === 0) {
    return "—";
  }
  if (view === "wl") {
    return `${rec.wins}–${rec.losses}${rec.draws ? `–${rec.draws}d` : ""}`;
  }
  return formatPct(recordWinRate(rec));
}

function renderRoundRobinResults(names, matrix, runs) {
  const view = appState.arenaResultView === "wl" ? "wl" : "winrate";
  const n = names.length;

  // Standings: aggregate each team's record across all its matchups.
  const standings = names.map((name, i) => {
    const total = { wins: 0, losses: 0, draws: 0 };
    for (let j = 0; j < n; j += 1) {
      if (j === i) {
        continue;
      }
      total.wins += matrix[i][j].wins;
      total.losses += matrix[i][j].losses;
      total.draws += matrix[i][j].draws;
    }
    return { name, total, rate: recordWinRate(total) };
  });
  standings.sort((a, b) => b.rate - a.rate || b.total.wins - a.total.wins);

  const viewToggle = `
    <div class="arena-view-toggle" role="group" aria-label="Result format">
      <button type="button" class="arena-view-btn ${view === "winrate" ? "is-active" : ""}" data-arena-view="winrate">Win %</button>
      <button type="button" class="arena-view-btn ${view === "wl" ? "is-active" : ""}" data-arena-view="wl">W–L</button>
    </div>`;
  arenaRecord.innerHTML = `<span class="arena-record-line">Round robin · ${n} teams · ${runs} battles per pairing</span>${viewToggle}`;

  const standingsRows = standings
    .map((entry, rank) => {
      const value = view === "wl"
        ? `${entry.total.wins}–${entry.total.losses}${entry.total.draws ? `–${entry.total.draws}d` : ""}`
        : formatPct(entry.rate);
      return `
        <tr>
          <td class="arena-rank">${rank + 1}</td>
          <td class="arena-opp-name">${escapeHtml(entry.name)}</td>
          <td>${escapeHtml(value)}</td>
        </tr>`;
    })
    .join("");

  const headerCells = names.map((name) => `<th class="arena-matrix-col" title="${escapeHtml(name)}">${escapeHtml(abbreviateName(name))}</th>`).join("");
  const matrixRows = names
    .map((rowName, i) => {
      const cells = names
        .map((_, j) => {
          if (i === j) {
            return '<td class="arena-matrix-self">—</td>';
          }
          const rec = matrix[i][j];
          const rate = recordWinRate(rec);
          const cls = (rec.wins + rec.losses) === 0 ? "" : rate >= 0.55 ? "arena-cell-win" : rate <= 0.45 ? "arena-cell-loss" : "";
          return `<td class="arena-matrix-cell ${cls}">${escapeHtml(formatRecordCell(rec, view))}</td>`;
        })
        .join("");
      return `<tr><th class="arena-matrix-row" title="${escapeHtml(rowName)}">${escapeHtml(abbreviateName(rowName))}</th>${cells}</tr>`;
    })
    .join("");

  arenaResults.innerHTML = `
    <div class="arena-rr">
      <section class="arena-rr-block">
        <h4 class="arena-rr-title">Standings</h4>
        <table class="arena-table">
          <thead><tr><th>#</th><th>Team</th><th>${view === "wl" ? "Record" : "Win %"}</th></tr></thead>
          <tbody>${standingsRows}</tbody>
        </table>
      </section>
      <section class="arena-rr-block">
        <h4 class="arena-rr-title">Head-to-head (row vs column)</h4>
        <div class="arena-matrix-scroll">
          <table class="arena-matrix">
            <thead><tr><th></th>${headerCells}</tr></thead>
            <tbody>${matrixRows}</tbody>
          </table>
        </div>
      </section>
    </div>`;
}

function abbreviateName(name) {
  return name
    .split(/\s+/)
    .map((word) => word[0]?.toUpperCase() ?? "")
    .join("")
    .slice(0, 4) || name.slice(0, 4);
}

replayStatsButton?.addEventListener("click", () => {
  openBattleStats();
});

statsCloseButton?.addEventListener("click", () => {
  closeBattleStats();
});

statsOverlay?.addEventListener("click", (event) => {
  if (event.target === statsOverlay) {
    closeBattleStats();
  }
});

window.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && statsOverlay && !statsOverlay.hidden) {
    closeBattleStats();
  }
});

function openBattleStats() {
  if (!statsOverlay || !statsBody) {
    return;
  }
  if (!appState.replay) {
    statsBody.innerHTML = '<div class="board-empty-state">Load or run a battle first, then open stats.</div>';
  } else {
    statsBody.innerHTML = renderBattleStats(appState.replay);
  }
  statsOverlay.hidden = false;
}

function closeBattleStats() {
  if (statsOverlay) {
    statsOverlay.hidden = true;
  }
}

// Aggregate per-character battle stats from the replay event stream.
function computeBattleStats(replay) {
  const stats = new Map();
  const ensure = (id) => {
    if (!id) {
      return null;
    }
    if (!stats.has(id)) {
      stats.set(id, { dealt: 0, taken: 0, healed: 0, kills: 0, mp: 0 });
    }
    return stats.get(id);
  };

  for (const event of replay.events ?? []) {
    const amount = Number(event.amount) || 0;
    if (event.type === "damage" || (event.type === "status_tick" && event.kind === "damage")) {
      const source = ensure(event.source_id);
      const target = ensure(event.target_id);
      if (source) source.dealt += amount;
      if (target) target.taken += amount;
      if (source && event.target_hp_after === 0) {
        source.kills += 1;
      }
    } else if (event.type === "heal" || (event.type === "status_tick" && event.kind === "heal")) {
      const source = ensure(event.source_id);
      if (source) source.healed += amount;
    } else if (event.type === "ability_used") {
      const actor = ensure(event.actor_id);
      if (actor) actor.mp += Number(event.mp_cost) || 0;
    }
  }

  return stats;
}

function renderBattleStats(replay) {
  const stats = computeBattleStats(replay);
  const teamA = renderStatsTable(replay.teams?.team_a, stats);
  const teamB = renderStatsTable(replay.teams?.team_b, stats);
  return `
    <div class="stats-grid">
      <section class="stats-team">
        <h4 class="stats-team-name stats-team-a">${escapeHtml(replay.teams?.team_a?.name ?? "Team A")}</h4>
        ${teamA}
      </section>
      <section class="stats-team">
        <h4 class="stats-team-name stats-team-b">${escapeHtml(replay.teams?.team_b?.name ?? "Team B")}</h4>
        ${teamB}
      </section>
    </div>`;
}

function renderStatsTable(team, stats) {
  const characters = Array.isArray(team?.characters) ? team.characters : [];
  if (characters.length === 0) {
    return '<div class="board-empty-state">No characters.</div>';
  }
  const rows = characters
    .map((character) => {
      const s = stats.get(character.id) ?? { dealt: 0, taken: 0, healed: 0, kills: 0, mp: 0 };
      return `
        <tr>
          <td class="stats-name">${escapeHtml(character.display_name || character.id)}</td>
          <td>${s.dealt}</td>
          <td>${s.taken}</td>
          <td>${s.healed}</td>
          <td>${s.kills}</td>
          <td>${s.mp}</td>
        </tr>`;
    })
    .join("");
  return `
    <table class="stats-table">
      <thead>
        <tr><th>Character</th><th title="Damage dealt">Dealt</th><th title="Damage taken">Taken</th><th title="HP healed">Healed</th><th>Kills</th><th title="MP spent on abilities">MP</th></tr>
      </thead>
      <tbody>${rows}</tbody>
    </table>`;
}

replayFileButton?.addEventListener("click", () => {
  replayFileInput?.click();
});

replayFileInput.addEventListener("change", async (event) => {
  const [file] = event.target.files ?? [];
  if (!file) {
    return;
  }

  try {
    const content = await file.text();
    replayJsonInput.value = content;
    loadReplayFromText(content.trim());
  } catch (error) {
    renderReplayValidation({
      ok: false,
      errors: [`Could not read replay file: ${error.message}`],
    });
    appState.replay = null;
    appState.selectedEventIndex = -1;
    appState.selectedCharacterId = null;
    stopPlayback();
    resetMetadata();
    resetBoards();
    renderPlaybackControls();
    renderInspector(null);
  }
});

replayPreviousButton.addEventListener("click", () => {
  setSelectedEventIndex(appState.selectedEventIndex - 1);
});

replayNextButton.addEventListener("click", () => {
  setSelectedEventIndex(appState.selectedEventIndex + 1);
});

replayRestartButton.addEventListener("click", () => {
  setSelectedEventIndex(-1);
});

replayPlayButton.addEventListener("click", () => {
  if (appState.playbackTimerId !== null) {
    stopPlayback();
    renderPlaybackControls();
    return;
  }

  startPlayback();
});

replayPauseButton.addEventListener("click", () => {
  stopPlayback();
  renderPlaybackControls();
});

for (const speedButton of replaySpeedButtons) {
  speedButton.addEventListener("click", () => {
    const nextSpeed = Number(speedButton.dataset.speed);
    if (!Number.isFinite(nextSpeed) || nextSpeed <= 0) {
      return;
    }

    const wasPlaying = appState.playbackTimerId !== null;
    stopPlayback();
    appState.playbackSpeed = nextSpeed;
    renderPlaybackControls();
    if (wasPlaying) {
      startPlayback();
    }
  });
}

replayEventSlider?.addEventListener("input", (event) => {
  const sliderValue = Number(event.target.value);
  setSelectedEventIndex(sliderValue - 1);
});


window.addEventListener("keydown", (event) => {
  if (shouldIgnoreGlobalKeydown(event)) {
    return;
  }

  if (event.key === "ArrowLeft" || event.key === "a" || event.key === "A") {
    event.preventDefault();
    setSelectedEventIndex(appState.selectedEventIndex - 1);
  } else if (event.key === "ArrowRight" || event.key === "d" || event.key === "D") {
    event.preventDefault();
    setSelectedEventIndex(appState.selectedEventIndex + 1);
  }
});

teamEditorConfig.loadButton?.addEventListener("click", () => {
  loadTeamFromText(teamEditorConfig.jsonInput?.value.trim() ?? "");
});

teamEditorConfig.fileInput.addEventListener("change", async (event) => {
  const [file] = event.target.files ?? [];
  if (!file) {
    return;
  }

  try {
    const content = await file.text();
    if (teamEditorConfig.jsonInput) {
      teamEditorConfig.jsonInput.value = content;
    }
    loadTeamFromText(content);
  } catch (error) {
    renderTeamValidation({
      ok: false,
      errors: [`Could not read team file: ${error.message}`],
    });
    resetTeamSummary();
  }
});

teamEditorConfig.copyButton?.addEventListener("click", async () => {
  await copyTeamJson();
});

teamEditorConfig.downloadButton?.addEventListener("click", () => {
  downloadTeamJson();
});

for (const root of builderRoots()) {
  root.addEventListener("input", (event) => {
    handleTeamEditorInput(event);
  });

  root.addEventListener("change", (event) => {
    void handleTeamEditorChange(event);
  });

  root.addEventListener("click", (event) => {
    handleTeamEditorAction(event);
  });
}

// Pointer-based drag to reposition formation cells (also handles tap-to-select
// and tap-empty-to-move-selected, so the formation grid needs no click action).
// Double-tapping a placed unit jumps to the Character tab. We detect the double
// tap manually because tap-select re-renders the grid, which would void a native
// dblclick (the original node is gone before the second click lands).
let formationDrag = null;
let lastFormationTap = { index: -1, time: 0 };
teamEditorConfig.editor.addEventListener("pointerdown", (event) => {
  const cell = event.target.closest?.(".formation-cell");
  if (!cell || !appState.teamConfig) {
    return;
  }
  const row = Number(cell.dataset.row);
  const col = Number(cell.dataset.col);
  const index = findCharacterIndexAtPosition(appState.teamConfig, row, col);
  formationDrag = { row, col, index, fromFilled: index >= 0, moved: false, startX: event.clientX, startY: event.clientY };
  if (index >= 0) {
    cell.classList.add("is-dragging");
  }
  event.preventDefault();
});
window.addEventListener("pointermove", (event) => {
  if (formationDrag && (Math.abs(event.clientX - formationDrag.startX) > 5 || Math.abs(event.clientY - formationDrag.startY) > 5)) {
    formationDrag.moved = true;
  }
});
window.addEventListener("pointerup", (event) => {
  if (!formationDrag) {
    return;
  }
  const drag = formationDrag;
  formationDrag = null;
  for (const el of teamEditorConfig.editor.querySelectorAll(".formation-cell.is-dragging")) {
    el.classList.remove("is-dragging");
  }
  const team = appState.teamConfig;
  if (!team) {
    return;
  }
  const targetCell = document.elementFromPoint(event.clientX, event.clientY)?.closest?.(".formation-cell");
  const targetRow = targetCell ? Number(targetCell.dataset.row) : null;
  const targetCol = targetCell ? Number(targetCell.dataset.col) : null;
  const sameCell = targetRow === drag.row && targetCol === drag.col;

  if (drag.fromFilled && targetCell && !sameCell) {
    moveCharacterToPosition(team, drag.index, targetRow, targetCol); // handles swap
    syncTeamUI();
  } else if (drag.fromFilled && sameCell) {
    const now = Date.now();
    const isDoubleTap = lastFormationTap.index === drag.index && now - lastFormationTap.time < 400;
    lastFormationTap = { index: drag.index, time: now };
    appState.selectedTeamCharacterIndex = drag.index;
    appState.expandedRuleIndex = null;
    appState.teamDetailTab = "design";
    syncTeamUI();
    if (isDoubleTap) {
      setActiveWorkspace("character-builder");
    }
  } else if (!drag.fromFilled && sameCell) {
    moveCharacterToPosition(team, appState.selectedTeamCharacterIndex, drag.row, drag.col);
    syncTeamUI();
  }
});

resetMetadata();
resetBoards();
renderPlaybackControls();
renderTimeline();
renderInspector(null);
appState.teamConfig = structuredClone(demoTeam);
if (teamEditorConfig.jsonInput) {
  teamEditorConfig.jsonInput.value = JSON.stringify(appState.teamConfig, null, 2);
}
resetTeamSummary();
renderTeamEditor();
renderCharacterLibrary();
renderTeamValidation(validateTeamConfig(appState.teamConfig));
void loadEditorCatalogs();
void loadLatestReplay();

// ===== localStorage persistence: team roster + character library =====
const STORAGE_TEAMS = "tarot:teams";
const STORAGE_CHARACTERS = "tarot:characters";

function readStoredArray(key) {
  try {
    const parsed = JSON.parse(localStorage.getItem(key) ?? "null");
    return Array.isArray(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

function writeStored(key, value) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // localStorage may be unavailable (e.g. private mode); state stays in memory.
  }
}

function persistRoster() {
  writeStored(STORAGE_TEAMS, appState.teamRoster);
}

function persistCharacterLibrary() {
  writeStored(STORAGE_CHARACTERS, appState.characterLibrary);
}

function teamDisplayName(team) {
  return (team?.name || "").trim() || "Untitled Team";
}

function characterLabel(character) {
  return (character?.display_name || character?.id || "").trim() || "Character";
}

function upsertTeamInRoster(team) {
  const snapshot = structuredClone(team);
  const name = teamDisplayName(snapshot);
  snapshot.name = name;
  const index = appState.teamRoster.findIndex((entry) => teamDisplayName(entry) === name);
  if (index >= 0) {
    appState.teamRoster[index] = snapshot;
  } else {
    appState.teamRoster.push(snapshot);
  }
  appState.activeTeamName = name;
  persistRoster();
}

function upsertCharacterInLibrary(character) {
  const snapshot = structuredClone(character);
  const label = characterLabel(snapshot);
  const index = appState.characterLibrary.findIndex((entry) => characterLabel(entry) === label);
  if (index >= 0) {
    appState.characterLibrary[index] = snapshot;
  } else {
    appState.characterLibrary.push(snapshot);
  }
  persistCharacterLibrary();
}

async function fetchTeamJson(path) {
  const response = await fetch(path, { cache: "no-store" });
  if (!response.ok) {
    throw new Error(`failed ${response.status}`);
  }
  return response.json();
}

async function seedRosterFromBundled() {
  const teams = [];
  const seen = new Set();
  const add = (team) => {
    if (!team) {
      return;
    }
    const name = teamDisplayName(team);
    if (seen.has(name)) {
      return;
    }
    seen.add(name);
    team.name = name;
    teams.push(team);
  };
  add(structuredClone(demoTeam));
  try {
    const manifest = await fetchTeamJson(gauntletManifestPath);
    for (const entry of manifest) {
      try {
        add(await fetchTeamJson(`${gauntletTeamsDir}${entry.file}`));
      } catch {
        // skip a missing bundled team
      }
    }
  } catch {
    // no manifest available; roster still has the demo team
  }
  return teams;
}

async function initPersistence() {
  appState.characterLibrary = readStoredArray(STORAGE_CHARACTERS) ?? [];
  const storedTeams = readStoredArray(STORAGE_TEAMS);
  if (storedTeams && storedTeams.length > 0) {
    appState.teamRoster = storedTeams;
  } else {
    appState.teamRoster = await seedRosterFromBundled();
    persistRoster();
  }
  appState.activeTeamName = appState.teamConfig ? teamDisplayName(appState.teamConfig) : null;
  renderTeamEditor();
  renderArena();
}

void initPersistence();

// Wait for the WASM module loader (a deferred module script) to publish the
// engine-ready promise, then resolve it.
async function waitForEngineReady(timeoutMs = 8000) {
  const start = Date.now();
  while (typeof window.tarotEngineReady === "undefined") {
    if (Date.now() - start > timeoutMs) {
      return false;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  return await window.tarotEngineReady;
}

function applyCatalogs(archetypes, passives, abilities, aspects, statuses) {
  appState.catalogs.archetypes = archetypes;
  appState.catalogs.archetypeIds = Object.keys(archetypes).sort();
  appState.catalogs.passives = Object.keys(passives).sort();
  appState.catalogs.abilities = Object.keys(abilities).sort();
  appState.catalogs.aspects = Object.keys(aspects).sort();
  appState.catalogs.statuses = buildRuleStatusOptions(statuses);
  appState.catalogs.conditions = [...conditionCatalog];
  appState.catalogs.passiveDescriptions = Object.fromEntries(
    Object.entries(passives).map(([name, definition]) => [name, definition?.description ?? ""]),
  );
  appState.catalogs.abilityDefinitions = abilities;
  appState.catalogs.abilityDescriptions = Object.fromEntries(
    Object.entries(abilities).map(([name, definition]) => [name, definition?.description ?? ""]),
  );
  appState.catalogs.aspectDescriptions = Object.fromEntries(
    Object.entries(aspects).map(([name, definition]) => [name, definition?.description ?? ""]),
  );
  appState.catalogs.aspectDefinitions = aspects;
  renderTeamEditor();
}

function applyEmptyCatalogs() {
  appState.catalogs.archetypes = {};
  appState.catalogs.archetypeIds = [];
  appState.catalogs.passives = [];
  appState.catalogs.abilities = [];
  appState.catalogs.aspects = [];
  appState.catalogs.statuses = [...ruleStatusCatalog];
  appState.catalogs.conditions = [...conditionCatalog];
  appState.catalogs.passiveDescriptions = {};
  appState.catalogs.abilityDefinitions = {};
  appState.catalogs.abilityDescriptions = {};
  appState.catalogs.aspectDescriptions = {};
  appState.catalogs.aspectDefinitions = {};
}

async function loadEditorCatalogs() {
  // Preferred: read the catalogs the engine embeds, so the static site needs no
  // separate data fetch (works on GitHub Pages without publishing engine source).
  try {
    const ready = await waitForEngineReady();
    if (ready && typeof window.tarotCatalog === "function") {
      const get = (name) => {
        try {
          return JSON.parse(window.tarotCatalog(name) ?? "null") ?? {};
        } catch {
          return {};
        }
      };
      applyCatalogs(get("archetypes"), get("passives"), get("abilities"), get("aspects"), get("statuses"));
      return;
    }
  } catch {
    // fall through to the fetch fallback
  }

  // Fallback: fetch the engine data files (works when served from the repo root).
  try {
    const [archetypeResponse, passiveResponse, abilityResponse, aspectResponse, statusResponse] = await Promise.all([
      fetch(archetypeCatalogPath, { cache: "no-store" }),
      fetch(passiveCatalogPath, { cache: "no-store" }),
      fetch(abilityCatalogPath, { cache: "no-store" }),
      fetch(aspectCatalogPath, { cache: "no-store" }).catch(() => null),
      fetch(statusCatalogPath, { cache: "no-store" }).catch(() => null),
    ]);

    if (!archetypeResponse.ok || !passiveResponse.ok || !abilityResponse.ok) {
      throw new Error(
        `catalog request failed (${archetypeResponse.status}/${passiveResponse.status}/${abilityResponse.status})`,
      );
    }

    const [archetypes, passives, abilities, aspects, statuses] = await Promise.all([
      archetypeResponse.json(),
      passiveResponse.json(),
      abilityResponse.json(),
      aspectResponse?.ok ? aspectResponse.json() : Promise.resolve({}),
      statusResponse?.ok ? statusResponse.json() : Promise.resolve({}),
    ]);

    applyCatalogs(archetypes, passives, abilities, aspects, statuses);
  } catch {
    applyEmptyCatalogs();
  }
}

async function loadLatestReplay() {
  let lastError = null;
  try {
    for (const replayPath of latestReplayPaths) {
      try {
        const response = await fetch(replayPath, { cache: "no-store" });
        if (!response.ok) {
          throw new Error(`Request failed with ${response.status}`);
        }

        const content = await response.text();
        replayJsonInput.value = content;
        loadReplayFromText(content.trim());
        return;
      } catch (error) {
        lastError = `${replayPath}: ${error.message}`;
      }
    }
  } catch (error) {
    lastError = error.message;
  }

  {
    renderReplayValidation({
      ok: false,
      errors: [
        `Could not load latest replay. Tried ${latestReplayPaths.join(", ")}. Last error: ${lastError}. Run the engine to generate it, then try again.`,
      ],
    });
    appState.replay = null;
    appState.selectedEventIndex = -1;
    appState.selectedCharacterId = null;
    stopPlayback();
    resetMetadata();
    resetBoards();
    renderPlaybackControls();
    renderInspector(null);
  }
}

function validateReplay(candidate) {
  const errors = [];

  if (!isPlainObject(candidate)) {
    return {
      ok: false,
      errors: ["Replay must be a JSON object."],
    };
  }

  if (typeof candidate.version !== "number") {
    errors.push("`version` must be a number.");
  }

  if (typeof candidate.seed !== "number") {
    errors.push("`seed` must be a number.");
  }

  if (!["team_a", "team_b", "draw"].includes(candidate.winner)) {
    errors.push('`winner` must be `"team_a"`, `"team_b"`, or `"draw"`.');
  }

  if (typeof candidate.tick_count !== "number") {
    errors.push("`tick_count` must be a number.");
  }

  if (!isPlainObject(candidate.teams)) {
    errors.push("`teams` must be an object with `team_a` and `team_b`.");
  } else {
    validateReplayTeam(candidate.teams.team_a, "team_a", errors);
    validateReplayTeam(candidate.teams.team_b, "team_b", errors);
  }

  if (!Array.isArray(candidate.events)) {
    errors.push("`events` must be an array.");
  }

  if (!Array.isArray(candidate.snapshots)) {
    errors.push("`snapshots` must be an array.");
  } else {
    if (candidate.snapshots.length === 0) {
      errors.push("`snapshots` must contain at least one entry.");
    } else {
      const maxEventIndex = Array.isArray(candidate.events) ? candidate.events.length - 1 : -1;
      let previousEventIndex = null;

      candidate.snapshots.forEach((snapshot, index) => {
        validateReplaySnapshot(snapshot, index, errors);

        if (!isPlainObject(snapshot) || typeof snapshot.event_index !== "number") {
          return;
        }

        if (index === 0 && snapshot.event_index !== -1) {
          errors.push("`snapshots[0].event_index` must be -1.");
        }

        if (snapshot.event_index < -1 || snapshot.event_index > maxEventIndex) {
          errors.push(
            `snapshots[${index}].event_index must be between -1 and ${maxEventIndex}.`,
          );
        }

        if (previousEventIndex !== null && snapshot.event_index < previousEventIndex) {
          errors.push("`snapshots` event_index values must be nondecreasing.");
        }

        previousEventIndex = snapshot.event_index;
      });

      if (Array.isArray(candidate.events) && candidate.events.length > 0) {
        const lastSnapshot = candidate.snapshots[candidate.snapshots.length - 1];
        if (isPlainObject(lastSnapshot) && lastSnapshot.event_index !== maxEventIndex) {
          errors.push(
            `The last snapshot must cover the last replay event (expected event_index ${maxEventIndex}).`,
          );
        }
      }
    }
  }

  return {
    ok: errors.length === 0,
    errors,
  };
}

function validateReplayTeam(team, label, errors) {
  if (!isPlainObject(team)) {
    errors.push(`teams.${label} must be an object.`);
    return;
  }

  if (typeof team.name !== "string" || team.name.trim() === "") {
    errors.push(`teams.${label}.name must be a non-empty string.`);
  }

  if (!Array.isArray(team.characters)) {
    errors.push(`teams.${label}.characters must be an array.`);
  }
}

function validateReplaySnapshot(snapshot, index, errors) {
  if (!isPlainObject(snapshot)) {
    errors.push(`snapshots[${index}] must be an object.`);
    return;
  }

  if (typeof snapshot.event_index !== "number") {
    errors.push(`snapshots[${index}].event_index must be a number.`);
  }

  if (typeof snapshot.tick !== "number") {
    errors.push(`snapshots[${index}].tick must be a number.`);
  }

  if (!isPlainObject(snapshot.teams)) {
    errors.push(`snapshots[${index}].teams must be an object with team snapshots.`);
    return;
  }

  validateReplayTeam(snapshot.teams.team_a, `snapshots[${index}].teams.team_a`, errors);
  validateReplayTeam(snapshot.teams.team_b, `snapshots[${index}].teams.team_b`, errors);
}

function renderReplayMetadata(replay) {
  metadataFields.seed && (metadataFields.seed.textContent = String(replay.seed));
  metadataFields.winner && (metadataFields.winner.textContent = replay.winner);
  metadataFields.tick_count && (metadataFields.tick_count.textContent = String(replay.tick_count));
  metadataFields.team_a && (metadataFields.team_a.textContent = replay.teams.team_a.name);
  metadataFields.team_b && (metadataFields.team_b.textContent = replay.teams.team_b.name);
}

function resetMetadata() {
  for (const field of Object.values(metadataFields)) {
    if (field) {
      field.textContent = "-";
    }
  }
}

function renderCurrentReplay() {
  if (!appState.replay) {
    resetBoards();
    replayTickDisplay.textContent = "0";
    replayEventLabel.textContent = "0 / 0";
    currentEventTick.textContent = "Tick 0";
    currentEventIndex.textContent = "Step 0";
    currentEventText.textContent = "No replay loaded — run battles in the Training Arena or with “Run Battle”, or open a replay JSON.";
    renderTimeline();
    renderInspector(null);
    return;
  }

  const replayState = getReplaySnapshot(appState.replay, appState.selectedEventIndex);
  if (!replayState) {
    resetBoards();
    renderInspector(null);
    return;
  }
  renderBoards(replayState);
  replayTickDisplay.textContent = String(getCurrentTick());
  replayEventLabel.textContent = formatEventIndexLabel();
  renderCurrentEventSummary();
  renderTimeline();
  renderInspector(getSelectedCharacter(replayState));
}

function renderBoards(replayState) {
  renderBattleBoard(battleBoard, replayState);
}

function resetBoards() {
  renderBattleBoard(battleBoard, null);
}

function renderPlaybackControls() {
  const hasReplay = Boolean(appState.replay);
  const maxEventIndex = hasReplay ? getMaxEventIndex() : -1;
  const sliderValue = hasReplay ? appState.selectedEventIndex + 1 : 0;
  const sliderMax = hasReplay ? appState.replay.events.length : 0;

  replayEventSlider.disabled = !hasReplay;
  replayPreviousButton.disabled = !hasReplay || appState.selectedEventIndex < 0;
  replayNextButton.disabled = !hasReplay || appState.selectedEventIndex >= maxEventIndex;
  replayRestartButton.disabled = !hasReplay || appState.selectedEventIndex < 0;
  replayPlayButton.disabled = !hasReplay || (appState.playbackTimerId === null && appState.selectedEventIndex >= maxEventIndex);
  replayPauseButton.disabled = appState.playbackTimerId === null;
  replayPlayButton.textContent = appState.playbackTimerId !== null ? "pause" : "▶ play";
  replayEventSlider.max = String(sliderMax);
  replayEventSlider.value = String(sliderValue);
  replayFileButton.disabled = false;

  if (logFocusButton) {
    logFocusButton.disabled = !appState.selectedCharacterId;
    if (!appState.selectedCharacterId && appState.logFocus) {
      appState.logFocus = false;
      logFocusButton.classList.remove("is-active");
    }
  }

  for (const speedButton of replaySpeedButtons) {
    speedButton.classList.toggle("is-active", Number(speedButton.dataset.speed) === appState.playbackSpeed);
  }
}

function renderTimeline() {
  if (!appState.replay) {
    timelineList.innerHTML = '<div class="board-empty-state">Load a replay to view the timeline.</div>';
    return;
  }

  const beats = getBeats();
  const focusId = appState.logFocus ? appState.selectedCharacterId : null;
  const visible = focusId
    ? beats.filter((beat) => beatInvolves(beat, focusId))
    : beats;

  if (visible.length === 0) {
    timelineList.innerHTML = '<div class="board-empty-state">No beats involve the selected unit.</div>';
    return;
  }

  let previousTick = null;
  const markup = visible.map((beat) => {
    const tickHeader = beat.tick !== previousTick
      ? `<div class="timeline-tick-label">Tick ${beat.tick}</div>`
      : "";
    previousTick = beat.tick;
    return `${tickHeader}${renderBeat(beat, focusId)}`;
  }).join("");

  timelineList.innerHTML = markup;
  bindTimelineEvents();
  scrollSelectedTimelineEventIntoView();
}

function bindTimelineEvents() {
  for (const button of timelineList.querySelectorAll("[data-jump-index]")) {
    const index = Number(button.dataset.jumpIndex);
    button.addEventListener("click", () => setSelectedEventIndex(index));
    // Hover-to-scrub: preview the board/narration at this beat without
    // committing the selection; restore on leave.
    button.addEventListener("mouseenter", () => previewEventIndex(index));
    button.addEventListener("mouseleave", clearEventPreview);
  }
}

function previewEventIndex(index) {
  if (!appState.replay) {
    return;
  }
  const replayState = getReplaySnapshot(appState.replay, index);
  renderBattleBoard(battleBoard, replayState, index);
  const beat = beatAtEventIndex(index);
  currentEventText.innerHTML = beat ? narrateBeat(beat) : "";
}

function clearEventPreview() {
  if (!appState.replay) {
    return;
  }
  const replayState = getReplaySnapshot(appState.replay, appState.selectedEventIndex);
  renderBattleBoard(battleBoard, replayState, appState.selectedEventIndex);
  renderCurrentEventSummary();
}

// ===== Beat model: fold the raw event stream into one entry per turn =====
function getBeats() {
  if (!appState.replay) {
    return [];
  }
  if (appState.beatsReplay !== appState.replay) {
    appState.beats = buildBeats(appState.replay.events ?? []);
    appState.beatsReplay = appState.replay;
  }
  return appState.beats;
}

function eventParticipants(event) {
  return [event.actor_id, event.source_id, event.target_id, event.new_target_id, event.character_id]
    .filter((id) => typeof id === "string");
}

function buildBeats(events) {
  const beats = [];
  let current = null;
  events.forEach((event, index) => {
    const type = event.type;
    if (type === "battle_start" || type === "battle_end") {
      current = null;
      beats.push({ kind: "system", type, tick: event.tick, startIndex: index, endIndex: index, winner: event.winner, participants: new Set() });
      return;
    }
    if (type === "turn_start") {
      current = {
        kind: "turn",
        actorId: event.actor_id,
        tick: event.tick,
        startIndex: index,
        endIndex: index,
        action: null,
        preTicks: [],
        segments: [{ cause: "action", events: [] }],
        participants: new Set([event.actor_id]),
      };
      beats.push(current);
      return;
    }
    if (!current) {
      beats.push({ kind: "loose", type, tick: event.tick, startIndex: index, endIndex: index, events: [event], participants: new Set(eventParticipants(event)) });
      return;
    }
    current.endIndex = index;
    for (const id of eventParticipants(event)) {
      current.participants.add(id);
    }
    if (!current.action && (type === "basic_attack" || type === "ability_used" || type === "turn_skipped")) {
      current.action = event;
    } else if (type === "status_tick" && !current.action) {
      current.preTicks.push(event);
    } else if (type === "passive_triggered") {
      current.segments.push({ cause: "passive", passive: event.passive, actorId: event.actor_id, events: [] });
    } else {
      current.segments[current.segments.length - 1].events.push(event);
    }
  });
  return beats;
}

function beatInvolves(beat, characterId) {
  return Boolean(characterId) && beat.participants?.has(characterId);
}

function beatAtEventIndex(eventIndex) {
  const beats = getBeats();
  if (eventIndex < 0) {
    return beats[0] ?? null;
  }
  let match = null;
  for (const beat of beats) {
    if (beat.startIndex <= eventIndex) {
      match = beat;
    } else {
      break;
    }
  }
  return match;
}

function setSelectedEventIndex(nextEventIndex) {
  if (!appState.replay) {
    return;
  }

  const clampedIndex = clampValue(nextEventIndex, -1, getMaxEventIndex());
  appState.selectedEventIndex = clampedIndex;
  renderCurrentReplay();
  renderPlaybackControls();
}

function setSelectedCharacterId(characterId) {
  appState.selectedCharacterId = characterId ?? null;
  // Selecting a unit surfaces its detail in the (single) replay sidebar.
  if (characterId) {
    appState.replaySidebarTab = "detail";
    appState.replaySidebarCollapsed = false;
    renderReplaySidebar();
  }
  renderCurrentReplay();
  renderPlaybackControls();
}

function getMaxEventIndex() {
  return appState.replay ? appState.replay.events.length - 1 : -1;
}

function getReplaySnapshot(replay, selectedEventIndex) {
  if (!replay || !Array.isArray(replay.snapshots) || replay.snapshots.length === 0) {
    return null;
  }

  const desiredEventIndex = Math.max(-1, selectedEventIndex);
  let snapshot = replay.snapshots[0];
  for (const candidate of replay.snapshots) {
    if (!isPlainObject(candidate) || typeof candidate.event_index !== "number") {
      continue;
    }
    if (candidate.event_index <= desiredEventIndex) {
      snapshot = candidate;
    } else {
      break;
    }
  }
  if (!isPlainObject(snapshot) || !isPlainObject(snapshot.teams)) {
    return null;
  }

  return {
    event_index: snapshot.event_index,
    tick: snapshot.tick,
    teams: {
      team_a: normalizeReplaySnapshotTeam(snapshot.teams.team_a, "team_a"),
      team_b: normalizeReplaySnapshotTeam(snapshot.teams.team_b, "team_b"),
    },
  };
}

function normalizeReplaySnapshotTeam(team, teamKey) {
  return {
    name: team?.name ?? (teamKey === "team_a" ? "Team A" : "Team B"),
    characters: Array.isArray(team?.characters)
      ? team.characters.map((character) => ({
          ...character,
          team_key: teamKey,
        }))
      : [],
  };
}

function getCurrentTick() {
  if (!appState.replay) {
    return 0;
  }

  const snapshot = getReplaySnapshot(appState.replay, appState.selectedEventIndex);
  return typeof snapshot?.tick === "number" ? snapshot.tick : 0;
}

function formatEventIndexLabel() {
  if (!appState.replay) {
    return "0 / 0";
  }

  return appState.selectedEventIndex < 0 ? `0 / ${appState.replay.events.length}` : `${appState.selectedEventIndex + 1} / ${appState.replay.events.length}`;
}

function renderCurrentEventSummary() {
  if (!appState.replay || appState.selectedEventIndex < 0) {
    currentEventTick.textContent = "Tick 0";
    currentEventIndex.textContent = "Step 0";
    currentEventText.textContent = "Battle has not started yet.";
    return;
  }

  const event = appState.replay.events[appState.selectedEventIndex];
  currentEventTick.textContent = `Tick ${event.tick ?? 0}`;
  currentEventIndex.textContent = `Step ${appState.selectedEventIndex + 1}`;
  // Narrate the beat (the whole turn) rather than the isolated sub-event, so
  // the headline reads as one coherent moment.
  const beat = beatAtEventIndex(appState.selectedEventIndex);
  currentEventText.innerHTML = beat ? narrateBeat(beat) : formatTimelineMarkup(event);
}

function stopPlayback() {
  if (appState.playbackTimerId !== null) {
    window.clearInterval(appState.playbackTimerId);
    appState.playbackTimerId = null;
  }
}

function startPlayback() {
  if (!appState.replay || appState.playbackTimerId !== null) {
    return;
  }

  const intervalMs = Math.max(120, Math.round(900 / appState.playbackSpeed));
  appState.playbackTimerId = window.setInterval(() => {
    const maxEventIndex = getMaxEventIndex();
    if (appState.selectedEventIndex >= maxEventIndex) {
      stopPlayback();
      renderPlaybackControls();
      return;
    }

    setSelectedEventIndex(appState.selectedEventIndex + 1);
  }, intervalMs);

  renderPlaybackControls();
}

function renderBattleBoard(container, replayState, eventIndex = appState.selectedEventIndex) {
  if (!container) {
    return;
  }

  const currentEvent = appState.replay && eventIndex >= 0
    ? appState.replay.events[eventIndex]
    : null;
  const currentEventActorId = getEventActorId(currentEvent);
  const currentEventTargetId = getEventTargetId(currentEvent);

  if (!replayState) {
    container.innerHTML = '<div class="board-empty-state">No replay loaded. Run battles in the Training Arena, use “Run Battle”, or open a replay JSON.</div>';
    clearBoardFx();
    return;
  }

  const occupantMap = new Map();
  for (const character of replayState.teams.team_a.characters) {
    if (isReplayPosition(character.position)) {
      occupantMap.set(`${character.position.col}:${2 - character.position.row}`, character);
    }
  }
  for (const character of replayState.teams.team_b.characters) {
    if (isReplayPosition(character.position)) {
      occupantMap.set(`${character.position.col}:${4 + character.position.row}`, character);
    }
  }

  const cellsMarkup = Array.from({ length: 3 }, (_, colIndex) => {
    return Array.from({ length: 7 }, (_, depthIndex) => {
      const isGap = depthIndex === 3;
      const character = occupantMap.get(`${colIndex}:${depthIndex}`);
      const isSelected = character && character.id === appState.selectedCharacterId;
      const isSource = character && currentEventActorId === character.id;
      const isTarget = character && currentEventTargetId === character.id;
      return `
        <div class="arena-cell ${isGap ? "arena-cell-gap" : ""} ${character ? "arena-cell-occupied" : ""} ${
          character ? `arena-cell-${character.team_key}` : ""
        } ${character && !character.alive ? "arena-cell-defeated" : ""} ${isSelected ? "arena-cell-selected" : ""} ${
          isSource ? "arena-cell-source" : ""
        } ${isTarget ? "arena-cell-target" : ""}">
          ${character ? renderUnitCard(character) : ""}
        </div>
      `;
    }).join("");
  }).join("");

  container.innerHTML = cellsMarkup;
  bindBoardSelection(container);
  renderBoardFx(container, currentEvent, currentEventActorId, currentEventTargetId);
}

function clearBoardFx() {
  if (boardFx) boardFx.innerHTML = "";
  if (boardPopups) boardPopups.innerHTML = "";
}

function boardEffectKind(event) {
  if (!event) return null;
  switch (event.type) {
    case "damage":
    case "basic_attack":
      return "harm";
    case "heal":
    case "healing":
    case "mp_restore":
      return "help";
    case "status_applied":
    case "condition_applied":
    case "status_removed":
      return "status";
    case "status_tick":
      return event.kind === "heal" ? "help" : "harm";
    case "retargeted":
      return "control";
    default:
      return null;
  }
}

// Draws the action vector (source → target) and a floating ±N popup so the
// board narrates the current step on its own.
function renderBoardFx(container, event, actorId, targetId) {
  clearBoardFx();
  if (!boardFx || !boardPopups || !event) {
    return;
  }
  const kind = boardEffectKind(event);
  const sourceId = event.type === "retargeted" ? event.actor_id : actorId;
  const destId = event.type === "retargeted" ? event.new_target_id : targetId;

  const origin = boardFx.getBoundingClientRect();
  if (origin.width === 0 || origin.height === 0) {
    return;
  }
  const cellOf = (id) =>
    id
      ? [...container.querySelectorAll("[data-character-id]")]
          .find((el) => el.dataset.characterId === id)
          ?.closest(".arena-cell") ?? null
      : null;
  const centerOf = (cell) => {
    const rect = cell.getBoundingClientRect();
    return { x: rect.left - origin.left + rect.width / 2, y: rect.top - origin.top + rect.height / 2 };
  };

  boardFx.setAttribute("viewBox", `0 0 ${origin.width} ${origin.height}`);
  boardFx.setAttribute("preserveAspectRatio", "none");

  const sourceCell = cellOf(sourceId);
  const destCell = cellOf(destId);
  if (kind && sourceCell && destCell && sourceCell !== destCell) {
    const a = centerOf(sourceCell);
    const b = centerOf(destCell);
    const dashed = event.type === "retargeted" ? "fx-line-dashed" : "";
    boardFx.innerHTML = `
      <line x1="${a.x}" y1="${a.y}" x2="${b.x}" y2="${b.y}" class="fx-line fx-line-${kind} ${dashed}" />
      <circle cx="${b.x}" cy="${b.y}" r="4.5" class="fx-dot fx-dot-${kind}" />`;
  }

  const hasAmount = ["damage", "heal", "healing", "status_tick"].includes(event.type);
  if (hasAmount && destCell && event.amount != null) {
    const b = centerOf(destCell);
    const help = kind === "help";
    const popup = document.createElement("div");
    popup.className = `board-popup board-popup-${help ? "help" : "harm"}`;
    popup.style.left = `${b.x}px`;
    popup.style.top = `${b.y}px`;
    popup.textContent = `${help ? "+" : "−"}${event.amount}`;
    boardPopups.appendChild(popup);
  }
}

function isReplayPosition(position) {
  return isPlainObject(position)
    && Number.isInteger(position.row)
    && Number.isInteger(position.col)
    && position.row >= 0
    && position.row < 3
    && position.col >= 0
    && position.col < 3;
}

function renderUnitCard(character) {
  const hpValue = Number(character.current_hp) || 0;
  const mpValue = Number(character.current_mp) || 0;
  const portraitGlyph = getCharacterInitials(character);
  const defeatedMarker = character.alive === false
    ? '<span class="unit-card-defeated-marker" aria-hidden="true">✕</span>'
    : "";

  return `
    <button class="grid-cell-button" type="button" data-character-id="${escapeHtml(character.id)}">
      <article class="unit-card unit-card-compact">
        <div class="unit-card-top">
          <div class="unit-card-portrait">${escapeHtml(portraitGlyph)}</div>
          <h5 class="unit-card-name">${escapeHtml(character.display_name || character.id || "Unknown")}</h5>
          ${defeatedMarker}
        </div>
        <div class="unit-card-bars unit-card-bars-compact">
          ${renderCompactBar("HP", hpValue, character.max_hp, "hp")}
          <div class="compact-bar-row compact-mana-row">
            <span class="compact-bar-label">MP</span>
            ${renderManaPips(mpValue, character.max_mp)}
          </div>
        </div>
        ${renderUnitCardChips(character)}
      </article>
    </button>
  `;
}

function statusCardChip(name, stacks, polarity) {
  const arrow = polarity === "buff" ? "▲" : polarity === "debuff" ? "▼" : "•";
  const label = `${name} x${stacks}`;
  return `<span class="unit-card-chip unit-card-chip-${polarity}" title="${escapeHtml(label)}"><span class="unit-card-chip-arrow">${arrow}</span>${escapeHtml(name)}<span class="unit-card-chip-stacks">${escapeHtml(stacks)}</span></span>`;
}

// Two dedicated rows: beneficial statuses on top, debuffs + conditions below.
function renderUnitCardChips(character) {
  const buffs = [];
  const debuffs = [];
  for (const { name, stacks } of normalizeStatusEntries(character.statuses)) {
    const polarity = statusPolarity(name);
    (polarity === "buff" ? buffs : debuffs).push(statusCardChip(name, stacks, polarity === "buff" ? "buff" : "debuff"));
  }
  for (const { name, stacks } of normalizeConditionEntries(character.conditions)) {
    debuffs.push(statusCardChip(name, stacks, "neutral"));
  }
  if (buffs.length === 0 && debuffs.length === 0) {
    return "";
  }
  const buffRow = buffs.length ? `<div class="unit-card-chip-row unit-card-chip-row-buff">${buffs.join("")}</div>` : "";
  const debuffRow = debuffs.length ? `<div class="unit-card-chip-row unit-card-chip-row-debuff">${debuffs.join("")}</div>` : "";
  return `<div class="unit-card-chips">${buffRow}${debuffRow}</div>`;
}

function renderCompactBar(label, currentValue, maxValue, type) {
  const safeMax = Math.max(maxValue, 1);
  const percent = Math.max(0, Math.min(100, (currentValue / safeMax) * 100));
  return `
    <div class="compact-bar-row">
      <span class="compact-bar-label">${label}</span>
      <div class="compact-bar-track">
        <span class="compact-bar-fill compact-bar-fill-${type}" style="width: ${percent}%;"></span>
      </div>
      <span class="compact-bar-value">${escapeHtml(currentValue)}/${escapeHtml(Math.max(maxValue, 0))}</span>
    </div>
  `;
}

// Mana is rendered as discrete pips (●●●○○), not a bar — it caps at MAX_MP (5).
function renderManaPips(current, max) {
  const total = Math.max(0, Number(max) || 0);
  const filled = Math.max(0, Math.min(Number(current) || 0, total));
  const pips = Array.from({ length: total }, (_, index) =>
    `<span class="mana-pip ${index < filled ? "is-filled" : ""}"></span>`,
  ).join("");
  return `<div class="mana-pips" title="${filled} / ${total} MP" aria-label="${filled} of ${total} mana">${pips}</div>`;
}

function renderBar(label, currentValue, maxValue, type) {
  const safeMax = Math.max(maxValue, 1);
  const percent = Math.max(0, Math.min(100, (currentValue / safeMax) * 100));

  return `
    <div class="bar-row">
      <div class="bar-label">
        <span>${label}</span>
        <strong>${currentValue}/${maxValue}</strong>
      </div>
      <div class="bar-track">
        <span class="bar-fill bar-fill-${type}" style="width: ${percent}%;"></span>
      </div>
    </div>
  `;
}

function bindBoardSelection(container) {
  const characterButtons = container.querySelectorAll("[data-character-id]");
  for (const button of characterButtons) {
    button.addEventListener("click", () => {
      setSelectedCharacterId(button.dataset.characterId);
    });
  }
}

function renderReplayValidation(result) {
  if (!replayValidationOutput) {
    return;
  }

  replayValidationOutput.className = "message-panel";

  if (result.ok) {
    replayValidationOutput.classList.add("message-panel-success");
    replayValidationOutput.textContent = "Replay JSON is valid at the top level.";
    return;
  }

  if (result.errors.length === 0) {
    replayValidationOutput.classList.add("message-panel-idle");
    replayValidationOutput.textContent = "Load a replay JSON file or paste JSON to validate it.";
    return;
  }

  replayValidationOutput.classList.add("message-panel-error");
  replayValidationOutput.textContent = result.errors.map((error) => `- ${error}`).join("\n");
}

function loadTeamFromText(sourceText) {
  if (!sourceText) {
    renderTeamValidation({
      ok: false,
      errors: ["Team JSON input is empty."],
    });
    appState.teamConfig = null;
    resetTeamSummary();
    return;
  }

  try {
    const parsedTeam = JSON.parse(sourceText);
    const validation = validateTeamConfig(parsedTeam);
    renderTeamValidation(validation);

    if (validation.ok) {
      appState.teamConfig = parsedTeam;
      appState.selectedTeamCharacterIndex = 0;
      appState.teamBrowserMode = "active";
      appState.teamBrowserSlotIndex = 0;
      appState.expandedRuleIndex = null;
      syncTeamUI();
    } else {
      appState.teamConfig = null;
      resetTeamSummary();
      renderTeamEditor();
    }
  } catch (error) {
    renderTeamValidation({
      ok: false,
      errors: [`Could not parse team JSON: ${error.message}`],
    });
    appState.teamConfig = null;
    resetTeamSummary();
    renderTeamEditor();
  }
}

function validateTeamConfig(candidate) {
  const errors = [];

  if (!isPlainObject(candidate)) {
    return {
      ok: false,
      errors: ["Team must be a JSON object."],
    };
  }

  if (candidate.version !== 2) {
    errors.push("`version` must be 2.");
  }

  if (typeof candidate.name !== "string" || candidate.name.trim() === "") {
    errors.push("`name` must be a non-empty string.");
  }

  if (!Array.isArray(candidate.characters) || candidate.characters.length < 1) {
    errors.push("`characters` must be an array with at least 1 character.");
  } else if (candidate.characters.length > TEAM_SLOT_POSITIONS.length) {
    errors.push(`\`characters\` must contain at most ${TEAM_SLOT_POSITIONS.length} characters.`);
  } else {
    validateTeamCharacters(candidate.characters, errors);
  }

  return {
    ok: errors.length === 0,
    errors,
  };
}

function validateTeamCharacters(characters, errors) {
  const seenIds = new Set();
  const seenPositions = new Set();
  const seenTemplates = new Set();
  const seenAspects = new Set();

  characters.forEach((character, index) => {
    const prefix = `characters[${index}]`;
    const characterErrors = validateCharacterConfig(character, prefix);
    for (const error of characterErrors) {
      errors.push(error);
    }

    if (typeof character?.id === "string" && character.id.trim() !== "") {
      if (seenIds.has(character.id)) {
        errors.push(`${prefix}.id must be unique within the team.`);
      } else {
        seenIds.add(character.id);
      }
    }

    if (typeof character?.template_id === "string" && character.template_id.trim() !== "") {
      if (seenTemplates.has(character.template_id)) {
        errors.push(`${prefix}.template_id repeats an archetype; one copy of each archetype is allowed.`);
      } else {
        seenTemplates.add(character.template_id);
      }
    }

    if (typeof character?.aspect === "string" && character.aspect.trim() !== "") {
      if (seenAspects.has(character.aspect)) {
        errors.push(`${prefix}.aspect repeats; one copy of each aspect is allowed.`);
      } else {
        seenAspects.add(character.aspect);
      }
    }

    const row = character?.position?.row;
    const col = character?.position?.col;
    if (Number.isInteger(row) && Number.isInteger(col)) {
      const positionKey = `${row}:${col}`;
      if (seenPositions.has(positionKey)) {
        errors.push(`${prefix}.position must be unique within the team.`);
      } else {
        seenPositions.add(positionKey);
      }
    }
  });

  const spend = computeTeamCost(characters);
  if (spend > TEAM_BUDGET) {
    errors.push(`Team costs ${spend} points, over the ${TEAM_BUDGET}-point budget.`);
  }
}

// Sum of archetype costs plus aspect costs for a team's characters.
function computeTeamCost(characters) {
  if (!Array.isArray(characters)) {
    return 0;
  }
  return characters.reduce((total, character) => {
    const archetypeCost = Number(getArchetypeDefinition(character?.template_id)?.cost ?? 0);
    const aspectCost = character?.aspect
      ? Number(getAspectDefinition(character.aspect)?.cost ?? 0)
      : 0;
    return total + archetypeCost + aspectCost;
  }, 0);
}

function validateCharacterConfig(candidate, prefix = "character") {
  const errors = [];

  if (!isPlainObject(candidate)) {
    return [`${prefix} must be an object.`];
  }

  if (typeof candidate.id !== "string" || candidate.id.trim() === "") {
    errors.push(`${prefix}.id must be a non-empty string.`);
  }

  if (typeof candidate.template_id !== "string" || candidate.template_id.trim() === "") {
    errors.push(`${prefix}.template_id must be a non-empty string.`);
  }

  const archetype = getArchetypeDefinition(candidate.template_id);
  if (candidate.template_id && !archetype) {
    errors.push(`${prefix}.template_id must reference a known archetype.`);
  }

  if (!isPlainObject(candidate.position)) {
    errors.push(`${prefix}.position must be an object.`);
  } else {
    const { row, col } = candidate.position;
    if (!Number.isInteger(row) || row < 0 || row > 2) {
      errors.push(`${prefix}.position.row must be an integer from 0 to 2.`);
    }
    if (!Number.isInteger(col) || col < 0 || col > 2) {
      errors.push(`${prefix}.position.col must be an integer from 0 to 2.`);
    }
  }

  if (typeof candidate.passive !== "string") {
    errors.push(`${prefix}.passive must be a string.`);
  } else if (archetype && candidate.passive && !archetype.passive_pool?.includes(candidate.passive)) {
    errors.push(`${prefix}.passive must come from the selected archetype's passive pool.`);
  }

  if (!Array.isArray(candidate.actives)) {
    errors.push(`${prefix}.actives must be an array.`);
  } else if (archetype) {
    for (const [index, ability] of candidate.actives.entries()) {
      if (typeof ability !== "string") {
        errors.push(`${prefix}.actives[${index}] must be a string.`);
      } else if (ability && !archetype.active_pool?.includes(ability)) {
        errors.push(`${prefix}.actives[${index}] must come from the selected archetype's active pool.`);
      }
    }
  }

  if (candidate.aspect != null && typeof candidate.aspect !== "string") {
    errors.push(`${prefix}.aspect must be a string or null.`);
  } else if (typeof candidate.aspect === "string" && candidate.aspect && !appState.catalogs.aspects.includes(candidate.aspect)) {
    errors.push(`${prefix}.aspect must reference a known aspect.`);
  }

  if (!Array.isArray(candidate.rules)) {
    errors.push(`${prefix}.rules must be an array.`);
  } else if (candidate.rules.length > 5) {
    errors.push(`${prefix}.rules must contain at most 5 rules.`);
  }

  return errors;
}

function renderTeamSummary(teamConfig) {
  void teamConfig;
}

function resetTeamSummary() {
  return;
}

function syncTeamUI() {
  const teamConfig = appState.teamConfig;
  const preservedFocus = captureTeamEditorFocus();
  if (!teamConfig) {
    resetTeamSummary();
    appState.selectedTeamCharacterIndex = 0;
    appState.expandedRuleIndex = null;
    renderTeamEditor();
    restoreTeamEditorFocus(preservedFocus);
    return;
  }
  appState.selectedTeamCharacterIndex = clampValue(
    appState.selectedTeamCharacterIndex,
    0,
    Math.max((teamConfig.characters?.length ?? 1) - 1, 0),
  );
  if ((teamConfig.characters?.[appState.selectedTeamCharacterIndex]?.rules?.length ?? 0) <= (appState.expandedRuleIndex ?? -1)) {
    appState.expandedRuleIndex = null;
  }
  if (teamEditorConfig.jsonInput) {
    teamEditorConfig.jsonInput.value = JSON.stringify(teamConfig, null, 2);
  }
  renderTeamSummary(teamConfig);
  renderTeamValidation(validateTeamConfig(teamConfig));
  renderTeamEditor();
  restoreTeamEditorFocus(preservedFocus);
}

function captureTeamEditorFocus() {
  const activeElement = document.activeElement;
  if (!(activeElement instanceof HTMLInputElement) && !(activeElement instanceof HTMLSelectElement) && !(activeElement instanceof HTMLTextAreaElement)) {
    return null;
  }

  if (!builderRoots().some((root) => root.contains(activeElement))) {
    return null;
  }

  return {
    teamField: activeElement.dataset.teamField ?? "",
    characterField: activeElement.dataset.characterField ?? "",
    statField: activeElement.dataset.statField ?? "",
    positionField: activeElement.dataset.positionField ?? "",
    ruleField: activeElement.dataset.ruleField ?? "",
    conditionField: activeElement.dataset.conditionField ?? "",
    characterIndex: activeElement.dataset.characterIndex ?? "",
    ruleIndex: activeElement.dataset.ruleIndex ?? "",
    conditionIndex: activeElement.dataset.conditionIndex ?? "",
    selectionStart: activeElement instanceof HTMLInputElement || activeElement instanceof HTMLTextAreaElement
      ? activeElement.selectionStart
      : null,
    selectionEnd: activeElement instanceof HTMLInputElement || activeElement instanceof HTMLTextAreaElement
      ? activeElement.selectionEnd
      : null,
  };
}

function restoreTeamEditorFocus(snapshot) {
  if (!snapshot) {
    return;
  }

  const selectorParts = [];
  if (snapshot.teamField) selectorParts.push(`[data-team-field="${snapshot.teamField}"]`);
  if (snapshot.characterField) selectorParts.push(`[data-character-field="${snapshot.characterField}"]`);
  if (snapshot.statField) selectorParts.push(`[data-stat-field="${snapshot.statField}"]`);
  if (snapshot.positionField) selectorParts.push(`[data-position-field="${snapshot.positionField}"]`);
  if (snapshot.ruleField) selectorParts.push(`[data-rule-field="${snapshot.ruleField}"]`);
  if (snapshot.conditionField) selectorParts.push(`[data-condition-field="${snapshot.conditionField}"]`);
  if (snapshot.characterIndex) selectorParts.push(`[data-character-index="${snapshot.characterIndex}"]`);
  if (snapshot.ruleIndex) selectorParts.push(`[data-rule-index="${snapshot.ruleIndex}"]`);
  if (snapshot.conditionIndex) selectorParts.push(`[data-condition-index="${snapshot.conditionIndex}"]`);

  if (selectorParts.length === 0) {
    return;
  }

  const selector = selectorParts.join("");
  let nextElement = null;
  for (const root of builderRoots()) {
    nextElement = root.querySelector(selector);
    if (nextElement) {
      break;
    }
  }
  if (!(nextElement instanceof HTMLElement)) {
    return;
  }

  nextElement.focus();
  if ((nextElement instanceof HTMLInputElement || nextElement instanceof HTMLTextAreaElement)
    && snapshot.selectionStart != null
    && snapshot.selectionEnd != null) {
    nextElement.setSelectionRange(snapshot.selectionStart, snapshot.selectionEnd);
  }
}

function renderTeamValidation(result) {
  const output = teamEditorConfig.validationOutput;
  if (!output) {
    return;
  }
  if (result.ok) {
    setTeamValidationStatus("success", "Valid");
    return;
  }

  if (result.errors.length === 0) {
    setTeamValidationStatus("idle", "No team loaded");
    return;
  }

  setTeamValidationStatus("error", `${result.errors.length} issue${result.errors.length === 1 ? "" : "s"}`, result.errors);
}

function setTeamValidationStatus(kind, label, errors = []) {
  const output = teamEditorConfig.validationOutput;
  if (!output) {
    return;
  }
  output.className = `validation-inline validation-inline-${kind}`;
  const icon = kind === "success" ? "✓" : kind === "error" ? "✕" : "•";

  if (kind === "error" && errors.length > 0) {
    output.innerHTML = `
      <span class="validation-inline-icon">${icon}</span>
      <details>
        <summary><span class="validation-inline-text">${escapeHtml(label)}</span></summary>
        <div class="validation-inline-errors">${escapeHtml(errors.map((error) => `- ${error}`).join("\n"))}</div>
      </details>
    `;
    return;
  }

  output.innerHTML = `
    <span class="validation-inline-icon">${icon}</span>
    <span class="validation-inline-text">${escapeHtml(label)}</span>
  `;
}

async function copyTeamJson() {
  const team = appState.teamConfig;
  if (!team) {
    renderTeamValidation({ ok: false, errors: ["No team is loaded to copy."] });
    return;
  }

  const jsonText = JSON.stringify(team, null, 2);

  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(jsonText);
      setTeamValidationStatus("success", "Team copied");
      return;
    }
  } catch (error) {
    renderTeamValidation({ ok: false, errors: [`Could not copy team JSON: ${error.message}`] });
    return;
  }

  renderTeamValidation({ ok: false, errors: ["Clipboard access is not available in this browser context."] });
}

function downloadTeamJson() {
  const team = appState.teamConfig;
  if (!team) {
    renderTeamValidation({ ok: false, errors: ["No team is loaded to download."] });
    return;
  }

  const jsonText = JSON.stringify(team, null, 2);
  const blob = new Blob([jsonText], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `${team.name || "team"}.json`;
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
  setTeamValidationStatus("success", "Team downloaded");
}

function renderTeamEditor() {
  renderTeamTab();
  renderCharacterTab();
}

function renderTeamTab() {
  const editor = teamEditorConfig.editor;
  if (!editor) {
    return;
  }
  const team = appState.teamConfig;
  if (!team) {
    editor.innerHTML = '<div class="board-empty-state">Load a team to edit it here.</div>';
    return;
  }

  editor.innerHTML = `
    <section class="team-tab">
      <div class="team-manage-row">
        <div class="roster-controls">
          <select class="roster-select" data-team-action="roster-select" aria-label="Saved teams">
            ${renderRosterOptions()}
          </select>
          <button type="button" class="button-quiet" data-team-action="save-team-roster" title="Save the current team to your roster">Save</button>
          <button type="button" class="button-quiet" data-team-action="new-team" title="Start a new team">New</button>
          <button type="button" class="button-quiet" data-team-action="delete-team-roster" title="Remove this team from your roster">Delete</button>
        </div>
        <label class="field-group team-name-field team-name-field-compact">
          <input type="text" data-team-field="name" value="${escapeHtml(team.name)}" aria-label="Team name">
        </label>
        <div class="file-icon-actions" aria-label="Team actions">
          <button type="button" class="button-quiet" data-team-action="open-team-library" title="Load a team from your roster or import JSON">Load Team</button>
          <button type="button" class="icon-button" data-team-action="open-character-library" title="Character library" aria-label="Character library">${icon("library")}</button>
          <button type="button" class="icon-button" data-team-action="save-team-file" title="Export team JSON" aria-label="Export team JSON">${icon("export")}</button>
        </div>
      </div>
      <div class="team-tab-body">
        <section class="builder-card team-formation-card">
          <h4 class="builder-card-title">Formation</h4>
          ${renderFormationGrid(team, appState.selectedTeamCharacterIndex)}
          <p class="formation-hint">Drag a unit to reposition. Double-click a unit to edit it.</p>
        </section>
        <section class="builder-card team-roster-card">
          <div class="builder-pane-header team-roster-head">
            <h4 class="builder-card-title" style="margin:0">Roster</h4>
            ${renderBudgetMeter(team)}
          </div>
          ${renderRosterTable(team)}
        </section>
      </div>
    </section>
  `;
}

function renderRosterTable(team) {
  const canAdd = team.characters.length < TEAM_SLOT_POSITIONS.length;
  const rows = team.characters
    .map((character, index) => {
      const archetype = getArchetypeDefinition(character.template_id);
      const stats = getDerivedCharacterStats(character);
      const cost = (Number(archetype?.cost ?? 0))
        + (character.aspect ? Number(getAspectDefinition(character.aspect)?.cost ?? 0) : 0);
      const isSelected = index === appState.selectedTeamCharacterIndex;
      const statBits = ["vit", "mgt", "mag", "arm", "res", "spd"]
        .map((key) => `<span class="roster-stat"><span class="roster-stat-key">${key.toUpperCase()}</span>${Number(stats?.[key] ?? 0)}</span>`)
        .join("");
      return `
        <div class="roster-row ${isSelected ? "is-selected" : ""}" data-team-action="edit-character" data-character-index="${index}" role="button" tabindex="0" title="Edit ${escapeHtml(character.display_name || character.id || "character")}">
          <div class="roster-row-main">
            <span class="roster-portrait">${escapeHtml(getCharacterInitials(character))}</span>
            <div class="roster-id">
              <span class="roster-name">${escapeHtml(character.display_name || character.id || `Character ${index + 1}`)}</span>
              <span class="roster-archetype">${escapeHtml(archetype?.display_name ?? character.template_id ?? "—")}</span>
            </div>
            <span class="roster-cost">${cost} pt${cost === 1 ? "" : "s"}</span>
          </div>
          <div class="roster-stats">${statBits}</div>
          <div class="roster-row-actions">
            <button type="button" class="icon-button icon-button-sm row-action" data-team-action="edit-character" data-character-index="${index}" title="Edit" aria-label="Edit character">${icon("edit")}</button>
            <button type="button" class="icon-button icon-button-sm row-action row-action-danger" data-team-action="remove-character" data-character-index="${index}" title="Remove" aria-label="Remove character">${icon("trash")}</button>
          </div>
        </div>`;
    })
    .join("");

  const addRow = canAdd
    ? `<button type="button" class="roster-add" data-team-action="add-character" title="Add a character">${icon("add")}<span>Add character</span></button>`
    : "";

  return `
    <div class="roster-list">
      ${rows || '<div class="board-empty-state">No characters yet. Add one to begin.</div>'}
      ${addRow}
    </div>`;
}

function renderCharacterTab() {
  const editor = characterEditor;
  if (!editor) {
    return;
  }
  const team = appState.teamConfig;
  if (!team || team.characters.length === 0) {
    editor.innerHTML = '<div class="board-empty-state">Add a character on the Team tab, then select it here to edit.</div>';
    return;
  }

  const selectedIndex = clampValue(appState.selectedTeamCharacterIndex, 0, team.characters.length - 1);
  const selectedCharacter = team.characters[selectedIndex];

  editor.innerHTML = `
    <section class="character-tab">
      <div class="character-chip-row">${renderCharacterSlots(team)}</div>
      ${selectedCharacter ? renderSelectedCharacterWorkspace(selectedCharacter, selectedIndex) : '<div class="board-empty-state">Select a character to begin editing.</div>'}
    </section>
  `;
}

function renderRosterOptions() {
  const currentName = appState.teamConfig ? teamDisplayName(appState.teamConfig) : "";
  const inRoster = appState.teamRoster.some((entry) => teamDisplayName(entry) === currentName);
  const unsaved = !inRoster && currentName
    ? `<option value="" selected>${escapeHtml(currentName)} — unsaved</option>`
    : "";
  const options = appState.teamRoster
    .map((entry) => {
      const name = teamDisplayName(entry);
      return `<option value="${escapeHtml(name)}" ${inRoster && name === currentName ? "selected" : ""}>${escapeHtml(name)}</option>`;
    })
    .join("");
  return `${unsaved}${options}`;
}

function renderBudgetMeter(team) {
  const spend = computeTeamCost(team.characters);
  const over = spend > TEAM_BUDGET;
  return `
    <div class="budget-meter ${over ? "is-over" : ""}" title="Character + aspect point cost vs the team budget">
      <span class="budget-meter-label">Budget</span>
      <strong>${spend} / ${TEAM_BUDGET}</strong>
    </div>`;
}

function renderCharacterSlots(team) {
  const canAddCharacter = team.characters.length < TEAM_SLOT_POSITIONS.length;
  const slots = TEAM_SLOT_POSITIONS.map((_, slotIndex) => {
    const character = team.characters[slotIndex];
    if (character) {
      const isSelected = slotIndex === appState.selectedTeamCharacterIndex;
      return `
        <button
          type="button"
          class="character-slot-card ${isSelected ? "is-selected" : ""}"
          data-team-action="select-character"
          data-character-index="${slotIndex}"
          aria-selected="${isSelected ? "true" : "false"}"
        >
          <span class="character-slot-name">${escapeHtml(character.display_name || character.id || `Character ${slotIndex + 1}`)}</span>
        </button>
      `;
    }

    return `
      <button
        type="button"
        class="character-slot-add ${canAddCharacter ? "" : "is-disabled"}"
        data-team-action="add-character-slot"
        data-slot-index="${slotIndex}"
        ${canAddCharacter ? "" : "disabled"}
      >+</button>
    `;
  }).join("");

  return `<div class="character-slot-strip" aria-label="Team character slots">${slots}</div>`;
}

function renderSelectedCharacterWorkspace(character, characterIndex) {
  const isDesignTab = appState.teamDetailTab !== "rules";
  return `
    <article class="builder-character-workspace">
      <div class="team-detail-tabbar" role="tablist" aria-label="Character editing tabs">
        <div class="team-detail-tabbar-tabs">
          <button type="button" class="team-detail-tab ${isDesignTab ? "is-active" : ""}" role="tab" aria-selected="${isDesignTab ? "true" : "false"}" data-team-action="select-detail-tab" data-detail-tab="design">Design</button>
          <button type="button" class="team-detail-tab ${!isDesignTab ? "is-active" : ""}" role="tab" aria-selected="${!isDesignTab ? "true" : "false"}" data-team-action="select-detail-tab" data-detail-tab="rules">Rules</button>
        </div>
        <div class="team-detail-tabbar-actions">
          <button type="button" class="button-quiet" data-team-action="save-character" data-character-index="${characterIndex}" title="Save this character build to your library">Save Character</button>
          <button type="button" class="button-quiet" data-team-action="load-character" data-character-index="${characterIndex}" title="Load a saved character into this slot">Load Character</button>
          <button type="button" class="button-quiet" data-team-action="remove-character" data-character-index="${characterIndex}">Delete</button>
        </div>
      </div>
      <input class="visually-hidden" type="file" accept=".json,application/json" data-team-action="load-character-file" data-character-index="${characterIndex}">
      ${
        isDesignTab
          ? `
            <div class="builder-design">
              ${renderIdentityCard(character, characterIndex)}
              <section class="builder-card builder-browser-card">
                ${renderSelectionBrowser(character)}
              </section>
              <section class="builder-card builder-loadout-card">
                <h4 class="builder-card-title">Loadout</h4>
                <div class="loadout-list">${renderLoadoutPane(character, characterIndex)}</div>
              </section>
            </div>
          `
          : `<div class="builder-rules">${renderRulesWorkspace(character, characterIndex)}</div>`
      }
    </article>
  `;
}

function renderIdentityCard(character, characterIndex) {
  const archetype = getArchetypeDefinition(character.template_id);
  const derivedStats = getDerivedCharacterStats(character);
  const subLine = `${archetype?.display_name ?? character.template_id ?? "Unknown"} · cost ${archetype?.cost ?? 0}`;
  return `
    <section class="builder-card builder-identity-card">
      <div class="identity-head">
        <div class="portrait-placeholder">${escapeHtml(getCharacterInitials(character))}</div>
        <div>
          <div class="identity-name">${escapeHtml(character.display_name || `Character ${characterIndex + 1}`)}</div>
          <div class="identity-sub">${escapeHtml(subLine)}</div>
        </div>
      </div>
      <div class="identity-fields">
        <label class="field-group field-group-compact">
          <span>Archetype</span>
          <select data-character-field="template_id" data-character-index="${characterIndex}">
            ${buildArchetypeOptions(character.template_id ?? "")}
          </select>
        </label>
        <label class="field-group field-group-compact">
          <span>Name</span>
          <input type="text" data-character-field="display_name" data-character-index="${characterIndex}" value="${escapeHtml(character.display_name ?? "")}">
        </label>
      </div>
      <h4 class="builder-card-title">Stats</h4>
      <div class="editor-inline-grid">
        ${["vit", "mgt", "mag", "arm", "res", "spd"].map((statKey) => `
          <div class="field-group field-group-readonly">
            <span class="stat-label-with-icon">
              ${renderStatIcon(statKey)}
              <span>${statKey.toUpperCase()}</span>
            </span>
            <div class="derived-stat-value">${renderDerivedStatValue(
              Number(archetype?.stats?.[statKey] ?? 0),
              Number(derivedStats?.[statKey] ?? 0),
            )}</div>
          </div>
        `).join("")}
      </div>
    </section>
  `;
}

function renderFormationGrid(team, selectedIndex) {
  // Facing orientation matching the replay viewer: depth (front/middle/back rows)
  // runs horizontally with the team's front toward the enemy on the right;
  // lanes (columns) stack vertically. Cells are repositioned by dragging.
  const depthOrder = [2, 1, 0]; // back, middle, front (left -> right, facing enemy)
  const depthLabels = { 0: "Front", 1: "Middle", 2: "Back" };
  let markup = '<div class="formation-corner"></div>';
  for (const depth of depthOrder) {
    markup += `<div class="formation-depth-label">${depthLabels[depth]}</div>`;
  }
  for (let lane = 0; lane < 3; lane += 1) {
    markup += `<div class="formation-lane-label">${lane + 1}</div>`;
    for (const depth of depthOrder) {
      const occ = findCharacterIndexAtPosition(team, depth, lane);
      const occupied = occ >= 0;
      const isSelected = occ === selectedIndex;
      const occupant = occupied ? team.characters[occ] : null;
      const title = occupied
        ? `${occupant.display_name || occupant.id || "Character"} — drag to move`
        : `${depthLabels[depth]} row, lane ${lane + 1}`;
      markup += `
        <button
          type="button"
          class="formation-cell ${occupied ? "is-filled" : ""} ${isSelected ? "is-selected" : ""}"
          data-row="${depth}"
          data-col="${lane}"
          title="${escapeHtml(title)}"
        >${occupied ? escapeHtml(getCharacterInitials(occupant)) : ""}</button>`;
    }
  }
  return `
    <div class="formation-wrap">
      <div class="formation-grid">${markup}</div>
      <div class="formation-facing" aria-hidden="true">enemy →</div>
    </div>`;
}

function renderLoadoutPane(character, characterIndex) {
  return `
    ${renderLoadoutSlot("Passive", character.passive, "passive", characterIndex)}
    ${normalizeActiveSelections(character.actives).map((abilityName, activeIndex) =>
      renderLoadoutSlot(`Active ${activeIndex + 1}`, abilityName, "active", characterIndex, activeIndex)).join("")}
    ${renderLoadoutSlot("Aspect", character.aspect, "aspect", characterIndex)}
  `;
}

function renderLoadoutSlot(label, value, mode, characterIndex, slotIndex = null) {
  const isSelectedBrowser =
    appState.teamBrowserMode === mode &&
    appState.teamBrowserSlotIndex === (slotIndex ?? 0);
  const displayValue =
    mode === "aspect"
      ? getAspectDisplayName(value)
      : value;
  const description =
    mode === "passive"
      ? getPassiveDescription(value)
      : mode === "aspect"
        ? getAspectSummary(value)
        : getAbilityDescription(value);
  const mpCost = mode === "active" ? getAbilityMpCost(value) : null;
  const typeKey = mode === "passive" ? "passive" : mode === "aspect" ? "aspect" : "active";
  const isEmpty = !displayValue;
  const tooltip = [label, displayValue ? `— ${displayValue}` : "(empty)", description ? `\n${description}` : ""]
    .filter(Boolean)
    .join(" ");

  return `
    <button
      type="button"
      class="loadout-slot loadout-slot-${typeKey} ${isSelectedBrowser ? "is-selected" : ""} ${isEmpty ? "is-empty" : ""}"
      data-team-action="focus-browser"
      data-browser-mode="${mode}"
      data-browser-slot-index="${slotIndex ?? 0}"
      data-character-index="${characterIndex}"
      aria-label="${escapeHtml(label)}"
      title="${escapeHtml(tooltip)}"
    >
      ${loadoutTypeIcon(mode)}
      <span class="loadout-slot-name">${escapeHtml(displayValue || "Empty")}</span>
      ${mode === "active" && displayValue ? renderAbilityDamageChip(value) : ""}
      ${mpCost == null ? "" : `<span class="loadout-slot-cost">${escapeHtml(`${mpCost}`)}<span class="loadout-slot-cost-unit">MP</span></span>`}
    </button>
  `;
}

function renderCompactRules(character, characterIndex) {
  const rules = character.rules ?? [];
  const ruleCount = rules.length;
  const canAddRule = ruleCount < 5;
  const rulesMarkup = rules.map((rule, ruleIndex) => {
    const isSelected = appState.expandedRuleIndex === ruleIndex;
    return `
      <article class="compact-rule-card ${isSelected ? "is-expanded" : ""}">
        <div class="compact-rule-header">
          <button type="button" class="rule-select-button" data-team-action="select-rule" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">
            <div class="compact-rule-index">Priority ${ruleIndex + 1}</div>
            <div class="compact-rule-text">${escapeHtml(formatRulePreview(rule))}</div>
          </button>
          <div class="rule-action-row">
            <button type="button" class="button-quiet rule-icon-button" title="Move rule up" aria-label="Move rule up" data-team-action="move-rule-up" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" ${ruleIndex === 0 ? "disabled" : ""}>&uarr;</button>
            <button type="button" class="button-quiet rule-icon-button" title="Move rule down" aria-label="Move rule down" data-team-action="move-rule-down" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" ${ruleIndex === ruleCount - 1 ? "disabled" : ""}>&darr;</button>
            <button type="button" class="button-quiet rule-icon-button" title="Delete rule" aria-label="Delete rule" data-team-action="remove-rule" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">&#128465;</button>
          </div>
        </div>
      </article>
    `;
  }).join("");

  return `
    <div class="builder-pane-header">
      <h4 class="builder-card-title" style="margin:0">Priority Rules</h4>
      <div class="editor-card-actions">
        <span class="rule-count-label">${ruleCount} / 5</span>
        <button type="button" class="button-secondary" data-team-action="add-rule" data-character-index="${characterIndex}" ${canAddRule ? "" : "disabled"}>Add Rule</button>
      </div>
    </div>
    <div class="compact-rule-list">
      ${rulesMarkup || '<div class="board-empty-state">Add a priority rule to script this character. If none match, the character uses Basic Attack.</div>'}
    </div>
  `;
}

function renderRulesWorkspace(character, characterIndex) {
  const selectedRuleIndex = clampValue(appState.expandedRuleIndex ?? 0, 0, Math.max((character.rules?.length ?? 1) - 1, 0));
  const selectedRule = character.rules?.[selectedRuleIndex] ?? null;
  return `
    <section class="builder-card builder-rules-list">
      ${renderCompactRules(character, characterIndex)}
    </section>
    <section class="builder-card builder-rules-editor">
      <h4 class="builder-card-title">Rule Detail</h4>
      ${renderRuleEditor(characterIndex, selectedRule, selectedRuleIndex)}
    </section>
  `;
}

function renderSelectionBrowser(character) {
  const mode = appState.teamBrowserMode;
  const slotIndex = appState.teamBrowserSlotIndex;
  const entries = getBrowserEntries(mode);
  const currentValue =
    mode === "passive"
      ? character.passive ?? ""
      : mode === "aspect"
        ? character.aspect ?? ""
        : normalizeActiveSelections(character.actives)[slotIndex] ?? "";
  const roleKey = mode === "passive" ? "passive" : mode === "aspect" ? "aspect" : "active";
  const roleLabel = mode === "passive" ? "Passive" : mode === "aspect" ? "Aspect" : "Active";
  const currentLabel =
    mode === "aspect"
      ? getAspectDisplayName(currentValue)
      : currentValue;

  return `
    <div class="builder-pane-header browser-header">
      <span class="role-chip role-chip-${roleKey}">
        <span class="role-chip-icon" aria-hidden="true">${UI_ICONS[roleKey]}</span>
        <span>${roleLabel}</span>
      </span>
      <div class="editor-card-actions">
        <span class="browser-current-label">${escapeHtml(currentLabel || "Nothing selected")}</span>
        <button
          type="button"
          class="button-quiet browser-clear-button"
          data-team-action="select-browser-entry"
          data-browser-mode="${mode}"
          data-browser-slot-index="${slotIndex}"
          data-entry-value=""
          ${currentValue === "" || currentValue == null ? "disabled" : ""}
          title="Remove the current ${mode === "active" ? `Active ${slotIndex + 1}` : mode}"
        >Clear</button>
      </div>
    </div>
    <div class="selection-browser-list">
      ${
        entries.length === 0
          ? `<div class="board-empty-state">${mode === "aspect" ? "Aspects are not in the catalog yet." : "No entries are available for this browser."}</div>`
          : entries.map((entry) => renderBrowserEntry(entry, mode, slotIndex, currentValue)).join("")
      }
    </div>
  `;
}

function renderBrowserEntry(entry, mode, slotIndex, currentValue) {
  const isAbility = mode === "active";
  const mpCost = isAbility ? getAbilityMpCost(entry.name) : null;
  const entryLabel = mode === "aspect" ? getAspectDisplayName(entry.name) : entry.name;
  const damageChip = isAbility ? renderAbilityDamageChip(entry.name) : "";
  const descriptionMarkup = mode === "aspect"
    ? renderAspectSummaryMarkup(entry.name)
    : isAbility
      ? formatAbilityDescriptionMarkup(entry.description)
      : escapeHtml(entry.description || "No description yet.");
  const descriptionBlock = descriptionMarkup.trim() === ""
    ? ""
    : `<span class="selection-browser-entry-desc">${descriptionMarkup}</span>`;
  return `
    <button
      type="button"
      class="selection-browser-entry ${entry.name === currentValue ? "is-selected" : ""}"
      data-team-action="select-browser-entry"
      data-browser-mode="${mode}"
      data-browser-slot-index="${slotIndex}"
      data-entry-value="${escapeHtml(entry.name)}"
    >
      <span class="selection-browser-entry-header">
        <span class="selection-browser-entry-title">
          <strong>${escapeHtml(entryLabel)}</strong>
          ${damageChip}
        </span>
        ${mpCost == null ? "" : `<span class="selection-browser-entry-cost">${escapeHtml(`${mpCost} MP`)}</span>`}
      </span>
      ${descriptionBlock}
    </button>
  `;
}

function renderRuleEditor(characterIndex, rule, ruleIndex) {
  if (!rule) {
    return `
      <div class="board-empty-state">Select or add a rule to edit its condition.</div>
    `;
  }

  const equippedAbilityNames = normalizeActiveSelections(appState.teamConfig?.characters?.[characterIndex]?.actives)
    .filter((name) => name && name.trim() !== "");
  const abilityOptions = buildSelectOptions(
    [basicAttackActionName, ...equippedAbilityNames.filter((name) => name !== basicAttackActionName)],
    rule.ability ?? "",
    "No ability selected",
  );
  const condition = rule.when?.[0] ?? null;

  return `
    <article class="editor-card">
      <div class="editor-card-header">
        <div class="rule-preview">${escapeHtml(formatRulePreview(rule))}</div>
      </div>
      <label class="field-group">
        <span>Ability</span>
        <select data-rule-field="ability" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">
          ${abilityOptions}
        </select>
      </label>
      <div class="editor-card-header">
        <div class="editor-card-actions">
          ${
            condition
              ? ``
              : `<button type="button" class="button-secondary" data-team-action="add-condition" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">Add Condition</button>`
          }
        </div>
      </div>
      <div class="condition-editor-list">${condition ? renderConditionEditor(characterIndex, ruleIndex, condition, 0) : '<div class="board-empty-state">Add one condition to decide when this priority fires.</div>'}</div>
    </article>
  `;
}

function renderConditionEditor(characterIndex, ruleIndex, condition, conditionIndex) {
  const allowedValueOptions = getAllowedRuleValueOptions(condition.subject ?? "self");
  const value = condition.value;
  const valueType = getConditionValueType(condition);
  const statValue = valueType === "stat" ? value.stat : "vit";
  const statusValue =
    valueType === "status_stacks"
        ? (value.status_stacks ?? value.has_status)
        : valueType === "condition_stacks"
          ? (value.condition_stacks ?? value.has_condition)
          : "Ward";
  const statusOptions = valueType === "condition_stacks"
    ? buildRequiredSelectOptions(appState.catalogs.conditions, statusValue)
    : buildRequiredSelectOptions(appState.catalogs.statuses, statusValue);
  const detailFieldMarkup = valueType === "stat"
    ? `
        <label class="field-group">
          <span>Stat</span>
          <select data-condition-field="value_stat" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" data-condition-index="${conditionIndex}">
            ${statFieldOptions.map((option) => `<option value="${option}" ${statValue === option ? "selected" : ""}>${option.toUpperCase()}</option>`).join("")}
          </select>
        </label>
      `
    : valueType === "status_stacks" || valueType === "condition_stacks"
      ? `
        <label class="field-group">
          <span>${valueType === "condition_stacks" ? "Condition" : "Status"}</span>
          <select data-condition-field="value_status" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" data-condition-index="${conditionIndex}">
            ${statusOptions}
          </select>
        </label>
      `
      : "";

  return `
    <div class="editor-card">
      <div class="editor-card-header">
        <div class="condition-preview">${escapeHtml(formatConditionPreview(condition))}</div>
        <div class="editor-card-actions">
          <button type="button" class="button-quiet rule-icon-button" title="Remove condition" aria-label="Remove condition" data-team-action="remove-condition" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" data-condition-index="${conditionIndex}">&#128465;</button>
        </div>
      </div>
      <div class="condition-grid">
        <label class="field-group">
          <span>Subject</span>
          <select data-condition-field="subject" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" data-condition-index="${conditionIndex}">
            ${ruleSubjectOptions.map((option) => `<option value="${option.value}" ${condition.subject === option.value ? "selected" : ""}>${option.label}</option>`).join("")}
          </select>
        </label>
        <label class="field-group">
          <span>Value</span>
          <select data-condition-field="value_type" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" data-condition-index="${conditionIndex}">
            ${allowedValueOptions.map((option) => `<option value="${option.value}" ${valueType === option.value ? "selected" : ""}>${option.label}</option>`).join("")}
          </select>
        </label>
        ${detailFieldMarkup}
        <label class="field-group">
          <span>Operator</span>
          <select data-condition-field="op" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" data-condition-index="${conditionIndex}">
            ${ruleOperatorOptions.map((option) => `<option value="${option.value}" ${(condition.op ?? condition.comparator) === option.value ? "selected" : ""}>${option.label}</option>`).join("")}
          </select>
        </label>
        <label class="field-group">
          <span>Threshold</span>
          <input type="number" data-condition-field="threshold" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" data-condition-index="${conditionIndex}" value="${condition.threshold ?? 0}">
        </label>
      </div>
    </div>
  `;
}

function handleTeamEditorInput(event) {
  const team = appState.teamConfig;
  if (!team) {
    return;
  }

  const target = event.target;
  const characterIndex = Number(target.dataset.characterIndex);
  const ruleIndex = Number(target.dataset.ruleIndex);
  const conditionIndex = Number(target.dataset.conditionIndex);

    if (target.dataset.teamField === "name") {
      team.name = target.value;
      syncTeamUI();
      return;
    }

  if (target.dataset.characterField) {
    const character = team.characters[characterIndex];
    if (!character) {
      return;
    }

    if (target.dataset.characterField === "template_id") {
      applyTemplateToCharacter(character, target.value);
    } else if (target.dataset.characterField.startsWith("active_")) {
      const activeIndex = Number(target.dataset.characterField.split("_")[1]);
      const nextActives = normalizeActiveSelections(character.actives);
      nextActives[activeIndex] = target.value.trim();
      character.actives = nextActives.filter(Boolean);
    } else if (target.dataset.characterField === "aspect") {
      character.aspect = target.value.trim() === "" ? null : target.value;
    } else {
      character[target.dataset.characterField] = target.value;
    }

    syncTeamUI();
    return;
  }

  if (target.dataset.positionField) {
    const character = team.characters[characterIndex];
    if (!character) {
      return;
    }
    const nextValue = Number(target.value);
    character.position = {
      row: character.position?.row ?? 0,
      col: character.position?.col ?? 0,
    };
    character.position[target.dataset.positionField] = nextValue;
    syncTeamUI();
    return;
  }

  if (target.dataset.ruleField === "ability") {
    const rule = team.characters[characterIndex]?.rules?.[ruleIndex];
    if (!rule) {
      return;
    }
    rule.ability = target.value;
    syncTeamUI();
    return;
  }

  if (target.dataset.conditionField) {
    const condition = team.characters[characterIndex]?.rules?.[ruleIndex]?.when?.[conditionIndex];
    if (!condition) {
      return;
    }

    if (target.dataset.conditionField === "subject") {
      condition.subject = target.value;
      normalizeConditionForSubject(condition);
    } else if (target.dataset.conditionField === "value_type") {
      if (target.value === "stat") {
        condition.value = { stat: "vit" };
      } else if (target.value === "status_stacks") {
        condition.value = { status_stacks: "Empower:MGT" };
      } else if (target.value === "condition_stacks") {
        condition.value = { condition_stacks: "Stunned" };
      } else {
        condition.value = target.value;
      }
    } else if (target.dataset.conditionField === "value_stat") {
      condition.value = { stat: target.value };
    } else if (target.dataset.conditionField === "value_status") {
      if (isPlainObject(condition.value) && typeof condition.value.condition_stacks === "string") {
        condition.value = { condition_stacks: target.value };
      } else {
        condition.value = { status_stacks: target.value };
      }
    } else if (target.dataset.conditionField === "op") {
      condition.op = target.value;
      delete condition.comparator;
    } else if (target.dataset.conditionField === "threshold") {
      condition.threshold = Number(target.value);
    }

    syncTeamUI();
  }
}

async function handleTeamEditorChange(event) {
  const target = event.target;
  if (target.dataset.teamAction === "load-character-file") {
    await loadCharacterFromFileInput(target);
    return;
  }

  if (target.dataset.teamAction === "roster-select") {
    loadRosterTeam(target.value);
    return;
  }

  handleTeamEditorInput(event);
}

function loadRosterTeam(name) {
  if (!name) {
    return;
  }
  const entry = appState.teamRoster.find((team) => teamDisplayName(team) === name);
  if (!entry) {
    return;
  }
  appState.teamConfig = structuredClone(entry);
  appState.activeTeamName = name;
  appState.selectedTeamCharacterIndex = 0;
  appState.expandedRuleIndex = null;
  appState.teamDetailTab = "design";
  syncTeamUI();
}

// ===== Load Team overlay (roster teams + import JSON) =====
const teamLibraryOverlay = document.querySelector("#team-library-overlay");
const teamLibraryBody = document.querySelector("#team-library-body");
document.querySelector("#team-library-close-button")?.addEventListener("click", () => closeTeamLibrary());
teamLibraryOverlay?.addEventListener("click", (event) => {
  if (event.target === teamLibraryOverlay) {
    closeTeamLibrary();
    return;
  }
  handleTeamLibraryAction(event);
});
window.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && teamLibraryOverlay && !teamLibraryOverlay.hidden) {
    closeTeamLibrary();
  }
});

function openTeamLibrary() {
  if (!teamLibraryOverlay) {
    return;
  }
  renderTeamLibrary();
  teamLibraryOverlay.hidden = false;
}

function closeTeamLibrary() {
  if (teamLibraryOverlay) {
    teamLibraryOverlay.hidden = true;
  }
}

function renderTeamLibrary() {
  if (!teamLibraryBody) {
    return;
  }
  const roster = appState.teamRoster ?? [];
  const list = roster.length === 0
    ? '<div class="board-empty-state">No saved teams yet. Build a team and press “Save” to add it to your roster.</div>'
    : roster
        .map((team, index) => {
          const name = teamDisplayName(team);
          const cost = computeTeamCost(team.characters);
          const count = team.characters?.length ?? 0;
          return `
            <div class="library-entry">
              <div class="library-entry-info">
                <div class="library-entry-name">${escapeHtml(name)}</div>
                <div class="library-entry-sub">${count} characters · ${cost} pts</div>
              </div>
              <div class="library-entry-actions">
                <button type="button" class="button-secondary" data-team-library-action="load" data-team-index="${index}">Load</button>
                <button type="button" class="button-quiet" data-team-library-action="export" data-team-index="${index}">Export</button>
                <button type="button" class="button-quiet" data-team-library-action="delete" data-team-index="${index}">Delete</button>
              </div>
            </div>`;
        })
        .join("");
  teamLibraryBody.innerHTML = `
    <p class="library-intro">Load a team from your roster, or import one from JSON.</p>
    <div class="library-actions">
      <button type="button" class="button-secondary" data-team-library-action="import">Import JSON…</button>
      <input class="visually-hidden" type="file" id="team-library-import-input" accept=".json,application/json">
    </div>
    <div class="library-list">${list}</div>
  `;
  const importInput = teamLibraryBody.querySelector("#team-library-import-input");
  importInput?.addEventListener("change", () => void importTeamFromFile(importInput));
}

function handleTeamLibraryAction(event) {
  const actionTarget = event.target.closest?.("[data-team-library-action]");
  if (!actionTarget) {
    return;
  }
  const action = actionTarget.dataset.teamLibraryAction;
  const index = Number(actionTarget.dataset.teamIndex);

  if (action === "import") {
    teamLibraryBody?.querySelector("#team-library-import-input")?.click();
    return;
  }
  const team = appState.teamRoster[index];
  if (action === "load") {
    if (team) {
      loadRosterTeam(teamDisplayName(team));
      closeTeamLibrary();
    }
    return;
  }
  if (action === "export") {
    if (team) {
      triggerJsonDownload(JSON.stringify(team, null, 2), `${slugifyFileStem(teamDisplayName(team)) || "team"}.json`);
    }
    return;
  }
  if (action === "delete") {
    const name = team ? teamDisplayName(team) : null;
    appState.teamRoster.splice(index, 1);
    if (name && appState.activeTeamName === name) {
      appState.activeTeamName = null;
    }
    persistRoster();
    renderTeamLibrary();
    renderTeamEditor();
    renderArena();
  }
}

async function importTeamFromFile(input) {
  const [file] = input.files ?? [];
  if (!file) {
    return;
  }
  try {
    const parsed = JSON.parse(await file.text());
    const validation = validateTeamConfig(parsed);
    if (!validation.ok) {
      renderTeamValidation(validation);
    } else {
      upsertTeamInRoster(parsed);
      loadRosterTeam(teamDisplayName(parsed));
      closeTeamLibrary();
    }
  } catch (error) {
    renderTeamValidation({ ok: false, errors: [`Could not import team JSON: ${error.message}`] });
  } finally {
    input.value = "";
  }
}

function createNewTeam() {
  return {
    version: 2,
    name: "New Team",
    characters: [createEmptyCharacter(0, TEAM_SLOT_POSITIONS[0].row, TEAM_SLOT_POSITIONS[0].col)],
  };
}

// ===== Character library overlay =====
const libraryOverlay = document.querySelector("#library-overlay");
const libraryCloseButton = document.querySelector("#library-close-button");
const libraryBody = document.querySelector("#library-body");

libraryCloseButton?.addEventListener("click", () => closeCharacterLibrary());
libraryOverlay?.addEventListener("click", (event) => {
  if (event.target === libraryOverlay) {
    closeCharacterLibrary();
    return;
  }
  handleLibraryAction(event);
});
window.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && libraryOverlay && !libraryOverlay.hidden) {
    closeCharacterLibrary();
  }
});

function openCharacterLibrary() {
  if (!libraryOverlay) {
    return;
  }
  renderCharacterLibraryOverlay();
  libraryOverlay.hidden = false;
}

function closeCharacterLibrary() {
  if (libraryOverlay) {
    libraryOverlay.hidden = true;
  }
  appState.libraryTargetSlot = null;
}

function renderCharacterLibraryOverlay() {
  if (!libraryBody) {
    return;
  }
  const lib = appState.characterLibrary;
  const target = appState.libraryTargetSlot;
  const intro = typeof target === "number"
    ? `<p class="library-intro">Choose a saved character to load into slot ${target + 1}.</p>`
    : `<p class="library-intro">Your saved character builds. Add one to the current team, export it, or import one.</p>`;
  const list = lib.length === 0
    ? '<div class="board-empty-state">No saved characters yet. Use “Save Character” on a character to add a reusable build here.</div>'
    : lib
        .map((character, index) => {
          const archetype = getArchetypeDefinition(character.template_id);
          return `
            <div class="library-entry">
              <div class="library-entry-info">
                <div class="library-entry-name">${escapeHtml(characterLabel(character))}</div>
                <div class="library-entry-sub">${escapeHtml(archetype?.display_name ?? character.template_id ?? "")} · cost ${archetype?.cost ?? 0}</div>
              </div>
              <div class="library-entry-actions">
                <button type="button" class="button-secondary" data-library-action="use" data-library-index="${index}">${typeof target === "number" ? "Load into slot" : "Add to team"}</button>
                <button type="button" class="button-quiet" data-library-action="export" data-library-index="${index}">Export</button>
                <button type="button" class="button-quiet" data-library-action="delete" data-library-index="${index}">Delete</button>
              </div>
            </div>`;
        })
        .join("");
  libraryBody.innerHTML = `
    ${intro}
    <div class="library-actions">
      <button type="button" class="button-secondary" data-library-action="import">Import JSON…</button>
      <input class="visually-hidden" type="file" id="library-import-input" accept=".json,application/json">
    </div>
    <div class="library-list">${list}</div>
  `;
  const importInput = libraryBody.querySelector("#library-import-input");
  importInput?.addEventListener("change", () => void importCharacterToLibrary(importInput));
}

function handleLibraryAction(event) {
  const actionTarget = event.target.closest?.("[data-library-action]");
  if (!actionTarget) {
    return;
  }
  const action = actionTarget.dataset.libraryAction;
  const index = Number(actionTarget.dataset.libraryIndex);

  if (action === "import") {
    libraryBody?.querySelector("#library-import-input")?.click();
    return;
  }
  if (action === "use") {
    if (placeLibraryCharacter(index)) {
      closeCharacterLibrary();
      syncTeamUI();
    }
    return;
  }
  if (action === "export") {
    const character = appState.characterLibrary[index];
    if (character) {
      triggerJsonDownload(JSON.stringify(character, null, 2), buildCharacterFilename(character, `character_${index + 1}`));
    }
    return;
  }
  if (action === "delete") {
    appState.characterLibrary.splice(index, 1);
    persistCharacterLibrary();
    renderCharacterLibraryOverlay();
  }
}

async function importCharacterToLibrary(input) {
  const [file] = input.files ?? [];
  if (!file) {
    return;
  }
  try {
    const parsed = JSON.parse(await file.text());
    const errors = validateCharacterConfig(parsed, "character");
    if (errors.length > 0) {
      renderTeamValidation({ ok: false, errors });
    } else {
      upsertCharacterInLibrary(parsed);
      renderCharacterLibraryOverlay();
    }
  } catch (error) {
    renderTeamValidation({ ok: false, errors: [`Could not import character JSON: ${error.message}`] });
  } finally {
    input.value = "";
  }
}

function uniqueCharacterId(team, desiredId, excludeIndex = -1) {
  const base = (desiredId || "character").trim() || "character";
  const taken = new Set(team.characters.filter((_, i) => i !== excludeIndex).map((c) => c.id));
  if (!taken.has(base)) {
    return base;
  }
  let suffix = 2;
  while (taken.has(`${base}_${suffix}`)) {
    suffix += 1;
  }
  return `${base}_${suffix}`;
}

function placeLibraryCharacter(libIndex) {
  const team = appState.teamConfig;
  const source = appState.characterLibrary[libIndex];
  if (!team || !source) {
    return false;
  }
  const character = structuredClone(source);
  const target = appState.libraryTargetSlot;
  if (typeof target === "number" && team.characters[target]) {
    const pos = team.characters[target].position ?? { row: 0, col: 0 };
    character.position = { row: pos.row, col: pos.col };
    character.id = uniqueCharacterId(team, character.id, target);
    team.characters[target] = character;
    appState.selectedTeamCharacterIndex = target;
  } else {
    const slotIndex = findFirstOpenSlotIndex(team);
    if (slotIndex < 0) {
      renderTeamValidation({ ok: false, errors: [`A team can have at most ${TEAM_SLOT_POSITIONS.length} characters.`] });
      return false;
    }
    const pos = TEAM_SLOT_POSITIONS[slotIndex];
    character.position = { row: pos.row, col: pos.col };
    character.id = uniqueCharacterId(team, character.id);
    team.characters.push(character);
    appState.selectedTeamCharacterIndex = team.characters.length - 1;
  }
  appState.expandedRuleIndex = null;
  return true;
}

function handleTeamEditorAction(event) {
  const actionTarget = event.target.closest?.("[data-team-action]");
  const action = actionTarget?.dataset.teamAction;
  if (!action || !(actionTarget instanceof HTMLElement)) {
    return;
  }

  const characterIndex = Number(actionTarget.dataset.characterIndex);
  const ruleIndex = Number(actionTarget.dataset.ruleIndex);
  const conditionIndex = Number(actionTarget.dataset.conditionIndex);
  const team = appState.teamConfig;

  if (!team) {
    return;
  }

  let navigateToCharacterTab = false;

  switch (action) {
    case "open-team-library":
      openTeamLibrary();
      return;
    case "save-team-file":
      downloadTeamJson();
      return;
    case "save-team-roster":
      upsertTeamInRoster(team);
      setTeamValidationStatus("success", `Saved “${teamDisplayName(team)}” to roster`);
      break;
    case "new-team":
      appState.teamConfig = createNewTeam();
      appState.activeTeamName = null;
      appState.selectedTeamCharacterIndex = 0;
      appState.expandedRuleIndex = null;
      break;
    case "delete-team-roster": {
      const name = teamDisplayName(team);
      appState.teamRoster = appState.teamRoster.filter((entry) => teamDisplayName(entry) !== name);
      appState.activeTeamName = null;
      persistRoster();
      setTeamValidationStatus("idle", `Removed “${name}” from roster`);
      break;
    }
    case "open-character-library":
      appState.libraryTargetSlot = null;
      openCharacterLibrary();
      return;
    case "add-character":
      addCharacterAtFirstOpenPosition(team);
      appState.teamDetailTab = "design";
      navigateToCharacterTab = true;
      break;
    case "add-character-slot":
      addCharacterAtSlot(team, Number(actionTarget.dataset.slotIndex));
      appState.teamDetailTab = "design";
      navigateToCharacterTab = true;
      break;
    case "select-detail-tab":
      appState.teamDetailTab = actionTarget.dataset.detailTab === "rules" ? "rules" : "design";
      break;
    case "select-character":
      appState.selectedTeamCharacterIndex = characterIndex;
      appState.expandedRuleIndex = null;
      appState.teamDetailTab = "design";
      break;
    case "edit-character":
      appState.selectedTeamCharacterIndex = characterIndex;
      appState.expandedRuleIndex = null;
      appState.teamDetailTab = "design";
      navigateToCharacterTab = true;
      break;
    case "focus-browser":
      appState.teamDetailTab = "design";
      appState.teamBrowserMode = actionTarget.dataset.browserMode ?? "active";
      appState.teamBrowserSlotIndex = Number(actionTarget.dataset.browserSlotIndex ?? 0);
      appState.selectedTeamCharacterIndex = characterIndex;
      break;
    case "select-rule":
      appState.teamDetailTab = "rules";
      appState.expandedRuleIndex = ruleIndex;
      appState.selectedTeamCharacterIndex = characterIndex;
      break;
    case "select-browser-entry":
      applyBrowserSelection(actionTarget);
      return;
    case "copy-team":
      void copyTeamJson();
      return;
    case "download-team":
      downloadTeamJson();
      break;
    case "save-character": {
      const character = team.characters[characterIndex];
      if (character) {
        upsertCharacterInLibrary(character);
        setTeamValidationStatus("success", `Saved “${characterLabel(character)}” to library`);
      }
      return;
    }
    case "load-character":
      appState.libraryTargetSlot = characterIndex;
      openCharacterLibrary();
      return;
    case "remove-character":
      team.characters.splice(characterIndex, 1);
      appState.selectedTeamCharacterIndex = Math.max(0, Math.min(appState.selectedTeamCharacterIndex, team.characters.length - 1));
      appState.expandedRuleIndex = null;
      break;
    case "add-rule":
      if ((team.characters[characterIndex]?.rules?.length ?? 0) < 5) {
        team.characters[characterIndex]?.rules.push(createEmptyRule());
        appState.expandedRuleIndex = team.characters[characterIndex].rules.length - 1;
        appState.teamDetailTab = "rules";
      }
      break;
    case "remove-rule":
      team.characters[characterIndex]?.rules.splice(ruleIndex, 1);
      if (appState.expandedRuleIndex === ruleIndex) {
        appState.expandedRuleIndex = null;
      } else if ((appState.expandedRuleIndex ?? -1) > ruleIndex) {
        appState.expandedRuleIndex -= 1;
      }
      break;
    case "move-rule-up":
      moveArrayItem(team.characters[characterIndex]?.rules, ruleIndex, ruleIndex - 1);
      if (appState.expandedRuleIndex === ruleIndex) {
        appState.expandedRuleIndex = ruleIndex - 1;
      } else if (appState.expandedRuleIndex === ruleIndex - 1) {
        appState.expandedRuleIndex = ruleIndex;
      }
      break;
    case "move-rule-down":
      moveArrayItem(team.characters[characterIndex]?.rules, ruleIndex, ruleIndex + 1);
      if (appState.expandedRuleIndex === ruleIndex) {
        appState.expandedRuleIndex = ruleIndex + 1;
      } else if (appState.expandedRuleIndex === ruleIndex + 1) {
        appState.expandedRuleIndex = ruleIndex;
      }
      break;
    case "add-condition":
      if (team.characters[characterIndex]?.rules?.[ruleIndex]) {
        team.characters[characterIndex].rules[ruleIndex].when = [createEmptyCondition()];
      }
      break;
    case "remove-condition":
      if (team.characters[characterIndex]?.rules?.[ruleIndex]) {
        team.characters[characterIndex].rules[ruleIndex].when = [];
      }
      break;
    default:
      return;
  }

  syncTeamUI();
  if (navigateToCharacterTab) {
    setActiveWorkspace("character-builder");
  }
}

function downloadCharacterJson(characterIndex) {
  const team = appState.teamConfig;
  const character = team?.characters?.[characterIndex];
  if (!character) {
    renderTeamValidation({ ok: false, errors: ["No character is available to download."] });
    return;
  }

  const jsonText = JSON.stringify(character, null, 2);
  triggerJsonDownload(jsonText, buildCharacterFilename(character, `character_${characterIndex + 1}`));
  setTeamValidationStatus("success", `${character.display_name || character.id || `Character ${characterIndex + 1}`} downloaded`);
}

async function loadCharacterFromFileInput(fileInput) {
  const team = appState.teamConfig;
  const characterIndex = Number(fileInput.dataset.characterIndex);
  const [file] = fileInput.files ?? [];

  if (!team || !file) {
    return;
  }

  try {
    const content = await file.text();
    const parsedCharacter = JSON.parse(content);
    const validationErrors = validateCharacterConfig(parsedCharacter, "character");

    if (validationErrors.length > 0) {
      renderTeamValidation({ ok: false, errors: validationErrors });
      fileInput.value = "";
      return;
    }

    const existingPosition = team.characters[characterIndex]?.position ?? { row: 0, col: 0 };
    parsedCharacter.position = { row: existingPosition.row, col: existingPosition.col };
    team.characters[characterIndex] = parsedCharacter;
    syncTeamUI();
    setTeamValidationStatus("success", `Loaded into slot ${characterIndex + 1}`);
  } catch (error) {
    renderTeamValidation({ ok: false, errors: [`Could not load character JSON: ${error.message}`] });
  } finally {
    fileInput.value = "";
  }
}

function renderCharacterLibrary() {}

function cloneCharacterConfig(character) {
  return JSON.parse(JSON.stringify(character));
}

function addCharacterAtFirstOpenPosition(team) {
  const firstOpen = findFirstOpenSlotIndex(team);
  if (firstOpen < 0) {
    renderTeamValidation({ ok: false, errors: [`A team can have at most ${TEAM_SLOT_POSITIONS.length} characters.`] });
    return;
  }
  addCharacterAtSlot(team, firstOpen);
}

function addCharacterAtSlot(team, slotIndex) {
  if (team.characters.length >= TEAM_SLOT_POSITIONS.length) {
    renderTeamValidation({ ok: false, errors: [`A team can have at most ${TEAM_SLOT_POSITIONS.length} characters.`] });
    return;
  }

  if (!Number.isInteger(slotIndex) || slotIndex < 0 || slotIndex >= TEAM_SLOT_POSITIONS.length) {
    return;
  }

  if (team.characters[slotIndex]) {
    return;
  }

  team.characters.splice(slotIndex, 0, createEmptyCharacter(slotIndex));
  appState.selectedTeamCharacterIndex = slotIndex;
  appState.expandedRuleIndex = null;
}

function findFirstOpenSlotIndex(team) {
  for (let slotIndex = 0; slotIndex < TEAM_SLOT_POSITIONS.length; slotIndex += 1) {
    if (!team.characters[slotIndex]) {
      return slotIndex;
    }
  }
  return -1;
}

function findCharacterIndexAtPosition(team, row, col) {
  return team.characters.findIndex((character) => character.position?.row === row && character.position?.col === col);
}

function isWithinGrid(row, col) {
  return Number.isInteger(row) && Number.isInteger(col) && row >= 0 && row <= 2 && col >= 0 && col <= 2;
}

function moveCharacterToPosition(team, sourceIndex, row, col) {
  if (!isWithinGrid(row, col)) {
    return false;
  }

  const source = team.characters[sourceIndex];
  if (!source) {
    return false;
  }

  const targetIndex = findCharacterIndexAtPosition(team, row, col);
  const sourceRow = source.position?.row ?? 0;
  const sourceCol = source.position?.col ?? 0;

  if (sourceRow === row && sourceCol === col) {
    return false;
  }

  if (targetIndex >= 0 && targetIndex !== sourceIndex) {
    team.characters[targetIndex].position = { row: sourceRow, col: sourceCol };
  }

  source.position = { row, col };
  appState.selectedTeamCharacterIndex = sourceIndex;
  return true;
}

function triggerJsonDownload(jsonText, filename) {
  const blob = new Blob([jsonText], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
}

function buildCharacterFilename(character, fallbackName) {
  return `${slugifyFileStem(character.display_name || character.id || fallbackName)}.json`;
}

function slugifyFileStem(value) {
  const normalized = String(value)
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "");
  return normalized || "export";
}

function createEmptyCharacter(index, row = 0, col = 0) {
  const slotPosition = TEAM_SLOT_POSITIONS[index] ?? { row, col };
  const fallbackTemplateId = appState.catalogs.archetypeIds?.[0] ?? "the_emperor";
  const template = getArchetypeDefinition(fallbackTemplateId);
  return {
    id: `new_character_${index + 1}`,
    template_id: fallbackTemplateId,
    display_name: template?.display_name ?? "",
    position: { row: slotPosition.row, col: slotPosition.col },
    passive: template?.default_passive ?? "",
    actives: Array.isArray(template?.active_pool) ? template.active_pool.slice(0, 3) : [],
    aspect: null,
    rules: [],
  };
}

function applyTemplateToCharacter(character, templateId) {
  const template = getArchetypeDefinition(templateId);
  character.template_id = templateId;
  if (!template) {
    character.passive = "";
    character.actives = [];
    character.rules = [];
    character.display_name = "";
    return;
  }

  character.passive = template.default_passive ?? "";
  character.actives = Array.isArray(template.active_pool) ? template.active_pool.slice(0, 3) : [];
  character.rules = [];
  character.display_name = template.display_name ?? "";
}

function normalizeActiveSelections(actives) {
  const values = Array.isArray(actives) ? actives.slice(0, 3) : [];
  while (values.length < 3) {
    values.push("");
  }
  return values;
}

function applyBrowserSelection(target) {
  const team = appState.teamConfig;
  const character = team?.characters?.[appState.selectedTeamCharacterIndex];
  if (!character) {
    renderTeamValidation({ ok: false, errors: ["No character is selected for browser assignment."] });
    return;
  }

  const mode = target.dataset.browserMode ?? appState.teamBrowserMode;
  const slotIndex = Number(target.dataset.browserSlotIndex ?? appState.teamBrowserSlotIndex ?? 0);
  const value = target.dataset.entryValue ?? "";

  if (mode === "passive") {
    character.passive = value;
  } else if (mode === "aspect") {
    character.aspect = value || null;
  } else {
    const nextActives = normalizeActiveSelections(character.actives);
    nextActives[slotIndex] = value;
    character.actives = nextActives.filter(Boolean);
  }

  appState.teamDesignRightPane = "loadout";
  syncTeamUI();
}

function formatGridPosition(row, col) {
  if (!isWithinGrid(row, col)) {
    return "Unplaced";
  }
  const rowLabels = ["Front row", "Middle row", "Back row"];
  return `${rowLabels[row]} · col ${col + 1}`;
}

function getBrowserEntries(mode) {
  const character = appState.teamConfig?.characters?.[appState.selectedTeamCharacterIndex];
  const archetype = getArchetypeDefinition(character?.template_id);
  if (mode === "passive") {
    const pool = Array.isArray(archetype?.passive_pool) ? archetype.passive_pool : appState.catalogs.passives;
    return pool.map((name) => ({ name, description: getPassiveDescription(name) }));
  }
  if (mode === "aspect") {
    return appState.catalogs.aspects.map((name) => ({ name, description: getAspectDescription(name) }));
  }
  const pool = Array.isArray(archetype?.active_pool) ? archetype.active_pool : appState.catalogs.abilities;
  return pool.map((name) => ({ name, description: getAbilityDescription(name) }));
}

function buildSelectOptions(options, currentValue, emptyLabel) {
  const normalizedOptions = Array.isArray(options) ? [...options] : [];
  if (currentValue && !normalizedOptions.includes(currentValue)) {
    normalizedOptions.unshift(currentValue);
  }

  const entries = ["", ...normalizedOptions];
  return entries
    .map((optionValue, index) => {
      const label = optionValue || emptyLabel;
      const isSelected =
        optionValue === (currentValue ?? "") || (!currentValue && optionValue === "" && index === 0);
      return `<option value="${escapeHtml(optionValue)}" ${isSelected ? "selected" : ""}>${escapeHtml(label)}</option>`;
    })
    .join("");
}

function buildRequiredSelectOptions(options, currentValue) {
  const normalizedOptions = Array.isArray(options)
    ? [...new Set(options.filter((option) => typeof option === "string" && option.trim() !== ""))]
    : [];
  if (currentValue && !normalizedOptions.includes(currentValue)) {
    normalizedOptions.unshift(currentValue);
  }

  return normalizedOptions
    .map((optionValue) => `<option value="${escapeHtml(optionValue)}" ${optionValue === currentValue ? "selected" : ""}>${escapeHtml(optionValue)}</option>`)
    .join("");
}

function buildRuleStatusOptions(statusDefinitions) {
  const available = [];
  const hasBaseStatus = (name) => isPlainObject(statusDefinitions) && Object.hasOwn(statusDefinitions, name);

  if (hasBaseStatus("Omen")) {
    available.push("Omen");
  }
  if (hasBaseStatus("Restoration")) {
    available.push("Restoration");
  }
  if (hasBaseStatus("Ward")) {
    available.push("Ward");
  }
  if (hasBaseStatus("Empower")) {
    available.push("Empower:MGT", "Empower:MAG", "Empower:ARM", "Empower:RES");
  }
  if (hasBaseStatus("Weaken")) {
    available.push("Weaken:MGT", "Weaken:MAG", "Weaken:ARM", "Weaken:RES");
  }

  return available.length > 0 ? available : [...ruleStatusCatalog];
}

function buildArchetypeOptions(currentValue) {
  const options = [...(appState.catalogs.archetypeIds ?? [])];
  if (currentValue && !options.includes(currentValue)) {
    options.unshift(currentValue);
  }

  return ["", ...options].map((templateId, index) => {
    const definition = getArchetypeDefinition(templateId);
    const label = templateId ? (definition?.display_name ?? templateId) : "No archetype selected";
    const isSelected =
      templateId === (currentValue ?? "") || (!currentValue && templateId === "" && index === 0);
    return `<option value="${escapeHtml(templateId)}" ${isSelected ? "selected" : ""}>${escapeHtml(label)}</option>`;
  }).join("");
}

function createEmptyRule() {
  return {
    ability: "",
    when: [],
  };
}

function createEmptyCondition() {
  return {
    subject: "self",
    value: "hp",
    op: "gte",
    threshold: 1,
  };
}

function getAllowedRuleValueOptions(subject) {
  const optionValues = ruleValueOptionsBySubject[subject] ?? ruleValueOptionsBySubject.self;
  return optionValues
    .map((value) => ruleValueTypeOptions.find((option) => option.value === value))
    .filter(Boolean);
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

function moveArrayItem(array, fromIndex, toIndex) {
  if (!Array.isArray(array) || toIndex < 0 || toIndex >= array.length) {
    return;
  }
  const [item] = array.splice(fromIndex, 1);
  array.splice(toIndex, 0, item);
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

  const contextualLabel = getContextualRuleValueLabel(subject, valueType);
  return `${prefix}${contextualLabel} ${operatorLabel} ${threshold}`;
}

function formatRulePreview(rule) {
  const abilityLabel = rule?.ability || "an ability";
  const conditions = Array.isArray(rule?.when) ? rule.when : [];
  if (conditions.length === 0) {
    return `Use ${abilityLabel} always`;
  }

  return `Use ${abilityLabel} if ${conditions.map((condition) => formatConditionPreview(condition)).join(" and ")}`;
}

function getContextualRuleValueLabel(subject, valueType) {
  if (valueType === "self_row") {
    return "Row";
  }
  if (valueType === "self_companion_count" || valueType === "target_companion_count") {
    return "Companion Count";
  }
  return getRuleOptionLabel(ruleValueTypeOptions, valueType);
}

function getRuleOptionLabel(options, value) {
  return options.find((option) => option.value === value)?.label ?? String(value);
}

function isPlainObject(value) {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function shouldIgnoreGlobalKeydown(event) {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  const tagName = target.tagName;
  return target.isContentEditable || tagName === "INPUT" || tagName === "TEXTAREA" || tagName === "SELECT";
}

function clampValue(value, minValue, maxValue) {
  return Math.max(minValue, Math.min(maxValue, value));
}

function normalizeStatusEntries(statuses) {
  if (Array.isArray(statuses)) {
    return statuses
      .filter((entry) => isPlainObject(entry) && typeof entry.name === "string")
      .map((entry) => ({
        name: entry.name,
        stacks: Number(entry.stacks) || 0,
      }))
      .filter((entry) => entry.stacks > 0);
  }

  if (!isPlainObject(statuses)) {
    return [];
  }

  return Object.entries(statuses)
    .map(([name, stacks]) => ({
      name,
      stacks: Number(stacks) || 0,
    }))
    .filter((entry) => entry.stacks > 0);
}

function normalizeConditionEntries(conditions) {
  return normalizeStatusEntries(conditions);
}

function formatStatuses(statuses) {
  const entries = normalizeStatusEntries(statuses);
  if (entries.length === 0) {
    return "No statuses";
  }

  return entries.map(({ name, stacks }) => `${name} x${stacks}`).join(" • ");
}

function formatEffects(statuses, conditions) {
  const statusEntries = normalizeStatusEntries(statuses);
  const conditionEntries = normalizeConditionEntries(conditions);
  const entries = [...statusEntries, ...conditionEntries];
  if (entries.length === 0) {
    return "No effects";
  }

  return entries.map(({ name, stacks }) => `${name} x${stacks}`).join(" • ");
}

function getPassiveDescription(passiveName) {
  if (!passiveName) {
    return "";
  }

  return appState.catalogs.passiveDescriptions?.[passiveName] ?? "";
}

function getAbilityDescription(abilityName) {
  if (!abilityName) {
    return "";
  }

  return appState.catalogs.abilityDescriptions?.[abilityName] ?? "";
}

function getAbilityMpCost(abilityName) {
  if (!abilityName) {
    return null;
  }

  const abilityDefinition = appState.catalogs.abilityDefinitions?.[abilityName];
  const rawCost = abilityDefinition?.mp_cost;
  return Number.isFinite(rawCost) ? rawCost : null;
}

function getAspectDescription(aspectName) {
  if (!aspectName) {
    return "";
  }

  return appState.catalogs.aspectDescriptions?.[aspectName] ?? "";
}

function getAspectDisplayName(aspectName) {
  if (!aspectName) {
    return "";
  }

  const definition = getAspectDefinition(aspectName);
  if (definition?.display_name) {
    return definition.display_name;
  }

  return aspectName
    .split("_")
    .filter(Boolean)
    .map((segment) => segment[0].toUpperCase() + segment.slice(1))
    .join(" ");
}

function getAspectSummary(aspectName) {
  if (!aspectName) {
    return "";
  }

  const definition = getAspectDefinition(aspectName);
  if (!definition) {
    return getAspectDescription(aspectName);
  }

  const statSummary = formatAspectStatBonuses(definition.stat_bonuses);
  const passiveSummary = definition.passive ? getPassiveDescription(definition.passive) : "";
  const details = [statSummary, passiveSummary].filter(Boolean).join(" • ");
  return details || definition.description || "";
}

function renderAspectSummaryMarkup(aspectName) {
  if (!aspectName) {
    return escapeHtml("No description yet.");
  }

  const definition = getAspectDefinition(aspectName);
  if (!definition) {
    return escapeHtml(getAspectSummary(aspectName) || "No description yet.");
  }

  const statSummary = formatAspectStatBonuses(definition.stat_bonuses);
  const passiveSummary = definition.passive ? getPassiveDescription(definition.passive) : "";
  const lines = [statSummary, passiveSummary].filter(Boolean);
  if (lines.length === 0) {
    return escapeHtml(definition.description || "No description yet.");
  }

  return lines.map((line) => `<span class="aspect-summary-line">${escapeHtml(line)}</span>`).join("");
}

function formatAspectStatBonuses(statBonuses) {
  const bonuses = normalizeStatBlock(statBonuses);
  const entries = statFieldOptions
    .map((statKey) => {
      const amount = Number(bonuses?.[statKey] ?? 0);
      if (amount === 0) {
        return null;
      }
      const sign = amount > 0 ? "+" : "";
      return `${statKey.toUpperCase()} ${sign}${amount}`;
    })
    .filter(Boolean);
  return entries.length > 0 ? `Stats: ${entries.join(", ")}` : "";
}

function getArchetypeDefinition(templateId) {
  if (!templateId) {
    return null;
  }
  return appState.catalogs.archetypes?.[templateId] ?? null;
}

function getDerivedCharacterStats(character) {
  const archetype = getArchetypeDefinition(character?.template_id);
  const baseStats = normalizeStatBlock(archetype?.stats);
  const itemBonuses = normalizeStatBlock(getAspectDefinition(character?.aspect)?.stat_bonuses);
  const result = { ...baseStats };

  for (const statKey of statFieldOptions) {
    result[statKey] = Number(baseStats[statKey] ?? 0) + Number(itemBonuses[statKey] ?? 0);
  }

  return result;
}

function getAspectDefinition(aspectName) {
  if (!aspectName) {
    return null;
  }
  return appState.catalogs.aspectDefinitions?.[aspectName] ?? null;
}

function renderDerivedStatValue(baseValue, finalValue) {
  const bonus = finalValue - baseValue;
  if (bonus === 0) {
    return `${finalValue}`;
  }
  const sign = bonus > 0 ? "+" : "";
  return `${finalValue} (${sign}${bonus})`;
}

function getCharacterInitials(character) {
  const source = String(character.display_name || character.id || "?")
    .replace(/^the\s+/i, "")
    .trim();
  if (!source) {
    return "?";
  }

  return source
    .split(/\s+/)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase() ?? "")
    .join("");
}

function renderStatIcon(statKey) {
  const icons = {
    vit: '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 13s-4.5-2.7-4.5-6.1A2.4 2.4 0 0 1 8 5a2.4 2.4 0 0 1 4.5 1.9C12.5 10.3 8 13 8 13Z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/></svg>',
    mgt: '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="m9.9 2.1 4 4-1.4 1.4-.9-.9-2.2 2.2-1.4-1.4 2.2-2.2-.9-.9 1.4-1.4ZM6.8 8.5l.7.7-4.2 4.2H2.6v-.7l4.2-4.2Z" fill="currentColor"/><path d="m8.4 3.6 4 4" fill="none" stroke="currentColor" stroke-width="1.1" stroke-linecap="round"/></svg>',
    mag: '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="m8 2 1.2 3.1L12.5 6 9.9 8.1l.8 3.4L8 9.7l-2.7 1.8.8-3.4L3.5 6l3.3-.9L8 2Z" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round"/></svg>',
    arm: '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 2.2 12 3.7v3.1c0 2.5-1.5 4.7-4 6-2.5-1.3-4-3.5-4-6V3.7L8 2.2Z" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/></svg>',
    res: '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 2.2 12.8 5v6L8 13.8 3.2 11V5L8 2.2Z" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round"/><circle cx="8" cy="8" r="1.7" fill="none" stroke="currentColor" stroke-width="1.3"/></svg>',
    spd: '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 9.5h4L5.7 13l7.3-6.5H9L10.3 3 3 9.5Z" fill="currentColor"/></svg>',
    wil: '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 2.5c2 1.7 3 3.2 3 5 0 1.9-1.3 3.5-3 5-1.7-1.5-3-3.1-3-5 0-1.8 1-3.3 3-5Z" fill="none" stroke="currentColor" stroke-width="1.3"/><path d="M8 5.2c.8.8 1.1 1.5 1.1 2.2 0 .9-.4 1.6-1.1 2.4-.7-.8-1.1-1.5-1.1-2.4 0-.7.3-1.4 1.1-2.2Z" fill="currentColor"/></svg>',
  };
  return `<span class="stat-icon" aria-hidden="true">${icons[statKey] ?? ""}</span>`;
}

function renderTitleAttribute(text) {
  return text ? ` title="${escapeHtml(text)}"` : "";
}

function renderInspector(character) {
  if (!character) {
    inspectorPanel.innerHTML = '<div class="board-empty-state">click any character to inspect</div>';
    return;
  }

  const characterConfig = getReplayCharacterConfig(character.id);
  const baseStats = normalizeStatBlock(characterConfig?.stats ?? character.stats);
  const effectiveStats = normalizeStatBlock(calculateEffectiveStats(character));
  const effectsMarkup = renderInspectorEffects(character);
  const passiveName = characterConfig?.passive ?? character.passive;
  const focusLabel = character.current_target_id
    ? formatCharacterLabel(character.current_target_id, getReplayCharacterName(character.current_target_id))
    : "None";
  const rules = Array.isArray(characterConfig?.rules) ? characterConfig.rules : [];

  inspectorPanel.innerHTML = `
    <div class="inspector-header inspector-header-compact">
      <div class="inspector-title-stack">
        <div class="inspector-portrait">${escapeHtml(getCharacterInitials(character))}</div>
        <div>
          <h4>${escapeHtml(character.display_name || character.id)}</h4>
          <div class="inspector-subtle">${escapeHtml(character.team_key === "team_a" ? "your team" : "opponent")} · row ${character.position.col + 1} · ${formatDepthLabel(character.position.row)}</div>
        </div>
      </div>
    </div>
    <div class="unit-card-bars inspector-bars">
      ${renderBar("HP", character.current_hp, character.max_hp, "hp")}
      <div class="bar-row mana-row">
        <div class="bar-label"><span>MP</span><strong>${character.current_mp}/${character.max_mp}</strong></div>
        ${renderManaPips(character.current_mp, character.max_mp)}
      </div>
    </div>
    <section class="inspector-section">
      <div class="inspector-inline-detail"><strong>Passive:</strong> <span${renderTitleAttribute(getPassiveDescription(passiveName))}>${escapeHtml(passiveName || "No passive")}</span></div>
      <div class="inspector-inline-detail"><strong>Focus:</strong> <span>${escapeHtml(focusLabel)}</span></div>
    </section>
    <section class="inspector-section">
      <h5>Effective Stats</h5>
      <div class="inspector-stat-grid">
        ${renderEffectiveStats(baseStats, effectiveStats)}
      </div>
    </section>
    <section class="inspector-section">
      <h5>Status / Conditions</h5>
      ${effectsMarkup}
    </section>
    <section class="inspector-section">
      <h5>Rules</h5>
      <div class="replay-rule-list">${renderReplayRules(rules)}</div>
    </section>
  `;
}

function renderEffectiveStats(baseStats, effectiveStats) {
  const statOrder = ["vit", "mgt", "mag", "arm", "res", "spd"];
  return statOrder.map((statKey) => {
    const baseValue = Number(baseStats?.[statKey] ?? 0);
    const effectiveValue = Number(effectiveStats?.[statKey] ?? baseValue);
    const delta = effectiveValue - baseValue;
    const deltaText = delta === 0 ? "" : ` (${delta > 0 ? `+${delta}` : delta})`;
    const valueClass = delta > 0 ? "stat-value-buff" : delta < 0 ? "stat-value-nerf" : "";
    return `
      <div class="inspector-stat-row">
        <span class="inspector-stat-label">${statKey.toUpperCase()}</span>
        <span class="inspector-stat-value ${valueClass}">${effectiveValue}${deltaText}</span>
      </div>
    `;
  }).join("");
}

// Grouped buff / debuff / condition rows for the inspector, in the shared
// hybrid language (▲ buff / ▼ debuff, single status hue; conditions neutral).
function renderInspectorEffects(character) {
  const buffs = [];
  const debuffs = [];
  for (const { name, stacks } of normalizeStatusEntries(character.statuses)) {
    const polarity = statusPolarity(name);
    (polarity === "buff" ? buffs : debuffs).push(statusCardChip(name, stacks, polarity === "buff" ? "buff" : "debuff"));
  }
  const conditions = normalizeConditionEntries(character.conditions).map(({ name, stacks }) =>
    statusCardChip(name, stacks, "neutral"),
  );
  if (buffs.length === 0 && debuffs.length === 0 && conditions.length === 0) {
    return '<div class="inspector-effects-empty">None</div>';
  }
  const row = (label, chips) =>
    chips.length ? `<div class="inspector-effect-row"><span class="inspector-effect-label">${label}</span><div class="inspector-effect-chips">${chips.join("")}</div></div>` : "";
  return `<div class="inspector-effects">${row("Buffs", buffs)}${row("Debuffs", debuffs)}${row("Conditions", conditions)}</div>`;
}

function calculateEffectiveStats(character) {
  if (isPlainObject(character.effective_stats)) {
    return { ...character.effective_stats };
  }

  const baseStats = { ...(character.stats ?? {}) };
  const effectiveStats = { ...baseStats };

  for (const { name: status, stacks } of normalizeStatusEntries(character.statuses)) {
    if (status.startsWith("Empower:")) {
      applyStatusModifier(effectiveStats, status.slice("Empower:".length), stacks);
    } else if (status.startsWith("Fortify:")) {
      applyStatusModifier(effectiveStats, status.slice("Fortify:".length), stacks);
    } else if (status.startsWith("Weaken:")) {
      applyStatusModifier(effectiveStats, status.slice("Weaken:".length), -stacks);
    } else if (status.startsWith("Enfeeble:")) {
      applyStatusModifier(effectiveStats, status.slice("Enfeeble:".length), -stacks);
    }
  }

  return effectiveStats;
}

function normalizeStatBlock(stats) {
  if (!isPlainObject(stats)) {
    return {};
  }

  const normalized = {};
  for (const [key, value] of Object.entries(stats)) {
    normalized[String(key).toLowerCase()] = Number(value ?? 0);
  }
  return normalized;
}

function applyStatusModifier(stats, rawStatKey, delta) {
  const statKey = rawStatKey.toLowerCase();
  if (!(statKey in stats)) {
    return;
  }

  stats[statKey] += delta;
}

function formatEventType(type) {
  return type.replaceAll("_", " ");
}

function getReplayCharacterName(characterId) {
  if (!appState.replay || !characterId) {
    return null;
  }

  for (const team of Object.values(appState.replay.teams ?? {})) {
    const match = team.characters.find((character) => character.id === characterId);
    if (match) {
      return match.display_name || match.id || null;
    }
  }

  return null;
}

function getReplayCharacterConfig(characterId) {
  if (!appState.replay || !characterId) {
    return null;
  }

  for (const team of Object.values(appState.replay.teams ?? {})) {
    const match = team.characters?.find((character) => character.id === characterId);
    if (match) {
      return match;
    }
  }

  return null;
}

function formatCharacterLabel(characterId, fallbackName) {
  return fallbackName || getReplayCharacterName(characterId) || characterId || "Unknown";
}

// Team identity is encoded in the replay id prefix (team_a: / team_b:);
// fall back to the config team_key for older replays. team_a is "your team".
function teamKeyOf(characterId) {
  if (typeof characterId === "string") {
    if (characterId.startsWith("team_a:")) return "team_a";
    if (characterId.startsWith("team_b:")) return "team_b";
  }
  return getReplayCharacterConfig(characterId)?.team_key ?? null;
}

function sideOf(characterId) {
  const key = teamKeyOf(characterId);
  return key === "team_b" ? "enemy" : key === "team_a" ? "ally" : null;
}

function getReplayCharacterTeamClass(characterId) {
  const side = sideOf(characterId);
  return side ? `ent-${side}` : "";
}

// Identity token: name + a small side tag, colored ally (cool) / enemy (warm).
// The tag disambiguates duplicate arcana names across teams.
function formatCharacterLabelMarkup(characterId, fallbackName) {
  const side = sideOf(characterId);
  const tag = side === "ally" ? "A" : side === "enemy" ? "B" : "";
  const tagMarkup = tag ? `<span class="ent-tag">${tag}</span>` : "";
  return `<span class="ent ${getReplayCharacterTeamClass(characterId)}">${escapeHtml(
    formatCharacterLabel(characterId, fallbackName),
  )}${tagMarkup}</span>`;
}

function getEventActorId(event) {
  if (!event || !isPlainObject(event)) {
    return null;
  }
  return event.actor_id ?? event.source_id ?? event.character_id ?? event.reflector_id ?? null;
}

function getEventTargetId(event) {
  if (!event || !isPlainObject(event)) {
    return null;
  }
  return event.target_id ?? event.new_target_id ?? null;
}

function formatWinnerLabel(winner) {
  if (!appState.replay || !winner) {
    return winner || "no one";
  }

  const team = appState.replay.teams?.[winner];
  return team?.name || winner;
}

function damageMarkup(amount) {
  return `<span class="event-amount event-amount-damage">${escapeHtml(amount ?? "?")}</span>`;
}

function healMarkup(amount) {
  return `<span class="event-amount event-amount-heal">${escapeHtml(amount ?? "?")}</span>`;
}

function statusNameMarkup(name) {
  return `<span class="event-status">${escapeHtml(name ?? "a status")}</span>`;
}

function damageKindWord(event) {
  return event.damage_kind === "physical" || event.damage_kind === "magical"
    ? `${escapeHtml(event.damage_kind)} `
    : "";
}

function damageSourceSuffix(event) {
  return event.source_name && event.source_kind === "ability"
    ? ` with ${escapeHtml(event.source_name)}`
    : "";
}

// ===== Hybrid effect language: help/harm for HP, one status hue (▲ buff / ▼
// debuff), amber control, neutral resource/move. =====
const BUFF_STATUSES = new Set(["Empower", "Fortify", "Regen", "Ward", "Restoration", "Haste", "Barrier"]);
const DEBUFF_STATUSES = new Set(["Weaken", "Enfeeble", "Bleed", "Poison", "Omen", "Curse", "Slow"]);

function statusBaseName(name) {
  return String(name ?? "").split(":")[0];
}

// "buff" | "debuff" | "neutral"
function statusPolarity(name) {
  const base = statusBaseName(name);
  if (BUFF_STATUSES.has(base)) return "buff";
  if (DEBUFF_STATUSES.has(base)) return "debuff";
  return "neutral";
}

function statusChip(name, stacksAfter, { removed = false } = {}) {
  const polarity = statusPolarity(name);
  const arrow = removed ? "−" : polarity === "buff" ? "▲" : polarity === "debuff" ? "▼" : "•";
  const stacks = stacksAfter == null || removed ? "" : `<span class="chip-stacks">${escapeHtml(stacksAfter)}</span>`;
  return `<span class="fx-chip fx-status fx-status-${polarity} ${removed ? "is-removed" : ""}"><span class="chip-arrow">${arrow}</span>${escapeHtml(name)}${stacks}</span>`;
}

function amountChip(kind, amount, { lethal = false } = {}) {
  // kind: "harm" | "help" | "mp"
  const sign = kind === "harm" ? "−" : "+";
  const unit = kind === "mp" ? '<span class="chip-unit">MP</span>' : "";
  const skull = lethal ? '<span class="fx-lethal" title="lethal">✕</span>' : "";
  return `<span class="fx-chip fx-${kind}">${sign}${escapeHtml(amount ?? "?")}${unit}${skull}</span>`;
}

// Render a single in-beat effect event as a chip (used inline on the headline
// or on a passive sub-line). Returns "" for events with no chip representation.
function effectChip(event) {
  switch (event.type) {
    case "damage":
      return amountChip("harm", event.amount, { lethal: event.target_hp_after === 0 });
    case "heal":
    case "healing":
      return amountChip("help", event.amount);
    case "mp_restore":
      return amountChip("mp", event.amount);
    case "status_applied":
      return statusChip(event.status, event.stacks_after);
    case "status_removed":
      return statusChip(event.status, event.stacks_after, { removed: true });
    case "condition_applied":
      return statusChip(event.condition, event.stacks_after);
    case "status_tick":
      return amountChip(event.kind === "heal" ? "help" : "harm", event.amount, { lethal: event.target_hp_after === 0 });
    default:
      return "";
  }
}

// Group a segment's effect events by target so we render "→ Target  −5 ▼Omen"
// once per distinct target.
function groupEffectsByTarget(events) {
  const order = [];
  const byTarget = new Map();
  for (const event of events) {
    if (event.type === "defeat" || event.type === "retargeted" || event.type === "moved") {
      continue; // promoted to their own lines
    }
    const targetId = event.target_id ?? event.new_target_id ?? null;
    const key = targetId ?? "_";
    if (!byTarget.has(key)) {
      byTarget.set(key, { targetId, chips: [] });
      order.push(key);
    }
    const chip = effectChip(event);
    if (chip) {
      byTarget.get(key).chips.push(chip);
    }
  }
  return order.map((key) => byTarget.get(key)).filter((group) => group.chips.length > 0);
}

function targetEffectsMarkup(events, { showArrow = true } = {}) {
  const groups = groupEffectsByTarget(events);
  return groups
    .map((group) => {
      const target = group.targetId ? `${formatCharacterLabelMarkup(group.targetId)} ` : "";
      const arrow = showArrow && group.targetId ? '<span class="fx-arrow">→</span> ' : "";
      return `<span class="beat-target">${arrow}${target}${group.chips.join("")}</span>`;
    })
    .join("");
}

function actionVerbIcon(name) {
  return `<span class="beat-verb" aria-hidden="true">${name}</span>`;
}

// The headline for a turn beat: actor + what they did + inline result chips.
function renderBeatHead(beat) {
  const actor = formatCharacterLabelMarkup(beat.actorId);
  const action = beat.action;
  const actionEffects = beat.segments[0]?.events ?? [];

  if (!action) {
    // No chosen action (pure passive/ tick turn, or an unrecognized stream).
    const fx = targetEffectsMarkup(actionEffects);
    return `${actor} ${fx || '<span class="beat-muted">waits</span>'}`;
  }

  switch (action.type) {
    case "basic_attack": {
      const fx = targetEffectsMarkup(actionEffects);
      return `${actor} <span class="beat-verb beat-verb-attack">⚔</span> ${fx || '<span class="beat-muted">attacks</span>'}`;
    }
    case "ability_used": {
      const ability = `<span class="beat-ability">✦${escapeHtml(action.ability ?? "Ability")}</span>`;
      const fx = targetEffectsMarkup(actionEffects);
      return `${actor} ${ability}${fx ? ` ${fx}` : ""}`;
    }
    case "turn_skipped":
      return `${actor} <span class="fx-chip fx-control">⊘ ${escapeHtml(action.reason ?? "skipped")}</span>`;
    default:
      return actor;
  }
}

// Sub-lines: start-of-turn ticks, passive procs, retargets, moves, defeats.
function renderBeatSubLines(beat) {
  const lines = [];

  for (const tick of beat.preTicks ?? []) {
    lines.push(`<span class="beat-sub-icon">↳</span> ${statusNameMarkup(tick.status)} ${effectChip(tick)}`);
  }

  // Passive segments (segment[0] is the action segment, already in the head).
  for (const segment of (beat.segments ?? []).slice(1)) {
    if (segment.cause !== "passive") continue;
    const label = `<span class="beat-passive">⚡${escapeHtml(segment.passive ?? "Passive")}</span>`;
    const fx = targetEffectsMarkup(segment.events);
    lines.push(`<span class="beat-sub-icon">↳</span> ${label}${fx ? ` ${fx}` : ""}`);
  }

  // Retargets and moves anywhere in the beat.
  for (const segment of beat.segments ?? []) {
    for (const event of segment.events) {
      if (event.type === "retargeted") {
        const target = event.new_target_id
          ? formatCharacterLabelMarkup(event.new_target_id, event.new_target_name)
          : '<span class="beat-muted">no target</span>';
        lines.push(`<span class="fx-chip fx-control">⟲</span> ${formatCharacterLabelMarkup(event.actor_id ?? beat.actorId)} now targets ${target}`);
      } else if (event.type === "moved") {
        lines.push(`<span class="beat-sub-icon">↳</span> ${formatCharacterLabelMarkup(event.actor_id ?? beat.actorId)} <span class="beat-muted">repositions</span>`);
      }
    }
  }

  // Defeats are story-critical — strong treatment.
  for (const segment of beat.segments ?? []) {
    for (const event of segment.events) {
      if (event.type === "defeat") {
        lines.push(`<span class="beat-defeat"><span class="fx-lethal">✕</span> ${formatCharacterLabelMarkup(event.actor_id, event.actor_name)} defeated</span>`);
      }
    }
  }

  return lines.map((line) => `<div class="beat-sub">${line}</div>`).join("");
}

function renderSystemBeat(beat) {
  const isActive = beat.startIndex === appState.selectedEventIndex
    || (appState.selectedEventIndex < 0 && beat.type === "battle_start");
  let text;
  if (beat.type === "battle_start") {
    text = "Battle starts.";
  } else if (beat.type === "battle_end") {
    text = `Battle ends — <strong>${escapeHtml(formatWinnerLabel(beat.winner))}</strong> wins.`;
  } else {
    text = formatEventType(beat.type);
  }
  return `
    <button class="timeline-beat timeline-beat-system ${isActive ? "is-active" : ""}" type="button" data-jump-index="${beat.startIndex}">
      <span class="beat-system-text">${text}</span>
    </button>`;
}

function renderBeat(beat, focusId) {
  if (beat.kind === "system") {
    return renderSystemBeat(beat);
  }
  if (beat.kind === "loose") {
    const isActive = appState.selectedEventIndex >= beat.startIndex && appState.selectedEventIndex <= beat.endIndex;
    return `<button class="timeline-beat ${isActive ? "is-active" : ""}" type="button" data-jump-index="${beat.endIndex}"><div class="beat-head">${formatTimelineMarkup(beat.events[0])}</div></button>`;
  }

  const isActive = appState.selectedEventIndex >= beat.startIndex && appState.selectedEventIndex <= beat.endIndex;
  const side = sideOf(beat.actorId);
  const focusClass = focusId && beat.actorId === focusId ? "is-focus" : "";

  if (appState.logMode === "detailed") {
    const rows = beat.segments
      .flatMap((segment) => segment.events)
      .concat(beat.preTicks ?? [])
      .map((event) => `<div class="beat-detail-row">${formatTimelineMarkup(event)}</div>`)
      .join("");
    return `
      <button class="timeline-beat beat-${side ?? "neutral"} ${isActive ? "is-active" : ""} ${focusClass}" type="button" data-jump-index="${beat.endIndex}">
        <div class="beat-head">${renderBeatHead(beat)}</div>
        ${rows}
      </button>`;
  }

  return `
    <button class="timeline-beat beat-${side ?? "neutral"} ${isActive ? "is-active" : ""} ${focusClass}" type="button" data-jump-index="${beat.endIndex}">
      <div class="beat-head">${renderBeatHead(beat)}</div>
      ${renderBeatSubLines(beat)}
    </button>`;
}

// One-line narration for the board header — the active beat headline.
function narrateBeat(beat) {
  if (!beat) {
    return "";
  }
  if (beat.kind === "system") {
    return beat.type === "battle_end"
      ? `Battle ends — ${escapeHtml(formatWinnerLabel(beat.winner))} wins.`
      : "Battle starts.";
  }
  if (beat.kind === "loose") {
    return formatTimelineMarkup(beat.events[0]);
  }
  return renderBeatHead(beat);
}

function formatTimelineMarkup(event) {
  switch (event.type) {
    case "battle_start":
      return "Battle starts.";
    case "turn_start":
      return `${formatCharacterLabelMarkup(event.actor_id, event.actor_name)} begins a turn at ${escapeHtml(event.current_hp ?? "?")} HP and ${escapeHtml(event.current_mp ?? "?")} MP.`;
    case "basic_attack":
      return `${formatCharacterLabelMarkup(event.actor_id, event.actor_name)} attacks ${formatCharacterLabelMarkup(event.target_id, event.target_name)} with a basic hit and gains ${healMarkup(event.mp_restored)} MP.`;
    case "ability_used":
      return `${formatCharacterLabelMarkup(event.actor_id, event.actor_name)} uses ${escapeHtml(event.ability ?? "an ability")} for ${escapeHtml(event.mp_cost ?? "?")} MP.`;
    case "damage":
      return `${formatCharacterLabelMarkup(event.source_id)} deals ${damageMarkup(event.amount)} ${damageKindWord(event)}damage to ${formatCharacterLabelMarkup(event.target_id, event.target_name)}${damageSourceSuffix(event)}.`;
    case "heal":
    case "healing":
      return `${formatCharacterLabelMarkup(event.source_id)} restores ${healMarkup(event.amount)} HP to ${formatCharacterLabelMarkup(event.target_id, event.target_name)}${damageSourceSuffix(event)}.`;
    case "mp_restore":
      return `${formatCharacterLabelMarkup(event.source_id)} restores ${healMarkup(event.amount)} MP to ${formatCharacterLabelMarkup(event.target_id, event.target_name)}${damageSourceSuffix(event)}.`;
    case "status_applied":
      return `${formatCharacterLabelMarkup(event.target_id, event.target_name)} gains ${statusNameMarkup(event.status)} (${escapeHtml(event.stacks_after ?? "?")} stacks).`;
    case "condition_applied":
      return `${formatCharacterLabelMarkup(event.target_id, event.target_name)} gains ${statusNameMarkup(event.condition ?? "a condition")} (${escapeHtml(event.stacks_after ?? "?")} stacks).`;
    case "status_removed":
      return `${formatCharacterLabelMarkup(event.target_id, event.target_name)} loses ${statusNameMarkup(event.status)} (${escapeHtml(event.stacks_after ?? 0)} stacks remain).`;
    case "status_tick": {
      const tickAmount = event.kind === "heal" ? healMarkup(event.amount) : damageMarkup(event.amount);
      return `${formatCharacterLabelMarkup(event.target_id, event.target_name)} resolves ${statusNameMarkup(event.status)} for ${tickAmount} ${escapeHtml(event.kind ?? "effect")}.`;
    }
    case "passive_triggered":
      return `${formatCharacterLabelMarkup(event.actor_id, event.actor_name)} triggers ${escapeHtml(event.passive ?? "a passive")} on ${escapeHtml(event.trigger ?? "an event")}.`;
    case "turn_skipped":
      return `${formatCharacterLabelMarkup(event.actor_id, event.actor_name)} skips a turn because of ${escapeHtml(event.reason ?? "an effect")}.`;
    case "resource_changed":
      return `${formatCharacterLabelMarkup(event.actor_id, event.actor_name)} ${event.delta >= 0 ? "gains" : "spends"} ${escapeHtml(Math.abs(event.delta ?? 0))} ${escapeHtml(event.resource ?? "resource")}.`;
    case "retargeted":
      return `${formatCharacterLabelMarkup(event.actor_id ?? event.character_id, event.actor_name ?? event.character_name)} retargets to ${
        event.new_target_id || event.new_target_name
          ? formatCharacterLabelMarkup(event.new_target_id, event.new_target_name)
          : "no target"
      } (${escapeHtml(event.mode ?? "retarget")}).`;
    case "moved":
      return `${formatCharacterLabelMarkup(event.actor_id ?? event.character_id, event.actor_name ?? event.character_name)} moves to row ${escapeHtml(event.to_row ?? "?")}, col ${escapeHtml(event.to_col ?? "?")}.`;
    case "defeat":
      return `${formatCharacterLabelMarkup(event.actor_id, event.actor_name)} is defeated.`;
    case "battle_end":
      return `Battle ends with ${escapeHtml(formatWinnerLabel(event.winner))} winning.`;
    default:
      return escapeHtml(JSON.stringify(event));
  }
}

function getPrimaryEventCharacter(event) {
  return event?.actor_id ?? event?.source_id ?? event?.target_id ?? null;
}

function getSelectedCharacter(replayState) {
  if (!appState.selectedCharacterId) {
    return null;
  }

  for (const team of Object.values(replayState.teams)) {
    const match = team.characters.find((character) => character.id === appState.selectedCharacterId);
    if (match) {
      return match;
    }
  }

  return null;
}

function renderReplayRules(rules) {
  if (!Array.isArray(rules) || rules.length === 0) {
    return '<div class="board-empty-state board-empty-state-inline">Rules are not available in this replay.</div>';
  }

  return rules
    .map((rule, index) => `
      <div class="replay-rule-entry">
        <span class="replay-rule-index">${index + 1}.</span>
        <span class="replay-rule-text">${escapeHtml(formatRulePreview(rule))}</span>
      </div>
    `)
    .join("");
}

function formatDepthLabel(row) {
  if (row === 0) {
    return "front";
  }
  if (row === 1) {
    return "middle";
  }
  if (row === 2) {
    return "back";
  }
  return `row ${row}`;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}
