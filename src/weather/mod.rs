mod current;
mod forecast;
mod radar;

pub use current::fetch_current;
pub use forecast::fetch_forecast;
pub use radar::fetch_radar;

use crate::error::{AppError, Result};
use crate::models::Location;

const FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";

pub struct WeatherClient {
    http: reqwest::blocking::Client,
    base_url: String,
}

impl WeatherClient {
    pub fn new(http: reqwest::blocking::Client) -> Self {
        let base_url =
            std::env::var("IBUKI_FORECAST_URL").unwrap_or_else(|_| FORECAST_URL.to_string());
        Self::with_base_url(http, base_url)
    }

    pub fn with_base_url(http: reqwest::blocking::Client, base_url: impl Into<String>) -> Self {
        Self {
            http,
            base_url: base_url.into(),
        }
    }

    pub fn current(&self, location: &Location) -> Result<crate::models::CurrentWeather> {
        fetch_current(&self.http, &self.base_url, location)
    }

    pub fn forecast(
        &self,
        location: &Location,
        days: u8,
    ) -> Result<Vec<crate::models::ForecastDay>> {
        fetch_forecast(&self.http, &self.base_url, location, days)
    }

    pub fn radar(&self, location: &Location) -> Result<crate::models::RadarMetrics> {
        fetch_radar(&self.http, &self.base_url, location)
    }
}

pub(crate) fn map_reqwest_error(err: reqwest::Error) -> AppError {
    if err.is_timeout() {
        AppError::Timeout
    } else {
        AppError::Network(err)
    }
}

pub(crate) fn check_status(response: &reqwest::blocking::Response) -> Result<()> {
    if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(AppError::RateLimited);
    }
    if !response.status().is_success() {
        return Err(AppError::Api(format!(
            "Weather API returned status {}",
            response.status()
        )));
    }
    Ok(())
}
