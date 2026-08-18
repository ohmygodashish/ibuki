use serde::Deserialize;

use crate::error::{AppError, Result, check_status, map_reqwest_error};
use crate::models::{AirQuality, Location};

const AIR_QUALITY_URL: &str = "https://air-quality-api.open-meteo.com/v1/air-quality";

#[derive(Debug, Deserialize)]
struct ApiResponse {
    current: Option<ApiCurrent>,
    timezone_abbreviation: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiCurrent {
    time: Option<String>,
    us_aqi: Option<i32>,
    european_aqi: Option<i32>,
    pm2_5: Option<f64>,
    pm10: Option<f64>,
    ozone: Option<f64>,
    nitrogen_dioxide: Option<f64>,
    carbon_monoxide: Option<f64>,
}

pub struct AirQualityClient {
    http: reqwest::blocking::Client,
    base_url: String,
}

impl AirQualityClient {
    pub fn new(http: reqwest::blocking::Client) -> Self {
        let base_url =
            std::env::var("IBUKI_AIR_QUALITY_URL").unwrap_or_else(|_| AIR_QUALITY_URL.to_string());
        Self::with_base_url(http, base_url)
    }

    pub fn with_base_url(http: reqwest::blocking::Client, base_url: impl Into<String>) -> Self {
        Self {
            http,
            base_url: base_url.into(),
        }
    }

    pub fn current(&self, location: &Location) -> Result<AirQuality> {
        let response = self
            .http
            .get(&self.base_url)
            .query(&[
                ("latitude", location.latitude.to_string()),
                ("longitude", location.longitude.to_string()),
                (
                    "current",
                    "us_aqi,european_aqi,pm2_5,pm10,ozone,nitrogen_dioxide,carbon_monoxide"
                        .to_string(),
                ),
                ("timezone", "auto".to_string()),
            ])
            .send()
            .map_err(map_reqwest_error)?;

        check_status(&response, "Air quality")?;

        let body: ApiResponse = response.json().map_err(AppError::Parse)?;
        let current = body.current.ok_or(AppError::EmptyData)?;

        Ok(AirQuality {
            aqi_us: current.us_aqi,
            aqi_eu: current.european_aqi,
            pm2_5_ug_m3: current.pm2_5,
            pm10_ug_m3: current.pm10,
            ozone_ug_m3: current.ozone,
            nitrogen_dioxide_ug_m3: current.nitrogen_dioxide,
            carbon_monoxide_ug_m3: current.carbon_monoxide,
            // `timezone=auto` means this is local time at the location, not UTC.
            timestamp: current.time.unwrap_or_default(),
            timezone: body.timezone_abbreviation,
        })
    }
}
