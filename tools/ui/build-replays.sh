#!/usr/bin/env bash
#
# Regenerate the sample battle replays in tools/ui/sample-data/replays/.
#
# Discovers every valid team in tools/ui/sample-data/teams/ at run time (no
# hardcoded list) and plays a full round-robin — each unordered pair once —
# through the release engine, writing one replay per matchup named
# "<winner>_defeats_<loser>.json" (or "<a>_draws_<b>.json" on a draw).
#
# Stale replays are cleared first, so the output is always a coherent,
# reproducible set that stays honest against the current engine. Each matchup
# uses a deterministic per-pair seed, so reruns produce identical files.
#
# It also rewrites sample-data/teams.json, the manifest the static UI reads to
# seed its bundled roster (a browser can't enumerate the teams/ directory).
#
# Usage: tools/ui/build-replays.sh
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

echo "Building release engine..."
(cd battle_engine && cargo build --release --quiet)
engine="$repo_root/battle_engine/target/release/battle_engine"

python3 - "$engine" <<'PY'
import itertools
import json
import os
import subprocess
import sys
import tempfile
import zlib

engine = sys.argv[1]
teams_dir = "tools/ui/sample-data/teams"
replays_dir = "tools/ui/sample-data/replays"
os.makedirs(replays_dir, exist_ok=True)

stems = sorted(f[:-5] for f in os.listdir(teams_dir) if f.endswith(".json"))


def team_path(stem):
    return os.path.join(teams_dir, stem + ".json")


def run(team_a, team_b, seed):
    """Run one battle and return (replay_dict, error). --json prints the replay
    to stdout; --json-out redirects the engine's file write to a throwaway temp
    so the gitignored latest_replay.json is left untouched."""
    with tempfile.NamedTemporaryFile(suffix=".json") as tmp:
        proc = subprocess.run(
            [engine,
             "--team-a", team_path(team_a),
             "--team-b", team_path(team_b),
             "--json", "--json-out", tmp.name, str(seed)],
            capture_output=True, text=True,
        )
    if proc.returncode != 0:
        return None, proc.stderr.strip()
    return json.loads(proc.stdout), None


# Validity probe: a team plays itself. If that runs, the team is usable.
valid = []
for stem in stems:
    replay, err = run(stem, stem, 42)
    if replay is None:
        last = err.splitlines()[-1] if err else "failed"
        print(f"  skip {stem}: {last}")
    else:
        valid.append(stem)

print(f"Valid teams ({len(valid)}): {', '.join(valid)}")

# Rewrite the UI roster manifest from the same scan, so it never drifts from
# the bundled team files.
manifest_path = "tools/ui/sample-data/teams.json"
with open(manifest_path, "w") as fh:
    json.dump([f"{stem}.json" for stem in valid], fh, indent=2)
    fh.write("\n")
print(f"Wrote {manifest_path} ({len(valid)} teams)")

# Clear stale replays so phantom teams and contradictory both-direction
# duplicates do not linger.
for name in os.listdir(replays_dir):
    if name.endswith(".json"):
        os.remove(os.path.join(replays_dir, name))

written = 0
for team_a, team_b in itertools.combinations(valid, 2):
    # Deterministic per-pair seed: reproducible across runs, varied per match.
    seed = zlib.crc32(f"{team_a}|{team_b}".encode()) % 100000
    replay, err = run(team_a, team_b, seed)
    if replay is None:
        print(f"  FAILED {team_a} vs {team_b}: {err}")
        continue
    winner = replay["winner"]
    if winner == "team_a":
        name = f"{team_a}_defeats_{team_b}.json"
    elif winner == "team_b":
        name = f"{team_b}_defeats_{team_a}.json"
    else:
        name = f"{team_a}_draws_{team_b}.json"
    with open(os.path.join(replays_dir, name), "w") as fh:
        json.dump(replay, fh, indent=2)
        fh.write("\n")
    written += 1

print(f"Wrote {written} replay(s) to {replays_dir}/")
PY
