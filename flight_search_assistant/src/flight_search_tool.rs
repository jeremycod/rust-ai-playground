use chrono::{Duration, Utc};
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;

#[derive(Deserialize)]
pub struct FlightSearchArgs {
    source: String,
    destination: String,
    date: Option<String>,
    sort: Option<String>,
    service: Option<String>,
    itinerary_type: Option<String>,
    adults: Option<u8>,
    seniors: Option<u8>,
    currency: Option<String>,
    nearby: Option<String>,
    nonstop: Option<String>,
}

#[derive(Serialize)]
pub struct FlightOption {
    pub airline: String,
    pub flight_number: String,
    pub departure: String,
    pub arrival: String,
    pub duration: String,
    pub stops: usize,
    pub price: f64,
    pub currency: String,
    pub booking_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum FlightSearchError {
    #[error("HTTP request failed: {0}")]
    HttpRequestFailed(String),
    #[error("Invalid response structure")]
    InvalidResponse,
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Missing API key")]
    MissingApiKey,
}

pub struct FlightSearchTool;

impl Tool for FlightSearchTool {
    const NAME: &'static str = "search_flights";

    type Args = FlightSearchArgs;
    type Output = String;
    type Error = FlightSearchError;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Search for flights between two airports".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "source": {"type": "string", "description":"Source airport code (e.g., 'JFK')"},
                    "destination": {"type": "string", "description":"Destination airport code (e.g., 'LAX')" },
                    "date": {"type": "string", "description":"Flight date in 'YYYY-MM-DD' format"},

                },
                "required": ["source", "destination"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Fetch the API Key
        let api_key = env::var("RAPIDAPI_KEY").map_err(|_| FlightSearchError::MissingApiKey)?;

        // Set Default Values
        // If the user doesn't provide a date, we'll default to 30 days from now.
        let date = args.date.unwrap_or_else(|| {
            let date = Utc::now() + Duration::days(30);
            date.format("%Y-%m-%d").to_string()
        });

        // Build Query Parameters
        let mut query_params = HashMap::new();
        query_params.insert("sourceAirportCode", args.source);
        query_params.insert("destinationAirportCode", args.destination);
        query_params.insert("date", date);
        query_params.insert("itineraryType", "ONE_WAY".to_string());
        query_params.insert("classOfService", "ECONOMY".to_string());

        // Make the API Request
        let client = reqwest::Client::new();
        let response = client
            .get("https://tripadvisor16.p.rapidapi.com/api/v1/flights/searchFlights")
            .headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    "X-RapidAPI-Host",
                    "tripadvisor16.p.rapidapi.com".parse().unwrap(),
                );
                headers.insert("X-RapidAPI-Key", api_key.parse().unwrap());
                headers
            })
            .query(&query_params)
            .send()
            .await
            .map_err(|e| FlightSearchError::HttpRequestFailed(e.to_string()))?;

        // Parse and Format the Response
        let text = response
            .text()
            .await
            .map_err(|e| FlightSearchError::HttpRequestFailed(e.to_string()))?;

        let _data: Value = serde_json::from_str(&text)
            .map_err(|e| FlightSearchError::HttpRequestFailed(e.to_string()))?;

        let flight_options: Vec<FlightOption> = Vec::new();

        // Here, we need to extract the flight options. (It's quite detailed, so we've omitted the full code to keep the focus clear.)

        // Format the flight options into a readable string
        let mut output = String::new();
        output.push_str("Here are some flight options:\n\n");

        for (i, option) in flight_options.iter().enumerate() {
            output.push_str(&format!("{}. **Airline**: {}\n", i + 1, option.airline));
            // Additional formatting...
        }


        Ok("Flight search results".to_string())
    }
}
