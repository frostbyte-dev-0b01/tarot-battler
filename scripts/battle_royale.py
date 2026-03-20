#!/usr/bin/env python3

from __future__ import annotations

import argparse
import itertools
import json
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


@dataclass
class Record:
    wins: int = 0
    losses: int = 0
    draws: int = 0


@dataclass
class TeamSummary:
    wins: int = 0
    losses: int = 0
    draws: int = 0
    played: int = 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run a round-robin battle royale across all sample teams.",
    )
    parser.add_argument(
        "--rounds",
        type=int,
        default=5,
        help="Matches per pairing. Defaults to 5.",
    )
    return parser.parse_args()


def load_team_names(teams_dir: Path) -> dict[str, str]:
    team_names: dict[str, str] = {}
    for path in sorted(teams_dir.glob("*.json")):
        payload = json.loads(path.read_text())
        team_names[path.stem] = payload["name"]
    return team_names


def sanitize_name(name: str) -> str:
    normalized = "".join(char.lower() if char.isalnum() else "_" for char in name)
    parts = [part for part in normalized.split("_") if part]
    return "_".join(parts) or "unknown_team"


def run_match(
    repo_root: Path,
    team_a: str,
    team_b: str,
    seed: int,
    replay_out_path: Path | None = None,
) -> str:
    command = [
        "cargo",
        "run",
        "-q",
        "--",
        "--teams",
        team_a,
        team_b,
    ]
    if replay_out_path is not None:
        command.extend(["--json-out", str(replay_out_path)])
    command.append(str(seed))
    result = subprocess.run(
        command,
        cwd=repo_root / "battle_engine",
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout.strip()


def parse_result(line: str, team_a_name: str, team_b_name: str) -> str:
    if line == "draw":
        return "draw"
    if line == f"{team_a_name} defeats {team_b_name}":
        return team_a_name
    if line == f"{team_b_name} defeats {team_a_name}":
        return team_b_name
    raise RuntimeError(f"Unexpected battle result: {line!r}")


def build_table(headers: list[str], rows: list[list[str]]) -> str:
    widths = [len(header) for header in headers]
    for row in rows:
        for idx, value in enumerate(row):
            widths[idx] = max(widths[idx], len(value))

    def format_row(values: list[str]) -> str:
        return " | ".join(value.ljust(widths[idx]) for idx, value in enumerate(values))

    parts = [format_row(headers), "-+-".join("-" * width for width in widths)]
    parts.extend(format_row(row) for row in rows)
    return "\n".join(parts)


def main() -> int:
    args = parse_args()
    repo_root = Path(__file__).resolve().parents[1]
    teams_dir = repo_root / "tools" / "ui" / "sample-data" / "teams"
    replays_dir = repo_root / "tools" / "ui" / "sample-data" / "replays"
    replays_dir.mkdir(parents=True, exist_ok=True)
    temp_replay_path = replays_dir / "__battle_royale_tmp.json"
    teams = load_team_names(teams_dir)
    team_keys = sorted(teams)
    if len(team_keys) < 2:
        print("Need at least two team JSON files in tools/ui/sample-data/teams", file=sys.stderr)
        return 1

    rounds = args.rounds
    if rounds < 1:
        print("--rounds must be at least 1", file=sys.stderr)
        return 1
    pair_records: dict[tuple[str, str], Record] = {
        pair: Record() for pair in itertools.combinations(team_keys, 2)
    }
    totals: dict[str, TeamSummary] = {name: TeamSummary() for name in team_keys}

    for pair_index, (team_one, team_two) in enumerate(itertools.combinations(team_keys, 2)):
        for round_index in range(rounds):
            if round_index % 2 == 0:
                team_a, team_b = team_one, team_two
            else:
                team_a, team_b = team_two, team_one
            seed = 1000 + pair_index * rounds + round_index
            replay_out_path = temp_replay_path if round_index == 0 else None
            winner = parse_result(
                run_match(repo_root, team_a, team_b, seed, replay_out_path),
                teams[team_a],
                teams[team_b],
            )
            if round_index == 0 and temp_replay_path.exists():
                if winner == "draw":
                    replay_name = (
                        f"{sanitize_name(teams[team_one])}_draws_with_"
                        f"{sanitize_name(teams[team_two])}.json"
                    )
                else:
                    loser = teams[team_b] if winner == teams[team_a] else teams[team_a]
                    replay_name = (
                        f"{sanitize_name(winner)}_defeats_{sanitize_name(loser)}.json"
                    )
                shutil.copyfile(temp_replay_path, replays_dir / replay_name)
            pair_record = pair_records[(team_one, team_two)]
            totals[team_one].played += 1
            totals[team_two].played += 1
            if winner == "draw":
                pair_record.draws += 1
                totals[team_one].draws += 1
                totals[team_two].draws += 1
            elif winner == teams[team_one]:
                pair_record.wins += 1
                totals[team_one].wins += 1
                totals[team_two].losses += 1
            else:
                pair_record.losses += 1
                totals[team_one].losses += 1
                totals[team_two].wins += 1

    headers = ["team", *(teams[key] for key in team_keys), "W", "L", "D"]
    rows: list[list[str]] = []
    for row_team in team_keys:
        row = [teams[row_team]]
        for col_team in team_keys:
            if row_team == col_team:
                row.append("—")
                continue
            pair = tuple(sorted((row_team, col_team)))
            record = pair_records[pair]
            if pair[0] == row_team:
                cell = f"{record.wins}-{record.losses}-{record.draws}"
            else:
                cell = f"{record.losses}-{record.wins}-{record.draws}"
            row.append(cell)
        summary = totals[row_team]
        row.extend([str(summary.wins), str(summary.losses), str(summary.draws)])
        rows.append(row)

    print(f"Battle royale across {len(team_keys)} teams ({rounds} matches per pairing)\n")
    print(build_table(headers, rows))
    if temp_replay_path.exists():
        temp_replay_path.unlink()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
