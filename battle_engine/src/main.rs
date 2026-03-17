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

    let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/data");
    let characters = loader::load_characters(&data_dir.join("characters.json"))
        .expect("Failed to load characters");
    let abilities = loader::load_abilities(&data_dir.join("abilities.json"))
        .expect("Failed to load abilities");
    let passives = loader::load_passives(&data_dir.join("passives.json"))
        .expect("Failed to load passives");
    let statuses = loader::load_statuses(&data_dir.join("statuses.json"))
        .expect("Failed to load statuses");
    loader::validate_content(&characters, &abilities, &passives, &statuses)
        .expect("Invalid battle content");

    // First 5 characters = Team A, last 5 = Team B
    let (team_a, team_b) = characters.split_at(characters.len() / 2);

    let seed: u64 = args.iter()
        .find_map(|s| s.parse().ok())
        .unwrap_or(42);

    eprintln!("=== Team A ===");
    for c in team_a {
        eprintln!("  {} (row {}, col {})", c.base_name, c.position.row, c.position.col);
    }
    eprintln!("=== Team B ===");
    for c in team_b {
        eprintln!("  {} (row {}, col {})", c.base_name, c.position.row, c.position.col);
    }
    eprintln!("Seed: {}\n", seed);

    let battle = engine::BattleState::new(team_a, team_b, abilities, passives, statuses, seed);
    let log = battle.run();
    if json_output {
        println!("{}", log.to_json());
    } else {
        println!("{}", log.to_text());
    }
}
