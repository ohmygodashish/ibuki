use std::process::ExitCode;
use std::time::Duration;

use anyhow::Context;
use clap::Parser;

use ibuki::air_quality::AirQualityClient;
use ibuki::cli::{Cli, Command};
use ibuki::format;
use ibuki::geocoding::GeocodingClient;
use ibuki::models::{
    AirQualityResponse, CurrentResponse, ForecastResponse, Location, RadarResponse,
};
use ibuki::weather::WeatherClient;

fn main() -> ExitCode {
    if let Err(err) = run() {
        eprintln!("{err:#}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let http = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(concat!("ibuki/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("Failed to create HTTP client")?;

    let geocoding = GeocodingClient::new(http.clone());
    let weather = WeatherClient::new(http.clone());
    let air_quality = AirQualityClient::new(http);

    let location = resolve_location(&cli, &geocoding)?;
    let json = cli.json;
    let fahrenheit = cli.fahrenheit;

    // JSON is always metric; --fahrenheit converts for human output only.
    let output = match &cli.command {
        Command::Current { .. } => {
            let current = weather.current(&location)?;
            let data = CurrentResponse { location, current };
            if json {
                to_json(&data)?
            } else {
                format::current(&data, fahrenheit)
            }
        }
        Command::Forecast { days, .. } => {
            let forecast = weather.forecast(&location, *days)?;
            let data = ForecastResponse { location, forecast };
            if json {
                to_json(&data)?
            } else {
                format::forecast(&data, fahrenheit)
            }
        }
        Command::Radar { .. } => {
            let radar = weather.radar(&location)?;
            let data = RadarResponse { location, radar };
            if json {
                to_json(&data)?
            } else {
                format::radar(&data)
            }
        }
        Command::AirQuality { .. } => {
            let aq = air_quality.current(&location)?;
            let data = AirQualityResponse {
                location,
                air_quality: aq,
            };
            if json {
                to_json(&data)?
            } else {
                format::air_quality(&data)
            }
        }
    };

    println!("{output}");
    Ok(())
}

/// `--lat`/`--lon` name a point directly; otherwise the city (or `IBUKI_CITY`)
/// is geocoded. A coordinate lookup has no name or country, so it labels itself.
fn resolve_location(cli: &Cli, geocoding: &GeocodingClient) -> anyhow::Result<Location> {
    let (Some(lat), Some(lon)) = (cli.lat, cli.lon) else {
        let city = cli
            .command
            .city()
            .context("Error: Provide a city name, or --lat and --lon, or set IBUKI_CITY.")?;
        return Ok(geocoding.resolve(city)?);
    };

    anyhow::ensure!(
        (-90.0..=90.0).contains(&lat),
        "Error: Latitude must be between -90 and 90."
    );
    anyhow::ensure!(
        (-180.0..=180.0).contains(&lon),
        "Error: Longitude must be between -180 and 180."
    );

    Ok(Location {
        name: format!("{lat:.4}, {lon:.4}"),
        admin1: None,
        country: String::new(),
        latitude: lat,
        longitude: lon,
    })
}

fn to_json<T: serde::Serialize>(value: &T) -> anyhow::Result<String> {
    serde_json::to_string_pretty(value).context("Failed to serialize response")
}
