use stock_price_simulator::config::GlobalConfig;
// run_server is not directly used here anymore, http_server module's handlers are referenced.
use actix_web::{dev::ServerHandle, web, App, HttpServer, middleware::Logger, rt as actix_rt};
use std::net::TcpListener;
use std::time::Duration;


// Helper function to spawn the server for testing
// Returns the base URL and the ServerHandle to stop it
async fn spawn_test_app_server(config: GlobalConfig) -> (String, ServerHandle) {
    let port = portpicker::pick_unused_port().expect("No free ports found");
    let address = format!("127.0.0.1:{}", port);

    let listener = TcpListener::bind(&address).unwrap_or_else(|e| panic!("Failed to bind to picked port {}: {}", port, e));
    drop(listener);

    let server_address = address.clone();
    let app_config_data = web::Data::new(config.clone());

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let system_runner = actix_rt::System::new();

        let server = HttpServer::new(move || {
            App::new()
                .app_data(app_config_data.clone())
                .wrap(Logger::default())
                .route("/simulate/stock", web::get().to(stock_price_simulator::http_server::simulate_stock_handler))
                .route("/simulate/option/black_scholes", web::post().to(stock_price_simulator::http_server::simulate_option_bs_handler))
                .route("/simulate/option/monte_carlo", web::post().to(stock_price_simulator::http_server::simulate_option_mc_handler))
                .route("/simulate/future", web::post().to(stock_price_simulator::http_server::simulate_future_handler))
                .route("/simulate/etf", web::post().to(stock_price_simulator::http_server::simulate_etf_handler))
        })
        .bind(&server_address)
        .unwrap_or_else(|e| panic!("Failed to bind test server: {}", e))
        .workers(1)
        .run();

        let _ = tx.send(server.handle());
        system_runner.block_on(server).unwrap_or_else(|e| eprintln!("Test server runtime error: {}", e));
    });

    let handle = rx.recv_timeout(Duration::from_secs(10))
                   .expect("Test server handle not received within timeout. Server might have panicked on startup.");

    tokio::time::sleep(Duration::from_millis(200)).await;

    (format!("http://{}", address), handle)
}


#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;
    use stock_price_simulator::api_models::{ApiResponse, StockData, ApiErrorResponse, OptionData};
    use serde_json::json;

    #[actix_web::test]
    async fn initial_server_spawn_test() {
        let test_config = stock_price_simulator::config::load_config("config.test.toml")
            .expect("Failed to load config.test.toml for server spawn test");

        let (base_url, server_handle) = spawn_test_app_server(test_config.clone()).await;
        println!("Test server spawned at {}", base_url);

        let client = Client::new();
        let resp_result = client.get(&base_url).send().await;
        assert!(resp_result.is_ok(), "Failed to send request to test server root: {:?}", resp_result.err());
        let resp = resp_result.unwrap();
        println!("Root path response status: {}", resp.status());
        assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);

        server_handle.stop(true).await;
    }

    #[actix_web::test]
    async fn test_simulate_stock_success_with_config() {
        let test_config = stock_price_simulator::config::load_config("config.test.toml")
            .expect("Failed to load test config for stock success");
        let (base_url, server_handle) = spawn_test_app_server(test_config).await;

        let client = Client::new();
        let url = format!(
            "{}/simulate/stock?asset_identifier=TEST_DEFAULT&initial_price=100.0&days=10&time_step_days=1.0&seed=123",
            base_url
        );
        let resp = client.get(&url).send().await.expect("Request failed");

        assert_eq!(resp.status(), reqwest::StatusCode::OK);
        let api_resp = resp.json::<ApiResponse<StockData>>().await.expect("Failed to parse success response");
        assert_eq!(api_resp.status, "success");
        assert_eq!(api_resp.data.symbol, "TEST_DEFAULT");
        assert_eq!(api_resp.data.prices.len(), 10);
        assert_eq!(api_resp.data.timestamps.len(), 10);
        server_handle.stop(true).await;
    }

    #[actix_web::test]
    async fn test_simulate_stock_success_with_overrides() {
        let test_config = stock_price_simulator::config::load_config("config.test.toml")
            .expect("Failed to load test config for stock overrides");
        let (base_url, server_handle) = spawn_test_app_server(test_config).await;

        let client = Client::new();
        let url = format!(
            "{}/simulate/stock?asset_identifier=TEST_OVERRIDE&initial_price=100.0&days=5&time_step_days=1.0&drift=0.15&volatility=0.35&seed=456",
            base_url
        );
        let resp = client.get(&url).send().await.expect("Request failed");

        assert_eq!(resp.status(), reqwest::StatusCode::OK);
        let api_resp = resp.json::<ApiResponse<StockData>>().await.expect("Failed to parse success response");
        assert_eq!(api_resp.status, "success");
        assert_eq!(api_resp.data.symbol, "TEST_OVERRIDE");
        assert_eq!(api_resp.data.prices.len(), 5);
        server_handle.stop(true).await;
    }

    #[actix_web::test]
    async fn test_simulate_stock_failure_invalid_asset_identifier() {
        let test_config = stock_price_simulator::config::load_config("config.test.toml")
            .expect("Failed to load test config for invalid asset ID");
        let (base_url, server_handle) = spawn_test_app_server(test_config).await;

        let client = Client::new();
        let url = format!(
            "{}/simulate/stock?asset_identifier=NON_EXISTENT&initial_price=100.0&days=10&time_step_days=1.0",
            base_url
        );
        let resp = client.get(&url).send().await.expect("Request failed");

        assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
        let err_resp = resp.json::<ApiErrorResponse>().await.expect("Failed to parse error response");
        assert_eq!(err_resp.status, "error");
        assert!(err_resp.error.contains("No model config found for stock identifier: NON_EXISTENT"));
        server_handle.stop(true).await;
    }

    #[actix_web::test]
    async fn test_simulate_stock_failure_missing_param() {
        let test_config = stock_price_simulator::config::load_config("config.test.toml")
            .expect("Failed to load test config for missing param");
        let (base_url, server_handle) = spawn_test_app_server(test_config).await;

        let client = Client::new();
        let url = format!("{}/simulate/stock?asset_identifier=TEST_DEFAULT&days=10&time_step_days=1.0", base_url);
        let resp = client.get(&url).send().await.expect("Request failed");

        assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
        server_handle.stop(true).await;
    }

    #[actix_web::test]
    async fn test_simulate_option_bs_success() {
        let test_config = stock_price_simulator::config::load_config("config.test.toml")
            .expect("Failed to load test config for BS option success");
        let (base_url, server_handle) = spawn_test_app_server(test_config).await;
        let client = Client::new();
        let url = format!("{}/simulate/option/black_scholes", base_url);

        let option_input = json!({
            "underlying_price": 100.0,
            "strike_price": 105.0,
            "time_to_maturity_years": 0.5,
            "risk_free_rate": 0.02,
            "volatility": 0.22,
            "option_type": "Call"
        });

        let resp = client.post(&url).json(&option_input).send().await.expect("Request failed");
        assert_eq!(resp.status(), reqwest::StatusCode::OK);
        let api_resp = resp.json::<ApiResponse<OptionData>>().await.expect("Failed to parse success response");
        assert_eq!(api_resp.status, "success");
        assert!(api_resp.data.price.is_some());
        assert!(api_resp.data.price.unwrap() > 0.0);
        assert_eq!(api_resp.data.option_type, "Call");
        server_handle.stop(true).await;
    }

    #[actix_web::test]
    async fn test_simulate_option_bs_malformed_json() {
        let test_config = stock_price_simulator::config::load_config("config.test.toml")
            .expect("Failed to load test config for BS malformed JSON");
        let (base_url, server_handle) = spawn_test_app_server(test_config).await;
        let client = Client::new();
        let url = format!("{}/simulate/option/black_scholes", base_url);

        let malformed_body = "{ \"underlying_price\": 100.0, \"strike_price\": 105.0, ";

        let resp = client.post(&url)
            .header("Content-Type", "application/json")
            .body(malformed_body)
            .send()
            .await
            .expect("Request failed");

        assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
        server_handle.stop(true).await;
    }

    #[actix_web::test]
    async fn test_simulate_option_bs_invalid_data_value() {
        let test_config = stock_price_simulator::config::load_config("config.test.toml")
            .expect("Failed to load test config for BS invalid data");
        let (base_url, server_handle) = spawn_test_app_server(test_config).await;
        let client = Client::new();
        let url = format!("{}/simulate/option/black_scholes", base_url);

        let invalid_data_input = json!({
            "underlying_price": 100.0,
            "strike_price": 105.0,
            "time_to_maturity_years": -0.5,
            "risk_free_rate": 0.02,
            "volatility": 0.22,
            "option_type": "Call"
        });

        let resp = client.post(&url).json(&invalid_data_input).send().await.expect("Request failed");
        assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
        let err_resp = resp.json::<ApiErrorResponse>().await.expect("Failed to parse error response");
        assert_eq!(err_resp.status, "error");
        assert!(err_resp.error.contains("Time to maturity (T) must be positive if not zero"));
        server_handle.stop(true).await;
    }
}
