use crate::random_process::TimeSeries;
use anyhow::Error;
use serde::Deserialize; // Added for derive

#[derive(Debug, Clone, Deserialize)] // Added Deserialize
pub struct FuturesContract {
    pub underlying_symbol: String,
    pub initial_spot_price: f64,
    pub risk_free_rate: f64,
    pub volatility: f64, // Volatility of the underlying spot price
    pub time_to_maturity_days: u32, // Initial time to maturity in days
    pub time_step_days: f64,        // Granularity of simulation steps in days
    pub seed: Option<u64>,
}

use crate::random_process::{GeometricBrownianMotion, StochasticProcess};

const DAYS_IN_YEAR: f64 = 365.0;

pub fn simulate_futures_price(contract: &FuturesContract) -> Result<TimeSeries, Error> {
    if contract.initial_spot_price <= 0.0 {
        return Err(anyhow::anyhow!("Initial spot price must be positive."));
    }
    if contract.volatility < 0.0 {
        return Err(anyhow::anyhow!("Volatility cannot be negative."));
    }
    if contract.time_step_days <= 0.0 {
        return Err(anyhow::anyhow!("Time step in days must be positive."));
    }
    if contract.time_to_maturity_days == 0 && contract.time_step_days > 0.0 {
         // Allow simulation if maturity is 0, but it will be a single point.
         // If time_step_days is also 0, it's an error, caught by previous check.
    }


    let num_steps = (contract.time_to_maturity_days as f64 / contract.time_step_days).ceil() as usize;

    // If TTM is 0, we should have 1 step (current time).
    // If TTM > 0 but less than time_step_days, num_steps will be 1.
    // The number of *price points* will be num_steps + 1 if we consider S0.
    // However, for futures prices, each spot price point will have a corresponding futures price.
    // If TTM is 0, spot_path will have 1 price (S0), futures_prices will have 1 price (F0=S0).
    let gbm_steps = if contract.time_to_maturity_days == 0 { 1 } else { num_steps +1 };


    let gbm = GeometricBrownianMotion {
        drift: contract.risk_free_rate, // Assuming risk-neutral drift for spot
        volatility: contract.volatility,
    };

    // Generate spot price path
    // dt for generate_path is in days, which contract.time_step_days is.
    let spot_path = gbm.generate_path(
        contract.initial_spot_price,
        contract.time_step_days,
        gbm_steps, // Number of price points
        contract.seed,
    );

    let mut futures_prices = Vec::with_capacity(spot_path.prices.len());

    for i in 0..spot_path.prices.len() {
        let spot_price_t = spot_path.prices[i];

        // Calculate remaining time to maturity in days for this step
        let days_elapsed = i as f64 * contract.time_step_days;
        let remaining_days = contract.time_to_maturity_days as f64 - days_elapsed;

        // Ensure remaining_time_years is not negative.
        // Max(0.0, ...) handles cases where days_elapsed might slightly exceed TTM due to ceiling or floating point.
        let remaining_time_years = (remaining_days.max(0.0)) / DAYS_IN_YEAR;

        let futures_price_t = spot_price_t * (contract.risk_free_rate * remaining_time_years).exp();
        futures_prices.push(futures_price_t);
    }

    Ok(TimeSeries {
        timestamps: spot_path.timestamps, // Reuse timestamps from spot path
        prices: futures_prices,
    })
}
