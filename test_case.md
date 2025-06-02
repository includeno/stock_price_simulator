# API Test Cases & cURL Examples

## Introduction

This document provides examples of how to interact with the Stock Price Simulator's HTTP API.
The default base URL for the server is `http://127.0.0.1:8080`.

All successful responses are wrapped in an `ApiResponse` structure:
```json
{
  "status": "success",
  "data": { /* specific data payload for the endpoint */ }
}
```

Error responses also follow a standard structure:
```json
{
  "status": "error",
  "error": "A message describing the error."
}
```

---

## 1. GET `/simulate/stock`

Simulates a stock price path based on configuration (looked up by `asset_identifier`) and allows for optional overrides of drift and volatility.

-   **HTTP Method:** `GET`
-   **URL Structure:** `/simulate/stock?asset_identifier=<id>&initial_price=<price>&days=<days>&time_step_days=<step>&[seed=<seed>]&[drift=<drift>]&[volatility=<volatility>]`

**Query Parameters:**

-   `asset_identifier` (string, required): Identifier for stock configuration (e.g., "DEFAULT_STOCK" from `config.toml`).
-   `initial_price` (float, required): Starting price of the stock.
-   `days` (integer, required): Number of simulation steps (e.g., trading days).
-   `time_step_days` (float, required): Duration of each simulation step in days (e.g., 1.0 for daily).
-   `seed` (integer, optional): Random seed for deterministic simulation.
-   `drift` (float, optional): Overrides the drift configured for the `asset_identifier`.
-   `volatility` (float, optional): Overrides the volatility configured for the `asset_identifier`.

**`curl` Example (using config values for drift/volatility):**

```bash
curl "http://127.0.0.1:8080/simulate/stock?asset_identifier=DEFAULT_STOCK&initial_price=100.0&days=20&time_step_days=1.0&seed=123"
```

**`curl` Example (overriding drift/volatility):**

```bash
curl "http://127.0.0.1:8080/simulate/stock?asset_identifier=DEFAULT_STOCK&initial_price=100.0&days=20&time_step_days=1.0&seed=123&drift=0.10&volatility=0.30"
```

**Example Success Response (200 OK):**

```json
{
  "status": "success",
  "data": {
    "symbol": "DEFAULT_STOCK",
    "timestamps": [
      "2024-01-01T00:00:00",
      "2024-01-02T00:00:00",
      "..."
    ],
    "prices": [
      100.00,
      100.12,
      // ...
    ]
  }
}
```

**Example Error Response (400 Bad Request - e.g., missing `initial_price`):**
(Note: Exact error for missing query params can depend on Actix's default Query extractor behavior if not customized)
```json
{
  "status": "error",
  "error": "Query deserialize error: missing field `initial_price` at line 1 column 70"
}
```

**Example Error Response (400 Bad Request - `asset_identifier` not found):**
```json
{
  "status": "error",
  "error": "No model config found for stock identifier: UNKNOWN_STOCK"
}
```

---

## 2. POST `/simulate/option/black_scholes`

Calculates the price of a European option using the Black-Scholes model.

-   **HTTP Method:** `POST`
-   **URL Structure:** `/simulate/option/black_scholes`

**Request Body (JSON):**

```json
{
  "underlying_price": 100.0,      // Current price of the underlying asset
  "strike_price": 105.0,          // Strike price of the option
  "time_to_maturity_years": 0.5,  // Time to maturity in years
  "risk_free_rate": 0.02,         // Annualized risk-free interest rate (e.g., 0.02 for 2%)
  "volatility": 0.22,             // Annualized volatility of the underlying asset (e.g., 0.22 for 22%)
  "option_type": "Call"           // Type of option: "Call" or "Put"
}
```

**`curl` Example:**

```bash
curl -X POST -H "Content-Type: application/json" \
-d '{ "underlying_price": 100.0, "strike_price": 105.0, "time_to_maturity_years": 0.5, "risk_free_rate": 0.02, "volatility": 0.22, "option_type": "Call" }' \
http://127.0.0.1:8080/simulate/option/black_scholes
```

**Example Success Response (200 OK):**

```json
{
  "status": "success",
  "data": {
    "underlying_symbol": "N/A", // Black-Scholes direct input doesn't typically include a symbol
    "option_type": "Call",
    "strike_price": 105.0,
    "maturity_date": "N/A (calculated from TTM)", // Input is TTM, not a specific date
    "price": 4.0319, // Example calculated price
    "underlying_prices": null,
    "option_prices": null,
    "timestamps": null
  }
}
```

**Example Error Response (400 Bad Request - e.g., negative volatility):**

```json
{
  "status": "error",
  "error": "Volatility (sigma) must be positive. Got -0.22"
}
```

---

## 3. POST `/simulate/option/monte_carlo`

Calculates the price of a European option using Monte Carlo simulation.

-   **HTTP Method:** `POST`
-   **URL Structure:** `/simulate/option/monte_carlo`

**Request Body (JSON):**

```json
{
  "underlying_initial_price": 100.0, // Initial price of the underlying asset
  "strike_price": 102.0,
  "time_to_maturity_years": 0.75,
  "risk_free_rate": 0.025,           // Used for discounting and risk-neutral drift
  "underlying_volatility": 0.20,     // Volatility for the GBM simulation of the underlying
  "option_type": "Put",              // "Call" or "Put"
  "num_paths": 10000,                // Number of simulation paths for Monte Carlo
  "num_steps_per_path": 100,         // Number of time steps in each path until maturity
  "seed": 456                        // Optional random seed
}
```

**`curl` Example:**

```bash
curl -X POST -H "Content-Type: application/json" \
-d '{ "underlying_initial_price": 100.0, "strike_price": 102.0, "time_to_maturity_years": 0.75, "risk_free_rate": 0.025, "underlying_volatility": 0.20, "option_type": "Put", "num_paths": 10000, "num_steps_per_path": 100, "seed": 456 }' \
http://127.0.0.1:8080/simulate/option/monte_carlo
```

**Example Success Response (200 OK):**

```json
{
  "status": "success",
  "data": {
    "underlying_symbol": "N/A",
    "option_type": "Put",
    "strike_price": 102.0,
    "maturity_date": "N/A (calculated from TTM)",
    "price": 6.9321, // Example calculated price
    "underlying_prices": null,
    "option_prices": null,
    "timestamps": null
  }
}
```

**Example Error Response (400 Bad Request - e.g., num_paths is 0):**

```json
{
  "status": "error",
  "error": "Invalid parameters for Monte Carlo pricing. Ensure T > 0, num_paths > 0, num_steps > 0."
}
```

---

## 4. POST `/simulate/future`

Simulates a futures contract price path based on the spot price evolving via GBM.

-   **HTTP Method:** `POST`
-   **URL Structure:** `/simulate/future`

**Request Body (JSON):**
(Corresponds to `FuturesContract` struct)
```json
{
  "underlying_symbol": "CRUDE_OIL", // Symbol for the underlying, used as contract identifier
  "initial_spot_price": 70.0,
  "risk_free_rate": 0.03,
  "volatility": 0.25,              // Volatility of the underlying spot price
  "time_to_maturity_days": 90,     // Initial time to maturity in days
  "time_step_days": 1,             // Granularity of simulation steps in days
  "seed": 789                       // Optional random seed
}
```

**`curl` Example:**

```bash
curl -X POST -H "Content-Type: application/json" \
-d '{ "underlying_symbol": "CRUDE_OIL", "initial_spot_price": 70.0, "risk_free_rate": 0.03, "volatility": 0.25, "time_to_maturity_days": 90, "time_step_days": 1, "seed": 789 }' \
http://127.0.0.1:8080/simulate/future
```

**Example Success Response (200 OK):**

```json
{
  "status": "success",
  "data": {
    "contract_symbol": "CRUDE_OIL",
    "timestamps": [
      "2024-01-01T00:00:00",
      "2024-01-02T00:00:00",
      "..."
    ],
    "prices": [
      70.52, // F_0 = S_0 * exp(rT)
      70.61,
      // ...
    ],
    "spot_prices": null // Not currently populated by this endpoint
  }
}
```

**Example Error Response (400 Bad Request - e.g., negative initial spot price):**

```json
{
  "status": "error",
  "error": "Initial spot price must be positive."
}
```

---

## 5. POST `/simulate/etf`

Simulates the Net Asset Value (NAV) of an ETF based on the simulated paths of its constituents.

-   **HTTP Method:** `POST`
-   **URL Structure:** `/simulate/etf`

**Request Body (JSON):**
(Corresponds to `EtfDefinition` struct)
```json
{
  "constituents": [
    { "symbol": "STOCK_A", "initial_price": 50.0, "drift": 0.1, "volatility": 0.3, "weight": 0.6 },
    { "symbol": "STOCK_B", "initial_price": 80.0, "drift": 0.05, "volatility": 0.2, "weight": 0.4 }
  ],
  "simulation_days": 15,   // Number of simulation steps/days for the NAV path
  "time_step_days": 1,     // Granularity of each step for constituent simulations
  "seed": 101              // Optional random seed for the overall ETF simulation
}
```

**`curl` Example:**

```bash
curl -X POST -H "Content-Type: application/json" \
-d '{ "constituents": [ { "symbol": "STOCK_A", "initial_price": 50.0, "drift": 0.1, "volatility": 0.3, "weight": 0.6 }, { "symbol": "STOCK_B", "initial_price": 80.0, "drift": 0.05, "volatility": 0.2, "weight": 0.4 } ], "simulation_days": 15, "time_step_days": 1, "seed": 101 }' \
http://127.0.0.1:8080/simulate/etf
```

**Example Success Response (200 OK):**

```json
{
  "status": "success",
  "data": {
    "etf_symbol": "SIMULATED_ETF", // Placeholder symbol
    "timestamps": [
      "2024-01-01T00:00:00",
      "2024-01-02T00:00:00",
      "..."
    ],
    "nav_values": [
      1.00,  // NAV is normalized to 1.0 at t=0 by construction
      1.003,
      // ...
    ]
  }
}
```

**Example Error Response (400 Bad Request - e.g., empty constituents list):**

```json
{
  "status": "error",
  "error": "ETF constituents list cannot be empty."
}
```
