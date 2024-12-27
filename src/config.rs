use std::env;
use std::fs::File;
use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<AppConfig> {
        // read from ./app.yml or /etc/config/app.yml or from env CHAT_CONFIG
        let ret = match (
                File::open("app.yml"),
                File::open("/etc/config/app.yml"),
                env::var("CHAT_CONFIG")
        ) {
            (Ok(reader), _, _) => serde_yaml::from_reader(reader)?,
            (_,Ok(reader),_) => serde_yaml::from_reader(reader)?,
            (_,_,Ok(path)) => serde_yaml::from_reader(File::open(path)?)?,
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
        let app_config =  AppConfig::load();
        assert!(app_config.is_ok());
        println!("{:#?}", app_config.unwrap());
    }
}