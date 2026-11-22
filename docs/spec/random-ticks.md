# Random ticks of the growth blocks: behavioural specification

Version: see `assets/minecraft/version.json`.

This document states what a vanilla block does when the machinery of `block-simulation.md`
delivers a random tick to it. It also states what a block does when a player or a dispenser
uses bone meal on it. It covers the crops, the stems, sugar cane, cactus, bamboo, the saplings, the grass block
and mycelium, the leaves, ice and snow, and farmland.

`block-simulation.md` states which sections receive random ticks, how many picks a section gets
and how a pick selects a position (SIM-RND-001 to SIM-RND-004). This document does not repeat
these rules. It states only the handler of each block. The precipitation picks (ice from water,
snow layers, cauldrons under rain) are in SIM-RND-004 and are not repeated here.

Out of scope:

- The use of the bone meal item by a player: the start, the hold and the release of the use,
  the cooldown, the hand animation and the particles. This document states only the response
  of the block (section 12). `item-stack.md` states the count of a stack.
- The trample of farmland by an entity. The entity rules state it.
- Fire and soul fire (`block-simulation.md`, section 12.1), the fluids, and the blocks that
  the first paragraph does not name: nylium, the mushrooms, the vines, kelp, cocoa, nether
  wart, the chorus plants, turtle eggs, the copper blocks, the dripstone, the amethyst, the
  eyeblossoms, the sweet berry bush and the pitcher crop. Each of them gets its rules in a
  later document. Section 1 lists them, so that the table of randomly ticking states is
  complete.
- The drops of a block that a handler destroys: `block-loot.md` states the tables.

## 0. Conventions

### 0.1 Terms

- **Position**, **block state**, **block type**, **set**, **neighbor update**, **shape update**,
  **scheduled tick**, **placement side effect**, **removal side effect**, **randomly ticking**,
  **section**, **dimension random**: as `block-simulation.md`, section 0.1, defines them.
- **Random tick**: one call of the random tick handler of the block state at a position P. It
  is step 3 of a pick (SIM-RND-002). A fluid random tick is step 4 of a pick.
- **Randomly ticking state**: a block state or a fluid state with a row in Table 1 of
  GRW-TBL-001 whose condition is true for that state.
- **Random of the pick**: the random source that the handler draws from. In vanilla it is the
  dimension random (SIM-RNG-001). `r(n)` is a uniform integer draw in `[0, n)`, and `u` is a
  uniform draw in `[0, 1)`, both from the random of the pick. A draw occurs only where a rule
  names it.
- **Block light at P**, **sky light at P**: the two stored light values at P, each 0 to 15. The
  sky light does not depend on the time of day.
- **Light at P**: the larger of the block light at P and the sky light at P.
- **Sky darken**: an integer 0 to 11 that the dimension computes from the time of day and the
  weather. It is 0 in full daylight and 11 at midnight. Rain and thunder increase it (U2).
- **Ambient light at P**: the larger of the block light at P and the sky light at P minus the
  sky darken. A negative value counts as 0.
- **Open sky at P**: the sky light at P is 15.
- **Age**, **stage**, **moisture**, **distance**, **persistent**, **layers**: the properties of
  the block states of this document. The block definitions under `assets/mcrs/block_definition/`
  give their values.
- **Growth speed** of a crop or stem at P: the value `g` of GRW-CRP-002.
- **Moist farmland**: a farmland state with `moisture` above 0. **Dry farmland**: `moisture` 0.
  Only `moisture` 7 has the wet texture on the client.
- **Fruit**: `minecraft:melon` for a melon stem and `minecraft:pumpkin` for a pumpkin stem.
- **Tree feature**: a configured feature of the data pack under `data/minecraft/worldgen/feature/`
  with the type `minecraft:tree`, for example `minecraft:oak`.
- **Bone meal response**: what a block does when bone meal is used on it. The block **accepts**
  the bone meal when its response changes the world, and **rejects** it otherwise. Section 12
  states the responses.
- **Rain at a position**: the test of `block-simulation.md`, SIM-FIRE-004.
- **Remove P**: the "remove block" of SIM-SET-004 at P, without drops.
- **Destroy P**: the destroy of SIM-SET-004 at P, with the drop flag set.
- **Set P to S**: a set with flags 3 (SIM-SET-002), unless the rule names other flags.
- **Full occluder**: a block state whose block definition sets `minecraft:light_dampening` to 15.

### 0.2 Provenance

- `black_box_confirmed`: shown by a game-produced artefact that is cited by path, or by an
  experiment of section 16 that ran.
- `source_suggested_black_box_confirmed`: taken from the reference behaviour, and consistent
  with a cited artefact or with an experiment that ran.
- `source_suggested`: taken from the reference behaviour, with no artefact yet. Confidence is
  at most medium, and an experiment of section 16 is attached.
- `documentation`: taken from public documentation of player-facing behaviour.
- `derived`: it comes from other rules of this document or of `block-simulation.md`.
- `design`: a choice of this project. The rule states the reason.
- `unresolved`: not decided. Section 15 collects these items.

Each rule states its kind, as `docs/spec/README.md` defines it. Each number of this document has
a provenance, or the rule names the item of section 15 that holds the open question.

### 0.3 Artefacts cited

- The block definitions under `assets/mcrs/block_definition/`: the properties `age`, `stage`,
  `moisture`, `distance`, `persistent`, `layers`, `leaves`, `snowy` and `facing`, the
  component `minecraft:light_dampening`, and the component `minecraft:placement_filter` of the
  stems, the saplings, the bamboo sapling and the snow layer.
- `assets/mcrs/registry_definition/game_rule.json`: `minecraft:random_tick_speed` (integer,
  default 3, minimum 0).
- The data pack of the game, under `data/minecraft/tags/block/`: `supports_crops`,
  `supports_stem_crops`, `supports_melon_stem`, `supports_pumpkin_stem`,
  `supports_melon_stem_fruit`, `supports_pumpkin_stem_fruit`, `supports_stem_fruit`,
  `supports_vegetation`, `substrate_overworld`, `supports_sugar_cane`,
  `supports_sugar_cane_adjacently`, `supports_cactus`, `supports_bamboo`, `maintains_farmland`,
  `prevents_nearby_leaf_decay`, `logs`, `snow`, `saplings`, `crops`, `leaves`, `dirt`,
  `grass_blocks`, `flowers`.
- The data pack of the game, under `data/minecraft/worldgen/feature/`: the tree features of
  Table 7 (GRW-SAP-003) and the features `minecraft:grass` and `minecraft:flower_default`; under
  `data/minecraft/worldgen/placed_feature/`: `minecraft:grass_bonemeal`.
- The public documentation of the game: the pages of the Minecraft Wiki for each block of this
  document and for bone meal.

### 0.4 ID prefixes

| Prefix | Subject |
|--------|---------|
| `GRW-TBL` | which block states and fluid states tick randomly |
| `GRW-CRP` | the crops: wheat, carrots, potatoes, beetroots, torchflower crop |
| `GRW-STM` | the melon stem and the pumpkin stem |
| `GRW-CAN` | sugar cane |
| `GRW-CAC` | cactus |
| `GRW-BAM` | the bamboo sapling and bamboo |
| `GRW-SAP` | the saplings and their trees |
| `GRW-SPR` | the grass block and mycelium |
| `GRW-LEF` | the leaves |
| `GRW-ICE` | ice, frosted ice and the snow layer |
| `GRW-FRM` | farmland |
| `GRW-BON` | the bone meal responses |
| `GRW-FOOT` | the footprint of each handler |
| `GRW-DEV` | intentional deviations |

### 0.5 Directions and offsets

The directions are `D`, `U`, `N`, `S`, `W` and `E`, as `block-simulation.md`, section 0.3,
defines them. P + d is the neighbor of P in the direction d. An offset `(dx, dy, dz)` is added
to a position. The **horizontal neighbors** of P are P + N, P + E, P + S and P + W. The **eight
positions around** P are the positions P + (dx, 0, dz) with dx and dz in −1..1, P itself
excluded.

---

## 1. Randomly ticking states

### GRW-TBL-001: The table of randomly ticking states

- **Behavior:** a block state ticks randomly when its block type has a row in Table 1 and the
  condition of the row is true for the state. A fluid state ticks randomly when it has a row in
  Table 2. A state that has no row does not tick randomly. SIM-RND-002 reads this property for
  each pick.

**Table 1. Randomly ticking block states.** "all" means each state of the block type. The column
"Rules" names the section that states the handler, or "later" for a block type whose handler is
not in this document.

| Block types | Condition | Rules |
|-------------|-----------|-------|
| `wheat`, `carrots`, `potatoes` | `age` < 7 | section 2 |
| `beetroots` | `age` < 3 | section 2 |
| `torchflower_crop` | all (`age` 0 and 1) | section 2 |
| `melon_stem`, `pumpkin_stem` | all | section 3 |
| `attached_melon_stem`, `attached_pumpkin_stem` | none | — |
| `sugar_cane` | all | section 4 |
| `cactus` | all | section 5 |
| `bamboo_sapling` | all | section 6 |
| `bamboo` | `stage` = 0 | section 6 |
| each block of the tag `#minecraft:saplings`, except `azalea`, `flowering_azalea` and `mangrove_propagule` | all | section 7 |
| `azalea`, `flowering_azalea` | all | section 7 (U8) |
| `mangrove_propagule` | `hanging` = false, or `hanging` = true with `age` < 4 | section 7 for the planted propagule (U8); later for the hanging one |
| `grass_block`, `mycelium` | all | section 8 |
| each block of the tag `#minecraft:leaves` | `distance` = 7 and `persistent` = false | section 9 |
| `ice` | all | section 10 |
| `frosted_ice` | none: its melt is a scheduled tick | section 10 |
| `snow` | all | section 10 |
| `farmland` | all | section 11 |
| `crimson_nylium`, `warped_nylium`, `pitcher_crop`, `cocoa`, `nether_wart`, `sweet_berry_bush`, `brown_mushroom`, `red_mushroom`, `vine`, `kelp`, `twisting_vines`, `weeping_vines`, `cave_vines`, `chorus_flower`, `turtle_egg`, each copper block that ages, `pointed_dripstone`, `icicle`, `sulfur_spike`, `budding_amethyst`, `redstone_ore` and `deepslate_redstone_ore` (`lit` = true), `nether_portal`, `open_eyeblossom`, `closed_eyeblossom`, `potted_open_eyeblossom`, `potted_closed_eyeblossom`, `dried_ghast`, `firefly_bush` | as `block-simulation.md`, SIM-MODEL-009, Table 1, lists them; the condition of each is U1 | later |

**Table 2. Randomly ticking fluid states.**

| Fluid type | Condition | Rules |
|------------|-----------|-------|
| `minecraft:lava` | all (source and flowing) | later (`block-simulation.md`, SIM-FIRE-002, names its spread gate) |
| `minecraft:water`, `minecraft:empty` | none | — |

- **Edge cases:**
  - A crop at its last age does not tick randomly. As a result, a section whose only crops are
    fully grown gets no picks for them (SIM-RND-002, Behavior).
  - Leaves with `distance` 7 and `persistent` false tick randomly also when a log is next to
    them, until their scheduled tick corrects the distance (GRW-LEF-001).
  - A lava block is a randomly ticking block whose handler runs the fluid handler. The pick then
    runs the fluid handler again (SIM-RND-002, Edge cases).
- **Conformance tests:** make a section with one `wheat[age=7]` and nothing else that ticks.
  Use `random_tick_speed` 4096 for 10 000 ticks. Result: the position generator of the section
  does not advance (SIM-RND-002, Invariants). Failing twin: the generator advances, because the
  implementation treats each crop state as randomly ticking.

Provenance: the block types and the fluid type are `source_suggested_black_box_confirmed`:
`block-simulation.md`, SIM-MODEL-009, Table 1, lists the same block types, and the data pack
gives the tags. The conditions of the rows of this document are `source_suggested`. The
conditions of the "later" row are U1. The existence of the properties is
`black_box_confirmed` by the block definitions. Kind: semantic. Confidence: medium.

### GRW-TBL-002: The source of the table

- **Behavior:** the block definitions do not carry the property "randomly ticking" at this
  version. A data pack that adds a block cannot declare it. The table of GRW-TBL-001 is the
  only statement of this property.
- **Edge cases:** a block state with a row in Table 1 and no handler in any specification
  receives picks and does nothing.

Provenance: `black_box_confirmed` by the block definitions under `assets/mcrs/block_definition/`:
no component names random ticks. The form of the data is U1. Kind: unspecified. Confidence: high.

---

## 2. Crops

The crops are `wheat`, `carrots`, `potatoes`, `beetroots` and `torchflower_crop`. Each has one
property `age`. The last age is 7 for wheat, carrots and potatoes, 3 for beetroots, and 1 for the
torchflower crop. The crop that grows past `age` 1 of `torchflower_crop` is the block
`minecraft:torchflower` (GRW-CRP-004). The base of a crop is the block at P + D.

### GRW-CRP-001: Base and survival

- **Inputs:** a shape update at a crop at P, or a placement of a crop at P.
- **Behavior:** a crop can stay at P when both conditions are true:
  - The block at P + D is in the tag `#minecraft:supports_crops`.
  - The light at P is 8 or more, or P has open sky.

  When a crop cannot stay, the server destroys P (`block-support.md`, SUP-RUN-001). The test
  runs on a shape update from each direction and on a placement. A random tick does not run the
  test.
- **Edge cases:**
  - A crop in darkness stays until a shape update reaches it. A change of the light alone does
    not remove it.
  - At night, the sky light at P is still 15 under the open sky. A crop under the open sky
    never pops at night.
  - A crop under glass has open sky, because glass does not reduce the sky light. A crop under
    a leaf block does not have open sky (the leaf block has `minecraft:light_dampening` 1).
- **Conformance tests:** experiment E-G2, steps 5 to 8.

Provenance: the base tag is `black_box_confirmed` by `data/minecraft/tags/block/supports_crops.json`
(its only member is `minecraft:farmland`). The light rule is `documentation` (the wiki states
"a light level of 8 or greater to plant"), and `source_suggested` for the open sky and for the
moment of the test. Kind: semantic. Confidence: medium.

### GRW-CRP-002: Growth speed

- **Inputs:** a crop or a stem of block type B at P. F is the position P + D.
- **Behavior:** the growth speed `g` is the sum of these terms:
  - 1.
  - The block at F: 3 when it is moist farmland, 1 when it is dry farmland, 0 otherwise.
  - Each of the eight positions around F: 0.75 when it holds moist farmland, 0.25 when it holds
    dry farmland, 0 otherwise.

  Then `g` is halved when at least one of these conditions is true:
  - P + N and P + S both hold a block of type B.
  - P + E and P + W both hold a block of type B.
  - At least one of the four diagonal positions P + (±1, 0, ±1) holds a block of type B.
- **Numeric semantics:** `g` is a 32-bit float. Its value before the half rule is 2 to 10 in
  steps of 0.25 for a crop on farmland. A crop on another base has `g` from 1 to 7 (U3).
- **Edge cases:**
  - A different crop next to the crop does not halve `g`. Carrots next to wheat count as
    nothing.
  - The farmland with `moisture` 1 to 6 counts as moist. The client shows the wet texture only
    at `moisture` 7.
- **Conformance tests:** experiment E-G1.

Provenance: `documentation` for the terms 2 and 4, 0.25 and 0.75 and the half rule (the wiki page
"Tutorial:Crop farming" states "a base speed level of 2 if dry or 4 if hydrated", "each
surrounding dry farmland adds 0.25", "each surrounding hydrated farmland adds 0.75", and the
half rule for the diagonal or both axes). The moisture threshold 0 is `source_suggested`.
Kind: semantic. Confidence: medium.

### GRW-CRP-003: The random tick of a crop

- **Inputs:** a random tick at a crop at P with age `a`.
- **Behavior:** the tick does these steps in this order:
  1. If the light at P is less than 9, the tick ends.
  2. If the crop is `beetroots`, draw `r(3)`. If the result is not 0, the tick ends.
  3. Let `g` be the growth speed (GRW-CRP-002). Draw `r(floor(25 / g) + 1)`. If the result is
     not 0, the tick ends.
  4. Set P to the crop with age `a + 1`, with flags 2.
- **Numeric semantics:** the chance of step 3 is `1 / (floor(25 / g) + 1)`. For a lone crop on
  moist farmland with dry surroundings, `g` = 4 and the chance is 1/7. For a crop with moist
  farmland below and around, `g` = 10 and the chance is 1/3. For a lone crop on dry farmland,
  `g` = 2 and the chance is 1/13. Two crops of one type diagonal to each other on dry
  farmland have `g` = 1.125 and the chance 1/23.
- **Edge cases:**
  - The light test reads the light at P, not at P + U. The sky light counts in full at night.
    As a result, a crop under the open sky grows at night.
  - The tick does not test the base and does not remove the crop (GRW-CRP-001).
  - A set with flags 2 makes no neighbor updates. The clients get the change. The shape updates
    of the set run (SIM-SET-002), so an observer that faces the crop pulses.
  - The torchflower crop has `age` 0 and 1 only. Its step 4 at `age` 1 is GRW-CRP-004. Whether
    the torchflower crop also skips ticks as step 2 does is U4.
- **Conformance tests:** experiments E-G1 and E-G2.

Provenance: `documentation` for the light threshold 9, the chance formula and the beetroot skip
(the wiki states "9 or greater to grow", "1/(floor(25 / speedLevel) + 1)" and for beetroots
"each growth tick has a 1/3 chance of not advancing the growth stage"). The flags and the order
of the steps are `source_suggested`. Kind: semantic. Confidence: medium.

### GRW-CRP-004: The last age of the torchflower crop

- **Inputs:** step 4 of GRW-CRP-003 or a bone meal response (GRW-BON-001) at a
  `torchflower_crop[age=1]` at P.
- **Behavior:** the server sets P to `minecraft:torchflower` with flags 2 instead of a crop with
  `age` 2. The torchflower is a plant of `block-support.md`, SUP-VEG-001, and it does not tick
  randomly.
- **Conformance tests:** use bone meal two times on a `torchflower_crop[age=0]`. Result: P holds
  `minecraft:torchflower`.

Provenance: `documentation` (the wiki states that the torchflower crop grows "through three
stages" and that "the final growth stage of the torchflower crop uses the same ID as the actual
torchflower"); the block definition of `torchflower_crop` has `age` 0 and 1 only
(`black_box_confirmed`). Kind: semantic. Confidence: high.

---

## 3. Stems

A stem is `melon_stem` or `pumpkin_stem`, with the property `age` 0 to 7. An attached stem is
`attached_melon_stem` or `attached_pumpkin_stem`, with the property `facing`. The block definition
of a stem has a `minecraft:placement_filter` on the tag `#minecraft:supports_melon_stem` or
`#minecraft:supports_pumpkin_stem`; both resolve to `minecraft:farmland` in the vanilla data pack.

### GRW-STM-001: The random tick of a stem

- **Inputs:** a random tick at a stem at P with age `a`.
- **Behavior:** the tick does these steps in this order:
  1. If the light at P is less than 9, the tick ends.
  2. Let `g` be the growth speed (GRW-CRP-002) with B the block type of the stem. Draw
     `r(floor(25 / g) + 1)`. If the result is not 0, the tick ends.
  3. If `a` < 7, set P to the stem with age `a + 1`, with flags 2. The tick ends.
  4. Draw a horizontal direction d, uniform over `N, E, S, W`. Let Q = P + d.
  5. If the block at Q is not air, the tick ends.
  6. If the block at Q + D is not in the tag `#minecraft:supports_melon_stem_fruit` (melon) or
     `#minecraft:supports_pumpkin_stem_fruit` (pumpkin), the tick ends.
  7. Set Q to the fruit. Then set P to the attached stem with `facing` d.
- **Numeric semantics:** at `age` 7 with one free side, the chance of a fruit in one tick is
  `1 / (4 × (floor(25 / g) + 1))`.
- **Edge cases:**
  - A stem keeps `age` 7 for all time when no side is free. It does not tick less often.
  - The half rule of `g` counts other stems of the same block type, not fruits.
  - The attached stem does not tick randomly. When its fruit is removed, its shape function
    sets the stem with `age` 7 again. That shape function is not in this document (U5).
  - Both tags resolve to `#minecraft:supports_vegetation`: `#minecraft:substrate_overworld`
    (dirt, mud, moss blocks and grass blocks) and `minecraft:farmland`.
- **Conformance tests:** experiment E-G3.

Provenance: `documentation` for the light threshold, the growth of the stem, the fruit on a
free side and the list of bases (the wiki lists dirt, coarse dirt, rooted dirt, grass block,
farmland, podzol, mycelium, moss block, pale moss block, mud and muddy mangrove roots, and this
list is the resolved tag). The tags are `black_box_confirmed` by
`data/minecraft/tags/block/supports_stem_fruit.json` and `supports_vegetation.json`. The uniform
direction, the air test and the order of the two sets are `source_suggested`. Kind: semantic.
Confidence: medium.

---

## 4. Sugar cane

Sugar cane is `sugar_cane`, with the property `age` 0 to 15.

### GRW-CAN-001: The random tick of sugar cane

- **Inputs:** a random tick at sugar cane at P with age `a`.
- **Behavior:** the tick does these steps in this order:
  1. If the block at P + U is not air, the tick ends.
  2. Let `n` be the number of sugar cane blocks in the vertical line from P downward, P
     included. The count stops at the first block that is not sugar cane. If `n` is 3 or more,
     the tick ends.
  3. If `a` is 15: set P + U to `sugar_cane[age=0]`, then set P to `sugar_cane[age=0]` with
     flags 4. The tick ends.
  4. Set P to `sugar_cane` with age `a + 1`, with flags 4.
- **Numeric semantics:** the top block of a column grows one block after 16 of its random ticks.
  A column grows from 1 to 3 blocks after 32 random ticks of its top block.
- **Edge cases:**
  - The light does not matter. Sugar cane grows in darkness.
  - A set with flags 4 does not reach the clients and makes no neighbor updates. The client
    does not see the age. It sees the new block of step 3, which is a set with flags 3.
  - The tick does not test the water next to the base. The survival of sugar cane runs on a
    shape update (U5).
  - When a player breaks the second block of a column, the base keeps its age. The new top is
    the base, and its count restarts from its own age.
- **Conformance tests:** experiment E-G4.

Provenance: `documentation` for the height 3, the 16 random ticks and the growth without light
(the wiki states "a height of three blocks, adding a block of height when the top sugar cane
block has received 16 random ticks" and "grows regardless of light level"). The flags and the
count from P are `source_suggested`. Kind: semantic. Confidence: medium.

---

## 5. Cactus

Cactus is `cactus`, with the property `age` 0 to 15. A cactus flower is `cactus_flower`.

### GRW-CAC-001: The random tick of a cactus

- **Inputs:** a random tick at a cactus at P with age `a`.
- **Behavior:** the tick does these steps in this order:
  1. If the block at P + U is not air, the tick ends.
  2. Let `n` be the number of cactus blocks in the vertical line from P downward, P included,
     as in GRW-CAN-001 step 2.
  3. If `a` is 8 (U6): if the four horizontal neighbors of P + U are air, draw `u`. If `u` is
     less than 0.1 when `n` is 1 or 2, or less than 0.25 when `n` is 3 or more, set P + U to
     `cactus_flower`, and the tick ends.
  4. If `n` is 3 or more, the tick ends.
  5. If `a` is 15: set P + U to `cactus[age=0]`, set P to `cactus[age=0]` with flags 4, and
     make a neighbor update at P + U for the new cactus. The tick ends.
  6. Set P to `cactus` with age `a + 1`, with flags 4.
- **Edge cases:**
  - The light does not matter.
  - The new cactus of step 5 runs its survival test at once through the neighbor update. When a
    solid block is next to it, it breaks at once and drops its item. The growth still occurs.
  - A cactus with a flower on top never grows past the flower, because P + U is not air.
  - The break of a cactus next to a solid block is its shape function. It is not in this
    document (U5).
- **Conformance tests:** experiment E-G4.

Provenance: `documentation` for the height 3, the 16 random ticks, the growth without light,
the growth under a block that breaks at once, and the flower (the wiki states "exactly 9 random
ticks", "10%" for 1 or 2 blocks and "25%" at 3 or more, and "no blocks in any cardinal direction
adjacent to it"). The flags, the neighbor update and the exact age of the flower draw are
`source_suggested` (U6). Kind: semantic. Confidence: medium.

---

## 6. Bamboo

The bamboo sapling is `bamboo_sapling`, with one state. Bamboo is `bamboo`, with the properties
`age` (0 or 1: the thick form), `leaves` (`none`, `small`, `large`) and `stage` (0: grows, 1:
stopped). The block definition of the bamboo sapling has a `minecraft:placement_filter` on the
tag `#minecraft:supports_bamboo`.

### GRW-BAM-001: The random tick of the bamboo sapling

- **Inputs:** a random tick at a bamboo sapling at P.
- **Behavior:** the tick does these steps in this order:
  1. Draw `r(3)`. If the result is not 0, the tick ends.
  2. If the block at P + U is not air, the tick ends.
  3. If the light at P + U is less than 9, the tick ends.
  4. Set P + U to `bamboo[age=0,leaves=small,stage=0]`. Set P to
     `bamboo[age=0,leaves=none,stage=0]` (U7).
- **Edge cases:** the light test reads P + U, the position of the new block.
- **Conformance tests:** experiment E-G4, steps for bamboo.

Provenance: `documentation` for the chance 1/3 and the light threshold 9 at the top (the wiki
states "a 1/3 chance of growing" and "the top of a bamboo plant requires a ... light level of 9
or above to grow"). The states of step 4 are `source_suggested` (U7). Kind: semantic.
Confidence: medium.

### GRW-BAM-002: The random tick of bamboo

- **Inputs:** a random tick at bamboo at P with `stage` 0.
- **Behavior:** the tick does these steps in this order:
  1. Draw `r(3)`. If the result is not 0, the tick ends.
  2. If the block at P + U is not air, the tick ends.
  3. If the light at P + U is less than 9, the tick ends.
  4. Let `h` be the number of bamboo blocks in the vertical line from P downward, P included,
     with a limit of 16. If `h` is 16, the tick ends.
  5. Grow one block (GRW-BAM-003).
- **Edge cases:** a bamboo with `stage` 1 does not tick randomly. It never grows again, also
  when the blocks above it are removed. Bone meal on it is rejected (GRW-BON-005).
- **Conformance tests:** experiment E-G4.

Provenance: `documentation` for the chance, the light and the height 12 to 16 (the wiki states
"Bamboo can grow up to 12–16 blocks tall"). The limit 16 of the count is `source_suggested`.
Kind: semantic. Confidence: medium.

### GRW-BAM-003: One block of bamboo growth

- **Inputs:** a bamboo at P that grows, with `h` from GRW-BAM-002 step 4 or from a bone meal
  response. B1 is the block at P + D, B2 is the block at P + 2D.
- **Behavior:** these steps occur in this order:
  1. Let `L` be `none`. If B1 is bamboo with `leaves` not `none`: let `L` be `large`, and if B2
     is also bamboo with `leaves` not `none`, set P + D to B1 with `leaves` `small` and set
     P + 2D to B2 with `leaves` `none`. If B1 is not bamboo with leaves, let `L` be `small`.
  2. Let `A` be 1 when the state at P has `age` 1 or B2 is bamboo, and 0 otherwise.
  3. Let `S` be 1 when `h` is 15, or when `h` is 11 or more and `u` < 0.25, and 0 otherwise.
     The draw occurs only when `h` is 11 or more and is not 15.
  4. Set P + U to `bamboo` with `age` A, `leaves` L and `stage` S.
- **Numeric semantics:** a column stops at a height of 12 to 16 blocks. The stop at height 16 is
  certain. The heights 12 to 15 each stop with probability 0.25 of the ticks that grow.
- **Edge cases:**
  - The three top blocks carry the leaves: `large` at the top, `small` below it, `none` lower.
  - The thick form (`age` 1) starts at the third block of a column and stays.
- **Conformance tests:** experiment E-G4.

Provenance: `documentation` for the leaves on the top blocks, the thick form and the height
range (the wiki states that at 4 blocks the thickness grows and that at 5 blocks "the top
three have leaves"). The exact steps are `source_suggested`. Kind: semantic. Confidence: low.

---

## 7. Saplings

A sapling is a block of the tag `#minecraft:saplings` with the property `stage` (0 or 1). The
block definition of each sapling has a `minecraft:placement_filter` on the tag
`#minecraft:supports_vegetation` (`block-support.md`, SUP-VEG-001). `azalea`, `flowering_azalea`
and `mangrove_propagule` are also in the tag; their rows of Table 7 are U8.

### GRW-SAP-001: The random tick of a sapling

- **Inputs:** a random tick at a sapling at P with stage `s`.
- **Behavior:** the tick does these steps in this order:
  1. If the ambient light at P + U is less than 9, the tick ends.
  2. Draw `r(7)`. If the result is not 0, the tick ends.
  3. If `s` is 0, set P to the sapling with `stage` 1, with flags 4. The tick ends.
  4. Grow the tree (GRW-SAP-002).
- **Numeric semantics:** with `random_tick_speed` 3, a sapling in light gets a pick about one
  time in 1365 ticks, and it passes step 2 one time in 7 picks. A sapling needs two passes.
- **Edge cases:**
  - The test reads the ambient light, which the sky darken reduces at night. A sapling under the
    open sky and without a light source does not grow around midnight. Crops grow then
    (GRW-CRP-003).
  - The client does not see the change of `stage`. It sees the tree.
- **Conformance tests:** experiment E-G5.

Provenance: `documentation` for the light 9 at P + U (the wiki states "the block above the
sapling requires a light level of at least 9"). The chance 1/7, the ambient light and the two
stages with flags 4 are `source_suggested`. Kind: semantic. Confidence: medium.

### GRW-SAP-002: The growth of a tree

- **Inputs:** a sapling of block type B at P that grows, from GRW-SAP-001 step 4 or from a bone
  meal response. R is the random of the pick.
- **Behavior:** these steps occur in this order:
  1. Search a 2 × 2 square of blocks of type B that contains P. Test the squares with these
     north-west corners in this order: P, P + N, P + W, P + N + W. The first square whose four
     positions all hold B is the square. When B has no row "2 × 2" in Table 7, skip this step.
  2. Select the tree feature from Table 7 (GRW-SAP-003) with R. When the table gives no feature
     (for example a dark oak sapling that is not in a square), nothing occurs.
  3. Set each sapling of the square, or P alone, to air with flags 4.
  4. Run the tree feature with R at the north-west corner of the square, or at P. The feature
     makes its sets as the generation does. Each block of the tree is a set (SIM-SET-002).
  5. If the feature placed nothing, set each position of step 3 to its sapling again, with
     flags 4.
- **Edge cases:**
  - A feature that finds no room places nothing. The sapling stays with `stage` 1. The next
    pass of GRW-SAP-001 step 2 tries again.
  - The leaves that a tree feature places have `persistent` false and `distance` 7. The leaves
    then take their distance through GRW-LEF-001.
  - Steps 3 and 5 do not reach the clients. A client sees the sapling vanish only when the
    tree is placed.
  - The room that a feature needs, and which blocks it can replace, are the rules of the
    feature. They are not in this document.
- **Conformance tests:** experiment E-G5.

Provenance: `documentation` for the 2 × 2 order (the wiki states "southeast, northeast, southwest
and northwest, only the first found 2 × 2 sapling square tries to grow") and for the failure
without room. The leaves are `black_box_confirmed` by each tree feature file, whose
`foliage_provider` has `"persistent": "false"` and `"distance": "7"`. Steps 3 to 5 are
`source_suggested`. Kind: semantic. Confidence: medium.

### GRW-SAP-003: Which tree feature a sapling grows

- **Behavior:** Table 7 gives the tree feature for each sapling. "Flowers near" is true when at
  least one block of the tag `#minecraft:flowers` is in the box dx ±2, dy −1..+1, dz ±2 around
  P. A row with two features and a draw selects with R.

**Table 7. Tree features of the saplings.** A feature name is a key under
`data/minecraft/worldgen/feature/`.

| Sapling | Square | Feature |
|---------|--------|---------|
| `oak_sapling` | no | `fancy_oak` when `r(10)` = 0, else `oak`. With flowers near: `fancy_oak_bees_005` or `oak_bees_005` |
| `birch_sapling` | no | `birch`; with flowers near: `birch_bees_005` |
| `spruce_sapling` | no | `spruce` |
| `spruce_sapling` | 2 × 2 | `mega_spruce` when `r(2)` = 0, else `mega_pine` |
| `jungle_sapling` | no | `jungle_tree_no_vine` |
| `jungle_sapling` | 2 × 2 | `mega_jungle_tree` |
| `acacia_sapling` | no | `acacia` |
| `dark_oak_sapling` | no | none |
| `dark_oak_sapling` | 2 × 2 | `dark_oak` |
| `pale_oak_sapling` | no | none |
| `pale_oak_sapling` | 2 × 2 | `pale_oak_bonemeal` |
| `cherry_sapling` | no | `cherry`; with flowers near: `cherry_bees_005` |
| `poplar_sapling` | no | one of `red_poplar`, `orange_poplar`, `yellow_poplar` (U8) |
| `azalea`, `flowering_azalea` | no | `azalea_tree` (U8) |
| `mangrove_propagule` (planted) | no | `mangrove` or `tall_mangrove` (U8) |

- **Edge cases:**
  - A spruce or jungle sapling that is in a square never grows its small tree. When the large
    feature finds no room, nothing grows.
  - A birch sapling never grows a tall birch. `super_birch_bees` is a feature of the generation
    only.
- **Conformance tests:** experiment E-G5.

Provenance: `documentation` for the tree of each sapling and for "Oak or Fancy Oak" (the
wiki table of the sapling page), and for the bee nest near flowers ("5% chance", which the
`_bees_005` features carry). The features exist in the data pack (`black_box_confirmed` by
their files). The draw `r(10)` for the fancy oak, the draw `r(2)` for the mega spruce, the
`_no_vine` and `_bonemeal` variants, and the flower box are `source_suggested`. Kind: semantic.
Confidence: medium.

---

## 8. Grass block and mycelium

The grass block is `grass_block` and mycelium is `mycelium`. Both have the property `snowy`. In
this section, "the spread block" is the block type of the state at P.

### GRW-SPR-001: Can the spread block stay

- **Inputs:** a position Q and the block A at Q + U.
- **Behavior:** the spread block *can stay* at Q when one of these conditions is true, tested
  in this order:
  1. A is `snow` with `layers` 1: can stay.
  2. A holds a fluid with `level` 8 (a source or a falling fluid): cannot stay.
  3. A is a full occluder: cannot stay.
  4. Otherwise: can stay.
- **Edge cases:**
  - Snow with 2 or more layers is a full occluder. The grass block below it changes to dirt.
  - Glass, leaves, slabs, stairs and fences are not full occluders. The grass block below them
    stays.
  - The light at Q + U does not matter here. A grass block under a torch in a closed room of
    glass stays.
  - A waterlogged block at Q + U holds a fluid with `level` 8. The grass block below it changes
    to dirt. Flowing water with `level` below 8 does not kill it.

Provenance: `documentation` for the snow layer and the water (the wiki states "a single snow
layer makes grass snowy; thicker layers cause the grass block to revert to plain dirt", and that
water kills grass). The occluder test, in place of the "light level four or below" that the wiki
states, is `source_suggested`. Experiment E-G6 decides between them (U9). Kind: semantic.
Confidence: medium.

### GRW-SPR-002: The random tick of the spread block

- **Inputs:** a random tick at a grass block or at mycelium at P.
- **Behavior:** the tick does these steps in this order:
  1. If the spread block cannot stay at P (GRW-SPR-001), set P to `dirt`. The tick ends.
  2. If the ambient light at P + U is less than 9, the tick ends.
  3. Do these steps four times:
     1. Draw the offset `(r(3) − 1, r(5) − 3, r(3) − 1)`. Let Q = P plus the offset.
     2. If the block at Q is not `dirt`, continue with the next repetition.
     3. If the spread block cannot stay at Q, or Q + U holds water of any level, continue with
        the next repetition.
     4. Set Q to the spread block with `snowy` true when the block at Q + U is in the tag
        `#minecraft:snow`, and `snowy` false otherwise.
- **Numeric semantics:** the offset is uniform over the 3 × 5 × 3 box dx ±1, dy −3..+1, dz ±1.
  P itself can be drawn; it is not dirt, so nothing occurs. Each repetition draws three values
  also when an earlier repetition changed a block.
- **Edge cases:**
  - Only `dirt` is a target. Coarse dirt, rooted dirt, podzol and a dirt path are not.
  - Mycelium spreads mycelium, and a grass block spreads grass. A grass block never changes
    mycelium, and mycelium never changes a grass block.
  - The spread block reads the ambient light at P + U, as a sapling does. At midnight under
    the open sky, with no light source, it does not spread.
  - The death of step 1 does not read the light. A grass block in a dark room with a glass
    ceiling stays (GRW-SPR-001).
- **Conformance tests:** experiment E-G6.

Provenance: `documentation` for the range, the four draws, the light 9 above the source and
the targets (the wiki states "within a 3×5×3 range", "checking 4 blocks within range", "a light
level of 9 or brighter directly above it", and "but not coarse dirt or rooted dirt"). The
ambient light, the water test at Q + U and the `snowy` tag are `source_suggested`. Kind:
semantic. Confidence: medium.

---

## 9. Leaves

Leaves are the blocks of the tag `#minecraft:leaves`, with the properties `distance` (1 to 7),
`persistent` and `waterlogged`. A log is a block of the tag `#minecraft:prevents_nearby_leaf_decay`,
which resolves to `#minecraft:logs` in the vanilla data pack.

### GRW-LEF-001: The distance of a leaf block

- **Inputs:** a shape update at leaves at P from the direction d, or the scheduled tick of the
  leaves at P.
- **Behavior:**
  - The *distance from* a neighbor state N is 0 when N is a log, the `distance` of N when N is
    leaves, and 7 otherwise.
  - On a shape update from d: let `k` be the distance from the state at P + d, plus 1, with a
    limit of 7. If `k` is less than the `distance` at P, or if the state at P + d is leaves or a
    log, schedule a block tick at P with the delay 1 (SIM-SCH-003). The shape function does not
    change the state.
  - On the scheduled tick: let `m` be 1 plus the smallest distance from the six neighbors, with
    a limit of 7. If `m` is not the `distance` at P, set P to the leaves with `distance` `m`,
    with flags 3.
- **Edge cases:**
  - Persistent leaves update their distance as other leaves do. They never decay.
  - The change reaches the clients one tick after the change of the neighbor. An observer that
    faces the leaves pulses on the tick.
  - A chain of leaves updates one block for each tick, up to 6 blocks from the log.
- **Conformance tests:** experiment E-G7.

Provenance: `documentation` for the taxicab distance up to 6, the one-tick delay and the
persistent leaves (the wiki states "a maximum distance of 6 blocks", "changes after 1 game tick",
and "Leaves placed by players never decay"). The tag is `black_box_confirmed` by
`data/minecraft/tags/block/prevents_nearby_leaf_decay.json`. The exact condition that schedules
the tick is `source_suggested` (U10). Kind: semantic. Confidence: medium.

### GRW-LEF-002: The random tick of leaves

- **Inputs:** a random tick at leaves at P with `distance` 7 and `persistent` false.
- **Behavior:** the server spawns the drops of the state at P (`block-loot.md`), then removes P.
  Waterlogged leaves leave a water source (SIM-SET-004).
- **Edge cases:**
  - Leaves with `distance` below 7 do not tick randomly (GRW-TBL-001). A leaf block whose log
    was removed decays only after GRW-LEF-001 set its distance to 7.
  - Whether the clients receive the level event 2001 (the break particles and sound) for a
    decay is U10.
- **Conformance tests:** experiment E-G7.

Provenance: `documentation` for the decay and the drops (the wiki states that leaves "decay
(disappear) when they receive a block tick if they are not connected to any log", and gives the
drop chances of the tables). The remove without the level event is `source_suggested` (U10).
Kind: semantic. Confidence: medium.

---

## 10. Ice and snow

Ice is `ice`, with one state and `minecraft:light_dampening` 1. Frosted ice is `frosted_ice`,
with the property `age` 0 to 3. The snow layer is `snow`, with the property `layers` 1 to 8.

### GRW-ICE-001: The random tick of ice

- **Inputs:** a random tick at ice at P.
- **Behavior:** if the block light at P is more than `11 − 1`, that is, 11 or more, the ice
  melts: when the dimension type has `ultrawarm` true, remove P; otherwise set P to a water
  source. Then the water makes a neighbor update at P, so that the fluid schedules its tick.
- **Edge cases:**
  - The sky light does not melt ice. Ice under the sun stays.
  - Ice melts also when the block below is air.
  - The threshold is the block light at P itself, not at a neighbor. The light dampening of the
    ice (1) is the reason for the threshold 11 and not 12 (U11).
- **Conformance tests:** experiment E-G7.

Provenance: `documentation` for the melt by block light, the water, the nether and the block
below (the wiki states "melts into water if the block light level ... is higher than 11, even if
there is no block below it", "Sky light level is ignored", and "If ice melts or is broken in the
Nether, no water is produced"). The threshold 11 in place of the 12 of the wiki is
`source_suggested`, with the dampening 1 `black_box_confirmed` by `ice.json` (U11).
Kind: semantic. Confidence: medium.

### GRW-ICE-002: Frosted ice

- **Inputs:** a scheduled tick at frosted ice at P with age `a`. Frosted ice does not tick
  randomly (GRW-TBL-001).
- **Behavior:** the placement side effect of frosted ice schedules its tick with the delay
  `20 + r(20)` (U12). The tick does these steps:
  1. Count the frosted ice blocks among the six neighbors of P. Let `c` be the count.
  2. If `r(3)` = 0 or `c` < 4, and the block light at P is more than `11 − a`: if `a` < 3, set
     P to frosted ice with age `a + 1`; otherwise melt P into a water source, and increase the
     age of each neighbor that is frosted ice by one tick of this rule.
  3. Schedule the tick again with the delay `20 + r(20)`.
- **Edge cases:**
  - Frosted ice in darkness never melts.
  - When `random_tick_speed` is 0, frosted ice does not melt (U12).
- **Conformance tests:** none in this document. The frost walker enchantment that makes
  frosted ice is not in this version.

Provenance: `documentation` (the wiki states "ticked every 1–2 seconds", "If a 1/3 chance
succeeds or the block has fewer than 4 frosted ice blocks in the 6 blocks surrounding it, and
the internal light level at the block would be greater than 11 minus the age, then the age is
incremented", and "When the age reaches 4, the block turns into water and all adjacent blocks
have their age incremented"). Kind: semantic. Confidence: low.

### GRW-ICE-003: The random tick of a snow layer

- **Inputs:** a random tick at `snow` at P.
- **Behavior:** if the block light at P is more than 11, the server spawns the drops of the
  state at P (`block-loot.md`) and removes P. All layers go at one time.
- **Edge cases:**
  - The sky light does not melt snow.
  - The snow layer has `minecraft:light_dampening` 0, so the threshold is 12 (compare
    GRW-ICE-001).
  - The drops of the table of `snow` without a tool are nothing.
- **Conformance tests:** experiment E-G7.

Provenance: `documentation` (the wiki states "Snow melts if there is ... block light level of 12
or more" and "they melt all at once in Java Edition"). Kind: semantic. Confidence: medium.

---

## 11. Farmland

Farmland is `farmland`, with the property `moisture` 0 to 7.

### GRW-FRM-001: Water near farmland

- **Inputs:** a farmland at P.
- **Behavior:** farmland at P *has water* when at least one position of the box dx ±4,
  dy 0..+1, dz ±4 around P holds a fluid of type `minecraft:water`, of any level. A waterlogged
  block counts.
- **Edge cases:** the box has 9 × 2 × 9 = 162 positions. Blocks between the water and the
  farmland do not matter.

Provenance: `documentation` (the wiki states "up to four blocks away horizontally (including
diagonally)" and "at the same level or one block above the farmland block level", and that
flowing water counts). Kind: semantic. Confidence: high.

### GRW-FRM-002: The random tick of farmland

- **Inputs:** a random tick at farmland at P with moisture `m`.
- **Behavior:** the tick does these steps in this order:
  1. If the farmland has water (GRW-FRM-001), or there is rain at P + U: if `m` < 7, set P to
     farmland with `moisture` 7, with flags 2. The tick ends.
  2. If `m` > 0, set P to farmland with `moisture` `m − 1`, with flags 2. The tick ends.
  3. If the block at P + U is in the tag `#minecraft:maintains_farmland`, the tick ends.
  4. Set P to `dirt`. The entities on P move up by the height difference (farmland is 15/16 of
     a block high). The crop case does not occur here, because of step 3.
- **Edge cases:**
  - Farmland with a crop on it dries to `moisture` 0 and stays farmland.
  - The moisture jumps from any value to 7 in one tick. It goes down one level in each tick.
  - A fence gate above farmland keeps it farmland; the tag names `#minecraft:fence_gates`.
  - A new farmland from a hoe has `moisture` 0. Its first wet tick sets 7.
  - The change to dirt from a trample, from a solid block placed above, or from a piston is not
    a random tick. Those rules are not in this document.
- **Conformance tests:** experiment E-G8.

Provenance: `documentation` for the count down, the jump to 7 and the dirt without a crop (the
wiki states "The wetness value counts down to 0", "Newly hydrated farmland jumps from 0 to 7",
and "Farmland eventually decays into normal dirt if it's dehydrated and nothing is planted in
it"). The tag is `black_box_confirmed` by `data/minecraft/tags/block/maintains_farmland.json`.
The rain, the flags, the order of the steps and one level for each tick are `source_suggested`.
Kind: semantic. Confidence: medium.

---

## 12. Bone meal responses

The bone meal item is `minecraft:bone_meal`. When a player or a dispenser uses it on a block at
P, the block gives one of two results: it accepts the bone meal and changes the world, or it
rejects it. On an accept, the item logic takes one bone meal in survival and asks the clients
for the growth particles at P. On a reject, the use does nothing: the
item stays, and the player can place a block. These item rules are not in this document.

A response has two parts. The **test** decides whether the block is a target. The **draw**, when
the rule has one, decides whether this use succeeds. A block that passes the test but fails the
draw still accepts the bone meal: the item is consumed and the particles play, and the world
does not change.

### GRW-BON-001: Crops and stems

- **Inputs:** bone meal on a crop or a stem at P with age `a`.
- **Behavior:**
  - Test: `a` is less than the last age.
  - Response: draw `r(4) + 2`, that is, 2 to 5. For `beetroots`, divide the result by 3 with
    integer division, that is, 0 with probability 1/4 and 1 otherwise. For `torchflower_crop`,
    the increase is 1. Set P to the crop or stem with age `min(a + increase, last age)`, with
    flags 2. For `torchflower_crop` at the last age, GRW-CRP-004 applies.
- **Edge cases:**
  - A stem at `age` 7 rejects the bone meal. Bone meal never makes a fruit.
  - A beetroot that draws 0 accepts the bone meal and does not change.
- **Conformance tests:** experiment E-G9.

Provenance: `documentation` (the wiki states "The plant matures 2–5 growth stages" for wheat,
carrots, potatoes and the stems, "75% chance of growing to the next growth stage" for beetroots,
"Grows every stage" for the torchflower crop, and "bone meal does not help produce the actual
melons"). The division by 3 is `derived` from the 75%. Kind: semantic. Confidence: medium.

### GRW-BON-002: Saplings

- **Inputs:** bone meal on a sapling at P.
- **Behavior:**
  - Test: true.
  - Draw: `u` < 0.45.
  - Response: GRW-SAP-001 steps 3 and 4, without the light test.
- **Edge cases:**
  - Bone meal grows a sapling in darkness.
  - A dark oak sapling that is not in a square accepts the bone meal and does not change.
  - A sapling whose tree finds no room accepts the bone meal and does not change.
- **Conformance tests:** experiment E-G9.

Provenance: `documentation` (the wiki states "45% chance of growing to the next growth stage, if
possible", "even without sufficient light", and that bone meal on an obstructed sapling is
wasted). Kind: semantic. Confidence: medium.

### GRW-BON-003: Grass block

- **Inputs:** bone meal on a grass block at P.
- **Behavior:**
  - Test: the block at P + U is air.
  - Response: do 128 attempts. Attempt number `i`, from 0 to 127, does these steps:
    1. Let Q = P + U. Do `floor(i / 16)` steps: draw the offset `(r(3) − 1, (r(3) − 1) × r(3) / 2,
       r(3) − 1)` with integer division, and add it to Q. After each step, if the block at
       Q + D is not a grass block, or the block at Q has a full collision box, end this attempt.
    2. If the block at Q is `short_grass` and `r(10)` = 0, run the bone meal response of the
       short grass at Q (it becomes `tall_grass`). Continue with step 3.
    3. If the block at Q is air: with `r(8)` = 0, place the flower feature of the biome at Q
       (U13); otherwise place the placed feature `minecraft:grass_bonemeal` at Q. A feature
       places nothing where its block cannot stay.
- **Numeric semantics:** the attempts reach at most 7 steps from P + U, that is, the box
  dx ±7, dy ±7, dz ±7. The vertical step is 0 with probability 2/3.
- **Edge cases:**
  - The response accepts the bone meal also when all 128 attempts place nothing.
  - Mycelium has no response. It rejects bone meal.
  - A two-block flower never comes from this response.
- **Conformance tests:** experiment E-G9.

Provenance: `documentation` for the area and the plants (the wiki states "Grass blocks within a
15×15×15 ... area of the target block randomly choose which type of foliage to grow" and names
short grass, tall grass, flowers and pink petals). The placed feature `grass_bonemeal` is
`black_box_confirmed` by its file (the feature `minecraft:grass` places `minecraft:short_grass`
where the block is air). The 128 attempts, the walk, the draws 1/10 and 1/8 are
`source_suggested`. The flower feature of the biome is U13. Kind: semantic. Confidence: low.

### GRW-BON-004: Bamboo

- **Inputs:** bone meal on a bamboo sapling or on bamboo at P.
- **Behavior:**
  - Bamboo sapling. Test: the block at P + U is air. Response: GRW-BAM-001 step 4.
  - Bamboo. Test: let T be the top bamboo of the column of P, and `h` its height as in
    GRW-BAM-002 step 4. The test is true when the state at T has `stage` 0, the block at T + U
    is air, and `h` < 16. Response: let `k` be `r(2) + 1`. Do `k` times: if the block at T + U is
    air and `h` < 16, grow one block at T (GRW-BAM-003), then let T be T + U and `h` be `h + 1`.
- **Edge cases:**
  - Bone meal on a bamboo in the middle of a column grows the top of the column.
  - A bamboo with `stage` 1 at the top rejects the bone meal.
- **Conformance tests:** experiment E-G9.

Provenance: `documentation` (the wiki states "Grows the bamboo by 1–2 stems", "Transforms a
bamboo shoot into a 2-block tall bamboo", and that bamboo blocked above wastes the bone meal).
The test at the top of the column is `source_suggested`. Kind: semantic. Confidence: medium.

### GRW-BON-005: Blocks that reject bone meal

- **Behavior:** `sugar_cane`, `cactus`, `attached_melon_stem`, `attached_pumpkin_stem`, a
  crop or a stem at its last age, bamboo with `stage` 1 at the top of its column, `mycelium`,
  `farmland`, `ice`, `snow`, the leaves, and each block that no rule of this document or of a
  later document gives a response, reject bone meal. The item stays.
- **Conformance tests:** experiment E-G9, step for stone.

Provenance: `documentation` (the wiki states "Bone meal does not affect: Cactus, Vines, Nether
wart, Sugar cane (Java only), Chorus plants, or other plants not listed"). Kind: semantic.
Confidence: high.

---

## 13. Footprints

### GRW-FOOT-001: The footprint of each handler

- **Behavior:** Table 8 gives the box of positions that one call of a handler of this document
  reads or writes, relative to P, in the notation of `block-simulation.md`, SIM-MODEL-009. The
  light reads are at the positions of the box. A set of the handler is in the box; the neighbor
  updates and the shape updates of the set are new items with their own bounds.

**Table 8. Footprints.**

| Rule | `dx` | `dy` | `dz` | Sets | Other effects |
|------|------|------|------|------|---------------|
| GRW-CRP-003 | ±1 | −1..0 | ±1 | P | none |
| GRW-STM-001 | ±1 | −1..0 | ±1 | P and one horizontal neighbor | none |
| GRW-CAN-001 | 0 | −n..+1 | 0 | P and P + U | none. `n` as Note 2 of SIM-MODEL-009 |
| GRW-CAC-001 | ±1 | −n..+2 | ±1 | P and P + U | a neighbor update at P + U |
| GRW-BAM-001 | 0 | 0..+1 | 0 | P and P + U | none |
| GRW-BAM-002, GRW-BAM-003 | 0 | −16..+1 | 0 | P + U, P + D, P + 2D | none |
| GRW-SAP-001, GRW-SAP-002 | ±10 | −13..+34 | ±10 | the blocks of one tree feature, and the saplings of the square | none. Note 1 of SIM-MODEL-009: the bound holds for the vanilla features |
| GRW-SPR-002 | ±1 | −3..+2 | ±1 | P, or up to 4 dirt blocks in the box | none |
| GRW-LEF-001 (tick) | ±1 | ±1 | ±1 | P | none |
| GRW-LEF-002 | 0 | 0 | 0 | P | item drops |
| GRW-ICE-001 | 0 | 0 | 0 | P | a neighbor update at P |
| GRW-ICE-002 | ±1 | ±1 | ±1 | P and its six neighbors | none |
| GRW-ICE-003 | 0 | 0 | 0 | P | item drops |
| GRW-FRM-002 | ±4 | 0..+1 | ±4 | P | the rain at P + U; the entities on P move up |
| GRW-BON-001 | 0 | 0 | 0 | P | none |
| GRW-BON-002 | as GRW-SAP-002 | | | | |
| GRW-BON-003 | ±7, or more (U13) | ±7 | ±7 | up to 128 blocks in the box | none |
| GRW-BON-004 | 0 | −16..+2 | 0 | up to 2 new blocks and 2 leaf changes | none |

- **Edge cases:** a tree feature that a data pack adds can reach past the bound of GRW-SAP-002.
  An instance that cannot measure the extent of a feature must treat the handler as a handler
  with no bound (SIM-MODEL-009, Note 1).

Provenance: `derived` from the rules of this document, and consistent with SIM-MODEL-009,
Table 1 (`source_suggested_black_box_confirmed` against that table). GRW-CAC-001 adds +2 for
the flower neighbors. Kind: semantic. Confidence: medium.

---

## 14. Intentional deviations

- **GRW-DEV-001.** **Rule:** each rule with a draw. **Reference:** all draws of all handlers
  come from one dimension random, and the picks of one section are consecutive outputs of one
  generator (SIM-RND-002, SIM-RNG-001). **Reason:** an instance draws the picks and the values
  of each handler from a stream of its own for each pick. SIM-MODEL-106(b) allows this: only
  the distributions are invariants. **Visible to the client:** no. The rates of each rule stay.
  **Bug reference:** none. **Test:** a differential test compares distributions over many
  runs, never one run.
- **GRW-DEV-002.** **Rule:** each rule of this document. **Reference:** the order of the picks
  across sections is the iteration order of a hash table (`block-simulation.md`, OQ-04).
  **Reason:** a player sees no effect of the order, and vanilla cannot repeat it. An instance
  uses a stable order. **Visible to the client:** no. **Bug reference:** none. **Test:** none.

---

## 15. Unresolved

Each item names the rules that it affects and the experiment or the source that decides it.

- **U1: The condition of each randomly ticking state, and its data form** (GRW-TBL-001,
  GRW-TBL-002). The block definitions carry no flag. The conditions of the "later" row of
  Table 1 are not stated. Proposal to the owner: let the update tool that writes the block
  definitions also write the property "randomly ticking" for each block state, from the game,
  into the definitions or into a report under `assets/mcrs/reports/`. Until then, Table 1 is
  the source, and a data pack cannot add a randomly ticking block.
- **U2: The sky darken** (GRW-SAP-001, GRW-SPR-002). Its exact function of the time of day and
  the weather is not in any specification. The client computes the same value for the light
  that it shows. Experiment E-G5 shows the two ends: noon and midnight.
- **U3: Crops on a base that is not farmland** (GRW-CRP-002). A crop cannot stay there
  (GRW-CRP-001), so only a command makes the case. Decide by policy: the formula gives `g` from
  1.
- **U4: The skip of the torchflower crop** (GRW-CRP-003, step 2). The wiki states the skip for
  beetroots only. Experiment E-G1 with `torchflower_crop` decides it.
- **U5: Shape functions of the growth blocks** (GRW-CAN-001, GRW-CAC-001, GRW-STM-001). The
  survival of sugar cane (water next to the base, or `#minecraft:supports_sugar_cane_adjacently`),
  the break of a cactus next to a solid block, and the attached stem that loses its fruit are
  shape functions. `block-support.md` does not state them, and its data form (SUP-PRED-002)
  allows only offsets of one step, while the water test of sugar cane reads P + D + horizontal.
  Question to the author of `block-support.md`.
- **U6: The age at which a cactus draws for a flower** (GRW-CAC-001, step 3). The wiki states
  "exactly 9 random ticks". Experiment E-G4 with a cactus at each age decides whether the draw
  is at `age` 8 or at another age, and whether the `n` for the chance counts P.
- **U7: The states that the bamboo sapling sets** (GRW-BAM-001, step 4). Experiment E-G4.
- **U8: The azalea, the mangrove propagule and the poplar sapling** (GRW-SAP-003). Their
  features and draws are not documented for this version. Experiment E-G5 with each of them.
- **U9: Death of the grass block under a block that is not a full occluder** (GRW-SPR-001). The
  wiki states a light level of 4; this document states the full occluder. Experiment E-G6 with
  glass, an upside-down slab and a torch decides it.
- **U10: Leaves** (GRW-LEF-001, GRW-LEF-002). Which shape updates schedule the distance tick,
  and whether a decay sends the level event 2001. Experiment E-G7 with an observer and a
  client recorder.
- **U11: The melt threshold of ice** (GRW-ICE-001). 11 or 12. Experiment E-G7 with a torch at
  the Manhattan distances 3 and 4.
- **U12: The schedule of frosted ice** (GRW-ICE-002). The delay and the role of
  `random_tick_speed`. No experiment at this version: nothing makes frosted ice.
- **U13: The flower feature of the biome for bone meal** (GRW-BON-003). The data pack of this
  version has no configured feature of the type `minecraft:flower`; the flowers of a biome are
  placed features such as `minecraft:flower_plains` with their own `count` and `offset`. Which
  feature the response places, and with which placement, is not known. Experiment E-G9 with a
  flat world in plains: count the flowers and their distance from P after 100 uses.
- **U14: The distribution of the growth time** (GRW-CRP-003). Experiment E-G1 gives the
  fixture for the exhaustive tier. It has not run.

---

## 16. Experiments

Each experiment runs on a vanilla dedicated server of the version of
`assets/minecraft/version.json`, with `online-mode=false`, `level-type=minecraft:flat`,
`pause-when-empty-seconds=-1` and `spawn-protection=0`. The commands go to the console. The
default flat world has a grass block at y −61 and air above. Each case stands in its own column,
force-loaded with `forceload add`. `tick freeze` stops the ticks before the scene is built, and
`tick step <n>` runs exactly `n` ticks. `gamerule random_tick_speed <k>` sets the rate. The
result of a step is read with `execute if block <P> <state>`, which prints "Test passed" to the
console. A histogram over a field is read with `clone` of the field to a scratch area, then
`fill ... minecraft:structure_void replace <state>` for each state, which prints the count.

A dark room is a box of stone of 15 × 5 × 15 inside, with a roof two blocks thick. The block
light at a position inside comes from a torch at a known Manhattan distance: a torch gives 14,
and the light falls by 1 for each block. The sky light inside is 0.

A fixture of an experiment holds the input (the `setblock` commands, `k`, the number of ticks
and of runs) and the observation: a list of `(tick, position, state)` for a deterministic case, a
histogram (state → count) over `N` runs for a random case, and a list of `(dx, dy, dz, block)`
relative to the sapling for a tree.

- **E-G1** (the growth formula). In a dark room with torches that give light 15 at each crop:
  a lone wheat on moist farmland (a water source at distance 4 at the same level); a 9 × 9
  field of wheat on moist farmland with water in the center; a lone wheat on dry farmland; two
  wheat diagonal to each other on dry farmland; the same with `beetroots` and
  `torchflower_crop`. Use `k` = 3 and `tick step 20000`, in 20 runs. Record the histogram of
  the ages. Expected: the mean count of growth steps follows GRW-CRP-003 (1/7 for the lone
  moist wheat, 1/3 in the center of the field, 1/13 for the dry wheat, 1/23 for the diagonal
  pair, 2/3 of these rates for beetroots).
- **E-G2** (light and survival). In a dark room, wheat on moist farmland with a torch at the
  Manhattan distances 5 (light 9), 6 (light 8) and 7 (light 7). Use `k` = 300 and
  `tick step 2000`. Expected: the wheat at light 9 reaches `age` 7; the wheat at light 8 stays
  at `age` 0 and stays wheat; the wheat at light 7 stays wheat until a shape update: then set a
  block next to it; expected: it pops. Then, under the open sky: `time set midnight`, wheat on
  moist farmland with no torch, `tick step 2000`. Expected: `age` 7.
- **E-G3** (the fruit). Under light 15, `melon_stem[age=7]` on farmland with dirt at the four
  horizontal neighbors, in 200 copies. Use `k` = 300 and `tick step 400`. Expected: each copy
  that grew a fruit is an attached stem that faces the fruit; the four directions each have
  about 1/4 of the fruits. Then the same with stone instead of dirt. Expected: no fruit.
- **E-G4** (the columns). Under light 15: sugar cane on sand next to water; cactus on sand;
  `bamboo_sapling` on dirt; `cactus[age=8]` with free sides, in 100 copies. Use `k` = 300 and
  `tick step 4000`. Expected: sugar cane and cactus reach a height of 3 and not 4; bamboo
  reaches 12 to 16 and never 17, with `large` leaves at the top, `small` below it, and `age` 1
  from the third block; the cactus copies grow a flower in about 10 of 100 cases, and a cactus
  with a flower stays at its height.
- **E-G5** (the saplings). Under the open sky at noon: one sapling of each type of Table 7, a
  2 × 2 square of dark oak, of pale oak, of spruce and of jungle, and an oak sapling under a
  stone block at P + 2U. Use `k` = 4096 and `tick step 400`. Expected: each sapling grows the
  tree of Table 7, with leaves that have `persistent` false; the squares grow one large tree
  from the north-west corner; the sapling under stone stays, with `stage` 1 after a time. Then
  `time set midnight`, a new oak sapling, `tick step 2000`. Expected: it stays a sapling.
  Record each tree as `(dx, dy, dz, block)`.
- **E-G6** (grass and mycelium). Under light 15, a grass block next to dirt, coarse dirt and
  rooted dirt; a grass block under stone, under glass, under a water source, under one snow
  layer, under two snow layers, under an upside-down stone slab, under a torch in a glass
  room; the same cases with mycelium. Use `k` = 300 and `tick step 2000`. Expected: only the
  dirt becomes a grass block (mycelium for mycelium); the grass under stone, under water and
  under two snow layers is dirt; the others stay (U9 decides the slab).
- **E-G7** (leaves, ice, snow). `oak_leaves[distance=7]` alone, in 100 copies, `k` = 300,
  `tick step 2000`: expected: none remains, and the drops lie on the ground. A line of 8 leaves
  from an oak log: expected: `distance` 1 to 6 for the first six, and the two last decay. Ice
  with a torch at the Manhattan distance 3 and another at 4, `tick step 2000`: expected: the
  first is water, the second decides U11. A snow layer with `layers` 5 next to a torch at the
  distance 2: expected: air. Ice under the sun with no torch: expected: ice.
- **E-G8** (farmland). Farmland at the distances 1 to 5 from a water source, at the same
  level, one below and one above; farmland with no water, with and without wheat on it. Use
  `k` = 300 and `tick step 400`. Expected: `moisture` 7 up to the distance 4 at the same level
  and one above; 0 at the distance 5 and for the water one below; the dry farmland without
  wheat is dirt, and the dry farmland with wheat has `moisture` 0 and is farmland.
- **E-G9** (bone meal). A dispenser with 64 bone meal that faces a `wheat[age=0]`, pulsed
  one time with a block of redstone and reset, in 200 copies: expected: the histogram of the
  ages is 2 to 5 with about 50 copies each, and each dispenser holds 63. The same at stone:
  expected: the dispenser holds 64. The same at `beetroots[age=0]`: expected: about 150 copies
  at `age` 1 and 50 at `age` 0, and each dispenser holds 63. The same at an oak sapling in a
  dark room, 200 copies: expected: about 90 copies grew a tree or changed `stage`. The same at
  a grass block in plains, 100 uses at one block: expected: short grass, tall grass and flowers
  within dx ±7; record the farthest distance for U13.

The experiments E-G1 and E-G9 give the fixtures for the exhaustive tier of the growth tests.
