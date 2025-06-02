use serde::{Serialize, Deserialize}; // Added Deserialize
// use chrono::NaiveDateTime; // Not directly used in these structs, but for transformation logic later

#[derive(Serialize, Deserialize, Debug, PartialEq)] // Added Deserialize
// Let derive macro add the Deserialize bound for T
pub struct ApiResponse<T: Serialize + PartialEq> {
    pub status: String,
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)] // Added Deserialize
pub struct ApiErrorResponse {
    pub status: String,
    pub error: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)] // Added Deserialize
pub struct StockData {
    pub symbol: String,
    pub timestamps: Vec<String>, // ISO 8601 format
    pub prices: Vec<f64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Default)] // Added Deserialize
pub struct OptionData {
    pub underlying_symbol: String,
    pub option_type: String, // "Call" or "Put"
    pub strike_price: f64,
    pub maturity_date: String, // ISO 8601 or similar
    pub price: Option<f64>,
    pub underlying_prices: Option<Vec<f64>>,
    pub option_prices: Option<Vec<f64>>,
    pub timestamps: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)] // Added Deserialize
pub struct FutureData {
    pub contract_symbol: String,
    pub timestamps: Vec<String>,
    pub prices: Vec<f64>,
    pub spot_prices: Option<Vec<f64>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)] // Added Deserialize
pub struct EtfData {
    pub etf_symbol: String,
    pub timestamps: Vec<String>,
    pub nav_values: Vec<f64>,
}
