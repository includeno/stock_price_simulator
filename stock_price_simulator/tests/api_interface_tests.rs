use stock_price_simulator::api_interface::*;
use stock_price_simulator::{ // These are re-exported from lib.rs
    OptionType, FuturesContract, EtfDefinition, EtfConstituent,
    MonteCarloEuropeanOptionInput,
};
use stock_price_simulator::config::{
    GlobalConfig, AssetModelConfig, ModelType, ModelParameters, GeometricBrownianMotionParams
};

const TEST_DAYS: usize = 5;
const TEST_TIME_STEP: f64 = 1.0;

#[test]
fn test_simulate_stock_direct_api() { // Renamed from test_simulate_stock_api
    let result = simulate_stock(100.0, 0.05, 0.2, TEST_DAYS, TEST_TIME_STEP, Some(123));
    assert!(result.is_ok());
    let ts = result.unwrap();
    assert_eq!(ts.prices.len(), TEST_DAYS);
    assert_eq!(ts.timestamps.len(), TEST_DAYS);

    // Test error case (e.g., negative price, though simulate_stock_price handles this)
    let err_result = simulate_stock(-100.0, 0.05, 0.2, TEST_DAYS, TEST_TIME_STEP, Some(123));
    assert!(err_result.is_err());
}

// Helper to create a simple GlobalConfig for stock tests
fn create_test_global_config() -> GlobalConfig {
    GlobalConfig {
        random_seed: Some(12345),
        simulation_period_days: 252,
        time_step_minutes: 1440,
        asset_models: Some(vec![
            AssetModelConfig {
                asset_type: "stock".to_string(),
                asset_identifier_pattern: "TEST_STOCK_1".to_string(),
                default_model: ModelType::GeometricBrownianMotion,
                parameters: ModelParameters {
                    gbm: Some(GeometricBrownianMotionParams { drift: 0.05, volatility: 0.2 }),
                },
            },
            AssetModelConfig {
                asset_type: "stock".to_string(),
                asset_identifier_pattern: "TEST_STOCK_NO_GBM".to_string(),
                default_model: ModelType::GeometricBrownianMotion,
                parameters: ModelParameters { gbm: None }, // Missing GBM params
            },
        ]),
    }
}

#[test]
fn test_simulate_stock_with_config_success() {
    let config = create_test_global_config();
    let result = simulate_stock_with_config(
        "TEST_STOCK_1",
        &config,
        100.0,
        TEST_DAYS,
        TEST_TIME_STEP,
        Some(456),
        None, None, // No overrides
    );
    assert!(result.is_ok(), "Simulation failed: {:?}", result.err());
    let ts = result.unwrap();
    assert_eq!(ts.prices.len(), TEST_DAYS);
    assert_eq!(ts.timestamps.len(), TEST_DAYS);
}

#[test]
fn test_simulate_stock_with_config_overrides() {
    let config = create_test_global_config();
    let override_drift = 0.10;
    let override_volatility = 0.30;
    let result = simulate_stock_with_config(
        "TEST_STOCK_1",
        &config,
        100.0,
        TEST_DAYS,
        TEST_TIME_STEP,
        Some(789),
        Some(override_drift),
        Some(override_volatility),
    );
    assert!(result.is_ok(), "Simulation with overrides failed: {:?}", result.err());
    // Further checks could involve comparing output if we knew exact impact of overrides,
    // but for now, success is sufficient.
}

#[test]
fn test_simulate_stock_with_config_asset_not_found() {
    let config = create_test_global_config();
    let result = simulate_stock_with_config(
        "NON_EXISTENT_STOCK",
        &config,
        100.0,
        TEST_DAYS,
        TEST_TIME_STEP,
        Some(111),
        None, None,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No model config found for stock identifier: NON_EXISTENT_STOCK"));
}

#[test]
fn test_simulate_stock_with_config_gbm_params_missing() {
    let config = create_test_global_config();
    let result = simulate_stock_with_config(
        "TEST_STOCK_NO_GBM",
        &config,
        100.0,
        TEST_DAYS,
        TEST_TIME_STEP,
        Some(222),
        None, None,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("GBM parameters not configured for identifier: TEST_STOCK_NO_GBM"));
}


#[test]
fn test_price_european_option_black_scholes_api() {
    let result_call = price_european_option_black_scholes(
        100.0, 100.0, 1.0, 0.05, 0.2, OptionType::Call,
    );
    assert!(result_call.is_ok());
    assert!(result_call.unwrap() > 0.0);

    let result_put = price_european_option_black_scholes(
        100.0, 100.0, 1.0, 0.05, 0.2, OptionType::Put,
    );
    assert!(result_put.is_ok());
    assert!(result_put.unwrap() > 0.0);

    // Test error case (e.g., negative volatility)
    let err_result = price_european_option_black_scholes(
        100.0, 100.0, 1.0, 0.05, -0.2, OptionType::Call,
    );
    assert!(err_result.is_err());
     let err_result_ttm = price_european_option_black_scholes(
        100.0, 100.0, 0.0, 0.05, 0.2, OptionType::Call,
    );
    assert!(err_result_ttm.is_ok()); // TTM=0 should return intrinsic value, so Ok.
    assert_eq!(err_result_ttm.unwrap(), 0.0); // S=K, Call intrinsic = 0
}

#[test]
fn test_price_european_option_monte_carlo_api() {
    let input = MonteCarloEuropeanOptionInput {
        underlying_initial_price: 100.0,
        strike_price: 100.0,
        time_to_maturity_years: 0.1, // Shorter TTM for faster test
        risk_free_rate: 0.05,
        underlying_volatility: 0.2,
        option_type: OptionType::Call,
        num_paths: 100, // Fewer paths for faster test
        num_steps_per_path: 10, // Corrected field name
        seed: Some(42),
    };
    let result = price_european_option_monte_carlo(&input);
    assert!(result.is_ok(), "MC pricing failed: {:?}", result.err());
    assert!(result.unwrap() > 0.0);

    let invalid_input = MonteCarloEuropeanOptionInput {
        num_paths: 0, // Invalid
        ..input
    };
    let err_result = price_european_option_monte_carlo(&invalid_input);
    assert!(err_result.is_err());
}

#[test]
fn test_simulate_futures_api() {
    let contract = FuturesContract {
        underlying_symbol: "CRUDE".to_string(),
        initial_spot_price: 70.0,
        risk_free_rate: 0.02,
        volatility: 0.3,
        time_to_maturity_days: 30,
        time_step_days: TEST_TIME_STEP,
        seed: Some(789),
    };
    let result = simulate_futures(&contract);
    assert!(result.is_ok());
    let ts = result.unwrap();
    let expected_len = (contract.time_to_maturity_days as f64 / TEST_TIME_STEP).ceil() as usize + 1;
    assert_eq!(ts.prices.len(), expected_len);

    let invalid_contract = FuturesContract {
        initial_spot_price: -70.0, // Invalid
        ..contract
    };
    let err_result = simulate_futures(&invalid_contract);
    assert!(err_result.is_err());
}

#[test]
fn test_simulate_etf_api() {
    let etf_def = EtfDefinition {
        constituents: vec![
            EtfConstituent {
                symbol: "AAPL".to_string(),
                initial_price: 150.0,
                drift: 0.1,
                volatility: 0.2,
                weight: 0.5,
            },
            EtfConstituent {
                symbol: "MSFT".to_string(),
                initial_price: 280.0,
                drift: 0.08,
                volatility: 0.18,
                weight: 0.5,
            },
        ],
        simulation_days: TEST_DAYS,
        time_step_days: TEST_TIME_STEP,
        seed: Some(101),
    };
    let result = simulate_etf(&etf_def);
    assert!(result.is_ok());
    let ts = result.unwrap();
    assert_eq!(ts.prices.len(), TEST_DAYS);

    let invalid_etf_def = EtfDefinition {
        constituents: vec![], // Empty
        ..etf_def
    };
    let err_result = simulate_etf(&invalid_etf_def);
    assert!(err_result.is_err());
}
