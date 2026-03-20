const tabButtons = document.querySelectorAll("[data-tab-target]");
const workspaces = document.querySelectorAll(".workspace");
const replayFileInput = document.querySelector("#replay-file-input");
const replayFileButton = document.querySelector("#replay-file-button");
const replayJsonInput = document.querySelector("#replay-json-input");
const replayDemoButton = document.querySelector("#replay-demo-button");
const replayValidationOutput = document.querySelector("#replay-validation-output");
const latestReplayPath = "./sample-data/latest_replay.json";
const archetypeCatalogPath = "../../battle_engine/src/data/archetypes.json";
const passiveCatalogPath = "../../battle_engine/src/data/passives.json";
const abilityCatalogPath = "../../battle_engine/src/data/abilities.json";
const itemCatalogPath = "../../battle_engine/src/data/items.json";
const statusCatalogPath = "../../battle_engine/src/data/statuses.json";
const conditionCatalog = ["Stunned", "Marked", "Severed"];
const ruleStatusCatalog = ["Omen", "Restoration", "Ward", "Empower:MGT", "Empower:MAG", "Empower:ARM", "Empower:RES", "Weaken:MGT", "Weaken:MAG", "Weaken:ARM", "Weaken:RES"];
const TEAM_SLOT_POSITIONS = [
  { row: 0, col: 0 },
  { row: 0, col: 2 },
  { row: 1, col: 1 },
  { row: 2, col: 1 },
];
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
  { value: "status_stacks", label: "Status Stacks" },
  { value: "condition_stacks", label: "Condition Stacks" },
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
const statFieldOptions = ["vit", "mgt", "mag", "arm", "res", "spd", "wil"];
const teamEditorConfig = {
  fileInput: document.querySelector("#team-file-input"),
  jsonInput: document.querySelector("#team-json-input"),
  loadButton: document.querySelector("#team-load-button"),
  copyButton: document.querySelector("#team-copy-button"),
  downloadButton: document.querySelector("#team-download-button"),
  validationOutput: document.querySelector("#team-validation-output"),
  editor: document.querySelector("#team-editor"),
};
const replayPreviousButton = document.querySelector("#replay-previous-button");
const replayPlayButton = document.querySelector("#replay-play-button");
const replayPauseButton = document.querySelector("#replay-pause-button");
const replayNextButton = document.querySelector("#replay-next-button");
const replayRestartButton = document.querySelector("#replay-restart-button");
const replayEventSlider = document.querySelector("#replay-event-slider");
const replayEventLabel = document.querySelector("#replay-event-label");
const replayTickDisplay = document.querySelector("#replay-tick-display");
const replaySpeedButtons = document.querySelectorAll(".speed-button");
const replayInlineActions = document.querySelector("#replay-inline-actions");
const currentEventTick = document.querySelector("#current-event-tick");
const currentEventIndex = document.querySelector("#current-event-index");
const currentEventText = document.querySelector("#current-event-text");
const timelineMajorOnlyInput = document.querySelector("#timeline-major-only");
const timelineSelectedOnlyInput = document.querySelector("#timeline-selected-only");
const timelineSelectedOnlyLabel = timelineSelectedOnlyInput.closest(".toggle-pill");
const timelineList = document.querySelector("#timeline-list");
const inspectorPanel = document.querySelector("#inspector-panel");
const battleBoard = document.querySelector("#battle-board");
const replaySidebarPanels = document.querySelectorAll("[data-replay-panel]");
const replaySidebarToggles = document.querySelectorAll("[data-replay-panel-toggle]");
const appState = {
  replay: null,
  selectedEventIndex: -1,
  selectedCharacterId: null,
  playbackTimerId: null,
  playbackSpeed: 1,
  teamConfig: null,
  characterLibrary: [],
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
    items: [],
    statuses: [],
    conditions: [...conditionCatalog],
    passiveDescriptions: {},
    abilityDescriptions: {},
    itemDescriptions: {},
    itemDefinitions: {},
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
        item: "vitality_charm",
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
        item: null,
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
              { subject: "self", value: "mp", op: "gte", threshold: 6 },
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
        item: null,
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
    const targetId = button.dataset.tabTarget;

    for (const workspace of workspaces) {
      workspace.classList.toggle("is-active", workspace.id === targetId);
    }

    for (const tabButton of tabButtons) {
      tabButton.classList.toggle("is-active", tabButton === button);
    }

    replayInlineActions?.classList.toggle("is-visible", targetId === "replay-viewer");
  });
}

replayInlineActions?.classList.remove("is-visible");

function setActiveReplaySidebarPanel(panelName) {
  for (const panel of replaySidebarPanels) {
    const isActive = panel.dataset.replayPanel === panelName;
    panel.classList.toggle("is-open", isActive);
    panel.classList.toggle("is-collapsed", !isActive);
  }

  for (const toggle of replaySidebarToggles) {
    const isActive = toggle.dataset.replayPanelToggle === panelName;
    toggle.setAttribute("aria-expanded", isActive ? "true" : "false");
  }

  if (panelName === "log") {
    window.requestAnimationFrame(() => {
      const selectedEvent = timelineList?.querySelector(".timeline-event.is-selected");
      selectedEvent?.scrollIntoView({ block: "center" });
    });
  }
}

for (const toggle of replaySidebarToggles) {
  toggle.addEventListener("click", () => {
    setActiveReplaySidebarPanel(toggle.dataset.replayPanelToggle);
  });
}

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
  void loadLatestReplay();
});

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

timelineMajorOnlyInput?.addEventListener("change", () => {
  renderTimeline();
});

timelineSelectedOnlyInput?.addEventListener("change", () => {
  renderTimeline();
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

teamEditorConfig.editor.addEventListener("input", (event) => {
  handleTeamEditorInput(event);
});

teamEditorConfig.editor.addEventListener("change", (event) => {
  void handleTeamEditorChange(event);
});

teamEditorConfig.editor.addEventListener("click", (event) => {
  handleTeamEditorAction(event);
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

async function loadEditorCatalogs() {
  try {
    const [archetypeResponse, passiveResponse, abilityResponse, itemResponse, statusResponse] = await Promise.all([
      fetch(archetypeCatalogPath, { cache: "no-store" }),
      fetch(passiveCatalogPath, { cache: "no-store" }),
      fetch(abilityCatalogPath, { cache: "no-store" }),
      fetch(itemCatalogPath, { cache: "no-store" }).catch(() => null),
      fetch(statusCatalogPath, { cache: "no-store" }).catch(() => null),
    ]);

    if (!archetypeResponse.ok || !passiveResponse.ok || !abilityResponse.ok) {
      throw new Error(
        `catalog request failed (${archetypeResponse.status}/${passiveResponse.status}/${abilityResponse.status})`,
      );
    }

    const [archetypes, passives, abilities, items, statuses] = await Promise.all([
      archetypeResponse.json(),
      passiveResponse.json(),
      abilityResponse.json(),
      itemResponse?.ok ? itemResponse.json() : Promise.resolve({}),
      statusResponse?.ok ? statusResponse.json() : Promise.resolve({}),
    ]);

    appState.catalogs.archetypes = archetypes;
    appState.catalogs.archetypeIds = Object.keys(archetypes).sort();
    appState.catalogs.passives = Object.keys(passives).sort();
    appState.catalogs.abilities = Object.keys(abilities).sort();
    appState.catalogs.items = Object.keys(items).sort();
    appState.catalogs.statuses = buildRuleStatusOptions(statuses);
    appState.catalogs.conditions = [...conditionCatalog];
    appState.catalogs.passiveDescriptions = Object.fromEntries(
      Object.entries(passives).map(([name, definition]) => [name, definition?.description ?? ""]),
    );
    appState.catalogs.abilityDescriptions = Object.fromEntries(
      Object.entries(abilities).map(([name, definition]) => [name, definition?.description ?? ""]),
    );
    appState.catalogs.itemDescriptions = Object.fromEntries(
      Object.entries(items).map(([name, definition]) => [name, definition?.description ?? ""]),
    );
    appState.catalogs.itemDefinitions = items;
    renderTeamEditor();
  } catch (_error) {
    appState.catalogs.archetypes = {};
    appState.catalogs.archetypeIds = [];
    appState.catalogs.passives = [];
    appState.catalogs.abilities = [];
    appState.catalogs.items = [];
    appState.catalogs.statuses = [...ruleStatusCatalog];
    appState.catalogs.conditions = [...conditionCatalog];
    appState.catalogs.passiveDescriptions = {};
    appState.catalogs.abilityDescriptions = {};
    appState.catalogs.itemDescriptions = {};
    appState.catalogs.itemDefinitions = {};
  }
}

async function loadLatestReplay() {
  try {
    const response = await fetch(latestReplayPath, { cache: "no-store" });
    if (!response.ok) {
      throw new Error(`Request failed with ${response.status}`);
    }

    const content = await response.text();
    replayJsonInput.value = content;
    loadReplayFromText(content.trim());
  } catch (error) {
    renderReplayValidation({
      ok: false,
      errors: [
        `Could not load latest replay from ${latestReplayPath}: ${error.message}. Run the engine to generate it, then try again.`,
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
    if (Array.isArray(candidate.events) && candidate.snapshots.length !== candidate.events.length + 1) {
      errors.push("`snapshots` must contain exactly one more entry than `events`.");
    }

    candidate.snapshots.forEach((snapshot, index) => {
      validateReplaySnapshot(snapshot, index, errors);
    });
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
    currentEventText.textContent = "Load a replay and move through events to see the current step here.";
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
  timelineSelectedOnlyInput.disabled = !appState.selectedCharacterId;
  timelineSelectedOnlyLabel?.classList.toggle("is-disabled", !appState.selectedCharacterId);
  if (!appState.selectedCharacterId) {
    timelineSelectedOnlyInput.checked = false;
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

  const entries = appState.replay.events
    .map((event, index) => ({ event, index }))
    .filter(({ event }) => shouldRenderTimelineEvent(event));

  if (entries.length === 0) {
    timelineList.innerHTML = '<div class="board-empty-state">No events match the active timeline filters.</div>';
    return;
  }

  let previousTick = null;
  const markup = entries.map(({ event, index }) => {
    const tickHeader = event.tick !== previousTick
      ? `<div class="timeline-tick-label">Tick ${event.tick}</div>`
      : "";
    previousTick = event.tick;

    return `
      <section class="timeline-tick-group">
        ${tickHeader}
        <button class="timeline-event ${index === appState.selectedEventIndex ? "is-selected" : ""}" type="button" data-event-index="${index}">
          <div class="timeline-event-meta">
            <span>${formatEventType(event.type)}</span>
            <span>#${index + 1}</span>
          </div>
          <p class="timeline-event-text">${formatTimelineMarkup(event)}</p>
        </button>
      </section>
    `;
  }).join("");

  timelineList.innerHTML = markup;
  bindTimelineEvents();
}

function bindTimelineEvents() {
  const eventButtons = timelineList.querySelectorAll("[data-event-index]");
  for (const button of eventButtons) {
    button.addEventListener("click", () => {
      const index = Number(button.dataset.eventIndex);
      setSelectedEventIndex(index);
    });
  }
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
  if (characterId) {
    setActiveReplaySidebarPanel("detail");
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

  const snapshotIndex = clampValue(selectedEventIndex + 1, 0, replay.snapshots.length - 1);
  const snapshot = replay.snapshots[snapshotIndex];
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
    currentEventText.textContent = "Battle state before the first logged event.";
    return;
  }

  const event = appState.replay.events[appState.selectedEventIndex];
  currentEventTick.textContent = `Tick ${event.tick ?? 0}`;
  currentEventIndex.textContent = `Step ${appState.selectedEventIndex + 1}`;
  currentEventText.innerHTML = formatTimelineMarkup(event);
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

function renderBattleBoard(container, replayState) {
  if (!container) {
    return;
  }

  const currentEvent = appState.replay && appState.selectedEventIndex >= 0
    ? appState.replay.events[appState.selectedEventIndex]
    : null;
  const currentEventActorId = getEventActorId(currentEvent);
  const currentEventTargetId = getEventTargetId(currentEvent);

  if (!replayState) {
    container.innerHTML = '<div class="board-empty-state">Load a replay to view the battle grid.</div>';
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

  return `
    <button class="grid-cell-button" type="button" data-character-id="${escapeHtml(character.id)}">
      <article class="unit-card unit-card-compact">
        <div class="unit-card-portrait">${escapeHtml(portraitGlyph)}</div>
        <h5 class="unit-card-name">${escapeHtml(character.display_name || character.id || "Unknown")}</h5>
        <div class="unit-card-bars unit-card-bars-compact">
          ${renderCompactBar("HP", hpValue, character.max_hp, "hp")}
          ${renderCompactBar("MP", mpValue, character.max_mp, "mp")}
        </div>
      </article>
    </button>
  `;
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
    </div>
  `;
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

  if (candidate.item != null && typeof candidate.item !== "string") {
    errors.push(`${prefix}.item must be a string or null.`);
  } else if (typeof candidate.item === "string" && candidate.item && !appState.catalogs.items.includes(candidate.item)) {
    errors.push(`${prefix}.item must reference a known item.`);
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

  if (!teamEditorConfig.editor?.contains(activeElement)) {
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
  if (!snapshot || !teamEditorConfig.editor) {
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

  const nextElement = teamEditorConfig.editor.querySelector(selectorParts.join(""));
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
  const editor = teamEditorConfig.editor;
  const team = appState.teamConfig;
  if (!team) {
    editor.innerHTML = '<div class="board-empty-state">Load a team to edit it here.</div>';
    return;
  }

  const selectedIndex = appState.selectedTeamCharacterIndex;
  const selectedCharacter = team.characters[selectedIndex];
  const characterSlots = renderCharacterSlots(team);

  editor.innerHTML = `
    <section class="team-builder-workspace">
      <div class="team-builder-topbar">
        <label class="field-group team-name-field team-name-field-compact">
          <input type="text" data-team-field="name" value="${escapeHtml(team.name)}">
        </label>
        ${characterSlots}
        <div class="file-icon-actions" aria-label="Team file actions">
          <button type="button" class="icon-button" data-team-action="open-team-file" title="Open team JSON" aria-label="Open team JSON">📂</button>
          <button type="button" class="icon-button" data-team-action="save-team-file" title="Save team JSON" aria-label="Save team JSON">💾</button>
        </div>
      </div>
      ${selectedCharacter ? renderSelectedCharacterWorkspace(selectedCharacter, selectedIndex) : '<div class="board-empty-state">Add a character to begin editing.</div>'}
    </section>
  `;
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
  const designRightPane = appState.teamDesignRightPane ?? "loadout";
  const archetype = getArchetypeDefinition(character.template_id);
  const derivedStats = getDerivedCharacterStats(character);
  return `
    <article class="builder-character-workspace">
      <div class="team-detail-tabbar" role="tablist" aria-label="Character editing tabs">
        <button type="button" class="team-detail-tab ${isDesignTab ? "is-active" : ""}" role="tab" aria-selected="${isDesignTab ? "true" : "false"}" data-team-action="select-detail-tab" data-detail-tab="design">Design</button>
        <button type="button" class="team-detail-tab ${!isDesignTab ? "is-active" : ""}" role="tab" aria-selected="${!isDesignTab ? "true" : "false"}" data-team-action="select-detail-tab" data-detail-tab="rules">Rules</button>
      </div>
      <input class="visually-hidden" type="file" accept=".json,application/json" data-team-action="load-character-file" data-character-index="${characterIndex}">
      ${
        isDesignTab
          ? `
            <section class="builder-pane builder-pane-stats">
              <div class="editor-card-actions editor-card-actions-wide">
                <button type="button" class="button-quiet" data-team-action="save-character" data-character-index="${characterIndex}">Save Character</button>
                <button type="button" class="button-quiet" data-team-action="load-character" data-character-index="${characterIndex}">Load Character</button>
                <button type="button" class="button-quiet" data-team-action="remove-character" data-character-index="${characterIndex}">Delete</button>
              </div>
              <div class="portrait-card">
                <div class="portrait-placeholder">${escapeHtml(getCharacterInitials(character))}</div>
                <div class="portrait-meta">
                  <div>${escapeHtml(character.display_name || `Character ${characterIndex + 1}`)}</div>
                  <div>${escapeHtml(formatGridPosition(character.position?.row, character.position?.col))}</div>
                </div>
              </div>
              <div class="editor-grid">
                <label class="field-group field-group-compact">
                  <span>Archetype</span>
                  <select data-character-field="template_id" data-character-index="${characterIndex}">
                    ${buildArchetypeOptions(character.template_id ?? "")}
                  </select>
                </label>
                <label class="field-group field-group-compact">
                  <input type="text" data-character-field="display_name" data-character-index="${characterIndex}" value="${escapeHtml(character.display_name ?? "")}">
                </label>
                <label class="field-group field-group-compact">
                  <span>Row</span>
                  <select data-position-field="row" data-character-index="${characterIndex}">
                    <option value="0" ${Number(character.position?.row ?? 0) === 0 ? "selected" : ""}>Front</option>
                    <option value="1" ${Number(character.position?.row ?? 0) === 1 ? "selected" : ""}>Middle</option>
                    <option value="2" ${Number(character.position?.row ?? 0) === 2 ? "selected" : ""}>Back</option>
                  </select>
                </label>
                <label class="field-group field-group-compact">
                  <span>Col</span>
                  <select data-position-field="col" data-character-index="${characterIndex}">
                    <option value="0" ${Number(character.position?.col ?? 0) === 0 ? "selected" : ""}>1</option>
                    <option value="1" ${Number(character.position?.col ?? 0) === 1 ? "selected" : ""}>2</option>
                    <option value="2" ${Number(character.position?.col ?? 0) === 2 ? "selected" : ""}>3</option>
                  </select>
                </label>
              </div>
              <div class="editor-inline-grid">
                ${["vit", "mgt", "mag", "arm", "res", "spd", "wil"].map((statKey) => `
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
            <section class="builder-pane builder-pane-right ${designRightPane === "browser" ? "selection-browser" : "builder-pane-loadout"}">
              ${designRightPane === "browser" ? renderSelectionBrowser(character) : renderLoadoutPane(character, characterIndex)}
            </section>
          `
          : renderRulesWorkspace(character, characterIndex)
      }
    </article>
  `;
}

function renderLoadoutPane(character, characterIndex) {
  return `
    ${renderLoadoutSlot("Passive", character.passive, "passive", characterIndex)}
    ${normalizeActiveSelections(character.actives).map((abilityName, activeIndex) =>
      renderLoadoutSlot(`Active ${activeIndex + 1}`, abilityName, "active", characterIndex, activeIndex)).join("")}
    ${renderLoadoutSlot("Item", character.item, "item", characterIndex)}
  `;
}

function renderLoadoutSlot(label, value, mode, characterIndex, slotIndex = null) {
  const isSelectedBrowser =
    appState.teamBrowserMode === mode &&
    appState.teamBrowserSlotIndex === (slotIndex ?? 0);
  const description =
    mode === "passive"
      ? getPassiveDescription(value)
      : mode === "item"
        ? getItemDescription(value)
        : getAbilityDescription(value);

  return `
    <button
      type="button"
      class="loadout-slot ${isSelectedBrowser ? "is-selected" : ""}"
      data-team-action="focus-browser"
      data-browser-mode="${mode}"
      data-browser-slot-index="${slotIndex ?? 0}"
      data-character-index="${characterIndex}"
    >
      <span class="loadout-slot-label">${label}</span>
      <span class="loadout-slot-value"${renderTitleAttribute(description)}>
        ${escapeHtml(value || `No ${label.toLowerCase()} selected`)}
      </span>
      <span class="loadout-slot-description">${escapeHtml(description || "No description yet.")}</span>
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
            <button type="button" class="button-quiet rule-icon-button" title="Move rule up" aria-label="Move rule up" data-team-action="move-rule-up" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">&uarr;</button>
            <button type="button" class="button-quiet rule-icon-button" title="Move rule down" aria-label="Move rule down" data-team-action="move-rule-down" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">&darr;</button>
            <button type="button" class="button-quiet rule-icon-button" title="Delete rule" aria-label="Delete rule" data-team-action="remove-rule" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">&#128465;</button>
          </div>
        </div>
      </article>
    `;
  }).join("");

  return `
    <div class="builder-pane-header">
      <div></div>
      <div class="editor-card-actions">
        <span class="rule-count-label">${ruleCount}/5</span>
        <button type="button" class="button-secondary" data-team-action="add-rule" data-character-index="${characterIndex}" ${canAddRule ? "" : "disabled"}>Add Rule</button>
      </div>
    </div>
    <div class="compact-rule-list">
      ${rulesMarkup || '<div class="board-empty-state">Add a priority rule to script this character. If none match, the character rests.</div>'}
    </div>
  `;
}

function renderRulesWorkspace(character, characterIndex) {
  const selectedRuleIndex = clampValue(appState.expandedRuleIndex ?? 0, 0, Math.max((character.rules?.length ?? 1) - 1, 0));
  const selectedRule = character.rules?.[selectedRuleIndex] ?? null;
  return `
    <section class="builder-pane builder-pane-rules">
      ${renderCompactRules(character, characterIndex)}
    </section>
    <section class="builder-pane builder-pane-rule-editor">
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
      : mode === "item"
        ? character.item ?? ""
        : normalizeActiveSelections(character.actives)[slotIndex] ?? "";
  const title =
    mode === "passive"
      ? "Passive Browser"
      : mode === "item"
        ? "Item Browser"
        : `Active ${slotIndex + 1} Browser`;

  return `
    <div class="builder-pane-header">
      <div>
        <p class="panel-kicker">Selection Browser</p>
        <h4>${title}</h4>
      </div>
      <div class="editor-card-actions">
        <span class="browser-current-label">${escapeHtml(currentValue || "Nothing selected")}</span>
      </div>
    </div>
    <div class="selection-browser-list">
      <button
        type="button"
        class="selection-browser-entry ${currentValue === "" || currentValue == null ? "is-selected" : ""}"
        data-team-action="select-browser-entry"
        data-browser-mode="${mode}"
        data-browser-slot-index="${slotIndex}"
        data-entry-value=""
      >
        <strong>Clear Selection</strong>
        <span>Remove the current ${mode === "active" ? `Active ${slotIndex + 1}` : mode}.</span>
      </button>
      ${
        entries.length === 0
          ? `<div class="board-empty-state">${mode === "item" ? "Items are not in the catalog yet." : "No entries are available for this browser."}</div>`
          : entries.map((entry) => renderBrowserEntry(entry, mode, slotIndex, currentValue)).join("")
      }
    </div>
  `;
}

function renderBrowserEntry(entry, mode, slotIndex, currentValue) {
  return `
    <button
      type="button"
      class="selection-browser-entry ${entry.name === currentValue ? "is-selected" : ""}"
      data-team-action="select-browser-entry"
      data-browser-mode="${mode}"
      data-browser-slot-index="${slotIndex}"
      data-entry-value="${escapeHtml(entry.name)}"
    >
      <strong>${escapeHtml(entry.name)}</strong>
      <span>${escapeHtml(entry.description || "No description yet.")}</span>
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
    equippedAbilityNames,
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
    } else if (target.dataset.characterField === "item") {
      character.item = target.value.trim() === "" ? null : target.value;
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

  handleTeamEditorInput(event);
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

  switch (action) {
    case "open-team-file":
      teamEditorConfig.fileInput.click();
      return;
    case "save-team-file":
      downloadTeamJson();
      return;
    case "add-character":
      addCharacterAtFirstOpenPosition(team);
      break;
    case "add-character-slot":
      addCharacterAtSlot(team, Number(actionTarget.dataset.slotIndex));
      break;
    case "select-detail-tab":
      appState.teamDetailTab = actionTarget.dataset.detailTab === "rules" ? "rules" : "design";
      if (appState.teamDetailTab === "design") {
        appState.teamDesignRightPane = "loadout";
      }
      break;
    case "select-character":
      appState.selectedTeamCharacterIndex = characterIndex;
      appState.expandedRuleIndex = null;
      appState.teamDetailTab = "design";
      appState.teamDesignRightPane = "loadout";
      break;
    case "focus-browser":
      appState.teamDetailTab = "design";
      appState.teamDesignRightPane = "browser";
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
    case "save-character":
      downloadCharacterJson(characterIndex);
      return;
    case "load-character":
      teamEditorConfig.editor
        .querySelector(`[data-team-action="load-character-file"][data-character-index="${characterIndex}"]`)
        ?.click();
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
    item: null,
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
  } else if (mode === "item") {
    character.item = value || null;
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
  if (mode === "item") {
    return appState.catalogs.items.map((name) => ({ name, description: getItemDescription(name) }));
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
  const prefix = subject === "world" ? "" : `${subjectLabel} `;

  if (valueType === "stat") {
    return `${prefix}${String(condition.value?.stat ?? "vit").toUpperCase()} ${operatorLabel} ${threshold}`;
  }

  if (valueType === "status_stacks") {
    return `${prefix}${condition.value?.status_stacks ?? "Empower:MGT"} Stacks ${operatorLabel} ${threshold}`;
  }

  if (valueType === "condition_stacks") {
    return `${prefix}${condition.value?.condition_stacks ?? "Stunned"} Stacks ${operatorLabel} ${threshold}`;
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

function getItemDescription(itemName) {
  if (!itemName) {
    return "";
  }

  return appState.catalogs.itemDescriptions?.[itemName] ?? "";
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
  const itemBonuses = normalizeStatBlock(getItemDefinition(character?.item)?.stat_bonuses);
  const result = { ...baseStats };

  for (const statKey of statFieldOptions) {
    result[statKey] = Number(baseStats[statKey] ?? 0) + Number(itemBonuses[statKey] ?? 0);
  }

  return result;
}

function getItemDefinition(itemName) {
  if (!itemName) {
    return null;
  }
  return appState.catalogs.itemDefinitions?.[itemName] ?? null;
}

function renderDerivedStatValue(baseValue, finalValue) {
  const bonus = finalValue - baseValue;
  if (bonus === 0) {
    return `${finalValue}`;
  }
  return `${finalValue} (+${bonus})`;
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
    mgt: '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M5 10.5c.8 1.2 2 2 3.7 2h1.8v-2.2l-1.3-.6-.4-1.5 1.2-1.7 2 .6.8 1.9v4H8.7c-2.4 0-4-1.1-4.9-2.9L5 10.5Zm5.5-6 .8-1.4 1.9.8-.8 1.5-1.9-.9Z" fill="currentColor"/></svg>',
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
  const statusMarkup = renderStatusList(character.statuses);
  const conditionMarkup = renderConditionList(character.conditions);
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
      ${renderBar("MP", character.current_mp, character.max_mp, "mp")}
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
      <div class="status-list">${statusMarkup}${conditionMarkup}</div>
    </section>
    <section class="inspector-section">
      <h5>Rules</h5>
      <div class="replay-rule-list">${renderReplayRules(rules)}</div>
    </section>
  `;
}

function renderEffectiveStats(baseStats, effectiveStats) {
  const statOrder = ["vit", "mgt", "mag", "arm", "res", "spd", "wil"];
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

function renderStatusList(statuses) {
  const entries = normalizeStatusEntries(statuses);
  if (entries.length === 0) {
    return "";
  }

  return entries.map(({ name, stacks }) => `<span>${escapeHtml(name)} x${stacks}</span>`).join("");
}

function renderConditionList(conditions) {
  const entries = normalizeConditionEntries(conditions);
  if (entries.length === 0) {
    return "";
  }

  return entries.map(({ name, stacks }) => `<span>${escapeHtml(name)} x${stacks}</span>`).join("");
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

function shouldRenderTimelineEvent(event) {
  if (timelineMajorOnlyInput.checked && !isMajorEvent(event.type)) {
    return false;
  }

  if (timelineSelectedOnlyInput.checked && appState.selectedCharacterId) {
    const eventCharacters = [
      event.actor_id,
      event.source_id,
      event.target_id,
      event.new_target_id,
    ].filter(Boolean);

    return eventCharacters.includes(appState.selectedCharacterId);
  }

  return true;
}

function isMajorEvent(type) {
  return [
    "ability_used",
    "rest",
    "basic_attack",
    "damage",
    "healing",
    "status_applied",
    "condition_applied",
    "status_removed",
    "status_tick",
    "passive_triggered",
    "turn_skipped",
    "retargeted",
    "moved",
    "defeat",
    "battle_end",
  ].includes(type);
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

function getReplayCharacterTeamClass(characterId) {
  const config = getReplayCharacterConfig(characterId);
  if (!config?.team_key) {
    return "";
  }
  return `event-character-${config.team_key}`;
}

function formatCharacterLabelMarkup(characterId, fallbackName) {
  return `<span class="event-character ${getReplayCharacterTeamClass(characterId)}">${escapeHtml(
    formatCharacterLabel(characterId, fallbackName),
  )}</span>`;
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

function formatTimelineText(event) {
  switch (event.type) {
    case "battle_start":
      return "Battle starts.";
    case "turn_start":
      return `${formatCharacterLabel(event.actor_id, event.actor_name)} begins a turn at ${event.current_hp ?? "?"} HP and ${event.current_mp ?? "?"} MP.`;
    case "rest":
      return `${formatCharacterLabel(event.actor_id, event.actor_name)} rests and restores ${event.mp_restored ?? "?"} MP.`;
    case "basic_attack":
      return `${formatCharacterLabel(event.actor_id, event.actor_name)} attacks ${formatCharacterLabel(event.target_id, event.target_name)} with a ${event.damage_kind ?? "basic"} hit.`;
    case "ability_used":
      return `${formatCharacterLabel(event.actor_id, event.actor_name)} uses ${event.ability ?? "an ability"} for ${event.mp_cost ?? "?"} MP.`;
    case "damage":
      return `${formatCharacterLabel(event.source_id, event.source_name)} deals ${event.amount ?? "?"} ${event.damage_kind ?? ""} damage to ${formatCharacterLabel(event.target_id, event.target_name)}.`;
    case "healing":
      return `${formatCharacterLabel(event.source_id, event.source_name)} restores ${event.amount ?? "?"} HP to ${formatCharacterLabel(event.target_id, event.target_name)}.`;
    case "mp_restore":
      return `${formatCharacterLabel(event.source_id, event.source_name)} restores ${event.amount ?? "?"} MP to ${formatCharacterLabel(event.target_id, event.target_name)}.`;
    case "status_applied":
      return `${formatCharacterLabel(event.target_id, event.target_name)} gains ${event.status ?? "a status"} (${event.stacks_after ?? "?"} stacks).`;
    case "condition_applied":
      return `${formatCharacterLabel(event.target_id, event.target_name)} gains ${event.condition ?? "a condition"} (${event.stacks_after ?? "?"} stacks).`;
    case "status_removed":
      return `${formatCharacterLabel(event.target_id, event.target_name)} loses ${event.status ?? "a status"} (${event.stacks_after ?? 0} stacks remain).`;
    case "status_tick":
      return `${formatCharacterLabel(event.target_id, event.target_name)} resolves ${event.status ?? "a status"} for ${event.amount ?? "?"} ${event.kind ?? "effect"}.`;
    case "passive_triggered":
      return `${formatCharacterLabel(event.actor_id, event.actor_name)} triggers ${event.passive ?? "a passive"} on ${event.trigger ?? "an event"}.`;
    case "turn_skipped":
      return `${formatCharacterLabel(event.actor_id, event.actor_name)} skips a turn because of ${event.reason ?? "an effect"}.`;
    case "resource_changed":
      return `${formatCharacterLabel(event.actor_id, event.actor_name)} ${event.delta >= 0 ? "gains" : "spends"} ${Math.abs(event.delta ?? 0)} ${event.resource ?? "resource"}.`;
    case "retargeted":
      return `${formatCharacterLabel(event.actor_id ?? event.character_id, event.actor_name ?? event.character_name)} retargets to ${
        event.new_target_id || event.new_target_name
          ? formatCharacterLabel(event.new_target_id, event.new_target_name)
          : "no target"
      } (${event.mode ?? "retarget"}).`;
    case "moved":
      return `${formatCharacterLabel(event.actor_id ?? event.character_id, event.actor_name ?? event.character_name)} moves to row ${event.to_row ?? "?"}, col ${event.to_col ?? "?"}.`;
    case "defeat":
      return `${formatCharacterLabel(event.actor_id, event.actor_name)} is defeated.`;
    case "battle_end":
      return `Battle ends with ${formatWinnerLabel(event.winner)} winning.`;
    default:
      return JSON.stringify(event);
  }
}

function formatTimelineMarkup(event) {
  switch (event.type) {
    case "battle_start":
      return "Battle starts.";
    case "turn_start":
      return `${formatCharacterLabelMarkup(event.actor_id, event.actor_name)} begins a turn at ${escapeHtml(event.current_hp ?? "?")} HP and ${escapeHtml(event.current_mp ?? "?")} MP.`;
    case "rest":
      return `${formatCharacterLabelMarkup(event.actor_id, event.actor_name)} rests and restores ${escapeHtml(event.mp_restored ?? "?")} MP.`;
    case "basic_attack":
      return `${formatCharacterLabelMarkup(event.actor_id, event.actor_name)} attacks ${formatCharacterLabelMarkup(event.target_id, event.target_name)} with a ${escapeHtml(event.damage_kind ?? "basic")} hit.`;
    case "ability_used":
      return `${formatCharacterLabelMarkup(event.actor_id, event.actor_name)} uses ${escapeHtml(event.ability ?? "an ability")} for ${escapeHtml(event.mp_cost ?? "?")} MP.`;
    case "damage":
      return `${formatCharacterLabelMarkup(event.source_id, event.source_name)} deals ${escapeHtml(event.amount ?? "?")} ${escapeHtml(event.damage_kind ?? "")} damage to ${formatCharacterLabelMarkup(event.target_id, event.target_name)}.`;
    case "healing":
      return `${formatCharacterLabelMarkup(event.source_id, event.source_name)} restores ${escapeHtml(event.amount ?? "?")} HP to ${formatCharacterLabelMarkup(event.target_id, event.target_name)}.`;
    case "mp_restore":
      return `${formatCharacterLabelMarkup(event.source_id, event.source_name)} restores ${escapeHtml(event.amount ?? "?")} MP to ${formatCharacterLabelMarkup(event.target_id, event.target_name)}.`;
    case "status_applied":
      return `${formatCharacterLabelMarkup(event.target_id, event.target_name)} gains ${escapeHtml(event.status ?? "a status")} (${escapeHtml(event.stacks_after ?? "?")} stacks).`;
    case "condition_applied":
      return `${formatCharacterLabelMarkup(event.target_id, event.target_name)} gains ${escapeHtml(event.condition ?? "a condition")} (${escapeHtml(event.stacks_after ?? "?")} stacks).`;
    case "status_removed":
      return `${formatCharacterLabelMarkup(event.target_id, event.target_name)} loses ${escapeHtml(event.status ?? "a status")} (${escapeHtml(event.stacks_after ?? 0)} stacks remain).`;
    case "status_tick":
      return `${formatCharacterLabelMarkup(event.target_id, event.target_name)} resolves ${escapeHtml(event.status ?? "a status")} for ${escapeHtml(event.amount ?? "?")} ${escapeHtml(event.kind ?? "effect")}.`;
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
