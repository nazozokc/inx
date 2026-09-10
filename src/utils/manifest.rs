use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Top-level inx.toml manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub package: PackageMetadata,

    #[serde(default)]
    pub dependencies: HashMap<String, String>,

    #[serde(default)]
    pub scripts: Scripts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub author: Option<String>,

    #[serde(default)]
    pub license: Option<String>,

    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Scripts {
    #[serde(rename = "pre-install", default)]
    pub pre_install: Option<String>,

    #[serde(rename = "post-install", default)]
    pub post_install: Option<String>,

    #[serde(rename = "pre-uninstall", default)]
    pub pre_uninstall: Option<String>,

    #[serde(rename = "post-uninstall", default)]
    pub post_uninstall: Option<String>,
}

impl Scripts {
    pub fn has_any(&self) -> bool {
        self.pre_install.is_some()
            || self.post_install.is_some()
            || self.pre_uninstall.is_some()
            || self.post_uninstall.is_some()
    }
}

/// Legacy package.json (backward compat).
#[derive(Debug, Deserialize)]
struct LegacyPackageJson {
    name: String,
    version: String,
    description: Option<String>,
    author: Option<serde_json::Value>,
    license: Option<String>,
}

impl PackageManifest {
    /// Read manifest from a package directory.
    /// Prefers inx.toml; falls back to package.json.
    pub fn from_dir(dir: &Path) -> Result<Self> {
        let inx_toml = dir.join("inx.toml");
        let package_json = dir.join("package.json");

        if inx_toml.exists() {
            return Self::from_toml(&inx_toml);
        }

        if package_json.exists() {
            return Self::from_package_json(&package_json);
        }

        anyhow::bail!(
            "No manifest found in '{}': expected inx.toml or package.json",
            dir.display()
        );
    }

    /// Parse inx.toml.
    fn from_toml(path: &Path) -> Result<Self> {
        let data = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let manifest: PackageManifest = toml::from_str(&data)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(manifest)
    }

    /// Parse legacy package.json into a PackageManifest.
    fn from_package_json(path: &Path) -> Result<Self> {
        let data = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let legacy: LegacyPackageJson = serde_json::from_str(&data)
            .with_context(|| format!("Failed to parse {}", path.display()))?;

        let author_str = legacy.author.map(|v| match v {
            serde_json::Value::String(s) => s,
            serde_json::Value::Object(m) => {
                m.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string()
            }
            _ => "unknown".to_string(),
        });

        Ok(PackageManifest {
            package: PackageMetadata {
                name: legacy.name,
                version: legacy.version,
                description: legacy.description,
                author: author_str.filter(|a| !a.is_empty() && a != "unknown"),
                license: legacy.license,
                tags: None,
            },
            dependencies: HashMap::new(),
            scripts: Scripts::default(),
        })
    }
}
