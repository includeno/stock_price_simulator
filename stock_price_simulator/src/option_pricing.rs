use serde::Deserialize; // Added for derive

#[derive(Debug, Clone, Copy, Deserialize)] // Added Deserialize
pub enum OptionType {
    Call,
    Put,
}

#[derive(Debug, Clone, Deserialize)] // Added Deserialize
pub struct EuropeanOption {
    pub underlying_price: f64, // S
    pub strike_price: f64,     // K
    pub time_to_maturity_years: f64, // T
    pub risk_free_rate: f64,   // r
    pub volatility: f64,       // sigma (annualized)
    pub option_type: OptionType,
}

// Parameters of an option that are fixed, except for underlying price and time to maturity (for now T is fixed)
#[derive(Debug, Clone)]
pub struct FixedOptionParams {
    pub strike_price: f64,     // K
    pub time_to_maturity_years: f64, // T
    pub risk_free_rate: f64,   // r
    pub volatility: f64,       // sigma (annualized)
    pub option_type: OptionType,
}

pub fn price_series_for_black_scholes(
    fixed_params: &FixedOptionParams,
    underlying_prices: &[f64],
    // current_time_to_maturity_years: f64, // If we want to make T dynamic
) -> Result<Vec<f64>, Error> { // Changed return type
    underlying_prices
        .iter()
        .map(|&s_price| {
            let option_at_price = EuropeanOption {
                underlying_price: s_price,
                strike_price: fixed_params.strike_price,
                time_to_maturity_years: fixed_params.time_to_maturity_years, // Use fixed T for now
                risk_free_rate: fixed_params.risk_free_rate,
                volatility: fixed_params.volatility,
                option_type: fixed_params.option_type,
            };
            black_scholes_price(&option_at_price) // This now returns Result<f64, Error>
        })
        .collect::<Result<Vec<f64>, Error>>() // Collect into a Result of a Vec
}

use crate::random_process::{TimeSeries, GeometricBrownianMotion, StochasticProcess};
use statrs::distribution::ContinuousCDF; // Added for Normal.cdf()
use anyhow::Error;

// --- Monte Carlo Framework ---

pub trait OptionPricer {
    fn price(&self, seed: Option<u64>) -> Result<f64, Error>; // Added seed for reproducibility in MC
    // simulate_option_paths is more of an internal helper for MonteCarloOptionPricer,
    // but could be part of a more general trait if other pricers also simulate.
    // For now, let's make it specific to MonteCarlo or a helper function.
}

#[derive(Debug, Clone)]
pub struct MonteCarloOptionPricer {
    // Parameters for the option itself
    pub strike_price: f64,
    pub time_to_maturity_years: f64,
    pub risk_free_rate: f64,
    pub option_type: OptionType,
    // Parameters for the underlying asset's stochastic process (GBM)
    pub underlying_initial_price: f64,
    pub underlying_drift: f64,       // Corresponds to risk_free_rate for risk-neutral pricing
    pub underlying_volatility: f64, // Same as option's volatility for BS compatibility
    // Simulation parameters
    pub num_paths: usize,
    pub num_steps_per_path: usize,
}

impl MonteCarloOptionPricer {
    // Helper to generate underlying paths
    fn simulate_underlying_paths(&self, seed: Option<u64>) -> Result<Vec<TimeSeries>, Error> {
        let gbm = GeometricBrownianMotion {
            drift: self.underlying_drift,
            volatility: self.underlying_volatility,
        };

        // dt for gbm.generate_path is expected in days.
        // self.time_to_maturity_years is in years.
        // self.num_steps_per_path is the number of steps for the option's life.
        let total_simulation_days = self.time_to_maturity_years * 252.0; // Approx trading days in a year
        let dt_for_gbm_step_in_days = total_simulation_days / self.num_steps_per_path as f64;

        let mut all_paths = Vec::with_capacity(self.num_paths);

        // Seed handling for reproducibility:
        // If a seed is provided, we want each path to be different but the whole set deterministic.
        // So, we'll derive seeds for each path from the initial seed.
        let mut path_seeds: Vec<Option<u64>> = vec![None; self.num_paths];
        if let Some(initial_seed) = seed {
            for i in 0..self.num_paths {
                path_seeds[i] = Some(initial_seed + i as u64); // Simple seed derivation
            }
        }

        for i in 0..self.num_paths {
            let path = gbm.generate_path(
                self.underlying_initial_price,
                dt_for_gbm_step_in_days, // dt is in days
                self.num_steps_per_path + 1, // +1 to include S_T (num_steps_per_path intervals)
                path_seeds[i],
            );
            all_paths.push(path);
        }
        Ok(all_paths)
    }
}

impl OptionPricer for MonteCarloOptionPricer {
    fn price(&self, seed: Option<u64>) -> Result<f64, Error> {
        if self.time_to_maturity_years <= 0.0 || self.num_paths == 0 || self.num_steps_per_path == 0 {
             return Err(anyhow::anyhow!("Invalid parameters for Monte Carlo pricing. Ensure T > 0, num_paths > 0, num_steps > 0."));
        }

        let underlying_paths = self.simulate_underlying_paths(seed)?;
        let mut total_payoff = 0.0;

        for path in underlying_paths {
            let s_t = path.prices.last().ok_or_else(|| anyhow::anyhow!("Generated path has no prices"))?;
            let payoff = match self.option_type {
                OptionType::Call => (s_t - self.strike_price).max(0.0),
                OptionType::Put => (self.strike_price - s_t).max(0.0),
            };
            total_payoff += payoff;
        }

        let average_payoff = total_payoff / self.num_paths as f64;
        let discounted_price = average_payoff * (-self.risk_free_rate * self.time_to_maturity_years).exp();

        Ok(discounted_price)
    }
}


pub fn black_scholes_price(option: &EuropeanOption) -> Result<f64, Error> {
    let s = option.underlying_price;
    let k = option.strike_price;
    let t = option.time_to_maturity_years;
    let r = option.risk_free_rate;
    let sigma = option.volatility;

    // Input Validation
    if s <= 0.0 { return Err(anyhow::anyhow!("Underlying price (S) must be positive. Got {}", s)); }
    if k <= 0.0 { return Err(anyhow::anyhow!("Strike price (K) must be positive. Got {}", k)); }
    if t <= 0.0 { // If time to maturity is zero or negative, return intrinsic value.
        return Ok(match option.option_type {
            OptionType::Call => (s - k).max(0.0),
            OptionType::Put => (k - s).max(0.0),
        });
    }
    if r < 0.0 && (-r * t).exp().is_infinite() { // Guard against extreme negative rates if not desired
        return Err(anyhow::anyhow!("Risk-free rate (r) is too negative, leading to instability. Got {}", r));
    }
    if sigma <= 0.0 { return Err(anyhow::anyhow!("Volatility (sigma) must be positive. Got {}", sigma)); }


    let d1 = ( (s / k).ln() + (r + 0.5 * sigma.powi(2)) * t ) / (sigma * t.sqrt());
    let d2 = d1 - sigma * t.sqrt();

    let normal_dist = statrs::distribution::Normal::new(0.0, 1.0).unwrap();
    let cnd_d1 = normal_dist.cdf(d1);
    let cnd_d2 = normal_dist.cdf(d2);

    Ok(match option.option_type {
        OptionType::Call => {
            s * cnd_d1 - k * (-r * t).exp() * cnd_d2
        }
        OptionType::Put => {
            k * (-r * t).exp() * (1.0 - cnd_d2) - s * (1.0 - cnd_d1)
        }
    })
}
