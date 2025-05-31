use crate::random_process::TimeSeries;
use crate::stock_simulation::StockSimulator;
use crate::option_pricing::{EuropeanOption, OptionType, OptionPricer};
use crate::futures_simulation::FuturesContract;
use crate::etf_simulation::EtfDefinition;
use anyhow::Result;
use serde::Deserialize; // Added for MonteCarloEuropeanOptionInput

// --- Stock Simulation ---
pub fn simulate_stock(
    initial_price: f64,
    drift: f64,
    volatility: f64,
    days: usize, // Number of steps
    time_step_days: f64,
    seed: Option<u64>,
) -> Result<TimeSeries> {
    StockSimulator::simulate_stock_price(
        initial_price,
        drift,
        volatility,
        days,
        time_step_days,
        seed,
    )
}

// --- Option Pricing ---

// Black-Scholes
pub fn price_european_option_black_scholes(
    underlying_price: f64,
    strike_price: f64,
    time_to_maturity_years: f64,
    risk_free_rate: f64,
    volatility: f64,
    option_type: OptionType,
) -> Result<f64> {
    let option = EuropeanOption {
        underlying_price,
        strike_price,
        time_to_maturity_years,
        risk_free_rate,
        volatility,
        option_type,
    };
    // The black_scholes_price function needs to be modified to return Result
    // and perform input validation. This will be handled in a subsequent step.
    crate::option_pricing::black_scholes_price(&option)
}

// Monte Carlo
#[derive(Debug, Clone, Deserialize)] // Added Deserialize
pub struct MonteCarloEuropeanOptionInput {
    pub underlying_initial_price: f64,
    pub strike_price: f64,
    pub time_to_maturity_years: f64,
    pub risk_free_rate: f64,
    pub underlying_volatility: f64,
    pub option_type: OptionType,
    pub num_paths: usize,
    pub num_steps_per_path: usize, // Corrected field name
    pub seed: Option<u64>,
}

pub fn price_european_option_monte_carlo(
    input: &MonteCarloEuropeanOptionInput,
) -> Result<f64> {
    let pricer = crate::option_pricing::MonteCarloOptionPricer {
        underlying_initial_price: input.underlying_initial_price,
        strike_price: input.strike_price,
        time_to_maturity_years: input.time_to_maturity_years,
        risk_free_rate: input.risk_free_rate,
        underlying_drift: input.risk_free_rate, // Assuming risk-neutral drift for MC
        underlying_volatility: input.underlying_volatility,
        option_type: input.option_type, // OptionType is Copy
        num_paths: input.num_paths,
        num_steps_per_path: input.num_steps_per_path, // Corrected field name
    };
    pricer.price(input.seed)
}

// --- Futures Simulation ---
pub fn simulate_futures(contract_params: &FuturesContract) -> Result<TimeSeries> {
    crate::futures_simulation::simulate_futures_price(contract_params)
}

// --- ETF Simulation ---
pub fn simulate_etf(etf_params: &EtfDefinition) -> Result<TimeSeries> {
    crate::etf_simulation::simulate_etf_nav(etf_params)
}
