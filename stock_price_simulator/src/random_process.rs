use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct TimeSeries {
    pub timestamps: Vec<NaiveDateTime>,
    pub prices: Vec<f64>,
}

use chrono::{NaiveDate, Duration}; // NaiveDateTime removed from here
use rand::SeedableRng; // Rng removed
use rand_distr::{Normal, Distribution};
use rand::rngs::StdRng;

pub trait StochasticProcess {
    fn generate_path(&self, initial_value: f64, dt: f64, steps: usize, seed: Option<u64>) -> TimeSeries;
}

pub struct GeometricBrownianMotion {
    pub drift: f64,
    pub volatility: f64,
}

impl StochasticProcess for GeometricBrownianMotion {
    fn generate_path(&self, initial_value: f64, dt: f64, steps: usize, seed: Option<u64>) -> TimeSeries {
        let mut rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };
        let normal_dist = Normal::new(0.0, 1.0).unwrap();

        let mut prices = Vec::with_capacity(steps);
        let mut timestamps = Vec::with_capacity(steps);

        let mut current_price = initial_value;
        let mut current_time = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();

        // Assuming dt is in days. Convert dt to seconds for Duration.
        // dt is also used in the GBM formula, typically as a fraction of a year.
        // If drift and volatility are annualized, dt should be scaled.
        // Let's assume dt for GBM formula is in years: dt_annual = dt / 252.0 (approx trading days)
        // For timestamp increment, dt is in days.
        let dt_for_formula = dt / 252.0; // Assuming dt is in days, converting to year fraction
        let dt_duration = Duration::seconds((dt * 24.0 * 60.0 * 60.0) as i64); // dt in days to seconds

        for _ in 0..steps {
            prices.push(current_price);
            timestamps.push(current_time);

            let w_t = normal_dist.sample(&mut rng);
            current_price *= ((self.drift - 0.5 * self.volatility.powi(2)) * dt_for_formula + self.volatility * dt_for_formula.sqrt() * w_t).exp();
            current_time += dt_duration;
        }

        TimeSeries { timestamps, prices }
    }
}
