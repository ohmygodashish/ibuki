use ibuki::format::create_formatter;
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
            temperature_f: 72.5,
            feels_like_c: 23.1,
            feels_like_f: 73.6,
            humidity_percent: 65,
            wind_speed_kmh: 12.3,
            wind_direction_deg: 180.0,
            weather_code: 1,
            weather_description: "Mainly clear".into(),
            timestamp: "2026-06-08T14:30:00Z".into(),
        },
    }
}

#[test]
fn json_current_matches_schema_fields() {
    let formatter = create_formatter(true);
    let out = formatter.current(&sample_current(), false);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert_eq!(v["location"]["name"], "Tokyo");
    assert_eq!(v["current"]["temperature_c"], 22.5);
    assert_eq!(v["current"]["temperature_f"], 72.5);
    assert_eq!(v["current"]["humidity_percent"], 65);
    assert_eq!(v["current"]["weather_description"], "Mainly clear");
}

#[test]
fn human_current_shows_celsius_by_default() {
    let formatter = create_formatter(false);
    let out = formatter.current(&sample_current(), false);
    assert!(out.contains("Tokyo, Japan"));
    assert!(out.contains("22.5°C"));
    assert!(out.contains("Mainly clear"));
    assert!(out.contains("65%"));
}

#[test]
fn human_current_fahrenheit() {
    let formatter = create_formatter(false);
    let out = formatter.current(&sample_current(), true);
    assert!(out.contains("72.5°F"));
    assert!(!out.contains("22.5°C"));
}

#[test]
fn json_forecast_is_array() {
    let data = ForecastResponse {
        location: tokyo(),
        forecast: vec![ForecastDay {
            date: "2026-06-08".into(),
            temperature_max_c: 28.0,
            temperature_min_c: 20.0,
            precipitation_sum_mm: 2.5,
            precipitation_probability_percent: 40,
            wind_speed_max_kmh: 25.0,
            weather_code: 2,
            weather_description: "Partly cloudy".into(),
        }],
    };
    let formatter = create_formatter(true);
    let out = formatter.forecast(&data, false);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["forecast"].as_array().unwrap().len(), 1);
    assert_eq!(v["forecast"][0]["date"], "2026-06-08");
}

#[test]
fn json_radar_and_air_quality() {
    let radar = RadarResponse {
        location: tokyo(),
        radar: RadarMetrics {
            precipitation_last_hour_mm: 0.5,
            precipitation_forecast_next_hour_mm: 1.2,
            rain_intensity_mm_h: 0.3,
            snowfall_cm_h: 0.0,
            showers_mm_h: 0.2,
            weather_code: 61,
            weather_description: "Slight rain".into(),
        },
    };
    let aq = AirQualityResponse {
        location: tokyo(),
        air_quality: AirQuality {
            aqi_us: 45,
            aqi_eu: 38,
            pm2_5_ug_m3: 12.3,
            pm10_ug_m3: 25.1,
            ozone_ug_m3: 68.0,
            nitrogen_dioxide_ug_m3: 18.5,
            carbon_monoxide_ug_m3: 250.0,
            timestamp: "2026-06-08T14:30:00Z".into(),
        },
    };

    let formatter = create_formatter(true);
    let radar_json: serde_json::Value = serde_json::from_str(&formatter.radar(&radar)).unwrap();
    let aq_json: serde_json::Value = serde_json::from_str(&formatter.air_quality(&aq)).unwrap();

    assert_eq!(radar_json["radar"]["weather_code"], 61);
    assert_eq!(aq_json["air_quality"]["aqi_us"], 45);
}
