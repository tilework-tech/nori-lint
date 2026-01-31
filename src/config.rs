use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub anthropic_api_key: String,
}

pub fn load_config(path: &Path) -> Result<Config, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file at {}: {e}", path.display()))?;

    let config: Config = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config file at {}: {e}", path.display()))?;

    if config.anthropic_api_key.trim().is_empty() {
        return Err("anthropic_api_key must not be empty or blank".to_string());
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn loads_valid_config() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.json");
        fs::write(
            &config_path,
            r#"{"anthropic_api_key": "sk-ant-test-key-123"}"#,
        )
        .unwrap();

        let config = load_config(&config_path).unwrap();
        assert_eq!(config.anthropic_api_key, "sk-ant-test-key-123");
    }

    #[test]
    fn returns_error_for_missing_file() {
        let result = load_config(Path::new("/nonexistent/config.json"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("read") || err.contains("Read") || err.contains("No such file"),
            "error should mention read failure, got: {err}"
        );
    }

    #[test]
    fn returns_error_for_malformed_json() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.json");
        fs::write(&config_path, "not valid json {{{").unwrap();

        let result = load_config(&config_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("parse") || err.contains("Parse") || err.contains("expected"),
            "error should mention parse failure, got: {err}"
        );
    }

    #[test]
    fn returns_error_for_missing_api_key_field() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.json");
        fs::write(&config_path, r#"{"some_other_field": "value"}"#).unwrap();

        let result = load_config(&config_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("anthropic_api_key"),
            "error should mention the missing field, got: {err}"
        );
    }

    #[test]
    fn returns_error_for_empty_api_key() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.json");
        fs::write(&config_path, r#"{"anthropic_api_key": ""}"#).unwrap();

        let result = load_config(&config_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("empty") || err.contains("blank"),
            "error should mention empty key, got: {err}"
        );
    }

    #[test]
    fn returns_error_for_whitespace_only_api_key() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.json");
        fs::write(&config_path, r#"{"anthropic_api_key": "   "}"#).unwrap();

        let result = load_config(&config_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("empty") || err.contains("blank"),
            "error should mention empty/blank key, got: {err}"
        );
    }
}
