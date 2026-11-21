# The other passive and neutral mobs with goals: goal lists and hooks

Version: see `assets/minecraft/version.json`.

| Field | Value |
|-------|-------|
| Entity types | `minecraft:cat`, `minecraft:ocelot`, `minecraft:rabbit`, `minecraft:polar_bear`, `minecraft:panda`, `minecraft:horse`, `minecraft:donkey`, `minecraft:mule`, `minecraft:skeleton_horse`, `minecraft:zombie_horse`, `minecraft:mooshroom`, `minecraft:trader_llama`, `minecraft:bat`, `minecraft:parrot`, `minecraft:squid`, `minecraft:glow_squid`, `minecraft:cod`, `minecraft:salmon`, `minecraft:tropical_fish`, `minecraft:pufferfish`, `minecraft:dolphin`, `minecraft:turtle`, `minecraft:bee`, `minecraft:strider`, `minecraft:fox`, `minecraft:wandering_trader` |
| Data | `assets/mcrs/reports/registries.json`, the data pack of the server jar, the type artefact |

This document states the goal lists and the target lists of the passive and the neutral mobs that
use goals and that `mob-goals.md`, `mob-wolf.md` and `mob-llama.md` do not state. It uses the goal
framework of `mob-goals.md` (prefix `GOAL`), the combat goals of `mob-combat-goals.md` (prefix
`CGOAL`) and the animal rules of `mob-animals.md` (prefix `ANIM`) with no change.

A mob whose behaviour needs a part of the engine that no specification states yet gets a hook: the
goal row names the later specification, and the row is absent from the goal list of the server
until that specification exists. A mob that moves in water or in the air uses a navigation that
`pathfinding.md` does not state; such a mob stands still on the server until its navigation exists.
This is a stated gap, not a deviation.

## 0. Conventions

### 0.1 Terms

- The terms of `mob-goals.md`, `mob-combat-goals.md` and `mob-animals.md`, section 0.1 each, apply.
- **Horse kind**: `horse`, `donkey`, `mule`, `skeleton_horse` and `zombie_horse`.
- **Fish**: `cod`, `salmon`, `tropical_fish` and `pufferfish`.
- **Swimmer**: a mob with a navigation in water: the fish, the squids, the dolphin, the turtle in
  water and the axolotl.
- **Flyer**: a mob with a navigation in the air: the bat, the parrot, the bee, the allay, the vex,
  the phantom and the ghast.

### 0.2 Provenance, kind, confidence and fidelity

The labels of `mob-goals.md`, sections 0.2 and 0.3, apply. The label `documentation` marks a rule
from the public documentation of the game and from the data of the type artefact and the attribute
table of the data pack.

### 0.3 ID prefix

| Prefix | Subject |
|--------|---------|
| `PAS-TYPE` | type data |
| `PAS-CAT` | the cat and the ocelot |
| `PAS-RAB` | the rabbit |
| `PAS-BEAR` | the polar bear |
| `PAS-PAN` | the panda |
| `PAS-HRS` | the horse kind |
| `PAS-MOO` | the mooshroom |
| `PAS-TLL` | the trader llama |
| `PAS-TRD` | the wandering trader |
| `PAS-FOX` | the fox |
| `PAS-TUR` | the turtle |
| `PAS-STR` | the strider |
| `PAS-SWIM` | the swimmers and the flyers |

## 1. Type data

**PAS-TYPE-001** (`documentation`, semantic, confidence: high, fidelity: exact). The dimensions,
the eye height and the fire immunity of each type come from the type artefact. The base attributes
come from the attribute table of the data pack. The values that the goals read:

| Type | `follow_range` | `attack_damage` | `movement_speed` | Tempt items |
|------|----------------|-----------------|------------------|-------------|
| `cat` | 16 | 3 | `0.3F` | `#cat_food` |
| `ocelot` | 16 | 3 | `0.3F` | `#ocelot_food` |
| `rabbit` | 16 | 3 | `0.3F` | `#rabbit_food`; also `carrot` as a block to eat (hook) |
| `polar_bear` | 20 | 6 | 0.25 | none |
| `panda` | 16 | 6 | `0.15F` | `#panda_food` |
| `horse`, `skeleton_horse`, `zombie_horse` | 16 | 2 | `0.225F` | `#horse_tempt_items` |
| `donkey`, `mule` | 16 | 2 | `0.175F` | `#horse_tempt_items` |
| `mooshroom` | 16 | 2 | `0.2F` | `#cow_food` |
| `trader_llama` | 16 | 2 | `0.175F` | `#llama_tempt_items` |
| `fox` | 16 | 2 | `0.3F` | none |
| `turtle` | 16 | 2 | 0.25 | `#turtle_food` |
| `strider` | 16 | 2 | `0.175F` | `#strider_tempt_items` |
| `wandering_trader` | 16 | 2 | 0.5 | none |
| `bat` | 16 | 2 | default | none |
| `parrot` | 16 | 3 | `0.2F` | `#parrot_food` |
| `dolphin` | 16 | 3 | `1.2F` | none |

Each type of the table that is an animal in the sense of ANIM has the age, the love and the breed of
ANIM with its own breeding food; the item sets are the item tags of the data pack with the name of
the type. The head turn speed, the pitch limit and the head yaw limit are the defaults of
GOAL-ANI-006 for each type.

## 2. The cat and the ocelot

**PAS-CAT-001** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of the
cat, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 1 | float (GOAL-FLT-001) | `jump` | |
| 1 | panic (GOAL-PNC-002) | `move` | `m = 1.5` |
| 1 | sit when ordered (CGOAL-OWNER-006) | `jump`, `move` | |
| 2 | relax on a bed | `move` | hook |
| 3 | tempt (GOAL-TMP-001) | `move`, `look` | `#cat_food`; `m = 0.6` |
| 5 | sit on a block | `move`, `jump` | hook |
| 6 | follow owner (CGOAL-OWNER-004) | `move`, `look` | `m = 1.0`; `a = 10`; `b = 5` |
| 7 | lie on a bed | `move` | hook |
| 8 | avoid entity (CGOAL-AVOID-001) | `move` | the players; `D = 16`; `m1 = 0.8`; `m2 = 1.33`; only a wild cat |
| 8 | breed (GOAL-BRD-001) | `move`, `look` | `m = 0.8` |
| 9 | stroll (GOAL-STR-001) | `move` | `m = 0.8` |
| 10 | leap at target (CGOAL-LEAP-001) | `jump`, `move` | `y0 = 0.3` |
| 11 | melee attack (CGOAL-MELEE-001) | `move`, `look` | `m = 1.0`; stops when the target is not seen |
| 11 | look at a player (GOAL-LKP-001) | `look` | range 10 |

The target list of the cat: the non-tame nearest attackable target goal (CGOAL-NEAR-007) at the
priority 1 for the type set `rabbit`, and at the priority 1 for the type set `turtle` with the
filter "the turtle is a baby and is not in water"; both with `r = 10` and must see.

**PAS-CAT-002** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of the
ocelot, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 1 | float (GOAL-FLT-001) | `jump` | |
| 1 | ocelot tempt (GOAL-TMP-001) | `move`, `look` | `#ocelot_food`; `m = 0.6`; the ocelot does not run from the player while it runs |
| 3 | leap at target (CGOAL-LEAP-001) | `jump`, `move` | `y0 = 0.3` |
| 4 | ocelot attack (CGOAL-MELEE-001) | `move`, `look` | `m = 1.0`; stops when the target is not seen |
| 5 | breed (GOAL-BRD-001) | `move`, `look` | `m = 0.8` |
| 6 | stroll (GOAL-STR-001) | `move` | `m = 0.8` |
| 7 | avoid entity (CGOAL-AVOID-001) | `move` | the players; `D = 16`; `m1 = 0.8`; `m2 = 1.33` |
| 9 | look at a player (GOAL-LKP-001) | `look` | range 10 |

The target list of the ocelot: the nearest attackable target goal at the priority 1 for the type
set `chicken`, and at the priority 1 for the type set `turtle` with the filter "the turtle is a
baby and is not in water"; both with `r = 10`, must see.

## 3. The rabbit

**PAS-RAB-001** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of the
rabbit, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 1 | float (GOAL-FLT-001) | `jump` | |
| 1 | rabbit panic (GOAL-PNC-002) | `move` | `m = 2.2` |
| 2 | breed (GOAL-BRD-001) | `move`, `look` | `m = 0.8` |
| 3 | tempt (GOAL-TMP-001) | `move`, `look` | `#rabbit_food`; `m = 1.0` |
| 4 | avoid entity (CGOAL-AVOID-001) | `move` | the players; `D = 8`; `m1 = 2.2`; `m2 = 2.2` |
| 4 | avoid entity | `move` | type set: `wolf`; `D = 10`; `m1 = 2.2`; `m2 = 2.2` |
| 4 | avoid entity | `move` | type set: each monster; `D = 4`; `m1 = 2.2`; `m2 = 2.2` |
| 5 | raid a garden | `move` | hook |
| 6 | stroll (GOAL-STR-001) | `move` | `m = 0.6` |
| 11 | look at a player (GOAL-LKP-001) | `look` | range 10 |

A rabbit of the variant `evil` has the melee goal at the priority 4 with `m = 1.4` and the target
goals hurt-by and nearest player; the variant is a hook. The rabbit moves by jumps: its jump
control is a hook of `pathfinding.md`; the server moves it as a walker until then.

## 4. The polar bear

**PAS-BEAR-001** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of
the polar bear, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 0 | float (GOAL-FLT-001) | `jump` | |
| 1 | melee attack (CGOAL-MELEE-001) | `move`, `look` | `m = 1.25`; follow a target that it does not see |
| 1 | panic (GOAL-PNC-002) | `move` | `m = 2.0`; only a baby panics |
| 4 | follow an adult (GOAL-FOL-001) | none | `m = 1.25` |
| 5 | stroll (GOAL-STR-001) | `move` | `m = 1.0` |
| 6 | look at a player (GOAL-LKP-001) | `look` | range 6 |
| 7 | look around (GOAL-LKR-001) | `move`, `look` | |

The target list of the polar bear, in this order:

| Priority | Goal | Parameters |
|----------|------|------------|
| 1 | hurt-by target (CGOAL-HURT-002) | alert |
| 2 | attack players near a baby | hook: the nearest attackable target goal for the players, `r = 10`, must see, only while a baby polar bear is within 8 blocks |
| 3 | nearest attackable target (CGOAL-NEAR-004) | the players; `r = 10`; must see; filter: the mob is angry at the player (CGOAL-RESET-ANGER-004) |
| 4 | nearest attackable target | type set: `fox`; `r = 10`; must see |
| 5 | reset universal anger (CGOAL-RESET-ANGER-005) | no alert |

The polar bear is a neutral mob with the anger duration uniform from 400 to 780 ticks. Its melee
hit knocks the target back with the strength of its type; the knockback is a hook (CGOAL U1).

## 5. The panda

**PAS-PAN-001** (`documentation`, semantic, confidence: medium, fidelity: exact). The goal list of
the panda, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 0 | float (GOAL-FLT-001) | `jump` | |
| 2 | panda panic (GOAL-PNC-002) | `move` | `m = 2.0`; only a baby or a worried panda panics |
| 2 | melee attack (CGOAL-MELEE-001) | `move`, `look` | `m = 1.0`; stops when the target is not seen |
| 3 | breed (GOAL-BRD-001) | `move`, `look` | `m = 1.0`; needs bamboo near (hook) |
| 4 | avoid entity (CGOAL-AVOID-001) | `move` | the players; `D = 8`; `m1 = 2.0`; `m2 = 2.0`; only a worried panda |
| 4 | avoid entity | `move` | type set: each monster; `D = 4`; `m1 = 2.0`; `m2 = 2.0`; only a worried panda |
| 5 | tempt (GOAL-TMP-001) | `move`, `look` | `#panda_food`; `m = 1.0` |
| 6 | lie on the back, sneeze, eat, roll, sit | `move` | hooks of the panda gene |
| 7 | follow an adult (GOAL-FOL-001) | none | `m = 1.25` |
| 8 | stroll (GOAL-STR-001) | `move` | `m = 1.0` |
| 9 | look at a player (GOAL-LKP-001) | `look` | range 6 |
| 10 | look around (GOAL-LKR-001) | `move`, `look` | |

The target list of the panda: the hurt-by target goal (CGOAL-HURT-002) at the priority 1 with the
alert; only an aggressive panda keeps the target, and a panda of another gene stops the target in
its next full evaluation. The genes are a hook.

## 6. The horse kind

**PAS-HRS-001** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of
the horse kind, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 0 | float (GOAL-FLT-001) | `jump` | |
| 1 | panic (GOAL-PNC-002) | `move` | `m = 1.2` |
| 1 | buck a rider | `move` | hook (LLAMA-HOOK-003) |
| 2 | breed (GOAL-BRD-001) | `move`, `look` | `m = 1.0`; the mates are tame adults of the horse kind |
| 4 | tempt (GOAL-TMP-001) | `move`, `look` | `#horse_tempt_items`; `m = 1.25` |
| 6 | follow an adult (GOAL-FOL-001) | none | `m = 1.0`; the adults are the horse kind |
| 7 | stroll (GOAL-STR-001) | `move` | `m = 0.7` |
| 8 | look at a player (GOAL-LKP-001) | `look` | range 6 |
| 9 | look around (GOAL-LKR-001) | `move`, `look` | |

The horse kind has an empty target list. The `skeleton_horse` and the `zombie_horse` have no tempt
goal and no breed goal. A horse that a player rides has no AI step (GOAL-FRM-009); the ride is a
hook.

## 7. The mooshroom and the trader llama

**PAS-MOO-001** (`documentation`, semantic, confidence: high, fidelity: exact). The mooshroom has
the goal list of the cow (GOAL-ANI-002) with the same parameters. Its shear and its stew are hooks of
the interactions.

**PAS-TLL-001** (`documentation`, semantic, confidence: high, fidelity: exact). The trader llama has
the goal list and the target list of the llama (LLAMA-GOAL-001, LLAMA-GOAL-002) with these
additions: the panic goal of the trader llama has `m = 2.0` when the trader is hurt (hook), the
target list adds the defence of its trader at the priority 1 (hook), and a trader llama without a
trader despawns after a timer (hook of `natural-spawning.md`).

## 8. The wandering trader

**PAS-TRD-001** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of the
wandering trader, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 0 | float (GOAL-FLT-001) | `jump` | |
| 0 | drink invisibility, drink milk | none | hooks |
| 1 | trade with a player | `move`, `look` | hook |
| 1 | look at the trading player | `look` | hook |
| 1 | panic (GOAL-PNC-002) | `move` | `m = 0.5` |
| 1 | avoid entity (CGOAL-AVOID-001) | `move` | type set: `zombie`, `evoker`, `vindicator`, `vex`, `pillager`, `illusioner`, `zoglin`; `D = 8`; `m1 = 0.5`; `m2 = 0.5` |
| 1 | avoid entity | `move` | type set: `zombie_villager`; `D = 8`; `m1 = 0.5`; `m2 = 0.5` |
| 2 | move to the wander target | `move` | hook |
| 2 | stroll (GOAL-STR-001) | `move` | `m = 0.35`; `L(10, 7)` only |
| 1 | interact with a player | `look` | hook |
| 2 | look at a player (GOAL-LKP-001) | `look` | range 8 |

The wandering trader has an empty target list. It never despawns by the distance rules; its own
despawn timer is a hook of `natural-spawning.md`.

## 9. The fox

**PAS-FOX-001** (`documentation`, semantic, confidence: medium, fidelity: exact). The goal list of
the fox, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 0 | float, fox form | `jump` | |
| 0 | fox panic (GOAL-PNC-002) | `move` | `m = 2.2` |
| 0 | breed (GOAL-BRD-001) | `move`, `look` | `m = 1.0` |
| 1 | flee from a player, a wolf and a polar bear | `move` | hook: the avoid goal with `D = 8`, `m1 = 1.6`, `m2 = 1.4`, with the trust of a player |
| 2 | sit and look | `move`, `look` | hook |
| 3 | pounce | `jump`, `move` | hook |
| 4 | melee attack (CGOAL-MELEE-001) | `move`, `look` | `m = 1.2`; follow a target that it does not see |
| 5 | follow the parent (GOAL-FOL-001) | none | `m = 1.25` |
| 6 | sleep | `move`, `look`, `jump` | hook |
| 7 | follow a player that holds food | `move`, `look` | hook |
| 8 | fox stroll (GOAL-STR-001) | `move` | `m = 1.0` |
| 9 | eat a berry bush | `move`, `jump` | hook |
| 10 | search for items | `move`, `look` | hook |
| 11 | look at a player (GOAL-LKP-001) | `look` | range 24 |
| 12 | perch and look around | `move`, `look` | hook |

The target list of the fox, in this order:

| Priority | Goal | Parameters |
|----------|------|------------|
| 3 | fox hurt-by | hook: the hurt-by target goal with the alert to the foxes of the same trust |
| 4 | nearest attackable target (CGOAL-NEAR-004) | type set: each animal of the prey of the fox: `chicken`, `rabbit`, `cod`, `salmon`, `tropical_fish`, `pufferfish`; `r = 10`; must see; filter: the fox is not sleeping (hook) |
| 6 | nearest attackable target | type set: `turtle`; `r = 10`; must see; filter: the turtle is a baby and is not in water |

A fox hunts at night. Until the day cycle reaches the server, the fox hunts at each time.

## 10. The turtle, the strider, the swimmers and the flyers

**PAS-TUR-001** (`documentation`, semantic, confidence: medium, fidelity: exact). The goal list of
the turtle, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 0 | turtle panic (GOAL-PNC-002) | `move` | `m = 1.2` |
| 1 | breed (GOAL-BRD-001) | `move`, `look` | `m = 1.0` |
| 1 | lay an egg | `move` | hook |
| 2 | tempt (GOAL-TMP-001) | `move`, `look` | `#turtle_food`; `m = 1.1` |
| 3 | go to the water | `move` | hook |
| 4 | go home | `move` | hook |
| 7 | travel in the water | `move` | hook |
| 8 | stroll (GOAL-STR-001) | `move` | `m = 1.0`; on land only |
| 9 | look at a player (GOAL-LKP-001) | `look` | range 8 |

The turtle has an empty target list. The turtle has an amphibious navigation, a hook of
`pathfinding.md`. On land the server moves it as a walker.

**PAS-STR-001** (`documentation`, semantic, confidence: high, fidelity: exact). The goal list of
the strider, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 1 | strider panic (GOAL-PNC-002) | `move` | `m = 1.65` |
| 2 | breed (GOAL-BRD-001) | `move`, `look` | `m = 1.0` |
| 3 | tempt (GOAL-TMP-001) | `move`, `look` | `#strider_tempt_items`; `m = 1.4` |
| 4 | go to the lava | `move` | hook |
| 5 | follow an adult (GOAL-FOL-001) | none | `m = 1.0` |
| 7 | stroll (GOAL-STR-001) | `move` | `m = 1.0`; the stroll of a strider prefers lava (hook) |
| 8 | look at a player (GOAL-LKP-001) | `look` | range 8 |
| 8 | look around (GOAL-LKR-001) | `move`, `look` | |

The strider has an empty target list. Its walk on lava and its shivering are hooks of
`entity-physics.md`.

**PAS-SWIM-001** (`documentation`, semantic, confidence: high, fidelity: exact). The goal lists of
the swimmers and the flyers: the fish, the squids, the dolphin, the bat, the parrot and the bee. Each
of these mobs uses a navigation in water or in the air that `pathfinding.md` does not state. Their
goal lists are hooks of the specification of that navigation. Until it exists, each of these types
has an empty goal list and an empty target list on the server, and stands still, except the
parrot and the bee, which have these land goals:

| Type | Priority | Goal | Flags | Parameters |
|------|----------|------|-------|------------|
| `parrot` | 0 | panic (GOAL-PNC-002) | `move` | `m = 1.25` |
| `parrot` | 1 | sit when ordered (CGOAL-OWNER-006) | `jump`, `move` | |
| `parrot` | 2 | follow owner (CGOAL-OWNER-004) | `move`, `look` | `m = 1.0`; `a = 5`; `b = 1` |
| `parrot` | 3 | look at a player (GOAL-LKP-001) | `look` | range 8 |
| `bee` | 1 | breed (GOAL-BRD-001) | `move`, `look` | `m = 1.0` |
| `bee` | 2 | tempt (GOAL-TMP-001) | `move`, `look` | `#bee_food`; `m = 1.25` |
| `bee` | 5 | follow an adult (GOAL-FOL-001) | none | `m = 1.25` |

## 11. Unresolved

- **U1**: each hook of this document names a later specification. The row of a hook is absent from
  the goal list of the server until that specification exists.
- **U2**: the trust of a fox, the gene of a panda, the variant of a rabbit and the tame state of a
  horse kind are server states that no specification states yet.
- **U3**: the swimmers and the flyers wait for their navigation.
