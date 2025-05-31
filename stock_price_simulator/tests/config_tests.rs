use stock_price_simulator::config::load_config;
use std::io::Write;
use tempfile::NamedTempFile;

// Only ModelType is directly used in assertions after GlobalConfig.
// GlobalConfig itself is used via load_config.
use stock_price_simulator::config::ModelType;

#[test]
fn test_load_config_full() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let toml_content = r#"
random_seed = 42
simulation_period_days = 60
time_step_minutes = 10

[[asset_models]]
asset_type = "stock"
asset_identifier_pattern = "DEFAULT_STOCK"
default_model = "GeometricBrownianMotion"
[asset_models.parameters.gbm]
drift = 0.05
volatility = 0.2

[[asset_models]]
asset_type = "stock"
asset_identifier_pattern = "TECH_STOCK"
default_model = "GeometricBrownianMotion"
[asset_models.parameters.gbm]
drift = 0.1
volatility = 0.3
"#;
    temp_file.write_all(toml_content.as_bytes()).unwrap();

    let loaded_config = load_config(temp_file.path().to_str().unwrap()).unwrap();

    assert_eq!(loaded_config.random_seed, Some(42));
    assert_eq!(loaded_config.simulation_period_days, 60);
    assert_eq!(loaded_config.time_step_minutes, 10);

    assert!(loaded_config.asset_models.is_some());
    let asset_models = loaded_config.asset_models.unwrap();
    assert_eq!(asset_models.len(), 2);

    // Check first asset model
    let model1 = &asset_models[0];
    assert_eq!(model1.asset_type, "stock");
    assert_eq!(model1.asset_identifier_pattern, "DEFAULT_STOCK");
    assert_eq!(model1.default_model, ModelType::GeometricBrownianMotion);
    assert!(model1.parameters.gbm.is_some());
    let gbm1_params = model1.parameters.gbm.as_ref().unwrap();
    assert_eq!(gbm1_params.drift, 0.05);
    assert_eq!(gbm1_params.volatility, 0.2);

    // Check second asset model
    let model2 = &asset_models[1];
    assert_eq!(model2.asset_type, "stock");
    assert_eq!(model2.asset_identifier_pattern, "TECH_STOCK");
    assert_eq!(model2.default_model, ModelType::GeometricBrownianMotion);
    assert!(model2.parameters.gbm.is_some());
    let gbm2_params = model2.parameters.gbm.as_ref().unwrap();
    assert_eq!(gbm2_params.drift, 0.1);
    assert_eq!(gbm2_params.volatility, 0.3);
}

#[test]
fn test_load_config_minimal() {
    // Test loading a config with no asset_models section (optional)
    let mut temp_file = NamedTempFile::new().unwrap();
    let toml_content = r#"
random_seed = 123
simulation_period_days = 30
time_step_minutes = 5
# No asset_models array
"#;
    temp_file.write_all(toml_content.as_bytes()).unwrap();
    let loaded_config = load_config(temp_file.path().to_str().unwrap()).unwrap();

    assert_eq!(loaded_config.random_seed, Some(123));
    assert_eq!(loaded_config.simulation_period_days, 30);
    assert_eq!(loaded_config.time_step_minutes, 5);
    assert!(loaded_config.asset_models.is_none());
}
