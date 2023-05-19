use std::path::Path;

use netvr_data::Pose;
use serde::{Deserialize, Serialize};
use tokio::fs;
use xr_layer::log::LogWarn;

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct Config {
    pub name: String,
    #[serde(default)]
    pub server_space_pose: Pose,

    #[serde(skip)]
    data_directory: String,
}

impl Config {
    pub async fn load(data_directory: String) -> Self {
        let Ok(config) = fs::read_to_string(Path::new(&data_directory).join("config.json")).await else {
            return Self {
                data_directory,
                ..Self::default()
            };
        };
        match serde_json::from_str(&config) {
            Ok(config) => Self {
                data_directory,
                ..config
            },
            Err(err) => {
                LogWarn::string(format!("Failed to parse config.json: {:?}", err));
                Self {
                    data_directory,
                    ..Self::default()
                }
            }
        }
    }

    pub async fn write(&self) {
        let config = serde_json::to_string(&self).unwrap();
        if let Err(err) =
            fs::write(Path::new(&self.data_directory).join("config.json"), config).await
        {
            println!("Failed to write config.json: {:?}", err)
        }
    }
}
