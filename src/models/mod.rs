use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurrentWeather {
    pub temperature_c: f64,
    pub temperature_f: f64,
    pub feels_like_c: f64,
    pub feels_like_f: f64,
    pub humidity_percent: i32,
    pub wind_speed_kmh: f64,
    pub wind_direction_deg: f64,
    pub weather_code: i32,
    pub weather_description: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ForecastDay {
    pub date: String,
    pub temperature_max_c: f64,
    pub temperature_min_c: f64,
    pub precipitation_sum_mm: f64,
    pub precipitation_probability_percent: i32,
    pub wind_speed_max_kmh: f64,
    pub weather_code: i32,
    pub weather_description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RadarMetrics {
    pub precipitation_last_hour_mm: f64,
    pub precipitation_forecast_next_hour_mm: f64,
    pub rain_intensity_mm_h: f64,
    pub snowfall_cm_h: f64,
    pub showers_mm_h: f64,
    pub weather_code: i32,
    pub weather_description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AirQuality {
    pub aqi_us: i32,
    pub aqi_eu: i32,
    pub pm2_5_ug_m3: f64,
    pub pm10_ug_m3: f64,
    pub ozone_ug_m3: f64,
    pub nitrogen_dioxide_ug_m3: f64,
    pub carbon_monoxide_ug_m3: f64,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct CurrentResponse {
    pub location: Location,
    pub current: CurrentWeather,
}

#[derive(Debug, Serialize)]
pub struct ForecastResponse {
    pub location: Location,
    pub forecast: Vec<ForecastDay>,
}

#[derive(Debug, Serialize)]
pub struct RadarResponse {
    pub location: Location,
    pub radar: RadarMetrics,
}

#[derive(Debug, Serialize)]
pub struct AirQualityResponse {
    pub location: Location,
    pub air_quality: AirQuality,
}
