


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

The calculation core has no GUI, network, or platform-specific dependencies. It accepts the attack inputs and returns each multiplier with the damage range. The desktop client loads the last locally saved servant snapshot, falling back to a bundled seed. Servant data updates happen only when the user chooses **Update servant data**. The selected servant's portrait streams separately from Atlas Academy. The web client runs the same UI in the browser with the bundled seed only; it has no servant data updates or portrait streaming.

```rust
pub fn calculate(input: DamageInput) -> DamageResult;
```

## Scope

Select a servant from the dropdown below the enlarged portrait; search by name or class. Click attack position 1, 2, or 3, then choose a card tile to replace it. Supported NP tiles appear first and are labelled with their variant (for example "Base" or "Upgraded"); damaging-NP servants initially select NP in position 1. Each physical deck card has its own tile, and a card already used in another position is disabled and marked "In position N". Enemy class, attribute and defense are set together in the Enemy panel. In wide windows the results start with a turn summary; in narrow windows a summary stays pinned to the bottom of the screen. The portrait and card controls sit side by side when space allows and stack in smaller windows.

The calculator estimates a selected servant's three ordered attacks against one enemy. Choose distinct command cards from the servant's five-card deck. When supported NP damage data is available, one slot can be an NP and the other two can be normal cards; a servant with a non-damaging NP uses three normal cards. A damaging-NP servant can also choose three normal cards. Each card, and the Extra attack from a Brave Chain, has its own damage range. The app does not add them into a turn total.

Enter buffs and enemy defense as whole percentages: `20` means 20%. Attack and defense are additive in the damage formula. Buster, Arts, and Quick buffs have separate inputs so mixed-color sequences can be calculated. NP level and Overcharge are selected independently; only levels with explicit data for the chosen NP are available. The calculation uses the 0.23 attack factor, normal-card positions, first-card effects, class attack rate, class and attribute affinity, NP multiplier and NP damage buff, Buster Chain damage, and Extra attack rules. Each range uses random modifiers from 90.0% through 109.9%. These terms follow [Atlas Academy's damage formula](https://apps.atlasacademy.io/fgo-docs/deeper/battle/damage.html), [card values](https://api.atlasacademy.io/export/JP/NiceCard.json), and [random modifier range](https://apps.atlasacademy.io/fgo-docs/).

Visible percentage inputs accept finite values from `−100` to `999`; a blank field, or a lone sign or decimal point while typing, means `0`. Invalid text or an out-of-range value pauses the damage results and sticky summary until the inputs are valid again. NP damage buff values up to `999%` can be entered, but NP damage uses Atlas Academy's `+500%` buff cap. The calculation breakdown lists the formula terms in order. It adds the Buster Chain flat addition after random variance only for cards that receive it, and the Extra attack factor (`×2.00` or `×3.50` for a same-color chain) only for the Extra attack, following [Atlas Academy's buff bounds](https://apps.atlasacademy.io/fgo-docs/#lower-and-upper-bounds-of-buffs).

NP mechanic handling is grouped in `src/np_mechanics`. Atlas Academy's defense-piercing NP type ignores positive enemy defense while retaining Defense Down; NP piercing does not affect normal cards or Extra attacks. Space Ereshkigal's NP gains 10% damage per affection level (level 1 is the normal starting value; level 0 has no bonus) and ignores enemy defense at level 7 or higher. Set her affection level at the moment damage lands; Overcharge does not change it automatically.

The calculator imports multi-component damage for Arash's base and upgraded NPs, Chen Gong, and Bhima. It uses explicitly imported rates and multipliers, shows the component breakdown, and floors each component's damage endpoints before summing them against the selected enemy. NP level and Overcharge remain independent inputs. The estimate does not simulate the enemy's HP, death and retargeting between components, ally sacrifice, or automatic buff application and removal. Overcharge values are never inferred: older snapshots only establish OC1, and the app prompts you to update servant data when values are missing. Audited `damageNpIndividual` variants apply their conditional trait correction only when you manually assume the enemy has the listed trait at NP damage time; see [trait-targeted NP support](docs/trait-np-bonuses.md). Six audited low-HP NP variants use manually entered attacker HP at the moment NP damage lands; see [low-HP NP support](docs/low-hp-nps.md). Seven audited NPs also support a manually selected enemy-state condition at damage time; see [enemy-status NP support](docs/enemy-status-nps.md). Other conditional mechanics—including high-HP scaling, stack bonuses, other target states, rarity, and compound conditions—remain unsupported and are not guessed. Critical hits, skill effects, and party-wide card hands are not simulated. The app labels unsupported or missing NP damage data; those servants can still use three normal cards. The selected servant's command cards represent cards available in the user's current hand. The app does not generate a party-wide hand.

## Data

The bundled seed is in `data/game_data.json`. Snapshot schema v6 stores ordered NP damage components, explicitly available Overcharge values, low-HP scaling, enemy-status metadata, and audited trait corrections. The app reads only the three newest snapshot formats (currently v4–v6); a snapshot in an older format is ignored and the bundled seed is used until you choose **Update servant data**. Older NP data that establishes only OC1 stays that way until you update servant data. After an explicit update, the app downloads Atlas Academy's lightweight NA servant export, then fetches the selected gameplay fields for each included servant from the NA nice-servant endpoint. It stores the five-card deck and NP damage metadata, including defense piercing, supported component data, optional maximum HP, and audited status and trait corrections, with the existing attack, class, and attribute data. The complete result is validated and saved in the operating system's application data directory. The snapshot is staged and atomically replaced; an unavailable network, invalid response, or failed write leaves the previous snapshot in place. If you already have a saved catalog, choose **Update servant data** to import newly supported NP cards and metadata.

The update includes the ten additional Atlas NA servant classes previously skipped: seven Beast variants, Solomon's lore Grand Caster form, and both Olga Marie collection forms. It reports incomplete basic-export entries and keeps the rest of the valid servant list when no more than one in five rows must be skipped. A malformed export, empty usable result, duplicate servant ID, higher skip ratio, or invalid required deck data rejects the whole update. The data requests have bounded concurrency, retries, timeouts, and response-size limits. NP data can be marked as supported, non-damaging, unavailable, or unsupported without discarding a valid servant deck. Beast IV's special card type has no Buster, Arts, or Quick mapping, so the servant remains listed but turn damage is unavailable.

Atlas Academy recommends its static exports or `/basic` endpoints for indexing; this app uses the [NA basic servant export](https://api.atlasacademy.io/export/NA/basic_servant.json).

The desktop UI requests the selected servant's first ascension portrait from Atlas Academy's `extraAssets.charaGraph.ascension["1"]` URL. It decodes the image in memory and keeps only the currently selected texture. Changing servants releases the previous texture; portraits and portrait URLs are never saved to disk. The portrait window shows a fallback message when the network or artwork is unavailable. No generated images are used.

Please respect the data source's terms, licensing, `robots.txt`, and rate limits. The app uses Atlas Academy's official data and artwork URLs.

## Project status

The desktop milestone supports three-card sequences, explicitly imported NP-level and Overcharge damage values for supported NPs, defense-piercing NPs, Space Ereshkigal's affection-scaled NP damage, audited trait-targeted NP bonuses, six audited low-HP NP variants with manual attacker HP at damage time, seven audited enemy-status NP variants with a manual status assumption at damage time, and per-card damage ranges against a selected enemy class and attribute. Multi-component NP profiles are currently limited to Arash (base and upgraded), Chen Gong, and Bhima. The additional servant classes use their specific Atlas NA attacker affinity and class attack rates against the fourteen enemy classes in the Enemy panel. Other conditional NP effects, high-HP scaling, critical hits, party-wide card hands, enemy HP, ally sacrifice, and automatic buff effects remain future work.

## Development

Run the desktop app:

```sh
cargo run
```

Run the web client (needs the `wasm32-unknown-unknown` target and [Trunk](https://trunkrs.dev/), installed with `cargo install trunk`):

```sh
trunk serve            # development server at http://127.0.0.1:8080
trunk build --release  # static files in dist/
```

Run the calculation tests:

```sh
cargo test
```

Run every check (formatting, clippy, tests and the web build) with `scripts/check.sh`; add `--audit` to include `cargo audit`. To run the checks before each push, enable the tracked hook once per clone:

```sh
git config core.hooksPath .githooks
```

Integration tests are grouped into four test binaries under `tests/`: `damage` (formula, affinity and turn calculation), `import` (bundled data and offline Atlas fixtures), `np_mechanics` (NP mechanic runtime and import), and `live` (ignored by default; run with `cargo test --test live -- --ignored`). Fixtures live in `tests/fixtures/`.

## License

License to be decided before the first public release.
