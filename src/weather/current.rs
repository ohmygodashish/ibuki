use serde::Deserialize;

use crate::error::{AppError, Result, check_status, map_reqwest_error};
use crate::models::{CurrentWeather, Location};
use crate::units::weather_description;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    current: Option<ApiCurrent>,
    timezone_abbreviation: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiCurrent {
    time: Option<String>,
    temperature_2m: Option<f64>,
    apparent_temperature: Option<f64>,
    relative_humidity_2m: Option<i32>,
    wind_speed_10m: Option<f64>,
    wind_direction_10m: Option<f64>,
    weather_code: Option<i32>,
}

pub fn fetch_current(
    http: &reqwest::blocking::Client,
    base_url: &str,
    location: &Location,
) -> Result<CurrentWeather> {
    let response = http
        .get(base_url)
        .query(&[
            ("latitude", location.latitude.to_string()),
            ("longitude", location.longitude.to_string()),
            (
                "current",
                "temperature_2m,apparent_temperature,relative_humidity_2m,wind_speed_10m,wind_direction_10m,weather_code".to_string(),
            ),
            ("timezone", "auto".to_string()),
            ("wind_speed_unit", "kmh".to_string()),
        ])
        .send()
        .map_err(map_reqwest_error)?;

    check_status(&response, "Weather")?;

    let body: ApiResponse = response.json().map_err(AppError::Parse)?;
    let current = body.current.ok_or(AppError::EmptyData)?;

    let temperature_c = current.temperature_2m.ok_or(AppError::EmptyData)?;
    let weather_code = current.weather_code;

    Ok(CurrentWeather {
        temperature_c,
        feels_like_c: current.apparent_temperature.unwrap_or(temperature_c),
        humidity_percent: current.relative_humidity_2m,
        wind_speed_kmh: current.wind_speed_10m,
        wind_direction_deg: current.wind_direction_10m,
        weather_code,
        weather_description: weather_code
            .map_or("Unknown", weather_description)
            .to_string(),
        // `timezone=auto` means this is local time at the location, not UTC.
        timestamp: current.time.unwrap_or_default(),
        timezone: body.timezone_abbreviation,
    })
}
