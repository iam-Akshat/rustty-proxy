use serde::{Deserialize, Serialize};
use std::fs::read_to_string;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ProxyConfig {
    pub name: String,
    pub ports: Vec<u16>,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ProxyConfigFile {
    pub apps: Vec<ProxyConfig>,
}

pub fn get_config(path: String) -> ProxyConfigFile {
    let string = read_to_string(path).unwrap_or_else({
        |e| {
            panic!("Error reading config file: {}", e);
        }
    });
    let config: ProxyConfigFile = serde_json::from_str(&string).unwrap_or_else({
        |e| {
            println!("Error: {}", e);
            panic!("Error parsing config file");
        }
    });
    config
}
