use crate::random_process::TimeSeries;
use anyhow::Error;
use serde::Deserialize; // Added for derive

#[derive(Debug, Clone, Deserialize)] // Added Deserialize
pub struct EtfConstituent {
    pub symbol: String,
    pub initial_price: f64,
    pub drift: f64,
    pub volatility: f64,
    pub weight: f64, // Proportion in the ETF
}

#[derive(Debug, Clone, Deserialize)] // Added Deserialize
pub struct EtfDefinition {
    pub constituents: Vec<EtfConstituent>,
    pub simulation_days: usize, // Number of simulation steps/days
    pub time_step_days: f64,    // Granularity of each step
    pub seed: Option<u64>,
}

use crate::stock_simulation::StockSimulator;

const WEIGHT_SUM_ACCURACY: f64 = 1e-6;

pub fn simulate_etf_nav(etf_def: &EtfDefinition) -> Result<TimeSeries, Error> {
    if etf_def.constituents.is_empty() {
        return Err(anyhow::anyhow!("ETF constituents list cannot be empty."));
    }

    let total_weight: f64 = etf_def.constituents.iter().map(|c| c.weight).sum();
    if (total_weight - 1.0).abs() > WEIGHT_SUM_ACCURACY {
        return Err(anyhow::anyhow!(
            "Sum of constituent weights ({}) must be close to 1.0.",
            total_weight
        ));
    }
    if etf_def.simulation_days == 0 {
        return Err(anyhow::anyhow!("Simulation days must be greater than 0."));
    }
     if etf_def.time_step_days <= 0.0 {
        return Err(anyhow::anyhow!("Time step in days must be positive."));
    }


    let mut constituent_price_paths: Vec<Vec<f64>> = Vec::with_capacity(etf_def.constituents.len());
    let mut timestamps: Option<Vec<chrono::NaiveDateTime>> = None;

    for (i, constituent) in etf_def.constituents.iter().enumerate() {
        if constituent.initial_price <= 0.0 {
            return Err(anyhow::anyhow!("Constituent '{}' initial price must be positive.", constituent.symbol));
        }
        if constituent.volatility < 0.0 {
             return Err(anyhow::anyhow!("Constituent '{}' volatility cannot be negative.", constituent.symbol));
        }
        if constituent.weight < 0.0 { // Weight can be 0, but not negative
             return Err(anyhow::anyhow!("Constituent '{}' weight cannot be negative.", constituent.symbol));
        }


        let constituent_seed = etf_def.seed.map(|s| s + i as u64);
        let stock_path_result = StockSimulator::simulate_stock_price(
            constituent.initial_price,
            constituent.drift,
            constituent.volatility,
            etf_def.simulation_days, // This is 'steps' for simulate_stock_price
            etf_def.time_step_days,
            constituent_seed,
        );

        match stock_path_result {
            Ok(stock_path) => {
                if i == 0 {
                    timestamps = Some(stock_path.timestamps);
                }
                constituent_price_paths.push(stock_path.prices);
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to simulate stock price for constituent {}: {}",
                    constituent.symbol,
                    e
                ));
            }
        }
    }

    let final_timestamps = timestamps.ok_or_else(|| anyhow::anyhow!("Timestamps could not be generated."))?;
    // Number of price points for each stock is simulation_days + 1 (due to initial price)
    // but simulate_stock_price uses 'days' as number of steps, so it returns 'days' price points.
    // If simulate_stock_price's 'days' means number of *steps*, then it produces 'days' points.
    // Let's assume etf_def.simulation_days is the number of *points* we want in the final NAV path including t=0.
    // StockSimulator::simulate_stock_price(..., days, ...) -> days = number of steps.
    // So it produces `days` data points. If simulation_days is 1, it means 1 step, so 1 price point (the one after S0).
    // This means the NAV path will have `etf_def.simulation_days` points.
    // The loop for NAV calculation should go up to `etf_def.simulation_days`.
    // Each `constituent_price_paths[j]` will have `etf_def.simulation_days` elements.

    let num_nav_points = etf_def.simulation_days;
    // If constituent_price_paths[0] has N elements, NAV path also has N elements.
    // The number of points in constituent_price_paths[j] is etf_def.simulation_days as per StockSimulator.
    // So, the NAV path will also have etf_def.simulation_days points.

    let mut etf_nav_path = Vec::with_capacity(num_nav_points);

    for t_idx in 0..num_nav_points {
        let mut nav_at_t = 0.0;
        for (j, constituent) in etf_def.constituents.iter().enumerate() {
            let price_j_t = constituent_price_paths[j][t_idx];
            // Initial number of shares of constituent j for a $1 initial ETF investment
            let num_shares_j = constituent.weight / constituent.initial_price;
            nav_at_t += num_shares_j * price_j_t;
        }
        etf_nav_path.push(nav_at_t);
    }

    // If the NAV path has fewer points than timestamps due to how steps vs points are handled:
    // The timestamps from simulate_stock_price will match the number of price points it generates.
    // If simulation_days is 1 for stock_simulator, it produces 1 price point.
    // So final_timestamps should have the same length as etf_nav_path.

    Ok(TimeSeries {
        timestamps: final_timestamps,
        prices: etf_nav_path,
    })
}
