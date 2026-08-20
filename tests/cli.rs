use std::fs;
use std::path::PathBuf;
use std::process::Command;

use assert_cmd::prelude::*;
use mockito::Matcher;
use predicates::prelude::*;

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    fs::read_to_string(path).unwrap()
}

fn mock_json(server: &mut mockito::Server, path: &str, body: &str) -> mockito::Mock {
    server
        .mock("GET", path)
        .match_query(Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create()
}

#[test]
fn help_shows_subcommands() {
    let mut cmd = Command::cargo_bin("ibuki").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("current"))
        .stdout(predicate::str::contains("forecast"))
        .stdout(predicate::str::contains("radar"))
        .stdout(predicate::str::contains("air-quality"));
}

#[test]
fn version_flag() {
    let mut cmd = Command::cargo_bin("ibuki").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("ibuki"));
}

#[test]
fn forecast_days_out_of_range() {
    let mut cmd = Command::cargo_bin("ibuki").unwrap();
    cmd.args(["forecast", "Tokyo", "--days", "20"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("1..=16"));
}

#[test]
fn forecast_days_zero_rejected() {
    let mut cmd = Command::cargo_bin("ibuki").unwrap();
    cmd.args(["forecast", "Tokyo", "--days", "0"])
        .assert()
        .failure();
}

#[test]
fn current_with_mocked_apis_json() {
    let mut geo = mockito::Server::new();
    let mut weather = mockito::Server::new();

    let _geo_mock = mock_json(&mut geo, "/v1/search", &fixture("geocoding_tokyo.json"));
    let _weather_mock = mock_json(
        &mut weather,
        "/v1/forecast",
        &fixture("current_weather.json"),
    );

    let mut cmd = Command::cargo_bin("ibuki").unwrap();
    cmd.env("IBUKI_GEOCODING_URL", format!("{}/v1/search", geo.url()))
        .env(
            "IBUKI_FORECAST_URL",
            format!("{}/v1/forecast", weather.url()),
        )
        .env("NO_COLOR", "1")
        .args(["current", "Tokyo", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"Tokyo\""))
        .stdout(predicate::str::contains("\"temperature_c\": 22.5"))
        .stdout(predicate::str::contains("Mainly clear"));
}

#[test]
fn current_city_not_found() {
    let mut geo = mockito::Server::new();
    let _geo_mock = mock_json(&mut geo, "/v1/search", &fixture("geocoding_empty.json"));

    let mut cmd = Command::cargo_bin("ibuki").unwrap();
    cmd.env("IBUKI_GEOCODING_URL", format!("{}/v1/search", geo.url()))
        .args(["current", "XyzNotACity"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn forecast_mocked_human() {
    let mut geo = mockito::Server::new();
    let mut weather = mockito::Server::new();

    let _geo_mock = mock_json(&mut geo, "/v1/search", &fixture("geocoding_tokyo.json"));
    let _weather_mock = mock_json(&mut weather, "/v1/forecast", &fixture("forecast.json"));

    let mut cmd = Command::cargo_bin("ibuki").unwrap();
    cmd.env("IBUKI_GEOCODING_URL", format!("{}/v1/search", geo.url()))
        .env(
            "IBUKI_FORECAST_URL",
            format!("{}/v1/forecast", weather.url()),
        )
        .env("NO_COLOR", "1")
        .args(["forecast", "Tokyo", "--days", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Tokyo, Japan"))
        .stdout(predicate::str::contains("2026-06-08"));
}

#[test]
fn air_quality_mocked_json() {
    let mut geo = mockito::Server::new();
    let mut aq = mockito::Server::new();

    let _geo_mock = mock_json(&mut geo, "/v1/search", &fixture("geocoding_tokyo.json"));
    let _aq_mock = mock_json(&mut aq, "/v1/air-quality", &fixture("air_quality.json"));

    let mut cmd = Command::cargo_bin("ibuki").unwrap();
    cmd.env("IBUKI_GEOCODING_URL", format!("{}/v1/search", geo.url()))
        .env(
            "IBUKI_AIR_QUALITY_URL",
            format!("{}/v1/air-quality", aq.url()),
        )
        .args(["air-quality", "Tokyo", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"aqi_us\": 45"))
        .stdout(predicate::str::contains("\"pm2_5_ug_m3\": 12.3"));
}

#[test]
fn error_output_includes_underlying_cause() {
    let mut geo = mockito::Server::new();
    let _geo_mock = mock_json(&mut geo, "/v1/search", "{ not json");

    let mut cmd = Command::cargo_bin("ibuki").unwrap();
    cmd.env("IBUKI_GEOCODING_URL", format!("{}/v1/search", geo.url()))
        .args(["current", "Tokyo"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Failed to parse API response"))
        .stderr(predicate::str::contains("error decoding response body"));
}

#[test]
fn coordinates_skip_geocoding() {
    let mut weather = mockito::Server::new();
    let _weather_mock = mock_json(
        &mut weather,
        "/v1/forecast",
        &fixture("current_weather.json"),
    );

    let mut cmd = Command::cargo_bin("ibuki").unwrap();
    // Geocoding points at a dead port: success proves it was never called.
    cmd.env("IBUKI_GEOCODING_URL", "http://127.0.0.1:9/v1/search")
        .env(
            "IBUKI_FORECAST_URL",
            format!("{}/v1/forecast", weather.url()),
        )
        .env("NO_COLOR", "1")
        .args(["current", "--lat", "35.6895", "--lon", "139.6917"])
        .assert()
        .success()
        .stdout(predicate::str::contains("35.6895, 139.6917"))
        .stdout(predicate::str::contains("Mainly clear"));
}

#[test]
fn lat_without_lon_is_rejected() {
    let mut cmd = Command::cargo_bin("ibuki").unwrap();
    cmd.args(["current", "--lat", "35.6895"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--lon"));
}

#[test]
fn out_of_range_latitude_is_rejected() {
    let mut cmd = Command::cargo_bin("ibuki").unwrap();
    cmd.args(["current", "--lat", "91.0", "--lon", "0.0"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("-90"));
}

#[test]
fn ibuki_city_supplies_the_default_city() {
    let mut geo = mockito::Server::new();
    let mut weather = mockito::Server::new();
    let _geo_mock = mock_json(&mut geo, "/v1/search", &fixture("geocoding_tokyo.json"));
    let _weather_mock = mock_json(
        &mut weather,
        "/v1/forecast",
        &fixture("current_weather.json"),
    );

    let mut cmd = Command::cargo_bin("ibuki").unwrap();
    cmd.env("IBUKI_GEOCODING_URL", format!("{}/v1/search", geo.url()))
        .env(
            "IBUKI_FORECAST_URL",
            format!("{}/v1/forecast", weather.url()),
        )
        .env("IBUKI_CITY", "Tokyo")
        .env("NO_COLOR", "1")
        .arg("current")
        .assert()
        .success()
        .stdout(predicate::str::contains("Tokyo, Japan"));
}

#[test]
fn no_city_and_no_coordinates_is_a_clear_error() {
    let mut cmd = Command::cargo_bin("ibuki").unwrap();
    cmd.env_remove("IBUKI_CITY")
        .arg("current")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--lat"));
}
