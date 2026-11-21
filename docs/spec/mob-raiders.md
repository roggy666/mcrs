# The raiders with goals: the pillager, the vindicator, the evoker, the illusioner, the ravager and the vex

Version: see `assets/minecraft/version.json`.

| Field | Value |
|-------|-------|
| Entity types | `minecraft:pillager`, `minecraft:vindicator`, `minecraft:evoker`, `minecraft:illusioner`, `minecraft:ravager`, `minecraft:vex` |
| Data | `assets/mcrs/reports/registries.json`, the data pack of the server jar, the type artefact |

This document states the goal lists and the target lists of the raiders that use goals, and the
rules that the raiders share. It uses the goal framework of `mob-goals.md` (prefix `GOAL`), the
combat goals of `mob-combat-goals.md` (prefix `CGOAL`) and the shared monster rules of
`mob-monsters.md` (prefix `MON`) with no change. The witch is a raider too; its lists are in
`mob-monsters.md`, section 9.

A raider as a member of a raid (the patrol, the walk to the raid, the hold of the ground, the
celebration, the captain and its banner) needs the raid data that no specification states yet.
Those goals are hooks of a later specification `raids.md`. The spells of the evoker and the
illusioner, the crossbow of the pillager, the roar and the stun of the ravager, the charge of the
vex and the door open of a raider are hooks too. The goal rows of the hooks are absent from the goal
lists of the server until their specification exists.

## 0. Conventions

### 0.1 Terms

- The terms of `mob-goals.md`, `mob-combat-goals.md` and `mob-monsters.md`, section 0.1 each,
  apply.
- **Raider**: `pillager`, `vindicator`, `evoker`, `illusioner`, `ravager` and `witch`. The vex is
  not a raider.
- **Raid prey**: `villager` and `wandering_trader`.
- **Raid alert**: the alert of CGOAL-HURT-004 whose set is the raiders instead of the own type.

### 0.2 Provenance, kind, confidence and fidelity

The labels of `mob-goals.md`, sections 0.2 and 0.3, apply. The label `documentation` marks a rule
from the public documentation of the game and from the data of the type artefact and the attribute
table of the data pack.

### 0.3 ID prefix

| Prefix | Subject |
|--------|---------|
| `RAID-TYPE` | type data |
| `RAID-SHR` | the rules that the raiders share |
| `RAID-PIL` | the pillager |
| `RAID-VIN` | the vindicator |
| `RAID-EVO` | the evoker |
| `RAID-ILL` | the illusioner |
| `RAID-RAV` | the ravager |
| `RAID-VEX` | the vex |

### 0.4 Scope

In scope: the goal list and the target list of each type, the shared rules of the raiders, and the
parameters of the goals that GOAL and CGOAL state.

Out of scope:

| Subject | Where |
|---------|-------|
| The goal framework, the controls, the queries | `mob-goals.md` |
| The target goals, the melee goal, the ranged goal, the avoid goal | `mob-combat-goals.md` |
| The raid: its waves, its members, its captain, the patrol, the celebration | `raids.md` (later); hooks here |
| The crossbow and its charge, the arrow | `projectiles.md`, `items.md`; hooks here |
| The spells: the fangs, the summon of a vex, the colour change of a sheep, the blindness, the mirror images | a later specification; hooks here |
| The roar, the stun and the block break of the ravager | a later specification; hooks here |
| The flight of the vex and its charge | `entity-physics.md`, `pathfinding.md` (later); hooks here |

## 1. Type data

**RAID-TYPE-001** (`documentation`, semantic, confidence: high, fidelity: exact). The dimensions,
the eye height and the fire immunity of each type come from the type artefact. The base attributes
come from the attribute table of the data pack. The values that the goals read:

| Type | `follow_range` | `attack_damage` | `movement_speed` | Other |
|------|----------------|-----------------|------------------|-------|
| `pillager` | default | 5 | `0.35F` | |
| `vindicator` | 12 | 5 | `0.35F` | |
| `evoker` | 12 | default | 0.5 | |
| `illusioner` | 18 | default | 0.5 | |
| `ravager` | default | 12 | 0.3 | `attack_knockback` 1.5, `knockback_resistance` 0.75, `step_height` 1 |
| `vex` | 16 | 4 | default | flies |

A value that the table does not give is the default of the attribute (`attributes.md`). The spawn
bonus of `follow_range` (ANIM-SPAWN-001) applies to each raider of the first spawn setup.

**RAID-TYPE-002** (`documentation`, semantic, confidence: medium, fidelity: exact). The head turn
speed, the pitch limit and the head yaw limit of each raider are the defaults of GOAL-ANI-006. The
malus rows of the raiders (PATH-MAL-001) are the default table.

## 2. The rules that the raiders share

**RAID-SHR-001** (`documentation`, semantic, confidence: high, fidelity: exact). The raid alert:
when the hurt-by target goal of a raider starts, the alert of CGOAL-HURT-004 goes to each raider in
the box, not only to the own type. Each receiver that has no attack target gets the attacker as its
attack target. The grant of CGOAL-HURT-005 checks each receiver again with "is a raider" in place
of "is of the same type".

**RAID-SHR-002** (`documentation`, semantic, confidence: medium, fidelity: exact). The shared
target rows of the pillager, the vindicator, the evoker and the illusioner, in this order after the
hurt-by target:

| Goal | Parameters |
|------|------------|
| nearest attackable target (CGOAL-NEAR-004) | the players; `r = 10`; must see |
| nearest attackable target | type set: the raid prey; `r = 10`; does not need to see |
| nearest attackable target | type set: `iron_golem`; `r = 10`; must see |

The two rows of the same priority are in this order in the list: the raid prey first, the iron
golem second.

**RAID-SHR-003** (`documentation`, semantic, confidence: medium, fidelity: exact). The raid rows
of each raider are hooks of `raids.md`: the open-door goal, the walk to the raid, the patrol, the
hold of the ground after a raid, the celebration, and the pick-up of a banner. Their priorities are
in the lists below for the order; the rows are absent on the server.

**RAID-SHR-004** (`design`, semantic, confidence: high, fidelity: exact). The vex has no goal and
no target goal that the server runs: its charge and its flight are hooks, and a vex stands still
and selects no target until its specification exists (PAS-SWIM-003 states the same form for the
flyers). A vex that a server
command summons stands where it is and does not fall through a block, by the physics of a flyer.

## 3. The pillager

**RAID-PIL-001** (`documentation`, semantic, confidence: medium, fidelity: exact for the order,
envelope for the crossbow). The goal list of the pillager, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 0 | float (GOAL-FLT-001) | `jump` | |
| 2 | open a door as a raider | `move` | hook (RAID-SHR-003) |
| 3 | the raid rows | | hook (RAID-SHR-003) |
| 3 | crossbow attack (RAID-PIL-003) | `move`, `look` | `m = 1.0`; `R = 8` |
| 8 | stroll (GOAL-STR-001) | `move` | `m = 0.6` |
| 9 | look at a player (GOAL-LKP-001) | `look` | range 15 |
| 10 | look around (GOAL-LKR-001) | `move`, `look` | |

**RAID-PIL-002** (`documentation`, semantic, confidence: medium, fidelity: exact). The target list
of the pillager, in this order:

| Priority | Goal | Parameters |
|----------|------|------------|
| 1 | hurt-by target (CGOAL-HURT-002) | the raid alert (RAID-SHR-001); no ignored type |
| 2, 3, 3 | the shared rows (RAID-SHR-002) | |

**RAID-PIL-003** (`documentation`, semantic, confidence: low, fidelity: envelope). The crossbow
attack is the ranged goal of CGOAL-RANGED-001 with `Imin = Imax = 40` and `R = 8`. The attack
action shoots the crossbow at the target; the charge of the crossbow, its arrow and the time of the
charge are hooks of `projectiles.md`. The envelope: at least 25 ticks pass between two shots at a
seen target in range, and the pillager keeps its distance while the target is in range and seen for
5 ticks (CGOAL-RANGED-002 step 2). The interval 40 is a stand-in until the crossbow specification
states the charge; it is not a measured value.

## 4. The vindicator

**RAID-VIN-001** (`documentation`, semantic, confidence: medium, fidelity: exact). The goal list of
the vindicator, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 0 | float (GOAL-FLT-001) | `jump` | |
| 1 | open a door as a raider | `move` | hook (RAID-SHR-003) |
| 2 | break a door as a raider | `move` | hook (RAID-SHR-003) |
| 3 | the raid rows | | hook (RAID-SHR-003) |
| 4 | melee attack (CGOAL-MELEE-001) | `move`, `look` | `m = 1.0`; follow only a seen target; `I = 20` |
| 8 | stroll (GOAL-STR-001) | `move` | `m = 0.6` |
| 9 | look at a player (GOAL-LKP-001) | `look` | range 3 |
| 10 | look around (GOAL-LKR-001) | `move`, `look` | |

**RAID-VIN-002** (`documentation`, semantic, confidence: medium, fidelity: exact). The target list
of the vindicator, in this order:

| Priority | Goal | Parameters |
|----------|------|------------|
| 1 | hurt-by target (CGOAL-HURT-002) | the raid alert (RAID-SHR-001); no ignored type |
| 2, 3, 3 | the shared rows (RAID-SHR-002) | |
| 4 | nearest attackable target: each living entity | hook: only a vindicator with the custom name `Johnny` has it; the custom name as a brain input is a later specification |

**RAID-VIN-003** (`documentation`, semantic, confidence: medium, fidelity: exact). The melee
attack of the vindicator raises its arms while the melee goal runs: the aggressive bit of the
entity data follows the start and the stop of the goal (the same form as MON-ZMB-003 and
MON-DEV-001).

## 5. The evoker

**RAID-EVO-001** (`documentation`, semantic, confidence: medium, fidelity: exact for the rows of
the server, envelope for the spells). The goal list of the evoker, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 0 | float (GOAL-FLT-001) | `jump` | |
| 1 | cast a spell | `move`, `look` | hook: the spell in progress |
| 2 | avoid an entity (CGOAL-AVOID-001) | `move` | set: the players; `d = 8`; `m_walk = 0.6`; `m_sprint = 1.0` |
| 3 | the raid rows | | hook (RAID-SHR-003) |
| 4 | summon vexes | `move`, `look` | hook |
| 5 | fangs | `move`, `look` | hook |
| 6 | colour a blue sheep | `move`, `look` | hook |
| 8 | stroll (GOAL-STR-001) | `move` | `m = 0.6` |
| 9 | look at a player (GOAL-LKP-001) | `look` | range 3 |
| 10 | look around (GOAL-LKR-001) | `move`, `look` | |

**RAID-EVO-002** (`documentation`, semantic, confidence: medium, fidelity: exact). The target list
of the evoker, in this order:

| Priority | Goal | Parameters |
|----------|------|------------|
| 1 | hurt-by target (CGOAL-HURT-002) | the raid alert (RAID-SHR-001); no ignored type |
| 2, 3, 3 | the shared rows (RAID-SHR-002) | |

**RAID-EVO-003** (`design`, semantic, confidence: high, fidelity: exact). The avoid goal of
CGOAL-AVOID-001 takes the players as its set: the candidates are the rows of the player table
whose feet are in the box of the goal, and the rest of the goal is unchanged. The probe of the
goal is "a qualified player exists" in place of "a mob of the set exists in the neighbour tiles".
On the server, an evoker with no spell keeps away from each player within 8 blocks and attacks no
one in melee; it has no melee goal.

## 6. The illusioner

**RAID-ILL-001** (`documentation`, semantic, confidence: medium, fidelity: exact for the rows of
the server, envelope for the spells). The goal list of the illusioner, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 0 | float (GOAL-FLT-001) | `jump` | |
| 1 | cast a spell | `move`, `look` | hook: the spell in progress |
| 3 | the raid rows | | hook (RAID-SHR-003) |
| 4 | mirror images | `move`, `look` | hook |
| 5 | blindness | `move`, `look` | hook |
| 6 | bow attack (CGOAL-RANGED-001) | `move`, `look` | `m = 0.5`; `I = 20`; `R = 15` |
| 8 | stroll (GOAL-STR-001) | `move` | `m = 0.6` |
| 9 | look at a player (GOAL-LKP-001) | `look` | range 3 |
| 10 | look around (GOAL-LKR-001) | `move`, `look` | |

The bow attack is the ranged goal with the arrow of `projectiles.md` as its attack action; the
form is that of the skeleton (MON-SKL-003), with one difference: the interval of the illusioner is
the fixed value 20, it does not read the difficulty (MON U1 does not apply).

**RAID-ILL-002** (`documentation`, semantic, confidence: medium, fidelity: exact). The target list
of the illusioner is the list of the evoker (RAID-EVO-002).

## 7. The ravager

**RAID-RAV-001** (`documentation`, semantic, confidence: medium, fidelity: exact for the rows of
the server, envelope for the roar and the stun). The goal list of the ravager, in this order:

| Priority | Goal | Flags | Parameters |
|----------|------|-------|------------|
| 0 | float (GOAL-FLT-001) | `jump` | |
| 3 | the raid rows | | hook (RAID-SHR-003) |
| 4 | melee attack (CGOAL-MELEE-001) | `move`, `look` | `m = 1.0`; follow a target that it does not see; `I = 20` |
| 5 | stroll (GOAL-STR-001) | `move` | `m = 0.4` |
| 6 | look at a player (GOAL-LKP-001) | `look` | range 6 |
| 10 | look around (GOAL-LKR-001) | `move`, `look` | |

**RAID-RAV-002** (`documentation`, semantic, confidence: medium, fidelity: exact). The target list
of the ravager, in this order:

| Priority | Goal | Parameters |
|----------|------|------------|
| 2 | hurt-by target (CGOAL-HURT-002) | the raid alert (RAID-SHR-001); no ignored type |
| 3 | nearest attackable target (CGOAL-NEAR-004) | the players; `r = 10`; must see |
| 4 | nearest attackable target | type set: the raid prey; `r = 10`; does not need to see |
| 4 | nearest attackable target | type set: `iron_golem`; `r = 10`; must see |

**RAID-RAV-003** (`documentation`, semantic, confidence: medium, fidelity: envelope). The roar
after a stun, the stun by a shield block, the break of the leaves and the crops on the path and
the riding of a raider are hooks of a later specification. The envelope: a ravager on the server
walks to its target and hits it in melee as CGOAL-MELEE states, with the attack damage 12 and the
knockback of its attributes; it breaks no block.

## 8. The vex

**RAID-VEX-001** (`documentation`, semantic, confidence: medium, fidelity: envelope). The goal list
of the vex: float at 0, the charge attack at 4 (hook), the move toward the owner or the bound
position at 8 and 9 (hook), look at a player at 10 (range 3), look around at 10. Its target list:
the hurt-by target at 1 (no alert), the target of its owner at 2 (hook), the nearest player at 3.
The vex flies and lives for a limited time after a summon; both are hooks. On the server a vex
stands still (RAID-SHR-004): its goal list and its target list are empty, so the rows above are
the order for the later specification, not rows of the server.

## 9. Intentional deviations

### RAID-DEV-001: The crossbow interval

- **Rule:** RAID-PIL-003.
- **Reference:** the charge of the crossbow sets the time between two shots.
- **This server:** the ranged goal with the fixed interval 40 until the crossbow specification
  exists.
- **Visible to the client:** yes, by the time between two arrows and by the absence of the charge
  pose.

## 10. Unresolved

- **U1**: each hook of this document names a later specification. The goal rows of the hooks are
  absent from the goal lists of the server until that specification exists.
- **U2**: the custom name `Johnny` of a vindicator (RAID-VIN-002) needs the custom name as an input
  of the brain; no specification states that input.
- **U3**: the sight rule of the raid prey row (RAID-SHR-002, "does not need to see") has the
  confidence medium; a parity test of a raider behind a wall decides it.
- **U4**: the player range of the brain rules of each raider is at least its follow range, so the
  player table of a turn holds each player that a target row can select; the illusioner, with the
  follow range 18, uses the range of the monsters (35).
