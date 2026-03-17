mod abilities;
mod damage;
mod engine;
mod loader;
mod logger;
mod models;
mod rules;
mod statuses;
mod targeting;

use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let json_output = args.iter().any(|arg| arg == "--json");
    let mut team_a_path: Option<String> = None;
    let mut team_b_path: Option<String> = None;
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
            _ => i += 1,
        }
    }

    let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/data");
    let abilities =
        loader::load_abilities(&data_dir.join("abilities.json")).expect("Failed to load abilities");
    let passives =
        loader::load_passives(&data_dir.join("passives.json")).expect("Failed to load passives");
    let statuses =
        loader::load_statuses(&data_dir.join("statuses.json")).expect("Failed to load statuses");
    let (team_a_name, team_b_name, team_a, team_b) = match (team_a_path, team_b_path) {
        (Some(team_a_path), Some(team_b_path)) => {
            let team_a_config = loader::load_team_config(Path::new(&team_a_path))
                .expect("Failed to load team A");
            let team_b_config = loader::load_team_config(Path::new(&team_b_path))
                .expect("Failed to load team B");
            let team_a = loader::validate_team_config(&team_a_config, &abilities, &passives, &statuses)
                .expect("Invalid team A content");
            let team_b = loader::validate_team_config(&team_b_config, &abilities, &passives, &statuses)
                .expect("Invalid team B content");
            loader::validate_teams(&team_a, &team_b, &abilities, &passives, &statuses)
                .expect("Invalid battle content");
            (team_a_config.name, team_b_config.name, team_a, team_b)
        }
        (None, None) => {
            let characters = loader::load_characters(&data_dir.join("characters.json"))
                .expect("Failed to load characters");
            loader::validate_content(&characters, &abilities, &passives, &statuses)
                .expect("Invalid battle content");
            let (team_a, team_b) = characters.split_at(characters.len() / 2);
            (
                "Team A".to_string(),
                "Team B".to_string(),
                team_a.to_vec(),
                team_b.to_vec(),
            )
        }
        _ => panic!("--team-a and --team-b must be provided together"),
    };

    let seed: u64 = args.iter().find_map(|s| s.parse().ok()).unwrap_or(42);

    eprintln!("=== {} ===", team_a_name);
    for c in &team_a {
        eprintln!(
            "  {} (row {}, col {})",
            c.base_name, c.position.row, c.position.col
        );
    }
    eprintln!("=== {} ===", team_b_name);
    for c in &team_b {
        eprintln!(
            "  {} (row {}, col {})",
            c.base_name, c.position.row, c.position.col
        );
    }
    eprintln!("Seed: {}\n", seed);

    let battle = engine::BattleState::new(&team_a, &team_b, abilities, passives, statuses, seed);
    let log = battle.run();
    if json_output {
        println!("{}", log.to_json());
    } else {
        println!("{}", log.to_text());
    }
}
