use chrono::NaiveDate;
use chrono::Datelike;
use crate::hotel_search_tool::HotelSearchError;

/// Parses a date string and infers the year.
/// If the year is missing, it assumes the *next upcoming* occurrence of that month/day.
pub fn parse_and_infer_year(date_str: &str, today: NaiveDate) -> Result<NaiveDate, HotelSearchError> {
    // Try parsing with year first (YYYY-MM-DD)
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Ok(date);
    }
    // Try parsing without year (MM-DD)
    if let Ok(mut date) = NaiveDate::parse_from_str(date_str, "%m-%d") {
        // Assume current year initially
        date = date.with_year(today.year()).unwrap();

        // If the parsed date (in current year) has already passed, use next year
        if date < today {
            date = date.with_year(today.year() + 1).unwrap();
        }
        return Ok(date);
    }
    // Try parsing common month-day formats (e.g., "August 8th")
    if let Ok(mut date) = NaiveDate::parse_from_str(date_str, "%B %e") {
        date = date.with_year(today.year()).unwrap();
        if date < today {
            date = date.with_year(today.year() + 1).unwrap();
        }
        return Ok(date);
    }
    // Add more `parse_from_str` patterns if you expect other formats like "8th August"
    if let Ok(mut date) = NaiveDate::parse_from_str(date_str, "%e %B") {
        date = date.with_year(today.year()).unwrap();
        if date < today {
            date = date.with_year(today.year() + 1).unwrap();
        }
        return Ok(date);
    }
    // If all attempts fail
    Err(HotelSearchError::InvalidResponse(format!("Could not parse date: {}", date_str)))
}