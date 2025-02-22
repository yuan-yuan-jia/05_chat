use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::{env, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub sk: String,
    pub pk: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub db_url: String,
    pub base_dir: PathBuf,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<AppConfig> {
        // read from ./chat.yml or /etc/config/chat.yml or from env CHAT_CONFIG
        let ret = match (
            File::open("chat.yml"),
            File::open("/etc/config/chat.yml"),
            env::var("CHAT_CONFIG"),
        ) {
            (Ok(reader), _, _) => serde_yaml::from_reader(reader)?,
            (_, Ok(reader), _) => serde_yaml::from_reader(reader)?,
            (_, _, Ok(path)) => serde_yaml::from_reader(File::open(path)?)?,
            _ => bail!("Config file not found"),
        };

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_load() {
        let app_config = AppConfig::load();
        assert!(app_config.is_ok());
        println!("{:#?}", app_config.unwrap());
    }
}
