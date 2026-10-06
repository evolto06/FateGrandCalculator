# Enemy-status NP damage bonuses

Seven audited Atlas Academy NA NPs use `damageNpStateIndividualFix`. For each
one, the calculator keeps the NP's base `Value` separate from its conditional
`Correction`, and applies the correction only when the matching enemy status is
assumed present at the moment damage lands. The source records were refreshed
on 2026-10-05.

## Supported variants

The target ID is the positive Atlas function target matched against the enemy
buff individuality; negated targets are not supported. `IncludeIgnoreIndividuality`
determines whether matching also includes enemy buffs whose own script sets
`IgnoreIndividuality=1`.

| Servant | NP ID | Enemy status | Target ID | Includes `IgnoreIndividuality=1` buffs |
|---|---:|---|---:|:---:|
| [Robin Hood](https://api.atlasacademy.io/nice/NA/servant/200300?lang=en) | 200301 | Poison | 3011 | No |
| [Charlotte Corday (Caster)](https://api.atlasacademy.io/nice/NA/servant/504800?lang=en) | 504801 | Skill Seal | 3025 | Yes |
| [Kashin Koji](https://api.atlasacademy.io/nice/NA/servant/604900?lang=en) | 604901 | Bind | 3087 | Yes |
| [Sizuki Soujyuro](https://api.atlasacademy.io/nice/NA/servant/704900?lang=en) | 704901 | Defense Up | 3058 | No |
| [Kama (Avenger)](https://api.atlasacademy.io/nice/NA/servant/1101100?lang=en) | 1101101 | Charm | 3012 | Yes |
| [Utsumi Erice (Avenger)](https://api.atlasacademy.io/nice/NA/servant/1101400?lang=en) | 1101401 | Curse | 3026 | No |
| [Yang Guifei](https://api.atlasacademy.io/nice/NA/servant/2500400?lang=en) | 2500401 | Burn | 3015 | No |

That matching flag is separate from whether a buff can be removed. Poison,
Defense Up, Curse, and Burn use the source's ordinary individuality matching;
Skill Seal, Bind, and Charm also match buffs marked `IgnoreIndividuality=1`.
The importer accepts only the audited target, matching flag, explicit source
`Rate=1000`, and complete source rows. Unexpected or incomplete definitions
stay unsupported instead of producing an approximate multiplier.

## Formula and manual status input

The source `Value` supplies the NP base multiplier. The source `Correction`
supplies an independent conditional multiplier:

```text
base multiplier        = Value / 1000
conditional multiplier = Correction / 1000, when the matching status is present
                         1, otherwise
effective NP multiplier = base multiplier × conditional multiplier
```

The UI shows the base rate, whether the condition is active, the conditional
factor, and the effective rate. The condition is initially absent. Select the
checkbox only when the matching enemy buff is actually present when NP damage
lands. The calculator does not simulate status application, chance, duration,
or removal. An effect that may fail requires the user to choose the successful
or absent outcome manually.

Timing matters for two supported NPs. Kashin Koji's NP can apply Bind before its
damage, so select Bind when that application succeeded. Yang Guifei's NP applies
Burn after its damage, so that Burn does not activate its own current NP bonus;
select Burn only when it was already present before the NP hit.

Defense Up is a status condition for Sizuki Soujyuro's NP. Its checkbox only
controls the NP's conditional correction; it does not change the separate
`Enemy defense (%)` input. Enter the numeric defense value independently.

## Data updates and reset behavior

The status assumption belongs to the currently selected servant, NP, and
condition. Changing any of those resets it to absent and displays a notice. A
successful catalog refresh also resets the assumption and displays a notice; a
failed refresh preserves the existing assumption. Changing NP level or
Overcharge preserves the status assumption while the same NP remains selected.

Snapshot schema v5 stores the audited correction rows. Schemas v1–v5 are
readable; snapshots without status metadata preserve their existing NP
mechanics, and no status bonus is inferred. Choose **Update servant data** to
fetch newly supported NP profiles. The app does not invent a status correction
or fill missing NP-level or Overcharge rows. See [NP components and Overcharge](np-components.md)
for component rounding and migration behavior.

## References

The source values and target IDs come from the linked Atlas Academy NA servant
records above. The conditional matching and separate correction factor follow
the pinned Chaldea reference implementation: [`damage.dart` lines 144–156](https://github.com/chaldea-center/chaldea/blob/daf4e0af5ec6b4ae42f3e9410b26ae143aa3d725/lib/app/battle/functions/damage.dart#L144-L156),
[`battle_utils.dart` line 69](https://github.com/chaldea-center/chaldea/blob/daf4e0af5ec6b4ae42f3e9410b26ae143aa3d725/lib/app/battle/utils/battle_utils.dart#L69),
and [`battle_utils.dart` line 104](https://github.com/chaldea-center/chaldea/blob/daf4e0af5ec6b4ae42f3e9410b26ae143aa3d725/lib/app/battle/utils/battle_utils.dart#L104).
The underlying damage terms follow [Atlas Academy's battle damage reference](https://apps.atlasacademy.io/fgo-docs/deeper/battle/damage.html).
