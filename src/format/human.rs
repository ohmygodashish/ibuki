use std::io::IsTerminal;

use owo_colors::OwoColorize;

use super::Formatter;
use crate::models::{AirQualityResponse, CurrentResponse, ForecastResponse, RadarResponse};
use crate::units::{celsius_to_fahrenheit, wind_direction_label};

pub struct HumanFormatter {
    color: bool,
}

impl Default for HumanFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl HumanFormatter {
    pub fn new() -> Self {
        let no_color = std::env::var_os("NO_COLOR").is_some();
        let tty = std::io::stdout().is_terminal();
        Self {
            color: tty && !no_color,
        }
    }

    fn title(&self, text: &str) -> String {
        if self.color {
            text.bold().to_string()
        } else {
            text.to_string()
        }
    }

    fn label(&self, text: &str) -> String {
        if self.color {
            text.cyan().to_string()
        } else {
            text.to_string()
        }
    }

    fn box_lines(&self, lines: &[String]) -> String {
        let width = lines
            .iter()
            .map(|l| visible_len(l))
            .max()
            .unwrap_or(40)
            .max(40);
        let inner = width + 2;
        let mut out = String::new();
        out.push_str(&format!("┌{}┐\n", "─".repeat(inner)));
        for line in lines {
            let pad = width.saturating_sub(visible_len(line));
            out.push_str(&format!("│ {line}{} │\n", " ".repeat(pad)));
        }
        out.push_str(&format!("└{}┘", "─".repeat(inner)));
        out
    }
}

impl Formatter for HumanFormatter {
    fn current(&self, data: &CurrentResponse, fahrenheit: bool) -> String {
        let c = &data.current;
        let (temp, feels, unit) = if fahrenheit {
            (c.temperature_f, c.feels_like_f, "°F")
        } else {
            (c.temperature_c, c.feels_like_c, "°C")
        };
        let wind_dir = wind_direction_label(c.wind_direction_deg);
        let lines = vec![
            self.title(&format!(
                "  {}, {}",
                data.location.name, data.location.country
            )),
            self.title("  Current Weather"),
            format!("  {}", "─".repeat(37)),
            format!(
                "  {}:  {:.1}{} (feels {:.1}{})",
                self.label("Temperature"),
                temp,
                unit,
                feels,
                unit
            ),
            format!(
                "  {}:   {}",
                self.label("Conditions"),
                c.weather_description
            ),
            format!("  {}:     {}%", self.label("Humidity"), c.humidity_percent),
            format!(
                "  {}:         {:.1} km/h ({})",
                self.label("Wind"),
                c.wind_speed_kmh,
                wind_dir
            ),
            format!(
                "  {}:      {}",
                self.label("Updated"),
                format_timestamp(&c.timestamp)
            ),
        ];
        self.box_lines(&lines)
    }

    fn forecast(&self, data: &ForecastResponse, fahrenheit: bool) -> String {
        let mut lines = vec![
            self.title(&format!(
                "  {}, {}",
                data.location.name, data.location.country
            )),
            self.title(&format!("  {}-Day Forecast", data.forecast.len())),
            format!("  {}", "─".repeat(45)),
        ];

        for day in &data.forecast {
            let (max, min, unit) = if fahrenheit {
                (
                    celsius_to_fahrenheit(day.temperature_max_c),
                    celsius_to_fahrenheit(day.temperature_min_c),
                    "°F",
                )
            } else {
                (day.temperature_max_c, day.temperature_min_c, "°C")
            };
            lines.push(format!(
                "  {}  {:.0}{unit}/{:.0}{unit}  {}  {:.0}% rain  wind {:.0} km/h",
                day.date,
                max,
                min,
                day.weather_description,
                day.precipitation_probability_percent,
                day.wind_speed_max_kmh,
                unit = unit,
            ));
        }

        self.box_lines(&lines)
    }

    fn radar(&self, data: &RadarResponse) -> String {
        let r = &data.radar;
        let lines = vec![
            self.title(&format!(
                "  {}, {}",
                data.location.name, data.location.country
            )),
            self.title("  Radar / Precipitation"),
            format!("  {}", "─".repeat(37)),
            format!(
                "  {}:     {}",
                self.label("Conditions"),
                r.weather_description
            ),
            format!(
                "  {}: {:.1} mm",
                self.label("Precip (last hour)"),
                r.precipitation_last_hour_mm
            ),
            format!(
                "  {}: {:.1} mm",
                self.label("Precip (next hour)"),
                r.precipitation_forecast_next_hour_mm
            ),
            format!(
                "  {}:     {:.1} mm/h",
                self.label("Rain"),
                r.rain_intensity_mm_h
            ),
            format!("  {}:  {:.1} mm/h", self.label("Showers"), r.showers_mm_h),
            format!("  {}:  {:.1} cm/h", self.label("Snowfall"), r.snowfall_cm_h),
        ];
        self.box_lines(&lines)
    }

    fn air_quality(&self, data: &AirQualityResponse) -> String {
        let a = &data.air_quality;
        let lines = vec![
            self.title(&format!(
                "  {}, {}",
                data.location.name, data.location.country
            )),
            self.title("  Air Quality"),
            format!("  {}", "─".repeat(37)),
            format!("  {}:         {}", self.label("US AQI"), a.aqi_us),
            format!("  {}:         {}", self.label("EU AQI"), a.aqi_eu),
            format!(
                "  {}:        {:.1} µg/m³",
                self.label("PM2.5"),
                a.pm2_5_ug_m3
            ),
            format!(
                "  {}:         {:.1} µg/m³",
                self.label("PM10"),
                a.pm10_ug_m3
            ),
            format!(
                "  {}:        {:.1} µg/m³",
                self.label("Ozone"),
                a.ozone_ug_m3
            ),
            format!(
                "  {}:          {:.1} µg/m³",
                self.label("NO₂"),
                a.nitrogen_dioxide_ug_m3
            ),
            format!(
                "  {}:           {:.1} µg/m³",
                self.label("CO"),
                a.carbon_monoxide_ug_m3
            ),
            format!(
                "  {}:      {}",
                self.label("Updated"),
                format_timestamp(&a.timestamp)
            ),
        ];
        self.box_lines(&lines)
    }
}

fn format_timestamp(ts: &str) -> String {
    // Accept "2026-06-08T14:30" or "2026-06-08T14:30:00Z"
    let cleaned = ts.trim_end_matches('Z').replace('T', " ");
    if cleaned.is_empty() {
        "unknown".to_string()
    } else {
        format!("{cleaned} UTC")
    }
}

fn visible_len(s: &str) -> usize {
    // Strip common ANSI sequences for width calculation
    let mut len = 0;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for c2 in chars.by_ref() {
                    if c2.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            len += 1;
        }
    }
    len
}
