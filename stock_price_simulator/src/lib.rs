pub mod config;
pub mod random_process;
pub mod stock_simulation;
pub mod option_pricing;
pub mod futures_simulation;
pub mod etf_simulation;
pub mod api_models;
pub mod api_interface;
pub mod http_server; // Added http_server module

// Re-exports for convenience
pub use random_process::TimeSeries;
pub use option_pricing::{EuropeanOption, OptionType};
pub use futures_simulation::FuturesContract;
pub use etf_simulation::{EtfDefinition, EtfConstituent};
pub use api_interface::MonteCarloEuropeanOptionInput;
