


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

The calculation core has no GUI, network, or platform-specific dependencies. It accepts the attack inputs and returns each multiplier with the damage range. The desktop client loads the last locally saved servant snapshot, falling back to a bundled seed. Servant data updates happen only when the user chooses **Update servant data**. The selected servant's portrait streams separately from Atlas Academy.

```rust
pub fn calculate(input: DamageInput) -> DamageResult;
```

## Scope

Select a servant from the searchable dropdown below the enlarged portrait. Click attack position 1, 2, or 3, then choose a card tile to replace it. Supported NP tiles appear first; damaging-NP servants initially select NP in position 1. Each physical deck card has its own tile, and a card already used in another position is disabled. The portrait and card controls sit side by side when space allows and stack in smaller windows.

The calculator estimates a selected servant's three ordered attacks against one enemy. Choose distinct command cards from the servant's five-card deck. When supported NP damage data is available, one slot can be an NP and the other two can be normal cards; a servant with a non-damaging NP uses three normal cards. A damaging-NP servant can also choose three normal cards. Each card, and the Extra attack from a Brave Chain, has its own damage range. The app does not add them into a turn total.

Enter buffs and enemy defense as whole percentages: `20` means 20%. Attack and defense are additive in the damage formula. Buster, Arts, and Quick buffs have separate inputs so mixed-color sequences can be calculated. NP level and available NP variants are selectable. The calculation uses the 0.23 attack factor, normal-card positions, first-card effects, class attack rate, class and attribute affinity, NP multiplier and NP damage buff, Buster Chain damage, and Extra attack rules. Each range uses random modifiers from 90.0% through 109.9%. These terms follow [Atlas Academy's damage formula](https://apps.atlasacademy.io/fgo-docs/deeper/battle/damage.html), [card values](https://api.atlasacademy.io/export/JP/NiceCard.json), and [random modifier range](https://apps.atlasacademy.io/fgo-docs/).

Damage shown for an NP is its base estimate, except Space Ereshkigal's NP, which applies the selected affection level's 10% damage bonus per level and ignores enemy defense at level 7 or higher. Level 1 is the normal starting value; level 0 has no affection bonus. Set the affection level at the moment damage lands; an Overcharged NP can raise the gauge before damage, and the app does not calculate that gain automatically. Other conditional trait bonuses, Overcharge effects, automatic NP effects before or after damage, critical hits, skill effects, and enemy HP or retargeting are not simulated. The app labels unsupported or missing NP damage data rather than guessing; those servants can still use three normal cards. The selected servant's command cards represent cards available in the user's current hand. The app does not generate a party-wide hand.

## Data

The bundled seed is in `data/game_data.json`. After an explicit update, the app downloads Atlas Academy's lightweight NA servant export, then fetches the selected gameplay fields for each included servant from the NA nice-servant endpoint. It stores the five-card deck and compact NP damage metadata with the existing attack, class, and attribute data. The complete result is validated and saved in the operating system's application data directory. The snapshot is staged and atomically replaced; an unavailable network, invalid response, or failed write leaves the previous snapshot in place. An older snapshot remains readable and prompts the user to update for card data.

The update includes the ten additional Atlas NA servant classes previously skipped: seven Beast variants, Solomon's lore Grand Caster form, and both Olga Marie collection forms. It reports incomplete basic-export entries and keeps the rest of the valid servant list when no more than one in five rows must be skipped. A malformed export, empty usable result, duplicate servant ID, higher skip ratio, or invalid required deck data rejects the whole update. The data requests have bounded concurrency, retries, timeouts, and response-size limits. NP data can be marked as supported, non-damaging, unavailable, or unsupported without discarding a valid servant deck. Beast IV's special card type has no Buster, Arts, or Quick mapping, so the servant remains listed but turn damage is unavailable.

Atlas Academy recommends its static exports or `/basic` endpoints for indexing; this app uses the [NA basic servant export](https://api.atlasacademy.io/export/NA/basic_servant.json).

The desktop UI requests the selected servant's first ascension portrait from Atlas Academy's `extraAssets.charaGraph.ascension["1"]` URL. It decodes the image in memory and keeps only the currently selected texture. Changing servants releases the previous texture; portraits and portrait URLs are never saved to disk. The portrait window shows a fallback message when the network or artwork is unavailable. No generated images are used.

Please respect the data source's terms, licensing, `robots.txt`, and rate limits. The app uses Atlas Academy's official data and artwork URLs.

## Project status

The desktop milestone supports three-card sequences, base NP damage for supported NP forms, Space Ereshkigal's affection-scaled NP damage, and per-card damage ranges against a selected enemy class and attribute. The additional servant classes use their specific Atlas NA attacker affinity and class attack rates against the fourteen enemy classes in the matchup selector. Other conditional NP effects, critical hits, party-wide card hands, and target HP remain future work.

## Development

Run the console example:

```sh
cargo run
```

Run the calculation tests:

```sh
cargo test
```

Integration tests are grouped by purpose in `tests/`: face-card and turn damage, card selection, NP import, class affinity, attribute affinity, game-data loading, percent input, servant damage, and servant model construction.

## License

License to be decided before the first public release.
