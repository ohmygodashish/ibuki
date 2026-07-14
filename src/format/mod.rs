mod human;
mod json;

pub use human::HumanFormatter;
pub use json::JsonFormatter;

use crate::models::{AirQualityResponse, CurrentResponse, ForecastResponse, RadarResponse};

pub trait Formatter {
    fn current(&self, data: &CurrentResponse, fahrenheit: bool) -> String;
    fn forecast(&self, data: &ForecastResponse, fahrenheit: bool) -> String;
    fn radar(&self, data: &RadarResponse) -> String;
    fn air_quality(&self, data: &AirQualityResponse) -> String;
}

pub fn create_formatter(json: bool) -> Box<dyn Formatter> {
    if json {
        Box::new(JsonFormatter)
    } else {
        Box::new(HumanFormatter::new())
    }
}
