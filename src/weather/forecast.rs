use serde::Deserialize;

use crate::error::{AppError, Result, check_status, map_reqwest_error};
use crate::models::{ForecastDay, Location};
use crate::units::weather_description;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    daily: Option<ApiDaily>,
}

#[derive(Debug, Deserialize)]
struct ApiDaily {
    time: Vec<String>,
    temperature_2m_max: Option<Vec<Option<f64>>>,
    temperature_2m_min: Option<Vec<Option<f64>>>,
    precipitation_sum: Option<Vec<Option<f64>>>,
    precipitation_probability_max: Option<Vec<Option<i32>>>,
    wind_speed_10m_max: Option<Vec<Option<f64>>>,
    weather_code: Option<Vec<Option<i32>>>,
}

pub fn fetch_forecast(
    http: &reqwest::blocking::Client,
    base_url: &str,
    location: &Location,
    days: u8,
) -> Result<Vec<ForecastDay>> {
    let response = http
        .get(base_url)
        .query(&[
            ("latitude", location.latitude.to_string()),
            ("longitude", location.longitude.to_string()),
            (
                "daily",
                "temperature_2m_max,temperature_2m_min,precipitation_sum,precipitation_probability_max,wind_speed_10m_max,weather_code".to_string(),
            ),
            ("timezone", "auto".to_string()),
            ("forecast_days", days.to_string()),
            ("wind_speed_unit", "kmh".to_string()),
        ])
        .send()
        .map_err(map_reqwest_error)?;

    check_status(&response, "Weather")?;

    let body: ApiResponse = response.json().map_err(AppError::Parse)?;
    let daily = body.daily.ok_or(AppError::EmptyData)?;

    if daily.time.is_empty() {
        return Err(AppError::EmptyData);
    }

    let max_temps = daily.temperature_2m_max.unwrap_or_default();
    let min_temps = daily.temperature_2m_min.unwrap_or_default();
    let precip = daily.precipitation_sum.unwrap_or_default();
    let precip_prob = daily.precipitation_probability_max.unwrap_or_default();
    let wind = daily.wind_speed_10m_max.unwrap_or_default();
    let codes = daily.weather_code.unwrap_or_default();

    let at = |col: &[Option<f64>], i: usize| col.get(i).copied().flatten();

    let forecast = daily
        .time
        .into_iter()
        .enumerate()
        .map(|(i, date)| {
            let weather_code = codes.get(i).copied().flatten();
            ForecastDay {
                date,
                temperature_max_c: at(&max_temps, i),
                temperature_min_c: at(&min_temps, i),
                precipitation_sum_mm: at(&precip, i),
                precipitation_probability_percent: precip_prob.get(i).copied().flatten(),
                wind_speed_max_kmh: at(&wind, i),
                weather_code,
                weather_description: weather_code
                    .map_or("Unknown", weather_description)
                    .to_string(),
            }
        })
        .collect();

    Ok(forecast)
}
