use stock_price_simulator::option_pricing::{
    EuropeanOption, OptionType, black_scholes_price, FixedOptionParams, price_series_for_black_scholes,
    MonteCarloOptionPricer, OptionPricer
};
// use stock_price_simulator::random_process::TimeSeries; // Not directly used in assertions yet

const TEST_ACCURACY: f64 = 0.01; // For BS price comparisons
const MC_ACCURACY_VS_BS: f64 = 0.5; // Monte Carlo is an approximation

#[test]
fn test_black_scholes_call_price() {
    // Values from an online Black-Scholes calculator
    // S=100, K=100, T=1yr, r=0.05, sigma=0.2
    let option = EuropeanOption {
        underlying_price: 100.0,
        strike_price: 100.0,
        time_to_maturity_years: 1.0,
        risk_free_rate: 0.05,
        volatility: 0.2,
        option_type: OptionType::Call,
    };
    let expected_price = 10.450583572185565;
    let calculated_price_result = black_scholes_price(&option);
    assert!(calculated_price_result.is_ok());
    let calculated_price = calculated_price_result.unwrap();
    assert!((calculated_price - expected_price).abs() < TEST_ACCURACY, "Call price mismatch. Expected: {:.7}, Got: {:.7}", expected_price, calculated_price);

    // S=100, K=95, T=0.5yr, r=0.03, sigma=0.25
     let option2 = EuropeanOption {
        underlying_price: 100.0,
        strike_price: 95.0,
        time_to_maturity_years: 0.5,
        risk_free_rate: 0.03,
        volatility: 0.25,
        option_type: OptionType::Call,
    };
    let expected_price2 = 10.49687533872639; // Updated from 9.52
    let calculated_price2_result = black_scholes_price(&option2);
    assert!(calculated_price2_result.is_ok());
    let calculated_price2 = calculated_price2_result.unwrap();
    assert!((calculated_price2 - expected_price2).abs() < TEST_ACCURACY, "Call price 2 mismatch. Expected: {:.7}, Got: {:.7}", expected_price2, calculated_price2);
}

#[test]
fn test_black_scholes_put_price() {
    // S=100, K=100, T=1yr, r=0.05, sigma=0.2
    let option = EuropeanOption {
        underlying_price: 100.0,
        strike_price: 100.0,
        time_to_maturity_years: 1.0,
        risk_free_rate: 0.05,
        volatility: 0.2,
        option_type: OptionType::Put,
    };
    let expected_price = 5.573526022256971;
    let calculated_price_result = black_scholes_price(&option);
    assert!(calculated_price_result.is_ok());
    let calculated_price = calculated_price_result.unwrap();
     assert!((calculated_price - expected_price).abs() < TEST_ACCURACY, "Put price mismatch. Expected: {:.7}, Got: {:.7}", expected_price, calculated_price);

    // S=60, K=65, T=0.25yr, r=0.04, sigma=0.3
    let option2 = EuropeanOption {
        underlying_price: 60.0,
        strike_price: 65.0,
        time_to_maturity_years: 0.25, // 3 months
        risk_free_rate: 0.04,
        volatility: 0.30,
        option_type: OptionType::Put,
    };
    let expected_price2 = 6.290973155899039; // Updated from 6.24
    let calculated_price2_result = black_scholes_price(&option2);
    assert!(calculated_price2_result.is_ok());
    let calculated_price2 = calculated_price2_result.unwrap();
    assert!((calculated_price2 - expected_price2).abs() < TEST_ACCURACY, "Put price 2 mismatch. Expected: {:.7}, Got: {:.7}", expected_price2, calculated_price2);
}

#[test]
fn test_price_series_for_black_scholes_func() {
    let fixed_params = FixedOptionParams {
        strike_price: 100.0,
        time_to_maturity_years: 0.5,
        risk_free_rate: 0.05,
        volatility: 0.2,
        option_type: OptionType::Call,
    };
    let underlying_prices = [90.0, 100.0, 110.0];

    // Expected prices calculated individually using BS logic for each underlying price
    // S=90, K=100, T=0.5, r=0.05, sigma=0.2 => Call ~2.3494282954139862
    // S=100, K=100, T=0.5, r=0.05, sigma=0.2 => Call ~6.888728613688083
    // S=110, K=100, T=0.5, r=0.05, sigma=0.2 => Call ~14.07986274534926
    let expected_option_prices = [2.3494282954139862, 6.888728613688083, 14.07986274534926]; // Updated values
    let calculated_option_prices_result = price_series_for_black_scholes(&fixed_params, &underlying_prices);
    assert!(calculated_option_prices_result.is_ok());
    let calculated_option_prices = calculated_option_prices_result.unwrap();

    assert_eq!(calculated_option_prices.len(), expected_option_prices.len());
    for i in 0..calculated_option_prices.len() {
        assert!((calculated_option_prices[i] - expected_option_prices[i]).abs() < TEST_ACCURACY,
                "Price series mismatch at index {}. Expected: {:.7}, Got: {:.7}", i, expected_option_prices[i], calculated_option_prices[i]);
    }
}

// Removed test_mc_simulate_underlying_paths as it was empty and caused unused variable warning.
// Its intent is covered by test_monte_carlo_vs_black_scholes_call/put.

#[test]
fn test_monte_carlo_vs_black_scholes_call() {
    let s = 100.0;
    let k = 100.0;
    let t = 1.0;
    let r = 0.05;
    let sigma = 0.2;
    let seed = Some(42u64); // For reproducibility

    let bs_option = EuropeanOption {
        underlying_price: s,
        strike_price: k,
        time_to_maturity_years: t,
        risk_free_rate: r,
        volatility: sigma,
        option_type: OptionType::Call,
    };
    let bs_price_result = black_scholes_price(&bs_option);
    assert!(bs_price_result.is_ok());
    let bs_price = bs_price_result.unwrap();

    let mc_pricer = MonteCarloOptionPricer {
        strike_price: k,
        time_to_maturity_years: t,
        risk_free_rate: r, // Used for discounting
        option_type: OptionType::Call,
        underlying_initial_price: s,
        underlying_drift: r, // For risk-neutral simulation
        underlying_volatility: sigma,
        num_paths: 20000, // Increased for better accuracy
        num_steps_per_path: 100, // More steps for better path accuracy
    };

    let mc_price = mc_pricer.price(seed).unwrap();

    println!("BS Call Price: {:.7}, MC Call Price: {:.7}", bs_price, mc_price);
    assert!((mc_price - bs_price).abs() < MC_ACCURACY_VS_BS,
            "Monte Carlo call price ({:.7}) is too far from Black-Scholes price ({:.7}). Difference: {:.7}",
            mc_price, bs_price, (mc_price - bs_price).abs());
}

#[test]
fn test_monte_carlo_vs_black_scholes_put() {
    let s = 100.0;
    let k = 100.0;
    let t = 1.0;
    let r = 0.05;
    let sigma = 0.2;
    let seed = Some(1234u64);

    let bs_option = EuropeanOption {
        underlying_price: s,
        strike_price: k,
        time_to_maturity_years: t,
        risk_free_rate: r,
        volatility: sigma,
        option_type: OptionType::Put,
    };
    let bs_price_result = black_scholes_price(&bs_option);
    assert!(bs_price_result.is_ok());
    let bs_price = bs_price_result.unwrap();

    let mc_pricer = MonteCarloOptionPricer {
        strike_price: k,
        time_to_maturity_years: t,
        risk_free_rate: r,
        option_type: OptionType::Put,
        underlying_initial_price: s,
        underlying_drift: r,
        underlying_volatility: sigma,
        num_paths: 20000,
        num_steps_per_path: 100,
    };

    let mc_price = mc_pricer.price(seed).unwrap();

    println!("BS Put Price: {:.7}, MC Put Price: {:.7}", bs_price, mc_price);
    assert!((mc_price - bs_price).abs() < MC_ACCURACY_VS_BS,
            "Monte Carlo put price ({:.7}) is too far from Black-Scholes price ({:.7}). Difference: {:.7}",
            mc_price, bs_price, (mc_price - bs_price).abs());
}

#[test]
fn test_mc_pricer_invalid_inputs() {
     let mc_pricer_invalid_t = MonteCarloOptionPricer {
        strike_price: 100.0, time_to_maturity_years: 0.0, risk_free_rate: 0.05, option_type: OptionType::Call,
        underlying_initial_price: 100.0, underlying_drift: 0.05, underlying_volatility: 0.2,
        num_paths: 100, num_steps_per_path: 10,
    };
    assert!(mc_pricer_invalid_t.price(None).is_err());

    let mc_pricer_invalid_paths = MonteCarloOptionPricer {
        strike_price: 100.0, time_to_maturity_years: 1.0, risk_free_rate: 0.05, option_type: OptionType::Call,
        underlying_initial_price: 100.0, underlying_drift: 0.05, underlying_volatility: 0.2,
        num_paths: 0, num_steps_per_path: 10,
    };
    assert!(mc_pricer_invalid_paths.price(None).is_err());

     let mc_pricer_invalid_steps = MonteCarloOptionPricer {
        strike_price: 100.0, time_to_maturity_years: 1.0, risk_free_rate: 0.05, option_type: OptionType::Call,
        underlying_initial_price: 100.0, underlying_drift: 0.05, underlying_volatility: 0.2,
        num_paths: 100, num_steps_per_path: 0,
    };
    assert!(mc_pricer_invalid_steps.price(None).is_err());
}
