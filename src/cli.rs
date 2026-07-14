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
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Display current weather conditions
    Current {
        /// City name (e.g., "Tokyo", "New York", "London")
        city: String,

        /// Output in JSON format
        #[arg(long)]
        json: bool,

        /// Display temperature in Fahrenheit
        #[arg(long)]
        fahrenheit: bool,
    },

    /// Display multi-day weather forecast
    Forecast {
        /// City name (e.g., "Tokyo", "New York", "London")
        city: String,

        /// Output in JSON format
        #[arg(long)]
        json: bool,

        /// Display temperature in Fahrenheit
        #[arg(long)]
        fahrenheit: bool,

        /// Number of forecast days (1-16, default: 7)
        #[arg(long, default_value_t = 7, value_parser = clap::value_parser!(u8).range(1..=16))]
        days: u8,
    },

    /// Display precipitation and radar metrics
    Radar {
        /// City name (e.g., "Tokyo", "New York", "London")
        city: String,

        /// Output in JSON format
        #[arg(long)]
        json: bool,

        /// Display temperature in Fahrenheit
        #[arg(long)]
        fahrenheit: bool,
    },

    /// Display air quality information
    #[command(name = "air-quality")]
    AirQuality {
        /// City name (e.g., "Tokyo", "New York", "London")
        city: String,

        /// Output in JSON format
        #[arg(long)]
        json: bool,

        /// Display temperature in Fahrenheit
        #[arg(long)]
        fahrenheit: bool,
    },
}

impl Command {
    pub fn city(&self) -> &str {
        match self {
            Self::Current { city, .. }
            | Self::Forecast { city, .. }
            | Self::Radar { city, .. }
            | Self::AirQuality { city, .. } => city,
        }
    }

    pub fn json(&self) -> bool {
        match self {
            Self::Current { json, .. }
            | Self::Forecast { json, .. }
            | Self::Radar { json, .. }
            | Self::AirQuality { json, .. } => *json,
        }
    }

    pub fn fahrenheit(&self) -> bool {
        match self {
            Self::Current { fahrenheit, .. }
            | Self::Forecast { fahrenheit, .. }
            | Self::Radar { fahrenheit, .. }
            | Self::AirQuality { fahrenheit, .. } => *fahrenheit,
        }
    }
}
