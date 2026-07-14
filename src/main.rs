use std::process::ExitCode;
use std::time::Duration;

use anyhow::Context;
use clap::Parser;

use ibuki::air_quality::AirQualityClient;
use ibuki::cli::{Cli, Command};
use ibuki::format::create_formatter;
use ibuki::geocoding::GeocodingClient;
use ibuki::models::{AirQualityResponse, CurrentResponse, ForecastResponse, RadarResponse};
use ibuki::weather::WeatherClient;

fn main() -> ExitCode {
    if let Err(err) = run() {
        eprintln!("{err}");
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

    let location = geocoding.resolve(cli.command.city())?;
    let formatter = create_formatter(cli.command.json());
    let fahrenheit = cli.command.fahrenheit();

    let output = match &cli.command {
        Command::Current { .. } => {
            let current = weather.current(&location)?;
            formatter.current(&CurrentResponse { location, current }, fahrenheit)
        }
        Command::Forecast { days, .. } => {
            let forecast = weather.forecast(&location, *days)?;
            formatter.forecast(&ForecastResponse { location, forecast }, fahrenheit)
        }
        Command::Radar { .. } => {
            let radar = weather.radar(&location)?;
            formatter.radar(&RadarResponse { location, radar })
        }
        Command::AirQuality { .. } => {
            let aq = air_quality.current(&location)?;
            formatter.air_quality(&AirQualityResponse {
                location,
                air_quality: aq,
            })
        }
    };

    println!("{output}");
    Ok(())
}
