use stock_price_simulator::api_models::{
    ApiResponse, ApiErrorResponse, StockData, OptionData, FutureData, EtfData,
};
// No need for local serde::Deserialize import if api_models derive it.

// Helper macro for testing serde roundtrip
macro_rules! test_serde_roundtrip {
    ($name:ident, $struct_type:ty, $instance:expr) => {
        #[test]
        fn $name() {
            let original: $struct_type = $instance;

            // Serialize
            let json_string = serde_json::to_string(&original).expect("Serialization failed");

            // Deserialize directly into the original struct type
            let deserialized: $struct_type = serde_json::from_str(&json_string)
                .expect(&format!("Deserialization failed for {}", stringify!($struct_type)));

            assert_eq!(original, deserialized, "Roundtrip failed for {}", stringify!($struct_type));
        }
    };
}

test_serde_roundtrip!(
    test_api_stock_data_response,
    ApiResponse<StockData>,
    ApiResponse {
        status: "success".to_string(),
        data: StockData {
            symbol: "AAPL".to_string(),
            timestamps: vec!["2023-01-01T12:00:00Z".to_string()],
            prices: vec![150.0],
        },
    }
);

test_serde_roundtrip!(
    test_api_error_response,
    ApiErrorResponse,
    ApiErrorResponse {
        status: "error".to_string(),
        error: "Simulation failed".to_string(),
    }
);

test_serde_roundtrip!(
    test_stock_data_direct, // Test StockData itself if needed, though covered by ApiResponse
    StockData,
    StockData {
        symbol: "MSFT".to_string(),
        timestamps: vec!["2023-01-02T10:00:00Z".to_string()],
        prices: vec![280.50],
    }
);

test_serde_roundtrip!(
    test_option_data_full,
    OptionData,
    OptionData {
        underlying_symbol: "GOOG".to_string(),
        option_type: "Call".to_string(),
        strike_price: 1000.0,
        maturity_date: "2024-12-31".to_string(),
        price: Some(150.25),
        underlying_prices: Some(vec![950.0, 1000.0, 1050.0]),
        option_prices: Some(vec![50.0, 150.25, 250.50]),
        timestamps: Some(vec!["2023-01-01T00:00:00Z".to_string()]),
    }
);

test_serde_roundtrip!(
    test_option_data_single_price,
    OptionData,
    OptionData {
        underlying_symbol: "TSLA".to_string(),
        option_type: "Put".to_string(),
        strike_price: 200.0,
        maturity_date: "2025-06-30".to_string(),
        price: Some(25.50),
        ..Default::default() // Fill other Option fields with None
    }
);


test_serde_roundtrip!(
    test_future_data,
    FutureData,
    FutureData {
        contract_symbol: "ESZ23".to_string(),
        timestamps: vec!["2023-10-01T09:00:00Z".to_string()],
        prices: vec![4500.75],
        spot_prices: Some(vec![4490.25]),
    }
);

test_serde_roundtrip!(
    test_etf_data,
    EtfData,
    EtfData {
        etf_symbol: "SPY".to_string(),
        timestamps: vec!["2023-05-05T16:00:00Z".to_string()],
        nav_values: vec![450.55],
    }
);
