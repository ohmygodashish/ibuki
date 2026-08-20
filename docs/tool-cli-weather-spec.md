---
title: ibuki CLI Weather Tool Specification
version: 1.0
date_created: 2026-06-08
last_updated: 2026-06-08
owner: ibuki
tags: [tool, cli, rust, weather]
---

# Introduction

This specification defines the requirements, constraints, and interfaces for **ibuki**, a lightweight command-line weather tool written in Rust. ibuki provides real-time weather forecasts, radar metrics, and air quality information directly in the terminal with optional JSON output for programmatic use.

## 1. Purpose & Scope

**Purpose**: Define the complete functional and non-functional requirements for the ibuki CLI tool, enabling developers to implement a consistent, reliable, and user-friendly weather information utility.

**Scope**: This specification covers:
- CLI command structure and argument parsing
- Weather data retrieval from Open-Meteo APIs
- Terminal output formatting (human-readable and JSON)
- Error handling and user feedback
- Cross-platform compatibility

**Intended Audience**: Developers implementing or contributing to ibuki.

**Assumptions**:
- Users have internet connectivity to reach Open-Meteo APIs
- City names are provided in ASCII or UTF-8 encoding
- Open-Meteo APIs remain freely accessible without authentication

## 2. Definitions

| Term | Definition |
|------|------------|
| **AQI** | Air Quality Index — a numeric scale (0-500) indicating air pollution level |
| **API** | Application Programming Interface |
| **CLI** | Command-Line Interface |
| **Geocoding** | Converting a city name to geographic coordinates (latitude, longitude) |
| **Open-Meteo** | Free, open-source weather API service requiring no API key |
| **Radar Metrics** | Precipitation data including rain intensity, snowfall, and weather conditions |
| **UTC** | Coordinated Universal Time |
| **WMO Code** | World Meteorological Organization weather condition code |

## 3. Requirements, Constraints & Guidelines

### Functional Requirements

- **REQ-001**: The tool SHALL provide a `current` subcommand to display current weather conditions
- **REQ-002**: The tool SHALL provide a `forecast` subcommand to display multi-day weather forecasts (default: 7 days)
- **REQ-003**: The tool SHALL provide a `radar` subcommand to display precipitation and radar-related metrics
- **REQ-004**: The tool SHALL provide an `air-quality` subcommand to display current air quality data
- **REQ-005**: The tool SHALL accept a city name as a positional argument for all subcommands
- **REQ-006**: The tool SHALL support a `--json` flag to output machine-readable JSON instead of formatted terminal output
- **REQ-007**: The tool SHALL resolve city names to coordinates using the Open-Meteo Geocoding API
- **REQ-008**: The tool SHALL display temperature in Celsius by default
- **REQ-009**: The tool SHALL support a `--fahrenheit` flag to display temperature in Fahrenheit
- **REQ-010**: The tool SHALL display colored, formatted output in the terminal when stdout is a TTY
- **REQ-011**: The tool SHALL disable colors when stdout is not a TTY or when `NO_COLOR` environment variable is set
- **REQ-012**: The `forecast` subcommand SHALL support a `--days <N>` option to specify forecast duration (1-16 days)

### Non-Functional Requirements

- **NFR-001**: The tool SHALL complete API requests within 10 seconds under normal network conditions
- **NFR-002**: The tool SHALL have a binary size under 5 MB
- **NFR-003**: The tool SHALL support Linux, macOS, and Windows platforms
- **NFR-004**: The tool SHALL exit with code 0 on success, non-zero on failure

### Error Handling Requirements

- **ERR-001**: The tool SHALL display a user-friendly error message when a city is not found
- **ERR-002**: The tool SHALL display a user-friendly error message when network requests fail
- **ERR-003**: The tool SHALL display a user-friendly error message when API rate limits are exceeded
- **ERR-004**: The tool SHALL NOT panic on any input; all errors SHALL be handled gracefully

### Constraints

- **CON-001**: The tool MUST use Open-Meteo APIs (no API key required)
- **CON-002**: The tool MUST be implemented in Rust
- **CON-003**: The tool MUST NOT require user authentication or API key configuration
- **CON-004**: The tool MUST NOT store user data or track usage

### Guidelines

- **GUD-001**: Prefer explicit error messages over silent failures
- **GUD-002**: Use consistent formatting across all subcommands
- **GUD-003**: Keep terminal output concise; avoid information overload
- **GUD-004**: Follow Rust CLI best practices (clap for argument parsing, anyhow/thiserror for errors)

### Patterns

- **PAT-001**: Use the Repository pattern for API client abstraction
- **PAT-002**: Use trait-based dependency injection for testability
- **PAT-003**: Separate data fetching, transformation, and presentation layers

## 4. Interfaces & Data Contracts

### CLI Interface

```
ibuki <SUBCOMMAND> <CITY> [OPTIONS]

SUBCOMMANDS:
  current       Display current weather conditions
  forecast      Display multi-day weather forecast
  radar         Display precipitation and radar metrics
  air-quality   Display air quality information

ARGUMENTS:
  <CITY>        City name (e.g., "Tokyo", "New York", "London")

OPTIONS:
  --json        Output in JSON format
  --fahrenheit  Display temperature in Fahrenheit
  --days <N>    Number of forecast days (1-16, default: 7, forecast only)
  -h, --help    Print help information
  -V, --version Print version information
```

### JSON Output Schemas

**current**:
```json
{
  "location": {
    "name": "Tokyo",
    "admin1": "Tokyo",
    "country": "Japan",
    "latitude": 35.6762,
    "longitude": 139.6503
  },
  "current": {
    "temperature_c": 22.5,
    "temperature_f": 72.5,
    "feels_like_c": 23.1,
    "feels_like_f": 73.6,
    "humidity_percent": 65,
    "wind_speed_kmh": 12.3,
    "wind_direction_deg": 180,
    "weather_code": 1,
    "weather_description": "Mainly clear",
    "timestamp": "2026-06-08T14:30:00Z"
  }
}
```

**forecast**:
```json
{
  "location": {
    "name": "Tokyo",
    "admin1": "Tokyo",
    "country": "Japan",
    "latitude": 35.6762,
    "longitude": 139.6503
  },
  "forecast": [
    {
      "date": "2026-06-08",
      "temperature_max_c": 28.0,
      "temperature_min_c": 20.0,
      "precipitation_sum_mm": 2.5,
      "precipitation_probability_percent": 40,
      "wind_speed_max_kmh": 25.0,
      "weather_code": 2,
      "weather_description": "Partly cloudy"
    }
  ]
}
```

**radar**:
```json
{
  "location": {
    "name": "Tokyo",
    "admin1": "Tokyo",
    "country": "Japan",
    "latitude": 35.6762,
    "longitude": 139.6503
  },
  "radar": {
    "precipitation_last_hour_mm": 0.5,
    "precipitation_forecast_next_hour_mm": 1.2,
    "rain_intensity_mm_h": 0.3,
    "snowfall_cm_h": 0.0,
    "showers_mm_h": 0.2,
    "weather_code": 61,
    "weather_description": "Slight rain"
  }
}
```

**air-quality**:
```json
{
  "location": {
    "name": "Tokyo",
    "admin1": "Tokyo",
    "country": "Japan",
    "latitude": 35.6762,
    "longitude": 139.6503
  },
  "air_quality": {
    "aqi_us": 45,
    "aqi_eu": 38,
    "pm2_5_ug_m3": 12.3,
    "pm10_ug_m3": 25.1,
    "ozone_ug_m3": 68.0,
    "nitrogen_dioxide_ug_m3": 18.5,
    "carbon_monoxide_ug_m3": 250.0,
    "timestamp": "2026-06-08T14:30:00Z"
  }
}
```

### Open-Meteo API Contracts

**Geocoding API**:
- Endpoint: `https://geocoding-api.open-meteo.com/v1/search`
- Query: `name={city}&count=1&language=en&format=json`
- Response: `results[0].latitude`, `results[0].longitude`, `results[0].name`, `results[0].admin1`, `results[0].country`

**Weather Forecast API**:
- Endpoint: `https://api.open-meteo.com/v1/forecast`
- Query parameters: `latitude`, `longitude`, `current`, `daily`, `hourly`, `timezone=auto`

**Air Quality API**:
- Endpoint: `https://air-quality-api.open-meteo.com/v1/air-quality`
- Query parameters: `latitude`, `longitude`, `current=us_aqi,european_aqi,pm2_5,pm10,ozone,nitrogen_dioxide,carbon_monoxide`

## 5. Acceptance Criteria

- **AC-001**: Given a valid city name, When `ibuki current Tokyo` is executed, Then current weather is displayed with temperature, humidity, wind, and conditions
- **AC-002**: Given a valid city name, When `ibuki forecast London --days 3` is executed, Then a 3-day forecast is displayed
- **AC-003**: Given a valid city name, When `ibuki radar "New York"` is executed, Then precipitation metrics are displayed
- **AC-004**: Given a valid city name, When `ibuki air-quality Paris` is executed, Then AQI and pollutant data are displayed
- **AC-005**: Given any subcommand, When `--json` flag is provided, Then output is valid JSON matching the defined schema
- **AC-006**: Given an invalid city name, When any subcommand is executed, Then a clear error message is displayed and exit code is 1
- **AC-007**: Given network failure, When any subcommand is executed, Then a clear error message is displayed and exit code is 1
- **AC-008**: Given `--fahrenheit` flag, When temperature is displayed, Then values are in Fahrenheit
- **AC-009**: Given stdout is not a TTY, When output is displayed, Then ANSI color codes are omitted
- **AC-010**: Given `--days 20` (out of range), When `forecast` is executed, Then an error message indicates valid range is 1-16

## 6. Test Automation Strategy

- **Test Levels**:
  - Unit tests: Data transformation, formatting, argument parsing
  - Integration tests: API client with mocked HTTP responses
  - End-to-end tests: Full CLI execution with mocked APIs

- **Frameworks**:
  - `cargo test` (built-in Rust test framework)
  - `mockito` or `wiremock` for HTTP mocking
  - `assert_cmd` for CLI integration testing
  - `insta` for snapshot testing of terminal output

- **Test Data Management**:
  - Store sample API responses as JSON fixtures in `tests/fixtures/`
  - Use fixture files for consistent test data across test runs

- **CI/CD Integration**:
  - Run `cargo test` on every PR via GitHub Actions
  - Run `cargo clippy` and `cargo fmt --check` for code quality

- **Coverage Requirements**:
  - Minimum 80% code coverage for core logic
  - 100% coverage for error handling paths

- **Performance Testing**:
  - Benchmark API response parsing with `criterion`
  - Ensure cold-start to output is under 2 seconds

## 7. Rationale & Context

**Why Open-Meteo?**
- Free and open-source with no API key requirement
- Global coverage with reliable data from multiple weather models
- Simple REST API with comprehensive documentation
- No rate limiting for reasonable personal use

**Why Rust?**
- Single binary distribution (no runtime dependencies)
- Excellent performance and low memory footprint
- Strong type safety and error handling
- Cross-platform compilation support

**Why city name only (no coordinates/IP)?**
- Simplest user experience for the primary use case
- Geocoding API handles name resolution transparently
- Future versions may add coordinate and IP-based location

**Why Celsius default?**
- Metric is the international standard
- Fahrenheit available via flag for users who prefer it

## 8. Dependencies & External Integrations

### External Systems
- **EXT-001**: Open-Meteo Geocoding API — City name to coordinate resolution
- **EXT-002**: Open-Meteo Weather Forecast API — Current weather, forecasts, and radar metrics
- **EXT-003**: Open-Meteo Air Quality API — Air quality index and pollutant data

### Third-Party Services
- **SVC-001**: Open-Meteo — Free tier, no SLA required, best-effort availability

### Infrastructure Dependencies
- **INF-001**: Internet connectivity — Required for all API requests
- **INF-002**: DNS resolution — Required to resolve Open-Meteo API hostnames

### Data Dependencies
- **DAT-001**: Open-Meteo weather data — JSON format, real-time on each request

### Technology Platform Dependencies
- **PLT-001**: Rust toolchain — Edition 2024 or later
- **PLT-002**: Target platforms — Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x86_64)

### Compliance Dependencies
- **COM-001**: None — No user data collection, no authentication, no tracking

## 9. Examples & Edge Cases

### Example Commands

```bash
# Current weather in Tokyo
ibuki current Tokyo

# 5-day forecast for London in Fahrenheit
ibuki forecast London --days 5 --fahrenheit

# Radar metrics as JSON
ibuki radar "San Francisco" --json

# Air quality with JSON output
ibuki air-quality Berlin --json

# Help text
ibuki --help
ibuki forecast --help
```

### Example Terminal Output (current)

```
┌─────────────────────────────────────────┐
│  Tokyo, Japan                           │
│  Current Weather                        │
├─────────────────────────────────────────┤
│  Temperature:  22.5°C (feels 23.1°C)    │
│  Conditions:   Mainly clear             │
│  Humidity:     65%                      │
│  Wind:         12.3 km/h (S)            │
│  Updated:      2026-06-08 14:30 UTC     │
└─────────────────────────────────────────┘
```

### Edge Cases

| Scenario | Expected Behavior |
|----------|-------------------|
| City not found | Display "Error: City 'Xyz' not found. Please check the spelling." |
| Multiple cities with same name | Use the first result from geocoding (highest population) |
| City name with spaces | Accept quoted strings: `ibuki current "New York"` |
| Network timeout | Display "Error: Network request timed out. Please try again." |
| API returns empty data | Display "Error: No weather data available for this location." |
| Invalid UTF-8 in city name | Display "Error: Invalid city name encoding." |
| `--days 0` or `--days 100` | Display "Error: Days must be between 1 and 16." |

## 10. Validation Criteria

- [ ] All four subcommands (`current`, `forecast`, `radar`, `air-quality`) produce correct output
- [ ] `--json` flag produces valid JSON matching defined schemas
- [ ] `--fahrenheit` flag correctly converts temperatures
- [ ] `--days` flag works within valid range (1-16)
- [ ] Invalid city names produce clear error messages
- [ ] Network errors are handled gracefully
- [ ] Binary compiles and runs on Linux, macOS, and Windows
- [ ] `cargo test` passes with minimum 80% coverage
- [ ] `cargo clippy` produces no warnings
- [ ] `cargo fmt --check` passes
- [ ] Binary size is under 5 MB
- [ ] Cold-start to output is under 2 seconds

## 11. Related Specifications / Further Reading

- [Open-Meteo API Documentation](https://open-meteo.com/en/docs)
- [Open-Meteo Geocoding API](https://open-meteo.com/en/docs/geocoding-api)
- [Open-Meteo Air Quality API](https://open-meteo.com/en/docs/air-quality-api)
- [WMO Weather Condition Codes](https://www.nodc.noaa.gov/archive/arc0021/0002199/1.1/data/0-data/HTML/WMO-CODE/WMO4677.HTM)
- [Rust CLI Best Practices](https://rust-cli-recommendations.sunshowers.io/)
- [NO_COLOR Standard](https://no-color.org/)
