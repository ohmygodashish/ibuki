use ibuki::format;
use ibuki::models::{
    AirQuality, AirQualityResponse, CurrentResponse, CurrentWeather, ForecastDay, ForecastResponse,
    Location, RadarMetrics, RadarResponse,
};

fn tokyo() -> Location {
    Location {
        name: "Tokyo".into(),
        country: "Japan".into(),
        latitude: 35.6895,
        longitude: 139.6917,
    }
}

fn sample_current() -> CurrentResponse {
    CurrentResponse {
        location: tokyo(),
        current: CurrentWeather {
            temperature_c: 22.5,
            feels_like_c: 23.1,
            humidity_percent: Some(65),
            wind_speed_kmh: Some(12.3),
            wind_direction_deg: Some(180.0),
            weather_code: Some(1),
            weather_description: "Mainly clear".into(),
            timestamp: "2026-06-08T14:30".into(),
            timezone: Some("JST".into()),
        },
    }
}

fn sample_air_quality(present: bool) -> AirQualityResponse {
    let some = |v| if present { Some(v) } else { None };
    AirQualityResponse {
        location: tokyo(),
        air_quality: AirQuality {
            aqi_us: if present { Some(45) } else { None },
            aqi_eu: if present { Some(38) } else { None },
            pm2_5_ug_m3: some(12.3),
            pm10_ug_m3: some(25.1),
            ozone_ug_m3: some(68.0),
            nitrogen_dioxide_ug_m3: some(18.5),
            carbon_monoxide_ug_m3: some(250.0),
            timestamp: "2026-06-08T14:30".into(),
            timezone: Some("JST".into()),
        },
    }
}

fn json_of<T: serde::Serialize>(value: &T) -> serde_json::Value {
    serde_json::from_str(&serde_json::to_string_pretty(value).unwrap()).unwrap()
}

#[test]
fn json_current_matches_schema_fields() {
    let v = json_of(&sample_current());

    assert_eq!(v["location"]["name"], "Tokyo");
    assert_eq!(v["current"]["temperature_c"], 22.5);
    assert_eq!(v["current"]["humidity_percent"], 65);
    assert_eq!(v["current"]["weather_description"], "Mainly clear");
    // JSON is metric-only: no Fahrenheit twin fields to drift out of sync.
    assert!(v["current"].get("temperature_f").is_none());
}

#[test]
fn human_current_shows_celsius_by_default() {
    let out = format::current(&sample_current(), false);
    assert!(out.contains("Tokyo, Japan"));
    assert!(out.contains("22.5°C"));
    assert!(out.contains("Mainly clear"));
    assert!(out.contains("65%"));
}

#[test]
fn human_current_fahrenheit() {
    let out = format::current(&sample_current(), true);
    assert!(out.contains("72.5°F"));
    assert!(!out.contains("22.5°C"));
}

#[test]
fn timestamp_uses_api_timezone_not_utc() {
    let out = format::current(&sample_current(), false);
    assert!(
        out.contains("2026-06-08 14:30 JST"),
        "expected local tz label, got:\n{out}"
    );
    assert!(!out.contains("UTC"), "local time must not be called UTC");
}

#[test]
fn timestamp_without_timezone_is_unlabelled() {
    let mut data = sample_current();
    data.current.timezone = None;
    let out = format::current(&data, false);
    assert!(out.contains("2026-06-08 14:30"));
    assert!(!out.contains("UTC"));
}

#[test]
fn missing_values_render_as_na_not_zero() {
    let mut data = sample_current();
    data.current.humidity_percent = None;
    data.current.wind_speed_kmh = None;
    data.current.wind_direction_deg = None;
    let out = format::current(&data, false);

    assert!(out.contains("n/a"), "expected n/a, got:\n{out}");
    assert!(!out.contains("0%"), "missing humidity must not read as 0%");
}

#[test]
fn missing_air_quality_is_na_in_human_and_null_in_json() {
    let data = sample_air_quality(false);
    let out = format::air_quality(&data);
    assert!(!out.contains("AQI:         0"), "missing AQI must not be 0");
    assert_eq!(out.matches("n/a").count(), 7);

    let v = json_of(&data);
    assert!(v["air_quality"]["aqi_us"].is_null());
    assert!(v["air_quality"]["pm2_5_ug_m3"].is_null());
}

#[test]
fn box_borders_align_with_content() {
    let out = format::air_quality(&sample_air_quality(true));
    let widths: Vec<usize> = out.lines().map(|l| l.chars().count()).collect();
    assert!(
        widths.windows(2).all(|w| w[0] == w[1]),
        "ragged box:\n{out}"
    );
}

#[test]
fn json_forecast_is_array() {
    let data = ForecastResponse {
        location: tokyo(),
        forecast: vec![ForecastDay {
            date: "2026-06-08".into(),
            temperature_max_c: Some(28.0),
            temperature_min_c: Some(20.0),
            precipitation_sum_mm: Some(2.5),
            precipitation_probability_percent: Some(40),
            wind_speed_max_kmh: Some(25.0),
            weather_code: Some(2),
            weather_description: "Partly cloudy".into(),
        }],
    };
    let v = json_of(&data);
    assert_eq!(v["forecast"].as_array().unwrap().len(), 1);
    assert_eq!(v["forecast"][0]["date"], "2026-06-08");

    let out = format::forecast(&data, false);
    assert!(out.contains("28°C/20°C"), "got:\n{out}");
}

#[test]
fn json_radar_and_air_quality() {
    let radar = RadarResponse {
        location: tokyo(),
        radar: RadarMetrics {
            precipitation_last_hour_mm: Some(0.5),
            precipitation_forecast_next_hour_mm: Some(1.2),
            rain_intensity_mm_h: Some(0.3),
            snowfall_cm_h: Some(0.0),
            showers_mm_h: Some(0.2),
            weather_code: Some(61),
            weather_description: "Slight rain".into(),
        },
    };

    assert_eq!(json_of(&radar)["radar"]["weather_code"], 61);
    assert_eq!(
        json_of(&sample_air_quality(true))["air_quality"]["aqi_us"],
        45
    );
}
