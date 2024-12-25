use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::{
    client::Client,
    server::{Server, Servers},
};

lazy_static! {
    pub static ref CONFIG: Config = Config::from_file();
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Config {
    client: Server,
}

impl Config {
    fn from_file() -> Self {
        let dirs = directories::ProjectDirs::from("", "", "ani-cli-es")
            .expect("Could not get the config dir");
        let config_dir = dirs.config_dir();

        let config = std::fs::read_to_string(config_dir.join("config.json")).unwrap_or({
            let default_config = serde_json::to_string_pretty(&Config::default())
                .expect("Could not serialize config");

            std::fs::create_dir_all(config_dir).expect("Could not create config dir");
            std::fs::write(config_dir.join("config.json"), &default_config)
                .expect("Could not write default config");
            default_config
        });

        serde_json::from_str(&config).expect("Could not parse config")
    }

    pub fn get_client(&self) -> Box<dyn Client> {
        Servers::generate_current_client(&self.client)
    }

    pub fn set_client(&self, client: Server) {
        let mut config_clone = self.clone();
        config_clone.client = client;

        let dirs = directories::ProjectDirs::from("", "", "ani-cli-es")
            .expect("Could not get the config dir");
        let config_dir = dirs.config_dir();

        let config =
            serde_json::to_string_pretty(&config_clone).expect("Could not serialize config");

        std::fs::write(config_dir.join("config.json"), &config).expect("Could not write config");
    }
}
