use crate::model::{ApiError, ApiResponse, HotelSearchArgs, HotelSearchData};
use crate::utils::parse_and_infer_year;
use chrono::{Duration, Local, Utc};
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde_json::json;
use std::collections::HashMap;
use std::env;

pub struct HotelSearchTool;

impl Tool for HotelSearchTool {
    const NAME: &'static str = "search_hotel";
    type Error = HotelSearchError;
    type Args = HotelSearchArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Search for hotel based on the give parameters and location".to_string(),
            parameters: json!({
                            "type": "object",
                            "properties": {
                                "query": {"type": "string"},
                            "geoId": {
                                "type": "integer",
                                "description": "The geographic ID of the location to search in. Obtained from LocationSearchTool."
                            },
                            "checkIn": {
                                "type": "string",
                                 "description": "The check-in date. Can be YYYY-MM-DD (e.g., 2025-08-08) or MM-DD / Month Day (e.g., 08-08 or August 8th). If the year is omitted, the program will infer the next upcoming occurrence of that date."
                            },
                            "checkOut": {
                                "type": "string",
                                "description": "The check-out date. Can be YYYY-MM-DD (e.g., 2025-08-14) or MM-DD / Month Day (e.g., 08-14 or August 14th). If the year is omitted, the program will infer the next upcoming occurrence of that date."
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
        let today = Local::now().date_naive(); // Get today's date
        // Build Query Parameters
        let check_in_date = if let Some(ci_str) = args.check_in {
            parse_and_infer_year(&ci_str, today)? // Custom parsing function
        } else {
            today + Duration::days(1) // Default to tomorrow
        };

        let check_out_date = if let Some(co_str) = args.check_out {
            parse_and_infer_year(&co_str, today)? // Custom parsing function
        } else {
            check_in_date + Duration::days(1) // Default to day after check-in
        };
        let check_in = check_in_date.format("%Y-%m-%d").to_string();
        let check_out = check_out_date.format("%Y-%m-%d").to_string();
        let adults = args.adults.unwrap_or_else(|| 1);
        let children_ages = args.children_ages.unwrap_or_else(|| Vec::new());

        let mut query_params = HashMap::new();
        query_params.insert("geoId", args.geo_id.to_string());
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
        println!(
            "Sending query params to Tripadvisor:\n{}",
            serde_json::to_string_pretty(&query_params)
                .unwrap_or_else(|_| format!("{:?}", query_params))
        );
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
        if !status.is_success() {
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
        output.push_str("🏨 Here are some hotel options:\n\n");

        if response_data.data.data.is_empty() {
            output.push_str("No hotels found matching your criteria.");
        } else {
            for (i, option) in response_data.data.data.iter().enumerate() {
                output.push_str(&format!("{}. {}\n", i + 1, option.title));

                if let Some(rating) = &option.bubble_rating {
                    output.push_str(&format!(
                        "   Rating: {}★ ({} reviews)\n",
                        rating.rating, rating.count
                    ));
                }

                if let Some(info) = &option.primary_info {
                    output.push_str(&format!("   • Features: {}\n", info));
                }
                if let Some(info) = &option.secondary_info {
                    output.push_str(&format!("   • Location: {}\n", info));
                }

                if let Some(price) = &option.price_for_display {
                    output.push_str(&format!("   • Price: {}\n", price));
                } else if let Some(provider) = &option.provider {
                    output.push_str(&format!(
                        "   • Price: Not available directly. Check on {}.\n",
                        provider
                    ));
                } else {
                    output.push_str("   • Price: Not available. Please check provider websites.\n");
                }

                output.push('\n');

                if i >= 4 {
                    output.push_str("...and more options available.\n");
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
    #[error("Invalid response structure: {0}")]
    InvalidResponse(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Missing API key")]
    MissingApiKey,
}
