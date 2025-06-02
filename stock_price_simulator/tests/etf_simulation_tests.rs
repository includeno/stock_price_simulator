use stock_price_simulator::etf_simulation::{EtfConstituent, EtfDefinition, simulate_etf_nav};

const NAV_PRICE_ACCURACY: f64 = 1e-9;

#[test]
fn test_simulate_etf_nav_single_constituent() {
    let constituent1 = EtfConstituent {
        symbol: "STOCK_A".to_string(),
        initial_price: 100.0,
        drift: 0.1,
        volatility: 0.2,
        weight: 1.0,
    };
    let etf_def = EtfDefinition {
        constituents: vec![constituent1.clone()],
        simulation_days: 10,
        time_step_days: 1.0,
        seed: Some(123),
    };

    let etf_nav_result = simulate_etf_nav(&etf_def).unwrap();

    // For a single constituent with weight 1.0, the NAV path should mirror the stock's price path.
    // The NAV calculation is (weight / initial_price) * price_t.
    // If weight = 1.0, NAV_t = (1.0 / S0) * S_t.
    // If initial NAV is normalized to 1.0, then NAV_t = S_t / S0.
    // The simulate_etf_nav function calculates NAV assuming an initial $1 investment.
    // So, NAV0 = sum (weight_i / initial_price_i) * initial_price_i = sum (weight_i) = 1.0.
    // NAV_t = sum (weight_i / initial_price_i) * price_i_t.
    // For single stock, NAV_t = (1.0 / S0) * S_t.

    // Let's get the stock's own path for comparison
    let stock_a_path = stock_price_simulator::stock_simulation::StockSimulator::simulate_stock_price(
        constituent1.initial_price,
        constituent1.drift,
        constituent1.volatility,
        etf_def.simulation_days,
        etf_def.time_step_days,
        etf_def.seed.map(|s| s + 0), // Same seed derivation as in simulate_etf_nav
    ).unwrap();

    assert_eq!(etf_nav_result.prices.len(), stock_a_path.prices.len());
    for i in 0..etf_nav_result.prices.len() {
        let expected_nav_at_t = (1.0 / constituent1.initial_price) * stock_a_path.prices[i];
        assert!((etf_nav_result.prices[i] - expected_nav_at_t).abs() < NAV_PRICE_ACCURACY,
                "NAV at t={} mismatch. Expected: {}, Got: {}", i, expected_nav_at_t, etf_nav_result.prices[i]);
    }
    assert_eq!(etf_nav_result.timestamps, stock_a_path.timestamps);
}

#[test]
fn test_simulate_etf_nav_deterministic() {
    let constituents = vec![
        EtfConstituent { symbol: "A".to_string(), initial_price: 100.0, drift: 0.1, volatility: 0.2, weight: 0.5 },
        EtfConstituent { symbol: "B".to_string(), initial_price: 50.0, drift: 0.05, volatility: 0.15, weight: 0.5 },
    ];
    let etf_def1 = EtfDefinition {
        constituents: constituents.clone(),
        simulation_days: 5,
        time_step_days: 1.0,
        seed: Some(42),
    };
    let etf_def2 = EtfDefinition { // Same params and seed
        constituents: constituents.clone(),
        simulation_days: 5,
        time_step_days: 1.0,
        seed: Some(42),
    };

    let result1 = simulate_etf_nav(&etf_def1).unwrap();
    let result2 = simulate_etf_nav(&etf_def2).unwrap();

    assert_eq!(result1.prices, result2.prices, "ETF NAV prices should be deterministic with the same seed");
    assert_eq!(result1.timestamps, result2.timestamps, "ETF NAV timestamps should be deterministic");
}

#[test]
fn test_simulate_etf_nav_output_length() {
    let etf_def = EtfDefinition {
        constituents: vec![
            EtfConstituent { symbol: "C".to_string(), initial_price: 100.0, drift: 0.1, volatility: 0.2, weight: 1.0 }
        ],
        simulation_days: 20,
        time_step_days: 0.5,
        seed: None,
    };
    // simulate_stock_price with 'days' = 20 produces 20 data points.
    let expected_data_points = etf_def.simulation_days;
    let result = simulate_etf_nav(&etf_def).unwrap();
    assert_eq!(result.prices.len(), expected_data_points);
    assert_eq!(result.timestamps.len(), expected_data_points);
}

#[test]
fn test_simulate_etf_nav_basic_sanity_check() {
    // If all stocks go up, NAV should go up (assuming positive weights)
    // This is hard to guarantee with GBM, but let's test initial NAV calculation.
    let constituents = vec![
        EtfConstituent { symbol: "UP1".to_string(), initial_price: 10.0, drift: 0.1, volatility: 0.001, weight: 0.7 },
        EtfConstituent { symbol: "UP2".to_string(), initial_price: 20.0, drift: 0.1, volatility: 0.001, weight: 0.3 },
    ];
     // Low volatility, positive drift means prices are very likely to go up for a few steps.
    let etf_def = EtfDefinition {
        constituents,
        simulation_days: 3, // Few steps
        time_step_days: 1.0,
        seed: Some(777),
    };
    let result = simulate_etf_nav(&etf_def).unwrap();

    // Initial NAV should be 1.0 by construction
    assert!((result.prices[0] - 1.0).abs() < NAV_PRICE_ACCURACY, "Initial NAV should be 1.0. Got: {}", result.prices[0]);

    // Check if subsequent NAVs are generally increasing for this setup
    // This is a probabilistic check, might not always hold, but with low vol and high drift it should for a few steps.
    // For a more robust check, we'd need to mock the stock simulator.
    // For now, let's just ensure it runs and the values are positive.
    for price_val in &result.prices { // Iterate over a slice to avoid moving
        assert!(*price_val > 0.0, "ETF NAV price should be positive.");
    }
    if result.prices.len() > 1 {
         assert!(result.prices[1] > result.prices[0] - 0.1, "NAV for UP1/UP2 expected to increase or stay similar (step 1 vs 0), P1: {}, P0: {}", result.prices[1], result.prices[0]); // allow small decrease due to tiny vol
         if result.prices.len() > 2 {
            assert!(result.prices[2] > result.prices[1] - 0.1, "NAV for UP1/UP2 expected to increase or stay similar (step 2 vs 1), P2: {}, P1: {}", result.prices[2], result.prices[1]);
         }
    }
}

#[test]
fn test_etf_invalid_inputs() {
    assert!(simulate_etf_nav(&EtfDefinition {
        constituents: vec![], // Empty constituents
        simulation_days: 10, time_step_days: 1.0, seed: None
    }).is_err(), "Empty constituents list should be an error.");

    assert!(simulate_etf_nav(&EtfDefinition {
        constituents: vec![ EtfConstituent { symbol: "A".into(), initial_price: 100.0, drift: 0.1, volatility: 0.2, weight: 0.5 } ],
        simulation_days: 10, time_step_days: 1.0, seed: None
    }).is_err(), "Sum of weights not close to 1.0 should be an error.");

    assert!(simulate_etf_nav(&EtfDefinition {
        constituents: vec![ EtfConstituent { symbol: "A".into(), initial_price: 100.0, drift: 0.1, volatility: 0.2, weight: 1.0 } ],
        simulation_days: 0, time_step_days: 1.0, seed: None // simulation_days = 0
    }).is_err(), "Simulation days = 0 should be an error.");

    assert!(simulate_etf_nav(&EtfDefinition {
        constituents: vec![ EtfConstituent { symbol: "A".into(), initial_price: 100.0, drift: 0.1, volatility: 0.2, weight: 1.0 } ],
        simulation_days: 10, time_step_days: 0.0, seed: None // time_step_days = 0
    }).is_err(), "Time step days = 0 should be an error.");

    assert!(simulate_etf_nav(&EtfDefinition {
        constituents: vec![ EtfConstituent { symbol: "A".into(), initial_price: -10.0, drift: 0.1, volatility: 0.2, weight: 1.0 } ],
        simulation_days: 10, time_step_days: 1.0, seed: None
    }).is_err(), "Negative initial price for constituent should be an error.");

    assert!(simulate_etf_nav(&EtfDefinition {
        constituents: vec![ EtfConstituent { symbol: "A".into(), initial_price: 10.0, drift: 0.1, volatility: -0.2, weight: 1.0 } ],
        simulation_days: 10, time_step_days: 1.0, seed: None
    }).is_err(), "Negative volatility for constituent should be an error.");

    assert!(simulate_etf_nav(&EtfDefinition {
        constituents: vec![ EtfConstituent { symbol: "A".into(), initial_price: 10.0, drift: 0.1, volatility: 0.2, weight: -0.1 } ],
        simulation_days: 10, time_step_days: 1.0, seed: None
    }).is_err(), "Negative weight for constituent should be an error.");
}
