# Trait-targeted NP damage bonuses

An Atlas Academy NA audit found 112 `damageNpIndividual` variants across 88
servant forms. Full-catalog import confirmed 111 supported variants across 87
forms. BB Dubai's NP 2300601 remains base-only because its NP switching script
is outside this release. The audit reused the full NA nice-servant export
retrieved on 2026-10-07. These counts describe NP variants and forms, not every
conditional NP mechanic in the game.

## Matching and formula

For an audited variant, the source `Value` supplies the base NP multiplier and
`Correction` supplies a separate conditional multiplier:

```text
base multiplier        = Value / 1000
conditional multiplier = Correction / 1000, when the trait is assumed present
                         1, otherwise
effective multiplier   = base multiplier × conditional multiplier
```

The correction applies to its NP damage component only and stays separate from
the imported base `Value`. The UI shows the base, conditional, applied, and
effective multipliers. In the reference formula, it multiplies the NP
contribution after additive NP and power modifiers and before flat damage and
the final floor. There is no intermediate truncation of `Value × Correction`.
Normal cards and Extra attacks are unaffected.

The checkbox starts unchecked and names the positive Atlas trait condition, for
example, “Enemy has the Dragon trait at NP damage time.” The pinned Chaldea
reference matches the function target against the enemy's traits with its signed
individuality matcher. In this calculator, the checkbox is the user's manual
assumption; the app does not inspect an enemy trait set or infer traits from
class or attribute. Enter traits granted by earlier effects manually; the app
does not simulate trait-granting effects. No negative trait conditions were
verified in this audit.

## Import limits and data updates

Trait support is imported from source function shape, not a servant-ID list. The
function must be `damageNpIndividual`, target the enemy or all enemies, have no
execution script or function-level conditions, and contain complete source rows
with `Rate=1000`, a consistent recognized positive `Target`, and positive
`Value` and `Correction` fields for each supplied NP-level and Overcharge row.
Missing rows remain unavailable. NP quest-unlock conditions do not change the
damage-time trait check. Variants with extra scripts, function conditions,
targets, or unrecognized row shapes keep valid base NP damage with an
explanatory note; invalid base damage remains unavailable. Compound, rarity,
stack, and other target-state conditions are not inferred.

The assumption belongs to the selected servant, NP, and trait condition.
Changing one resets the checkbox and shows a notice. Changing NP level or
Overcharge preserves it. A successful servant-data update resets the assumption;
a failed update preserves it.

Snapshot schema v6 stores audited trait correction rows and reads schemas v1–v6.
Older snapshots have no trait metadata, so they keep base-only behavior until
you choose **Update servant data**. The app does not infer a correction or copy a
missing NP-level or Overcharge row.

## References

Trait targets and correction rows were audited from Atlas Academy's NA nice
servant records ([NA nice-servant export](https://api.atlasacademy.io/export/NA/nice_servant.json));
the 28 target labels follow [Atlas enums at the audited commit](https://github.com/atlasacademy/fgo-game-data-api/blob/a937bbb0f7eced10f45f57568786da10d9994027/app/schemas/enums.py#L1127-L1496).
The enemy-trait comparison and conditional correction follow the pinned Chaldea
implementation: [`damage.dart` lines 125–134](https://github.com/chaldea-center/chaldea/blob/daf4e0af5ec6b4ae42f3e9410b26ae143aa3d725/lib/app/battle/functions/damage.dart#L125-L134),
[`battle_utils.dart` line 69](https://github.com/chaldea-center/chaldea/blob/daf4e0af5ec6b4ae42f3e9410b26ae143aa3d725/lib/app/battle/utils/battle_utils.dart#L69),
and [`battle_utils.dart` line 104](https://github.com/chaldea-center/chaldea/blob/daf4e0af5ec6b4ae42f3e9410b26ae143aa3d725/lib/app/battle/utils/battle_utils.dart#L104).
