use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct Config {
    log_file_path: String,
    simulation_delay_ms: u32,
}

fn load(path: &Path) -> Result<Config> {
    let config_file_content = std::fs::read_to_string(path).context("Failed to read to string")?;
    let config = toml::from_str::<Config>(&config_file_content).context("Failed to deserialize")?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_succeeds_with_valid_toml() {
        let s = load(Path::new("src/config.toml")).unwrap();
        assert_eq!(s.log_file_path, "data/sample_navigation.log");
        assert_eq!(s.simulation_delay_ms, 500);
    }

    #[test]
    fn load_fails_with_missing_toml() {
        let s = load(Path::new("src/truc.toml"));
        assert!(s.is_err());
    }

    #[test]
    fn load_fails_with_incorrect_toml() {
        let s = load(Path::new("src/config_wrong.toml"));
        assert!(s.is_err());
    }
}
