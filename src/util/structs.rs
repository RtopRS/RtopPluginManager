use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PluginManifest {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) version: String,
    pub(crate) url: String,
    pub(crate) author: Option<String>,
    pub(crate) authors: Option<Vec<String>>,
    pub(crate) license: Option<String>,
    pub(crate) os: Option<Vec<String>>,
    pub(crate) arch: Option<Vec<String>>,
    pub(crate) provided_widgets: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RepositoryManifest {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) url: String,
    // Not currently supported.
    pub(crate) fallback_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RepositoryPlugin {
    pub(crate) plugins: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RtopConfig {
    pages: Vec<Vec<String>>,
    pub(crate) plugins: Vec<RtopConfigPlugins>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RtopConfigPlugins {
    pub(crate) path: String,
    pub(crate) provided_widgets: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RTPMConfig {
    pub repositories: Vec<String>,
    pub plugins: Vec<RTPMConfigPluginElement>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RTPMConfigPluginElement {
    pub id: String,
    pub name: String,
    pub version: String,
    pub repo: String,
    pub plugin_type: i8,
}
