const tabButtons = document.querySelectorAll("[data-tab-target]");
const workspaces = document.querySelectorAll(".workspace");
const replayFileInput = document.querySelector("#replay-file-input");
const replayJsonInput = document.querySelector("#replay-json-input");
const replayLoadButton = document.querySelector("#replay-load-button");
const replayDemoButton = document.querySelector("#replay-demo-button");
const replayValidationOutput = document.querySelector("#replay-validation-output");
const replayPreviousButton = document.querySelector("#replay-previous-button");
const replayPlayButton = document.querySelector("#replay-play-button");
const replayPauseButton = document.querySelector("#replay-pause-button");
const replayNextButton = document.querySelector("#replay-next-button");
const replayRestartButton = document.querySelector("#replay-restart-button");
const replayEventSlider = document.querySelector("#replay-event-slider");
const replayEventLabel = document.querySelector("#replay-event-label");
const replayTickDisplay = document.querySelector("#replay-tick-display");
const timelineMajorOnlyInput = document.querySelector("#timeline-major-only");
const timelineSelectedOnlyInput = document.querySelector("#timeline-selected-only");
const timelineList = document.querySelector("#timeline-list");
const teamABoard = document.querySelector("#team-a-board");
const teamBBoard = document.querySelector("#team-b-board");
const appState = {
  replay: null,
  selectedEventIndex: -1,
  selectedCharacterId: null,
  playbackTimerId: null,
};
const metadataFields = {
  seed: document.querySelector('[data-meta-field="seed"]'),
  winner: document.querySelector('[data-meta-field="winner"]'),
  tick_count: document.querySelector('[data-meta-field="tick_count"]'),
  team_a: document.querySelector('[data-meta-field="team_a"]'),
  team_b: document.querySelector('[data-meta-field="team_b"]'),
};

const demoReplay = {
  version: 1,
  seed: 42,
  winner: "team_a",
  tick_count: 7,
  teams: {
    team_a: {
      name: "Imperial Phalanx",
      characters: [
        {
          id: "the_emperor",
          display_name: "The Emperor",
          position: { row: 0, col: 0 },
          max_hp: 14,
          max_mp: 4,
          stats: { con: 7, str: 8, int: 3, for: 7, wis: 3, dex: 4, spi: 4 },
          passive: "Authority",
          actives: ["Crush", "Embolden"],
        },
        {
          id: "the_hermit",
          display_name: "The Hermit",
          position: { row: 2, col: 1 },
          max_hp: 12,
          max_mp: 5,
          stats: { con: 6, str: 3, int: 5, for: 5, wis: 7, dex: 3, spi: 5 },
          passive: "Solitude",
          actives: ["Lantern", "Absolve"],
        },
      ],
    },
    team_b: {
      name: "Arcane Gambit",
      characters: [
        {
          id: "the_fool",
          display_name: "The Fool",
          position: { row: 0, col: 0 },
          max_hp: 12,
          max_mp: 3,
          stats: { con: 6, str: 6, int: 4, for: 5, wis: 4, dex: 6, spi: 3 },
          passive: "Beginner's Luck",
          actives: ["Leap of Faith", "Stumble"],
        },
        {
          id: "the_star",
          display_name: "The Star",
          position: { row: 2, col: 1 },
          max_hp: 10,
          max_mp: 6,
          stats: { con: 5, str: 2, int: 6, for: 3, wis: 7, dex: 3, spi: 6 },
          passive: "Hope",
          actives: ["Restore", "Purify"],
        },
      ],
    },
  },
  events: [
    { tick: 0, type: "battle_start" },
    { tick: 2, type: "turn_start", actor_id: "the_emperor", current_hp: 14, current_mp: 4 },
    { tick: 2, type: "ability_used", actor_id: "the_emperor", ability: "Crush", mp_cost: 2 },
    {
      tick: 2,
      type: "damage",
      source_id: "the_emperor",
      target_id: "the_fool",
      amount: 4,
      damage_kind: "physical",
      source_kind: "ability",
      source_name: "Crush",
      target_hp_after: 8,
    },
    {
      tick: 3,
      type: "status_applied",
      source_id: "the_star",
      target_id: "the_hermit",
      status: "Regen",
      stacks_added: 1,
      stacks_after: 1,
    },
    {
      tick: 4,
      type: "resource_changed",
      actor_id: "the_emperor",
      resource: "mp",
      delta: 2,
      value_after: 4,
      reason: "turn_regen",
    },
    {
      tick: 5,
      type: "status_tick",
      target_id: "the_hermit",
      status: "Regen",
      amount: 2,
      kind: "heal",
      target_hp_after: 12,
    },
    { tick: 7, type: "battle_end", winner: "team_a" },
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
      renderReplayMetadata(parsedReplay);
      renderCurrentReplay();
      renderPlaybackControls();
    } else {
      appState.replay = null;
      appState.selectedEventIndex = -1;
      stopPlayback();
      resetMetadata();
      resetBoards();
      renderPlaybackControls();
    }
  } catch (error) {
    renderReplayValidation({
      ok: false,
      errors: [`Could not parse replay JSON: ${error.message}`],
    });
    appState.replay = null;
    appState.selectedEventIndex = -1;
    stopPlayback();
    resetMetadata();
    resetBoards();
    renderPlaybackControls();
  }
});

replayDemoButton.addEventListener("click", () => {
  replayJsonInput.value = JSON.stringify(demoReplay, null, 2);
  replayLoadButton.click();
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
    stopPlayback();
    resetMetadata();
    resetBoards();
    renderPlaybackControls();
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

resetMetadata();
resetBoards();
renderPlaybackControls();
renderTimeline();

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
    renderTimeline();
    return;
  }

  const replayState = buildReplayState(appState.replay, appState.selectedEventIndex);
  renderBoards(replayState);
  replayTickDisplay.textContent = String(getCurrentTick());
  replayEventLabel.textContent = `Event Index · ${formatEventIndexLabel()}`;
  renderTimeline();
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

function stopPlayback() {
  if (appState.playbackTimerId !== null) {
    window.clearInterval(appState.playbackTimerId);
    appState.playbackTimerId = null;
  }
}

function renderTeamBoard(container, characters, teamKey) {
  const occupantMap = new Map();

  for (const character of characters) {
    if (
      isPlainObject(character.position) &&
      Number.isInteger(character.position.row) &&
      Number.isInteger(character.position.col)
    ) {
      occupantMap.set(`${character.position.row}:${character.position.col}`, character);
    }
  }

  const rowLabels = ["Front", "Middle", "Back"];
  const rowsMarkup = rowLabels.map((label, rowIndex) => {
    const cellsMarkup = Array.from({ length: 4 }, (_, colIndex) => {
      const character = occupantMap.get(`${rowIndex}:${colIndex}`);
      return `<div class="grid-cell ${character ? "grid-cell-occupied" : ""} ${
        character && !character.alive ? "grid-cell-defeated" : ""
      }">${
        character ? renderUnitCard(character) : ""
      }</div>`;
    }).join("");

    return `
      <span class="grid-label">${label}</span>
      <div class="grid-row">${cellsMarkup}</div>
    `;
  }).join("");

  const emptyState = characters.length === 0
    ? `<p class="board-empty-state">No ${teamKey === "team_a" ? "Team A" : "Team B"} snapshot loaded yet.</p>`
    : "";

  container.innerHTML = `${emptyState}${rowsMarkup}`;
}

function renderUnitCard(character) {
  const hpValue = Number(character.current_hp) || 0;
  const mpValue = Number(character.current_mp) || 0;
  const statusesText = formatStatuses(character.statuses);

  return `
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

function isPlainObject(value) {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function buildReplayState(replay, selectedEventIndex) {
  const state = {
    teams: {
      team_a: {
        name: replay.teams.team_a.name,
        characters: replay.teams.team_a.characters.map(createCharacterState),
      },
      team_b: {
        name: replay.teams.team_b.name,
        characters: replay.teams.team_b.characters.map(createCharacterState),
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

function createCharacterState(character) {
  return {
    ...character,
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

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}
