use crate::location_search::get_location;
use crate::model::{
    ApiError, ApiResponse, HotelSearchArgs, HotelSearchData,
};
use chrono::{Duration, Utc};
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde_json::{json};
use std::collections::HashMap;
use std::env;

pub struct LocationSearchTool;

impl Tool for LocationSearchTool {
    const NAME: &'static str = "search_location";
    type Error = HotelSearchError;
    type Args = HotelSearchArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Search for location based on the city name".to_string(),
            parameters: json!({
                  "type": "object",
                  "properties": {
                      "query": {"type": "string"},
            /*      "geoId": {
                      "type": "integer",
                      "description": "The geographic ID of the location to search in. Obtained from LocationSearchTool."
                  },*/
                  "checkIn": {
                      "type": "string",
                      "description": "The check-in date for the hotel search, in YYYY-MM-DD format."
                  },
                  "checkOut": {
                      "type": "string",
                      "description": "The check-out date for the hotel search, in YYYY-MM-DD format."
                  },
                  "adults": {
                      "type": "integer",
                      "description": "The number of adults staying."
                  },
                  "childrenAges": { // This should match your Vec<u32>
                      "type": "array",
                      "items": {"type": "integer"},
                      "description": "A comma-separated list of children's ages, if any."
                  },
                  "rooms": {
                      "type": "integer",
                      "description": "The number of rooms needed."
                  },
              },
              "required": ["query", "checkIn", "checkOut", "adults"],
                  }),
        }
    }
async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Fetch the API Key
        let api_key = env::var("RAPIDAPI_KEY").map_err(|_| HotelSearchError::MissingApiKey)?;
        let location_opt = get_location(args.clone(), api_key.clone()).await?;
        println!("get_location returned: {:?}", location_opt);
        let location = location_opt.ok_or(HotelSearchError::InvalidResponse)?;
        // Build Query Parameters
        let check_in = args
            .check_in
            .unwrap_or_else(|| (Utc::now() + Duration::days(1)).date_naive().to_string());
        let check_out = args
            .check_out
            .unwrap_or_else(|| (Utc::now() + Duration::days(7)).date_naive().to_string());
        let adults = args.adults.unwrap_or_else(|| 1);
        let children_ages = args.children_ages.unwrap_or_else(|| Vec::new());

        let mut query_params = HashMap::new();
        query_params.insert("geoId", location.geo_id.to_string());
        query_params.insert("checkIn", check_in);
        query_params.insert("checkOut", check_out);
        query_params.insert("adults", adults.to_string());
        query_params.insert(
            "childrenAges",
            children_ages
                .iter()
                .map(|age| age.to_string())
                .collect::<Vec<_>>()
                .join(","),
        );
        query_params.insert("rooms", args.rooms.map_or(String::new(), |v| v.to_string()));
        println!("Sending query params to Tripadvisor: {:?}", query_params);
        // Make the API Request
        let client = reqwest::Client::new();
        let response = client
            .get("https://tripadvisor16.p.rapidapi.com/api/v1/hotels/searchHotels")
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
            .map_err(|e| HotelSearchError::HttpRequestFailed(e.to_string()))?;
        let status = response.status();
        // Parse and Format the Response
        let raw_text = response
            .text()
            .await
            .map_err(|e| HotelSearchError::HttpRequestFailed(e.to_string()))?;

        // Check if the HTTP status code indicates success (2xx range)
        if status.is_success() {
            // It's an HTTP error (e.g., 400, 500). Try to parse the error body.
            let api_error: ApiError = serde_json::from_str(&raw_text).map_err(|e| {
                // If the error response itself can't be parsed, log original text
                eprintln!(
                    "Failed to parse API error response (HTTP status {}): {}. Raw Text: {}",
                    status, e, raw_text
                );
                HotelSearchError::HttpRequestFailed(format!(
                    "API returned HTTP error ({}). Could not parse error details. Raw: {}",
                    status, raw_text
                ))
            })?;

            // Return a specific error message based on the API's error response
            return Err(HotelSearchError::HttpRequestFailed(format!(
                "Tripadvisor API Error (HTTP Status: {}): {}",
                status, api_error.message
            )));
        }

        let response_data: ApiResponse<HotelSearchData> =
            serde_json::from_str(&raw_text).map_err(|e| {
                // Log the JSON parsing error and the text that caused it
                eprintln!(
                    "Failed to parse JSON response: {} \nRaw Text: {}",
                    e, raw_text
                );
                HotelSearchError::HttpRequestFailed(format!("Failed to parse API response: {}", e))
            })?;

        let mut output = String::new();
        output.push_str("Here are some hotel options:\n\n");

        // Check if there are any hotels returned
        if response_data.data.data.is_empty() {
            output.push_str("No hotels found matching your criteria.");
        } else {
            for (i, option) in response_data.data.data.iter().enumerate() {
                output.push_str(&format!("{}. **{}**\n", i + 1, option.title));

                if let Some(rating) = &option.bubble_rating {
                    output.push_str(&format!(
                        "  Rating: {}/5 ({} reviews)\n",
                        rating.rating, rating.count
                    ));
                }

                if let Some(info) = &option.primary_info {
                    output.push_str(&format!("  Features: {}\n", info));
                }
                if let Some(info) = &option.secondary_info {
                    output.push_str(&format!("  Location: {}\n", info));
                }

                // Handle price, which is often null in these APIs
                if let Some(price) = &option.price_for_display {
                    output.push_str(&format!("  Price: {}\n", price));
                } else {
                    // Check if provider exists to suggest where to check
                    if let Some(provider) = &option.provider {
                        output.push_str(&format!(
                            "  Price: Not available directly. Check on {}.\n",
                            provider
                        ));
                    } else {
                        output.push_str(
                            "  Price: Not available directly. Please check provider websites.\n",
                        );
                    }
                }

                output.push_str("\n"); // Add a blank line between hotels

                // Limit output to, say, the top 5 hotels for brevity for the LLM
                if i >= 4 {
                    // Display up to 5 hotels
                    output.push_str("...and more options available if you'd like to see them.\n");
                    break;
                }
            }
        }

        Ok(output) // Return the formatted string
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HotelSearchError {
    #[error("HTTP request failed: {0}")]
    HttpRequestFailed(String),
    #[error("Invalid response structure")]
    InvalidResponse,
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Missing API key")]
    MissingApiKey,
}
