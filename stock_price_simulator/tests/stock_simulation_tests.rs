use stock_price_simulator::stock_simulation::StockSimulator;

#[test]
fn test_simulate_stock_price_deterministic() {
    let initial_price = 100.0;
    let drift = 0.05;
    let volatility = 0.2;
    let days = 10; // 10 steps
    let time_step_days = 1.0; // Each step is 1 day
    let seed = Some(123u64);

    let result1 = StockSimulator::simulate_stock_price(
        initial_price, drift, volatility, days, time_step_days, seed
    ).unwrap();
    let result2 = StockSimulator::simulate_stock_price(
        initial_price, drift, volatility, days, time_step_days, seed
    ).unwrap();

    assert_eq!(result1.prices, result2.prices, "Prices should be deterministic with the same seed");
    assert_eq!(result1.timestamps, result2.timestamps, "Timestamps should be deterministic with the same seed");
}

#[test]
fn test_simulate_stock_price_output_length() {
    let initial_price = 100.0;
    let drift = 0.05;
    let volatility = 0.2;
    let days_steps = 50; // 50 steps
    let time_step_days = 0.5; // Each step is half a day

    let result = StockSimulator::simulate_stock_price(
        initial_price, drift, volatility, days_steps, time_step_days, None
    ).unwrap();

    assert_eq!(result.prices.len(), days_steps, "Number of prices should match days_steps");
    assert_eq!(result.timestamps.len(), days_steps, "Number of timestamps should match days_steps");
}

#[test]
fn test_simulate_stock_price_initial_value() {
    let initial_price = 150.0;
    let drift = 0.03;
    let volatility = 0.1;
    let days = 5;
    let time_step_days = 1.0;

    let result = StockSimulator::simulate_stock_price(
        initial_price, drift, volatility, days, time_step_days, None
    ).unwrap();

    assert_eq!(result.prices[0], initial_price, "First price should be the initial price");
}

#[test]
fn test_simulate_stock_price_positive_prices() {
    let initial_price = 20.0;
    let drift = 0.1;
    let volatility = 0.3;
    let days = 100;
    let time_step_days = 1.0/252.0; // Small time step, e.g., daily for a year

    let result = StockSimulator::simulate_stock_price(
        initial_price, drift, volatility, days, time_step_days, None
    ).unwrap();

    for price in result.prices {
        assert!(price > 0.0, "All prices should be positive for a positive initial price");
    }
}

#[test]
fn test_simulate_stock_price_invalid_inputs() {
    assert!(StockSimulator::simulate_stock_price(-100.0, 0.05, 0.2, 10, 1.0, None).is_err(), "Initial price must be positive");
    assert!(StockSimulator::simulate_stock_price(100.0, 0.05, -0.2, 10, 1.0, None).is_err(), "Volatility cannot be negative");
    assert!(StockSimulator::simulate_stock_price(100.0, 0.05, 0.2, 10, -1.0, None).is_err(), "Time step must be positive");
    assert!(StockSimulator::simulate_stock_price(100.0, 0.05, 0.2, 0, 1.0, None).is_err(), "Number of days (steps) must be positive");
}
