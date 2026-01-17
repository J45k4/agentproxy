use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PolicyConfig {
    pub tables: HashMap<String, TablePolicy>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TablePolicy {
    #[serde(default)]
    pub allow_ops: Vec<String>,
    #[serde(default)]
    pub required_filters: Vec<RequiredFilter>,
    #[serde(default)]
    pub deny_columns: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequiredFilter {
    pub column: String,
    #[serde(default = "default_operator")]
    pub operator: String,
}

fn default_operator() -> String {
    "=".to_string()
}

#[derive(Debug)]
pub enum PolicyError {
    ReadFailed(String),
    ParseFailed(String),
}

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyError::ReadFailed(error) => write!(f, "Failed to read policy file: {error}"),
            PolicyError::ParseFailed(error) => write!(f, "Failed to parse policy file: {error}"),
        }
    }
}

impl std::error::Error for PolicyError {}

pub fn load_policy(path: impl AsRef<Path>) -> Result<PolicyConfig, PolicyError> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .map_err(|err| PolicyError::ReadFailed(err.to_string()))?;

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("yaml") | Some("yml") => serde_yaml::from_str(&content)
            .map_err(|err| PolicyError::ParseFailed(err.to_string())),
        Some("json") => serde_json::from_str(&content)
            .map_err(|err| PolicyError::ParseFailed(err.to_string())),
        _ => serde_json::from_str(&content)
            .or_else(|_| serde_yaml::from_str(&content))
            .map_err(|err| PolicyError::ParseFailed(err.to_string())),
    }
}
