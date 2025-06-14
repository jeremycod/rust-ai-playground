use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize)]
pub struct HotelSearchArgs {
    pub query: String,
   // #[serde(rename = "geoId")]
   // pub geo_id: u64,
    #[serde(rename = "checkIn")]
    pub check_in: Option<String>,
    #[serde(rename = "checkOut")]
    pub check_out: Option<String>,
    pub adults: Option<u32>,
    #[serde(rename = "childrenAges")]
    pub children_ages: Option<Vec<u32>>,
    pub rooms: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationOption {
    title: String,
    #[serde(rename = "geoId")]
    pub geo_id: u64,
    #[serde(rename = "documentId")]
    document_id: String,
    #[serde(rename = "trackingItems")]
    tracking_items: String,
    #[serde(rename = "secondaryText")]
    secondary_text: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse<T> {
    pub status: bool,
    pub message: String,
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct HotelSearchData {
    // sortDisclaimer: String, // If you need this
    pub data: Vec<HotelOption>, // This is the list of hotels
}

// Represents a single hotel option
#[derive(Debug, Deserialize)]
pub struct HotelOption {
    pub id: String,
    pub title: String, // e.g., "1. Abode Bombay"
    #[serde(rename = "primaryInfo")]
    pub primary_info: Option<String>, // Can be null
    #[serde(rename = "secondaryInfo")]
    pub secondary_info: Option<String>, // Can be null
    #[serde(rename = "bubbleRating")]
    pub bubble_rating: Option<BubbleRating>, // Can be null or absent
    #[serde(rename = "priceForDisplay")]
    pub price_for_display: Option<String>, // Can be null
    pub provider: Option<String>, // Can be null, e.g. if priceForDisplay is null
    // Add other fields if you need them, e.g., badge, isSponsored, cardPhotos
}

#[derive(Debug, Deserialize)]
pub struct BubbleRating {
    pub count: String, // "1,037"
    pub rating: f32,   // 4.5
}

#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub status: bool, // Will be `false` for errors
    pub timestamp: u64,
    pub message: String,
}