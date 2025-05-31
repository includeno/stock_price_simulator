use stock_price_simulator::config::load_config;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_load_config() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let toml_content = r#"
random_seed = 42
simulation_period_days = 60
time_step_minutes = 10
"#;
    temp_file.write_all(toml_content.as_bytes()).unwrap();

    let config = load_config(temp_file.path().to_str().unwrap()).unwrap();

    assert_eq!(config.random_seed, Some(42));
    assert_eq!(config.simulation_period_days, 60);
    assert_eq!(config.time_step_minutes, 10);
}
