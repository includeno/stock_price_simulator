use serde::{Serialize, Deserialize};
use std::fs;
use anyhow::Error;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ModelType {
    GeometricBrownianMotion,
    // Future models: Heston, JumpDiffusion, etc.
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GeometricBrownianMotionParams {
    pub drift: f64,
    pub volatility: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ModelParameters {
    // Optional fields for each model type
    pub gbm: Option<GeometricBrownianMotionParams>,
    // heston: Option<HestonParams>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AssetModelConfig {
    pub asset_type: String, // e.g., "stock", "option", "future"
    pub asset_identifier_pattern: String, // e.g., "AAPL", "DEFAULT_STOCK", "ESZ24"
    pub default_model: ModelType,
    pub parameters: ModelParameters,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GlobalConfig {
    pub random_seed: Option<u64>,
    pub simulation_period_days: u32, // Retained for now
    pub time_step_minutes: u64,      // Retained for now
    pub asset_models: Option<Vec<AssetModelConfig>>, // Changed to Option for backward compatibility if file missing this
}

pub fn load_config(file_path: &str) -> Result<GlobalConfig, Error> {
    let contents = fs::read_to_string(file_path)?;
    let config: GlobalConfig = toml::from_str(&contents)?;
    Ok(config)
}
