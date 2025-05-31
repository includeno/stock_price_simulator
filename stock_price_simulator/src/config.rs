use serde::{Serialize, Deserialize};
use std::fs;
use anyhow::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct GlobalConfig {
    pub random_seed: Option<u64>,
    pub simulation_period_days: u32,
    pub time_step_minutes: u64,
}

pub fn load_config(file_path: &str) -> Result<GlobalConfig, Error> {
    let contents = fs::read_to_string(file_path)?;
    let config: GlobalConfig = toml::from_str(&contents)?;
    Ok(config)
}
