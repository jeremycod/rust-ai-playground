use std::collections::HashMap;
use crate::hotel_search_tool::HotelSearchError;
use crate::model::{LocationSearchArgs, ApiResponse, LocationOption};

pub async fn get_location_from_api(args: LocationSearchArgs, api_key: String) -> Result<Option<LocationOption>, HotelSearchError> {

    let mut query_params = HashMap::new();
    query_params.insert("query", args.query);

    let client = reqwest::Client::new();
    let response = client
        .get("https://tripadvisor16.p.rapidapi.com/api/v1/hotels/searchLocation")
        .headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                "X-RapidApi-Host",
                "tripadvisor16.p.rapidapi.com".parse().unwrap(),
            );
            headers.insert("X-RapidAPI-Key", api_key.parse().unwrap());
            headers
        })
        .query(&query_params)
        .send()
        .await
        .map_err(|e| HotelSearchError::HttpRequestFailed(e.to_string()))?;

    // Parse and Format the Response
    let text = response
        .text()
        .await
        .map_err(|e| HotelSearchError::HttpRequestFailed(e.to_string()))?;

    let api_response: ApiResponse<Vec<LocationOption>> = serde_json::from_str(&text)
        .map_err(|e| HotelSearchError::HttpRequestFailed(e.to_string()))?;

    if !api_response.status {
        return Err(HotelSearchError::ApiError(api_response.message));
    }

    let location_options = api_response.data;

    Ok(location_options.into_iter().next())
}