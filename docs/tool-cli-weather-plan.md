---
title: ibuki Implementation Plan
version: 1.0
date_created: 2026-07-14
last_updated: 2026-07-14
owner: ibuki
tags: [implementation, plan, rust, cli]
related: tool-cli-weather-spec.md
---

# Implementation Plan

Phased plan to implement **ibuki** against `docs/tool-cli-weather-spec.md`. Goal: ship a working CLI with all four subcommands, JSON/human output, solid error handling, and tests.

## Current State

| Item | Status |
|------|--------|
| Repo / Cargo package | Exists (`edition = "2024"`) |
| Spec | Complete (`docs/tool-cli-weather-spec.md`) |
| Application code | Stub only (`Hello, world!`) |
| Dependencies | None |
| Tests / CI | None |

## Target Architecture

```
src/
├── main.rs              # Entry: parse CLI, run command, exit codes
├── cli.rs               # clap: subcommands, flags, validation
├── error.rs             # AppError + thiserror / anyhow boundaries
├── geocoding/
│   └── mod.rs           # City → Location (Open-Meteo Geocoding)
├── weather/
│   ├── mod.rs           # Weather client trait + Open-Meteo impl
│   ├── current.rs       # Current conditions fetch + model
│   ├── forecast.rs      # Daily forecast fetch + model
│   └── radar.rs         # Precipitation / radar metrics
├── air_quality/
│   └── mod.rs           # AQI client + model
├── models/
│   └── mod.rs           # Shared Location, WMO helpers, JSON DTOs
├── format/
│   ├── mod.rs           # Presenter trait
│   ├── human.rs         # Colored TTY tables / boxes
│   └── json.rs          # serde_json schemas from spec
└── units.rs             # °C/°F conversion, wind direction labels
tests/
├── fixtures/            # Sample Open-Meteo JSON responses
├── cli.rs               # assert_cmd E2E with mocked HTTP
└── ...
```

**Patterns (from spec):**
- Repository trait for HTTP clients (mockable)
- Separate fetch → transform → present
- No panics on user input; all errors mapped to exit code 1 + message

## Dependencies

| Crate | Role |
|-------|------|
| `clap` (derive) | CLI parsing, help, version |
| `reqwest` (blocking, json, rustls-tls) | HTTP client (sync CLI is fine for v1) |
| `serde` / `serde_json` | Deserialize APIs, serialize `--json` |
| `thiserror` | Typed domain errors |
| `anyhow` | Top-level error reporting in `main` |
| `owo-colors` or `colored` | TTY colors (respect `NO_COLOR`) |
| `is-terminal` | Detect stdout TTY for color |

**Dev / test:**
| Crate | Role |
|-------|------|
| `assert_cmd` | CLI integration tests |
| `predicates` | Output assertions |
| `mockito` or `wiremock` | HTTP mocking |
| `insta` | Snapshot human output (optional phase 5) |

Prefer `rustls-tls` over native-tls to keep cross-platform builds simple and binary size down (NFR-002).

## Phases

### Phase 0 — Project scaffolding

**Goal:** Compilable skeleton with module layout and deps; no real API calls yet.

- [ ] Add production and dev dependencies to `Cargo.toml`
- [ ] Create module tree (`cli`, `error`, `models`, `geocoding`, `weather`, `air_quality`, `format`, `units`)
- [ ] Wire `main.rs` to parse CLI and print a placeholder per subcommand
- [ ] Implement clap structure matching spec interface:
  - Subcommands: `current`, `forecast`, `radar`, `air-quality`
  - Positional `<CITY>`
  - Flags: `--json`, `--fahrenheit`, `--days` (forecast only, 1–16)
- [ ] Validate `--days` range in clap (`value_parser` 1..=16) → ERR for AC-010

**Exit criteria:** `cargo build` succeeds; `ibuki --help` and subcommand help render correctly.

---

### Phase 1 — Core models, errors, geocoding

**Goal:** Resolve city names to coordinates; shared types ready.

- [ ] Define `Location { name, country, latitude, longitude }`
- [ ] Define `AppError` variants: `CityNotFound`, `Network`, `Timeout`, `Api`, `EmptyData`, `InvalidInput`
- [ ] Map errors to user-facing messages (ERR-001–004, edge-case table in spec §9)
- [ ] Implement `GeocodingClient` trait + Open-Meteo impl
  - Endpoint: `https://geocoding-api.open-meteo.com/v1/search?name=…&count=1&language=en&format=json`
  - Use first result; empty results → `CityNotFound`
- [ ] HTTP timeout 10s (NFR-001)
- [ ] Unit tests: parse fixture responses; empty results; malformed JSON

**Exit criteria:** Given a fixture or live call, city resolves to lat/lon; unknown city yields clear error + exit 1.

---

### Phase 2 — Weather data (current, forecast, radar)

**Goal:** Fetch and model all weather-related payloads from Forecast API.

- [ ] Shared Open-Meteo forecast client (lat/lon + query params)
- [ ] **Current** (`REQ-001`, `REQ-008`):
  - Request current: temp, apparent temp, humidity, wind speed/dir, weather code, time
  - Map WMO code → description string
- [ ] **Forecast** (`REQ-002`, `REQ-012`):
  - Daily: max/min temp, precip sum, precip probability, max wind, weather code
  - Honor `--days` (1–16)
- [ ] **Radar** (`REQ-003`):
  - Current/hourly precip: last hour, next hour, rain/snow/showers intensity, weather code
- [ ] Domain models matching JSON schemas in spec §4 (always store °C internally; convert at present)
- [ ] Unit tests with fixtures under `tests/fixtures/`

**Exit criteria:** Each weather path returns structured models from fixtures; live smoke test optional.

---

### Phase 3 — Air quality

**Goal:** Air quality subcommand complete.

- [ ] Client for `https://air-quality-api.open-meteo.com/v1/air-quality`
- [ ] Current: US AQI, EU AQI, PM2.5, PM10, O₃, NO₂, CO
- [ ] Model matching `air-quality` JSON schema
- [ ] Fixture + unit tests

**Exit criteria:** `air-quality` path returns full pollutant set from fixture.

---

### Phase 4 — Presentation layer

**Goal:** Human and JSON output per acceptance criteria.

- [ ] **JSON** (`REQ-006`, AC-005):
  - Serialize models exactly to schemas in §4
  - Always include both `temperature_c` and `temperature_f` in current/forecast JSON (spec examples include both)
- [ ] **Human** (`REQ-010`, `REQ-011`, AC-009):
  - Box/table style as in §9 example
  - Colors when TTY and `NO_COLOR` unset
  - No ANSI when non-TTY or `NO_COLOR` set
  - Wind direction degrees → N/NE/E/… labels
  - `--fahrenheit` switches displayed temps only (AC-008)
- [ ] Consistent layout across all four subcommands (GUD-002, GUD-003)

**Exit criteria:** Snapshot or manual check for human output; JSON validates against documented shapes.

---

### Phase 5 — Wiring, polish, binary quality

**Goal:** End-to-end CLI, robust errors, size/perf checks.

- [ ] Orchestrate in `main` / command handlers:
  1. Parse args
  2. Geocode city
  3. Call appropriate client
  4. Format (human | json)
  5. Exit 0 / 1 (NFR-004)
- [ ] Ensure no panics on bad input (ERR-004)
- [ ] Messages for: city not found, timeout, network, rate limit if detectable, empty API data, invalid days (clap)
- [ ] Release profile in `Cargo.toml`: `lto`, `codegen-units = 1`, `strip = true` for size (NFR-002)
- [ ] Manual smoke:
  - `ibuki current Tokyo`
  - `ibuki forecast London --days 3`
  - `ibuki radar "New York"`
  - `ibuki air-quality Paris --json`
  - Invalid city / offline behavior

**Exit criteria:** All AC-001–AC-010 pass manually; binary &lt; 5 MB release; cold path feels &lt; 2s on normal network.

---

### Phase 6 — Tests and CI

**Goal:** Automated confidence matching §6 of the spec.

- [ ] Unit: units, WMO mapping, formatters, error messages
- [ ] Integration: HTTP clients against mockito/wiremock + fixtures
- [ ] CLI: `assert_cmd` for help, validation (`--days 20`), mocked happy paths
- [ ] GitHub Actions workflow:
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo test`
- [ ] Aim for ≥80% coverage on core logic; prioritize error paths

**Exit criteria:** `cargo test`, clippy, fmt clean; CI green on PR.

---

## Suggested Implementation Order (day-to-day)

1. Phase 0 scaffolding + clap  
2. Phase 1 geocoding + errors  
3. Phase 2 `current` only → human + JSON (vertical slice)  
4. Phase 2 `forecast` + `radar`  
5. Phase 3 `air-quality`  
6. Phase 4 polish formatting / colors  
7. Phase 5 release profile + smoke  
8. Phase 6 tests + CI  

Ship a **vertical slice early** (`current` + geocode + human/JSON) so the loop (build → run → fix) is short before filling remaining subcommands.

## Module Responsibilities (quick ref)

| Module | Owns |
|--------|------|
| `cli` | Args only; no I/O |
| `geocoding` | Name → `Location` |
| `weather` | Forecast API access + domain models |
| `air_quality` | Air quality API access + models |
| `format` | stdout presentation only |
| `error` | Error types + Display strings |
| `units` | Conversions and pure helpers |
| `main` | Composition root, process exit |

## Out of Scope (v1)

- Coordinate or IP-based location (spec §7: future)
- Caching / offline mode
- Config files or API keys
- Interactive TUI
- Async runtime (blocking HTTP is acceptable for v1 CLI)

## Tracking Checklist (spec validation §10)

- [ ] All four subcommands produce correct output  
- [ ] `--json` matches schemas  
- [ ] `--fahrenheit` converts correctly  
- [ ] `--days` 1–16; out of range errors  
- [ ] Invalid city / network errors graceful  
- [ ] Linux build (macOS/Windows CI matrix optional later)  
- [ ] Tests ≥80% core; clippy/fmt clean  
- [ ] Release binary &lt; 5 MB  
- [ ] Cold-start to output &lt; 2 s under normal network  

## References

- Spec: [tool-cli-weather-spec.md](./tool-cli-weather-spec.md)
- [Open-Meteo Forecast](https://open-meteo.com/en/docs)
- [Open-Meteo Geocoding](https://open-meteo.com/en/docs/geocoding-api)
- [Open-Meteo Air Quality](https://open-meteo.com/en/docs/air-quality-api)
- [NO_COLOR](https://no-color.org/)
- [Rust CLI recommendations](https://rust-cli-recommendations.sunshowers.io/)
