use stock_price_simulator::http_server::run_server;
use stock_price_simulator::config::GlobalConfig; // For type annotation
use actix_web::web; // For web::Data

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let address = "127.0.0.1:8080";
    let base_url = format!("http://{}", address);

    // Load configuration
    let config_path = "config.toml";
    if std::fs::metadata(config_path).is_err() {
        match std::fs::copy("config.example.toml", config_path) {
            Ok(_) => println!("Copied config.example.toml to {}", config_path),
            Err(e) => {
                eprintln!("Error copying config.example.toml to {}: {}. Please ensure config.example.toml exists or create a valid {}.", e, config_path, config_path);
                // Depending on desired behavior, might exit or use hardcoded defaults.
                // For this example, we'll proceed, and load_config will likely fail if file is still missing.
            }
        }
    }

    let config: GlobalConfig = match stock_price_simulator::config::load_config(config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load configuration from {}: {}", config_path, e);
            eprintln!("Please ensure '{}' is correctly formatted or remove it to use defaults from a copied example.", config_path);
            // Fallback or panic. For this example, we'll panic as config is essential.
            // In a real app, you might use default values or provide a clearer recovery path.
            panic!("Configuration error: {}", e);
        }
    };
    let app_config_data = web::Data::new(config.clone()); // Clone for Actix app data

    println!("Starting server on {} ...", base_url);
    println!("-----------------------------------------------------------------------");
    println!("API Endpoints - Try these in your browser or API client (e.g., Postman, curl):");
    println!("-----------------------------------------------------------------------");

    // Stock Simulation (GET)
    println!("\n[GET] Stock Simulation (using config):");
    println!("  Calculates a simulated stock price path. Uses parameters from config if not overridden.");
    println!("  Example (using 'DEFAULT_STOCK' from config): {}/simulate/stock?asset_identifier=DEFAULT_STOCK&initial_price=150&days=20&time_step_days=1&seed=123", base_url);
    println!("  Example (overriding drift & vol): {}/simulate/stock?asset_identifier=DEFAULT_STOCK&initial_price=150&days=20&time_step_days=1&drift=0.07&volatility=0.25&seed=123", base_url);


    // Option Pricing - Black-Scholes (POST)
    println!("\n[POST] Option Pricing (Black-Scholes):");
    println!("  Calculates the price of a European option using the Black-Scholes model.");
    println!("  Endpoint: {}/simulate/option/black_scholes", base_url);
    println!("  Method: POST");
    println!("  Body (JSON): {{ \"underlying_price\": 100.0, \"strike_price\": 105.0, \"time_to_maturity_years\": 0.5, \"risk_free_rate\": 0.02, \"volatility\": 0.22, \"option_type\": \"Call\" }}");
    println!("  (Note: OptionType can be \"Call\" or \"Put\")");

    // Option Pricing - Monte Carlo (POST)
    println!("\n[POST] Option Pricing (Monte Carlo):");
    println!("  Calculates the price of a European option using Monte Carlo simulation.");
    println!("  Endpoint: {}/simulate/option/monte_carlo", base_url);
    println!("  Method: POST");
    println!("  Body (JSON): {{ \"underlying_initial_price\": 100.0, \"strike_price\": 102.0, \"time_to_maturity_years\": 0.75, \"risk_free_rate\": 0.025, \"underlying_volatility\": 0.20, \"option_type\": \"Put\", \"num_paths\": 10000, \"num_steps_per_path\": 100, \"seed\": 456 }}");
    // Corrected field name to num_steps_per_path in the example

    // Futures Simulation (POST)
    println!("\n[POST] Futures Simulation:");
    println!("  Simulates a futures contract price path.");
    println!("  Endpoint: {}/simulate/future", base_url);
    println!("  Method: POST");
    println!("  Body (JSON): {{ \"underlying_symbol\": \"CRUDE_OIL\", \"initial_spot_price\": 70.0, \"risk_free_rate\": 0.03, \"volatility\": 0.25, \"time_to_maturity_days\": 90, \"time_step_days\": 1, \"seed\": 789 }}");

    // ETF Simulation (POST)
    println!("\n[POST] ETF Simulation:");
    println!("  Simulates the Net Asset Value (NAV) of an ETF based on its constituents.");
    println!("  Endpoint: {}/simulate/etf", base_url);
    println!("  Method: POST");
    println!("  Body (JSON):");
    println!("  {{");
    println!("    \"constituents\": [");
    println!("      {{ \"symbol\": \"STOCK_A\", \"initial_price\": 50.0, \"drift\": 0.1, \"volatility\": 0.3, \"weight\": 0.6 }},");
    println!("      {{ \"symbol\": \"STOCK_B\", \"initial_price\": 80.0, \"drift\": 0.05, \"volatility\": 0.2, \"weight\": 0.4 }}");
    println!("    ],");
    println!("    \"simulation_days\": 15,");
    println!("    \"time_step_days\": 1,");
    println!("    \"seed\": 101");
    println!("  }}");
    println!("-----------------------------------------------------------------------");

    // Pass app_config_data to run_server
    run_server(address, app_config_data).await
}
