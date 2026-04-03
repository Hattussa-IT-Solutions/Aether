use serde::Deserialize;

/// Aether project configuration (aether.toml).
#[derive(Debug, Deserialize)]
pub struct AetherToml {
    pub project: ProjectConfig,
    #[serde(default)]
    pub dependencies: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
}

/// Parse an aether.toml file.
pub fn parse_aether_toml(content: &str) -> Result<AetherToml, String> {
    toml::from_str(content).map_err(|e| format!("Failed to parse aether.toml: {}", e))
}
