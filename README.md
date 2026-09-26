


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
Atlas Academy export -- user-triggered import --> local servant snapshot
                                                   |                 |
                                            desktop client      web client
                                              eframe/egui        WebAssembly
```

The calculation core has no GUI, network, or platform-specific dependencies. It accepts the attack inputs and returns each multiplier with the damage range. The desktop client loads the last locally saved servant snapshot, falling back to a bundled seed. Servant data updates happen only when the user chooses **Update servant data**.

```rust
pub fn calculate(input: DamageInput) -> DamageResult;
```

## Scope

The first release will cover normal-card damage, attack/card buffs, class affinity, attribute affinity, enemy defense, and the random damage range. Noble Phantasms, critical hits, chains, trait-based bonuses, and conditional effects will follow incrementally, each with regression tests.

The current calculator estimates one non-critical card in the first command-card position. Enter buffs and enemy defense as whole percentages: `20` means 20%. The calculation includes the 0.23 attack factor, first-card bonus, class attack rate, class and attribute affinity, and the combined attack/defense modifier. Its random range uses 90.0% through 109.9%.

Card chains, later card positions, Noble Phantasms, and special damage effects are outside this screen's current scope. These terms follow [Atlas Academy's damage formula](https://apps.atlasacademy.io/fgo-docs/deeper/battle/damage.html), [card values](https://api.atlasacademy.io/export/JP/NiceCard.json), and [random modifier range](https://apps.atlasacademy.io/fgo-docs/).

## Data

The bundled seed is in `data/game_data.json`. After an explicit update, the app downloads Atlas Academy's lightweight NA servant export, keeps only the fields used by the calculator, maps Atlas attributes and class names to the local model, and validates the complete result before saving it in the operating system's application data directory. The snapshot is staged and atomically replaced; an unavailable network, invalid response, or failed write leaves the previous snapshot in place.

The update reports unsupported or incomplete entries and keeps the rest of the valid playable servant list when no more than one in five rows must be skipped. A malformed export, empty usable result, duplicate servant ID, or higher skip ratio rejects the whole update. The data request has bounded retries, timeouts, and a response-size limit.

Atlas Academy recommends its static exports or `/basic` endpoints for indexing; this app uses the [NA basic servant export](https://api.atlasacademy.io/export/NA/basic_servant.json).

Please respect the data source's terms, licensing, `robots.txt`, and rate limits. The app uses Atlas Academy's official data URLs.

## Project status

The first desktop milestone supports normal-card damage for selected servants against selectable enemy classes and attributes. Noble Phantasms, critical hits, chains, and traits remain future work.

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
