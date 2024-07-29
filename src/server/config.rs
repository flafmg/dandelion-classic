use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub addr: String,
    pub port: u16,
    pub heartbeat_url: String,
    pub name: String,
    pub motd: String,
    pub public: bool,
    pub do_user_auth: bool,
    pub max_players: u32,
    pub default_map: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            addr: "0.0.0.0".to_string(),
            port: 25565,
            heartbeat_url: "https://www.classicube.net/server/heartbeat".to_string(),
            name: "A classic server".to_string(),
            motd: "dandelion powered".to_string(),
            public: true,
            do_user_auth: true,
            max_players: 64,
            default_map: "default".to_string(),
        }
    }
}

impl Config {
    pub fn load(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if Path::new(file_path).exists() {
            let config_content = fs::read_to_string(file_path)?;
            let config = serde_yaml::from_str(&config_content)?;
            Ok(config)
        } else {
            let default_config = Config::default();
            let config_yaml = serde_yaml::to_string(&default_config)?;
            let mut file = File::create(file_path)?;
            file.write_all(config_yaml.as_bytes())?;
            Ok(default_config)
        }
    }
}
