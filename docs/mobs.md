# Mobs with goals: design

Design note for the vanilla mobs that run on goal lists: the extension of the AI stage, the
generic goals, the brain of each mob type and the host side. Behaviour comes from
`docs/spec/mob-goals.md` (GOAL), `docs/spec/mob-combat-goals.md` (CGOAL), `docs/spec/mob-animals.md`
(ANIM), `docs/spec/mob-wolf.md` (WOLF), `docs/spec/mob-llama.md` (LLAMA), the specifications of the
other mob families under `docs/spec/mob-*.md`, and `docs/spec/entity-ai-timing.md` (AIT).

The note builds on the AI stage of `docs/physics-architecture.md`, section 5: decisions in the AI
stage beside the tick, reflexes in the hot tick, a versioned snapshot (the replica), late
application of the decisions, proposals for the pair actions. It assumes the decision of #1067:
mobs become ECS entities. Every part of this note that reads or writes a mob on the host goes
through `MobView`, `Intents` and `Decision`, so the migration of the rows changes the host once and
the brain crates not at all.

## 1. Classification of the state

| Value | Kind | Owner |
|-------|------|-------|
| the brain of a mob (`TableBrain`): the selectors of the goal list and the target list, the state of each goal | authoritative truth of the AI stage | `Stage<B>` in `mcrs_minecraft_brain`; one tile task writes the brains of its tile |
| `MobView` of a mob | materialized derived state: the snapshot of the body, the records and the guidance state that a decision reads (AIT-LAG-001) | the tile task of the hot tick writes it once per tick; `Replica` keeps `max_lag + 1` versions |
| the column index of a tile (`TileIndex`) | materialized derived state: column of 16 × 16 blocks → the views of the tile, for the box queries | the tile task of the hot tick builds it beside the views; it lives in the same `Replica` version |
| `PlayerRow`: feet point, eye height, flags, visibility, vehicle, the items in both hands, dimensions | materialized derived state (AIT-CDN-002, GOAL-QRY-005) | one system before the AI stage, `O(players)` |
| `ItemTables`: the members of each item tag that a goal reads | materialized from the data pack | one bake per registry change |
| attack target, melee state, ranged state, swell state, sit state | guidance state of the own mob (AIT, section 0.1): the hot tick reads it, a decision writes it at its application | the host (`MobControl`) |
| hurt-by record, last-hit record, damage record, idle time, anger end time | authoritative truth of the mob | the hot tick (reflexes); the brain sees them through `MobView` |
| `Intents`, `Decision` | temporal facts: what a decision asks the host to apply | the brain writes, the host applies or drops (AIT-LAG-005) |
| `Proposal` | temporal fact: a pair action that a grant checks at the merge point (AIT-CPL-002) | the brain proposes, one serial step grants |
| `MeleeHit`, `MobHit` | temporal facts: a hit that a reflex decided | the tile task appends, one serial step applies in key order |
| `Fact` | temporal fact: an event of the hot tick that wakes a brain | the hot tick writes, the stage reads once |

No goal state leaves the brain. No navigation state enters the brain: the brain knows only the
ticket and the progress of its last path request (`Applied`).

## 2. Crate map

| Crate | Layer | Bevy | Content |
|-------|-------|------|---------|
| `mcrs_minecraft_brain` (exists) | 3, rules, `no_std` | none | the stage, the replica with `TileIndex`, `MobView`, `PlayerRow`, `Neighbours`, `ItemTables`, `Intents` with the new intents, `Proposal`, `TurnScratch`, the selector |
| `mcrs_minecraft_goals` (exists) | 3, rules, `no_std` | none | the generic goals of GOAL and CGOAL as values with a start, a continue, a tick and a stop; the queries; the random position searches; `TableBrain`, the brain that runs a goal table |
| `mcrs_minecraft_vanilla_mobs` (new) | 5, no Bevy | none | the goal table and the type data of each vanilla mob type; `VanillaBrain`; the per-type reflex rules (anger, swell, caravan); the item tags that the tables read |
| `mcrs_minecraft_mob_control` (exists) | 3, rules | none | the reflexes of the guidance, now with the melee reflex, the ranged reflex, the swell reflex and the target slot |
| `mcrs_minecraft_engine_brain` (exists) | 4 | required | the host: observe, apply, steer, the hit queue, the grants, the player table, the facts |
| `mcrs_minecraft_server` (exists) | 7 | required | `VanillaMobBrainPlugin` with the brain table of `mcrs_minecraft_vanilla_mobs` |

`mcrs_minecraft_goals` loses `pig_brain.rs`: the four farm animals become rows of the table of
`mcrs_minecraft_vanilla_mobs`. The benchmark pen (`pig_pen.rs`, feature `testing`) runs the table
brain of the pig.

A goal is a generic rule (GOAL, CGOAL); a mob type is data (its table). This is the split of
`mcrs_minecraft_redstone` and `mcrs_minecraft_vanilla_redstone`: the rules stay usable by a server
with custom mobs, which composes its own tables from the same goals.

## 3. The brain of a mob type

```rust
pub struct GoalRow {
    pub priority: u8,
    pub flags: ControlFlags,
    pub goal: GoalKind,
}

pub struct GoalTable {
    pub goals: &'static [GoalRow],
    pub targets: &'static [GoalRow],
    pub rules: BrainRules,
}

pub struct TableBrain {
    table: &'static GoalTable,
    goals: Selector,
    targets: Selector,
    state: Vec<GoalState>,
}
```

- `GoalKind` is an enum with the parameters of the goal as data: `Panic { speed, causes }`,
  `Tempt { items, speed }`, `NearestTarget { kinds, interval, must_see, filter, follow_scale }`,
  `Melee { speed, follow_unseen, interval }`, and so on. A mob type is a `&'static GoalTable`.
- `GoalState` is an enum of the state of one goal: the gate, the timers, the kept entity. A state
  is small: a timer is a `u8` or a `u16` in evaluation units, an entity is an `EntityId`, a point
  that a goal keeps is a `BlockPos`. `size_of::<VanillaBrain>()` has a test with a limit.
- The brain runs the target list, then the goal list, with the selector of GOAL-FRM-002. Each goal
  kind states its reflex part and its decision part by AIT-SPL-001; the brain computes the decision
  part and writes intents; the host runs the reflex part.
- Per-type parts that are not goals (the anger update of a wolf, the swell counter of a creeper,
  the caravan links of a llama) are reflex rules on the host, selected by the type, or decisions in
  `TableBrain` through a `TypeRules` value of the table (a `fn` pointer, never `dyn`).

## 4. The snapshot and the queries

`Turn` gains three members:

- `neighbours: &Neighbours<'a>`: the views of the 5 × 5 tiles around the tile of the mob, from the
  replica version of the issue tick, with their column indexes. `collect_in(min, max, kinds, out)`
  reads only the columns that the box intersects and sorts the result by the order key, so a tie
  of GOAL-QRY-004 costs nothing. A follow range of 40 (LLAMA-TYPE-003) fits in the 5 × 5 reach of
  ±64 blocks with tiles of 32 blocks.
- `items: &ItemTables`: the members of each item tag that a table names, as bitsets over item ids.
- `scratch: &mut TurnScratch` (a parameter of `evaluate`, not of `Turn`): the buffers that a query
  fills. One scratch per task of the stage. The path search of a later branch adds its arena here.

`MobView` gains the values that the goals of GOAL and CGOAL read: the type as a `u16` id, the
dimensions, the age, the love time, the health, the velocity as `f32`, the attack target, the
hurt-by record, the owner, the tame and sitting flags. `size_of::<MobView>()` stays at or under
128 bytes with a test.

`PlayerRow` gains the item in each hand as a `u16` item id, the dimensions, the creative flag and
the hurt-by record of the player.

The replica never holds interior mutability. The tile tasks of the stage read it in parallel; the
hot tick publishes a new version per tile and the publish moves vectors that the tile task built.

## 5. Intents and the hot tick

| Intent | Reflex on the host |
|--------|--------------------|
| `path`, `look` (exist) | the follower, the move control, the look control |
| `target: Set(entity) / Clear` | the target slot of the mob; `MobView::target` of the next snapshot |
| `melee: Start { speed, interval, follow_unseen } / Stop` | the melee reflex: the look at the target, the path timer countdown, the cooldown, the reach test, one ray, the hit (CGOAL-MELEE-010) |
| `ranged: Start { speed, interval, range } / Stop` | the ranged reflex: the see count, the attack time, the ray, the attack action (CGOAL-RANGED-004) |
| `impulse: velocity` | a write of the own velocity at the application (CGOAL-LEAP-002) |
| `speed: modifier` | the speed modifier of the current path (CGOAL-AVOID-005) |
| `pose: aggressive / sitting / interested / swelling bits` | the entity data bits of the own mob |
| `proposal: Breed { partner } / Alert { target } / CaravanJoin { anchor } / CaravanLeave` | a grant at the merge point (AIT-CPL-002) |

A hit of the melee reflex is not a write of the row of the target from the tile task. The tile
task appends a `MeleeHit { attacker, target, hit }` to its output. After the parallel phase, one
serial step sorts the hits by (order key of the target, order key of the attacker) and applies
them through `hurt_mob`, which writes the hit record and the hurt-by record of the target. A hit on
a player becomes the message `MobHit`; the engine has no damage path for a player yet (#103, #116),
so the message is the plugin point for it. The grants of the proposals run in the same serial step,
in the order of (issue tick, order key of the proposer).

The cost of the serial step is proportional to the number of hits and proposals of the tick, not
to the population.

## 6. Path search

This branch keeps the straight route of `mcrs_minecraft_goal_world`. The combat goals ask for a
path to the target; a wall between them makes the mob stop at the wall. A later branch adds
`mcrs_minecraft_pathfind` with the ground node evaluator and the search of `docs/spec/pathfinding.md`,
a `Navigation` on the host and a node budget per tile. `PathIntent::Walk` keeps its shape, so that
branch changes the resolver of a request, not the goals.

## 7. Use cases

| User | How the design serves it |
|------|--------------------------|
| A minigame server with custom mobs | It writes a `GoalTable` from the goals of `mcrs_minecraft_goals` and gives it to `MobBrainPlugin`; no vanilla crate is needed |
| A server that removes a family | It omits the rows of that family from its table |
| A bot or a test | It runs `TableBrain` on a `MobView` with no Bevy and no body |
| A replay or a determinism check | `AiDelay::Fixed`, the keyed random streams and the order keys make a tick a function of the snapshot |

## 8. Steps

1. `mcrs_minecraft_brain`: `MobView`, `PlayerRow`, `Neighbours`, `TileIndex`, `ItemTables`,
   `TurnScratch`, the new intents and `Proposal`; the binary search of the facts.
2. `mcrs_minecraft_goals`: `TableBrain`, `GoalKind`, `GoalState`; the goals of GOAL that are
   missing (breed, tempt, follow an adult, eat grass); the goals of CGOAL.
3. `mcrs_minecraft_vanilla_mobs`: the tables of the four farm animals, the wolf and the llama; the
   move of the farm animals out of `mcrs_minecraft_goals`; `VanillaBrain`.
4. `mcrs_minecraft_mob_control` and `mcrs_minecraft_engine_brain`: the target slot, the melee and
   ranged reflexes, the hit queue, the grants, the player hands, the type data of each mob.
5. A specification and a table for each other family: `docs/spec/mob-monsters.md` for the
   monsters with goals, `docs/spec/mob-passive.md` for the other passive and neutral mobs with
   goals and `docs/spec/mob-raiders.md` for the raiders. A behaviour that needs a part of the engine that no specification states yet (a
   projectile, an explosion, a swim or fly navigation, the village data) is a hook: the row is
   absent from the table of the server until that specification exists.
6. The brain kernel of `docs/spec/mob-brain.md` for the villager, the piglin, the goat, the
   axolotl, the frog, the allay, the sniffer, the camel, the armadillo and the warden: a later
   branch.

## 9. Open points

- The hit on a player waits for the damage pipeline of `docs/damage.md`.
- The path search is the next branch (section 6).
- The save of the anger state, the tame state and the owner goes through the persisted registry
  (#159); no rule states the save of a mob yet.
