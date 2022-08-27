use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct PluginManifest {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) version: String,
    pub(crate) author: Option<String>,
    pub(crate) authors: Option<Vec<String>>,
    pub(crate) license: Option<String>,
    pub(crate) os: Option<Vec<String>>,
    pub(crate) arch: Option<Vec<String>>,
    pub(crate) provided_widgets: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct RepositoryManifest {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) url: String,
    // Not currently supported.
    pub(crate) fallback_url: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct RepositoryPlugin {
    pub(crate) plugins: Vec<String>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct RtopConfig {
    pages: Vec<Vec<String>>,
    pub(crate) plugins: Vec<RtopConfigPlugins>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct RtopConfigPlugins {
    pub(crate) path: String,
    pub(crate) provided_widgets: Vec<String>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct RTPMConfig {
    pub repositories: Vec<String>,
    pub plugins: Vec<RTPMConfigPluginElement>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct RTPMConfigPluginElement {
    pub name: String,
    pub version: String,
    pub repo: String,
}
