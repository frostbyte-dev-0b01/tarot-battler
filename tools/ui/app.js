const tabButtons = document.querySelectorAll("[data-tab-target]");
const workspaces = document.querySelectorAll(".workspace");
const replayFileInput = document.querySelector("#replay-file-input");
const replayFileButton = document.querySelector("#replay-file-button");
const replayJsonInput = document.querySelector("#replay-json-input");
const replayLoadButton = document.querySelector("#replay-load-button");
const replayDemoButton = document.querySelector("#replay-demo-button");
const replayValidationOutput = document.querySelector("#replay-validation-output");
const latestReplayPath = "./sample-data/latest_replay.json";
const passiveCatalogPath = "../../battle_engine/src/data/passives.json";
const abilityCatalogPath = "../../battle_engine/src/data/abilities.json";
const itemCatalogPath = "../../battle_engine/src/data/items.json";
const ruleSubjectOptions = [
  { value: "self", label: "Self" },
  { value: "target", label: "Target" },
  { value: "companion", label: "Any Companion" },
  { value: "world", label: "World" },
];
const ruleValueTypeOptions = [
  { value: "hp", label: "HP" },
  { value: "mp", label: "MP" },
  { value: "self_row", label: "Row" },
  { value: "self_companion_count", label: "Own Companions" },
  { value: "target_companion_count", label: "Target Companions" },
  { value: "use_count", label: "Uses" },
  { value: "turns_since_use", label: "Turns Since Use" },
  { value: "tick_count", label: "Tick Count" },
  { value: "ally_count", label: "Allies Alive" },
  { value: "enemy_count", label: "Enemies Alive" },
  { value: "stat", label: "Stat" },
  { value: "has_status", label: "Has Status" },
  { value: "status_stacks", label: "Status Stacks" },
];
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
const characterLibraryShell = document.querySelector("#character-library");
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
const timelineMajorOnlyInput = document.querySelector("#timeline-major-only");
const timelineSelectedOnlyInput = document.querySelector("#timeline-selected-only");
const timelineSelectedOnlyLabel = timelineSelectedOnlyInput.closest(".toggle-pill");
const timelineList = document.querySelector("#timeline-list");
const inspectorPanel = document.querySelector("#inspector-panel");
const battleBoard = document.querySelector("#battle-board");
const appState = {
  replay: null,
  selectedEventIndex: -1,
  selectedCharacterId: null,
  playbackTimerId: null,
  playbackSpeed: 1,
  dragPreviewElement: null,
  teamConfig: null,
  characterLibrary: [],
  selectedTeamCharacterIndex: 0,
  teamBrowserMode: "active",
  teamBrowserSlotIndex: 0,
  expandedRuleIndex: null,
  catalogs: {
    passives: [],
    abilities: [],
    items: [],
    passiveDescriptions: {},
    abilityDescriptions: {},
    itemDescriptions: {},
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
  version: 1,
  name: "Imperial Phalanx",
  characters: [
      {
        id: "the_emperor",
        display_name: "The Emperor",
        position: { row: 0, col: 0 },
        stats: { vit: 12, mgt: 12, mag: 8, arm: 7, res: 5, spd: 8, wil: 12 },
        passive: "Imperial Formation",
        actives: ["Hold the Line", "Command", "Taunt"],
        item: null,
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
        display_name: "The Hierophant",
        position: { row: 0, col: 2 },
        stats: { vit: 14, mgt: 8, mag: 12, arm: 5, res: 8, spd: 8, wil: 14 },
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
        display_name: "The Chariot",
        position: { row: 1, col: 1 },
        stats: { vit: 10, mgt: 15, mag: 8, arm: 5, res: 4, spd: 14, wil: 10 },
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
  });
}

replayLoadButton.addEventListener("click", () => {
  const sourceText = replayJsonInput.value.trim();

  if (!sourceText) {
    renderReplayValidation({
      ok: false,
      errors: ["Replay JSON input is empty."],
    });
    resetMetadata();
    return;
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
    } else {
      appState.replay = null;
      appState.selectedEventIndex = -1;
      appState.selectedCharacterId = null;
      stopPlayback();
      resetMetadata();
      resetBoards();
      renderPlaybackControls();
      renderInspector(null);
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
  }
});

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
    replayLoadButton.click();
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

teamEditorConfig.loadButton.addEventListener("click", () => {
  loadTeamFromText(teamEditorConfig.jsonInput.value.trim());
});

teamEditorConfig.fileInput.addEventListener("change", async (event) => {
  const [file] = event.target.files ?? [];
  if (!file) {
    return;
  }

  try {
    const content = await file.text();
    teamEditorConfig.jsonInput.value = content;
    loadTeamFromText(content);
  } catch (error) {
    renderTeamValidation({
      ok: false,
      errors: [`Could not read team file: ${error.message}`],
    });
    resetTeamSummary();
  }
});

teamEditorConfig.copyButton.addEventListener("click", async () => {
  await copyTeamJson();
});

teamEditorConfig.downloadButton.addEventListener("click", () => {
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

teamEditorConfig.editor.addEventListener("dragstart", (event) => {
  handleTeamEditorDragStart(event);
});

teamEditorConfig.editor.addEventListener("dragover", (event) => {
  handleTeamEditorDragOver(event);
});

teamEditorConfig.editor.addEventListener("drop", (event) => {
  handleTeamEditorDrop(event);
});

characterLibraryShell?.addEventListener("click", (event) => {
  handleTeamEditorAction(event);
});

resetMetadata();
resetBoards();
renderPlaybackControls();
renderTimeline();
renderInspector(null);
appState.teamConfig = structuredClone(demoTeam);
teamEditorConfig.jsonInput.value = JSON.stringify(appState.teamConfig, null, 2);
resetTeamSummary();
renderTeamEditor();
renderCharacterLibrary();
renderTeamValidation(validateTeamConfig(appState.teamConfig));
void loadEditorCatalogs();
void loadLatestReplay();

async function loadEditorCatalogs() {
  try {
    const [passiveResponse, abilityResponse, itemResponse] = await Promise.all([
      fetch(passiveCatalogPath, { cache: "no-store" }),
      fetch(abilityCatalogPath, { cache: "no-store" }),
      fetch(itemCatalogPath, { cache: "no-store" }).catch(() => null),
    ]);

    if (!passiveResponse.ok || !abilityResponse.ok) {
      throw new Error(
        `catalog request failed (${passiveResponse.status}/${abilityResponse.status})`,
      );
    }

    const [passives, abilities, items] = await Promise.all([
      passiveResponse.json(),
      abilityResponse.json(),
      itemResponse?.ok ? itemResponse.json() : Promise.resolve({}),
    ]);

    appState.catalogs.passives = Object.keys(passives).sort();
    appState.catalogs.abilities = Object.keys(abilities).sort();
    appState.catalogs.items = Object.keys(items).sort();
    appState.catalogs.passiveDescriptions = Object.fromEntries(
      Object.entries(passives).map(([name, definition]) => [name, definition?.description ?? ""]),
    );
    appState.catalogs.abilityDescriptions = Object.fromEntries(
      Object.entries(abilities).map(([name, definition]) => [name, definition?.description ?? ""]),
    );
    appState.catalogs.itemDescriptions = Object.fromEntries(
      Object.entries(items).map(([name, definition]) => [name, definition?.description ?? ""]),
    );
    renderTeamEditor();
  } catch (_error) {
    appState.catalogs.passives = [];
    appState.catalogs.abilities = [];
    appState.catalogs.items = [];
    appState.catalogs.passiveDescriptions = {};
    appState.catalogs.abilityDescriptions = {};
    appState.catalogs.itemDescriptions = {};
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
    replayLoadButton.click();
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
  metadataFields.seed.textContent = String(replay.seed);
  metadataFields.winner.textContent = replay.winner;
  metadataFields.tick_count.textContent = String(replay.tick_count);
  metadataFields.team_a.textContent = replay.teams.team_a.name;
  metadataFields.team_b.textContent = replay.teams.team_b.name;
}

function resetMetadata() {
  for (const field of Object.values(metadataFields)) {
    field.textContent = "-";
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
          <p class="timeline-event-text">${escapeHtml(formatTimelineText(event))}</p>
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
      setSelectedCharacterId(getPrimaryEventCharacter(appState.replay.events[index]));
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
  currentEventText.textContent = formatTimelineText(event);
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

  const rowLabels = Array.from({ length: 4 }, (_, rowIndex) => `
    <div class="arena-row-label arena-row-label-${rowIndex}">row ${rowIndex + 1}</div>
  `).join("");

  const cellsMarkup = Array.from({ length: 4 }, (_, colIndex) => {
    return Array.from({ length: 7 }, (_, depthIndex) => {
      const isGap = depthIndex === 3;
      const character = occupantMap.get(`${colIndex}:${depthIndex}`);
      const isSelected = character && character.id === appState.selectedCharacterId;
      const isSource = character && currentEvent && [currentEvent.actor_id, currentEvent.source_id].includes(character.id);
      const isTarget = character && currentEvent && currentEvent.target_id === character.id;
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

  container.innerHTML = `${rowLabels}${cellsMarkup}`;
  bindBoardSelection(container);
}

function isReplayPosition(position) {
  return isPlainObject(position)
    && Number.isInteger(position.row)
    && Number.isInteger(position.col)
    && position.row >= 0
    && position.row < 3
    && position.col >= 0
    && position.col < 4;
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

  if (typeof candidate.version !== "number") {
    errors.push("`version` must be a number.");
  }

  if (typeof candidate.name !== "string" || candidate.name.trim() === "") {
    errors.push("`name` must be a non-empty string.");
  }

  if (!Array.isArray(candidate.characters) || candidate.characters.length < 1) {
    errors.push("`characters` must be an array with at least 1 character.");
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
  const requiredStats = ["vit", "mgt", "mag", "arm", "res", "spd", "wil"];

  if (!isPlainObject(candidate)) {
    return [`${prefix} must be an object.`];
  }

  if (typeof candidate.id !== "string" || candidate.id.trim() === "") {
    errors.push(`${prefix}.id must be a non-empty string.`);
  }

  if (!isPlainObject(candidate.position)) {
    errors.push(`${prefix}.position must be an object.`);
  } else {
    const { row, col } = candidate.position;
    if (!Number.isInteger(row) || row < 0 || row > 2) {
      errors.push(`${prefix}.position.row must be an integer from 0 to 2.`);
    }
    if (!Number.isInteger(col) || col < 0 || col > 3) {
      errors.push(`${prefix}.position.col must be an integer from 0 to 3.`);
    }
  }

  if (!isPlainObject(candidate.stats)) {
    errors.push(`${prefix}.stats must be an object.`);
  } else {
    for (const statKey of requiredStats) {
      if (typeof candidate.stats[statKey] !== "number") {
        errors.push(`${prefix}.stats.${statKey} must be a number.`);
      }
    }
  }

  if (typeof candidate.passive !== "string") {
    errors.push(`${prefix}.passive must be a string.`);
  }

  if (!Array.isArray(candidate.actives)) {
    errors.push(`${prefix}.actives must be an array.`);
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
  if (!teamConfig) {
    resetTeamSummary();
    appState.selectedTeamCharacterIndex = 0;
    appState.expandedRuleIndex = null;
    renderTeamEditor();
    renderCharacterLibrary();
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
  teamEditorConfig.jsonInput.value = JSON.stringify(teamConfig, null, 2);
  renderTeamSummary(teamConfig);
  renderTeamValidation(validateTeamConfig(teamConfig));
  renderTeamEditor();
  renderCharacterLibrary();
}

function renderTeamValidation(result) {
  const output = teamEditorConfig.validationOutput;
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
  const characterGrid = renderCharacterGrid(team);

  editor.innerHTML = `
    <section class="team-builder-workspace">
      <div class="team-builder-topbar">
        <label class="field-group team-name-field team-name-field-compact">
          <input type="text" data-team-field="name" value="${escapeHtml(team.name)}">
        </label>
        <div class="file-icon-actions" aria-label="Team file actions">
          <button type="button" class="icon-button" data-team-action="open-team-file" title="Open team JSON" aria-label="Open team JSON">📂</button>
          <button type="button" class="icon-button" data-team-action="save-team-file" title="Save team JSON" aria-label="Save team JSON">💾</button>
        </div>
      </div>
      ${characterGrid}
      ${selectedCharacter ? renderSelectedCharacterWorkspace(selectedCharacter, selectedIndex) : '<div class="board-empty-state">Add a character to begin editing.</div>'}
    </section>
  `;
}

function renderCharacterGrid(team) {
  const occupantMap = new Map();
  team.characters.forEach((character, characterIndex) => {
    const row = character.position?.row;
    const col = character.position?.col;
    if (Number.isInteger(row) && Number.isInteger(col)) {
      occupantMap.set(`${row}:${col}`, { character, characterIndex });
    }
  });

  const canAddCharacter = team.characters.length < 5;
  const cells = [];
  for (let row = 0; row < 3; row += 1) {
    for (let col = 0; col < 4; col += 1) {
      const occupant = occupantMap.get(`${row}:${col}`);
      cells.push(renderCharacterGridCell(occupant, row, col, canAddCharacter));
    }
  }

  return `
    <div class="character-grid" role="grid" aria-label="Team formation grid">
      ${cells.join("")}
    </div>
  `;
}

function renderCharacterGridCell(occupant, row, col, canAddCharacter) {
  if (occupant) {
    const { character, characterIndex } = occupant;
    const isSelected = characterIndex === appState.selectedTeamCharacterIndex;
    return `
      <div class="character-grid-cell is-occupied ${isSelected ? "is-selected" : ""}" data-team-grid-cell="true" data-grid-row="${row}" data-grid-col="${col}">
        <button
          type="button"
          class="character-grid-card ${isSelected ? "is-selected" : ""}"
          data-team-action="select-character"
          data-character-index="${characterIndex}"
          data-team-drag-character-index="${characterIndex}"
          draggable="true"
          role="gridcell"
          aria-selected="${isSelected ? "true" : "false"}"
        >
          <span class="character-grid-name">${escapeHtml(character.display_name || character.id || `Character ${characterIndex + 1}`)}</span>
        </button>
      </div>
    `;
  }

  return `
    <div class="character-grid-cell" data-team-grid-cell="true" data-grid-row="${row}" data-grid-col="${col}">
      <button
        type="button"
        class="character-grid-add ${canAddCharacter ? "" : "is-disabled"}"
        data-team-action="add-character-at-position"
        data-grid-row="${row}"
        data-grid-col="${col}"
        ${canAddCharacter ? "" : "disabled"}
      >+</button>
    </div>
  `;
}

function renderSelectedCharacterWorkspace(character, characterIndex) {
  return `
    <article class="builder-character-workspace">
      <section class="builder-pane builder-pane-stats">
        <div class="editor-card-actions editor-card-actions-wide">
          <button type="button" class="button-quiet" data-team-action="save-character-to-library" data-character-index="${characterIndex}">Save to Library</button>
          <button type="button" class="button-quiet" data-team-action="save-character" data-character-index="${characterIndex}">Save Character</button>
          <button type="button" class="button-quiet" data-team-action="load-character" data-character-index="${characterIndex}">Load Character</button>
          <button type="button" class="button-quiet" data-team-action="remove-character" data-character-index="${characterIndex}">Delete</button>
        </div>
        <input class="visually-hidden" type="file" accept=".json,application/json" data-team-action="load-character-file" data-character-index="${characterIndex}">
        <div class="portrait-card">
          <div class="portrait-placeholder">${escapeHtml(getCharacterInitials(character))}</div>
          <div class="portrait-meta">
            <div>${escapeHtml(character.display_name || `Character ${characterIndex + 1}`)}</div>
            <div>${escapeHtml(formatGridPosition(character.position?.row, character.position?.col))}</div>
          </div>
        </div>
        <div class="editor-grid">
          <label class="field-group field-group-compact">
            <input type="text" data-character-field="display_name" data-character-index="${characterIndex}" value="${escapeHtml(character.display_name ?? "")}">
          </label>
        </div>
        <div class="editor-inline-grid">
          ${["vit", "mgt", "mag", "arm", "res", "spd", "wil"].map((statKey) => `
            <label class="field-group">
              <span>${statKey.toUpperCase()}</span>
              <input type="number" data-stat-field="${statKey}" data-character-index="${characterIndex}" value="${character.stats?.[statKey] ?? 0}">
            </label>
          `).join("")}
        </div>
      </section>
      <section class="builder-pane builder-pane-rules">
        ${renderCompactRules(character, characterIndex)}
      </section>
      <section class="builder-pane builder-pane-loadout">
        <div class="builder-pane-header">
          <div>
            <p class="panel-kicker">Loadout</p>
            <h4>Passive, Actives, Item</h4>
          </div>
        </div>
        ${renderLoadoutSlot("Passive", character.passive, "passive", characterIndex)}
        ${normalizeActiveSelections(character.actives).map((abilityName, activeIndex) =>
          renderLoadoutSlot(`Active ${activeIndex + 1}`, abilityName, "active", characterIndex, activeIndex)).join("")}
        ${renderLoadoutSlot("Item", character.item, "item", characterIndex)}
      </section>
      ${renderSelectionBrowser(character)}
    </article>
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
      <div class="loadout-slot-header">
        <span class="editor-subsection-label">${label}</span>
      </div>
      <div class="loadout-slot-value"${renderTitleAttribute(description)}>
        ${escapeHtml(value || `No ${label.toLowerCase()} selected`)}
      </div>
    </button>
  `;
}

function renderCompactRules(character, characterIndex) {
  const rules = character.rules ?? [];
  const ruleCount = rules.length;
  const canAddRule = ruleCount < 5;
  const rulesMarkup = rules.map((rule, ruleIndex) => {
    const isExpanded = appState.expandedRuleIndex === ruleIndex;
    return `
      <article class="compact-rule-card ${isExpanded ? "is-expanded" : ""}">
        <div class="compact-rule-header">
          <div>
            <div class="compact-rule-index">Priority ${ruleIndex + 1}</div>
            <div class="compact-rule-text">${escapeHtml(formatRulePreview(rule))}</div>
          </div>
          <div class="rule-action-row">
            <button type="button" class="button-quiet rule-icon-button" title="Move rule up" aria-label="Move rule up" data-team-action="move-rule-up" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">&uarr;</button>
            <button type="button" class="button-quiet rule-icon-button" title="Move rule down" aria-label="Move rule down" data-team-action="move-rule-down" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">&darr;</button>
            <button type="button" class="button-quiet rule-icon-button" title="${isExpanded ? "Finish editing rule" : "Edit rule"}" aria-label="${isExpanded ? "Finish editing rule" : "Edit rule"}" data-team-action="toggle-rule-edit" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">&#9998;</button>
            <button type="button" class="button-quiet rule-icon-button" title="Delete rule" aria-label="Delete rule" data-team-action="remove-rule" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">&#128465;</button>
          </div>
        </div>
        ${isExpanded ? renderRuleEditor(characterIndex, rule, ruleIndex) : ""}
      </article>
    `;
  }).join("");

  return `
    <div class="builder-pane-header">
      <div>
        <p class="panel-kicker">Rules</p>
        <h4>Priority Rules</h4>
      </div>
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
    <section class="builder-pane selection-browser">
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
    </section>
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
  const abilityOptions = buildSelectOptions(
    appState.catalogs.abilities,
    rule.ability ?? "",
    "No ability selected",
  );
  const conditionsMarkup = (rule.when ?? []).map((condition, conditionIndex) => renderConditionEditor(characterIndex, ruleIndex, condition, conditionIndex)).join("");

  return `
    <article class="editor-card">
      <div class="editor-card-header">
        <div>
          <h6>Priority ${ruleIndex + 1}</h6>
          <div class="rule-preview">${escapeHtml(formatRulePreview(rule))}</div>
        </div>
        <div class="editor-card-actions">
          <button type="button" class="button-quiet" data-team-action="move-rule-up" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">Up</button>
          <button type="button" class="button-quiet" data-team-action="move-rule-down" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">Down</button>
          <button type="button" class="button-quiet" data-team-action="remove-rule" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">Remove</button>
        </div>
      </div>
      <label class="field-group">
        <span>Ability</span>
        <select data-rule-field="ability" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">
          ${abilityOptions}
        </select>
      </label>
      <div class="editor-card-header">
        <span class="editor-subsection-label">Conditions</span>
        <div class="editor-card-actions">
          <button type="button" class="button-secondary" data-team-action="add-condition" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">Add Condition</button>
        </div>
      </div>
      <div class="condition-editor-list">${conditionsMarkup || '<div class="board-empty-state">Add a condition to decide when this priority fires.</div>'}</div>
    </article>
  `;
}

function renderConditionEditor(characterIndex, ruleIndex, condition, conditionIndex) {
  const value = condition.value;
  const valueType = getConditionValueType(condition);
  const statValue = valueType === "stat" ? value.stat : "vit";
  const statusValue =
    valueType === "has_status"
      ? value.has_status
      : valueType === "status_stacks"
        ? value.status_stacks
        : "Ward";
  const detailFieldMarkup = valueType === "stat"
    ? `
        <label class="field-group">
          <span>Stat</span>
          <select data-condition-field="value_stat" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" data-condition-index="${conditionIndex}">
            ${statFieldOptions.map((option) => `<option value="${option}" ${statValue === option ? "selected" : ""}>${option.toUpperCase()}</option>`).join("")}
          </select>
        </label>
      `
    : valueType === "has_status" || valueType === "status_stacks"
      ? `
        <label class="field-group">
          <span>Status</span>
          <input type="text" data-condition-field="value_status" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" data-condition-index="${conditionIndex}" value="${escapeHtml(statusValue)}">
        </label>
      `
      : "";

  return `
    <div class="editor-card">
      <div class="editor-card-header">
        <h6>Condition ${conditionIndex + 1}</h6>
        <div class="editor-card-actions">
          <button type="button" class="button-quiet" data-team-action="remove-condition" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" data-condition-index="${conditionIndex}">Remove</button>
        </div>
      </div>
      <div class="condition-preview">${escapeHtml(formatConditionPreview(condition))}</div>
      <div class="editor-grid">
        <label class="field-group">
          <span>Subject</span>
          <select data-condition-field="subject" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" data-condition-index="${conditionIndex}">
            ${ruleSubjectOptions.map((option) => `<option value="${option.value}" ${condition.subject === option.value ? "selected" : ""}>${option.label}</option>`).join("")}
          </select>
        </label>
        <label class="field-group">
          <span>Value</span>
          <select data-condition-field="value_type" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" data-condition-index="${conditionIndex}">
            ${ruleValueTypeOptions.map((option) => `<option value="${option.value}" ${valueType === option.value ? "selected" : ""}>${option.label}</option>`).join("")}
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

    if (target.dataset.characterField.startsWith("active_")) {
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

  if (target.dataset.statField) {
    const character = team.characters[characterIndex];
    if (!character) {
      return;
    }
    character.stats[target.dataset.statField] = Number(target.value);
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
    } else if (target.dataset.conditionField === "value_type") {
      if (target.value === "stat") {
        condition.value = { stat: "vit" };
      } else if (target.value === "has_status") {
        condition.value = { has_status: "Ward" };
      } else if (target.value === "status_stacks") {
        condition.value = { status_stacks: "Empower:MGT" };
      } else {
        condition.value = target.value;
      }
    } else if (target.dataset.conditionField === "value_stat") {
      condition.value = { stat: target.value };
    } else if (target.dataset.conditionField === "value_status") {
      if (isPlainObject(condition.value) && typeof condition.value.has_status === "string") {
        condition.value = { has_status: target.value };
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

  if (!team && !["add-library-character", "replace-library-character", "remove-library-character"].includes(action)) {
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
    case "add-character-at-position":
      addCharacterAtPosition(team, Number(actionTarget.dataset.gridRow), Number(actionTarget.dataset.gridCol));
      break;
    case "select-character":
      appState.selectedTeamCharacterIndex = characterIndex;
      appState.expandedRuleIndex = null;
      break;
    case "focus-browser":
      appState.teamBrowserMode = actionTarget.dataset.browserMode ?? "active";
      appState.teamBrowserSlotIndex = Number(actionTarget.dataset.browserSlotIndex ?? 0);
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
    case "save-character-to-library":
      saveCharacterToLibrary(characterIndex);
      return;
    case "load-character":
      teamEditorConfig.editor
        .querySelector(`[data-team-action="load-character-file"][data-character-index="${characterIndex}"]`)
        ?.click();
      return;
    case "remove-library-character":
      removeCharacterFromLibrary(Number(actionTarget.dataset.libraryIndex));
      return;
    case "download-library-character":
      downloadLibraryCharacterJson(Number(actionTarget.dataset.libraryIndex));
      return;
    case "add-library-character":
      addLibraryCharacterToTeam(Number(actionTarget.dataset.libraryIndex));
      return;
    case "replace-library-character":
      replaceTeamCharacterFromLibrary(actionTarget);
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
    case "toggle-rule-edit":
      appState.expandedRuleIndex = appState.expandedRuleIndex === ruleIndex ? null : ruleIndex;
      break;
    case "add-condition":
      team.characters[characterIndex]?.rules[ruleIndex]?.when.push(createEmptyCondition());
      break;
    case "remove-condition":
      team.characters[characterIndex]?.rules[ruleIndex]?.when.splice(conditionIndex, 1);
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

function renderCharacterLibrary() {
  if (!characterLibraryShell) {
    return;
  }

  if (appState.characterLibrary.length === 0) {
    characterLibraryShell.innerHTML = '<div class="board-empty-state">Save a character from a team slot to build a reusable library here.</div>';
    return;
  }

  const markup = appState.characterLibrary
    .map((character, libraryIndex) => renderCharacterLibraryCard(character, libraryIndex))
    .join("");
  characterLibraryShell.innerHTML = `<div class="character-library-list">${markup}</div>`;
}

function renderCharacterLibraryCard(character, libraryIndex) {
  const team = appState.teamConfig;
  const replaceOptions = team?.characters?.length
    ? team.characters.map((teamCharacter, teamIndex) => `
        <option value="${teamIndex}">${escapeHtml(teamCharacter.display_name || teamCharacter.id || `Slot ${teamIndex + 1}`)}</option>
      `).join("")
    : "";
  const actives = Array.isArray(character.actives) && character.actives.length > 0
    ? character.actives.join(" · ")
    : "No actives";

  return `
    <article class="editor-card character-library-card">
      <div class="editor-card-header">
        <div>
          <h5>${escapeHtml(character.display_name || character.id || `Character ${libraryIndex + 1}`)}</h5>
          <div class="library-card-meta">${escapeHtml(character.id || "No id")} · row ${character.position?.row ?? "?"}, col ${character.position?.col ?? "?"}</div>
        </div>
        <div class="editor-card-actions">
          <button type="button" class="button-secondary" data-team-action="add-library-character" data-library-index="${libraryIndex}">Add to Team</button>
          <button type="button" class="button-quiet" data-team-action="download-library-character" data-library-index="${libraryIndex}">Download</button>
          <button type="button" class="button-quiet" data-team-action="remove-library-character" data-library-index="${libraryIndex}">Remove</button>
        </div>
      </div>
      <div class="library-card-tags">
        <span${renderTitleAttribute(getPassiveDescription(character.passive))}>${escapeHtml(character.passive || "No passive")}</span>
        <span>${escapeHtml(actives)}</span>
      </div>
      <div class="library-card-controls">
        <label class="field-group">
          <span>Replace Slot</span>
          <select data-library-field="replace-index" data-library-index="${libraryIndex}" ${replaceOptions ? "" : "disabled"}>
            ${replaceOptions || '<option value="">No team loaded</option>'}
          </select>
        </label>
        <button type="button" class="button-quiet" data-team-action="replace-library-character" data-library-index="${libraryIndex}" ${replaceOptions ? "" : "disabled"}>Replace Slot</button>
      </div>
    </article>
  `;
}

function saveCharacterToLibrary(characterIndex) {
  const team = appState.teamConfig;
  const character = team?.characters?.[characterIndex];
  if (!character) {
    renderTeamValidation({ ok: false, errors: ["No character is available to save to the library."] });
    return;
  }

  appState.characterLibrary.push(cloneCharacterConfig(character));
  renderCharacterLibrary();
  setTeamValidationStatus("success", `${character.display_name || character.id || `Character ${characterIndex + 1}`} saved`);
}

function removeCharacterFromLibrary(libraryIndex) {
  const [removed] = appState.characterLibrary.splice(libraryIndex, 1);
  renderCharacterLibrary();
  setTeamValidationStatus("success", removed
    ? `${removed.display_name || removed.id || `Character ${libraryIndex + 1}`} removed`
    : "Library entry removed");
}

function downloadLibraryCharacterJson(libraryIndex) {
  const character = appState.characterLibrary[libraryIndex];
  if (!character) {
    renderTeamValidation({ ok: false, errors: ["No saved character is available to download."] });
    return;
  }

  triggerJsonDownload(JSON.stringify(character, null, 2), buildCharacterFilename(character, `library_character_${libraryIndex + 1}`));
  setTeamValidationStatus("success", `${character.display_name || character.id || "Character"} downloaded`);
}

function addLibraryCharacterToTeam(libraryIndex) {
  const character = appState.characterLibrary[libraryIndex];
  if (!character) {
    renderTeamValidation({ ok: false, errors: ["No saved character was found in the library."] });
    return;
  }

  if (!appState.teamConfig) {
    appState.teamConfig = {
      version: 1,
      name: "New Team",
      characters: [],
    };
  }

  if (appState.teamConfig.characters.length >= 5) {
    renderTeamValidation({ ok: false, errors: ["A team can have at most 5 characters."] });
    return;
  }

  const nextCharacter = cloneCharacterConfig(character);
  const firstOpen = findFirstOpenPosition(appState.teamConfig);
  if (!firstOpen) {
    renderTeamValidation({ ok: false, errors: ["A team can have at most 5 characters."] });
    return;
  }
  nextCharacter.position = { row: firstOpen.row, col: firstOpen.col };
  appState.teamConfig.characters.push(nextCharacter);
  appState.selectedTeamCharacterIndex = appState.teamConfig.characters.length - 1;
  appState.expandedRuleIndex = null;
  syncTeamUI();
  setTeamValidationStatus("success", `${character.display_name || character.id || "Character"} added`);
}

function replaceTeamCharacterFromLibrary(button) {
  const libraryIndex = Number(button.dataset.libraryIndex);
  const character = appState.characterLibrary[libraryIndex];
  const card = button.closest(".character-library-card");
  const replaceSelect = card?.querySelector('[data-library-field="replace-index"]');
  const replaceIndex = Number(replaceSelect?.value);

  if (!character || !appState.teamConfig || !Number.isInteger(replaceIndex) || replaceIndex < 0) {
    renderTeamValidation({ ok: false, errors: ["Choose a valid team slot to replace from the library."] });
    return;
  }

  const nextCharacter = cloneCharacterConfig(character);
  const replacePosition = appState.teamConfig.characters[replaceIndex]?.position ?? { row: 0, col: 0 };
  nextCharacter.position = { row: replacePosition.row, col: replacePosition.col };
  appState.teamConfig.characters[replaceIndex] = nextCharacter;
  appState.selectedTeamCharacterIndex = replaceIndex;
  appState.expandedRuleIndex = null;
  syncTeamUI();
  setTeamValidationStatus("success", `${character.display_name || character.id || "Character"} replaced slot ${replaceIndex + 1}`);
}

function cloneCharacterConfig(character) {
  return JSON.parse(JSON.stringify(character));
}

function addCharacterAtFirstOpenPosition(team) {
  const firstOpen = findFirstOpenPosition(team);
  if (!firstOpen) {
    renderTeamValidation({ ok: false, errors: ["A team can have at most 5 characters."] });
    return;
  }
  addCharacterAtPosition(team, firstOpen.row, firstOpen.col);
}

function addCharacterAtPosition(team, row, col) {
  if (team.characters.length >= 5) {
    renderTeamValidation({ ok: false, errors: ["A team can have at most 5 characters."] });
    return;
  }

  if (!isWithinGrid(row, col)) {
    return;
  }

  if (findCharacterIndexAtPosition(team, row, col) !== -1) {
    return;
  }

  team.characters.push(createEmptyCharacter(team.characters.length, row, col));
  appState.selectedTeamCharacterIndex = team.characters.length - 1;
  appState.expandedRuleIndex = null;
}

function findFirstOpenPosition(team) {
  for (let row = 0; row < 3; row += 1) {
    for (let col = 0; col < 4; col += 1) {
      if (findCharacterIndexAtPosition(team, row, col) === -1) {
        return { row, col };
      }
    }
  }
  return null;
}

function findCharacterIndexAtPosition(team, row, col) {
  return team.characters.findIndex((character) => character.position?.row === row && character.position?.col === col);
}

function isWithinGrid(row, col) {
  return Number.isInteger(row) && Number.isInteger(col) && row >= 0 && row <= 2 && col >= 0 && col <= 3;
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
  return {
    id: `new_character_${index + 1}`,
    display_name: "",
    position: { row, col },
    stats: { vit: 5, mgt: 5, mag: 5, arm: 5, res: 5, spd: 5, wil: 5 },
    passive: "",
    actives: ["", "", ""].filter(Boolean),
    item: null,
    rules: [],
  };
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

  syncTeamUI();
}

function handleTeamEditorDragStart(event) {
  const dragTarget = event.target.closest?.("[data-team-drag-character-index]");
  if (!(dragTarget instanceof HTMLElement) || !(event.dataTransfer instanceof DataTransfer)) {
    return;
  }

  event.dataTransfer.effectAllowed = "move";
  event.dataTransfer.setData("text/plain", dragTarget.dataset.teamDragCharacterIndex ?? "");

  const rect = dragTarget.getBoundingClientRect();
  const dragPreview = dragTarget.cloneNode(true);
  if (dragPreview instanceof HTMLElement) {
    cleanupDragPreviewElement();
    dragPreview.style.position = "fixed";
    dragPreview.style.top = "-1000px";
    dragPreview.style.left = "-1000px";
    dragPreview.style.width = `${rect.width}px`;
    dragPreview.style.pointerEvents = "none";
    dragPreview.style.zIndex = "9999";
    dragPreview.style.boxSizing = "border-box";
    document.body.appendChild(dragPreview);
    appState.dragPreviewElement = dragPreview;
    event.dataTransfer.setDragImage(
      dragPreview,
      rect.width / 2,
      rect.height / 2,
    );
    window.setTimeout(() => {
      cleanupDragPreviewElement();
    }, 0);
  }
}

function handleTeamEditorDragOver(event) {
  const cell = event.target.closest?.("[data-team-grid-cell]");
  if (!(cell instanceof HTMLElement)) {
    return;
  }
  event.preventDefault();
  if (event.dataTransfer instanceof DataTransfer) {
    event.dataTransfer.dropEffect = "move";
  }
}

function handleTeamEditorDrop(event) {
  const cell = event.target.closest?.("[data-team-grid-cell]");
  if (!(cell instanceof HTMLElement) || !(event.dataTransfer instanceof DataTransfer)) {
    return;
  }

  event.preventDefault();
  const team = appState.teamConfig;
  if (!team) {
    return;
  }

  const sourceIndex = Number(event.dataTransfer.getData("text/plain"));
  const row = Number(cell.dataset.gridRow);
  const col = Number(cell.dataset.gridCol);
  if (!Number.isInteger(sourceIndex) || sourceIndex < 0) {
    return;
  }

  if (moveCharacterToPosition(team, sourceIndex, row, col)) {
    syncTeamUI();
  }
}

function cleanupDragPreviewElement() {
  if (appState.dragPreviewElement instanceof HTMLElement) {
    appState.dragPreviewElement.remove();
  }
  appState.dragPreviewElement = null;
}

function formatGridPosition(row, col) {
  if (!isWithinGrid(row, col)) {
    return "Unplaced";
  }
  const rowLabels = ["Front row", "Middle row", "Back row"];
  return `${rowLabels[row]} · col ${col + 1}`;
}

function getBrowserEntries(mode) {
  if (mode === "passive") {
    return appState.catalogs.passives.map((name) => ({ name, description: getPassiveDescription(name) }));
  }
  if (mode === "item") {
    return appState.catalogs.items.map((name) => ({ name, description: getItemDescription(name) }));
  }
  return appState.catalogs.abilities.map((name) => ({ name, description: getAbilityDescription(name) }));
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

function createEmptyRule() {
  return {
    ability: "",
    when: [createEmptyCondition()],
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
    return "has_status";
  }
  if (isPlainObject(value) && typeof value.status_stacks === "string") {
    return "status_stacks";
  }
  return String(value ?? "hp");
}

function formatConditionPreview(condition) {
  const subjectLabel = getRuleOptionLabel(ruleSubjectOptions, condition.subject ?? "self");
  const valueType = getConditionValueType(condition);
  const operatorLabel = getRuleOptionLabel(ruleOperatorOptions, condition.op ?? condition.comparator ?? "gte");
  const threshold = condition.threshold ?? 0;

  if (valueType === "stat") {
    return `${subjectLabel} ${String(condition.value?.stat ?? "vit").toUpperCase()} ${operatorLabel} ${threshold}`;
  }

  if (valueType === "has_status") {
    return `${subjectLabel} Has Status ${condition.value?.has_status ?? "Ward"} ${operatorLabel} ${threshold}`;
  }

  if (valueType === "status_stacks") {
    return `${subjectLabel} Status Stacks ${condition.value?.status_stacks ?? "Empower:MGT"} ${operatorLabel} ${threshold}`;
  }

  return `${subjectLabel} ${getRuleOptionLabel(ruleValueTypeOptions, valueType)} ${operatorLabel} ${threshold}`;
}

function formatRulePreview(rule) {
  const abilityLabel = rule?.ability || "an ability";
  const conditions = Array.isArray(rule?.when) ? rule.when : [];
  if (conditions.length === 0) {
    return `Use ${abilityLabel} if always available`;
  }

  return `Use ${abilityLabel} if ${conditions.map((condition) => formatConditionPreview(condition)).join(" and ")}`;
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
      <h5>Passive</h5>
      <div class="pill-list"><span${renderTitleAttribute(getPassiveDescription(passiveName))}>${escapeHtml(passiveName || "No passive")}</span></div>
    </section>
    <section class="inspector-section">
      <h5>Effective Stats</h5>
      <div class="inspector-stat-grid">
        ${renderEffectiveStats(baseStats, effectiveStats)}
      </div>
    </section>
    <section class="inspector-section">
      <h5>Statuses</h5>
      <div class="status-list">${statusMarkup}</div>
    </section>
    <section class="inspector-section">
      <h5>Conditions</h5>
      <div class="status-list">${conditionMarkup}</div>
    </section>
    <section class="inspector-section">
      <h5>Current Focus</h5>
      <div class="pill-list"><span>${escapeHtml(focusLabel)}</span></div>
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
    return "<span>No statuses</span>";
  }

  return entries.map(({ name, stacks }) => `<span>${escapeHtml(name)} x${stacks}</span>`).join("");
}

function renderConditionList(conditions) {
  const entries = normalizeConditionEntries(conditions);
  if (entries.length === 0) {
    return "<span>No conditions</span>";
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
