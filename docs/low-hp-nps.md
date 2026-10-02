# Low-HP Noble Phantasm damage

The calculator supports the six audited NP variants listed below. They use
Atlas Academy's `damageNpHpratioLow` function shape, which increases NP damage as
the attacking servant loses HP. The game data importer keeps the base rate and
HP coefficient for each NP level and Overcharge row; it does not infer missing
Overcharge values.

| Servant and NP variant | NP ID | Base `Value` by NP level (NP1–5) | `Target` by Overcharge (OC1–5) |
| --- | ---: | --- | --- |
| Anne Bonny & Mary Read, Archer | 202501 | 6000, 8000, 9000, 9500, 10000 | 6000, 6000, 6000, 6000, 6000 |
| Aśvatthāman | 203301 | 6000, 8000, 9000, 9500, 10000 | 6000, 7000, 8000, 9000, 10000 |
| Anne Bonny & Mary Read, Rider (base) | 400901 | 12000, 16000, 18000, 19000, 20000 | 12000, 14000, 16000, 18000, 20000 |
| Anne Bonny & Mary Read, Rider (upgraded) | 400902 | 16000, 20000, 22000, 23000, 24000 | 12000, 14000, 16000, 18000, 20000 |
| Hijikata Toshizō (base) | 702501 | 6000, 8000, 9000, 9500, 10000 | 6000, 7000, 8000, 9000, 10000 |
| Hijikata Toshizō (upgraded) | 702502 | 8000, 10000, 11000, 11500, 12000 | 8000, 9000, 10000, 11000, 12000 |

These are Atlas source values in thousandths. At the moment NP damage lands, the
calculator applies the same operation order as the pinned Chaldea reference:

```text
effective_value = Value + truncate((1 - current_hp / max_hp) * Target)
effective_multiplier = effective_value / 1000
```

The truncation uses the source's floating-point expression before adding the
contribution. At full HP the HP contribution is zero, so the NP uses its base
multiplier. As current HP falls, the contribution increases or stays the same.
NP level and Overcharge remain independent inputs; changing either one preserves
the entered HP and selects the corresponding imported value row.

## Entering HP

The HP controls appear when the selected NP requires attacker HP. Enter current
and maximum HP as positive whole numbers; current HP must be no greater than
maximum HP. The label refers to attacker HP at NP damage time, after any healing
or HP loss that happened earlier in the turn. The calculator does not execute
those effects or change HP automatically.

When Atlas Academy provides `hpMax`, the initial maximum is the servant's natural
maximum-level HP before player-added Fou bonuses or grail levels. You can edit it
for your own build. If saved servant data has no maximum HP, enter it manually.
A new HP state starts at full HP. Changing maximum HP resets current HP to the new
maximum and displays a notice. Changing servants resets HP for the newly selected
servant. A successful servant-data refresh also resets affected HP inputs and
reports that reset in the update status. Invalid or empty input pauses damage
calculation until corrected; stale results are cleared.

## Scope and data

HP scaling applies only to the selected NP component that carries the audited
mechanic. Normal cards and Extra attacks do not use the HP contribution. The app
does not simulate healing, self-damage, skill effects, enemy HP, or other battle
state. High-HP scaling, trait-count scaling, conditional fields or states, and
unverified low-HP function shapes remain unsupported. Unsupported or incomplete
Overcharge rows remain unavailable instead of copying OC1 values.

The formula is referenced against Chaldea's [`damage.dart` at commit
`daf4e0af5ec6b4ae42f3e9410b26ae143aa3d725`](https://github.com/chaldea-center/chaldea/blob/daf4e0af5ec6b4ae42f3e9410b26ae143aa3d725/lib/app/battle/functions/damage.dart).
The NP values come from Atlas Academy's NA records for [Archer Anne Bonny & Mary
Read (servant 202500)](https://api.atlasacademy.io/nice/NA/servant/202500?lang=en),
[Aśvatthāman (servant 203300)](https://api.atlasacademy.io/nice/NA/servant/203300?lang=en),
[Rider Anne Bonny & Mary Read (servant 400900)](https://api.atlasacademy.io/nice/NA/servant/400900?lang=en),
and [Hijikata Toshizō (servant 702500)](https://api.atlasacademy.io/nice/NA/servant/702500?lang=en).
Snapshot schema v4 stores the low-HP coefficients and optional maximum HP;
schemas v1–v3 remain readable.
