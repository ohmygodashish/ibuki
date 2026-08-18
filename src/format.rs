use std::io::IsTerminal;

use owo_colors::OwoColorize;

use crate::models::{AirQualityResponse, CurrentResponse, ForecastResponse, RadarResponse};
use crate::units::{celsius_to_fahrenheit, wind_direction_label};

const NA: &str = "n/a";

pub fn current(data: &CurrentResponse, fahrenheit: bool) -> String {
    let c = &data.current;
    let (temp, feels, unit) = if fahrenheit {
        (
            celsius_to_fahrenheit(c.temperature_c),
            celsius_to_fahrenheit(c.feels_like_c),
            "°F",
        )
    } else {
        (c.temperature_c, c.feels_like_c, "°C")
    };
    let wind = match (c.wind_speed_kmh, c.wind_direction_deg) {
        (Some(speed), Some(deg)) => {
            format!("{speed:.1} km/h ({})", wind_direction_label(deg))
        }
        (Some(speed), None) => format!("{speed:.1} km/h"),
        _ => NA.to_string(),
    };

    render(
        &[header(&data.location), "Current Weather".to_string()],
        &[
            (
                "Temperature:",
                format!("{temp:.1}{unit} (feels {feels:.1}{unit})"),
            ),
            ("Conditions:", c.weather_description.clone()),
            ("Humidity:", int(c.humidity_percent, "%")),
            ("Wind:", wind),
            (
                "Updated:",
                timestamp(&c.timestamp, c.timezone.as_deref()),
            ),
        ],
    )
}

pub fn forecast(data: &ForecastResponse, fahrenheit: bool) -> String {
    let temp = |v: Option<f64>| match v {
        Some(c) if fahrenheit => format!("{:.0}°F", celsius_to_fahrenheit(c)),
        Some(c) => format!("{c:.0}°C"),
        None => NA.to_string(),
    };

    let rows: Vec<(&str, String)> = data
        .forecast
        .iter()
        .map(|day| {
            (
                day.date.as_str(),
                format!(
                    "{}/{}  {}  {} rain  wind {}",
                    temp(day.temperature_max_c),
                    temp(day.temperature_min_c),
                    day.weather_description,
                    int(day.precipitation_probability_percent, "%"),
                    dec(day.wind_speed_max_kmh, " km/h"),
                ),
            )
        })
        .collect();

    render(
        &[
            header(&data.location),
            format!("{}-Day Forecast", data.forecast.len()),
        ],
        &rows,
    )
}

pub fn radar(data: &RadarResponse) -> String {
    let r = &data.radar;
    render(
        &[header(&data.location), "Radar / Precipitation".to_string()],
        &[
            ("Conditions:", r.weather_description.clone()),
            (
                "Precip (last hour):",
                dec(r.precipitation_last_hour_mm, " mm"),
            ),
            (
                "Precip (next hour):",
                dec(r.precipitation_forecast_next_hour_mm, " mm"),
            ),
            ("Rain:", dec(r.rain_intensity_mm_h, " mm/h")),
            ("Showers:", dec(r.showers_mm_h, " mm/h")),
            ("Snowfall:", dec(r.snowfall_cm_h, " cm/h")),
        ],
    )
}

pub fn air_quality(data: &AirQualityResponse) -> String {
    let a = &data.air_quality;
    render(
        &[header(&data.location), "Air Quality".to_string()],
        &[
            ("US AQI:", int(a.aqi_us, "")),
            ("EU AQI:", int(a.aqi_eu, "")),
            ("PM2.5:", dec(a.pm2_5_ug_m3, " µg/m³")),
            ("PM10:", dec(a.pm10_ug_m3, " µg/m³")),
            ("Ozone:", dec(a.ozone_ug_m3, " µg/m³")),
            ("NO₂:", dec(a.nitrogen_dioxide_ug_m3, " µg/m³")),
            ("CO:", dec(a.carbon_monoxide_ug_m3, " µg/m³")),
            (
                "Updated:",
                timestamp(&a.timestamp, a.timezone.as_deref()),
            ),
        ],
    )
}

/// Draws the box. Every line is laid out as plain text first, so widths are
/// measured before any escape codes exist; colour is applied only at the end.
///
/// ponytail: widths count `char`s, so double-width CJK names still overhang the
/// right border. Add `unicode-width` if that matters.
fn render(titles: &[String], rows: &[(&str, String)]) -> String {
    let color = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    let label_w = rows.iter().map(|(l, _)| l.chars().count()).max().unwrap_or(0);

    let plain: Vec<String> = titles
        .iter()
        .cloned()
        .chain(rows.iter().map(|(l, v)| format!("{l:<label_w$}  {v}")))
        .collect();
    let width = plain
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(40)
        .max(40);

    let mut out = format!("┌{}┐\n", "─".repeat(width + 4));
    let mut push = |shown: String, plain_len: usize| {
        out.push_str(&format!(
            "│  {shown}{}  │\n",
            " ".repeat(width.saturating_sub(plain_len))
        ));
    };

    for (i, title) in titles.iter().enumerate() {
        let shown = if color {
            title.bold().to_string()
        } else {
            title.clone()
        };
        push(shown, plain[i].chars().count());
    }
    push("─".repeat(width), width);
    for (i, (label, value)) in rows.iter().enumerate() {
        let padded = format!("{label:<label_w$}");
        let shown = if color {
            format!("{}  {value}", padded.cyan())
        } else {
            format!("{padded}  {value}")
        };
        push(shown, plain[titles.len() + i].chars().count());
    }
    out.push_str(&format!("└{}┘", "─".repeat(width + 4)));
    out
}

fn header(location: &crate::models::Location) -> String {
    format!("{}, {}", location.name, location.country)
}

fn dec(value: Option<f64>, unit: &str) -> String {
    value.map_or_else(|| NA.to_string(), |v| format!("{v:.1}{unit}"))
}

fn int(value: Option<i32>, unit: &str) -> String {
    value.map_or_else(|| NA.to_string(), |v| format!("{v}{unit}"))
}

/// Open-Meteo is queried with `timezone=auto`, so timestamps are local to the
/// location — label them with the API's own abbreviation, never "UTC".
fn timestamp(ts: &str, timezone: Option<&str>) -> String {
    if ts.is_empty() {
        return "unknown".to_string();
    }
    let local = ts.replace('T', " ");
    match timezone {
        Some(tz) => format!("{local} {tz}"),
        None => local,
    }
}
