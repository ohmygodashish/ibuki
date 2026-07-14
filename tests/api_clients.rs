use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use ibuki::air_quality::AirQualityClient;
use ibuki::error::AppError;
use ibuki::geocoding::GeocodingClient;
use ibuki::models::Location;
use ibuki::weather::WeatherClient;
use mockito::Matcher;

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    fs::read_to_string(path).unwrap_or_else(|e| panic!("read fixture {name}: {e}"))
}

fn http() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap()
}

fn tokyo() -> Location {
    Location {
        name: "Tokyo".into(),
        country: "Japan".into(),
        latitude: 35.6895,
        longitude: 139.6917,
    }
}

fn mock_get(server: &mut mockito::Server, path: &str, body: &str) -> mockito::Mock {
    server
        .mock("GET", path)
        .match_query(Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create()
}

#[test]
fn geocoding_resolves_city() {
    let mut server = mockito::Server::new();
    let body = fixture("geocoding_tokyo.json");
    let mock = mock_get(&mut server, "/v1/search", &body);

    let client = GeocodingClient::with_base_url(http(), format!("{}/v1/search", server.url()));
    let location = client.resolve("Tokyo").unwrap();

    assert_eq!(location.name, "Tokyo");
    assert_eq!(location.country, "Japan");
    assert!((location.latitude - 35.6895).abs() < 0.001);
    mock.assert();
}

#[test]
fn geocoding_city_not_found() {
    let mut server = mockito::Server::new();
    let body = fixture("geocoding_empty.json");
    let _mock = mock_get(&mut server, "/v1/search", &body);

    let client = GeocodingClient::with_base_url(http(), format!("{}/v1/search", server.url()));
    let err = client.resolve("Xyz").unwrap_err();
    assert!(
        matches!(err, AppError::CityNotFound(_)),
        "unexpected error: {err}"
    );
    assert!(err.to_string().contains("Xyz"));
}

#[test]
fn geocoding_empty_city() {
    let client = GeocodingClient::with_base_url(http(), "http://127.0.0.1:9");
    let err = client.resolve("   ").unwrap_err();
    assert!(matches!(err, AppError::CityNotFound(_)));
}

#[test]
fn geocoding_rate_limited() {
    let mut server = mockito::Server::new();
    let _mock = server
        .mock("GET", "/v1/search")
        .match_query(Matcher::Any)
        .with_status(429)
        .with_body("rate limited")
        .create();

    let client = GeocodingClient::with_base_url(http(), format!("{}/v1/search", server.url()));
    let err = client.resolve("Tokyo").unwrap_err();
    assert!(
        matches!(err, AppError::RateLimited),
        "unexpected error: {err}"
    );
}

#[test]
fn weather_current_parses_fixture() {
    let mut server = mockito::Server::new();
    let body = fixture("current_weather.json");
    let _mock = mock_get(&mut server, "/v1/forecast", &body);

    let client = WeatherClient::with_base_url(http(), format!("{}/v1/forecast", server.url()));
    let current = client.current(&tokyo()).unwrap();

    assert!((current.temperature_c - 22.5).abs() < f64::EPSILON);
    assert!((current.temperature_f - 72.5).abs() < f64::EPSILON);
    assert_eq!(current.humidity_percent, 65);
    assert_eq!(current.weather_code, 1);
    assert_eq!(current.weather_description, "Mainly clear");
}

#[test]
fn weather_forecast_parses_fixture() {
    let mut server = mockito::Server::new();
    let body = fixture("forecast.json");
    let _mock = mock_get(&mut server, "/v1/forecast", &body);

    let client = WeatherClient::with_base_url(http(), format!("{}/v1/forecast", server.url()));
    let days = client.forecast(&tokyo(), 3).unwrap();

    assert_eq!(days.len(), 3);
    assert_eq!(days[0].date, "2026-06-08");
    assert!((days[0].temperature_max_c - 28.0).abs() < f64::EPSILON);
    assert_eq!(days[0].precipitation_probability_percent, 40);
    assert_eq!(days[2].weather_description, "Slight rain");
}

#[test]
fn weather_radar_parses_fixture() {
    let mut server = mockito::Server::new();
    let body = fixture("radar.json");
    let _mock = mock_get(&mut server, "/v1/forecast", &body);

    let client = WeatherClient::with_base_url(http(), format!("{}/v1/forecast", server.url()));
    let radar = client.radar(&tokyo()).unwrap();

    assert!((radar.precipitation_last_hour_mm - 0.5).abs() < f64::EPSILON);
    assert!((radar.precipitation_forecast_next_hour_mm - 1.2).abs() < f64::EPSILON);
    assert!((radar.rain_intensity_mm_h - 0.3).abs() < f64::EPSILON);
    assert_eq!(radar.weather_description, "Slight rain");
}

#[test]
fn weather_empty_data() {
    let mut server = mockito::Server::new();
    let _mock = mock_get(&mut server, "/v1/forecast", "{}");

    let client = WeatherClient::with_base_url(http(), format!("{}/v1/forecast", server.url()));
    let err = client.current(&tokyo()).unwrap_err();
    assert!(
        matches!(err, AppError::EmptyData),
        "unexpected error: {err}"
    );
}

#[test]
fn air_quality_parses_fixture() {
    let mut server = mockito::Server::new();
    let body = fixture("air_quality.json");
    let _mock = mock_get(&mut server, "/v1/air-quality", &body);

    let client =
        AirQualityClient::with_base_url(http(), format!("{}/v1/air-quality", server.url()));
    let aq = client.current(&tokyo()).unwrap();

    assert_eq!(aq.aqi_us, 45);
    assert_eq!(aq.aqi_eu, 38);
    assert!((aq.pm2_5_ug_m3 - 12.3).abs() < f64::EPSILON);
    assert!((aq.carbon_monoxide_ug_m3 - 250.0).abs() < f64::EPSILON);
}
