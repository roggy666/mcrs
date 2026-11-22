# Fluids: behavioural specification

Version: see `assets/minecraft/version.json`.

This document states water and lava on the server: the fluid state of a position, the fluid
tick, the flow from a position to its neighbours, the formation of a new source, the blocks that
water and lava form when they meet, the blocks that hold water, and the bubble columns.

`block-simulation.md` states the machinery that this document uses: the sets and their flags
(SIM-SET-002), the neighbor updates (SIM-NBR-001), the shape updates (SIM-SHP-001), the
scheduled fluid ticks (SIM-SCH-001 to SIM-SCH-011) and the random ticks (SIM-RND-002). This
document refers to these rules by id and does not repeat them. `block-placing.md`,
`block-breaking.md` and `falling-block.md` state what a player action or a falling block does to
a fluid; this document refers to them.

Out of scope:

- The random tick of lava, which lights fire. `block-simulation.md`, section 12, states fire.
- What a fluid does to an entity inside it, and the push of a flow. `entity-physics.md`
  states the inside actions.
- Sponges, ice, frosted ice, snow layers, water in a dimension where the environment attribute
  `gameplay/water_evaporates` is true, cauldrons and buckets.
- The client: the render of a fluid, its sounds and its particles.

## 0. Conventions

### 0.1 Terms

- **Set**, **flags**, **neighbor update**, **shape update**, **scheduled tick**, **placement
  side effect**, **removal side effect**, **chain**, **randomly ticking**, **section**: as
  `block-simulation.md` defines them. This document writes flags as numbers.
- **Fluid type**, **fluid state**, **source**, **flowing**, **level**: as `block-simulation.md`,
  section 0.1, defines them. The level is 1 to 8. A source has the level 8.
- **Falling**: a flowing state with the level 8. A falling state has fluid of the same type
  directly above it, or had it when the state was computed.
- **Fluid block**: a block state of `minecraft:water` or `minecraft:lava`. Its property `level`
  is the **block level**, 0 to 15. FLD-STA-001 maps the block level to a fluid state.
- **Fluid state of a position**: the fluid state that the block state at the position gives
  through its component `mcrs:fluid_state`. A fluid block gives its own state. A waterlogged block
  and a bubble column give a water source. Each other block state gives `minecraft:empty`.
- **Waterloggable block**: a block whose states have the property `waterlogged`. A
  **waterlogged block** is such a state with `waterlogged` = true.
- **Fluid tick**: a scheduled fluid tick (SIM-SCH-001) that runs at a position whose fluid state
  is of the type of the tick (SIM-SCH-007).
- **Tick period**: the delay of the fluid tick that a fluid schedules for itself (FLD-TCK-002).
- **Drop**: the level that a fluid loses from one position to the next horizontal position
  (FLD-FLW-001).
- **Slope distance**: the number of positions that the search for a hole looks ahead
  (FLD-FLW-004).
- **Open position** for a fluid: a position whose block the fluid can replace (FLD-FLW-003).
- **Hole**: an open position below a horizontal neighbour, or a position that holds the same
  fluid below it (FLD-FLW-004).
- **New state of a position**: the fluid state that FLD-FLW-002 computes for a position from its
  neighbours.
- **Spread**: the write of a fluid from a position into a neighbour (FLD-FLW-005).
- **Conversion**: the change of a fluid block to a solid block when water and lava meet
  (FLD-CNV-001 to FLD-CNV-003).
- **Bubble column**: a block state of `minecraft:bubble_column`. Its property `drag` is false in
  an **upward** column and true in a **downward** column.
- **World rule**: a value of the registry `minecraft:game_rule` that a server operator changes
  (FLD-RUL-001).
- **Fast lava**: the value true of the environment attribute `minecraft:gameplay/fast_lava` at
  the position (`environment-attributes.md`).
- **`r(n)`**: a uniform integer draw in `[0, n)`. A draw occurs only where a rule names it.
  FLD-TCK-004 gives its source.

### 0.2 Provenance

- `black_box_confirmed`: shown by a game-produced artefact that is cited by path. An experiment
  of section 13 produced the artefact, or the game shipped it.
- `source_suggested_black_box_confirmed`: taken from the reference behaviour, and consistent
  with an artefact cited by path.
- `source_suggested`: taken from the reference behaviour, with no artefact yet. Confidence is at
  most medium, and section 12 names the experiment that decides it.
- `documentation`: taken from public documentation of player-facing behaviour.
- `design`: a choice of this project. The rule states the reason.
- `derived`: it follows from other rules of this document, through the argument in the rule.
- `unresolved`: not decided. Section 12 collects these items.

Each rule states its kind, as `docs/spec/README.md` defines it.

### 0.3 Artefacts cited

- `../../assets/mcrs/block_definition/water.json` and `lava.json`: the property `level` of the
  fluid blocks and the component `mcrs:fluid_state` of each block level.
- `../../assets/mcrs/block_definition/oak_slab.json`: the component `mcrs:fluid_state` of a
  waterlogged block.
- `../../assets/mcrs/block_definition/bubble_column.json`: the property `drag` and the component
  `mcrs:fluid_state` of a bubble column.
- `../../assets/mcrs/block_definition/soul_sand.json` and `magma_block.json`: the two bases of a
  bubble column.
- `../../assets/mcrs/registry_definition/game_rule.json`: the world rules
  `minecraft:water_source_conversion` and `minecraft:lava_source_conversion` with their defaults.
- `../../assets/mcrs/reports/registries.json`: the registry `minecraft:fluid`.
- `../../assets/mcrs/reports/packets.json`: the packet `minecraft:level_event` (play clientbound
  id 47).
- The fixtures under `FIXTURE_DIR/`: the records of the experiments of section 13. Each record
  holds the commands of the scene, and for each observed tick the block changes and the pending
  fluid ticks.

### 0.4 ID prefixes

| Prefix | Subject |
|--------|---------|
| `FLD-STA` | the fluid state of a position and the fluid blocks |
| `FLD-TCK` | the fluid tick: period, schedule, delay |
| `FLD-FLW` | the flow: levels, the new state, open positions, holes, spread |
| `FLD-SRC` | the formation of a source and the retreat of a flow |
| `FLD-CNV` | the blocks that water and lava form |
| `FLD-WLG` | waterlogged blocks |
| `FLD-BUB` | bubble columns |
| `FLD-RUL` | the world rules and the environment attribute of this domain |
| `FLD-FTP` | the footprint of each handler of this domain |
| `FLD-DEV` | intentional deviations (section 11) |

### 0.5 Directions

The directions are `D` (down), `U` (up), `N`, `S`, `W` and `E`. A position P plus a direction d
is the neighbour P + d. The **horizontal neighbours** of P are P + N, P + S, P + W and P + E.
Where a rule of this document visits the horizontal neighbours, the result does not depend on
the order of the visit, except where the rule states an order.

---

## 1. Fluid state

### FLD-STA-001: Block level and fluid state

- **Behavior:** a fluid block with the block level `b` gives this fluid state of its type:

| Block level `b` | Fluid state |
|-----------------|-------------|
| 0 | source, level 8 |
| 1 to 7 | flowing, level `8 − b` |
| 8 to 15 | falling: flowing, level 8 |

  The fluid block of a fluid state is the inverse: a source gives `b` = 0, a flowing state with
  the level `l` < 8 gives `b` = `8 − l`, and a falling state gives `b` = 8. The server writes
  only the block levels 0 to 8. A block level 9 to 15 comes only from a command or a save, and
  the server treats it as 8.
- **Edge cases:** a position outside the vertical extent, or in a section that is not loaded,
  has the fluid state `minecraft:empty` (SIM-SET-003).

Provenance: `black_box_confirmed` by `water.json` and `lava.json` (the component
`mcrs:fluid_state` of each block level) and by the registry `minecraft:fluid` of
`registries.json`. Kind: semantic. Confidence: high.

### FLD-STA-002: Blocks that hold a fluid

- **Behavior:** the fluid state of a position comes from its block state:
  - a fluid block: FLD-STA-001;
  - a waterlogged block: a water source (FLD-WLG-001);
  - a bubble column: a water source (FLD-BUB-001);
  - each other block state: `minecraft:empty`.

  A block state that is not a fluid block never holds lava, and never holds flowing water.

Provenance: `black_box_confirmed` by `oak_slab.json` and `bubble_column.json` (the component
`mcrs:fluid_state` appears only on the waterlogged permutation, and only as a water source).
Kind: semantic. Confidence: high.

---

## 2. The fluid tick

### FLD-TCK-001: The steps of a fluid tick

- **Inputs:** a fluid tick at P of the type F. S is the fluid state of P when the tick runs.
- **Behavior:** the tick does these steps in this order:
  1. If S is a source, go to step 3.
  2. Let N be the new state of P (FLD-FLW-002).
     - If N is `minecraft:empty`: set P to `minecraft:air` with flags 3. S becomes empty.
     - Else, if N is not S: set P to the fluid block of N with flags 3. Schedule the fluid tick
       (P, F) with the delay of FLD-TCK-003. S becomes N.
     - Else: P does not change.
  3. If S is not empty: spread from P with the state S (FLD-FLW-005).
- **Ordering constraints:** each set completes, with its chain, before the next step
  (SIM-SET-002). The sets of step 3 are part of the tick.
- **Edge cases:**
  - When step 2 removes P, step 3 does nothing. A position that became air spreads nothing.
  - When P does not change in step 2 and step 3 writes nothing, the tick schedules no tick. A
    flow that reached its final shape has no pending fluid tick (E-FL1: no fluid tick is pending
    from tick 40 on).
  - The tick of a source never changes P. It only spreads.
- **Conformance tests:** E-FL1, E-FL2, E-FL8.

Provenance: `source_suggested_black_box_confirmed` by E-FL1 and E-FL8 (`water_flat_floor.json`,
`water_source_removed.json`). Kind: semantic. Confidence: high.

### FLD-TCK-002: Tick period

- **Behavior:** the tick period of a fluid at P is:

| Fluid | Fast lava at P | Tick period |
|-------|----------------|-------------|
| water | any | 5 |
| lava | false | 30 |
| lava | true | 10 |

  The period is the delay of each fluid tick that this document schedules, except the delay of
  FLD-TCK-003 step 2. The attribute is the switch, not the dimension: a dimension whose
  environment sets `gameplay/fast_lava` to true has fast lava, and the nether of the vanilla
  data pack is such a dimension.
- **Conformance tests:** E-FL1 (water: a new ring of levels every 5 ticks), E-FL2 (lava: the
  first spread at tick 30), E-FL3.

Provenance: water `source_suggested_black_box_confirmed` by E-FL1 (`water_flat_floor.json`).
Lava in the overworld `source_suggested_black_box_confirmed` by E-FL2 (`lava_trench_source.json`:
the middle of the trench changes at tick 30 and at tick 60). Lava with fast lava
`source_suggested`; experiment E-FL3 (section 12, U-01). Kind: semantic. Confidence: high for
water and for lava without fast lava, medium for fast lava.

### FLD-TCK-003: The delay after a change

- **Inputs:** a fluid tick at P that changes P from the flowing state S to the flowing state N
  (FLD-TCK-001 step 2).
- **Behavior:** the delay of the tick that step 2 schedules is the tick period, except in this
  case:
  1. The fluid is lava, S and N are both flowing and not falling, and the level of N is higher
     than the level of S.
  2. Then draw `r(4)`. If the draw is not 0, the delay is four times the tick period.

  As a result, lava that rises at a position waits 120 ticks (40 with fast lava) with the
  probability 3/4, and 30 ticks (10 with fast lava) with the probability 1/4, before its next
  tick.
- **Edge cases:** water never uses the long delay. A fall (N falling) and a retreat (N lower
  than S) never use it.
- **Conformance tests:** E-FL3.

Provenance: `source_suggested`; experiment E-FL3 (section 12, U-02). Kind: semantic. Confidence:
medium.

### FLD-TCK-004: Who schedules a fluid tick

- **Behavior:** these events schedule the fluid tick (P, F) with the tick period as its delay,
  where F is the fluid type of the fluid state of P:
  1. The placement side effect of a set that puts a fluid block at P (SIM-SET-002 step 2.7),
     when the set did not end in a conversion (FLD-CNV-001). A set by a command, by a spread, by
     a bucket or by a tick all count.
  2. The neighbor-changed handler of a fluid block at P (SIM-NBR-001), when it did not end in a
     conversion.
  3. A shape update that a waterlogged block or a bubble column receives (FLD-WLG-003,
     FLD-BUB-003).
  4. The fluid tick itself, when it changes P (FLD-TCK-001 step 2; the delay of FLD-TCK-003).

  No other event schedules a fluid tick. A fluid tick that is already pending absorbs a second
  schedule of the same (P, F) (SIM-SCH-004). As a result, the work of a flow is proportional to
  the positions that change, not to the positions that hold a fluid.
- **Edge cases:**
  - The random tick of lava (SIM-RND-002) schedules no fluid tick.
  - A set with flags that have no *notify-neighbors* bit and no placement side effect (for
    example flags 530) puts a fluid that does not flow until an event of the list touches it.
  - A source that a set puts at P has a pending tick with the delay of the period (E-FL1: one
    pending `minecraft:water` tick with `t` = 5 at tick 0). A source whose neighbours change
    gets a new tick through event 2.
- **Conformance tests:** E-FL1, E-FL8.

Provenance: events 1, 2 and 4 `source_suggested_black_box_confirmed` by E-FL1 and E-FL8
(`water_flat_floor.json`: the pending fluid ticks after each set; `water_source_removed.json`:
the neighbours of a removed source change 5 ticks after the removal). Event 3
`source_suggested`; experiments E-FL6 and E-FL7 (section 12, U-05, U-07). Kind: semantic.
Confidence: high for events 1, 2 and 4, medium for event 3.

### FLD-TCK-005: The random draw

- **Behavior:** the draw `r(4)` of FLD-TCK-003 comes from the random stream of the fluid tick
  that makes it, as SIM-FIRE-011 states for a scheduled tick: the key of the stream is the
  dimension seed, the game time of the step, the kind (fluid), the type of the tick and its
  position. In vanilla, the draw comes from the dimension random (SIM-RNG-001). Parity with
  vanilla is by distribution only.
- **Postconditions:** the same stored state, the same inputs and the same seed give the same
  flow, for each count of threads.

Provenance: `derived` from SIM-RNG-001 and SIM-FIRE-011. Kind: unspecified for parity with
vanilla; the determinism is a requirement of this project. Confidence: high.

---

## 3. Flow

### FLD-FLW-001: Levels and the drop

- **Behavior:** the drop of a fluid at P is:

| Fluid | Fast lava at P | Drop | Slope distance |
|-------|----------------|------|----------------|
| water | any | 1 | 4 |
| lava | false | 2 | 2 |
| lava | true | 1 | 4 |

  A flowing state at a horizontal neighbour of a position with the level `l` has at most the
  level `l − drop` (FLD-FLW-002). A level of 0 or less is no fluid. As a result, on a flat floor
  water reaches 7 positions from a source (levels 7 to 1), lava reaches 3 positions (levels 6, 4,
  2), and lava with fast lava reaches 7 positions.
- **Conformance tests:** E-FL1 (`water_flat_floor.json`: at tick 100, the block levels 1 to 7
  at the Manhattan distances 1 to 7 from the source, and nothing at the distance 8), E-FL3.

Provenance: water `source_suggested_black_box_confirmed` by E-FL1. Lava drop 2
`source_suggested_black_box_confirmed` by E-FL2 (`lava_trench_source.json`: the middle of a
trench between two lava sources has the block level 2, that is the level 6). The slope distances
and fast lava `source_suggested`; experiments E-FL3 and E-FL10 (section 12, U-01, U-03). Kind:
semantic. Confidence: high for the drops of water and lava, medium for the slope distances.

### FLD-FLW-002: The new state of a position

- **Inputs:** a position Q and a fluid type F. The block state at Q.
- **Behavior:** the new state of Q for F is computed from the neighbours of Q in these steps:
  1. Let `m` be the highest level among the horizontal neighbours of Q whose fluid state is of
     the type F and whose face toward Q is open (FLD-FLW-003 step 3). Let `s` be the count of
     those neighbours that are sources. If no such neighbour exists, `m` = 0 and `s` = 0.
  2. If `s` ≥ 2, the world rule of FLD-SRC-001 allows a conversion to a source, and the block
     below Q is solid or holds a source of F, the new state is a source of F.
  3. Else, if the fluid state of Q + U is of the type F and the face between Q and Q + U is open,
     the new state is falling.
  4. Else, if `m − drop` > 0, the new state is flowing with the level `m − drop`. The drop is
     the drop at Q (FLD-FLW-001).
  5. Else the new state is `minecraft:empty`.

  The fluid state of Q itself is not an input. The new state is a function of the six
  neighbours of Q, of the block below Q and of the world rule.
- **Edge cases:**
  - A falling state above a floor gives its horizontal neighbours the level `8 − drop`, as a
    source does. The spread from a falling position is in FLD-FLW-005.
  - "Solid" in step 2 is the property of a block state that its definition sets. A full cube,
    glass and a slab are solid; air, a fluid and a plant are not. Section 12 (U-04) lists the
    open question for the partial blocks.
- **Conformance tests:** E-FL2 (`water_trench_source.json`: the middle of a 1 × 3 trench becomes
  a source at tick 5; the middles of a 1 × 4 trench stay at the level 7; a middle above a water
  source becomes a source; a middle above air stays flowing and falls).

Provenance: `source_suggested_black_box_confirmed` by E-FL1, E-FL2 and E-FL8. Kind: semantic.
Confidence: high; the solid test of step 2 medium.

### FLD-FLW-003: Open positions

- **Inputs:** a fluid F at P, a direction d and the neighbour Q = P + d.
- **Behavior:** F can flow from P into Q when all these conditions are true:
  1. The fluid state of Q permits the replacement:
     - `minecraft:empty`: yes;
     - water: only when F is lava and d = D (the lava then forms stone, FLD-CNV-002);
     - lava: never. Water never displaces lava.
  2. The block state at Q can hold F:
     - a fluid block or `minecraft:air`: yes;
     - a waterloggable block: only when F is water and the block is not waterlogged
       (FLD-WLG-004);
     - a block of these types: no: each door, each sign and hanging sign, `minecraft:ladder`,
       `minecraft:sugar_cane`, `minecraft:bubble_column`, `minecraft:nether_portal`,
       `minecraft:end_portal`, `minecraft:end_gateway`, `minecraft:structure_void`;
     - each other block state: yes when its collision box is empty (the component
       `minecraft:collision_box` of its definition), no otherwise. A torch, a rail, a flower and
       short grass take fluid; a carpet does not.
  3. The face between P and Q is open: neither the block at P nor the block at Q covers the full
     face between them with its collision box. A fluid block, air and a plant cover no face.
- **Edge cases:** when Q is a flowing state of F, condition 1 is false. The fluid at Q is not
  written by P; Q computes its own new state in its tick (FLD-FLW-002). This is why a flow
  retreats position by position when its source is gone (FLD-SRC-002).
- **Conformance tests:** E-FL1 (water flows into air), E-FL2 (the middle of a trench, already
  flowing, is not written by the ends; the retreat of E-FL8), E-FL5, E-FL9.

Provenance: conditions 1 (empty and flowing) and 2 (air) `source_suggested_black_box_confirmed`
by E-FL1 and E-FL8. The rest `source_suggested`; experiments E-FL5 and E-FL9 (section 12,
U-06, U-08). Kind: semantic. Confidence: high for air and the same fluid, medium for the other
rows.

### FLD-FLW-004: Holes and the slope search

- **Inputs:** a position P with the fluid F, and its horizontal neighbours.
- **Behavior:**
  - A position Q is a **hole** for F when the face between Q and Q + D is open and the fluid
    state of Q + D is of the type F, or the block at Q + D can hold F (FLD-FLW-003 condition 2).
  - The **slope** of a horizontal neighbour Q of P is:
    - 0, when Q is a hole;
    - else the smallest number of horizontal steps from Q to a hole, over positions that F can
      pass (FLD-FLW-003 conditions 2 and 3, and the position is not a source of F), without a
      step back toward the position that the search came from, up to the slope distance;
    - 1000 when no hole is in reach.
  - The **spread directions** of P are the horizontal directions d whose neighbour P + d has the
    smallest slope among the four neighbours, and that F can pass. When no hole is in reach of
    any neighbour, each passable direction is a spread direction.
- **Numeric semantics:** the search from a neighbour at the distance 1 from P looks up to the
  slope distance of steps. As a result, a hole at the horizontal distance `slope distance + 1`
  from P turns the flow, and a hole one position farther does not.
- **Edge cases:** the search reads only the positions that it steps over, their blocks below and
  the faces between them. It does not read the level of a fluid on its way.
- **Conformance tests:** E-FL10: a water source on a flat floor with one hole at the distance 5
  flows only toward the hole; with the hole at the distance 6 the water flows in each
  direction.

Provenance: `source_suggested`; experiment E-FL10 (section 12, U-03). Kind: semantic.
Confidence: medium.

### FLD-FLW-005: The spread

- **Inputs:** a position P with the fluid state S of the type F, from FLD-TCK-001 step 3.
- **Behavior:** the spread does these steps:
  1. Let B = P + D. If F can flow from P into B (FLD-FLW-003 with d = D): write B (step 4) with
     the new state of B. Then, if P has at least 3 horizontal neighbours that are sources of F,
     spread to the sides (step 3). The spread ends.
  2. Else, if S is a source, or B is not a hole for F: spread to the sides (step 3). Else the
     spread ends: a falling flow above a hole, or above the same fluid, does not spread to the
     sides.
  3. *Spread to the sides.* Let `n` be `8 − drop` when S is falling, else `level(S) − drop`. If
     `n` ≤ 0, nothing occurs. Else, for each spread direction d of P (FLD-FLW-004) such that F can
     flow from P into Q = P + d (FLD-FLW-003): write Q with the new state of Q, when that state
     is not empty.
  4. *Write Q with the state N.* If the block at Q is a waterloggable block: FLD-WLG-004. Else:
     if the block at Q is not air, destroy it (FLD-FLW-006). Then set Q to the fluid block of N
     with flags 3. The placement side effect of that set schedules the fluid tick of Q
     (FLD-TCK-004 event 1), or converts the block (FLD-CNV-001).
- **Ordering constraints:** step 1 before step 3. The order of the directions in step 3 is not
  observable: each write puts the new state of Q, and that state does not depend on the writes
  of the other directions in the same spread. The new state of Q reads the neighbours of Q at the
  time of the write (E-FL8: the position of a removed source gets the level 6 from a neighbour
  that still has the level 7, and the level 4 in its own tick 5 ticks later).
- **Edge cases:**
  - The state that a write puts at Q is the new state of Q, not `n`. The two are equal when P is
    the only neighbour of Q with fluid. When Q has two source neighbours, the write puts a
    source at Q at once (E-FL2: the middle of a 1 × 3 trench is a source at tick 5, in the
    tick of the ends).
  - A falling flow that reaches a floor spreads to the sides with `n` = `8 − drop`: water gives
    the level 7, lava the level 6.
- **Conformance tests:** E-FL1, E-FL2, E-FL4, E-FL8.

Provenance: steps 3 and 4 for air `source_suggested_black_box_confirmed` by E-FL1, E-FL2 and
E-FL8. Steps 1 and 2 `source_suggested`; experiment E-FL4 (section 12, U-09). Kind: semantic.
Confidence: high for the sideways spread, medium for the fall.

### FLD-FLW-006: The block that a flow displaces

- **Inputs:** a write of FLD-FLW-005 step 4 at a position Q whose block is not air and not a
  waterloggable block.
- **Behavior:**
  - Water drops the loot of the block at Q (the drops of SIM-SET-004, with no tool), then the
    set replaces the block. A player sees the item of the block appear.
  - Lava replaces the block with no drop. The step records the level event 1501 at Q
    (SIM-EFX-001). A player hears the fizz and sees smoke.
- **Edge cases:** the block at Q has no removal side effect that leaves another block, because
  only a block with an empty collision box or a fluid block is open (FLD-FLW-003).
- **Conformance tests:** E-FL9.

Provenance: `source_suggested`; experiment E-FL9 (section 12, U-08). Kind: semantic.
Confidence: medium.

---

## 4. Sources

### FLD-SRC-001: Formation of a source

- **Behavior:** a position Q becomes a source of F when its new state is a source (FLD-FLW-002
  step 2): at least two horizontal neighbours of Q are sources of F, the block below Q is solid
  or holds a source of F, and the world rule allows it. The world rule is
  `minecraft:water_source_conversion` for water (default true) and
  `minecraft:lava_source_conversion` for lava (default false) (FLD-RUL-001). Q becomes a source
  in the tick that writes Q: the spread of a neighbour (FLD-FLW-005 step 4) or the tick of Q
  (FLD-TCK-001 step 2).
- **Edge cases:**
  - Two sources at the ends of a 1 × 3 trench make the middle a source 5 ticks after their
    placement (water). With `water_source_conversion` false, the middle stays at the level 7.
  - Two sources with two positions between them make no source: each middle has one source
    neighbour.
  - Two lava sources at the ends of a 1 × 3 trench make the middle a lava source 30 ticks after
    their placement only when `lava_source_conversion` is true. With the default, the middle
    stays at the level 6.
  - A middle above a hole does not become a source. It becomes flowing, and the hole below it
    becomes falling.
- **Conformance tests:** E-FL2.

Provenance: `source_suggested_black_box_confirmed` by E-FL2 (`water_trench_source.json`,
`water_trench_no_conversion.json`, `lava_trench_source.json`). Kind: semantic. Confidence: high.

### FLD-SRC-002: Retreat of a flow

- **Behavior:** when a set removes a source or lowers a level at P, each fluid neighbour of P
  gets a neighbor update and schedules its fluid tick (FLD-TCK-004 event 2). In that tick, the
  neighbour computes its new state (FLD-FLW-002) from its own neighbours. A neighbour with no
  fluid neighbour of a higher level becomes empty, and its set gives the next ring its neighbor
  updates. As a result, the flow retreats one position in each tick period, from the removed
  source outward, and a position with another path to a source keeps the level of that path.
- **Edge cases:** a removed source in the middle of its flow is written again by its
  neighbours in their first tick, with the level that they still have minus the drop. It
  retreats with them in the ticks that follow (FLD-FLW-005, Ordering constraints).
- **Conformance tests:** E-FL8 (`water_source_removed.json`).

Provenance: `source_suggested_black_box_confirmed` by E-FL8. Kind: semantic. Confidence: high.

---

## 5. Blocks that water and lava form

### FLD-CNV-001: Lava that meets water

- **Inputs:** the placement side effect of a set that puts a lava block at P, and the
  neighbor-changed handler of a lava block at P.
- **Behavior:** the handler visits the positions P + U, P + N, P + S, P + W and P + E. If the
  fluid state of one of them is water:
  - when the fluid state of P is a source, set P to `minecraft:obsidian`;
  - else set P to `minecraft:cobblestone`.

  The set uses flags 3. The handler records the level event 1501 at P (SIM-EFX-001). The handler
  then schedules no fluid tick at P.
- **Edge cases:**
  - Water below a lava block converts nothing: P + D is not visited. Lava above water converts
    the water, not itself (FLD-CNV-002).
  - Water never flows into the lava position (FLD-FLW-003). A water flow that reaches a
    neighbour of lava converts the lava at the moment of its own set, inside the chain of that
    set, in the same tick.
  - A lava source that a set puts next to water becomes obsidian in the same set.
- **Conformance tests:** E-FL5.

Provenance: `source_suggested`; experiment E-FL5 (section 12, U-06). The player-facing result
(obsidian from a source, cobblestone from a flow) is `documentation`. Kind: semantic.
Confidence: medium.

### FLD-CNV-002: Lava that falls into water

- **Inputs:** a spread of lava from P into B = P + D (FLD-FLW-005 step 1) where the fluid state
  of B is water.
- **Behavior:** the write sets B to `minecraft:stone` with flags 3 and records the level event
  1501 at B. No lava is written at B.
- **Edge cases:** this is the only case where lava displaces water (FLD-FLW-003 condition 1).
  Lava that flows to the side of water does not enter it; the lava block converts by
  FLD-CNV-001 at its own set.
- **Conformance tests:** E-FL5.

Provenance: `source_suggested`; experiment E-FL5 (section 12, U-06). The player-facing result
is `documentation`. Kind: semantic. Confidence: medium.

### FLD-CNV-003: Basalt

- **Inputs:** the same handler as FLD-CNV-001.
- **Behavior:** when the block below P is `minecraft:soul_soil`, and one of the five visited
  positions holds `minecraft:blue_ice`, the handler sets P to `minecraft:basalt` with flags 3
  and records the level event 1501 at P. The water test of FLD-CNV-001 comes first at each
  visited position.
- **Conformance tests:** E-FL5.

Provenance: `source_suggested`; experiment E-FL5 (section 12, U-06). The player-facing result
is `documentation`. Kind: semantic. Confidence: medium.

### FLD-CNV-004: The fizz

- **Behavior:** each conversion of FLD-CNV-001 to FLD-CNV-003, and each block that lava
  displaces (FLD-FLW-006), records one level event with the id 1501 and the data 0 at the
  position of the new block. The server delivers it as SIM-EFX-003 states, in the packet
  `minecraft:level_event` (play clientbound id 47). The client plays the extinguish sound of
  lava and shows smoke.

Provenance: the packet id `black_box_confirmed` by `packets.json`. The event id `documentation`
(the public protocol documentation lists 1501 as the lava fizz); section 12 (U-10) names the
capture that confirms it. Kind: wire for the id, semantic for the occasions. Confidence:
medium.

---

## 6. Waterlogged blocks

### FLD-WLG-001: A waterlogged block is a water source

- **Behavior:** the fluid state of a waterlogged block is a water source (FLD-STA-002). For
  each rule of this document, a waterlogged block at Q counts as a water source at Q: it counts
  in `s` of FLD-FLW-002, it makes Q + D "holds a source" for the position above, and water
  cannot flow into Q.
- **Edge cases:** a waterloggable block never holds flowing water and never holds lava.

Provenance: `black_box_confirmed` by `oak_slab.json`. Kind: semantic. Confidence: high.

### FLD-WLG-002: A waterlogged block flows

- **Behavior:** the fluid tick of a waterlogged block is the tick of a water source (FLD-TCK-001
  with S a source): it spreads into the open neighbours of the block (FLD-FLW-005). The spread
  passes only the faces that the block does not cover (FLD-FLW-003 condition 3).
- **Conformance tests:** E-FL6.

Provenance: `source_suggested`; experiment E-FL6 (section 12, U-05). Kind: semantic.
Confidence: medium.

### FLD-WLG-003: When a waterlogged block ticks

- **Behavior:** a waterlogged block schedules the fluid tick (P, water) with the tick period of
  water when it receives a shape update (SIM-SHP-001). A set that puts a waterlogged block
  schedules no fluid tick by itself. As a result, a waterlogged block that a command puts in
  open air does not flow until a neighbour changes.
- **Conformance tests:** E-FL6.

Provenance: `source_suggested`; experiment E-FL6 (section 12, U-05). Kind: semantic.
Confidence: medium.

### FLD-WLG-004: Flowing water and a waterloggable block

- **Behavior:** a write of FLD-FLW-005 into a waterloggable block that is not waterlogged
  TODO-EXP. The result of experiment E-FL6 decides this rule. `block-placing.md`
  (PLC-STA-005) states the placement of a waterloggable block into water: only a source
  waterlogs it.
- **Conformance tests:** E-FL6.

Provenance: `unresolved` (section 12, U-05). Kind: semantic. Confidence: low.

### FLD-WLG-005: Removal of a waterlogged block

- **Behavior:** a destroy of a waterlogged block (SIM-SET-004) leaves a water source at its
  position, because the destroy sets the block form of the fluid at the position. The source
  then flows as each other source does. A set that puts another block at the position, without
  a destroy, leaves what that set puts. `block-breaking.md` states the break by a player, and
  `falling-block.md` (FBK-LND-004) states a falling block that lands in a source.
- **Conformance tests:** E-FL6.

Provenance: `derived` from SIM-SET-004 and FLD-WLG-001; experiment E-FL6 (section 12, U-05).
Kind: semantic. Confidence: high.

---

## 7. Bubble columns

### FLD-BUB-001: State

- **Behavior:** a bubble column is a block state of `minecraft:bubble_column` with the property
  `drag`. Its fluid state is a water source (FLD-STA-002). It is replaceable by a block
  placement, and a fluid cannot flow into it (FLD-FLW-003). For each rule of sections 3 and 4
  it counts as a water source.

Provenance: `black_box_confirmed` by `bubble_column.json`. Kind: semantic. Confidence: high.

### FLD-BUB-002: Formation and growth

- **Behavior:** a water source directly above `minecraft:soul_sand` becomes an upward column
  (`drag` = false); a water source directly above `minecraft:magma_block` becomes a downward
  column (`drag` = true). The column continues upward: each water source directly above a
  bubble column becomes a bubble column with the same `drag`. The first block above the column
  that is not a water source ends it.
- **Edge cases:** flowing water above the base does not become a column. A waterlogged block
  above the base does not become a column and ends it.
- **Conformance tests:** E-FL7.

Provenance: the result `documentation`; the delays `source_suggested`, experiment E-FL7 (section
12, U-07). Kind: semantic. Confidence: medium.

### FLD-BUB-003: Ticks of a column

- **Behavior:** TODO-EXP. Experiment E-FL7 gives the delay between the placement of a base
  under a water source and the first column block, and the delay after which a column with no
  base becomes water again. The reference behaviour that this document expects: the base
  schedules a block tick for itself with the delay 20 when it is placed and when a water source
  appears above it; its tick builds the column (FLD-BUB-002). A bubble column schedules a block
  tick for itself with the delay 5 on each shape update; its tick checks the block below: a
  base or a bubble column keeps it, each other block turns it into a water source. Each bubble
  column also schedules its fluid tick (FLD-TCK-004 event 3).
- **Conformance tests:** E-FL7.

Provenance: `source_suggested`; experiment E-FL7 (section 12, U-07). Kind: semantic.
Confidence: medium.

### FLD-BUB-004: Entities

- **Behavior:** the push of an entity in a bubble column is not in this document.
  `entity-physics.md` states the inside actions of an entity; issue #88 adds the push of a flow
  and of a column.

Provenance: `design`: a boundary of this document. Kind: unspecified. Confidence: high.

---

## 8. World rules and attributes

### FLD-RUL-001: World rules of this domain

- **Behavior:** each rule is a value of the world. A server operator changes one rule without
  the others.

| Rule | Type | Default | Effect |
|------|------|---------|--------|
| `minecraft:water_source_conversion` | boolean | true | FLD-FLW-002 step 2, FLD-SRC-001 for water |
| `minecraft:lava_source_conversion` | boolean | false | FLD-FLW-002 step 2, FLD-SRC-001 for lava |

  The environment attribute `minecraft:gameplay/fast_lava` (`environment-attributes.md`) is
  the switch of FLD-TCK-002 and FLD-FLW-001. It is a value of the environment at the position,
  not a test of the dimension key.
- **Edge cases:** this document adds no setting of the server. Each number of this document is
  the vanilla value, and the budget of fluid ticks is the budget of SIM-SCH-006.

Provenance: `black_box_confirmed` by `game_rule.json` (both rules, their types and defaults)
and by `registries.json` (the registry `minecraft:game_rule`). Kind: semantic. Confidence:
high.

---

## 9. Footprints

### FLD-FTP-001: The footprint of each handler

- **Behavior:** one call of each handler of this document reads and writes only positions in
  the box of Table 1, relative to the position P of the call. The box contains each read of
  FLD-FLW-002, FLD-FLW-003 and FLD-FLW-004 for P and for each position that the call writes,
  and each set of the call. Neighbor updates and shape updates that a set makes are new items
  (SIM-MODEL-005); their targets are at most one position outside the writes.

**Table 1. Footprints.** `d` is the slope distance at P (FLD-FLW-001).

| Handler | `dx` | `dy` | `dz` | Writes | Reads beyond the writes |
|---------|------|------|------|--------|-------------------------|
| fluid tick of water (`d` = 4) | ±5 | −2..+1 | ±5 | P, P + D and the 4 horizontal neighbours | the slope search to the distance `d` + 1 and the blocks below it; the neighbours of each write for its new state; the blocks that cover a face |
| fluid tick of lava without fast lava (`d` = 2) | ±3 | −2..+1 | ±3 | the same | the same, and the soul soil below P + D for FLD-CNV-003 |
| fluid tick of lava with fast lava (`d` = 4) | ±5 | −2..+1 | ±5 | the same | the same |
| placement side effect and neighbor-changed handler of a lava block (FLD-CNV-001, FLD-CNV-003) | ±1 | −1..+1 | ±1 | P | the 5 visited positions and the block below P |
| placement side effect and neighbor-changed handler of a water block | 0 | 0 | 0 | none | none; it schedules the fluid tick of P |
| shape update of a waterlogged block (FLD-WLG-003) | 0 | 0 | 0 | none | none; it schedules the fluid tick of P |
| block tick of a bubble column (FLD-BUB-003) | 0 | 0..+n | 0 | P and the water sources above P | the block below P. `n` is the height of the water above P |
| block tick of soul sand and of a magma block (FLD-BUB-003) | 0 | +1..+n | 0 | the water sources above P | none |
| random tick of lava | ±3 | 0..+3 | ±3 | fire | `block-simulation.md`, Table 1; the fire rules of its section 12 |

- **Edge cases:** the write at P + D can convert (FLD-CNV-001 at P + D reads P + D − D = P − 2
  in y for the soul soil). This is why `dy` reaches −2. The conversion of a neighbour that the
  spread writes is part of the tick (a chain item of its set).

Provenance: `derived` from the rules of this document and from SIM-MODEL-009 (Table 6 gives
±5, −1..+1, ±5 for a fluid tick; this document adds −2 for the soul soil read of a converted
write below P). Kind: semantic. Confidence: medium.

---

## 10. Numeric constants

| Constant | Value | Where | Provenance |
|----------|-------|-------|------------|
| Tick period of water | 5 | FLD-TCK-002 | `source_suggested_black_box_confirmed` (E-FL1) |
| Tick period of lava | 30; 10 with fast lava | FLD-TCK-002 | `source_suggested_black_box_confirmed` (E-FL2) for 30; `source_suggested` for 10 (U-01) |
| Long delay of a lava rise | 4 × the period, with the probability 3/4 | FLD-TCK-003 | `source_suggested` (U-02) |
| Drop of water | 1 | FLD-FLW-001 | `source_suggested_black_box_confirmed` (E-FL1) |
| Drop of lava | 2; 1 with fast lava | FLD-FLW-001 | `source_suggested_black_box_confirmed` (E-FL2) for 2; `source_suggested` for 1 (U-01) |
| Slope distance of water | 4 | FLD-FLW-001 | `source_suggested` (U-03) |
| Slope distance of lava | 2; 4 with fast lava | FLD-FLW-001 | `source_suggested` (U-01, U-03) |
| Sources that form a source | 2 horizontal neighbours | FLD-FLW-002 | `source_suggested_black_box_confirmed` (E-FL2) |
| Sources that keep a sideways spread above a hole | 3 horizontal neighbours | FLD-FLW-005 | `source_suggested` (U-09) |
| Level event of the fizz | 1501 | FLD-CNV-004 | `documentation` (U-10) |
| Block tick delay of a bubble column base | 20 | FLD-BUB-003 | `source_suggested` (U-07) |
| Block tick delay of a bubble column | 5 | FLD-BUB-003 | `source_suggested` (U-07) |
| `water_source_conversion` default | true | FLD-RUL-001 | `black_box_confirmed` (`game_rule.json`) |
| `lava_source_conversion` default | false | FLD-RUL-001 | `black_box_confirmed` (`game_rule.json`) |
| `level_event` packet id | 47 | FLD-CNV-004 | `black_box_confirmed` (`packets.json`) |

---

## 11. Intentional deviations

- **Rule:** FLD-TCK-001, FLD-FLW-005. **Reference:** the fluid ticks of one phase run in the
  collection order of SIM-SCH-006, and the vanilla order across columns depends on the column
  layout. **Reason:** the order of the fluid ticks of different positions inside one phase is
  the ordering policy of the instance (SIM-MODEL-008). Each rule of this document gives the
  result inside one position; the result of a flow after the phase does not depend on the
  order, because each write puts the new state of its target (FLD-FLW-005). **Visible to the
  client:** no: the client receives the block changes of a tick in one `section_blocks_update`
  for each section (SIM-SET-005). **Bug reference:** none. **Test:** a differential test
  compares the states after the tick, not inside it; the tick of each change is compared under
  the exact policy and reported, not failed.
- **Rule:** FLD-TCK-005. **Reference:** the draw of FLD-TCK-003 comes from the dimension
  random, which the clock seeds. **Reason:** the draws of this project come from the keyed
  stream of the work item, so that the same inputs give the same flow for each count of
  threads. **Visible to the client:** no: parity is by distribution. **Bug reference:** none.
  **Test:** a differential test compares the distribution of the delays, not the sequence.

## 12. Unresolved

Each item lists the rules that it affects and the experiment that answers it. The experiments
of this section have not run.

- **U-01: Fast lava** (FLD-TCK-002, FLD-FLW-001). The period 10, the drop 1 and the slope
  distance 4 under `gameplay/fast_lava`. Experiment E-FL3 on a stone platform above the roof of
  the nether: put a lava source, step one tick at a time to tick 80 and read the rings; put a
  second lava source 12 positions away and read how far each flow reaches.
- **U-02: The long delay of a lava rise** (FLD-TCK-003). Experiment E-FL3: in 40 walled 1 × 4
  trenches `[A][Q][P][B]`, put the source A, step 120 ticks, put the source B, step 30 ticks and
  save. Read the pending lava tick at each P: the count with `t` = 120 and the count with
  `t` = 30. The expected result is about 30 and 10.
- **U-03: The slope distance** (FLD-FLW-001, FLD-FLW-004). Experiment E-FL10: on a flat floor,
  put one hole at the horizontal distance 5 from a water source, and in a second scene at the
  distance 6. Step to tick 40. In the first scene, the expected result is water only on the side
  of the hole; in the second, water on each side.
- **U-04: Solid blocks under a new source** (FLD-FLW-002 step 2). Which partial blocks count as
  solid: a slab, a carpet, glass, a waterlogged block. Experiment: a 1 × 3 trench with each
  block under the middle; read whether the middle becomes a source.
- **U-05: Waterlogged blocks** (FLD-TCK-004 event 3, FLD-WLG-002 to FLD-WLG-005). Experiment
  E-FL6: put a waterlogged slab in open air with a command and step 20 ticks (expected: no
  flow); toggle a neighbour and step 5 ticks (expected: the level 7 at the open neighbours);
  destroy the slab with `setblock ... destroy` (expected: a water source); let a flow of the
  level 7 reach a fence (the result decides FLD-WLG-004).
- **U-06: Conversions** (FLD-CNV-001 to FLD-CNV-003, FLD-FLW-003 condition 1). Experiment
  E-FL5: a flow of lava met by water from the side (expected: cobblestone at the lava position
  in the tick of the water set); water put beside, above and below a lava source (expected:
  obsidian, obsidian, nothing); a lava source 3 positions above a water source (expected: stone
  at the water position at tick 90); a lava source put on soul soil beside blue ice (expected:
  basalt at once); a lava source put beside a water source (expected: obsidian at once).
- **U-07: Bubble columns** (FLD-TCK-004 event 3, FLD-BUB-002, FLD-BUB-003). Experiment E-FL7:
  a walled 1 × 1 × 5 column of water sources; put soul sand under it and step one tick at a time
  (expected: the five blocks become upward columns at tick 20); remove the soul sand and step
  (expected: the column becomes water from the bottom up, 5 ticks for each block); the same
  with a magma block and `drag` = true.
- **U-08: Displaced blocks** (FLD-FLW-003 condition 2, FLD-FLW-006). Experiment E-FL9: short
  grass, a torch, a rail and a carpet in the path of water and in the path of lava. Read the
  blocks and the item entities of the column after 40 ticks (water) and 120 ticks (lava).
- **U-09: The fall** (FLD-FLW-005 steps 1 and 2). Experiment E-FL4: a water source 3 positions
  above a floor (expected: falling states below it, the level 7 around the position where it lands
  from tick 20, a disc of the radius 7); a water source 3 positions above a 3 × 3 pool of
  sources in a pit (expected: no water on the floor around the pit); the same for lava above a
  floor.
- **U-10: The id of the fizz** (FLD-CNV-004). Capture the packets of a vanilla client during
  E-FL5 with `tools/mcrs_minecraft_client_recorder` and read the `level_event` packets.
- **U-11: The faces that a flow passes** (FLD-FLW-003 condition 3). Which blocks cover a face
  for a flow: stairs, a slab, a trapdoor, a waterlogged fence. Experiment: a waterlogged
  bottom slab and a waterlogged top slab beside air; read where water appears.

## 13. Experiments

The experiments ran on the vanilla dedicated server of the version of
`assets/minecraft/version.json`, with a flat overworld, no player, `random_tick_speed` 0,
`spawn_mobs` false and the chunks of the scene force-loaded. Each scene starts with
`/tick freeze`. The blocks of the scene come from `/setblock` and `/fill`. After each step of
`/tick step`, `/save-all flush` writes the columns, and the record of the experiment takes the
block changes and the `fluid_ticks` of the columns from the region files. The tick 0 of a record
is the state after the last command of the scene, before any step. The records are the fixtures
of section 0.3.

- **E-FL1** (`water_flat_floor.json`). Put a water source at (16, −60, 16) on the grass floor.
  Step one tick at a time to tick 45, then to tick 100. Result: at tick 0, one pending
  `minecraft:water` tick with `t` = 5 at the source. At tick 5, the 4 horizontal neighbours have
  the block level 1, and each has a pending tick with `t` = 5. At tick `5k`, for `k` = 2 to 7,
  the positions at the Manhattan distance `k` have the block level `k`. No position at the
  distance 8 changes. From tick 40 on, nothing changes. At tick 100, the disc has the radius 7.
- **E-FL8** (`water_source_removed.json`). Continue E-FL1. Set the source to air. Step one tick
  at a time to tick 50. Result: 5 ticks after the removal, the 4 neighbours have the block level
  3 and the removed position has the block level 2. 5 ticks later, the removed position and the
  ring at the distance 2 have the block level 4. Then one ring farther out changes every 5
  ticks, by one more level, until the ring that reaches the level 0 becomes air. 30 ticks after
  the removal, the ring at the distance 6 is air; 35 ticks after it, the ring at the distance 7
  is air; then nothing changes.
- **E-FL2** (`water_trench_source.json`, `water_trench_no_conversion.json`,
  `lava_trench_source.json`). Build four walled trenches on the grass floor: A is 1 × 3; B is
  1 × 4; C is 1 × 3 with a water source under its middle; D is 1 × 3 with air under its middle.
  Put water sources at the two ends of each. Step one tick at a time to tick 30. Result: at tick
  5, the middle of A and the middle of C are sources, the two middles of B have the block level
  1, and the middle of D has the block level 1. At tick 10, the air under the middle of D has the
  block level 8. Nothing else changes. Then set `water_source_conversion` to false and repeat A:
  the middle has the block level 1 at tick 5 and stays so to tick 30. Then build two lava
  trenches 1 × 3. With `lava_source_conversion` false, the middle has the block level 2 at tick
  30 and stays so to tick 90. With the rule true, the middle is a lava source at tick 30.
