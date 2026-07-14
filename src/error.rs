use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Error: City '{0}' not found. Please check the spelling.")]
    CityNotFound(String),

    #[error("Error: Network request timed out. Please try again.")]
    Timeout,

    #[error("Error: Network request failed. Please check your connection.")]
    Network(#[source] reqwest::Error),

    #[error("Error: Failed to parse API response.")]
    Parse(#[source] reqwest::Error),

    #[error("Error: API rate limit exceeded. Please try again later.")]
    RateLimited,

    #[error("Error: No weather data available for this location.")]
    EmptyData,

    #[error("Error: {0}")]
    Api(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
