mod damage;
mod engine;
mod loader;
mod logger;
mod models;
mod targeting;

use std::path::Path;

fn main() {
    let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/data");
    let characters = loader::load_characters(&data_dir.join("characters.json"))
        .expect("Failed to load characters");

    // Split into two teams: first two vs last two
    let (team_a, team_b) = characters.split_at(characters.len() / 2);

    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);

    let battle = engine::BattleState::new(team_a, team_b, seed);
    let log = battle.run();
    println!("{}", log.to_json());
}
