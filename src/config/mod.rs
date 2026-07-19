use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub devices: Vec<DeviceConfig>,
    /// Highest-priority first; config.toml uses the `ordered_recognised_processes` key.
    #[serde(default, alias = "ordered_recognised_processes")]
    pub recognised_processes: Vec<String>,
    pub ts6_api_key: Option<String>,
    pub ts6_self_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceConfig {
    pub name: Option<String>,
    pub vid: u16,
    pub pid: u16,
    pub usage_page: Option<u16>,
    pub usage: Option<u16>,
}
