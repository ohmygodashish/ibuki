---
title: ibuki Architecture & Backlog
version: 2.0
date_created: 2026-07-14
last_updated: 2026-08-21
owner: ibuki
tags: [architecture, backlog, rust, cli]
related: tool-cli-weather-spec.md
---

# Architecture & Backlog

How **ibuki** is built today, and what is left to do. Requirements live in
[`tool-cli-weather-spec.md`](./tool-cli-weather-spec.md); this document describes the
implementation that satisfies them.

> Superseded the original phased implementation plan (v1.0, 2026-07-14). Every phase in
> that plan shipped; it was rewritten rather than ticked off because several of its
> instructions had become actively wrong — see [Decisions that reversed the plan](#decisions-that-reversed-the-original-plan).

## Current State

| Item | Status |
|------|--------|
| Application | Complete — four subcommands, human and JSON output |
| Source | ~1,120 lines across 13 files in `src/` |
| Tests | 42 (`cargo test`): 3 unit, 13 API client, 14 CLI, 12 formatting |
| CI | GitHub Actions, Linux/macOS/Windows matrix, clippy + fmt + size gate |
| Release binary | ~3.5 MB against the 5 MB budget (NFR-002) |
| Dependencies | 7 production, 4 dev — no API key, no config file |

## Architecture

```
src/
├── main.rs              # Composition root: parse, resolve location, fetch, format, exit
├── lib.rs               # Module declarations
├── cli.rs               # clap: subcommands, global flags, IBUKI_CITY
├── error.rs             # AppError + shared map_reqwest_error / check_status
├── geocoding/
│   └── mod.rs           # City → Location, plus the on-disk coordinate cache
├── weather/
│   ├── mod.rs           # Shared forecast-API client and base URL
│   ├── current.rs       # Current conditions
│   ├── forecast.rs      # Daily forecast
│   └── radar.rs         # Precipitation metrics
├── air_quality/
│   └── mod.rs           # AQI + pollutants
├── models/
│   └── mod.rs           # Location, per-command models, JSON response types
├── format.rs            # All human-readable rendering (one file, four free functions)
└── units.rs             # °C/°F, wind labels, WMO code → description
tests/
├── fixtures/            # Six Open-Meteo sample payloads
├── api_clients.rs       # Clients against mockito
├── cli.rs               # assert_cmd end-to-end with mocked HTTP
└── format.rs            # Rendering and JSON schema
```

### Request flow

```
Cli::parse
  └─ resolve_location            --lat/--lon → Location directly (no network)
       └─ GeocodingClient        else: cache hit → Location
            └─ Open-Meteo        else: HTTP, then write cache
  └─ WeatherClient | AirQualityClient
  └─ format::* (human) | serde_json::to_string_pretty (--json)
  └─ stdout, exit 0 / 1
```

### Patterns in force

- **Plain structs, no traits.** Each API gets a client struct sharing one `reqwest`
  client. There is no repository trait and no formatter trait — a trait with a single
  implementation was removed as unearned indirection.
- **Base-URL injection is the test seam.** `with_base_url` in-process, or
  `IBUKI_GEOCODING_URL` / `IBUKI_FORECAST_URL` / `IBUKI_AIR_QUALITY_URL` for end-to-end
  tests. No test touches the network.
- **Metric internally, converted at the edge.** Models store °C; `--fahrenheit` affects
  human rendering only, so one JSON schema serves all four subcommands.
- **A missing reading is `None`, never `0`** — `n/a` in human output, `null` in JSON.
- **Measure, then colourise.** `format.rs` lays out every line as plain text, measures
  width, pads, and applies colour last, so no width calculation ever sees an escape code.

## Dependencies

| Crate | Role |
|-------|------|
| `clap` (derive, env) | CLI parsing, help, version, `IBUKI_CITY` |
| `reqwest` (blocking, json, rustls-tls) | HTTP client |
| `serde` / `serde_json` | Deserialize APIs, serialize `--json`, the geocoding cache |
| `thiserror` | Typed domain errors (`AppError`) |
| `anyhow` | Top-level error reporting in `main` |
| `owo-colors` | TTY colours |

TTY detection uses `std::io::IsTerminal` from the standard library, not a crate.
`rustls-tls` is preferred over native-tls for cross-platform builds and binary size
(NFR-002); it resolves to `ring`, which needs no NASM on Windows.

**Dev:** `assert_cmd` (CLI execution), `predicates` (output assertions),
`mockito` (HTTP mocking), `serde_json` (schema assertions).

## Decisions that reversed the original plan

| Original plan said | What shipped, and why |
|---|---|
| Repository trait for HTTP clients; presenter trait in `format/` | Both deleted. One implementation per trait is indirection with no payoff; `src/format/{mod,human,json}.rs` collapsed into a single `src/format.rs` |
| "Always include both `temperature_c` and `temperature_f`" in JSON | Reversed. JSON is metric-only (REQ-009); the `_f` twins were removed so one schema covers every subcommand |
| Timestamps labelled UTC | Wrong by up to a day. `timezone=auto` returns local time; output now carries the API's own abbreviation (REQ-014) |
| Out of scope: coordinate location | Shipped as `--lat`/`--lon` (REQ-016) |
| Out of scope: caching | Shipped as a best-effort geocoding cache (REQ-019) |
| `is-terminal`, `insta`, `wiremock` crates | Not used. `std::io::IsTerminal`, plain assertions, and `mockito` respectively |
| macOS/Windows CI "optional later" | Shipped as a three-OS matrix |

## Backlog

Nothing is in progress. Remaining items, roughly by value:

- [ ] **Coverage is unmeasured.** The spec asks for ≥80% on core logic and 100% on error
      paths; no tooling is wired into CI to confirm either. `cargo-llvm-cov` is the
      usual answer.
- [ ] **Double-width city names overhang the box.** Widths are counted in `char`s, so
      `東京` renders a ragged right border. `unicode-width` is the upgrade path.
      Marked `ponytail:` in `src/format.rs:119`.
- [ ] **The geocoding cache grows without bound.** No eviction, by design — a city's
      coordinates never change. Add eviction only if a real cache gets large.
      Marked `ponytail:` in `src/geocoding/mod.rs:129`.
- [ ] **Ambiguity is still resolved silently.** `count=1` means "Portland" picks one
      match; `admin1` now makes the pick *visible* but offers no way to choose the other.
      A `--region` filter or an interactive picker would close it.
- [ ] **No release artifacts.** CI builds and tests but publishes nothing; there is no
      tagged binary for the three platforms it verifies.

### Explicitly out of scope

- IP-based location — needs a third-party service and conflicts with CON-004
- Weather caching — the readings change; only coordinates are stable enough to cache
- Config files or API keys — Open-Meteo needs neither (CON-001, CON-003)
- Interactive TUI, async runtime — blocking HTTP is sufficient for a one-shot CLI

## Verification

```bash
cargo test                                   # 42 tests, no network access
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo build --release && stat -c%s target/release/ibuki   # < 5,242,880

# Manual smoke
ibuki current Tokyo
ibuki forecast London --days 5 --fahrenheit
ibuki current --lat 45.5234 --lon -122.6762
NO_COLOR=1 ibuki air-quality Paris
```

## References

- Spec: [tool-cli-weather-spec.md](./tool-cli-weather-spec.md)
- [Open-Meteo Forecast](https://open-meteo.com/en/docs)
- [Open-Meteo Geocoding](https://open-meteo.com/en/docs/geocoding-api)
- [Open-Meteo Air Quality](https://open-meteo.com/en/docs/air-quality-api)
- [NO_COLOR](https://no-color.org/)
- [Rust CLI recommendations](https://rust-cli-recommendations.sunshowers.io/)
