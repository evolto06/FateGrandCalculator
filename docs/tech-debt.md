# Technical debt plan

Audit date: 2026-11-06 (version 0.1.6).

## Progress

All twelve items are done (2026-11-07):

- #1: `scripts/check.sh` runs formatting, clippy, tests and the wasm32 build check; `.githooks/pre-push` calls it. Cargo aliases cannot chain commands, so there is no `cargo ci` alias; use `scripts/check.sh --audit` instead.
- #2: the web client has a real entry point (`src/main.rs`, `index.html`, `Trunk.toml`).
- #3 and #6: `card_damage` is the only formula implementation; `calculate` wraps it, and `calculate_turn` is split into normal-card, NP and Extra helpers. `TurnSelection::check` returns resolved card data, so the `expect` calls are gone.
- #4: clippy passes with `-D warnings`.
- #5: portrait streaming takes an injectable fetcher. Test builds use an offline fetcher, so the whole app streams portraits in tests without network access.
- #7: `ui/components.rs` is split into nine per-panel modules under `ui/components/`.
- #8: `enrich_servant` is split into import steps (`import_np`, `import_components`, `keep_complete_overcharge_rows`, `import_affection`, `np_notes`, …) with unit tests.
- #9: snapshots are read only in the three newest formats (currently v4–v6); see `OLDEST_READABLE_SCHEMA_VERSION` in `servant_data/store.rs` for the rule.
- #10: the old model types and `tests/servant_model.rs` are removed.
- #11: audited NP IDs, notes and trait labels live in `data/audited_nps.json`.
- #12: integration tests are four binaries: `damage`, `import`, `np_mechanics` and `live`.

## Current health (at audit time)

- All tests pass (about 140; 3 network-dependent tests are ignored on purpose).
- `cargo fmt --check` is clean.
- There are no `unwrap()` calls in `src` outside test code.
- `cargo clippy --all-targets` reports about 20 warnings.

## Prioritized items

Priority = (Impact + Risk) x (6 - Effort). Each factor is scored from 1 to 5.

| # | Item | Type | Impact | Risk | Effort | Score |
|---|------|------|---|---|---|---|
| 1 | **No automated checks.** Formatting, clippy, tests and the dependency audit run only when someone remembers to run them. GitHub Actions is not available, so the checks have to run locally. | Infrastructure | 3 | 4 | 1 | 35 |
| 2 | **The browser build is never checked.** The README advertises a WebAssembly web client and the code has 26 `cfg(wasm32)` gates. But `main.rs` only calls `run_native`, and there is no web entry point or build configuration. | Architecture/Docs | 3 | 4 | 2 | 28 |
| 3 | **The damage formula is written twice.** `damage::calculate` and `turn_card` in `src/damage.rs` implement the same formula separately. `calculate` is reached only through `loader.rs` and two tests. | Code | 3 | 3 | 2 | 24 |
| 4 | **About 20 clippy warnings.** Four are `too_many_arguments` (`turn_card`, `show_result`, `attack_panel`, `servant_setup`). The rest are collapsible `if`s, one very complex type and one unnecessary `let`. | Code | 2 | 2 | 1 | 20 |
| 5 | **The portrait code is never tested.** `portrait.rs` is switched off under `cfg(not(test))` in `app.rs`, so its network fetching and size limits have no test coverage. Test builds also report it as dead code. | Test | 2 | 3 | 2 | 20 |
| 6 | **`calculate_turn` is one long closure.** It is about 170 lines and passes a 6-value tuple around. Three `.expect("validated ...")` calls depend on `validate()` agreeing with the code that follows; if they drift apart, the app panics instead of returning an error. | Code | 3 | 3 | 3 | 18 |
| 7 | **`ui/components.rs` is 1,627 lines.** Drawing code is mixed with state syncing and input parsing (`sync_enemy_status`, `sync_trait_bonus`, HP input parsing). | Code | 4 | 2 | 3 | 18 |
| 8 | **`enrich_servant` is about 380 lines** (`servant_data/normalize.rs`). This is where the Atlas Academy data import is most likely to break. | Code | 3 | 3 | 3 | 18 |
| 9 | **The saved-data format has reached version 6.** `servant_data/store.rs` keeps upgrade paths for every version from v1 to v6, and there is no rule for retiring old ones. | Architecture | 2 | 2 | 2 | 16 |
| 10 | **Old model types are kept alive only by a test.** `model::Servant`, `NoblePhantasmType` and `SkillType` have fields nothing reads, and only `tests/servant_model.rs` uses them. The app uses `ServantRecord` instead. | Code | 2 | 1 | 1 | 15 |
| 11 | **The list of audited NPs is hard-coded.** `verified_enemy_status` matches servant and NP IDs, and trait names are hard-coded in `trait_bonus.rs`. Supporting only audited NPs is deliberate, but each newly audited NP needs a code change and a release. | Code | 3 | 2 | 3 | 15 |
| 12 | **The test layout is mixed.** There are 29 top-level test files, each compiled and linked separately, plus `tests/np_mechanics.rs`, which pulls in modules with `#[path]`. Some topics are split across both styles, for example `np_import.rs` and `np_mechanics/import.rs`. | Test | 2 | 1 | 2 | 12 |

## Remediation plan

### Phase 1: quick wins (about half a day)

- **Local checks (#1).** GitHub Actions is unavailable because of billing, so run the checks before each push:
  - Add a tracked `.githooks/pre-push` script that runs:
    - `cargo fmt --check`
    - `cargo clippy --all-targets -- -D warnings`
    - `cargo test`
    - `cargo check --target wasm32-unknown-unknown --lib`
  - Turn the hook on once per clone with `git config core.hooksPath .githooks`.
  - Add a `cargo ci` alias in `.cargo/config.toml` that runs the same checks, plus `cargo audit`. The audit downloads the advisory list, so it stays out of the hook to keep pushes fast.
  - One-time setup: `rustup target add wasm32-unknown-unknown` and `cargo install cargo-audit`.
- **Clippy warnings (#4).** Fix them so `-D warnings` passes. Group long argument lists into context structs.
- **Old model types (#10).** Delete them and `tests/servant_model.rs`.

### Phase 2: alongside the next formula or NP feature

- **One damage formula (#3).** Make `calculate` a thin wrapper over the turn calculation, or remove it.
- **Split `calculate_turn` (#6).** Give normal cards, NPs and the Extra attack their own functions. Have validation return already-checked data so the `expect` calls are no longer needed.
- **Decide on the browser build (#2).** Either add a real web entry point and build setup, or remove the claim from the README and drop the `cfg` gates.

### Phase 3: when you next work in these areas

- **Split `ui/components.rs` (#7).** Give each panel its own module, and move the state syncing out of the drawing code.
- **Break up `enrich_servant` (#8).** Split it into steps that each have their own tests.
- **Move the audited-NP list into data (#11).** Load it with the game data instead of hard-coding it.
- **Test the portrait code (#5).** Hide its network calls behind something tests can replace with a fake, then add tests.

### Ongoing

- **Retirement rule for old saved-data formats (#9).** For example, support only the last N versions and reset older caches to the bundled data.
- **Tidy the test layout (#12).** Group the test files by topic.
