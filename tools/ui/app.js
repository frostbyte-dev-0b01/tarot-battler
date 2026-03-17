const tabButtons = document.querySelectorAll("[data-tab-target]");
const workspaces = document.querySelectorAll(".workspace");
const replayFileInput = document.querySelector("#replay-file-input");
const replayJsonInput = document.querySelector("#replay-json-input");
const replayLoadButton = document.querySelector("#replay-load-button");
const replayDemoButton = document.querySelector("#replay-demo-button");
const replayValidationOutput = document.querySelector("#replay-validation-output");
const teamABoard = document.querySelector("#team-a-board");
const teamBBoard = document.querySelector("#team-b-board");
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
    { tick: 3, type: "ability_used", actor_id: "the_emperor", ability: "Crush", mp_cost: 2 },
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
      renderReplayMetadata(parsedReplay);
      renderBoards(parsedReplay);
    } else {
      resetMetadata();
      resetBoards();
    }
  } catch (error) {
    renderReplayValidation({
      ok: false,
      errors: [`Could not parse replay JSON: ${error.message}`],
    });
    resetMetadata();
    resetBoards();
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
    resetMetadata();
    resetBoards();
  }
});

resetMetadata();
resetBoards();

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

function renderBoards(replay) {
  renderTeamBoard(teamABoard, replay.teams.team_a.characters, "team_a");
  renderTeamBoard(teamBBoard, replay.teams.team_b.characters, "team_b");
}

function resetBoards() {
  renderTeamBoard(teamABoard, [], "team_a");
  renderTeamBoard(teamBBoard, [], "team_b");
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
      return `<div class="grid-cell ${character ? "grid-cell-occupied" : ""}">${
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
  const hpValue = Number(character.max_hp) || 0;
  const mpValue = Number(character.max_mp) || 0;

  return `
    <article class="unit-card">
      <div class="unit-card-header">
        <h5 class="unit-card-name">${escapeHtml(character.display_name || character.id || "Unknown")}</h5>
        <span class="unit-card-position">r${character.position.row} c${character.position.col}</span>
      </div>
      <div class="unit-card-bars">
        ${renderBar("HP", hpValue, hpValue, "hp")}
        ${renderBar("MP", mpValue, mpValue, "mp")}
      </div>
      <div class="unit-card-passive">${escapeHtml(character.passive || "No passive")}</div>
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

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}
