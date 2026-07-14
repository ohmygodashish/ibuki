use serde::Serialize;

use super::Formatter;
use crate::models::{AirQualityResponse, CurrentResponse, ForecastResponse, RadarResponse};

pub struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn current(&self, data: &CurrentResponse, _fahrenheit: bool) -> String {
        to_pretty_json(data)
    }

    fn forecast(&self, data: &ForecastResponse, _fahrenheit: bool) -> String {
        to_pretty_json(data)
    }

    fn radar(&self, data: &RadarResponse) -> String {
        to_pretty_json(data)
    }

    fn air_quality(&self, data: &AirQualityResponse) -> String {
        to_pretty_json(data)
    }
}

fn to_pretty_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
}
