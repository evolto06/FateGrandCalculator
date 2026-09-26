


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

## Architecture

```text
wiki importer --> versioned game data --> shared Rust core
                                          |          |
                                   desktop client  web client
                                     eframe/egui   WebAssembly
```

The calculation core has no GUI, network, or platform-specific dependencies. It accepts the attack inputs and returns each multiplier with the damage range. The desktop client loads a bundled, versioned servant snapshot; it does not contact the data source at runtime.

```rust
pub fn calculate(input: DamageInput) -> DamageResult;
```

## Scope

The first release will cover normal-card damage, attack/card buffs, class affinity, attribute affinity, enemy defense, and the random damage range. Noble Phantasms, critical hits, chains, trait-based bonuses, and conditional effects will follow incrementally, each with regression tests.

The current calculator estimates one non-critical card in the first command-card position. Enter buffs and enemy defense as whole percentages: `20` means 20%. The calculation includes the 0.23 attack factor, first-card bonus, class attack rate, class and attribute affinity, and the combined attack/defense modifier. Its random range uses 90.0% through 109.9%.

Card chains, later card positions, Noble Phantasms, and special damage effects are outside this screen's current scope. These terms follow [Atlas Academy's damage formula](https://apps.atlasacademy.io/fgo-docs/deeper/battle/damage.html), [card values](https://api.atlasacademy.io/export/JP/NiceCard.json), and [random modifier range](https://apps.atlasacademy.io/fgo-docs/).

## Data

The current bundled snapshot is in `data/game_data.json`. It contains a small NA servant list with max-level attack, class, and attribute, along with a version, source, and retrieval date. Class and attribute affinity tables are based on Atlas Academy's published FGO game data. A future offline importer can update this normalized snapshot.

Please respect the data source's terms, licensing, `robots.txt`, and rate limits. The app never scrapes from users' devices or browsers.

## Project status

The first desktop milestone supports normal-card damage for selected servants against selectable enemy classes and attributes. Noble Phantasms, critical hits, chains, traits, and a data importer remain future work.

## Development

Run the console example:

```sh
cargo run
```

Run the calculation tests:

```sh
cargo test
```

Integration tests are grouped by purpose in `tests/`: face-card damage, class affinity, attribute affinity, game-data loading, percent input, servant damage, and servant model construction.

## License

License to be decided before the first public release.
