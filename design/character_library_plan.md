# Character Library and Team Composition Plan

This file tracks the planned UI and data-flow work for saving individual character loadouts and composing teams from them.

## Goal

Support a workflow where players can:

- save a single character loadout as its own JSON
- load a saved character JSON into a team slot
- build a team by combining saved character configs
- still save and load full team JSON files directly

This is intended as an intermediate step before any later move toward a lighter `character_id + overrides` team schema.

## Why This Matters

The current Team Builder stores only full-team JSON.

That works, but it creates friction:

- repeated character edits across multiple teams
- no reusable character library
- harder experimentation with swapping one unit into different teams

Adding character-level save/load improves iteration without requiring a major schema redesign.

## Scope

This plan focuses on the Team Builder and dev-tool workflow.

It does not yet require:

- engine schema changes
- a persistent database
- final roster-reference architecture

The initial version can be fully client-side and file-based.

## Data Model Direction

### CharacterConfig File

Each saved character file should use the same shape already used inside a team:

```json
{
  "id": "the_emperor",
  "display_name": "The Emperor",
  "position": { "row": 0, "col": 0 },
  "stats": {
    "vit": 12,
    "mgt": 12,
    "mag": 8,
    "arm": 7,
    "res": 5,
    "spd": 8,
    "wil": 12
  },
  "passive": "Imperial Formation",
  "actives": ["Hold the Line", "Command", "Taunt"],
  "item": null,
  "rules": []
}
```

This avoids schema duplication and makes copy-in/copy-out trivial.

### Team File

For now, full team files can continue to store full characters inline:

```json
{
  "version": 1,
  "name": "Imperial Phalanx",
  "characters": [
    { "...full character config..." }
  ]
}
```

Later, teams may evolve toward references plus overrides, but that is not required for this phase.

## Planned Features

### Phase 1: Save/Load a Single Character in the Team Editor

Add per-character import/export controls inside the structured team editor.

Work:

- add `Save Character JSON` action for each character card
- add `Load Character JSON` action for each character card
- allow pasting or selecting a local file for one character
- replace the slot contents with the loaded character config

Result:

- character loadouts become reusable
- no separate team-library UI is required yet

Status:

- completed
- character cards now support direct JSON save/load
- loaded character files are validated before replacing the slot

### Phase 2: Add a Lightweight Character Library Panel

Add a builder-side character library workspace or panel.

Work:

- allow keeping a small in-memory list of saved characters during the session
- support:
  - `Save From Slot`
  - `Insert Into Team`
  - `Replace Slot`
  - `Remove From Library`
- show compact cards with:
  - name
  - passive
  - actives
  - position

Result:

- building variant teams gets much faster
- no filesystem-backed catalog is required yet

Status:

- completed
- the Team Builder now includes a lightweight in-memory character library panel
- characters can be saved from a team slot into the current session library
- saved library entries can be removed without affecting the current team

### Phase 3: Compose Team JSON from Saved Characters

Make it easy to assemble a team from saved character configs without editing raw JSON.

Work:

- add library actions such as:
  - `Add to Team`
  - `Replace Character`
- preserve existing team validation:
  - unique ids
  - unique positions
  - valid board bounds

Result:

- full team assembly becomes composition-first instead of only form-edit-first

Status:

- completed
- saved library characters can now be added directly into the current team
- library entries can replace an existing team slot without editing raw JSON
- if no team is loaded yet, adding from the library starts a new team shell automatically

### Phase 4: Optional Filesystem-Friendly Workflow Polish

Add better file-based ergonomics.

Possible work:

- better default filenames for character export
- optional suggested folder naming such as `tools/ui/characters/`
- clearer import/export instructions in the UI

Result:

- easier long-term reuse outside one browser session

Status:

- completed
- character exports now use cleaner slug-based filenames from display name or id
- saved library entries can be downloaded directly without first adding them to a team
- the Team Builder now includes clearer guidance around reusable character JSON files and a suggested local folder convention

## UI Direction

### Character Card Actions

Each team-editor character card should eventually support:

- `Save Character`
- `Load Character`
- `Remove`

This keeps character-level operations close to the thing being edited.

### Library Panel

If added, the library should be simple:

- not a visual gallery
- not drag-and-drop initially
- just a practical list of reusable character cards

Suggested fields on each library card:

- display name / id
- passive
- actives
- position
- buttons for insertion/replacement/export

## Validation Requirements

Loaded character JSON should validate:

- required object shape
- valid stat keys
- valid numeric stat values
- string passive
- array actives
- array rules
- valid position object

When inserted into a team, team-level validation still applies:

- unique `id`
- unique `(row, col)`
- at least one character

## Out of Scope

Not part of this phase:

- persistent browser storage
- team files that only store character references
- cloud sync
- shared roster management
- character versioning

## Recommended Order

1. Phase 1: per-character save/load in the team editor
2. Phase 2: lightweight in-memory character library
3. Phase 3: composition-first team assembly
4. Phase 4: file-workflow polish

## Verification

After each phase:

- `node --check tools/ui/app.js`
- manual browser check of:
  - saving a character
  - loading that character back into a slot
  - composing a team from saved characters
  - exporting final team JSON
