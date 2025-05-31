use serde::Deserialize;
use actix_web::{web, HttpResponse, http::StatusCode}; // Removed Responder
use chrono::NaiveDateTime;

// TimeSeries unused, removed.
use crate::api_models::{ApiResponse, StockData, ApiErrorResponse};
use crate::api_interface;

// --- Request Structs ---

#[derive(Deserialize, Debug)]
pub struct StockSimulationQueryParams {
    pub initial_price: f64,
    pub drift: f64,
    pub volatility: f64,
    pub days: usize,
    pub time_step_days: f64,
    pub seed: Option<u64>,
    pub symbol: String,
}

// --- Helper Functions ---

fn format_timestamps(timestamps: &[NaiveDateTime]) -> Vec<String> {
    timestamps.iter().map(|t| t.format("%Y-%m-%dT%H:%M:%S").to_string()).collect()
}

// Return HttpResponse directly to unify types in match arms
fn success_response<T: serde::Serialize + PartialEq>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse { // ApiResponse requires T: PartialEq
        status: "success".to_string(),
        data,
    })
}

// Return HttpResponse directly
fn error_response(err_msg: String, status_code: StatusCode) -> HttpResponse {
    HttpResponse::build(status_code).json(ApiErrorResponse {
        status: "error".to_string(),
        error: err_msg,
    })
}

// --- API Handlers ---

// GET /simulate/stock
async fn simulate_stock_handler(
    params: web::Query<StockSimulationQueryParams>,
) -> HttpResponse { // Return HttpResponse
    match api_interface::simulate_stock(
        params.initial_price,
        params.drift,
        params.volatility,
        params.days,
        params.time_step_days,
        params.seed,
    ) {
        Ok(time_series) => {
            let response_data = StockData {
                symbol: params.symbol.clone(),
                timestamps: format_timestamps(&time_series.timestamps),
                prices: time_series.prices,
            };
            success_response(response_data)
        }
        Err(e) => error_response(e.to_string(), StatusCode::BAD_REQUEST),
    }
}

use actix_web::{App, HttpServer, middleware::Logger};
use crate::api_models::{OptionData, FutureData, EtfData}; // Added FutureData, EtfData
use crate::option_pricing::EuropeanOption;
use crate::api_interface::MonteCarloEuropeanOptionInput;
use crate::futures_simulation::FuturesContract;
use crate::etf_simulation::EtfDefinition;


// POST /simulate/option/black_scholes
async fn simulate_option_bs_handler(
    option_params: web::Json<EuropeanOption>, // EuropeanOption needs Deserialize
) -> HttpResponse { // Return HttpResponse
    // Access inner data using .0 to avoid consuming web::Json if fields are needed later,
    // though for this specific handler, all fields are passed to the api_interface function.
    // If option_params itself was needed after the call, .into_inner() would consume it.
    // Here, option_params.0 provides a reference to EuropeanOption.
    match api_interface::price_european_option_black_scholes(
        option_params.0.underlying_price,
        option_params.0.strike_price,
        option_params.0.time_to_maturity_years,
        option_params.0.risk_free_rate,
        option_params.0.volatility,
        option_params.0.option_type,
    ) {
        Ok(price) => {
            let response_data = OptionData {
                underlying_symbol: "N/A".to_string(),
                option_type: format!("{:?}", option_params.0.option_type),
                strike_price: option_params.0.strike_price,
                maturity_date: "N/A (calculated from TTM)".to_string(),
                price: Some(price),
                ..Default::default()
            };
            success_response(response_data)
        }
        Err(e) => error_response(e.to_string(), StatusCode::BAD_REQUEST),
    }
}

// POST /simulate/option/monte_carlo
async fn simulate_option_mc_handler(
    params: web::Json<MonteCarloEuropeanOptionInput>,
) -> HttpResponse {
    // Use params.0 to access the inner MonteCarloEuropeanOptionInput data
    // The api_interface function takes a reference, so no ownership issues here.
    match api_interface::price_european_option_monte_carlo(&params.0) {
        Ok(price) => {
            let response_data = OptionData {
                underlying_symbol: "N/A".to_string(), // MC input doesn't have a separate symbol field
                option_type: format!("{:?}", params.0.option_type),
                strike_price: params.0.strike_price,
                maturity_date: "N/A (calculated from TTM)".to_string(),
                price: Some(price),
                ..Default::default()
            };
            success_response(response_data)
        }
        Err(e) => error_response(e.to_string(), StatusCode::BAD_REQUEST),
    }
}

// POST /simulate/future
async fn simulate_future_handler(
    params: web::Json<FuturesContract>,
) -> HttpResponse {
    // api_interface::simulate_futures expects a reference
    match api_interface::simulate_futures(&params.0) {
        Ok(time_series) => {
            let response_data = FutureData {
                contract_symbol: params.0.underlying_symbol.clone(), // Assuming FuturesContract has this
                timestamps: format_timestamps(&time_series.timestamps),
                prices: time_series.prices,
                spot_prices: None, // Current simulate_futures doesn't return spot path
            };
            success_response(response_data)
        }
        Err(e) => error_response(e.to_string(), StatusCode::BAD_REQUEST),
    }
}

// POST /simulate/etf
async fn simulate_etf_handler(
    params: web::Json<EtfDefinition>,
) -> HttpResponse {
    // api_interface::simulate_etf expects a reference
    match api_interface::simulate_etf(&params.0) {
        Ok(time_series) => {
            let response_data = EtfData {
                etf_symbol: "SIMULATED_ETF".to_string(), // EtfDefinition has no single symbol
                timestamps: format_timestamps(&time_series.timestamps),
                nav_values: time_series.prices,
            };
            success_response(response_data)
        }
        Err(e) => error_response(e.to_string(), StatusCode::BAD_REQUEST),
    }
}

// --- Server Setup ---
pub async fn run_server(address: &str) -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info,stock_price_simulator=info"); // Ensure RUST_LOG is set for info level for both
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/simulate/stock", web::get().to(simulate_stock_handler))
            .route("/simulate/option/black_scholes", web::post().to(simulate_option_bs_handler))
            .route("/simulate/option/monte_carlo", web::post().to(simulate_option_mc_handler))
            .route("/simulate/future", web::post().to(simulate_future_handler))
            .route("/simulate/etf", web::post().to(simulate_etf_handler))
    })
    .bind(address)?
    .run()
    .await
}
