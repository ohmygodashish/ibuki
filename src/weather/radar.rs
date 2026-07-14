use serde::Deserialize;

use super::{check_status, map_reqwest_error};
use crate::error::{AppError, Result};
use crate::models::{Location, RadarMetrics};
use crate::units::weather_description;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    current: Option<ApiCurrent>,
    hourly: Option<ApiHourly>,
}

#[derive(Debug, Deserialize)]
struct ApiCurrent {
    rain: Option<f64>,
    snowfall: Option<f64>,
    showers: Option<f64>,
    weather_code: Option<i32>,
    precipitation: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct ApiHourly {
    precipitation: Option<Vec<Option<f64>>>,
}

pub fn fetch_radar(
    http: &reqwest::blocking::Client,
    base_url: &str,
    location: &Location,
) -> Result<RadarMetrics> {
    let response = http
        .get(base_url)
        .query(&[
            ("latitude", location.latitude.to_string()),
            ("longitude", location.longitude.to_string()),
            (
                "current",
                "precipitation,rain,showers,snowfall,weather_code".to_string(),
            ),
            ("hourly", "precipitation".to_string()),
            ("forecast_hours", "2".to_string()),
            ("timezone", "auto".to_string()),
        ])
        .send()
        .map_err(map_reqwest_error)?;

    check_status(&response)?;

    let body: ApiResponse = response.json().map_err(AppError::Parse)?;
    let current = body.current.ok_or(AppError::EmptyData)?;

    let rain = current.rain.unwrap_or(0.0);
    let snowfall = current.snowfall.unwrap_or(0.0);
    let showers = current.showers.unwrap_or(0.0);
    let precipitation = current.precipitation.unwrap_or(rain + showers);
    let weather_code = current.weather_code.unwrap_or(0);

    // Next-hour forecast: second hourly precipitation value when available
    let next_hour = body
        .hourly
        .and_then(|h| h.precipitation)
        .and_then(|vals| vals.get(1).copied().flatten())
        .unwrap_or(0.0);

    Ok(RadarMetrics {
        precipitation_last_hour_mm: precipitation,
        precipitation_forecast_next_hour_mm: next_hour,
        rain_intensity_mm_h: rain,
        snowfall_cm_h: snowfall,
        showers_mm_h: showers,
        weather_code,
        weather_description: weather_description(weather_code).to_string(),
    })
}
