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
    pub(crate) os: Vec<String>,
    pub(crate) arch: Vec<String>,
    pub(crate) provided_widgets: Vec<String>,
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
