const tabButtons = document.querySelectorAll("[data-tab-target]");
const workspaces = document.querySelectorAll(".workspace");
const replayFileInput = document.querySelector("#replay-file-input");
const replayJsonInput = document.querySelector("#replay-json-input");
const replayLoadButton = document.querySelector("#replay-load-button");
const replayDemoButton = document.querySelector("#replay-demo-button");
const replayValidationOutput = document.querySelector("#replay-validation-output");
const latestReplayPath = "./sample-data/latest_replay.json";
const passiveCatalogPath = "../../battle_engine/src/data/passives.json";
const abilityCatalogPath = "../../battle_engine/src/data/abilities.json";
const teamConfigs = {
  team_a: {
    fileInput: document.querySelector("#team-a-file-input"),
    jsonInput: document.querySelector("#team-a-json-input"),
    loadButton: document.querySelector("#team-a-load-button"),
    demoButton: document.querySelector("#team-a-demo-button"),
    copyButton: document.querySelector("#team-a-copy-button"),
    downloadButton: document.querySelector("#team-a-download-button"),
    validationOutput: document.querySelector("#team-a-validation-output"),
    editor: document.querySelector("#team-a-editor"),
    metaName: document.querySelector('[data-team-meta="team_a_name"]'),
    metaCount: document.querySelector('[data-team-meta="team_a_count"]'),
  },
  team_b: {
    fileInput: document.querySelector("#team-b-file-input"),
    jsonInput: document.querySelector("#team-b-json-input"),
    loadButton: document.querySelector("#team-b-load-button"),
    demoButton: document.querySelector("#team-b-demo-button"),
    copyButton: document.querySelector("#team-b-copy-button"),
    downloadButton: document.querySelector("#team-b-download-button"),
    validationOutput: document.querySelector("#team-b-validation-output"),
    editor: document.querySelector("#team-b-editor"),
    metaName: document.querySelector('[data-team-meta="team_b_name"]'),
    metaCount: document.querySelector('[data-team-meta="team_b_count"]'),
  },
};
const replayPreviousButton = document.querySelector("#replay-previous-button");
const replayPlayButton = document.querySelector("#replay-play-button");
const replayPauseButton = document.querySelector("#replay-pause-button");
const replayNextButton = document.querySelector("#replay-next-button");
const replayRestartButton = document.querySelector("#replay-restart-button");
const replayEventSlider = document.querySelector("#replay-event-slider");
const replayEventLabel = document.querySelector("#replay-event-label");
const replayTickDisplay = document.querySelector("#replay-tick-display");
const currentEventTick = document.querySelector("#current-event-tick");
const currentEventIndex = document.querySelector("#current-event-index");
const currentEventText = document.querySelector("#current-event-text");
const timelineMajorOnlyInput = document.querySelector("#timeline-major-only");
const timelineSelectedOnlyInput = document.querySelector("#timeline-selected-only");
const timelineSelectedOnlyLabel = timelineSelectedOnlyInput.closest(".toggle-pill");
const timelineList = document.querySelector("#timeline-list");
const inspectorPanel = document.querySelector("#inspector-panel");
const teamABoard = document.querySelector("#team-a-board");
const teamBBoard = document.querySelector("#team-b-board");
const appState = {
  replay: null,
  selectedEventIndex: -1,
  selectedCharacterId: null,
  playbackTimerId: null,
  teamConfigs: {
    team_a: null,
    team_b: null,
  },
  catalogs: {
    passives: [],
    abilities: [],
  },
};
const metadataFields = {
  seed: document.querySelector('[data-meta-field="seed"]'),
  winner: document.querySelector('[data-meta-field="winner"]'),
  tick_count: document.querySelector('[data-meta-field="tick_count"]'),
  team_a: document.querySelector('[data-meta-field="team_a"]'),
  team_b: document.querySelector('[data-meta-field="team_b"]'),
};

const demoTeams = {
  team_a: {
    version: 1,
    name: "Imperial Phalanx",
    characters: [
      {
        id: "the_emperor",
        display_name: "The Emperor",
        position: { row: 0, col: 0 },
        stats: { con: 7, str: 8, int: 3, for: 7, wis: 3, dex: 4, spi: 4 },
        passive: "",
        actives: [],
        item: null,
        rules: [],
      },
    ],
  },
  team_b: {
    version: 1,
    name: "Arcane Gambit",
    characters: [
      {
        id: "the_star",
        display_name: "The Star",
        position: { row: 2, col: 1 },
        stats: { con: 5, str: 2, int: 6, for: 3, wis: 7, dex: 3, spi: 6 },
        passive: "",
        actives: [],
        item: null,
        rules: [],
      },
    ],
  },
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
  if (!appState.replay || appState.playbackTimerId !== null) {
    return;
  }

  appState.playbackTimerId = window.setInterval(() => {
    const maxEventIndex = getMaxEventIndex();
    if (appState.selectedEventIndex >= maxEventIndex) {
      stopPlayback();
      return;
    }

    setSelectedEventIndex(appState.selectedEventIndex + 1);
  }, 700);

  renderPlaybackControls();
});

replayPauseButton.addEventListener("click", () => {
  stopPlayback();
  renderPlaybackControls();
});

replayEventSlider.addEventListener("input", (event) => {
  const sliderValue = Number(event.target.value);
  setSelectedEventIndex(sliderValue - 1);
});

timelineMajorOnlyInput.addEventListener("change", () => {
  renderTimeline();
});

timelineSelectedOnlyInput.addEventListener("change", () => {
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

for (const [teamKey, teamConfig] of Object.entries(teamConfigs)) {
  teamConfig.loadButton.addEventListener("click", () => {
    loadTeamFromText(teamKey, teamConfig.jsonInput.value.trim());
  });

  teamConfig.demoButton.addEventListener("click", () => {
    teamConfig.jsonInput.value = JSON.stringify(demoTeams[teamKey], null, 2);
    loadTeamFromText(teamKey, teamConfig.jsonInput.value);
  });

  teamConfig.fileInput.addEventListener("change", async (event) => {
    const [file] = event.target.files ?? [];
    if (!file) {
      return;
    }

    try {
      const content = await file.text();
      teamConfig.jsonInput.value = content;
      loadTeamFromText(teamKey, content);
    } catch (error) {
      renderTeamValidation(teamKey, {
        ok: false,
        errors: [`Could not read team file: ${error.message}`],
      });
      resetTeamSummary(teamKey);
    }
  });

  teamConfig.copyButton.addEventListener("click", async () => {
    await copyTeamJson(teamKey);
  });

  teamConfig.downloadButton.addEventListener("click", () => {
    downloadTeamJson(teamKey);
  });

  teamConfig.editor.addEventListener("input", (event) => {
    handleTeamEditorInput(teamKey, event);
  });

  teamConfig.editor.addEventListener("change", (event) => {
    handleTeamEditorInput(teamKey, event);
  });

  teamConfig.editor.addEventListener("click", (event) => {
    handleTeamEditorAction(teamKey, event);
  });
}

resetMetadata();
resetBoards();
renderPlaybackControls();
renderTimeline();
renderInspector(null);
resetTeamSummary("team_a");
resetTeamSummary("team_b");
renderTeamEditor("team_a");
renderTeamEditor("team_b");
void loadEditorCatalogs();
void loadLatestReplay();

async function loadEditorCatalogs() {
  try {
    const [passiveResponse, abilityResponse] = await Promise.all([
      fetch(passiveCatalogPath, { cache: "no-store" }),
      fetch(abilityCatalogPath, { cache: "no-store" }),
    ]);

    if (!passiveResponse.ok || !abilityResponse.ok) {
      throw new Error(
        `catalog request failed (${passiveResponse.status}/${abilityResponse.status})`,
      );
    }

    const [passives, abilities] = await Promise.all([
      passiveResponse.json(),
      abilityResponse.json(),
    ]);

    appState.catalogs.passives = Object.keys(passives).sort();
    appState.catalogs.abilities = Object.keys(abilities).sort();
    renderTeamEditor("team_a");
    renderTeamEditor("team_b");
  } catch (_error) {
    appState.catalogs.passives = [];
    appState.catalogs.abilities = [];
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
    replayEventLabel.textContent = "Event Index · Start";
    currentEventTick.textContent = "Tick 0";
    currentEventIndex.textContent = "Start";
    currentEventText.textContent = "Load a replay and move through events to see a compact summary here.";
    renderTimeline();
    renderInspector(null);
    return;
  }

  const replayState = buildReplayState(appState.replay, appState.selectedEventIndex);
  renderBoards(replayState);
  replayTickDisplay.textContent = String(getCurrentTick());
  replayEventLabel.textContent = `Event Index · ${formatEventIndexLabel()}`;
  renderCurrentEventSummary();
  renderTimeline();
  renderInspector(getSelectedCharacter(replayState));
}

function renderBoards(replayState) {
  renderTeamBoard(teamABoard, replayState.teams.team_a.characters, "team_a");
  renderTeamBoard(teamBBoard, replayState.teams.team_b.characters, "team_b");
}

function resetBoards() {
  renderTeamBoard(teamABoard, [], "team_a");
  renderTeamBoard(teamBBoard, [], "team_b");
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
  replayPlayButton.disabled = !hasReplay || appState.playbackTimerId !== null || appState.selectedEventIndex >= maxEventIndex;
  replayPauseButton.disabled = appState.playbackTimerId === null;
  replayEventSlider.max = String(sliderMax);
  replayEventSlider.value = String(sliderValue);
  timelineSelectedOnlyInput.disabled = !appState.selectedCharacterId;
  timelineSelectedOnlyLabel?.classList.toggle("is-disabled", !appState.selectedCharacterId);
  if (!appState.selectedCharacterId) {
    timelineSelectedOnlyInput.checked = false;
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

function getCurrentTick() {
  if (!appState.replay || appState.selectedEventIndex < 0) {
    return 0;
  }

  const currentEvent = appState.replay.events[appState.selectedEventIndex];
  return typeof currentEvent?.tick === "number" ? currentEvent.tick : 0;
}

function formatEventIndexLabel() {
  return appState.selectedEventIndex < 0 ? "Start" : `${appState.selectedEventIndex + 1} / ${appState.replay.events.length}`;
}

function renderCurrentEventSummary() {
  if (!appState.replay || appState.selectedEventIndex < 0) {
    currentEventTick.textContent = "Tick 0";
    currentEventIndex.textContent = "Start";
    currentEventText.textContent = "Battle state before the first logged event.";
    return;
  }

  const event = appState.replay.events[appState.selectedEventIndex];
  currentEventTick.textContent = `Tick ${event.tick ?? 0}`;
  currentEventIndex.textContent = `Event ${appState.selectedEventIndex + 1}/${appState.replay.events.length}`;
  currentEventText.textContent = formatTimelineText(event);
}

function stopPlayback() {
  if (appState.playbackTimerId !== null) {
    window.clearInterval(appState.playbackTimerId);
    appState.playbackTimerId = null;
  }
}

function renderTeamBoard(container, characters, teamKey) {
  const occupantMap = new Map();
  const currentEvent = appState.replay && appState.selectedEventIndex >= 0
    ? appState.replay.events[appState.selectedEventIndex]
    : null;

  for (const character of characters) {
    if (
      isPlainObject(character.position) &&
      Number.isInteger(character.position.row) &&
      Number.isInteger(character.position.col)
    ) {
      occupantMap.set(`${character.position.row}:${character.position.col}`, character);
    }
  }

  const rowsMarkup = Array.from({ length: 3 }, (_, rowIndex) => {
    const cellsMarkup = Array.from({ length: 4 }, (_, colIndex) => {
      const character = occupantMap.get(`${rowIndex}:${colIndex}`);
      const isSelected = character && character.id === appState.selectedCharacterId;
      const isSource = character && currentEvent && [currentEvent.actor_id, currentEvent.source_id].includes(character.id);
      const isTarget = character && currentEvent && currentEvent.target_id === character.id;
      return `<div class="grid-cell ${character ? "grid-cell-occupied" : ""} ${
        character && !character.alive ? "grid-cell-defeated" : ""
      } ${isSelected ? "grid-cell-selected" : ""} ${isSource ? "grid-cell-source" : ""} ${
        isTarget ? "grid-cell-target" : ""
      }">${
        character ? renderUnitCard(character) : ""
      }</div>`;
    }).join("");

    return `
      <div class="grid-row">${cellsMarkup}</div>
    `;
  }).join("");

  const emptyState = characters.length === 0
    ? `<p class="board-empty-state">No ${teamKey === "team_a" ? "Team A" : "Team B"} snapshot loaded yet.</p>`
    : "";

  container.innerHTML = `${emptyState}${rowsMarkup}`;
  bindBoardSelection(container);
}

function renderUnitCard(character) {
  const hpValue = Number(character.current_hp) || 0;
  const mpValue = Number(character.current_mp) || 0;
  const statusesText = formatStatuses(character.statuses);

  return `
    <button class="grid-cell-button" type="button" data-character-id="${escapeHtml(character.id)}">
      <article class="unit-card">
      <div class="unit-card-header">
        <h5 class="unit-card-name">${escapeHtml(character.display_name || character.id || "Unknown")}</h5>
        <span class="unit-card-position">r${character.position.row} c${character.position.col}</span>
      </div>
      <div class="unit-card-bars">
        ${renderBar("HP", hpValue, character.max_hp, "hp")}
        ${renderBar("MP", mpValue, character.max_mp, "mp")}
      </div>
      <div class="unit-card-passive">${escapeHtml(character.passive || "No passive")}</div>
      <div class="unit-card-statuses">${escapeHtml(statusesText)}</div>
      </article>
    </button>
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

function loadTeamFromText(teamKey, sourceText) {
  if (!sourceText) {
    renderTeamValidation(teamKey, {
      ok: false,
      errors: ["Team JSON input is empty."],
    });
    appState.teamConfigs[teamKey] = null;
    resetTeamSummary(teamKey);
    return;
  }

  try {
    const parsedTeam = JSON.parse(sourceText);
    const validation = validateTeamConfig(parsedTeam);
    renderTeamValidation(teamKey, validation);

    if (validation.ok) {
      appState.teamConfigs[teamKey] = parsedTeam;
      syncTeamUI(teamKey);
    } else {
      appState.teamConfigs[teamKey] = null;
      resetTeamSummary(teamKey);
      renderTeamEditor(teamKey);
    }
  } catch (error) {
    renderTeamValidation(teamKey, {
      ok: false,
      errors: [`Could not parse team JSON: ${error.message}`],
    });
    appState.teamConfigs[teamKey] = null;
    resetTeamSummary(teamKey);
    renderTeamEditor(teamKey);
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
  const requiredStats = ["con", "str", "int", "for", "wis", "dex", "spi"];

  characters.forEach((character, index) => {
    const prefix = `characters[${index}]`;
    if (!isPlainObject(character)) {
      errors.push(`${prefix} must be an object.`);
      return;
    }

    if (typeof character.id !== "string" || character.id.trim() === "") {
      errors.push(`${prefix}.id must be a non-empty string.`);
    } else if (seenIds.has(character.id)) {
      errors.push(`${prefix}.id must be unique within the team.`);
    } else {
      seenIds.add(character.id);
    }

    if (!isPlainObject(character.position)) {
      errors.push(`${prefix}.position must be an object.`);
    } else {
      const { row, col } = character.position;
      if (!Number.isInteger(row) || row < 0 || row > 2) {
        errors.push(`${prefix}.position.row must be an integer from 0 to 2.`);
      }
      if (!Number.isInteger(col) || col < 0 || col > 3) {
        errors.push(`${prefix}.position.col must be an integer from 0 to 3.`);
      }
      if (Number.isInteger(row) && Number.isInteger(col)) {
        const positionKey = `${row}:${col}`;
        if (seenPositions.has(positionKey)) {
          errors.push(`${prefix}.position must be unique within the team.`);
        } else {
          seenPositions.add(positionKey);
        }
      }
    }

    if (!isPlainObject(character.stats)) {
      errors.push(`${prefix}.stats must be an object.`);
    } else {
      for (const statKey of requiredStats) {
        if (typeof character.stats[statKey] !== "number") {
          errors.push(`${prefix}.stats.${statKey} must be a number.`);
        }
      }
    }

    if (typeof character.passive !== "string") {
      errors.push(`${prefix}.passive must be a string.`);
    }

    if (!Array.isArray(character.actives)) {
      errors.push(`${prefix}.actives must be an array.`);
    }

    if (!Array.isArray(character.rules)) {
      errors.push(`${prefix}.rules must be an array.`);
    }
  });
}

function renderTeamSummary(teamKey, teamConfig) {
  const config = teamConfigs[teamKey];
  config.metaName.textContent = teamConfig.name;
  config.metaCount.textContent = String(teamConfig.characters.length);
}

function resetTeamSummary(teamKey) {
  const config = teamConfigs[teamKey];
  config.metaName.textContent = "-";
  config.metaCount.textContent = "-";
}

function syncTeamUI(teamKey) {
  const teamConfig = appState.teamConfigs[teamKey];
  if (!teamConfig) {
    resetTeamSummary(teamKey);
    renderTeamEditor(teamKey);
    return;
  }

  teamConfigs[teamKey].jsonInput.value = JSON.stringify(teamConfig, null, 2);
  renderTeamSummary(teamKey, teamConfig);
  renderTeamValidation(teamKey, validateTeamConfig(teamConfig));
  renderTeamEditor(teamKey);
}

function renderTeamValidation(teamKey, result) {
  const output = teamConfigs[teamKey].validationOutput;
  output.className = "message-panel";

  if (result.ok) {
    output.classList.add("message-panel-success");
    output.textContent = "Team JSON is valid at the top level.";
    return;
  }

  if (result.errors.length === 0) {
    output.classList.add("message-panel-idle");
    output.textContent = "Load a team JSON file or paste JSON to validate it.";
    return;
  }

  output.classList.add("message-panel-error");
  output.textContent = result.errors.map((error) => `- ${error}`).join("\n");
}

async function copyTeamJson(teamKey) {
  const team = appState.teamConfigs[teamKey];
  if (!team) {
    renderTeamValidation(teamKey, { ok: false, errors: ["No team is loaded to copy."] });
    return;
  }

  const jsonText = JSON.stringify(team, null, 2);

  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(jsonText);
      renderTeamValidation(teamKey, { ok: true, errors: [] });
      teamConfigs[teamKey].validationOutput.textContent = "Team JSON copied to clipboard.";
      return;
    }
  } catch (error) {
    renderTeamValidation(teamKey, { ok: false, errors: [`Could not copy team JSON: ${error.message}`] });
    return;
  }

  renderTeamValidation(teamKey, { ok: false, errors: ["Clipboard access is not available in this browser context."] });
}

function downloadTeamJson(teamKey) {
  const team = appState.teamConfigs[teamKey];
  if (!team) {
    renderTeamValidation(teamKey, { ok: false, errors: ["No team is loaded to download."] });
    return;
  }

  const jsonText = JSON.stringify(team, null, 2);
  const blob = new Blob([jsonText], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `${team.name || teamKey}.json`;
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
  renderTeamValidation(teamKey, { ok: true, errors: [] });
  teamConfigs[teamKey].validationOutput.textContent = "Team JSON downloaded.";
}

function renderTeamEditor(teamKey) {
  const editor = teamConfigs[teamKey].editor;
  const team = appState.teamConfigs[teamKey];
  if (!team) {
    editor.innerHTML = '<div class="board-empty-state">Load a team to edit it here.</div>';
    return;
  }

  const characterMarkup = team.characters.map((character, characterIndex) => renderCharacterEditor(teamKey, character, characterIndex)).join("");

  editor.innerHTML = `
    <div class="editor-toolbar">
      <button type="button" class="button-secondary" data-team-action="add-character">Add Character</button>
    </div>
    <label class="field-group">
      <span>Team Name</span>
      <input type="text" data-team-field="name" value="${escapeHtml(team.name)}">
    </label>
    <div class="character-editor-list">${characterMarkup}</div>
  `;
}

function renderCharacterEditor(teamKey, character, characterIndex) {
  const passiveOptions = buildSelectOptions(
    appState.catalogs.passives,
    character.passive ?? "",
    "No passive",
  );
  const activeSelections = normalizeActiveSelections(character.actives);
  const activeOptions = activeSelections.map((selection, activeIndex) =>
    buildSelectOptions(
      appState.catalogs.abilities,
      selection,
      `No active ${activeIndex + 1}`,
    ),
  );
  const rulesMarkup = (character.rules ?? []).map((rule, ruleIndex) => renderRuleEditor(teamKey, characterIndex, rule, ruleIndex)).join("");

  return `
    <article class="editor-card">
      <div class="editor-card-header">
        <h5>${escapeHtml(character.display_name || character.id || `Character ${characterIndex + 1}`)}</h5>
        <div class="editor-card-actions">
          <button type="button" class="button-quiet" data-team-action="remove-character" data-character-index="${characterIndex}">Remove</button>
        </div>
      </div>
      <div class="editor-grid">
        <label class="field-group">
          <span>ID</span>
          <input type="text" data-character-field="id" data-character-index="${characterIndex}" value="${escapeHtml(character.id ?? "")}">
        </label>
        <label class="field-group">
          <span>Display Name</span>
          <input type="text" data-character-field="display_name" data-character-index="${characterIndex}" value="${escapeHtml(character.display_name ?? "")}">
        </label>
        <label class="field-group">
          <span>Passive</span>
          <select data-character-field="passive" data-character-index="${characterIndex}">
            ${passiveOptions}
          </select>
        </label>
        <label class="field-group">
          <span>Item</span>
          <input type="text" data-character-field="item" data-character-index="${characterIndex}" value="${escapeHtml(character.item ?? "")}">
        </label>
        <label class="field-group">
          <span>Row</span>
          <input type="number" min="0" max="2" data-position-field="row" data-character-index="${characterIndex}" value="${character.position?.row ?? 0}">
        </label>
        <label class="field-group">
          <span>Col</span>
          <input type="number" min="0" max="3" data-position-field="col" data-character-index="${characterIndex}" value="${character.position?.col ?? 0}">
        </label>
        <label class="field-group">
          <span>Active 1</span>
          <select data-character-field="active_0" data-character-index="${characterIndex}">
            ${activeOptions[0]}
          </select>
        </label>
        <label class="field-group">
          <span>Active 2</span>
          <select data-character-field="active_1" data-character-index="${characterIndex}">
            ${activeOptions[1]}
          </select>
        </label>
        <label class="field-group">
          <span>Active 3</span>
          <select data-character-field="active_2" data-character-index="${characterIndex}">
            ${activeOptions[2]}
          </select>
        </label>
      </div>
      <div class="editor-inline-grid">
        ${["con", "str", "int", "for", "wis", "dex", "spi"].map((statKey) => `
          <label class="field-group">
            <span>${statKey.toUpperCase()}</span>
            <input type="number" data-stat-field="${statKey}" data-character-index="${characterIndex}" value="${character.stats?.[statKey] ?? 0}">
          </label>
        `).join("")}
      </div>
      <section class="editor-rule-section">
        <div class="editor-card-header">
          <span class="editor-subsection-label">Rules</span>
          <div class="editor-card-actions">
            <button type="button" class="button-secondary" data-team-action="add-rule" data-character-index="${characterIndex}">Add Rule</button>
          </div>
        </div>
        <div class="rule-editor-list">${rulesMarkup || '<div class="board-empty-state">No rules yet.</div>'}</div>
      </section>
    </article>
  `;
}

function renderRuleEditor(teamKey, characterIndex, rule, ruleIndex) {
  const conditionsMarkup = (rule.when ?? []).map((condition, conditionIndex) => renderConditionEditor(characterIndex, ruleIndex, condition, conditionIndex)).join("");

  return `
    <article class="editor-card">
      <div class="editor-card-header">
        <h6>Rule ${ruleIndex + 1}</h6>
        <div class="editor-card-actions">
          <button type="button" class="button-quiet" data-team-action="move-rule-up" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">Up</button>
          <button type="button" class="button-quiet" data-team-action="move-rule-down" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">Down</button>
          <button type="button" class="button-quiet" data-team-action="remove-rule" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">Remove</button>
        </div>
      </div>
      <label class="field-group">
        <span>Ability</span>
        <input type="text" data-rule-field="ability" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" value="${escapeHtml(rule.ability ?? "")}">
      </label>
      <div class="editor-card-header">
        <span class="editor-subsection-label">Conditions</span>
        <div class="editor-card-actions">
          <button type="button" class="button-secondary" data-team-action="add-condition" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}">Add Condition</button>
        </div>
      </div>
      <div class="condition-editor-list">${conditionsMarkup || '<div class="board-empty-state">No conditions yet.</div>'}</div>
    </article>
  `;
}

function renderConditionEditor(characterIndex, ruleIndex, condition, conditionIndex) {
  const value = condition.value;
  const valueType = isPlainObject(value) && typeof value.stat === "string" ? "stat" : String(value ?? "hp");
  const statValue = valueType === "stat" ? value.stat : "con";

  return `
    <div class="editor-card">
      <div class="editor-card-header">
        <h6>Condition ${conditionIndex + 1}</h6>
        <div class="editor-card-actions">
          <button type="button" class="button-quiet" data-team-action="remove-condition" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" data-condition-index="${conditionIndex}">Remove</button>
        </div>
      </div>
      <div class="editor-grid">
        <label class="field-group">
          <span>Subject</span>
          <select data-condition-field="subject" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" data-condition-index="${conditionIndex}">
            ${["self", "target", "companion", "world"].map((option) => `<option value="${option}" ${condition.subject === option ? "selected" : ""}>${option}</option>`).join("")}
          </select>
        </label>
        <label class="field-group">
          <span>Value</span>
          <select data-condition-field="value_type" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" data-condition-index="${conditionIndex}">
            ${["hp", "mp", "use_count", "turns_since_use", "tick_count", "ally_count", "enemy_count", "stat"].map((option) => `<option value="${option}" ${valueType === option ? "selected" : ""}>${option}</option>`).join("")}
          </select>
        </label>
        <label class="field-group">
          <span>Stat</span>
          <select data-condition-field="value_stat" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" data-condition-index="${conditionIndex}">
            ${["con", "str", "int", "for", "wis", "dex", "spi"].map((option) => `<option value="${option}" ${statValue === option ? "selected" : ""}>${option}</option>`).join("")}
          </select>
        </label>
        <label class="field-group">
          <span>Operator</span>
          <select data-condition-field="op" data-character-index="${characterIndex}" data-rule-index="${ruleIndex}" data-condition-index="${conditionIndex}">
            ${["gte", "lte"].map((option) => `<option value="${option}" ${(condition.op ?? condition.comparator) === option ? "selected" : ""}>${option}</option>`).join("")}
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

function handleTeamEditorInput(teamKey, event) {
  const team = appState.teamConfigs[teamKey];
  if (!team) {
    return;
  }

  const target = event.target;
  const characterIndex = Number(target.dataset.characterIndex);
  const ruleIndex = Number(target.dataset.ruleIndex);
  const conditionIndex = Number(target.dataset.conditionIndex);

  if (target.dataset.teamField === "name") {
    team.name = target.value;
    syncTeamUI(teamKey);
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

    syncTeamUI(teamKey);
    return;
  }

  if (target.dataset.positionField) {
    const character = team.characters[characterIndex];
    if (!character) {
      return;
    }
    character.position[target.dataset.positionField] = Number(target.value);
    syncTeamUI(teamKey);
    return;
  }

  if (target.dataset.statField) {
    const character = team.characters[characterIndex];
    if (!character) {
      return;
    }
    character.stats[target.dataset.statField] = Number(target.value);
    syncTeamUI(teamKey);
    return;
  }

  if (target.dataset.ruleField === "ability") {
    const rule = team.characters[characterIndex]?.rules?.[ruleIndex];
    if (!rule) {
      return;
    }
    rule.ability = target.value;
    syncTeamUI(teamKey);
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
      condition.value = target.value === "stat" ? { stat: "con" } : target.value;
    } else if (target.dataset.conditionField === "value_stat") {
      condition.value = { stat: target.value };
    } else if (target.dataset.conditionField === "op") {
      condition.op = target.value;
      delete condition.comparator;
    } else if (target.dataset.conditionField === "threshold") {
      condition.threshold = Number(target.value);
    }

    syncTeamUI(teamKey);
  }
}

function handleTeamEditorAction(teamKey, event) {
  const action = event.target.dataset.teamAction;
  if (!action) {
    return;
  }

  const team = appState.teamConfigs[teamKey];
  if (!team) {
    return;
  }

  const characterIndex = Number(event.target.dataset.characterIndex);
  const ruleIndex = Number(event.target.dataset.ruleIndex);
  const conditionIndex = Number(event.target.dataset.conditionIndex);

  switch (action) {
    case "add-character":
      team.characters.push(createEmptyCharacter(team.characters.length));
      break;
    case "remove-character":
      team.characters.splice(characterIndex, 1);
      break;
    case "add-rule":
      team.characters[characterIndex]?.rules.push(createEmptyRule());
      break;
    case "remove-rule":
      team.characters[characterIndex]?.rules.splice(ruleIndex, 1);
      break;
    case "move-rule-up":
      moveArrayItem(team.characters[characterIndex]?.rules, ruleIndex, ruleIndex - 1);
      break;
    case "move-rule-down":
      moveArrayItem(team.characters[characterIndex]?.rules, ruleIndex, ruleIndex + 1);
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

  syncTeamUI(teamKey);
}

function createEmptyCharacter(index) {
  return {
    id: `new_character_${index + 1}`,
    display_name: "",
    position: { row: 0, col: 0 },
    stats: { con: 5, str: 5, int: 5, for: 5, wis: 5, dex: 5, spi: 5 },
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

function buildReplayState(replay, selectedEventIndex) {
  const state = {
    teams: {
      team_a: {
        name: replay.teams.team_a.name,
        characters: replay.teams.team_a.characters.map((character) => createCharacterState(character, "team_a")),
      },
      team_b: {
        name: replay.teams.team_b.name,
        characters: replay.teams.team_b.characters.map((character) => createCharacterState(character, "team_b")),
      },
    },
  };

  const characterIndex = indexReplayCharacters(state);
  const cappedEventIndex = Math.min(selectedEventIndex, replay.events.length - 1);

  for (let index = 0; index <= cappedEventIndex; index += 1) {
    applyReplayEvent(characterIndex, replay.events[index]);
  }

  return state;
}

function createCharacterState(character, teamKey) {
  return {
    ...character,
    team_key: teamKey,
    current_hp: Number(character.max_hp) || 0,
    current_mp: Number(character.max_mp) || 0,
    alive: true,
    statuses: {},
  };
}

function indexReplayCharacters(state) {
  const characterIndex = new Map();

  for (const team of Object.values(state.teams)) {
    for (const character of team.characters) {
      characterIndex.set(character.id, character);
    }
  }

  return characterIndex;
}

function applyReplayEvent(characterIndex, event) {
  switch (event.type) {
    case "battle_start":
    case "basic_attack":
    case "passive_triggered":
    case "turn_skipped":
    case "battle_end":
      return;
    case "turn_start":
      syncTurnStart(characterIndex, event);
      return;
    case "ability_used":
      spendEventMp(characterIndex, event);
      return;
    case "damage":
      applyDamageEvent(characterIndex, event);
      return;
    case "healing":
      applyHealingEvent(characterIndex, event);
      return;
    case "status_applied":
      applyStatusEvent(characterIndex, event);
      return;
    case "status_removed":
      removeStatusEvent(characterIndex, event);
      return;
    case "status_tick":
      applyStatusTickEvent(characterIndex, event);
      return;
    case "resource_changed":
      applyResourceChangeEvent(characterIndex, event);
      return;
    case "defeat":
      applyDefeatEvent(characterIndex, event);
      return;
    default:
      return;
  }
}

function syncTurnStart(characterIndex, event) {
  const character = characterIndex.get(event.actor_id);
  if (!character) {
    return;
  }

  if (typeof event.current_hp === "number") {
    character.current_hp = clampValue(event.current_hp, 0, character.max_hp);
  }

  if (typeof event.current_mp === "number") {
    character.current_mp = clampValue(event.current_mp, 0, character.max_mp);
  }
}

function spendEventMp(characterIndex, event) {
  const character = characterIndex.get(event.actor_id);
  if (!character || typeof event.mp_cost !== "number") {
    return;
  }

  character.current_mp = clampValue(character.current_mp - event.mp_cost, 0, character.max_mp);
}

function applyDamageEvent(characterIndex, event) {
  const target = characterIndex.get(event.target_id);
  if (!target) {
    return;
  }

  if (typeof event.target_hp_after === "number") {
    target.current_hp = clampValue(event.target_hp_after, 0, target.max_hp);
  } else if (typeof event.amount === "number") {
    target.current_hp = clampValue(target.current_hp - event.amount, 0, target.max_hp);
  }

  if (target.current_hp <= 0) {
    target.alive = false;
  }
}

function applyHealingEvent(characterIndex, event) {
  const target = characterIndex.get(event.target_id);
  if (!target) {
    return;
  }

  if (typeof event.target_hp_after === "number") {
    target.current_hp = clampValue(event.target_hp_after, 0, target.max_hp);
  } else if (typeof event.amount === "number") {
    target.current_hp = clampValue(target.current_hp + event.amount, 0, target.max_hp);
  }

  if (target.current_hp > 0) {
    target.alive = true;
  }
}

function applyStatusEvent(characterIndex, event) {
  const target = characterIndex.get(event.target_id);
  if (!target || typeof event.status !== "string") {
    return;
  }

  if (typeof event.stacks_after === "number") {
    target.statuses[event.status] = Math.max(event.stacks_after, 0);
    return;
  }

  const currentStacks = target.statuses[event.status] ?? 0;
  const stacksAdded = typeof event.stacks_added === "number" ? event.stacks_added : 0;
  target.statuses[event.status] = Math.max(currentStacks + stacksAdded, 0);
}

function removeStatusEvent(characterIndex, event) {
  const target = characterIndex.get(event.target_id);
  if (!target || typeof event.status !== "string") {
    return;
  }

  if (typeof event.stacks_after === "number") {
    if (event.stacks_after <= 0) {
      delete target.statuses[event.status];
    } else {
      target.statuses[event.status] = event.stacks_after;
    }
    return;
  }

  const currentStacks = target.statuses[event.status] ?? 0;
  const removed = typeof event.stacks_removed === "number" ? event.stacks_removed : currentStacks;
  const nextStacks = Math.max(currentStacks - removed, 0);
  if (nextStacks <= 0) {
    delete target.statuses[event.status];
  } else {
    target.statuses[event.status] = nextStacks;
  }
}

function applyStatusTickEvent(characterIndex, event) {
  const target = characterIndex.get(event.target_id);
  if (!target) {
    return;
  }

  if (event.kind === "heal") {
    applyHealingEvent(characterIndex, event);
    return;
  }

  applyDamageEvent(characterIndex, event);
}

function applyResourceChangeEvent(characterIndex, event) {
  if (event.resource !== "mp") {
    return;
  }

  const target = characterIndex.get(event.actor_id);
  if (!target) {
    return;
  }

  if (typeof event.value_after === "number") {
    target.current_mp = clampValue(event.value_after, 0, target.max_mp);
    return;
  }

  if (typeof event.delta === "number") {
    target.current_mp = clampValue(target.current_mp + event.delta, 0, target.max_mp);
  }
}

function applyDefeatEvent(characterIndex, event) {
  const character = characterIndex.get(event.actor_id);
  if (!character) {
    return;
  }

  character.alive = false;
  character.current_hp = 0;
}

function clampValue(value, minValue, maxValue) {
  return Math.max(minValue, Math.min(maxValue, value));
}

function formatStatuses(statuses) {
  const entries = Object.entries(statuses ?? {}).filter(([, stacks]) => stacks > 0);
  if (entries.length === 0) {
    return "No statuses";
  }

  return entries.map(([status, stacks]) => `${status} x${stacks}`).join(" • ");
}

function renderInspector(character) {
  if (!character) {
    inspectorPanel.innerHTML = '<div class="board-empty-state">Select a unit on the board or load a replay event that references one.</div>';
    return;
  }

  const effectiveStats = calculateEffectiveStats(character);
  const statusMarkup = renderStatusList(character.statuses);
  const activeMarkup = (character.actives ?? []).map((active) => `<span>${escapeHtml(active)}</span>`).join("");
  const aliveLabel = character.alive ? "Alive" : "Defeated";

  inspectorPanel.innerHTML = `
    <div class="inspector-header">
      <div>
        <h4>${escapeHtml(character.display_name || character.id)}</h4>
        <div class="inspector-subtle">${escapeHtml(character.team_key === "team_a" ? "Team A" : "Team B")} · row ${character.position.row}, col ${character.position.col}</div>
      </div>
      <div class="inspector-status-line">
        <span class="status-chip">${aliveLabel}</span>
      </div>
    </div>
    <div class="unit-card-bars">
      ${renderBar("HP", character.current_hp, character.max_hp, "hp")}
      ${renderBar("MP", character.current_mp, character.max_mp, "mp")}
    </div>
    <section class="inspector-section">
      <h5>Passive</h5>
      <div class="pill-list"><span>${escapeHtml(character.passive || "No passive")}</span></div>
    </section>
    <section class="inspector-section">
      <h5>Actives</h5>
      <div class="pill-list">${activeMarkup || "<span>No actives</span>"}</div>
    </section>
    <section class="inspector-section">
      <h5>Statuses</h5>
      <div class="status-list">${statusMarkup}</div>
    </section>
    <section class="inspector-section">
      <h5>Stats</h5>
      <div class="stats-grid">
        ${renderStatsBlock("Base", character.stats)}
        ${renderStatsBlock("Effective", effectiveStats)}
      </div>
    </section>
  `;
}

function renderStatsBlock(label, stats) {
  const statOrder = ["con", "str", "int", "for", "wis", "dex", "spi"];
  const markup = statOrder.map((statKey) => `
    <dt>${statKey.toUpperCase()}</dt>
    <dd>${stats?.[statKey] ?? "-"}</dd>
  `).join("");

  return `
    <section class="stats-block">
      <h6>${label}</h6>
      <dl>${markup}</dl>
    </section>
  `;
}

function renderStatusList(statuses) {
  const entries = Object.entries(statuses ?? {}).filter(([, stacks]) => stacks > 0);
  if (entries.length === 0) {
    return "<span>No statuses</span>";
  }

  return entries.map(([status, stacks]) => `<span>${escapeHtml(status)} x${stacks}</span>`).join("");
}

function calculateEffectiveStats(character) {
  const baseStats = { ...(character.stats ?? {}) };
  const effectiveStats = { ...baseStats };

  for (const [status, stacks] of Object.entries(character.statuses ?? {})) {
    if (stacks <= 0) {
      continue;
    }

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
    ].filter(Boolean);

    return eventCharacters.includes(appState.selectedCharacterId);
  }

  return true;
}

function isMajorEvent(type) {
  return [
    "ability_used",
    "basic_attack",
    "damage",
    "healing",
    "status_applied",
    "status_removed",
    "status_tick",
    "passive_triggered",
    "turn_skipped",
    "defeat",
    "battle_end",
  ].includes(type);
}

function formatEventType(type) {
  return type.replaceAll("_", " ");
}

function formatTimelineText(event) {
  switch (event.type) {
    case "battle_start":
      return "Battle starts.";
    case "turn_start":
      return `${event.actor_id ?? "Unknown"} begins a turn at ${event.current_hp ?? "?"} HP and ${event.current_mp ?? "?"} MP.`;
    case "basic_attack":
      return `${event.actor_id ?? "Unknown"} attacks ${event.target_id ?? "Unknown"} with a ${event.damage_kind ?? "basic"} hit.`;
    case "ability_used":
      return `${event.actor_id ?? "Unknown"} uses ${event.ability ?? "an ability"} for ${event.mp_cost ?? "?"} MP.`;
    case "damage":
      return `${event.source_id ?? "Unknown"} deals ${event.amount ?? "?"} ${event.damage_kind ?? ""} damage to ${event.target_id ?? "Unknown"}.`;
    case "healing":
      return `${event.source_id ?? "Unknown"} restores ${event.amount ?? "?"} HP to ${event.target_id ?? "Unknown"}.`;
    case "status_applied":
      return `${event.target_id ?? "Unknown"} gains ${event.status ?? "a status"} (${event.stacks_after ?? "?"} stacks).`;
    case "status_removed":
      return `${event.target_id ?? "Unknown"} loses ${event.status ?? "a status"} (${event.stacks_after ?? 0} stacks remain).`;
    case "status_tick":
      return `${event.target_id ?? "Unknown"} resolves ${event.status ?? "a status"} for ${event.amount ?? "?"} ${event.kind ?? "effect"}.`;
    case "passive_triggered":
      return `${event.actor_id ?? "Unknown"} triggers ${event.passive ?? "a passive"} on ${event.trigger ?? "an event"}.`;
    case "turn_skipped":
      return `${event.actor_id ?? "Unknown"} skips a turn because of ${event.reason ?? "an effect"}.`;
    case "resource_changed":
      return `${event.actor_id ?? "Unknown"} ${event.delta >= 0 ? "gains" : "spends"} ${Math.abs(event.delta ?? 0)} ${event.resource ?? "resource"}.`;
    case "defeat":
      return `${event.actor_id ?? "Unknown"} is defeated.`;
    case "battle_end":
      return `Battle ends with ${event.winner ?? "no one"} winning.`;
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

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}
