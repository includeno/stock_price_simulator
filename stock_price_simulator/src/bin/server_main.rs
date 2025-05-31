use stock_price_simulator::http_server::run_server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let address = "127.0.0.1:8080";
    println!("Starting server on http://{} ...", address);
    println!("Try: http://{}/simulate/stock?initial_price=100&drift=0.05&volatility=0.2&days=10&time_step_days=1&symbol=TEST", address);
    run_server(address).await
}
