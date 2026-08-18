use serde::Deserialize;

use crate::error::{AppError, Result, check_status, map_reqwest_error};
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

    check_status(&response, "Weather")?;

    let body: ApiResponse = response.json().map_err(AppError::Parse)?;
    let current = body.current.ok_or(AppError::EmptyData)?;

    let weather_code = current.weather_code;

    // Next-hour forecast: second hourly precipitation value when available
    let next_hour = body
        .hourly
        .and_then(|h| h.precipitation)
        .and_then(|vals| vals.get(1).copied().flatten());

    Ok(RadarMetrics {
        precipitation_last_hour_mm: current
            .precipitation
            .or_else(|| sum_opt(current.rain, current.showers)),
        precipitation_forecast_next_hour_mm: next_hour,
        rain_intensity_mm_h: current.rain,
        snowfall_cm_h: current.snowfall,
        showers_mm_h: current.showers,
        weather_code,
        weather_description: weather_code.map_or("Unknown", weather_description).to_string(),
    })
}

/// `None` only when both parts are missing — one known component still beats no answer.
fn sum_opt(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    match (a, b) {
        (None, None) => None,
        _ => Some(a.unwrap_or(0.0) + b.unwrap_or(0.0)),
    }
}
