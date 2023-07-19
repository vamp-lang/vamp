use serde::{self, Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub package: Package,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Package {
    pub name: Option<String>,
    pub version: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default = "default_root")]
    pub root: String,
    #[serde(default = "default_entry")]
    pub entry: String,
}

fn default_root() -> String {
    "src".into()
}

fn default_entry() -> String {
    "main.vamp".into()
}
