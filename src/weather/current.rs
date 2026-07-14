use serde::Deserialize;

use super::{check_status, map_reqwest_error};
use crate::error::{AppError, Result};
use crate::models::{CurrentWeather, Location};
use crate::units::{celsius_to_fahrenheit, weather_description};

#[derive(Debug, Deserialize)]
struct ApiResponse {
    current: Option<ApiCurrent>,
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

    check_status(&response)?;

    let body: ApiResponse = response.json().map_err(AppError::Parse)?;
    let current = body.current.ok_or(AppError::EmptyData)?;

    let temperature_c = current.temperature_2m.ok_or(AppError::EmptyData)?;
    let feels_like_c = current.apparent_temperature.unwrap_or(temperature_c);
    let weather_code = current.weather_code.unwrap_or(0);
    let timestamp = current
        .time
        .map(|t| {
            if t.ends_with('Z') {
                t
            } else {
                format!("{t}:00Z")
            }
        })
        .unwrap_or_default();

    Ok(CurrentWeather {
        temperature_c,
        temperature_f: celsius_to_fahrenheit(temperature_c),
        feels_like_c,
        feels_like_f: celsius_to_fahrenheit(feels_like_c),
        humidity_percent: current.relative_humidity_2m.unwrap_or(0),
        wind_speed_kmh: current.wind_speed_10m.unwrap_or(0.0),
        wind_direction_deg: current.wind_direction_10m.unwrap_or(0.0),
        weather_code,
        weather_description: weather_description(weather_code).to_string(),
        timestamp,
    })
}
