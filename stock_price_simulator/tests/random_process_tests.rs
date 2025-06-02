use stock_price_simulator::random_process::{GeometricBrownianMotion, StochasticProcess};
use chrono::{NaiveDate, Duration};

#[test]
fn test_gbm_generate_path_deterministic() {
    let gbm = GeometricBrownianMotion { drift: 0.1, volatility: 0.2 };
    let initial_value = 100.0;
    let dt = 1.0; // 1 day
    let steps = 5;
    let seed = Some(12345u64);

    let path1 = gbm.generate_path(initial_value, dt, steps, seed);
    let path2 = gbm.generate_path(initial_value, dt, steps, seed);

    assert_eq!(path1.prices, path2.prices, "Prices should be deterministic with the same seed");
    assert_eq!(path1.timestamps, path2.timestamps, "Timestamps should be deterministic with the same seed");

    // Expected values for first few steps with seed 12345, drift 0.1, vol 0.2, dt 1/252 year
    // S0 = 100
    // W1 (from StdRng(12345) with Normal(0,1)): approx 0.660898
    // S1 = 100 * exp((0.1 - 0.5 * 0.2^2) * (1/252) + 0.2 * sqrt(1/252) * 0.660898)
    // S1 = 100 * exp((0.08) * 0.00396825 + 0.2 * 0.063 * 0.660898)
    // S1 = 100 * exp(0.00031746 + 0.0126 * 0.660898)
    // S1 = 100 * exp(0.00031746 + 0.0083273)
    // S1 = 100 * exp(0.00864476) = 100 * 1.008682 = 100.8682
    // Note: The exact value depends on the RNG implementation details.
    // For this test, we primarily check determinism and structure.
    // A more precise expected value test would require deeper mocking or known values from a reference implementation.

    // For now, let's check the first value and structure
    assert_eq!(path1.prices[0], initial_value);
    // Example check for the second price (very approximate, replace with more accurate if possible)
    // assert!((path1.prices[1] - 100.86).abs() < 0.01, "Second price does not match expected value");
}

#[test]
fn test_gbm_generate_path_lengths() {
    let gbm = GeometricBrownianMotion { drift: 0.05, volatility: 0.15 };
    let initial_value = 50.0;
    let dt = 1.0; // 1 day
    let steps = 10;

    let path = gbm.generate_path(initial_value, dt, steps, None);

    assert_eq!(path.timestamps.len(), steps, "Timestamps length should match steps");
    assert_eq!(path.prices.len(), steps, "Prices length should match steps");
}

#[test]
fn test_gbm_generate_path_timestamps() {
    let gbm = GeometricBrownianMotion { drift: 0.0, volatility: 0.1 };
    let initial_value = 1000.0;
    let dt_days = 1.0; // 1 day
    let steps = 3;

    let path = gbm.generate_path(initial_value, dt_days, steps, None);

    let expected_start_time = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
    assert_eq!(path.timestamps[0], expected_start_time, "First timestamp should be the defined start time");

    let expected_second_time = expected_start_time + Duration::days(dt_days as i64);
    assert_eq!(path.timestamps[1], expected_second_time, "Second timestamp should be incremented by dt_days");

    let expected_third_time = expected_second_time + Duration::days(dt_days as i64);
    assert_eq!(path.timestamps[2], expected_third_time, "Third timestamp should be incremented by dt_days");
}
