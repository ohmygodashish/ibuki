# ibuki

A lightweight CLI weather tool written in Rust. Get real-time forecasts, radar metrics, and air quality directly in your terminal.

Powered by [Open-Meteo](https://open-meteo.com/) — free, no API key required.

## Install

**From source** (requires [Rust](https://rustup.rs/)):

```bash
cargo install --path .
```

Or build a release binary:

```bash
cargo build --release
./target/release/ibuki current Tokyo
```

## Usage

```bash
ibuki <COMMAND> <CITY> [OPTIONS]
```

### Commands

| Command | Description |
|---------|-------------|
| `current` | Current weather conditions |
| `forecast` | Multi-day forecast (default: 7 days) |
| `radar` | Precipitation and radar metrics |
| `air-quality` | Air quality index and pollutants |

### Options

| Flag | Description |
|------|-------------|
| `--json` | Machine-readable JSON output (always metric) |
| `--fahrenheit` | Temperatures in °F (default: °C); affects human output only |
| `--days <N>` | Forecast length, 1–16 (forecast only) |
| `-h, --help` | Help |
| `-V, --version` | Version |

### Examples

```bash
# Current weather in Tokyo
ibuki current Tokyo

# 5-day forecast for London in Fahrenheit
ibuki forecast London --days 5 --fahrenheit

# Radar metrics as JSON
ibuki radar "New York" --json

# Air quality
ibuki air-quality Paris

# Help
ibuki --help
ibuki forecast --help
```

Quote city names that contain spaces:

```bash
ibuki current "San Francisco"
```

### Sample output

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

Ambiguous city names resolve to one arbitrary match, so the header names the region
when it differs from the city — "Portland, Oregon, United States". Timestamps are
local to the queried location — the trailing label is the timezone
Open-Meteo reports for it. Readings the API omits show as `n/a` rather than `0`.

Use `--json` for scripts and tooling: it is a stable metric schema (°C, mm, km/h),
with missing readings as `null`. Colors are enabled on TTY output and respect
[`NO_COLOR`](https://no-color.org/).

## Data sources

ibuki uses public Open-Meteo APIs:

- [Geocoding](https://open-meteo.com/en/docs/geocoding-api) — city → coordinates
- [Weather Forecast](https://open-meteo.com/en/docs) — current, forecast, precipitation
- [Air Quality](https://open-meteo.com/en/docs/air-quality-api) — AQI and pollutants

Internet access is required. No accounts or API keys.

## Development

```bash
# Run without installing
cargo run -- current Tokyo

# Tests
cargo test

# Lint & format
cargo clippy --all-targets -- -D warnings
cargo fmt
```

Project docs:

- [Specification](docs/tool-cli-weather-spec.md)
- [Implementation plan](docs/tool-cli-weather-plan.md)

## License

[MIT](LICENSE)
