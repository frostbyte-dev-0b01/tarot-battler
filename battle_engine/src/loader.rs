//! Load character configurations and ability definitions from JSON files.

use std::path::Path;

use crate::abilities::AbilityMap;
use crate::models::CharacterConfig;

pub fn load_characters(path: &Path) -> Result<Vec<CharacterConfig>, String> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

pub fn load_abilities(path: &Path) -> Result<AbilityMap, String> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Stat;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn load_characters_from_bundled_file() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/data/characters.json");
        let chars = load_characters(&path).unwrap();
        assert!(chars.len() >= 2);
        assert_eq!(chars[0].base_name, "The Emperor");
        assert_eq!(*chars[0].stats.get(&Stat::CON).unwrap(), 10);
    }

    #[test]
    fn load_characters_error_on_missing_file() {
        let result = load_characters(Path::new("nonexistent.json"));
        assert!(result.is_err());
    }

    #[test]
    fn load_characters_error_on_invalid_json() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "not valid json").unwrap();
        let result = load_characters(tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn roundtrip_character_config() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/data/characters.json");
        let chars = load_characters(&path).unwrap();
        let json = serde_json::to_string_pretty(&chars).unwrap();
        let reloaded: Vec<CharacterConfig> = serde_json::from_str(&json).unwrap();
        assert_eq!(chars.len(), reloaded.len());
        for (a, b) in chars.iter().zip(reloaded.iter()) {
            assert_eq!(a.base_name, b.base_name);
            assert_eq!(a.stats, b.stats);
        }
    }
}
