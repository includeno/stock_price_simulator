use stock_price_simulator::futures_simulation::{FuturesContract, simulate_futures_price};
use stock_price_simulator::random_process::StochasticProcess; // Added for gbm.generate_path()

const PRICE_ACCURACY: f64 = 1e-9; // For floating point comparisons

#[test]
fn test_futures_simulation_deterministic() {
    let contract1 = FuturesContract {
        underlying_symbol: "TEST".to_string(),
        initial_spot_price: 100.0,
        risk_free_rate: 0.05,
        volatility: 0.2,
        time_to_maturity_days: 30,
        time_step_days: 1.0,
        seed: Some(12345),
    };
    let contract2 = FuturesContract { // Same params, same seed
        underlying_symbol: "TEST".to_string(),
        initial_spot_price: 100.0,
        risk_free_rate: 0.05,
        volatility: 0.2,
        time_to_maturity_days: 30,
        time_step_days: 1.0,
        seed: Some(12345),
    };

    let result1 = simulate_futures_price(&contract1).unwrap();
    let result2 = simulate_futures_price(&contract2).unwrap();

    assert_eq!(result1.prices, result2.prices, "Futures prices should be deterministic with the same seed");
    assert_eq!(result1.timestamps, result2.timestamps, "Timestamps should be deterministic with the same seed");
}

#[test]
fn test_futures_simulation_output_length() {
    let ttm_days = 30;
    let time_step = 1.0;
    let contract = FuturesContract {
        underlying_symbol: "TEST_LEN".to_string(),
        initial_spot_price: 100.0,
        risk_free_rate: 0.05,
        volatility: 0.2,
        time_to_maturity_days: ttm_days,
        time_step_days: time_step,
        seed: None,
    };

    let expected_steps = (ttm_days as f64 / time_step).ceil() as usize;
    let expected_data_points = expected_steps + 1; // Includes initial point S0/F0

    let result = simulate_futures_price(&contract).unwrap();
    assert_eq!(result.prices.len(), expected_data_points, "Number of futures prices mismatch");
    assert_eq!(result.timestamps.len(), expected_data_points, "Number of timestamps mismatch");

    // Case: TTM = 0 days
    let contract_ttm_zero = FuturesContract {
        time_to_maturity_days: 0,
        ..contract // Use other params from above
    };
    let result_ttm_zero = simulate_futures_price(&contract_ttm_zero).unwrap();
    assert_eq!(result_ttm_zero.prices.len(), 1, "Should have 1 price point for TTM=0");
    assert_eq!(result_ttm_zero.timestamps.len(), 1, "Should have 1 timestamp for TTM=0");
}

#[test]
fn test_futures_price_convergence_to_spot() {
    let contract = FuturesContract {
        underlying_symbol: "CONVERGE".to_string(),
        initial_spot_price: 120.0,
        risk_free_rate: 0.03,
        volatility: 0.15,
        time_to_maturity_days: 5, // Short maturity
        time_step_days: 1.0,      // Daily steps
        seed: Some(99),
    };

    let result = simulate_futures_price(&contract).unwrap();
    let last_futures_price = result.prices.last().unwrap();

    // To get the last spot price, we need to simulate it separately or extract from an intermediate step if possible
    // For simplicity, let's re-simulate the spot path with the same parameters
    let gbm = stock_price_simulator::random_process::GeometricBrownianMotion {
        drift: contract.risk_free_rate,
        volatility: contract.volatility,
    };
    let num_steps = (contract.time_to_maturity_days as f64 / contract.time_step_days).ceil() as usize;
    let spot_path = gbm.generate_path(
        contract.initial_spot_price,
        contract.time_step_days,
        num_steps + 1,
        contract.seed,
    );
    let last_spot_price = spot_path.prices.last().unwrap();

    // At maturity (T-t = 0), F_T = S_T.
    // The last point in our simulation corresponds to TTM being very close to 0 for that step.
    assert!((last_futures_price - last_spot_price).abs() < PRICE_ACCURACY,
            "Last futures price ({}) should be very close to last spot price ({})",
            last_futures_price, last_spot_price);

    // Test with TTM = 0 directly
    let contract_ttm_zero = FuturesContract { time_to_maturity_days: 0, ..contract };
    let result_ttm_zero = simulate_futures_price(&contract_ttm_zero).unwrap();
    assert_eq!(result_ttm_zero.prices.len(), 1);
    assert!((result_ttm_zero.prices[0] - contract_ttm_zero.initial_spot_price).abs() < PRICE_ACCURACY,
            "For TTM=0, F0 should equal S0. F0={}, S0={}", result_ttm_zero.prices[0], contract_ttm_zero.initial_spot_price);

}

#[test]
fn test_futures_price_with_zero_risk_free_rate() {
    let contract = FuturesContract {
        underlying_symbol: "ZERO_RFR".to_string(),
        initial_spot_price: 75.0,
        risk_free_rate: 0.0, // Zero risk-free rate
        volatility: 0.22,
        time_to_maturity_days: 10,
        time_step_days: 1.0,
        seed: Some(101),
    };

    let result = simulate_futures_price(&contract).unwrap();

    // Need the spot prices to compare
    let gbm = stock_price_simulator::random_process::GeometricBrownianMotion {
        drift: contract.risk_free_rate, // which is 0.0
        volatility: contract.volatility,
    };
    let num_steps = (contract.time_to_maturity_days as f64 / contract.time_step_days).ceil() as usize;
    let spot_path = gbm.generate_path(
        contract.initial_spot_price,
        contract.time_step_days,
        num_steps + 1,
        contract.seed,
    );

    for i in 0..result.prices.len() {
        assert!((result.prices[i] - spot_path.prices[i]).abs() < PRICE_ACCURACY,
                "With r=0, F_t ({}) should equal S_t ({}) at step {}",
                result.prices[i], spot_path.prices[i], i);
    }
}

#[test]
fn test_futures_price_relationship_spot_futures() {
    // F_t = S_t * exp(r * (T-t))
    // If r > 0, F_t should generally be > S_t (contango)
    // If r < 0, F_t should generally be < S_t (backwardation) - not typical for rfr
    let contract_contango = FuturesContract {
        underlying_symbol: "CONTANGO".to_string(),
        initial_spot_price: 100.0,
        risk_free_rate: 0.05, // Positive r
        volatility: 0.2,
        time_to_maturity_days: 30,
        time_step_days: 1.0,
        seed: Some(111),
    };

    let result_contango = simulate_futures_price(&contract_contango).unwrap();

    let gbm_spot = stock_price_simulator::random_process::GeometricBrownianMotion {
        drift: contract_contango.risk_free_rate,
        volatility: contract_contango.volatility,
    };
    let num_steps_contango = (contract_contango.time_to_maturity_days as f64 / contract_contango.time_step_days).ceil() as usize;
    let spot_path_contango = gbm_spot.generate_path(
        contract_contango.initial_spot_price,
        contract_contango.time_step_days,
        num_steps_contango + 1,
        contract_contango.seed,
    );

    for i in 0..result_contango.prices.len() {
        let days_elapsed = i as f64 * contract_contango.time_step_days;
        let remaining_days = contract_contango.time_to_maturity_days as f64 - days_elapsed;

        if remaining_days > 1e-9 { // Avoid issues at the very last step where T-t is effectively 0
            assert!(result_contango.prices[i] >= spot_path_contango.prices[i] - PRICE_ACCURACY,
                    "In contango (r>0), F_t ({}) should generally be >= S_t ({}) (step {})",
                    result_contango.prices[i], spot_path_contango.prices[i], i);

            let expected_f_t = spot_path_contango.prices[i] * (contract_contango.risk_free_rate * (remaining_days.max(0.0) / 365.0)).exp();
            assert!((result_contango.prices[i] - expected_f_t).abs() < PRICE_ACCURACY,
                    "Futures price at step {} ({}) does not match formula S_t * exp(r(T-t)) ({})",
                    i, result_contango.prices[i], expected_f_t);

        } else {
             assert!((result_contango.prices[i] - spot_path_contango.prices[i]).abs() < PRICE_ACCURACY,
                    "At maturity, F_T ({}) should be very close to S_T ({}) (step {})",
                    result_contango.prices[i], spot_path_contango.prices[i], i);
        }
    }
}

#[test]
fn test_invalid_inputs_for_futures() {
    assert!(simulate_futures_price(&FuturesContract {
        initial_spot_price: -100.0, risk_free_rate: 0.05, volatility: 0.2, time_to_maturity_days: 30, time_step_days: 1.0, seed: None, underlying_symbol: "T".into()
    }).is_err(), "Initial spot price must be positive");

    assert!(simulate_futures_price(&FuturesContract {
        initial_spot_price: 100.0, risk_free_rate: 0.05, volatility: -0.2, time_to_maturity_days: 30, time_step_days: 1.0, seed: None, underlying_symbol: "T".into()
    }).is_err(), "Volatility cannot be negative");

    assert!(simulate_futures_price(&FuturesContract {
        initial_spot_price: 100.0, risk_free_rate: 0.05, volatility: 0.2, time_to_maturity_days: 30, time_step_days: 0.0, seed: None, underlying_symbol: "T".into()
    }).is_err(), "Time step must be positive");

    assert!(simulate_futures_price(&FuturesContract {
        initial_spot_price: 100.0, risk_free_rate: 0.05, volatility: 0.2, time_to_maturity_days: 30, time_step_days: -1.0, seed: None, underlying_symbol: "T".into()
    }).is_err(), "Time step must be positive");

    // TTM = 0 is allowed, should produce one price point
    let res_ttm_zero = simulate_futures_price(&FuturesContract {
        initial_spot_price: 100.0, risk_free_rate: 0.05, volatility: 0.2, time_to_maturity_days: 0, time_step_days: 1.0, seed: None, underlying_symbol: "T".into()
    });
    assert!(res_ttm_zero.is_ok(), "TTM=0 should be a valid scenario, got: {:?}", res_ttm_zero.err());
    if let Ok(ts) = res_ttm_zero {
        assert_eq!(ts.prices.len(), 1);
    }
}
