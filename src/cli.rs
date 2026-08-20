use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "ibuki",
    version,
    about = "A lightweight CLI weather tool",
    long_about = "Get real-time forecasts, radar metrics, and air quality directly in your terminal."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Output in JSON format (always metric)
    #[arg(long, global = true)]
    pub json: bool,

    /// Display temperature in Fahrenheit
    #[arg(long, global = true)]
    pub fahrenheit: bool,

    /// Latitude, skipping the geocoding lookup (requires --lon)
    #[arg(long, global = true, requires = "lon", allow_negative_numbers = true)]
    pub lat: Option<f64>,

    /// Longitude, skipping the geocoding lookup (requires --lat)
    #[arg(long, global = true, requires = "lat", allow_negative_numbers = true)]
    pub lon: Option<f64>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Display current weather conditions
    Current {
        /// City name (e.g., "Tokyo", "New York", "London")
        #[arg(env = "IBUKI_CITY")]
        city: Option<String>,
    },

    /// Display multi-day weather forecast
    Forecast {
        /// City name (e.g., "Tokyo", "New York", "London")
        #[arg(env = "IBUKI_CITY")]
        city: Option<String>,

        /// Number of forecast days (1-16, default: 7)
        #[arg(long, default_value_t = 7, value_parser = clap::value_parser!(u8).range(1..=16))]
        days: u8,
    },

    /// Display precipitation and radar metrics
    Radar {
        /// City name (e.g., "Tokyo", "New York", "London")
        #[arg(env = "IBUKI_CITY")]
        city: Option<String>,
    },

    /// Display air quality information
    #[command(name = "air-quality")]
    AirQuality {
        /// City name (e.g., "Tokyo", "New York", "London")
        #[arg(env = "IBUKI_CITY")]
        city: Option<String>,
    },
}

impl Command {
    pub fn city(&self) -> Option<&str> {
        match self {
            Self::Current { city }
            | Self::Forecast { city, .. }
            | Self::Radar { city }
            | Self::AirQuality { city } => city.as_deref(),
        }
    }
}
