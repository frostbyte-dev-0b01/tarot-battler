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

    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);

    eprintln!("=== Team A ===");
    for c in team_a {
        eprintln!("  {} (row {})", c.base_name, c.position.row);
    }
    eprintln!("=== Team B ===");
    for c in team_b {
        eprintln!("  {} (row {})", c.base_name, c.position.row);
    }
    eprintln!("Seed: {}\n", seed);

    let battle = engine::BattleState::new(team_a, team_b, abilities, passives, statuses, seed);
    let log = battle.run();
    println!("{}", log.to_json());
}
