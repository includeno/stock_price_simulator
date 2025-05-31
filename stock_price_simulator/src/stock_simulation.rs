use crate::random_process::{GeometricBrownianMotion, StochasticProcess, TimeSeries};
use anyhow::Error;

pub struct StockSimulator;

impl StockSimulator {
    pub fn simulate_stock_price(
        initial_price: f64,
        drift: f64,
        volatility: f64,
        days: usize, // Interpreted as number of steps
        time_step_days: f64,
        seed: Option<u64>,
    ) -> Result<TimeSeries, Error> {
        if initial_price <= 0.0 {
            return Err(anyhow::anyhow!("Initial price must be positive."));
        }
        if volatility < 0.0 {
            return Err(anyhow::anyhow!("Volatility cannot be negative."));
        }
        if time_step_days <= 0.0 {
            return Err(anyhow::anyhow!("Time step must be positive."));
        }
        if days == 0 {
            return Err(anyhow::anyhow!("Number of days (steps) must be positive."));
        }

        let gbm = GeometricBrownianMotion { drift, volatility };

        // 'days' is used as the number of steps directly.
        // 'time_step_days' is used as 'dt' for generate_path.
        let path = gbm.generate_path(initial_price, time_step_days, days, seed);

        Ok(path)
    }
}
