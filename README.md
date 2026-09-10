# FateGrandCalculator

An unofficial Fate/Grand Order damage calculator, built in Rust for both native desktop and web browsers.

FateGrandCalculator is intended to make damage calculations understandable as well as accurate: each result will show the relevant inputs, multipliers, and final damage range.

> This is an unofficial fan project. Fate/Grand Order and its related names, artwork, and assets belong to their respective owners.

## Goals

- One tested Rust calculation engine shared by every client.
- Native desktop application for Windows, macOS, and Linux.
- Browser-accessible version compiled to WebAssembly.
- Transparent, step-by-step damage breakdowns.
- Versioned game data so saved calculations remain reproducible.
- Local-first operation: neither client needs to scrape a wiki at runtime.

## Planned architecture

```text
wiki importer --> versioned game data --> shared Rust core
                                          |          |
                                   desktop client  web client
                                     eframe/egui   WebAssembly
```

The calculation core will have no GUI, network, or platform-specific dependencies. It will accept a complete calculation input and a game-data snapshot, then return a result with all intermediate values.

```rust
pub fn calculate(input: &DamageInput, data: &GameData) -> DamageResult;
```

## Scope

The first release will cover normal-card damage, attack/card buffs, class affinity, attribute affinity, enemy defense, and the random damage range. Noble Phantasms, critical hits, chains, trait-based bonuses, and conditional effects will follow incrementally, each with regression tests.

## Data

Game data is imported offline from publicly available community-wiki pages through their structured API where permitted. The importer caches raw responses, normalizes only the fields required for calculation, and records a source URL, retrieval date, and dataset version for each update.

Please respect the data source's terms, licensing, `robots.txt`, and rate limits. The app never scrapes from users' devices or browsers.

## Project status

Early planning. The first milestone is a small, fully tested damage engine paired with one basic calculator screen and a minimal local data set.

## Development

Run the console example:

```sh
cargo run
```

Run the calculation tests:

```sh
cargo test
```

The standalone damage-engine tests live in `tests/damage_calculation.rs`.

## License

License to be decided before the first public release.
