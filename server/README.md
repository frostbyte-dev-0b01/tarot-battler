# Tarot Battler — local season server

A small single-process server that hosts a **monthly drafting season** for a
group of friends. It serves the existing static UI (`tools/ui/`), exposes a JSON
`/api`, runs battles natively through the engine (`battle_engine::run_battle_json`),
and persists everything in an embedded **redb** database file — no external
database or build step.

One pod: everyone who joins shares the same pod and stays in it month to month.

## Run it

From the `server/` directory:

```bash
cargo run
```

Then open <http://127.0.0.1:8080> in a browser and click the **Season** tab.

Configuration (environment variables):

| Var            | Default                         | Purpose                          |
|----------------|---------------------------------|----------------------------------|
| `TAROT_PORT`   | `8080`                          | Listen port                      |
| `TAROT_DB`     | `tarot-data/tarot.redb`         | Embedded database file path      |
| `TAROT_UI_DIR` | `../tools/ui` (relative to crate) | Static UI directory            |

The database file and its parent directory are created on first run. Both
`server/target/` and `server/tarot-data/` are git-ignored.

## Share it with friends

The server binds to `127.0.0.1` (your machine only). To let friends join, expose
the port over a network you trust — for a friend group the simplest options are:

- A tunneling tool (e.g. `ssh -R`, `cloudflared`, `ngrok`) pointed at `:8080`.
- A LAN: bind your machine's address / forward the port, then share
  `http://<your-ip>:8080`.

Everyone opens the shared address, picks a **display name** on the Season tab to
join the pod, drafts, submits a team, and battles. Trust is assumed within the
group (lightweight identity, no passwords).

## Playing a season

The season is a fixed **8-beat** arc (two beats/week over four weeks); budget
grows 10 → 15 and the roster from ~5 to ~7. See `design/metagame.md`.

Day-to-day, players:

1. **Claim** the open draft beat (or it auto-resolves at random when the next
   beat reveals — swaps are skipped, never auto-applied).
2. **Submit** a team built from their unlocked pool within budget (Commander +
   team passives + the drafted banner).

The host drives the clock from the Season tab's admin controls:

- **Run match day** — play the pod round-robin over submitted teams; `+5/-5/0`
  points, floored at 0 (idempotent per day).
- **Run finals** — at season end, the top teams contest the Victors round; the
  winner gets a cosmetic **title** (no power).
- **New season** — roll to the next month, keeping the pod and carrying point
  totals + titles; teams, drafts, and results reset.

The same actions are available as `POST /api/admin/{run-day,advance-day,
reveal-beat,run-finals,reset-season}` if you prefer to script them.

## API summary

| Method | Path | Purpose |
|--------|------|---------|
| `POST` | `/api/join` | register/return a player; ensure the season |
| `GET`  | `/api/season` | schedule, clock, current beat, base budget |
| `GET`  | `/api/draft?player=` | revealed beats, offers, claims, unlocked pool/budget |
| `POST` | `/api/draft/claim` | claim a pick for the open beat |
| `POST`/`GET` | `/api/team` | submit/fetch a team (validated vs pool + budget) |
| `GET`  | `/api/standings` | league table |
| `GET`  | `/api/results?player=` | daily results |
| `GET`  | `/api/replays/{id}` | a stored replay's JSON |
| `GET`  | `/api/stats` | per-player W/L/D + points |
| `POST` | `/api/admin/run-day` · `advance-day` · `reveal-beat` · `run-finals` · `reset-season` | drive the season |

## Tests

```bash
cargo test   # from server/
```

This crate is a sibling of `battle_engine` with a path dependency (no Cargo
workspace), so the engine's target dir, WASM build, and replay tooling are
unaffected.
