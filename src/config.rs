use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "config.json";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub debounce_time: u32,
}

pub fn load_config() -> Result<Config> {
    let mut file = File::open(CONFIG_FILE)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: Config = serde_json::from_str(&contents)?;
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(CONFIG_FILE)?;
    let json = serde_json::to_string(config)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}
