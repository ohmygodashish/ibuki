use serde::Deserialize;

use crate::error::{AppError, Result, check_status, map_reqwest_error};
use crate::models::Location;

const GEOCODING_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";

#[derive(Debug, Deserialize)]
struct GeocodingResponse {
    results: Option<Vec<GeocodingResult>>,
}

#[derive(Debug, Deserialize)]
struct GeocodingResult {
    name: String,
    admin1: Option<String>,
    country: Option<String>,
    latitude: f64,
    longitude: f64,
}

pub struct GeocodingClient {
    http: reqwest::blocking::Client,
    base_url: String,
}

impl GeocodingClient {
    pub fn new(http: reqwest::blocking::Client) -> Self {
        let base_url =
            std::env::var("IBUKI_GEOCODING_URL").unwrap_or_else(|_| GEOCODING_URL.to_string());
        Self::with_base_url(http, base_url)
    }

    pub fn with_base_url(http: reqwest::blocking::Client, base_url: impl Into<String>) -> Self {
        Self {
            http,
            base_url: base_url.into(),
        }
    }

    pub fn resolve(&self, city: &str) -> Result<Location> {
        if city.trim().is_empty() {
            return Err(AppError::CityNotFound(city.to_string()));
        }

        let response = self
            .http
            .get(&self.base_url)
            .query(&[
                ("name", city),
                ("count", "1"),
                ("language", "en"),
                ("format", "json"),
            ])
            .send()
            .map_err(map_reqwest_error)?;

        check_status(&response, "Geocoding")?;

        let body: GeocodingResponse = response.json().map_err(AppError::Parse)?;

        let result = body
            .results
            .and_then(|mut results| results.pop())
            .ok_or_else(|| AppError::CityNotFound(city.to_string()))?;

        Ok(Location {
            name: result.name,
            admin1: result.admin1,
            country: result.country.unwrap_or_else(|| "Unknown".to_string()),
            latitude: result.latitude,
            longitude: result.longitude,
        })
    }
}
