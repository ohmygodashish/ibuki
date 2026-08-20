---
title: ibuki CLI Weather Tool Specification
version: 1.2
date_created: 2026-06-08
last_updated: 2026-08-21
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
| **admin1** | First-level administrative region (e.g. "Oregon"), used to disambiguate same-named cities |
| **AQI** | Air Quality Index — a numeric scale (0-500) indicating air pollution level |
| **API** | Application Programming Interface |
| **CLI** | Command-Line Interface |
| **Geocoding** | Converting a city name to geographic coordinates (latitude, longitude) |
| **Open-Meteo** | Free, open-source weather API service requiring no API key |
| **Radar Metrics** | Precipitation data including rain intensity, snowfall, and weather conditions |
| **Timezone abbreviation** | The label Open-Meteo reports for a location's local time (e.g. `GMT+9`) |
| **UTC** | Coordinated Universal Time |
| **WMO Code** | World Meteorological Organization weather condition code |

## 3. Requirements, Constraints & Guidelines

### Functional Requirements

- **REQ-001**: The tool SHALL provide a `current` subcommand to display current weather conditions
- **REQ-002**: The tool SHALL provide a `forecast` subcommand to display multi-day weather forecasts (default: 7 days)
- **REQ-003**: The tool SHALL provide a `radar` subcommand to display precipitation and radar-related metrics
- **REQ-004**: The tool SHALL provide an `air-quality` subcommand to display current air quality data
- **REQ-005**: The tool SHALL accept a city name as an optional positional argument for all subcommands
- **REQ-006**: The tool SHALL support a global `--json` flag to output machine-readable JSON instead of formatted terminal output
- **REQ-007**: The tool SHALL resolve city names to coordinates using the Open-Meteo Geocoding API
- **REQ-008**: The tool SHALL display temperature in Celsius by default
- **REQ-009**: The tool SHALL support a global `--fahrenheit` flag that converts temperature for human-readable output only; JSON output SHALL remain metric regardless of the flag
- **REQ-010**: The tool SHALL display colored, formatted output in the terminal when stdout is a TTY
- **REQ-011**: The tool SHALL disable colors when stdout is not a TTY or when `NO_COLOR` environment variable is set
- **REQ-012**: The `forecast` subcommand SHALL support a `--days <N>` option to specify forecast duration (1-16 days)
- **REQ-013**: The tool SHALL represent a reading the API omits as `null` in JSON and `n/a` in human-readable output, never as `0`
- **REQ-014**: The tool SHALL request `timezone=auto` and display timestamps in the location's local time, labelled with the timezone abbreviation the API reports; it SHALL NOT label local times as UTC
- **REQ-015**: The tool SHALL display the resolved region (`admin1`) alongside the city when it differs from the city name, so an ambiguous match is visible
- **REQ-016**: The tool SHALL support global `--lat` and `--lon` options that name a point directly and skip the geocoding lookup; each SHALL require the other, and out-of-range values SHALL be rejected
- **REQ-017**: The tool SHALL read a default city from the `IBUKI_CITY` environment variable when the positional argument is omitted; an explicit argument SHALL take precedence
- **REQ-018**: The tool SHALL resolve a location from exactly one source, in precedence order `--lat`/`--lon`, then positional city, then `IBUKI_CITY`; when none is supplied it SHALL fail with a message naming the alternatives
- **REQ-019**: The tool SHOULD cache city-to-coordinate results locally to avoid repeating the geocoding request. The cache SHALL be best-effort: an unwritable, missing, or malformed cache SHALL degrade to a normal lookup, never an error

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
- **ERR-005**: The tool SHALL append the underlying cause to the user-friendly message, so a failure reports why it failed

### Constraints

- **CON-001**: The tool MUST use Open-Meteo APIs (no API key required)
- **CON-002**: The tool MUST be implemented in Rust
- **CON-003**: The tool MUST NOT require user authentication or API key configuration
- **CON-004**: The tool MUST NOT transmit user data or track usage. The sole permitted local
  artifact is the geocoding cache (REQ-019): city names the user queried, mapped to public
  coordinates, written under the user's own cache directory and never transmitted anywhere.
  It contains no identity, credentials, or timestamps, and may be deleted at any time
- **CON-005**: The tool MUST NOT persist results obtained from a redirected endpoint
  (`IBUKI_GEOCODING_URL`), so test and mock coordinates cannot enter the user's cache

### Guidelines

- **GUD-001**: Prefer explicit error messages over silent failures
- **GUD-002**: Use consistent formatting across all subcommands
- **GUD-003**: Keep terminal output concise; avoid information overload
- **GUD-004**: Follow Rust CLI best practices (clap for argument parsing, anyhow/thiserror for errors)

### Patterns

- **PAT-001**: Give each API a plain client struct (`GeocodingClient`, `WeatherClient`, `AirQualityClient`) sharing one `reqwest` client; no trait abstraction over a single implementation
- **PAT-002**: Make clients testable by overriding their base URL — via `with_base_url` in-process, or the `IBUKI_GEOCODING_URL` / `IBUKI_FORECAST_URL` / `IBUKI_AIR_QUALITY_URL` environment variables for end-to-end tests
- **PAT-003**: Separate data fetching (`geocoding`, `weather`, `air_quality`), domain models (`models`), and presentation (`format`) layers
- **PAT-004**: Render human-readable output as plain text first, measure width, then apply color — never measure a string that already contains escape codes

## 4. Interfaces & Data Contracts

### CLI Interface

```
ibuki <SUBCOMMAND> [CITY] [OPTIONS]

SUBCOMMANDS:
  current       Display current weather conditions
  forecast      Display multi-day weather forecast
  radar         Display precipitation and radar metrics
  air-quality   Display air quality information

ARGUMENTS:
  [CITY]        City name (e.g., "Tokyo", "New York", "London").
                Optional; falls back to $IBUKI_CITY, or omit entirely when
                using --lat/--lon

OPTIONS:
  --json        Output in JSON format, always metric (global)
  --fahrenheit  Display temperature in Fahrenheit, human output only (global)
  --lat <LAT>   Latitude (-90..=90), skips geocoding; requires --lon (global)
  --lon <LON>   Longitude (-180..=180), skips geocoding; requires --lat (global)
  --days <N>    Number of forecast days (1-16, default: 7, forecast only)
  -h, --help    Print help information
  -V, --version Print version information

ENVIRONMENT:
  IBUKI_CITY    Default city when no positional argument is given
  NO_COLOR      Disable colored output
```

### Local Cache

City-to-coordinate results are cached at `$XDG_CACHE_HOME/ibuki/geocoding.json`, falling
back to `%LOCALAPPDATA%\ibuki\geocoding.json` on Windows and `~/.cache/ibuki/geocoding.json`
otherwise. Keys are the trimmed, lowercased city name; values are `Location` objects.
The file is safe to delete, and a corrupt one is treated as empty.

### JSON Output Schemas

The schema is metric-only and identical for every subcommand; `--fahrenheit` does not
change it. Any reading the API omits is serialized as `null`. Timestamps are local to
the queried location and carry no `Z` suffix; `timezone` names the offset the API
reported, and is `null` when the API omits it.

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
    "feels_like_c": 23.1,
    "humidity_percent": 65,
    "wind_speed_kmh": 12.3,
    "wind_direction_deg": 180.0,
    "weather_code": 1,
    "weather_description": "Mainly clear",
    "timestamp": "2026-06-08T14:30",
    "timezone": "GMT+9"
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
    "timestamp": "2026-06-08T14:30",
    "timezone": "GMT+9"
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
- **AC-008**: Given `--fahrenheit` flag, When human-readable temperature is displayed, Then values are in Fahrenheit; When `--json` is also given, Then values remain Celsius
- **AC-009**: Given stdout is not a TTY, When output is displayed, Then ANSI color codes are omitted
- **AC-010**: Given `--days 20` (out of range), When `forecast` is executed, Then an error message indicates valid range is `1..=16`
- **AC-011**: Given the API omits a reading, When output is displayed, Then it shows `n/a` in human output and `null` in JSON, never `0`
- **AC-012**: Given a location outside UTC, When a timestamp is displayed, Then it is local time labelled with the API's timezone abbreviation
- **AC-013**: Given a city whose region differs from its name (e.g. Portland), When any subcommand is executed, Then the header reads "Portland, Oregon, United States"
- **AC-014**: Given a request that fails, When the error is printed, Then the message includes the underlying cause
- **AC-015**: Given `--lat 35.6895 --lon 139.6917`, When any subcommand is executed, Then weather is returned with no geocoding request, headed by the coordinates
- **AC-016**: Given `--lat` without `--lon`, When any subcommand is executed, Then argument parsing fails naming the missing option
- **AC-017**: Given `--lat 91`, When any subcommand is executed, Then the command fails stating the valid range
- **AC-018**: Given `IBUKI_CITY=Tokyo` and no positional argument, When `ibuki current` is executed, Then Tokyo's weather is displayed
- **AC-019**: Given no city and no coordinates, When any subcommand is executed, Then the error names the positional argument, `--lat`/`--lon`, and `IBUKI_CITY`
- **AC-020**: Given a city resolved once, When it is requested again, Then the coordinates come from the cache and no geocoding request is made
- **AC-021**: Given a malformed cache file, When a city is requested, Then the lookup succeeds over the network and the cache is rewritten

## 6. Test Automation Strategy

- **Test Levels**:
  - Unit tests: Data transformation, formatting, argument parsing
  - Integration tests: API client with mocked HTTP responses
  - End-to-end tests: Full CLI execution with mocked APIs

- **Frameworks**:
  - `cargo test` (built-in Rust test framework)
  - `mockito` for HTTP mocking
  - `assert_cmd` and `predicates` for CLI integration testing

- **Test Data Management**:
  - Store sample API responses as JSON fixtures in `tests/fixtures/`
  - Use fixture files for consistent test data across test runs
  - Tests SHALL NOT reach the network; redirect every client at a mock server via
    `with_base_url` or the `IBUKI_*_URL` environment variables
  - Fixtures carry `"timezone_abbreviation": "JST"`, while live Open-Meteo returns
    `GMT+9`; assertions on a specific abbreviation MUST use the fixtures

- **CI/CD Integration**:
  - Run `cargo test` on every PR via GitHub Actions, across a Linux, macOS and Windows
    matrix with `fail-fast: false` so every platform reports independently (NFR-003, PLT-002)
  - Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` for code quality;
    both are platform-independent and run on Linux only
  - Enforce the NFR-002 binary size budget on the release build, on Linux only —
    `stat -c%s` is GNU-specific and the artifact is `ibuki.exe` on Windows

- **Coverage Requirements**:
  - Minimum 80% code coverage for core logic
  - 100% coverage for error handling paths
  - **Known gap**: no coverage tooling is wired into CI; this is currently unmeasured

- **Performance Testing**:
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

**Why city name first, with coordinates as an escape hatch?**
- A city name is the simplest user experience for the primary use case
- Geocoding API handles name resolution transparently
- `--lat`/`--lon` serve users who know the exact point, want a location geocoding
  cannot name, or want to skip the extra round trip entirely
- IP-based location remains out of scope: it would require a third-party service and
  contradicts CON-004

**Why cache geocoding but not weather?**
- A city's coordinates do not change, so the cache needs no invalidation strategy
- Weather does change, and caching it would serve stale readings
- The cache is best-effort by design: correctness never depends on it, so a read-only
  or full disk degrades latency rather than breaking the tool

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
┌────────────────────────────────────────────┐
│  Tokyo, Japan                              │
│  Current Weather                           │
│  ────────────────────────────────────────  │
│  Temperature:  22.5°C (feels 23.1°C)       │
│  Conditions:   Mainly clear                │
│  Humidity:     65%                         │
│  Wind:         12.3 km/h (S)               │
│  Updated:      2026-06-08 14:30 GMT+9      │
└────────────────────────────────────────────┘
```

Tokyo's `admin1` is also "Tokyo", so the region is suppressed. A city whose region
differs shows all three parts: `Portland, Oregon, United States`.

### Edge Cases

| Scenario | Expected Behavior |
|----------|-------------------|
| City not found | Display "Error: City 'Xyz' not found. Please check the spelling." |
| Multiple cities with same name | Use the single result geocoding returns for `count=1`, and show its `admin1` in the header so the pick is visible |
| City name with spaces | Accept quoted strings: `ibuki current "New York"` |
| Network timeout | Display "Error: Network request timed out. Please try again." |
| API returns empty data | Display "Error: No weather data available for this location." |
| Empty or whitespace-only city name | Display "Error: City '   ' not found. Please check the spelling." without issuing a request |
| Invalid UTF-8 in city name | Rejected by `clap` during argument parsing before any request is made |
| `--days 0` or `--days 100` | Display "error: invalid value '0' for '--days <DAYS>': 0 is not in 1..=16" |
| Double-width (CJK) city name | Box borders may overhang; widths are counted in `char`s, not display columns |
| `--lat` given without `--lon` | `clap` rejects the invocation, naming the missing option |
| Latitude outside -90..=90 | Display "Error: Latitude must be between -90 and 90." |
| Both a city and `--lat`/`--lon` given | Coordinates win; no geocoding request is made |
| Cache file missing, unreadable, or corrupt | Treated as empty; the lookup proceeds over the network |
| Cache directory not writable | The lookup still succeeds; only the round-trip saving is lost |

## 10. Validation Criteria

- [ ] All four subcommands (`current`, `forecast`, `radar`, `air-quality`) produce correct output
- [ ] `--json` flag produces valid JSON matching defined schemas, in metric regardless of `--fahrenheit`
- [ ] `--fahrenheit` flag correctly converts temperatures in human-readable output
- [ ] Omitted readings render as `n/a` / `null`, never `0`
- [ ] Timestamps are local and labelled with the API's timezone abbreviation
- [ ] Errors report their underlying cause
- [ ] `--lat`/`--lon` skip geocoding, require each other, and reject out-of-range values
- [ ] `IBUKI_CITY` supplies a default city, overridden by an explicit argument
- [ ] A repeated city lookup is served from the cache, and a corrupt cache self-heals
- [ ] `--days` flag works within valid range (1-16)
- [ ] Invalid city names produce clear error messages
- [ ] Network errors are handled gracefully
- [ ] Binary compiles and runs on Linux, macOS, and Windows (verified by the CI matrix)
- [ ] `cargo test` passes with minimum 80% coverage
- [ ] `cargo clippy` produces no warnings
- [ ] `cargo fmt --check` passes
- [ ] Binary size is under 5 MB
- [ ] Cold-start to output is under 2 seconds

## 11. Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-06-08 | Initial specification |
| 1.2 | 2026-08-21 | Added `--lat`/`--lon` coordinate input and the `IBUKI_CITY` default city (REQ-016 to REQ-018), making the positional city optional (REQ-005). Added a best-effort local geocoding cache (REQ-019). CON-004 amended: the cache is the one permitted local artifact, and CON-005 added so mock endpoints cannot write to it. |
| 1.1 | 2026-08-20 | Reconciled with the implementation. Timestamps corrected from UTC to location-local with an API-reported abbreviation (REQ-014); JSON declared metric-only and the `temperature_f`/`feels_like_f` twins dropped from the schema (REQ-009); omitted readings defined as `null`/`n/a` rather than `0` (REQ-013); `admin1` added to `Location` and the header (REQ-015); error output now carries its cause (ERR-005). PAT-001/PAT-002 rewritten — the formatter trait and factory were removed in favour of free functions and base-URL injection. Test strategy corrected to the frameworks actually in use, with unmeasured coverage recorded as a known gap. CI expanded from `ubuntu-latest` to a Linux/macOS/Windows matrix, so NFR-003 and PLT-002 are now verified rather than asserted. |

## 12. Related Specifications / Further Reading

- [Open-Meteo API Documentation](https://open-meteo.com/en/docs)
- [Open-Meteo Geocoding API](https://open-meteo.com/en/docs/geocoding-api)
- [Open-Meteo Air Quality API](https://open-meteo.com/en/docs/air-quality-api)
- [WMO Weather Condition Codes](https://www.nodc.noaa.gov/archive/arc0021/0002199/1.1/data/0-data/HTML/WMO-CODE/WMO4677.HTM)
- [Rust CLI Best Practices](https://rust-cli-recommendations.sunshowers.io/)
- [NO_COLOR Standard](https://no-color.org/)
