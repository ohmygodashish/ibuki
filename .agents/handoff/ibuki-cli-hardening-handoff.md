# Handoff — ibuki CLI hardening

- **Timestamp**: 2026-08-19
- **Repo**: `/home/ashish/.repositories/ibuki` (branch `main`, working tree clean)
- **Session focus**: Full read-through of the app, then implementation of sections A (bugs) and B (over-engineering deletions) from the review plan.
- **Next session goal**: Implement the deferred Section C items — starting with C4 (surface error cause chains, one line), then C1 (`admin1` in geocoding output) and C2 (geocoding cache).

## Executive summary

`ibuki` is a Rust CLI over the free Open-Meteo APIs (no key required) with four subcommands —
`current`, `forecast`, `radar`, `air-quality` — plus `--json`, `--fahrenheit`, and `--days`.
The flow is linear and lives in [`src/main.rs`](file:///home/ashish/.repositories/ibuki/src/main.rs):
`Cli::parse` → `GeocodingClient::resolve(city)` → a weather/air-quality client → formatter → stdout.

This session produced a written review (plan file below) and then implemented the agreed scope:
three correctness bugs (A1–A3) and five over-engineering deletions (B1–B5). Net result is
**-594/+517 lines** across 19 files — the codebase got smaller while gaining behaviour.
The headline fix: every request sends `timezone=auto`, so timestamps come back local to the
queried location, but the code appended `:00Z` and printed `UTC` — Tokyo's 03:15 local was
reported as 03:15 UTC, a 9-hour error that also shipped in the README sample.

State at handoff: 31 tests pass, `cargo clippy --all-targets -- -D warnings` clean,
`cargo fmt --check` clean, release binary 3.4 MB against the 5 MB CI gate. Seven commits
landed on `main` (`8728b0a`..`fedfe76`), all authored by the user. Nothing is in progress.

## Modified files & component changes

Full diff: `git diff fc82e6e..HEAD`. Seven commits, `8728b0a` → `fedfe76`.

| File | Change |
|---|---|
| [`src/error.rs`](file:///home/ashish/.repositories/ibuki/src/error.rs) | Gained the shared `map_reqwest_error` and `check_status(response, api)` helpers (B2) |
| [`src/cli.rs`](file:///home/ashish/.repositories/ibuki/src/cli.rs) | `--json`/`--fahrenheit` became `global = true` on `Cli`; `json()`/`fahrenheit()` accessors deleted, `city()` kept (B3) |
| [`src/format.rs`](file:///home/ashish/.repositories/ibuki/src/format.rs) | **New** — replaces the whole `src/format/` directory (`mod.rs` + `human.rs` + `json.rs`). Four free functions, plain-text-first box renderer (B1, B4, B5) |
| `src/format/{mod,human,json}.rs` | **Deleted** — trait, factory, and the JSON impl all removed |
| [`src/main.rs`](file:///home/ashish/.repositories/ibuki/src/main.rs) | Branches on `cli.json` directly with `serde_json::to_string_pretty`; local `to_json` helper (B1) |
| [`src/models/mod.rs`](file:///home/ashish/.repositories/ibuki/src/models/mod.rs) | Optional fields for omittable readings; `timezone` added to `CurrentWeather`/`AirQuality`; `temperature_f`/`feels_like_f` dropped (A1–A3) |
| [`src/weather/current.rs`](file:///home/ashish/.repositories/ibuki/src/weather/current.rs), [`radar.rs`](file:///home/ashish/.repositories/ibuki/src/weather/radar.rs), [`forecast.rs`](file:///home/ashish/.repositories/ibuki/src/weather/forecast.rs) | `:00Z` mangling removed, `timezone_abbreviation` deserialized, `unwrap_or(0.0)` replaced with `Option` passthrough, shared error helpers |
| [`src/weather/mod.rs`](file:///home/ashish/.repositories/ibuki/src/weather/mod.rs), [`src/geocoding/mod.rs`](file:///home/ashish/.repositories/ibuki/src/geocoding/mod.rs), [`src/air_quality/mod.rs`](file:///home/ashish/.repositories/ibuki/src/air_quality/mod.rs) | Local error-mapping/status-check copies deleted in favour of `error.rs` |
| [`tests/format.rs`](file:///home/ashish/.repositories/ibuki/tests/format.rs) | Rewritten for the free-function API; new tests for tz labelling, `n/a`/`null`, box alignment |
| [`tests/api_clients.rs`](file:///home/ashish/.repositories/ibuki/tests/api_clients.rs) | Updated for `Option` fields; new `air_quality_missing_pollutants_stay_none` |
| [`tests/fixtures/{current_weather,air_quality}.json`](file:///home/ashish/.repositories/ibuki/tests/fixtures/) | `"timezone_abbreviation": "JST"` added |
| [`README.md`](file:///home/ashish/.repositories/ibuki/README.md) | Sample output regenerated; `--json`/`--fahrenheit` semantics documented |

## Architectural decisions & technical context

Review and agreed scope: [`~/.claude/plans/gleaming-marinating-starlight.md`](file:///home/ashish/.claude/plans/gleaming-marinating-starlight.md).
Project spec and plan: [`docs/tool-cli-weather-spec.md`](file:///home/ashish/.repositories/ibuki/docs/tool-cli-weather-spec.md),
[`docs/tool-cli-weather-plan.md`](file:///home/ashish/.repositories/ibuki/docs/tool-cli-weather-plan.md).

1. **JSON is metric-only, and that is the contract.** `--fahrenheit` converts at display time
   only (as `forecast` always did). Twin `_c`/`_f` fields were removed from `CurrentWeather`
   rather than added to `ForecastDay`, so one schema covers all four commands. Note this
   contradicts a literal reading of REQ-009 in the spec — the spec predates the decision and
   should be amended if it is treated as authoritative.
2. **A missing reading is `None`, never `0`.** Absent values render `n/a` in human output and
   `null` in JSON. Deliberate exception: `feels_like_c` still falls back to `temperature_c`,
   and radar's `precipitation_last_hour_mm` falls back to `rain + showers` when either part is
   known (`sum_opt` in `radar.rs`) — both are defensible derivations, not invented data.
3. **Timestamps are local, labelled with the API's own `timezone_abbreviation`** (real
   responses say `GMT+9`, not `JST`; fixtures use `JST`). No suffix at all when the field is
   absent — never a hardcoded `UTC`.
4. **Colour is applied after padding.** The box renderer builds every line as plain text,
   measures, pads, then colourises — which is why the hand-rolled ANSI stripper could be
   deleted outright rather than fixed.
5. **No new dependencies.** `Cargo.toml` is untouched this session.

## Open tasks, blockers & pitfalls

Nothing is blocked or in progress. Remaining work, all optional (Section C of the plan):

- [ ] **C4 — error causes are dropped.** `main.rs:16` prints `{err}` only, so the `#[source]`
      chains on `AppError::Network`/`Parse` never surface; "Network request failed" never says
      why. One line. Do this first.
- [ ] **C1 — ambiguous cities.** Geocoding uses `count=1`, so "Portland"/"Springfield" silently
      resolve to one arbitrary match. Open-Meteo returns `admin1`; adding it to `Location` and
      the header line makes the pick visible. ~3 lines.
- [ ] **C2 — cache geocoding.** City → lat/lon never changes, yet every invocation pays a round
      trip. A JSON map under `$XDG_CACHE_HOME/ibuki/` is ~15 lines using `serde_json` (already a
      dependency) and roughly halves latency.
- [ ] **C3 — `--lat`/`--lon`** to skip geocoding; `IBUKI_CITY` for a default city.
- [ ] **C5 — CI is ubuntu-only** while the spec claims cross-platform support.

Pitfalls for the incoming agent:

- **Box widths count `char`s, not display columns.** Double-width CJK city names (`東京`)
  overhang the right border. Marked with a `ponytail:` comment in `src/format.rs`;
  `unicode-width` is the upgrade path if it matters.
- **`--fahrenheit` is global**, so `radar` and `air-quality` still accept it and correctly
  ignore it. That is clap-idiomatic, not a leftover bug — don't "fix" it by re-splitting the flags.
- **Fixtures carry `timezone_abbreviation: "JST"`** but live Open-Meteo returns `GMT+9`. Tests
  asserting a specific abbreviation must use the fixtures, not the network.
- **Tests never hit the network.** `IBUKI_GEOCODING_URL` / `IBUKI_FORECAST_URL` /
  `IBUKI_AIR_QUALITY_URL` redirect each client to a `mockito` server. Preserve that seam.
- **Colour is off when stdout is not a TTY**, so formatter assertions see plain text; add
  `NO_COLOR=1` for CLI-level tests that could run under a pty.

## Recommended agent skills & tools

1. `ponytail:ponytail` — the working mode for this repo all session; the codebase is now tuned
   to it (smallest diff that works, stdlib before dependencies, deletion over addition).
2. `superpowers:test-driven-development` — for C1/C2, which add behaviour rather than remove it.
3. `/code-review` — the diff is already reviewed, but useful before merging any Section C work.
4. Verification commands (all currently green): `cargo test`,
   `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, and the CI size gate
   `cargo build --release && stat -c%s target/release/ibuki` (3,407,072 bytes vs 5,242,880 limit).
   Manual smoke: `cargo run -- current Tokyo`, `... --json`,
   `cargo run -- forecast London --days 5 --fahrenheit`, `NO_COLOR=1 cargo run -- air-quality Paris`.
