use crate::forge::toml_parser::AetherToml;

/// Simple dependency resolution (no SAT solver for v1).
pub fn resolve_dependencies(config: &AetherToml) -> Vec<(String, String)> {
    config.dependencies.iter()
        .map(|(name, version)| (name.clone(), version.clone()))
        .collect()
}
