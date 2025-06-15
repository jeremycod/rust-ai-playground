use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde_json::json;
use crate::hotel_search_tool::HotelSearchError;
use crate::location_search::get_location_from_api;
use crate::model::{LocationSearchArgs, LocationOption, LocationSearchResult};
use std::env;

pub struct LocationSearchTool;

impl Tool for LocationSearchTool {
    const NAME: &'static str = "search_location";
    type Error = HotelSearchError;
    type Args = LocationSearchArgs;
    type Output = LocationSearchResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Finds the geographic ID for a given city or place name. Required before searching for hotels.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The name of the city, region, or place to find the geographic ID for (e.g., 'London', 'Paris, France', 'New York City')."
                    },
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("LocationSearchTool: Calling get_location_from_api for query: {}", args.query);
        let api_key = env::var("RAPIDAPI_KEY").map_err(|_| HotelSearchError::MissingApiKey)?;

        let location_opt = get_location_from_api(args.clone(), api_key).await?;

        let location = location_opt.ok_or(HotelSearchError::InvalidResponse("Location not found from API for input query".to_string()))?; // Or a more specific error

        println!("LocationSearchTool: Found geoId: {} for {}", location.geo_id, args.query);

        Ok(LocationSearchResult {
            geo_id: location.geo_id,
            location_name: args.query, // Return the original query so LLM knows what geoId maps to
        })
    }
}