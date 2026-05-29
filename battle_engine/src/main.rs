use battle_engine::{engine, loader};

use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let json_output = args.iter().any(|arg| arg == "--json");
    let mut team_a_path: Option<String> = None;
    let mut team_b_path: Option<String> = None;
    let mut named_teams: Option<(String, String)> = None;
    let mut json_out_path: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--team-a" => {
                let path = args
                    .get(i + 1)
                    .unwrap_or_else(|| panic!("--team-a requires a file path"));
                team_a_path = Some(path.clone());
                i += 2;
            }
            "--team-b" => {
                let path = args
                    .get(i + 1)
                    .unwrap_or_else(|| panic!("--team-b requires a file path"));
                team_b_path = Some(path.clone());
                i += 2;
            }
            "--teams" => {
                let team_a_name = args
                    .get(i + 1)
                    .unwrap_or_else(|| panic!("--teams requires two team names"));
                let team_b_name = args
                    .get(i + 2)
                    .unwrap_or_else(|| panic!("--teams requires two team names"));
                named_teams = Some((team_a_name.clone(), team_b_name.clone()));
                i += 3;
            }
            "--json-out" => {
                let path = args
                    .get(i + 1)
                    .unwrap_or_else(|| panic!("--json-out requires a file path"));
                json_out_path = Some(path.clone());
                i += 2;
            }
            _ => i += 1,
        }
    }

    let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/data");
    let archetypes =
        loader::load_archetypes(&data_dir.join("archetypes.json")).expect("Failed to load archetypes");
    let abilities =
        loader::load_abilities(&data_dir.join("abilities.json")).expect("Failed to load abilities");
    let passives =
        loader::load_passives(&data_dir.join("passives.json")).expect("Failed to load passives");
    let statuses =
        loader::load_statuses(&data_dir.join("statuses.json")).expect("Failed to load statuses");
    let aspects =
        loader::load_aspects(&data_dir.join("aspects.json")).expect("Failed to load aspects");
    let (team_a_name, team_b_name, team_a, team_b) = match (team_a_path, team_b_path, named_teams)
    {
        (Some(_), Some(_), Some(_)) | (Some(_), None, Some(_)) | (None, Some(_), Some(_)) => {
            panic!("Use either --team-a/--team-b or --teams, not both")
        }
        (Some(team_a_path), Some(team_b_path), None) => {
            let team_a_config = loader::load_team_config(Path::new(&team_a_path))
                .expect("Failed to load team A");
            let team_b_config = loader::load_team_config(Path::new(&team_b_path))
                .expect("Failed to load team B");
            let team_a = loader::validate_team_config(
                &team_a_config,
                &archetypes,
                &aspects,
                &abilities,
                &passives,
                &statuses,
            )
            .expect("Invalid team A content");
            let team_b = loader::validate_team_config(
                &team_b_config,
                &archetypes,
                &aspects,
                &abilities,
                &passives,
                &statuses,
            )
            .expect("Invalid team B content");
            loader::validate_teams(&team_a, &team_b, &abilities, &passives, &statuses)
                .expect("Invalid battle content");
            (team_a_config.name, team_b_config.name, team_a, team_b)
        }
        (None, None, Some((team_a_name, team_b_name))) => {
            let team_a_path = resolve_named_team_path(&team_a_name);
            let team_b_path = resolve_named_team_path(&team_b_name);
            let team_a_config = loader::load_team_config(&team_a_path)
                .unwrap_or_else(|_| panic!("Failed to load team A from {}", team_a_path.display()));
            let team_b_config = loader::load_team_config(&team_b_path)
                .unwrap_or_else(|_| panic!("Failed to load team B from {}", team_b_path.display()));
            let team_a = loader::validate_team_config(
                &team_a_config,
                &archetypes,
                &aspects,
                &abilities,
                &passives,
                &statuses,
            )
            .expect("Invalid team A content");
            let team_b = loader::validate_team_config(
                &team_b_config,
                &archetypes,
                &aspects,
                &abilities,
                &passives,
                &statuses,
            )
            .expect("Invalid team B content");
            loader::validate_teams(&team_a, &team_b, &abilities, &passives, &statuses)
                .expect("Invalid battle content");
            (team_a_config.name, team_b_config.name, team_a, team_b)
        }
        (None, None, None) => {
            let team_a_path = resolve_named_team_path("imperial_phalanx");
            let team_b_path = resolve_named_team_path("omen_engine");
            let team_a_config = loader::load_team_config(&team_a_path)
                .unwrap_or_else(|_| panic!("Failed to load team A from {}", team_a_path.display()));
            let team_b_config = loader::load_team_config(&team_b_path)
                .unwrap_or_else(|_| panic!("Failed to load team B from {}", team_b_path.display()));
            let team_a = loader::validate_team_config(
                &team_a_config,
                &archetypes,
                &aspects,
                &abilities,
                &passives,
                &statuses,
            )
            .expect("Invalid team A content");
            let team_b = loader::validate_team_config(
                &team_b_config,
                &archetypes,
                &aspects,
                &abilities,
                &passives,
                &statuses,
            )
            .expect("Invalid team B content");
            loader::validate_teams(&team_a, &team_b, &abilities, &passives, &statuses)
                .expect("Invalid battle content");
            (team_a_config.name, team_b_config.name, team_a, team_b)
        }
        _ => panic!("--team-a and --team-b must be provided together"),
    };

    let seed: u64 = args.iter().find_map(|s| s.parse().ok()).unwrap_or(42);

    let battle = engine::BattleState::new(&team_a, &team_b, abilities, passives, statuses, seed);
    let log = battle.run();
    let replay_json =
        log.to_replay_json(seed, &team_a_name, &team_b_name, &team_a, &team_b);
    let replay_path = json_out_path
        .map(PathBuf::from)
        .unwrap_or_else(default_replay_output_path);
    write_replay_json(&replay_path, &replay_json);

    if json_output {
        println!("{replay_json}");
    } else {
        match log.winner_key() {
            "team_a" => println!("{team_a_name} defeats {team_b_name}"),
            "team_b" => println!("{team_b_name} defeats {team_a_name}"),
            _ => println!("draw"),
        }
    }
}

fn default_replay_output_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../tools/ui/sample-data/latest_replay.json")
}

fn resolve_named_team_path(name: &str) -> PathBuf {
    let filename = if name.ends_with(".json") {
        name.to_string()
    } else {
        format!("{name}.json")
    };
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../tools/ui/sample-data/teams")
        .join(filename)
}

fn write_replay_json(path: &Path, replay_json: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|err| panic!("Failed to create {}: {}", parent.display(), err));
    }
    std::fs::write(path, replay_json)
        .unwrap_or_else(|err| panic!("Failed to write {}: {}", path.display(), err));
}
