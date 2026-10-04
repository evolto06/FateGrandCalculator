# NP components and Overcharge

The calculator estimates potential damage to one selected enemy. It does not cap
damage at enemy HP or simulate deaths, sacrifice eligibility, retargeting, buff
removal, or effects on subsequent cards.

## Execution reference

The component calculation follows the Chaldea reference simulator at commit
`daf4e0af5ec6b4ae42f3e9410b26ae143aa3d725`:

- [damage.dart](https://github.com/chaldea-center/chaldea/blob/daf4e0af5ec6b4ae42f3e9410b26ae143aa3d725/lib/app/battle/functions/damage.dart): a zero execution rate skips a damage function; each function evaluates its own damage and random-range endpoints.
- [battle_utils.dart](https://github.com/chaldea-center/chaldea/blob/daf4e0af5ec6b4ae42f3e9410b26ae143aa3d725/lib/app/battle/utils/battle_utils.dart): the calculation floors each function's damage.
- [function_executor.dart](https://github.com/chaldea-center/chaldea/blob/daf4e0af5ec6b4ae42f3e9410b26ae143aa3d725/lib/app/battle/functions/function_executor.dart): consecutive damage functions defer damage-release handling. `CheckDead` permits a dead target to remain eligible; it is not an alive-target requirement.

These are reference-simulator semantics, not a claim that the calculator implements
the game's entire battle engine. The underlying terms follow the
[Atlas Academy damage formula](https://apps.atlasacademy.io/fgo-docs/deeper/battle/damage.html).

For supported deterministic components, the displayed NP minimum is the sum of
separately floored component minima; the maximum is the corresponding sum of
component maxima. Flooring a combined multiplier can differ by one or more damage
points and is not used. The UI exposes each component's own breakdown.

For example, with ATK 1003, neutral Saber class/attribute terms, no buffs, and
Buster multipliers 6 and 1, the component ranges are 1868–2281 and 311–380.
Their total is 2179–2661. Combining the multipliers before flooring instead gives
2180–2662. This fixed example isolates component rounding from servant stats.

The range endpoints do not establish the correlation of in-game random rolls or
an expected-damage distribution. With nonnegative components that all execute,
the endpoint sums are the same whether the rolls are shared or independent.

## Supported multi-component profiles

The initial release covers these four audited Atlas NA NP IDs, with strict
validation of function order, target, rates, conditions, and numeric data:

| Servant | NP ID | Additional multiplier at Overcharge 1–5 |
|---|---:|---|
| Arash, base Stella | 201301 | 0, 1, 2, 3, 4 |
| Arash, upgraded Stella | 201302 | 0, 2, 4, 6, 8 |
| Chen Gong | 504401 | 0, 2.25, 4.5, 6.75, 9 |
| Bhīma | 305401 | 0, 2, 3, 4, 5 |

The first component scales with NP level. These additional components depend on
Overcharge and retain the same values across NP levels. Bhīma's extra component is
inactive at OC1; at OC2–5 its source data sets `CheckDead`.

Public source definitions: [Arash](https://api.atlasacademy.io/nice/NA/servant/201300?lang=en),
[Chen Gong](https://api.atlasacademy.io/nice/NA/servant/504400?lang=en), and
[Bhīma](https://api.atlasacademy.io/nice/NA/servant/305400?lang=en). Reduced test
fixtures record their source and retrieval date; values are not inferred from NP
names or upgrade status.

Other multi-function NPs remain unsupported until their execution shape is
verified. A probabilistic execution rate is not treated as a damage multiplier.
Non-damage effects remain manual. Arash's death and Chen Gong's ally requirement
and sacrifice are not simulated. For Bhīma, enter enemy defense after any buff
removal that applies before damage.

## Low-HP NP scaling

Six audited NP variants also support `damageNpHpratioLow`. Their source base rate
and HP coefficient stay separate for every NP-level and Overcharge row. The
effective rate adds the truncated missing-HP contribution before converting the
source value from thousandths to a multiplier. Users enter attacker HP at the
moment NP damage lands; preceding healing or HP loss remains manual. The supported
variants, defaults, and formula are documented in [low-HP NP support](low-hp-nps.md).

## Enemy-status NP scaling

Seven audited `damageNpStateIndividualFix` NPs apply a source `Correction` only
when their matching enemy status is explicitly selected as present at damage
time. The imported base `Value` and conditional correction remain separate, and
the status factor applies only to that NP component. The supported servants,
matching rules, timing assumptions, and formula are documented in
[enemy-status NP support](enemy-status-nps.md).

## Coverage and migration

NP level and Overcharge are independent, each ranging from 1 to 5. A component
stores an explicit OC row containing five NP-level values and execution metadata.
Missing OC rows remain unavailable rather than copying OC1.

Snapshot format v5 retains component, optional low-HP, and optional enemy-status
metadata. The app reads v1–v5 snapshots; snapshots without enemy-status metadata
preserve their existing NP mechanics, and no status bonus is inferred. Choose
**Update servant data** to fetch newly supported NP profiles. Older records with
five base multipliers migrate to a single OC1-only component, preserving
affection, defense-piercing flags, and notes. Updating servant data loads
explicitly available higher-OC values. An unsuccessful update preserves the
prior snapshot.

Overcharge selection changes only imported damage values. It does not calculate
NP gauge, chain-based Overcharge gain, automatic pre-damage buffs, conditional
trait bonuses, or other omitted NP effects. Space Ereshkigal's affection is still
entered at damage time; increasing OC does not automatically increase affection.
