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
        admin1: Some("Tokyo".into()),
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
    assert_eq!(location.admin1.as_deref(), Some("Tokyo"));
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
    assert_eq!(current.humidity_percent, Some(65));
    assert_eq!(current.weather_code, Some(1));
    assert_eq!(current.weather_description, "Mainly clear");
    // timezone=auto: the timestamp is local, so carry the API's own label.
    assert_eq!(current.timezone.as_deref(), Some("JST"));
    assert_eq!(current.timestamp, "2026-06-08T14:30");
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
    assert_eq!(days[0].temperature_max_c, Some(28.0));
    assert_eq!(days[0].precipitation_probability_percent, Some(40));
    assert_eq!(days[2].weather_description, "Slight rain");
}

#[test]
fn weather_radar_parses_fixture() {
    let mut server = mockito::Server::new();
    let body = fixture("radar.json");
    let _mock = mock_get(&mut server, "/v1/forecast", &body);

    let client = WeatherClient::with_base_url(http(), format!("{}/v1/forecast", server.url()));
    let radar = client.radar(&tokyo()).unwrap();

    assert_eq!(radar.precipitation_last_hour_mm, Some(0.5));
    assert_eq!(radar.precipitation_forecast_next_hour_mm, Some(1.2));
    assert_eq!(radar.rain_intensity_mm_h, Some(0.3));
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

    assert_eq!(aq.aqi_us, Some(45));
    assert_eq!(aq.aqi_eu, Some(38));
    assert_eq!(aq.pm2_5_ug_m3, Some(12.3));
    assert_eq!(aq.carbon_monoxide_ug_m3, Some(250.0));
    assert_eq!(aq.timezone.as_deref(), Some("JST"));
}

#[test]
fn air_quality_missing_pollutants_stay_none() {
    let mut server = mockito::Server::new();
    let _mock = mock_get(
        &mut server,
        "/v1/air-quality",
        r#"{"current": {"time": "2026-06-08T14:30", "us_aqi": 45}}"#,
    );

    let client =
        AirQualityClient::with_base_url(http(), format!("{}/v1/air-quality", server.url()));
    let aq = client.current(&tokyo()).unwrap();

    assert_eq!(aq.aqi_us, Some(45));
    // Absent readings must not be reported as clean air.
    assert_eq!(aq.pm2_5_ug_m3, None);
    assert_eq!(aq.aqi_eu, None);
}

fn cache_file(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_file(&path);
    path
}

/// A second client aimed at a dead port can only succeed from the cache.
fn offline_client(cache: &PathBuf) -> GeocodingClient {
    GeocodingClient::with_base_url(http(), "http://127.0.0.1:9/v1/search").with_cache(cache)
}

#[test]
fn geocoding_second_lookup_is_served_from_cache() {
    let mut server = mockito::Server::new();
    let mock = mock_get(&mut server, "/v1/search", &fixture("geocoding_tokyo.json")).expect(1);
    let cache = cache_file("cache_hit.json");

    let online = GeocodingClient::with_base_url(http(), format!("{}/v1/search", server.url()))
        .with_cache(&cache);
    let first = online.resolve("Tokyo").unwrap();
    let second = offline_client(&cache).resolve("Tokyo").unwrap();

    assert_eq!(second.name, first.name);
    assert_eq!(second.admin1, first.admin1);
    assert!((second.latitude - first.latitude).abs() < 1e-9);
    mock.assert(); // exactly one network call for two lookups
}

#[test]
fn cache_lookup_ignores_case_and_surrounding_space() {
    let mut server = mockito::Server::new();
    let _mock = mock_get(&mut server, "/v1/search", &fixture("geocoding_tokyo.json"));
    let cache = cache_file("cache_normalized.json");

    GeocodingClient::with_base_url(http(), format!("{}/v1/search", server.url()))
        .with_cache(&cache)
        .resolve("Tokyo")
        .unwrap();

    let hit = offline_client(&cache).resolve("  tOkYo  ").unwrap();
    assert_eq!(hit.name, "Tokyo");
}

#[test]
fn corrupt_cache_falls_back_to_the_network() {
    let mut server = mockito::Server::new();
    let _mock = mock_get(&mut server, "/v1/search", &fixture("geocoding_tokyo.json"));
    let cache = cache_file("cache_corrupt.json");
    fs::write(&cache, "{ this is not json").unwrap();

    let client = GeocodingClient::with_base_url(http(), format!("{}/v1/search", server.url()))
        .with_cache(&cache);
    assert_eq!(client.resolve("Tokyo").unwrap().name, "Tokyo");
}
