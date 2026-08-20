use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

use crate::error::{AppError, Result, check_status, map_reqwest_error};
use crate::models::Location;

const GEOCODING_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";

#[derive(Debug, Deserialize)]
struct GeocodingResponse {
    results: Option<Vec<GeocodingResult>>,
}

#[derive(Debug, Deserialize)]
struct GeocodingResult {
    name: String,
    admin1: Option<String>,
    country: Option<String>,
    latitude: f64,
    longitude: f64,
}

pub struct GeocodingClient {
    http: reqwest::blocking::Client,
    base_url: String,
    /// Where to persist city -> coordinates; `None` disables caching entirely.
    cache: Option<PathBuf>,
}

impl GeocodingClient {
    pub fn new(http: reqwest::blocking::Client) -> Self {
        // Only the real service is cached: coordinates from a redirected (test)
        // endpoint must never be written to the user's cache.
        match std::env::var("IBUKI_GEOCODING_URL") {
            Ok(base_url) => Self::with_base_url(http, base_url),
            Err(_) => Self {
                http,
                base_url: GEOCODING_URL.to_string(),
                cache: cache_path(),
            },
        }
    }

    pub fn with_base_url(http: reqwest::blocking::Client, base_url: impl Into<String>) -> Self {
        Self {
            http,
            base_url: base_url.into(),
            cache: None,
        }
    }

    pub fn with_cache(mut self, path: impl Into<PathBuf>) -> Self {
        self.cache = Some(path.into());
        self
    }

    pub fn resolve(&self, city: &str) -> Result<Location> {
        if city.trim().is_empty() {
            return Err(AppError::CityNotFound(city.to_string()));
        }

        let key = city.trim().to_lowercase();
        if let Some(hit) = self
            .cache
            .as_ref()
            .and_then(|path| read_cache(path).remove(&key))
        {
            return Ok(hit);
        }

        let response = self
            .http
            .get(&self.base_url)
            .query(&[
                ("name", city),
                ("count", "1"),
                ("language", "en"),
                ("format", "json"),
            ])
            .send()
            .map_err(map_reqwest_error)?;

        check_status(&response, "Geocoding")?;

        let body: GeocodingResponse = response.json().map_err(AppError::Parse)?;

        let result = body
            .results
            .and_then(|mut results| results.pop())
            .ok_or_else(|| AppError::CityNotFound(city.to_string()))?;

        let location = Location {
            name: result.name,
            admin1: result.admin1,
            country: result.country.unwrap_or_else(|| "Unknown".to_string()),
            latitude: result.latitude,
            longitude: result.longitude,
        };

        if let Some(path) = &self.cache {
            write_cache(path, key, &location);
        }
        Ok(location)
    }
}

/// `$XDG_CACHE_HOME`, `%LOCALAPPDATA%` on Windows, else `~/.cache`.
fn cache_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .or_else(|| std::env::var_os("LOCALAPPDATA"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))?;
    Some(base.join("ibuki").join("geocoding.json"))
}

/// Best-effort: an unreadable or malformed cache is simply an empty one.
fn read_cache(path: &PathBuf) -> HashMap<String, Location> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

/// Best-effort: a city's coordinates do not change, so there is nothing to
/// invalidate and a failed write only costs the next lookup a round trip.
///
/// ponytail: the map grows without bound. Add eviction if anyone ever looks up
/// enough cities for it to matter.
fn write_cache(path: &PathBuf, key: String, location: &Location) {
    let mut entries = read_cache(path);
    entries.insert(key, location.clone());
    let Ok(serialized) = serde_json::to_string(&entries) else {
        return;
    };
    if path
        .parent()
        .is_none_or(|dir| std::fs::create_dir_all(dir).is_ok())
    {
        let _ = std::fs::write(path, serialized);
    }
}
