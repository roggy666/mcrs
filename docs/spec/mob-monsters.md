# Monsters with goals: the shared monster rules and the goal lists of each family

Version: see `assets/minecraft/version.json`.

| Field | Value |
|-------|-------|
| Entity types | `minecraft:zombie`, `minecraft:husk`, `minecraft:drowned`, `minecraft:zombie_villager`, `minecraft:zombified_piglin`, `minecraft:skeleton`, `minecraft:stray`, `minecraft:bogged`, `minecraft:parched`, `minecraft:wither_skeleton`, `minecraft:creeper`, `minecraft:spider`, `minecraft:cave_spider`, `minecraft:enderman`, `minecraft:silverfish`, `minecraft:endermite`, `minecraft:witch`, `minecraft:blaze`, `minecraft:slime`, `minecraft:magma_cube`, `minecraft:iron_golem`, `minecraft:snow_golem`, `minecraft:giant` |
| Data | `assets/mcrs/reports/registries.json`, the data pack of the server jar, the type artefact |

This document states the goal lists and the target lists of the monsters that use goals, and the
rules that the families share. It uses the goal framework of `mob-goals.md` (prefix `GOAL`) and the
combat goals of `mob-combat-goals.md` (prefix `CGOAL`) with no change. A family that needs a goal
that those documents do not state gets the goal here.

The rules state results, not algorithms. Each rule has a fidelity class (GOAL section 0.3). A
monster whose behaviour needs a part of the engine that no specification states yet, for example
a projectile or an explosion, gets a hook: the goal states the action, and the rule names the
specification that must state the result.

## 0. Conventions

### 0.1 Terms

- The terms of `mob-goals.md`, section 0.1, and of `mob-combat-goals.md`, section 0.1, apply.
- **Monster**: an entity of one of the types of the table above.
- **Zombie kind**: `zombie`, `husk`, `drowned`, `zombie_villager` and `zombified_piglin`.
- **Skeleton kind**: `skeleton`, `stray`, `bogged`, `parched` and `wither_skeleton`.
- **Spider kind**: `spider` and `cave_spider`.
- **Slime kind**: `slime` and `magma_cube`.
- **Undead**: the zombie kind, the skeleton kind, `phantom` and `wither`. A damage type tag of the
  data pack is not the source of this set; the entity type tag `undead` is.
- **In the sun**: a mob is in the sun when all these conditions are true: it is daytime, the sky
  light at its block position is 15, the block position has no roof, and the mob is not in water
  or rain. The hot tick computes the state. `environment-damage.md` states the burn.
- **Attack action**: the action of a ranged goal (CGOAL-RANGED-002, step 4). The type names it.
- **Swell**: the state of a creeper that prepares its explosion (MON-CRP-001).
- **Aggressive bit**: bit 2 of the entity data entry 15 of a mob (TRK-DATA-002).

### 0.2 Provenance, kind, confidence and fidelity

The labels of `mob-goals.md`, sections 0.2 and 0.3, apply. The label `documentation` marks a rule
from the public documentation of the game and from the data of the type artefact and the attribute
table of the data pack.

### 0.3 ID prefix

| Prefix | Subject |
|--------|---------|
| `MON-TYPE` | type data and attributes that this document owns |
| `MON-SUN` | the sun: flee the sun, and the sun on the path |
| `MON-ZMB` | the zombie kind |
| `MON-SKL` | the skeleton kind |
| `MON-CRP` | the creeper and the swell |
| `MON-SPD` | the spider kind |
| `MON-END` | the enderman |
| `MON-BUG` | the silverfish and the endermite |
| `MON-WIT` | the witch |
| `MON-BLZ` | the blaze |
| `MON-SLM` | the slime kind |
| `MON-GOL` | the iron golem and the snow golem |
| `MON-DEV` | intentional deviations |

### 0.4 Scope

In scope: the goal list and the target list of each family, the shared goals of the monsters, and
the parameters of the goals that GOAL and CGOAL state.

Out of scope:

| Subject | Where |
|---------|-------|
| The goal framework, the controls, the queries | `mob-goals.md` |
| The target goals, the melee goal, the ranged goal, the leap, the avoid goal | `mob-combat-goals.md` |
| The projectile of a ranged attack: the arrow, the potion, the fireball, the snowball | `projectiles.md`, and the specification of each type |
| The explosion of a creeper, the damage of the sun, the drops, the spawn rules | `environment-damage.md`, `natural-spawning.md`, the loot specification |
| The conversion of a zombie villager, the reinforcements of a zombie, the door break, the village walk | a later specification; hooks here |
| The teleport, the stare and the block carry of an enderman | a later specification; hooks here |
| The jump motion and the split of a slime | a later specification; hooks here |
| The monsters with a brain: the piglin, the piglin brute, the hoglin, the zoglin, the warden, the breeze, the creaking | `mob-brain.md` |
| The raiders, the vex and the ravager | `mob-raiders.md` |
| The guardians, the phantom, the ghast, the wither, the ender dragon | a later specification |

## 1. Type data

**MON-TYPE-001** (`documentation`, semantic, confidence: high, fidelity: exact). The dimensions,
the eye height and the fire immunity of each type come from the type artefact. The base attributes
come from the attribute table of the data pack. The values that the goals read:

| Type | `follow_range` | `attack_damage` | `movement_speed` | Fire immune |
|------|----------------|-----------------|------------------|-------------|
| `zombie`, `husk`, `drowned`, `zombie_villager` | 35 | 3 | `0.23F` | no |
| `zombified_piglin` | 35 | 5 | `0.23F` | yes |
| `skeleton`, `stray`, `bogged`, `parched` | 16 | 2 | 0.25 | no |
| `wither_skeleton` | 16 | 2 | 0.25 | yes |
| `creeper` | 16 | 2 | 0.25 | no |
| `spider` | 16 | 2 | `0.3F` | no |
| `cave_spider` | 16 | 2 | `0.3F` | no |
| `enderman` | 64 | 7 | `0.3F` | no |
| `silverfish` | 16 | 1 | 0.25 | no |
| `endermite` | 16 | 2 | 0.25 | no |
| `witch` | 16 | 2 | 0.25 | no |
| `blaze` | 48 | 6 | `0.23F` | yes |
| `slime`, `magma_cube` | 16 | 2 | the size of the slime sets it | `magma_cube` yes |
| `iron_golem` | 16 | 15 | 0.25 | no |
| `snow_golem` | 16 | 2 | `0.2F` | no |
| `giant` | 16 | 50 | 0.5 | no |

A value that the table does not give is the default of the attribute (`attributes.md`). The spawn
bonus of `follow_range` (ANIM-SPAWN-001) applies to each monster of the first spawn setup.

**MON-TYPE-002** (`documentation`, semantic, confidence: high, fidelity: exact). The head turn
speed, the pitch limit and the head yaw limit of each monster are the defaults of GOAL-ANI-006,
with these exceptions: the head yaw limit of the zombie kind and the skeleton kind is 75, and the
pitch limit of the enderman is 40 while it does not carry a block.

**MON-TYPE-003** (`documentation`, semantic, confidence: medium, fidelity: exact). The malus rows
of the monsters (PATH-MAL-001): each type keeps the default table. The zombie kind, the skeleton
kind and the witch add no row. The `husk`, the `drowned`, the `zombified_piglin`, the `wither_skeleton`,
the `blaze` and the `magma_cube` have the malus 0 for `DANGER_FIRE`, `DAMAGE_FIRE` and `LAVA`,
because they are immune to fire or to lava. The `spider` kind has the malus 0 for `COCOWEB`-like
blocks: the block tag `pathfinding/sticky` does not slow a spider kind.

## 2. The sun

**MON-SUN-001** (`documentation`, semantic, confidence: high, fidelity: exact). The flee-sun goal
holds the flag `move`. It is an evaluation goal with a speed modifier `m`. Only an undead monster
that burns in the sun has it.

- Start condition: the attack target is none, the mob is on fire, the mob is in the sun, and a
  shaded position exists. The shaded position is the result of the default search `D(20, 3)`
  (GOAL-POS-002) whose block position is not in the sun; the search makes 10 tries, and each
  try also rejects a position in the sun.
- Start action: request a path to the shaded position with the speed modifier `m`.
- Continue condition: the path is not done.
- Stop action: none.

**MON-SUN-002** (`documentation`, semantic, confidence: medium, fidelity: envelope). The
restrict-sun goal holds no flag. It is an evaluation goal. While the mob is in a dimension with a
sky and the sun is up, the path search of the mob avoids a node in the sun: the node gets the extra
malus 8. The goal starts when the sun is up, and stops when it is down. The envelope: a path of a
mob with this goal does not enter the sun while a path of equal length in the shade exists.

**MON-SUN-003** (`design`, semantic, confidence: medium, fidelity: exact under the condition
below). A server without the day cycle and the sky light on the server side uses "not in the sun"
for each mob (GOAL-FLT-003 states the same form for the fluids). Then the flee-sun goal never starts
and the restrict-sun goal never starts. The rule is exact at night and under a roof.

## 3. The zombie kind

**MON-ZMB-001** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of
the `zombie`, the `husk`, the `zombie_villager` and the `drowned` on land, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 0 | float (GOAL-FLT-001) | `jump` | |
| 2 | melee attack (CGOAL-MELEE-001), zombie form (MON-ZMB-003) | `move`, `look` | `m = 1.0`; follow a target that it does not see; `I = 20` |
| 6 | move through a village | `move` | hook (MON-ZMB-005) |
| 7 | stroll (GOAL-STR-001) | `move` | `m = 1.0` |
| 8 | look at a player (GOAL-LKP-001) | `look` | range 8 |
| 8 | look around (GOAL-LKR-001) | `move`, `look` | |

The `zombie`, the `husk` and the `zombie_villager` also have the break-door goal at the priority 1
(MON-ZMB-005, hook). The `husk` does not burn in the sun. The `zombie`, the `zombie_villager` and the
`drowned` burn in the sun; they have no flee-sun goal.

**MON-ZMB-002** (`documentation`, semantic, confidence: high, fidelity: exact). The target list of
the `zombie`, the `husk`, the `zombie_villager` and the `drowned`, in this order:

| Priority | Goal | Parameters |
|----------|------|------------|
| 1 | hurt-by target (CGOAL-HURT-002) | alert; ignored: `zombified_piglin` |
| 2 | nearest attackable target (CGOAL-NEAR-004) | the players; `r = 10`; must see |
| 3 | nearest attackable target | type set: `villager`; `r = 10`; does not need to see |
| 3 | nearest attackable target | type set: `iron_golem`; `r = 10`; must see |
| 5 | nearest attackable target | type set: `turtle`; `r = 10`; must see; filter: the turtle is a baby and is not in water |

Each goal holds `target`. The follow distance is the follow range, 35 with the base value.

**MON-ZMB-003** (`documentation`, semantic, confidence: high, fidelity: exact). The zombie form of
the melee goal is CGOAL-MELEE-001 with these additions. The start action sets a raise-arm timer to
0. The goal tick adds 1 to the timer; while the timer is more than 0 and less than 5, the aggressive
bit is clear, and from 5 on it is set. The stop action clears the aggressive bit and sets the timer
to 0. Thus the client sees the arms rise 5 ticks after the start of the attack. A server can set
the bit at the start (MON-DEV-001).

**MON-ZMB-004** (`documentation`, semantic, confidence: high, fidelity: exact). The
`zombified_piglin` is a neutral mob (CGOAL-RESET-ANGER-001). Its anger duration is uniform over the
integers from 400 to 780 ticks. Its goal list is that of MON-ZMB-001 without the break-door goal.
Its target list, in this order:

| Priority | Goal | Parameters |
|----------|------|------------|
| 1 | hurt-by target | alert; ignored: `zombified_piglin` |
| 2 | nearest attackable target | the players; `r = 10`; must see; filter: the mob is angry at the player (CGOAL-RESET-ANGER-004) |
| 3 | reset universal anger (CGOAL-RESET-ANGER-005) | alert others of the same type |

The anger update runs with the option "stay angry while a target is present" false. An alert of a
zombified piglin makes each zombified piglin of the alert box angry at the attacker: the grant of
the alert also starts the anger timer of each receiver.

**MON-ZMB-005** (`documentation`, semantic, confidence: medium, fidelity: envelope). Hooks of the
zombie kind:

- The break-door goal (AIT-VAN-008, the break form) needs the difficulty `hard` and the game rule
  `mob_griefing`. A later specification states it.
- The move-through-village goal needs the village data. A later specification states it.
- The reinforcements of a hit zombie need the spawn rules. A later specification states it.
- The `drowned` in water uses a swim navigation and a trident. A later specification states it. On
  land, the drowned behaves as this document states, and it attacks only at night or in water.
- The conversion of a `zombie_villager` and of a `zombie` in water need the interactions. A later
  specification states them.

Until these specifications exist, the goal rows of the hooks are absent from the goal list of the
server, and the other rows keep their priorities.

## 4. The skeleton kind

**MON-SKL-001** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of the
`skeleton`, the `stray`, the `bogged` and the `parched`, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 2 | restrict sun (MON-SUN-002) | none | |
| 3 | flee sun (MON-SUN-001) | `move` | `m = 1.0` |
| 3 | avoid entity (CGOAL-AVOID-001) | `move` | type set: `wolf`; `D = 6`; `m1 = 1.0`; `m2 = 1.2` |
| 4 | bow attack (MON-SKL-003) | `move`, `look` | `m = 1.0`; `I = 20`; `R = 15` |
| 4 | melee attack (CGOAL-MELEE-001) | `move`, `look` | `m = 1.2`; stops when the target is not seen; `I = 20` |
| 5 | stroll (GOAL-STR-001) | `move` | `m = 1.0` |
| 6 | look at a player (GOAL-LKP-001) | `look` | range 8 |
| 6 | look around (GOAL-LKR-001) | `move`, `look` | |

The skeleton selects one of the two attack goals of priority 4 by its main hand item (MON-SKL-004).
The `wither_skeleton` has the same list without the restrict-sun and the flee-sun goals, and it
uses the melee goal with `m = 1.2`; it does not burn in the sun.

**MON-SKL-002** (`documentation`, semantic, confidence: high, fidelity: exact). The target list of
the skeleton kind, in this order:

| Priority | Goal | Parameters |
|----------|------|------------|
| 1 | hurt-by target (CGOAL-HURT-002) | no alert |
| 2 | nearest attackable target (CGOAL-NEAR-004) | the players; `r = 10`; must see |
| 3 | nearest attackable target | type set: `iron_golem`; `r = 10`; must see |
| 3 | nearest attackable target | type set: `turtle`; `r = 10`; must see; filter: the turtle is a baby and is not in water |

A `wither_skeleton` also has the nearest attackable target goal of the priority 3 for the type set
`piglin` and `piglin_brute`.

**MON-SKL-003** (`documentation`, semantic, confidence: medium, fidelity: exact for the timers,
envelope for the projectile). The bow attack is the ranged goal of CGOAL-RANGED-001 with
`Imin = Imax = I` and the range `R`. `I` is 20 on the difficulty `hard` and 40 on each other
difficulty. The attack action shoots an arrow at the target with the power of the goal; the arrow
is a hook of `projectiles.md`. While the goal runs, the mob draws the bow: the entity data of the
living entity shows the use of the item in the main hand. A `bogged` and a `parched` shoot an arrow
with the effect of their type; a `stray` shoots an arrow of slowness.

**MON-SKL-004** (`documentation`, semantic, confidence: high, fidelity: exact). A skeleton kind
selects its attack goal when it spawns, when it loads, and when its main hand item changes: with
a bow in the main hand, the bow attack goal is in the goal list and the melee goal is not; else the
melee goal is in the list and the bow attack goal is not. A skeleton of the first spawn setup holds a
bow. A `wither_skeleton` holds a stone sword and uses the melee goal.

## 5. The creeper

**MON-CRP-001** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of
the creeper, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 1 | float (GOAL-FLT-001) | `jump` | |
| 2 | swell (AIT-VAN-007) | `move` | |
| 3 | avoid entity (CGOAL-AVOID-001) | `move` | type set: `ocelot`; `D = 6`; `m1 = 1.0`; `m2 = 1.2` |
| 3 | avoid entity | `move` | type set: `cat`; `D = 6`; `m1 = 1.0`; `m2 = 1.2` |
| 4 | melee attack (CGOAL-MELEE-001) | `move`, `look` | `m = 1.0`; stops when the target is not seen; `I = 20` |
| 5 | stroll (GOAL-STR-001) | `move` | `m = 0.8` |
| 6 | look at a player (GOAL-LKP-001) | `look` | range 8 |
| 6 | look around (GOAL-LKR-001) | `move`, `look` | |

The melee goal of the creeper never hits: the swell takes `move` before the target is in reach
(AIT-RFX-002), and a creeper has no attack sound. The target list:

| Priority | Goal | Parameters |
|----------|------|------------|
| 1 | nearest attackable target (CGOAL-NEAR-004) | the players; `r = 10`; must see |
| 2 | hurt-by target (CGOAL-HURT-002) | no alert |

**MON-CRP-002** (`documentation`, semantic, confidence: high, fidelity: exact). The swell is the
reflex of AIT-RFX-002 with these values: the fuse is 30 ticks; the counter starts at 0; the swell
direction is +1 when the attack target is alive, within 7 blocks (squared distance at most 49), and
in sight, and else −1; the swell starts in a full evaluation when the attack target is within 3
blocks (squared distance less than 9). The entity data entry of the creeper shows the direction:
the client plays the swell. When the counter reaches the fuse, the creeper explodes; the explosion
is a hook of `environment-damage.md`. The server sends the state of the swell; the explosion is
the only part that waits for its specification.

## 6. The spider kind

**MON-SPD-001** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of
the spider kind, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 1 | float (GOAL-FLT-001) | `jump` | |
| 3 | leap at target (CGOAL-LEAP-001) | `jump`, `move` | `y0 = 0.4` |
| 4 | spider attack (MON-SPD-003) | `move`, `look` | `m = 1.0`; stops when the target is not seen; `I = 20` |
| 5 | stroll (GOAL-STR-001) | `move` | `m = 0.8` |
| 6 | look at a player (GOAL-LKP-001) | `look` | range 8 |
| 6 | look around (GOAL-LKR-001) | `move`, `look` | |

**MON-SPD-002** (`documentation`, semantic, confidence: high, fidelity: exact). The target list of
the spider kind, in this order:

| Priority | Goal | Parameters |
|----------|------|------------|
| 1 | hurt-by target (CGOAL-HURT-002) | no alert |
| 2 | spider target (MON-SPD-004) | the players; `r = 10`; must see |
| 3 | spider target | type set: `iron_golem`; `r = 10`; must see |

**MON-SPD-003** (`documentation`, semantic, confidence: medium, fidelity: exact). The spider attack
is the melee goal with one more stop rule: the goal stops when the light at the block position of
the mob is more than `0.5` of the full brightness and the keyed random value of the evaluation
passes with the chance 1 in 100. A spider in daylight thus stops its attack from time to time.
The light test is a hook until the light engine reaches the server side; a server without it uses
a light of 0.

**MON-SPD-004** (`documentation`, semantic, confidence: medium, fidelity: exact). The spider
target is the nearest attackable target goal with one more start rule: the goal does not start
when the light at the block position of the mob is `0.5` or more of the full brightness. The same
hook as MON-SPD-003 applies.

**MON-SPD-005** (`documentation`, semantic, confidence: high, fidelity: envelope). A spider climbs
a wall: while its horizontal collision flag is true, it moves up as on a ladder. The climb is a
rule of the walker, a hook of `entity-physics.md`. The `cave_spider` poisons its target on a hit
when the difficulty is not `easy`; the poison is a hook of `status-effects.md`.

## 7. The enderman

**MON-END-001** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of
the enderman, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 0 | float (GOAL-FLT-001) | `jump` | |
| 1 | stare at a player | `look` | hook (MON-END-003) |
| 2 | melee attack (CGOAL-MELEE-001) | `move`, `look` | `m = 1.0`; stops when the target is not seen; `I = 20` |
| 7 | stroll (GOAL-STR-001) | `move` | `m = 1.0` |
| 8 | look at a player (GOAL-LKP-001) | `look` | range 8 |
| 8 | look around (GOAL-LKR-001) | `move`, `look` | |
| 10 | take a block | none | hook (MON-END-003) |
| 11 | put a block | none | hook (MON-END-003) |

**MON-END-002** (`documentation`, semantic, confidence: high, fidelity: exact). The target list of
the enderman, in this order:

| Priority | Goal | Parameters |
|----------|------|------------|
| 1 | find a player that stares | hook (MON-END-003) |
| 2 | hurt-by target (CGOAL-HURT-002) | no alert |
| 3 | nearest attackable target (CGOAL-NEAR-004) | type set: `endermite`; `r = 10`; must see |
| 4 | reset universal anger (CGOAL-RESET-ANGER-005) | no alert |

The enderman is a neutral mob (CGOAL-RESET-ANGER-001) with the anger duration uniform from 400 to
780 ticks.

**MON-END-003** (`documentation`, semantic, confidence: medium, fidelity: envelope). Hooks of the
enderman: the stare of a player that makes the enderman angry, the stare goal that freezes the
enderman while a player looks at it, the teleport when it takes damage from water or a
projectile, and the carry of a block. A later specification states them. Until then, an enderman
attacks only an attacker and an endermite.

## 8. The silverfish and the endermite

**MON-BUG-001** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of
the silverfish and of the endermite, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 1 | float (GOAL-FLT-001) | `jump` | |
| 3 | hide in a block (silverfish only) | `move` | hook |
| 4 | melee attack (CGOAL-MELEE-001) | `move`, `look` | `m = 1.0`; stops when the target is not seen; `I = 20` |
| 5 | wake friends (silverfish only) | none | hook |
| 7 | stroll (GOAL-STR-001) | `move` | `m = 1.0` |
| 8 | look at a player (GOAL-LKP-001) | `look` | range 8 |
| 8 | look around (GOAL-LKR-001) | `move`, `look` | |

The target list of both types, in this order:

| Priority | Goal | Parameters |
|----------|------|------------|
| 1 | hurt-by target (CGOAL-HURT-002) | alert |
| 2 | nearest attackable target (CGOAL-NEAR-004) | the players; `r = 10`; must see |

The block hide of a silverfish and the wake of the friends in the infested blocks are hooks of the
block specifications. An endermite lives 2400 ticks unless it is persistent; the lifetime is a hook
of `natural-spawning.md`.

## 9. The witch

**MON-WIT-001** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of the
witch, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 1 | float (GOAL-FLT-001) | `jump` | |
| 2 | potion attack (MON-WIT-003) | `move`, `look` | `m = 1.0`; `I = 60`; `R = 10` |
| 2 | stroll (GOAL-STR-001) | `move` | `m = 1.0` |
| 3 | look at a player (GOAL-LKP-001) | `look` | range 8 |
| 3 | look around (GOAL-LKR-001) | `move`, `look` | |

The potion attack is first in the list, so it wins over the stroll in a full evaluation that can
start both.

**MON-WIT-002** (`documentation`, semantic, confidence: high, fidelity: exact). The target list of
the witch, in this order:

| Priority | Goal | Parameters |
|----------|------|------------|
| 1 | hurt-by target (CGOAL-HURT-002) | no alert; ignored: `raider` kinds |
| 2 | nearest attackable target (CGOAL-NEAR-004) | the players; `r = 10`; must see |
| 3 | nearest attackable target | type set: `iron_golem`; `r = 10`; must see; the witch attacks it only as a raider (hook) |

**MON-WIT-003** (`documentation`, semantic, confidence: medium, fidelity: exact for the timers,
envelope for the potion). The potion attack is the ranged goal of CGOAL-RANGED-001 with
`Imin = Imax = 60` and the range 10. The attack action throws a splash potion at the target; the
potion and its choice by the state of the target are hooks of `projectiles.md`. A witch does not
throw while it drinks a potion; the drink is a hook of `status-effects.md`.

## 10. The blaze

**MON-BLZ-001** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of
the blaze, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 4 | fireball attack (MON-BLZ-003) | `move`, `look` | |
| 5 | move toward the home | `move` | hook |
| 7 | stroll (GOAL-STR-001) | `move` | `m = 1.0` |
| 8 | look at a player (GOAL-LKP-001) | `look` | range 8 |
| 8 | look around (GOAL-LKR-001) | `move`, `look` | |

**MON-BLZ-002** (`documentation`, semantic, confidence: high, fidelity: exact). The target list of
the blaze, in this order:

| Priority | Goal | Parameters |
|----------|------|------------|
| 1 | hurt-by target (CGOAL-HURT-002) | no alert |
| 2 | nearest attackable target (CGOAL-NEAR-004) | the players; `r = 10`; must see |

**MON-BLZ-003** (`documentation`, semantic, confidence: medium, fidelity: envelope). The fireball
attack is a per-tick goal that holds `move` and `look`. It starts when the attack target is alive,
and continues while it is. Its goal tick: when the target is within 4 blocks (squared distance less
than 4), the blaze hits it in melee each 20 ticks; else, when the target is within 48 blocks and in
sight, the blaze shoots a volley of three small fireballs with 6 ticks between the shots, then waits
100 ticks; the fireballs are a hook of `projectiles.md`. While the target is out of sight for 5
ticks, the blaze requests a path to the target with `m = 1.0`. The blaze flies: its motion is a hook
of `entity-physics.md`. Until the fireballs exist, the server runs the melee part and the path
request only.

## 11. The slime kind

**MON-SLM-001** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of
the slime kind, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 1 | float, slime form | `jump`, `move` | hook |
| 2 | attack by a jump | `look` | hook |
| 3 | random direction | `look` | hook |
| 5 | keep on jumping | `jump`, `move` | hook |

**MON-SLM-002** (`documentation`, semantic, confidence: high, fidelity: envelope). The slime kind
moves by jumps toward its attack target, which it selects as the nearest player within 16 blocks
that it can attack; a slime of the size 1 does not attack. The motion by jumps, the split on death
and the size are hooks of a later specification. Until then, a slime kind has an empty goal list and
an empty target list, and it stands still. This is a stated gap, not a deviation.

## 12. The golems

**MON-GOL-001** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of the
iron golem, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 1 | melee attack (CGOAL-MELEE-001) | `move`, `look` | `m = 1.0`; follow a target that it does not see; `I = 20` |
| 2 | move toward the home | `move` | hook |
| 2 | move through a village | `move` | hook |
| 4 | golem stroll | `move` | hook: the stroll of GOAL-STR-001 with `m = 0.6` that prefers the village |
| 5 | offer a flower | `move`, `look` | hook |
| 6 | look at a player (GOAL-LKP-001) | `look` | range 6 |
| 7 | look around (GOAL-LKR-001) | `move`, `look` | |

The target list of the iron golem, in this order:

| Priority | Goal | Parameters |
|----------|------|------------|
| 1 | defend the village | hook |
| 2 | hurt-by target (CGOAL-HURT-002) | no alert |
| 3 | nearest attackable target (CGOAL-NEAR-004) | the players; `r = 10`; must see; filter: the golem was built by a player, or the player is a raider target (hook) |
| 3 | nearest attackable target | type set: each monster except the creeper; `r = 5`; must see; filter: the mob is not a creeper |
| 4 | reset universal anger (CGOAL-RESET-ANGER-005) | no alert |

The iron golem is a neutral mob with the anger duration uniform from 400 to 780 ticks. Until the
village data exists, the golem strolls with `m = 0.6` on the plain stroll goal, and its player
target goal is absent.

**MON-GOL-002** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of the
snow golem, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 1 | snowball attack (MON-GOL-003) | `move`, `look` | `m = 1.25`; `I = 20`; `R = 10` |
| 2 | stroll (GOAL-STR-001) | `move` | `m = 1.0`; `L(10, 7)` only |
| 3 | look at a player (GOAL-LKP-001) | `look` | range 6 |
| 4 | look around (GOAL-LKR-001) | `move`, `look` | |

The target list of the snow golem: the nearest attackable target goal at the priority 1 for the type
set of each monster, with `r = 10`, must see, and the filter "the mob is not a creeper".

**MON-GOL-003** (`documentation`, semantic, confidence: medium, fidelity: exact for the timers,
envelope for the snowball). The snowball attack is the ranged goal of CGOAL-RANGED-001 with
`Imin = Imax = 20` and the range 10. The attack action throws a snowball at the target; the
snowball is a hook of `projectiles.md`.

## 13. The giant

**MON-GIA-001** (`documentation`, semantic, confidence: high, fidelity: exact). The giant has an
empty goal list and an empty target list. It stands still.

## 14. Intentional deviations

### MON-DEV-001: The aggressive bit at the start of the zombie attack

- **Rule:** MON-ZMB-003.
- **Reference:** the reference sets the aggressive bit 5 ticks after the start of the attack goal
  and clears it 5 ticks after the stop.
- **This server:** the bit follows the start and the stop of the melee goal at once.
- **Visible to the client:** yes, by 5 ticks of the arm pose.

## 15. Unresolved

- **U1**: the attack interval of a skeleton reads the difficulty of the dimension. The server has no
  difficulty yet; the value 40 of the other difficulties applies.
- **U2**: each hook of this document names a later specification. The goal rows of the hooks are
  absent from the goal lists of the server until that specification exists.
- **U3**: the type set "each monster" of the golems needs the entity type tag of the monsters; the
  server reads the mob category `monster` of the spawn data of each type.
