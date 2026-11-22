# Random ticks of the growth blocks: behavioural specification

Version: see `assets/minecraft/version.json`.

This document states what a vanilla block does when the machinery of `block-simulation.md`
delivers a random tick to it. It also states what a block does when a player or a dispenser
uses bone meal on it. It covers the crops, the stems, sugar cane, cactus, bamboo, the saplings,
the grass block and mycelium, the leaves, ice and snow, and farmland.

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
- The room that a tree feature needs, and the blocks that it places: these are the rules of
  the feature, as the world generation runs it.

## 0. Conventions

### 0.1 Terms

- **Position**, **block state**, **block type**, **set**, **neighbor update**, **shape update**,
  **shape function**, **scheduled tick**, **placement side effect**, **removal side effect**,
  **randomly ticking**, **section**, **dimension random**: as `block-simulation.md`,
  section 0.1, defines them.
- **Random tick**: one call of the random tick handler of the block state at a position P. It
  is step 3 of a pick (SIM-RND-002). A fluid random tick is step 4 of a pick.
- **Random of the pick**: the random source that the handler draws from. In vanilla it is the
  dimension random (SIM-RNG-001). `r(n)` is a uniform integer draw in `[0, n)`, and `u` is a
  uniform draw in `[0, 1)`, both from the random of the pick. A draw occurs only where a rule
  names it.
- **Block light at P**, **sky light at P**: the two stored light values at P, each 0 to 15. The
  sky light does not depend on the time of day.
- **Sky darken**: `15 − v`, where `v` is the value of the environment attribute
  `gameplay/sky_light_level` of the dimension (`environment-attributes.md`). It is 0 in full
  daylight and 11 at midnight.
- **Light at P**: the larger of the block light at P and the sky light at P.
- **Ambient light at P**: the larger of the block light at P and the sky light at P minus the
  sky darken.
- **Age**, **stage**, **moisture**, **distance**, **persistent**, **layers**, **leaves**,
  `snowy`, `facing`, `hanging`: the properties of the block states of this document. The block
  definitions under `assets/mcrs/block_definition/` give their values.
- **Growth speed** of a crop or stem at P: the value `g` of GRW-CRP-002.
- **Moist farmland**: a farmland state with `moisture` above 0. **Dry farmland**: `moisture` 0.
  Only `moisture` 7 has the wet texture on the client.
- **Fruit**: `minecraft:melon` for a melon stem and `minecraft:pumpkin` for a pumpkin stem.
- **Tree feature**: a configured feature of the data pack under `data/minecraft/worldgen/feature/`
  with the type `minecraft:tree`, for example `minecraft:oak`.
- **Bone meal response**: what a block does when bone meal is used on it. The block **accepts**
  the bone meal when it is a target of section 12, and **rejects** it otherwise.
- **Rain at a position**: the test of `block-simulation.md`, SIM-FIRE-004, for one position.
- **Solid**: the property of a block state of `block-support.md`, SUP-SOLID-001.
- **Remove P**: the "remove block" of SIM-SET-004 at P, without drops.
- **Destroy P**: the destroy of SIM-SET-004 at P, with the drop flag set.
- **Set P to S**: a set with flags 3 (SIM-SET-002), unless the rule names other flags. Flags 2
  sync the clients and make no neighbor updates. Flags 260 make no neighbor updates and do not
  reach the clients.
- **Column of P** for sugar cane, cactus and bamboo: the blocks of the same block type in the
  vertical line through P. Its **height** `h` counts P and the blocks below P, down to the
  first block of another type.

### 0.2 Provenance

- `black_box_confirmed`: shown by a game-produced artefact that is cited by path, or by an
  experiment of section 16.
- `source_suggested_black_box_confirmed`: taken from the reference behaviour, and consistent
  with a cited artefact or with an experiment of section 16.
- `source_suggested`: taken from the reference behaviour, with no artefact yet. Section 15
  names the experiment that is still to run.
- `documentation`: taken from public documentation of player-facing behaviour.
- `derived`: it comes from other rules of this document or of `block-simulation.md`.
- `design`: a choice of this project. The rule states the reason.
- `unresolved`: not decided. Section 15 collects these items.

Each rule states its kind, as `docs/spec/README.md` defines it. Each number of this document has
a provenance, or the rule names the item of section 15 that holds the open question.

### 0.3 Artefacts cited

- The block definitions under `assets/mcrs/block_definition/`: the properties `age`, `stage`,
  `moisture`, `distance`, `persistent`, `layers`, `leaves`, `snowy`, `facing` and `hanging`,
  the component `minecraft:light_dampening`, and the component `minecraft:placement_filter`
  of the stems, the saplings, the bamboo sapling and the snow layer.
- `assets/mcrs/registry_definition/game_rule.json`: `minecraft:random_tick_speed` (integer,
  default 3, minimum 0).
- The data pack of the game, under `data/minecraft/tags/block/`: `supports_crops`,
  `grows_crops`, `supports_melon_stem`, `supports_pumpkin_stem`, `supports_melon_stem_fruit`,
  `supports_pumpkin_stem_fruit`, `supports_stem_fruit`, `supports_vegetation`,
  `substrate_overworld`, `supports_sugar_cane`, `supports_sugar_cane_adjacently`,
  `supports_cactus`, `supports_bamboo`, `maintains_farmland`, `prevents_nearby_leaf_decay`,
  `logs`, `snow`, `saplings`, `leaves`, `flowers`, `cannot_support_snow_layer`,
  `support_override_snow_layer`, `support_override_cactus_flower`; under
  `data/minecraft/tags/fluid/`: `water`, `supports_sugar_cane_adjacently`; under
  `data/minecraft/tags/worldgen/feature/`: `can_spawn_from_bone_meal`.
- The data pack of the game, under `data/minecraft/worldgen/feature/`: the tree features of
  Table 7 (GRW-SAP-003) and the features of the tag `can_spawn_from_bone_meal`; under
  `data/minecraft/worldgen/placed_feature/`: `minecraft:grass_bonemeal`.
- `environment-attributes.md`: the attributes `gameplay/sky_light_level` and
  `gameplay/water_evaporates`.
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
| each block of the tag `#minecraft:saplings`, except `azalea` and `flowering_azalea` | all | section 7 |
| `azalea`, `flowering_azalea` | none: they react to bone meal only (GRW-BON-002) | — |
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
  - A mangrove propagule ticks randomly in each state. A propagule that hangs with `age` 4
    receives the tick and does nothing (GRW-SAP-001).
  - A lava block is a randomly ticking block whose handler runs the fluid handler. The pick then
    runs the fluid handler again (SIM-RND-002, Edge cases).
- **Conformance tests:** make a section with one `wheat[age=7]` and nothing else that ticks.
  Use `random_tick_speed` 4096 for 10 000 ticks. Result: the position generator of the section
  does not advance (SIM-RND-002, Invariants). Failing twin: the generator advances, because the
  implementation treats each crop state as randomly ticking.

Provenance: `source_suggested_black_box_confirmed` for the block types and the fluid type:
`block-simulation.md`, SIM-MODEL-009, Table 1, lists the same block types, and the data pack
gives the tags. `source_suggested` for the conditions of the rows that name a section. The
conditions of the "later" row are U1. The existence of the properties is `black_box_confirmed`
by the block definitions. Kind: semantic. Confidence: high.

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
  - The light at P is 8 or more.

  When a crop cannot stay, its shape function gives air, and the server destroys P
  (`block-support.md`, SUP-RUN-001). The test runs on a shape update from each direction and
  on a placement. A random tick does not run the test.
- **Edge cases:**
  - A crop in darkness stays until a shape update reaches it. A change of the light alone does
    not remove it.
  - The sky light at P is 15 under the open sky, also at night. A crop under the open sky never
    pops at night. A crop under glass is in the same case, because glass has
    `minecraft:light_dampening` 0.
- **Conformance tests:** experiment E-G2.

Provenance: `source_suggested_black_box_confirmed` for the base: the tag file
`data/minecraft/tags/block/supports_crops.json` has the member `minecraft:farmland`.
`source_suggested_black_box_confirmed` for the light 8 and for the moment of the test:
experiment E-G2 (wheat at light 8 stays through a shape update; wheat at light 7 and in
darkness stays until a shape update, then pops). The wiki agrees ("a light level of 8 or greater
to plant"). Kind: semantic. Confidence: high.

### GRW-CRP-002: Growth speed

- **Inputs:** a crop or a stem of block type B at P. F is the position P + D.
- **Behavior:** the growth speed `g` is the sum of these terms:
  - 1.
  - The block at F: 3 when it is in the tag `#minecraft:grows_crops` with `moisture` above 0,
    1 when it is in the tag with `moisture` 0 or without that property, 0 otherwise.
  - Each of the eight positions around F: a quarter of the same value for the block there:
    0.75, 0.25 or 0.

  Then `g` is halved when one of these conditions is true:
  - At least one of P + W and P + E holds a block of type B, and at least one of P + N and
    P + S holds a block of type B.
  - Otherwise, at least one of the four diagonal positions P + (±1, 0, ±1) holds a block of
    type B.
- **Numeric semantics:** `g` is a 32-bit float. For a crop on farmland its value before the
  half rule is 2 to 10, in steps of 0.25. A crop on another base has `g` from 1 to 7 (U3).
- **Edge cases:**
  - A different crop next to the crop does not halve `g`. Carrots next to wheat count as
    nothing.
  - The half rule reads only P + W, P + E, P + N, P + S and the four diagonals. Two crops of one
    type in one row (P + W and P + E both wheat, nothing to the north or south) do not halve
    `g`. One crop to the west and one to the north do.
  - Farmland with `moisture` 1 to 6 counts as moist. The client shows the wet texture only at
    `moisture` 7.
- **Conformance tests:** kernel: compute `g` for the scenes of GRW-CRP-003, Numeric semantics,
  and compare. Failing twin: a half rule that needs both P + W and P + E gives `g` = 2.5 for the
  "west and north" scene instead of 1.25.

Provenance: `source_suggested_black_box_confirmed` for the tag: the file
`data/minecraft/tags/block/grows_crops.json` has the member `minecraft:farmland`.
`source_suggested` for the terms and the half rule; the wiki agrees on 2 and 4, 0.25 and 0.75
and on the diagonal, and its words "in both the north-south and east-west directions" do not
state the half rule exactly. Kind: semantic. Confidence: high. U15 names the distribution test.

### GRW-CRP-003: The random tick of a crop

- **Inputs:** a random tick at a crop at P with age `a`.
- **Behavior:** the tick does these steps in this order:
  1. If the crop is `beetroots` or `torchflower_crop`, draw `r(3)`. If the result is 0, the
     tick ends.
  2. If the light at P is less than 9, the tick ends.
  3. Let `g` be the growth speed (GRW-CRP-002). Draw `r(floor(25 / g) + 1)`. If the result is
     not 0, the tick ends.
  4. Set P to the crop with age `a + 1`, with flags 2.
- **Numeric semantics:** the chance of step 3 is `1 / (floor(25 / g) + 1)`. For a lone crop on
  moist farmland with no farmland around, `g` = 4 and the chance is 1/7. For a crop with moist
  farmland below and around, `g` = 10 and the chance is 1/3. For a lone crop on dry farmland,
  `g` = 2 and the chance is 1/13. Two crops of one type diagonal to each other on dry
  farmland have `g` = 1.125 and the chance 1/23. Beetroots and the torchflower crop pass step 1
  in 2 of 3 ticks.
- **Edge cases:**
  - Step 1 draws before the light test. The draw occurs also in darkness.
  - The light test reads the light at P, not at P + U. The sky light counts in full at night.
    As a result, a crop under the open sky grows at night.
  - The tick does not test the base and does not remove the crop (GRW-CRP-001).
  - A set with flags 2 makes no neighbor updates. The clients get the change. The shape updates
    of the set run (SIM-SET-002), so an observer that faces the crop pulses.
  - The torchflower crop has `age` 0 and 1 only. Its step 4 at `age` 1 is GRW-CRP-004.
- **Conformance tests:** experiment E-G2. Kernel: a lone wheat on moist farmland in light 15
  over 100 000 picks grows in 1/7 of them (binomial tolerance). Failing twin: 1/6, because the
  division did not floor.

Provenance: `source_suggested_black_box_confirmed` for the light threshold and the sky light at
night: experiment E-G2 (growth at light 9, none at light 8, growth at midnight under the open
sky). `source_suggested` for the chance formula and the skip (U15). The wiki agrees on the
light 9, the chance formula and the skip of beetroots ("a 1/3 chance of not advancing"), and
does not state the skip of the torchflower crop. Kind: semantic. Confidence: high.

### GRW-CRP-004: The last age of the torchflower crop

- **Inputs:** step 4 of GRW-CRP-003 or a bone meal response (GRW-BON-001) at a
  `torchflower_crop[age=1]` at P.
- **Behavior:** the server sets P to `minecraft:torchflower` with flags 2 instead of a crop with
  `age` 2. The torchflower is a plant of `block-support.md`, SUP-VEG-001, and it does not tick
  randomly.
- **Conformance tests:** use bone meal two times on a `torchflower_crop[age=0]`. Result: P holds
  `minecraft:torchflower`.

Provenance: `source_suggested_black_box_confirmed`: the block definition of `torchflower_crop`
has `age` 0 and 1 only. The wiki agrees ("the final growth stage of the torchflower crop uses
the same ID as the actual torchflower"). Kind: semantic. Confidence: high.

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
  - The attached stem does not tick randomly (GRW-STM-002).
  - Both fruit tags resolve to `#minecraft:supports_vegetation`: `#minecraft:substrate_overworld`
    (dirt, mud, moss blocks and grass blocks) and `minecraft:farmland`.
- **Conformance tests:** kernel: a `melon_stem[age=7]` on farmland with dirt at the four
  horizontal neighbors, over 100 000 picks in light 15. Result: each direction gets 1/4 of the
  fruits (binomial tolerance), and each fruit has an attached stem that faces it. U15 names the
  black-box test.

Provenance: `source_suggested_black_box_confirmed` for the tags: the files
`data/minecraft/tags/block/supports_stem_fruit.json` and `supports_vegetation.json`. The wiki
agrees on the light 9, the fruit on a free side and the list of bases. `source_suggested` for
the uniform direction, the air test and the order of the two sets. Kind: semantic.
Confidence: high.

### GRW-STM-002: The attached stem

- **Inputs:** a shape update at an attached stem at P from the direction d.
- **Behavior:** when d is the `facing` of the attached stem and the block at P + d is not the
  fruit, the shape function gives the stem with `age` 7. The attached stem can stay when the
  block at P + D is in the tag `#minecraft:supports_melon_stem` or
  `#minecraft:supports_pumpkin_stem`, as the stem can.
- **Edge cases:** a player who takes the fruit gets a stem with `age` 7 at once, in the chain
  of the removal. The stem then grows a new fruit by GRW-STM-001.
- **Conformance tests:** kernel: remove the fruit of an attached stem. Result: P holds the stem
  with `age` 7 after the chain.

Provenance: `source_suggested`. The wiki agrees ("When the melon is removed, the stem returns to
its straight shape"). Kind: semantic. Confidence: high.

---

## 4. Sugar cane

Sugar cane is `sugar_cane`, with the property `age` 0 to 15.

### GRW-CAN-001: The random tick of sugar cane

- **Inputs:** a random tick at sugar cane at P with age `a`.
- **Behavior:** the tick does these steps in this order:
  1. If the block at P + U is not air, the tick ends.
  2. Let `h` be the height of the column of P. If `h` is 3 or more, the tick ends.
  3. If `a` is 15: set P + U to `sugar_cane[age=0]`, then set P to `sugar_cane[age=0]` with
     flags 260. The tick ends.
  4. Set P to `sugar_cane` with age `a + 1`, with flags 260.
- **Numeric semantics:** the top block of a column grows one block after 16 of its random ticks.
  A column grows from 1 to 3 blocks after 32 random ticks of its top block.
- **Edge cases:**
  - The light does not matter. Sugar cane grows in darkness.
  - A set with flags 260 does not reach the clients and makes no neighbor updates. The client
    does not see the age. It sees the new block of step 3, which is a set with flags 3.
  - The tick does not test the water next to the base (GRW-CAN-002).
  - When a player breaks the second block of a column, the base keeps its age. The base is the
    new top, and its count continues from its own age.
- **Conformance tests:** kernel: sugar cane with `age` 15 on sand next to water, P + U air.
  Result: after one tick, P + U holds `sugar_cane[age=0]` and P has `age` 0. A column of 3
  with `age` 15 at the top does not grow. U15 names the black-box test.

Provenance: `source_suggested`. The wiki agrees on the height 3, the 16 random ticks and the
growth without light. Kind: semantic. Confidence: high.

### GRW-CAN-002: Survival of sugar cane

- **Inputs:** a shape update at sugar cane at P, or the scheduled tick of the sugar cane at P.
- **Behavior:** sugar cane can stay at P when one of these conditions is true:
  - The block at P + D is sugar cane.
  - The block at P + D is in the tag `#minecraft:supports_sugar_cane`, and at least one
    horizontal neighbor of P + D holds a fluid of the fluid tag
    `#minecraft:supports_sugar_cane_adjacently` or a block of the block tag
    `#minecraft:supports_sugar_cane_adjacently`.

  On a shape update, when the sugar cane cannot stay, the server schedules a block tick at P
  with the delay 1 (SIM-SCH-003). The shape function does not change the state. On the
  scheduled tick, when the sugar cane cannot stay, the server destroys P.
- **Edge cases:**
  - The fluid tag resolves to `#minecraft:water`, and the block tag to `minecraft:frosted_ice`.
    Flowing water counts. A waterlogged block counts, because its fluid is water.
  - The water is read next to the base, not next to the sugar cane. A block between the base
    and the water does not matter, because the test reads the fluid at the four positions only.
  - The destroy comes one tick after the shape update. A column loses its blocks from the
    bottom, one tick for each block, through the shape updates of each destroy.
- **Conformance tests:** kernel: sugar cane on sand with water to the east of the sand. Set
  the water to air. Result: a tick is pending at P; one tick later P is air and the drop is
  spawned.

Provenance: `source_suggested_black_box_confirmed` for the tags: the files
`data/minecraft/tags/block/supports_sugar_cane.json`, `supports_sugar_cane_adjacently.json` and
`data/minecraft/tags/fluid/supports_sugar_cane_adjacently.json`. `source_suggested` for the
tick. The wiki agrees that sugar cane "uproots on the next block update". Kind: semantic.
Confidence: high.

---

## 5. Cactus

Cactus is `cactus`, with the property `age` 0 to 15. A cactus flower is `cactus_flower`.

### GRW-CAC-001: The random tick of a cactus

- **Inputs:** a random tick at a cactus at P with age `a`.
- **Behavior:** the tick does these steps in this order:
  1. If the block at P + U is not air, the tick ends.
  2. Let `h` be the height of the column of P. If `h` is 3 or more and `a` is 15, the tick
     ends.
  3. If `a` is 8 and a cactus can stay at P + U (GRW-CAC-002): draw `u`. If `u` ≤ 0.1 when `h`
     is 1 or 2, or `u` ≤ 0.25 when `h` is 3 or more, set P + U to `cactus_flower`.
  4. Otherwise, if `a` is 15 and `h` is less than 3: set P + U to `cactus[age=0]`, set P to
     `cactus[age=0]` with flags 260, and make a neighbor update at P + U for the new cactus.
  5. If `a` is less than 15, set P to `cactus` with age `a + 1`, with flags 260.
- **Numeric semantics:** the draw of step 3 is a 64-bit float in `[0, 1)`. A cactus of height 1
  or 2 grows a flower with probability 0.1 at its ninth random tick, and a cactus of height 3
  with probability 0.25.
- **Edge cases:**
  - The light does not matter.
  - Step 5 runs also after step 3 placed a flower. The cactus then has `age` 9 and never
    grows past the flower, because P + U is not air.
  - At `age` 15 and height 3, the tick ends at step 2. The age stays 15.
  - The new cactus of step 4 runs its survival test at once through the neighbor update. When a
    solid block is next to it, it is destroyed one tick later (GRW-CAC-002). The growth still
    occurs.
  - The flower test of step 3 reads the survival predicate of a cactus at P + U: no solid
    block and no lava at the four horizontal neighbors of P + U, and no liquid at P + 2U.
- **Conformance tests:** kernel: `cactus[age=8]` with free sides, over 100 000 ticks. Result: a
  flower in 0.1 of them. `cactus[age=15]` of height 3: no change. U15 names the black-box test.

Provenance: `source_suggested`. The wiki agrees on the height 3, the 16 random ticks, the
growth without light, the flower after "exactly 9 random ticks" with "10%" and "25%".
Kind: semantic. Confidence: high.

### GRW-CAC-002: Survival of a cactus

- **Inputs:** a shape update at a cactus at P, or the scheduled tick of the cactus at P.
- **Behavior:** a cactus can stay at P when all these conditions are true:
  - No horizontal neighbor of P holds a solid block state or a fluid of the tag
    `#minecraft:lava`.
  - The block at P + D is cactus or is in the tag `#minecraft:supports_cactus`.
  - The block at P + U does not hold a liquid.

  On a shape update, when the cactus cannot stay, the server schedules a block tick at P with
  the delay 1. On the scheduled tick, when the cactus cannot stay, the server destroys P.
- **Edge cases:** the cactus flower on top of a cactus is not solid and not a liquid. It does
  not break the cactus.
- **Conformance tests:** kernel: put stone next to a cactus. Result: one tick later P is air and
  the drop is spawned.

Provenance: `source_suggested_black_box_confirmed` for the tag: the file
`data/minecraft/tags/block/supports_cactus.json`. `source_suggested` for the predicate and the
tick. The wiki agrees ("A cactus breaks itself ... if any block with the legacy solid property,
or lava, occupies any of the 4 horizontally adjacent blocks"). Kind: semantic. Confidence: high.

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
  4. Set P + U to `bamboo[age=0,leaves=small,stage=0]`.

  The shape update of this set reaches the sapling at P from `U`. Its shape function gives
  `bamboo[age=0,leaves=none,stage=0]` when the block at P + U is bamboo. As a result, P holds
  bamboo after the chain of the set.
- **Edge cases:**
  - The light test reads P + U, the position of the new block. It reads the light, not the
    ambient light.
  - A player who puts bamboo on a bamboo sapling gets the same result through the same shape
    function.
- **Conformance tests:** kernel: a bamboo sapling in light 15 with air above, over 100 000
  ticks with the draw forced to 0. Result: P + U is `bamboo[leaves=small]` and P is
  `bamboo[leaves=none]` after the first tick. U15 names the black-box test.

Provenance: `source_suggested`. The wiki agrees on the chance 1/3 and on the light 9 at the top.
Kind: semantic. Confidence: high.

### GRW-BAM-002: The random tick of bamboo

- **Inputs:** a random tick at bamboo at P with `stage` 0.
- **Behavior:** the tick does these steps in this order:
  1. Draw `r(3)`. If the result is not 0, the tick ends.
  2. If the block at P + U is not air, the tick ends.
  3. If the light at P + U is less than 9, the tick ends.
  4. Let `h` be the height of the column of P, with a limit of 16. If `h` is 16, the tick ends.
  5. Grow one block (GRW-BAM-003).
- **Edge cases:** a bamboo with `stage` 1 does not tick randomly. It never grows again, also
  when the blocks above it are removed. Bone meal on its column is rejected (GRW-BON-004).
- **Conformance tests:** kernel: a column of 16 bamboo. Result: no tick grows it. U15 names the
  black-box test.

Provenance: `source_suggested`. The wiki agrees on the chance, the light and the height 12 to
16. Kind: semantic. Confidence: high.

### GRW-BAM-003: One block of bamboo growth

- **Inputs:** a bamboo at P that grows, with the height `h` of its column. B1 is the block at
  P + D, B2 is the block at P + 2D.
- **Behavior:** these steps occur in this order:
  1. Let `L` be `small` when B1 is not bamboo, or is bamboo with `leaves` `none`. Otherwise
     let `L` be `large`, and if B2 is bamboo, set P + D to B1 with `leaves` `small` and set
     P + 2D to B2 with `leaves` `none`.
  2. Let `A` be 1 when the state at P has `age` 1 or B2 is bamboo, and 0 otherwise.
  3. Let `S` be 1 when `h` is 15, or when `h` is 11 or more and `u` < 0.25, and 0 otherwise.
     The draw occurs only when `h` is 11 or more and is not 15. `u` is a 32-bit float.
  4. Set P + U to `bamboo` with `age` A, `leaves` L and `stage` S.
- **Numeric semantics:** a column stops at a height of 12 to 16 blocks. The stop at height 16 is
  certain. Each growth from a height of 11 to 14 stops the column with probability 0.25.
- **Edge cases:**
  - The three top blocks carry the leaves: `large` at the top, `small` below it, `none` lower.
    The second block of a column has `small` leaves and the first has `none`.
  - The thick form (`age` 1) starts at the third block of a column and stays.
- **Conformance tests:** kernel: grow a column from a sapling with the draws forced. Result: the
  leaves and the ages of the column follow Numeric semantics and Edge cases.

Provenance: `source_suggested`. The wiki agrees on the leaves of the top blocks, the thick form
and the height range. Kind: semantic. Confidence: high.

---

## 7. Saplings

A sapling is a block of the tag `#minecraft:saplings` with the property `stage` (0 or 1), except
`azalea` and `flowering_azalea`, which have no `stage` and no random tick. The block definition
of each sapling has a `minecraft:placement_filter` on the tag `#minecraft:supports_vegetation`
(`block-support.md`, SUP-VEG-001); the mangrove propagule uses `#minecraft:supports_mangrove_propagule`.

### GRW-SAP-001: The random tick of a sapling

- **Inputs:** a random tick at a sapling at P with stage `s`.
- **Behavior:** the tick does these steps in this order:
  1. If the ambient light at P + U is less than 9, the tick ends. A mangrove propagule skips
     this step.
  2. Draw `r(7)`. If the result is not 0, the tick ends.
  3. If `s` is 0, set P to the sapling with `stage` 1, with flags 260. The tick ends.
  4. Grow the tree (GRW-SAP-002).
- **Numeric semantics:** with `random_tick_speed` 3, a sapling in light gets a pick about one
  time in 1365 ticks, and it passes step 2 one time in 7 picks. A sapling needs two passes.
- **Edge cases:**
  - The test reads the ambient light, which the sky darken reduces at night. A sapling under the
    open sky and without a light source does not grow around midnight. Crops grow then
    (GRW-CRP-003).
  - The client does not see the change of `stage`. It sees the tree.
  - A mangrove propagule that hangs (`hanging` true) does not grow a tree. Its tick sets
    `age` + 1 with flags 2 while `age` is less than 4, and does nothing at `age` 4.
- **Conformance tests:** experiment E-G5.

Provenance: `source_suggested_black_box_confirmed` for the ambient light: experiment E-G2 (an
oak sapling under the open sky does not grow in 2 000 ticks at midnight, then grows at noon)
and E-G5. `source_suggested` for the chance 1/7 and the flags (U15). The wiki agrees on the
light 9 at the block above. Kind: semantic. Confidence: high.

### GRW-SAP-002: The growth of a tree

- **Inputs:** a sapling of block type B at P that grows, from GRW-SAP-001 step 4 or from a bone
  meal response. R is the random of the pick.
- **Behavior:** these steps occur in this order:
  1. If Table 7 has a row "2 × 2" for B: search a 2 × 2 square of blocks of type B that contains
     P. Test the squares with these north-west corners in this order: P, P + N, P + W,
     P + N + W. The first square whose four positions all hold B is the square. When there is a
     square, select the feature of the row "2 × 2" with R, set each of the four positions to the
     block form of its fluid with flags 818, and run the feature with R at the north-west
     corner. If the feature placed nothing, set the four positions to their saplings again with
     flags 260. These steps end the growth, also when the feature placed nothing.
  2. Select the feature of the row "no" of Table 7 with R. "Flowers near" is true when at least
     one block of the tag `#minecraft:flowers` is in the box dx ±2, dy −1..+1, dz ±2 around P.
     When the row gives no feature, nothing occurs.
  3. Set P to the block form of its fluid with flags 818. Run the feature with R at P. If the
     feature placed nothing, set P to its sapling again with flags 260.
- **Edge cases:**
  - A feature that finds no room places nothing. The sapling stays with `stage` 1. The next
    pass of GRW-SAP-001 step 2 tries again.
  - A spruce or jungle sapling that is in a square never grows its small tree. When the large
    feature finds no room, nothing grows.
  - The leaves that a tree feature places have `persistent` false and `distance` 7. The leaves
    then take their distance through GRW-LEF-001.
  - Flags 818 sync the clients and make no neighbor updates, no shape updates and no drops. The
    reset with flags 260 does not reach the clients. A client sees the sapling vanish, then the
    tree, or the sapling again when the server sends the column later.
  - A waterlogged mangrove propagule leaves a water source when it grows.
  - The room that a feature needs, and which blocks it can replace, are the rules of the
    feature. They are not in this document.
- **Conformance tests:** experiment E-G5.

Provenance: `source_suggested_black_box_confirmed` for the leaves: each tree feature file has
`"persistent": "false"` and `"distance": "7"` in its `foliage_provider`, and experiment E-G5
(no persistent leaf in any grown tree; a square grows one tree with logs at its four positions;
a sapling under a stone plate stays). `source_suggested` for the flags and the order of the
steps. The wiki agrees on the search order ("southeast, northeast, southwest and northwest")
and on the failure without room. Kind: semantic. Confidence: high. U16 records one run in which
a square of dark oak did not grow.

### GRW-SAP-003: Which tree feature a sapling grows

- **Behavior:** Table 7 gives the tree feature for each sapling. A row with weights selects one
  feature with R, with the probability of its weight over the sum of the weights of the row.

**Table 7. Tree features of the saplings.** A feature name is a key under
`data/minecraft/worldgen/feature/`.

| Sapling | Square | Feature |
|---------|--------|---------|
| `oak_sapling` | no | `oak` with weight 9, `fancy_oak` with weight 1. With flowers near: `oak_bees_005` with weight 9, `fancy_oak_bees_005` with weight 1 |
| `birch_sapling` | no | `birch`; with flowers near: `birch_bees_005` |
| `spruce_sapling` | no | `spruce` |
| `spruce_sapling` | 2 × 2 | `mega_spruce` with weight 1, `mega_pine` with weight 1 |
| `jungle_sapling` | no | `jungle_tree_no_vine` |
| `jungle_sapling` | 2 × 2 | `mega_jungle_tree` |
| `acacia_sapling` | no | `acacia` |
| `dark_oak_sapling` | no | none |
| `dark_oak_sapling` | 2 × 2 | `dark_oak` |
| `pale_oak_sapling` | no | none |
| `pale_oak_sapling` | 2 × 2 | `pale_oak_bonemeal` |
| `cherry_sapling` | no | `cherry`; with flowers near: `cherry_bees_005` |
| `poplar_sapling` | no | `red_poplar`, `orange_poplar`, `yellow_poplar`, each with weight 1 |
| `mangrove_propagule` | no | `mangrove` with weight 15, `tall_mangrove` with weight 85 |
| `azalea`, `flowering_azalea` (bone meal only) | no | `azalea_tree` |

- **Edge cases:**
  - A birch sapling never grows a tall birch. `super_birch_bees` is a feature of the generation
    only.
  - A dark oak or pale oak sapling that is not in a square grows nothing, and the pass of
    GRW-SAP-001 that reached step 4 has no effect.
- **Conformance tests:** experiment E-G5.

Provenance: `source_suggested_black_box_confirmed`: each feature of the table is a file of the
data pack, and experiment E-G5 grows the tree of each row (the logs and the leaves of its wood;
a poplar sapling gives one of the three poplars; a single dark oak sapling gives nothing). The
weights and the variants `_no_vine`, `_bonemeal` and `_bees_005` are `source_suggested`. The
wiki agrees on the trees of each sapling and on "Oak or Fancy Oak". Kind: semantic.
Confidence: high.

---

## 8. Grass block and mycelium

The grass block is `grass_block` and mycelium is `mycelium`. Both have the property `snowy`. In
this section, "the spread block" is the block type of the state at P.

### GRW-SPR-001: Can the spread block stay

- **Inputs:** a position Q and the block state A at Q + U.
- **Behavior:** the spread block *can stay* at Q when one of these conditions is true, tested
  in this order:
  1. A is `snow` with `layers` 1: can stay.
  2. A holds a fluid with `level` 8 (a source or a falling fluid): cannot stay.
  3. The light dampening into the top face is 15 or more: cannot stay. The light dampening into
     the top face is the `minecraft:light_dampening` of A, or 16 when the occlusion shape of the
     spread block and the occlusion shape of A together cover the face between Q and Q + U.
  4. Otherwise: can stay.
- **Edge cases:**
  - Snow with 2 or more layers has `minecraft:light_dampening` 15. The grass block below it
    changes to dirt.
  - Glass and leaves have a light dampening below 15 and no occlusion shape that covers the
    face. The grass block below them stays.
  - A bottom slab or a slab with its full face down, on a grass block, covers the face
    together with the grass block. The grass block changes to dirt. A top slab does not cover
    the face from below, so the grass block stays.
  - The light at Q + U does not matter here. A grass block under a torch in a closed room of
    glass stays.
  - A waterlogged block at Q + U holds a fluid with `level` 8. The grass block below it changes
    to dirt. Flowing water with `level` below 8 does not kill it.
- **Conformance tests:** experiment E-G6.

Provenance: `source_suggested_black_box_confirmed`: experiment E-G6 (dirt under stone, under a
water source, under two snow layers and under a bottom slab; the grass block stays under glass,
under leaves, under one snow layer and under a top slab). The wiki agrees on the snow layer and
on water. The wiki states "a light level of four or below" for the death; this version reads
the light dampening, not the light. Kind: semantic. Confidence: high.

### GRW-SPR-002: The random tick of the spread block

- **Inputs:** a random tick at a grass block or at mycelium at P.
- **Behavior:** the tick does these steps in this order:
  1. If the spread block cannot stay at P (GRW-SPR-001), set P to `dirt`. The tick ends.
  2. If the ambient light at P + U is less than 9, the tick ends.
  3. Do these steps four times:
     1. Draw the offset `(r(3) − 1, r(5) − 3, r(3) − 1)`. Let Q = P plus the offset.
     2. If the block at Q is not `dirt`, continue with the next repetition.
     3. If the spread block cannot stay at Q, or Q + U holds a fluid of the tag
        `#minecraft:water` of any level, continue with the next repetition.
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

Provenance: `source_suggested_black_box_confirmed` for the tags: the files
`data/minecraft/tags/block/snow.json` and `data/minecraft/tags/fluid/water.json`.
`source_suggested_black_box_confirmed` for the targets and the spread of mycelium: experiment
E-G6 (dirt becomes a grass block, coarse dirt and rooted dirt stay, dirt in a ring of mycelium
becomes mycelium, mycelium under stone becomes dirt). `source_suggested` for the four draws and
the ambient light (U15). The wiki agrees on the range, the four draws, the light 9 above the
source and the targets. Kind: semantic. Confidence: high.

---

## 9. Leaves

Leaves are the blocks of the tag `#minecraft:leaves`, with the properties `distance` (1 to 7),
`persistent` and `waterlogged`. A log is a block of the tag `#minecraft:prevents_nearby_leaf_decay`,
which resolves to `#minecraft:logs` in the vanilla data pack.

### GRW-LEF-001: The distance of a leaf block

- **Inputs:** a shape update at leaves at P from the direction d, or the scheduled tick of the
  leaves at P.
- **Behavior:**
  - The *distance from* a block state N is 0 when N is a log, the `distance` of N when N has
    that property, and 7 otherwise.
  - On a shape update from d: let `k` be the distance from the state at P + d, plus 1. If `k` is
    not the `distance` at P, schedule a block tick at P with the delay 1 (SIM-SCH-003). The
    shape function does not change the state.
  - On the scheduled tick: let `m` be the smallest of 7 and, for each of the six neighbors, the
    distance from it plus 1. If `m` is not the `distance` at P, set P to the leaves with
    `distance` `m`, with flags 3.
- **Edge cases:**
  - Persistent leaves update their distance as other leaves do. They never decay.
  - The change reaches the clients one tick after the change of the neighbor. An observer that
    faces the leaves pulses on the tick.
  - A chain of leaves updates one block for each tick, up to 6 blocks from the log. The seventh
    leaf block keeps `distance` 7 and decays.
  - A shape update from a neighbor whose distance from is 7 gives `k` = 8. Leaves with
    `distance` 7 then schedule a tick that changes nothing, when the other neighbors give 7 too.
  - Leaves that a player places have `persistent` true and the distance of GRW-LEF-001 at the
    moment of the placement.
- **Conformance tests:** experiment E-G7.

Provenance: `source_suggested_black_box_confirmed` for the tag: the file
`data/minecraft/tags/block/prevents_nearby_leaf_decay.json`, and experiment E-G7 (a line of
eight leaf blocks from a log has the distances 1 to 6 after 20 ticks, and the two last keep 7).
`source_suggested` for the exact condition that schedules the tick. The wiki agrees on the
distance up to 6, on the one tick of delay and on the persistent leaves. Kind: semantic.
Confidence: high.

### GRW-LEF-002: The random tick of leaves

- **Inputs:** a random tick at leaves at P with `distance` 7 and `persistent` false.
- **Behavior:** the server spawns the drops of the state at P (`block-loot.md`), then removes P.
  Waterlogged leaves leave a water source (SIM-SET-004). The server sends no level event 2001
  for the decay.
- **Edge cases:**
  - Leaves with `distance` below 7 do not tick randomly (GRW-TBL-001). A leaf block whose log
    was removed decays only after GRW-LEF-001 set its distance to 7.
  - Without the level event, a client shows no break particles and plays no break sound for a
    decay. The item drops are the only visible effect.
- **Conformance tests:** experiment E-G7.

Provenance: `source_suggested_black_box_confirmed`: experiment E-G7 (20 lone leaf blocks with
`distance` 7 are all gone after 2 000 ticks; the two leaf blocks past the sixth of a line are
gone, and the six others stay). `source_suggested` for the absence of the level event. The wiki
agrees on the decay and on the drops. Kind: semantic. Confidence: high.

---

## 10. Ice and snow

Ice is `ice`, with one state and `minecraft:light_dampening` 1. Frosted ice is `frosted_ice`,
with the property `age` 0 to 3 and `minecraft:light_dampening` 1. The snow layer is `snow`, with
the property `layers` 1 to 8 and `minecraft:light_dampening` 0.

### GRW-ICE-001: The random tick of ice

- **Inputs:** a random tick at ice at P.
- **Behavior:** if the block light at P is more than `11 − 1`, that is, 11 or more, the ice
  melts: the server sends the game event `block_destroy` at P; then, when the environment
  attribute `gameplay/water_evaporates` is true at P, it removes P; otherwise it sets P to a
  water source and makes a neighbor update at P for the water.
- **Edge cases:**
  - The sky light does not melt ice. Ice under the sun stays.
  - Ice melts also when the block below is air.
  - The threshold is the block light at P itself, not at a neighbor. The light dampening of the
    ice (1) is the reason for the threshold 11 and not 12.
  - In the nether, `gameplay/water_evaporates` is true, and the melt leaves air.
- **Conformance tests:** experiment E-G7.

Provenance: `source_suggested_black_box_confirmed` for the dampening: `ice.json` has
`minecraft:light_dampening` 1, and experiment E-G7 (ice with a torch at the Manhattan distance 3,
block light 11, is water after 2 000 ticks; ice with a torch at the distance 4, block light 10,
stays; ice under the sun stays). `source_suggested` for the melt in a dimension where water
evaporates. The wiki states "higher than 11" and agrees on the water, the nether and the block
below. Kind: semantic. Confidence: high.

### GRW-ICE-002: Frosted ice

- **Inputs:** the placement side effect of frosted ice at P, a scheduled tick at frosted ice at
  P with age `a`, or a neighbor update at frosted ice at P from a frosted ice block. Frosted ice
  does not tick randomly (GRW-TBL-001).
- **Behavior:**
  - The placement side effect schedules a block tick at P with the delay `60 + r(61)`, that is,
    60 to 120, from the dimension random.
  - The *brightness* at P is the block light at P in the dimension `minecraft:the_end`, and the
    ambient light at P in each other dimension.
  - *Crack* at P: when `a` < 3, set P to frosted ice with age `a + 1`, with flags 2, play the
    cracking sound at P, and send the game event `block_change`. When `a` is 3, melt P as
    GRW-ICE-001 does.
  - The scheduled tick does these steps:
    1. Let `c` be the count of frosted ice among the six neighbors of P. If `r(3)` is not 0 and
       `c` is 4 or more, go to step 4. The draw occurs before the count.
    2. If the brightness at P is not more than `11 − a − 1`, go to step 4.
    3. Crack at P. If P melted: for each neighbor of P that is frosted ice, crack at it; when
       that neighbor did not melt, schedule a block tick at it with the delay `20 + r(21)`. The
       tick ends.
    4. Schedule a block tick at P with the delay `20 + r(21)`.
  - On a neighbor update from a frosted ice block, when fewer than 2 of the six neighbors of P
    are frosted ice, melt P as GRW-ICE-001 does.
- **Edge cases:**
  - Frosted ice in darkness never melts by itself. The thresholds are 10, 9, 8 and 7 for the
    ages 0 to 3.
  - `random_tick_speed` does not change the delays.
  - Nothing makes frosted ice in this version: the frost walker enchantment is not in scope.
- **Conformance tests:** none against vanilla in this document (U14).

Provenance: `source_suggested`. The wiki states "ticked every 1–2 seconds" and the condition
"greater than 11 minus the age"; this version has the first delay 60 to 120 and the dampening
in the threshold. Kind: semantic. Confidence: medium.

### GRW-ICE-003: The random tick of a snow layer

- **Inputs:** a random tick at `snow` at P.
- **Behavior:** if the block light at P is more than 11, the server spawns the drops of the
  state at P (`block-loot.md`), removes P, and sends the game event `block_destroy` at P. All
  layers go at one time.
- **Edge cases:**
  - The sky light does not melt snow.
  - The threshold is 12, because the rule does not read the light dampening of the snow.
  - The drops of the table of `snow` without a tool are nothing.
- **Conformance tests:** experiment E-G7.

Provenance: `source_suggested_black_box_confirmed`: experiment E-G7 (five layers with a torch at
the distance 2, block light 12, are gone; five layers with a torch at the distance 3, block
light 11, stay). The wiki agrees ("block light level of 12 or more", "they melt all at once").
Kind: semantic. Confidence: high.

---

## 11. Farmland

Farmland is `farmland`, with the property `moisture` 0 to 7.

### GRW-FRM-001: Water near farmland

- **Inputs:** a farmland at P.
- **Behavior:** farmland at P *has water* when at least one position of the box dx ±4,
  dy 0..+1, dz ±4 around P holds a fluid of the tag `#minecraft:water`, of any level. A
  waterlogged block counts.
- **Edge cases:** the box has 9 × 2 × 9 = 162 positions. Blocks between the water and the
  farmland do not matter.

Provenance: `source_suggested_black_box_confirmed`: experiment E-G8 (farmland at the distances
1 to 4 from a water source at its level or one level below it is moist, farmland at the distance
5 is dry, and farmland one level above the water is dry). The wiki agrees ("up to four blocks
away horizontally (including diagonally)", "at the same level or one block above").
Kind: semantic. Confidence: high.

### GRW-FRM-002: The random tick of farmland

- **Inputs:** a random tick at farmland at P with moisture `m`.
- **Behavior:** the tick does these steps in this order:
  1. If the farmland has water (GRW-FRM-001), or there is rain at P + U: if `m` < 7, set P to
     farmland with `moisture` 7, with flags 2. The tick ends.
  2. If `m` > 0, set P to farmland with `moisture` `m − 1`, with flags 2. The tick ends.
  3. If the block at P + U is in the tag `#minecraft:maintains_farmland`, the tick ends.
  4. Turn P to dirt (GRW-FRM-003).
- **Edge cases:**
  - Farmland with a crop on it dries to `moisture` 0 and stays farmland.
  - The moisture jumps from any value to 7 in one tick. It goes down one level in each tick.
  - A fence gate above farmland keeps it farmland; the tag names `#minecraft:fence_gates`.
  - A new farmland from a hoe has `moisture` 0. Its first wet tick sets 7.
  - The change to dirt from a trample or from a piston is not a random tick. Those rules are
    not in this document.
- **Conformance tests:** experiment E-G8.

Provenance: `source_suggested_black_box_confirmed` for the tag: the file
`data/minecraft/tags/block/maintains_farmland.json`, and experiment E-G8 (dry farmland with
`moisture` 7 and no crop is dirt after 400 ticks, and a grass block takes that dirt; dry farmland
with wheat has `moisture` 0 and keeps the wheat; farmland near water has `moisture` 7).
`source_suggested` for the rain and the flags. The wiki agrees on the count down, the jump to 7
and the dirt without a crop. Kind: semantic. Confidence: high.

### GRW-FRM-003: Farmland under a solid block, and the turn to dirt

- **Inputs:** a shape update at farmland at P from `U`, or the scheduled tick of the farmland at
  P, or step 4 of GRW-FRM-002.
- **Behavior:**
  - Farmland can stay at P when the block at P + U is not solid, or is in the tag
    `#minecraft:maintains_farmland`.
  - On a shape update from `U`, when the farmland cannot stay, the server schedules a block
    tick at P with the delay 1. On the scheduled tick, when the farmland cannot stay, it turns P
    to dirt.
  - *Turn P to dirt*: the server sets P to `dirt` with flags 3, and sends the game event
    `block_change` at P. Before the set, the entities whose box intersects the block above the
    farmland shape move up to stand on the dirt (farmland is 15/16 of a block high).
- **Edge cases:** a pumpkin or a melon that a stem grows on farmland is solid and not in the tag.
  The farmland under it turns to dirt one tick later.
- **Conformance tests:** kernel: put stone on farmland. Result: one tick later P is dirt.

Provenance: `source_suggested`. The wiki agrees ("solid block covering top"). Kind: semantic.
Confidence: high.

---

## 12. Bone meal responses

The bone meal item is `minecraft:bone_meal`. When a player or a dispenser uses it on a block at
P, the block gives one of two results: it accepts the bone meal, or it rejects it. On an accept,
the item logic takes one bone meal in survival and asks the clients for the growth particles at
P. On a reject, the use does nothing: the item stays, and the player can place a block. These
item rules are not in this document.

A response has two parts. The **test** decides whether the block is a target. The **draw**, when
the rule has one, decides whether this use changes the world. A block that passes the test
accepts the bone meal also when it fails the draw: the item is consumed and the particles play,
and the world does not change. The draw comes from the dimension random.

### GRW-BON-001: Crops and stems

- **Inputs:** bone meal on a crop or a stem at P with age `a`.
- **Behavior:**
  - Test: `a` is less than the last age.
  - Response: draw `2 + r(4)`, that is, 2 to 5. For `beetroots`, divide the result by 3 with
    integer division, that is, 0 with probability 1/4 and 1 otherwise. For `torchflower_crop`,
    the increase is 1. Set P to the crop or stem with age `min(a + increase, last age)`, with
    flags 2. For `torchflower_crop` at the last age, GRW-CRP-004 applies.
  - For a stem whose new age is 7, the server then runs the random tick of the stem at P
    (GRW-STM-001) at once, with the dimension random.
- **Edge cases:**
  - A stem at `age` 7 rejects the bone meal. The bone meal that brings a stem to `age` 7 can
    make a fruit in the same use, through the random tick that follows.
  - A beetroot that draws 0 accepts the bone meal and does not change.
- **Conformance tests:** kernel: bone meal on `wheat[age=0]` over 100 000 uses. Result: the ages
  2 to 5 each in 1/4 of the uses. U15 names the black-box test.

Provenance: `source_suggested`. The wiki agrees on "2–5 growth stages", "75% chance" for
beetroots and "Grows every stage" for the torchflower crop; it states that bone meal "does not
help produce the actual melons", which the random tick after the last stage contradicts.
Kind: semantic. Confidence: high.

### GRW-BON-002: Saplings and azaleas

- **Inputs:** bone meal on a sapling, a mangrove propagule, an azalea or a flowering azalea at P.
- **Behavior:**
  - Sapling. Test: when Table 7 has only a row "2 × 2" for the sapling, P is in a square
    (GRW-SAP-002 step 1); and the position `base_height` blocks above P is inside the build
    height, where `base_height` is the field of the `trunk_placer` of the first feature of
    the row "no" of Table 7. Draw: `u` < 0.45. Response: GRW-SAP-001 steps 3 and 4, without the
    light test.
  - Mangrove propagule that hangs. Test: `age` < 4. No draw. Response: set P to the propagule
    with `age` + 1, with flags 2.
  - Azalea and flowering azalea. Test: the position 6 blocks above P is inside the build
    height, and P + U holds no fluid. Draw: `u` < 0.45. Response: GRW-SAP-002 steps 2 and 3
    with the feature `azalea_tree`.
- **Edge cases:**
  - Bone meal grows a sapling in darkness.
  - A dark oak sapling or a pale oak sapling that is not in a square rejects the bone meal. The
    item stays.
  - A sapling whose tree finds no room accepts the bone meal and does not change.
- **Conformance tests:** kernel: bone meal on an oak sapling with `stage` 0 over 100 000 uses.
  Result: 0.45 of them give `stage` 1. U15 names the black-box test.

Provenance: `source_suggested`. The wiki agrees on "45% chance", on the growth "even without
sufficient light" and on the waste of bone meal on an obstructed sapling; it states that a dark
oak sapling outside a square wastes the bone meal, which the test contradicts. Kind: semantic.
Confidence: high.

### GRW-BON-003: Grass block

- **Inputs:** bone meal on a grass block at P.
- **Behavior:**
  - Test: the block at P + U is air, and P + U is inside the build height.
  - Response: do 128 attempts. Attempt number `i`, from 0 to 127, does these steps:
    1. Let Q = P + U. Do `floor(i / 16)` steps: draw `dx` and `dz` uniform in −1..1, draw `dy`
       as a uniform value in −1..1 times `r(3)`, with integer division by 2, and add
       `(dx, dy, dz)` to Q. After each step, if the block at Q + D is not the grass block, or the
       block at Q has a full collision box, end this attempt.
    2. If the block at Q is `short_grass`, draw `u`. If `u` < 0.1 and the short grass accepts
       bone meal there, run its response at Q (it becomes `tall_grass`).
    3. If the block at Q is air and Q is inside the build height, draw `u`. If `u` < 0.125:
       select with the dimension random one feature of the *bone meal features* of the biome
       at Q, and place that configured feature at Q; when the biome has none, this attempt
       ends. Otherwise place the placed feature `minecraft:grass_bonemeal` at Q.

  The *bone meal features* of a biome are the configured features that its placed features
  name, in the order of the biome, that are in the feature tag
  `#minecraft:can_spawn_from_bone_meal`.
- **Numeric semantics:** the attempts reach at most 7 steps from P + U, that is, the box
  dx ±7, dy ±7, dz ±7. The vertical step is 0 with probability 2/3. The features of the tag are
  `simple_block` features that place one flower at Q when the flower can stay there.
- **Edge cases:**
  - The response accepts the bone meal also when all 128 attempts place nothing.
  - Mycelium has no response. It rejects bone meal.
  - A two-block flower never comes from this response: no feature of the tag places one.
  - The placed feature `grass_bonemeal` places `short_grass` where the block is air and the
    short grass can stay.
  - The feature of a flower is placed as a configured feature: the `count` and the `offset` of
    its placed feature do not apply. One use places at most one block for each attempt.
- **Conformance tests:** kernel: bone meal on a grass block in plains, over 10 000 uses. Result:
  each new block is within dx ±7, dz ±7 of P; the flowers are poppies, dandelions and the
  flowers of `flower_plain`, and 1/8 of the placed blocks are flowers (binomial tolerance). U15
  names the black-box test.

Provenance: `source_suggested_black_box_confirmed` for the data: the files
`data/minecraft/worldgen/placed_feature/grass_bonemeal.json` (the feature `minecraft:grass`
places `minecraft:short_grass` where the block is air) and
`data/minecraft/tags/worldgen/feature/can_spawn_from_bone_meal.json` (`flower_default`,
`flower_flower_forest`, `flower_swamp`, `flower_plain`, `flower_meadow`, `flower_cherry`,
`wildflower`, `flower_pale_garden`). `source_suggested` for the 128 attempts, the walk and the
draws. The wiki agrees on the "15×15×15" area and the plants. Kind: semantic. Confidence: high.

### GRW-BON-004: Bamboo

- **Inputs:** bone meal on a bamboo sapling or on bamboo at P.
- **Behavior:**
  - Bamboo sapling. Test: the block at P + U is air, and P + U is inside the build height. No
    draw. Response: GRW-BAM-001 step 4.
  - Bamboo. Let T be the top bamboo of the column of P, and `h` the height of the column, with
    each count limited to 16. Test: `h` < 16, the state at T has `stage` 0, and T + U is air and
    inside the build height. No draw. Response: let `k` be `1 + r(2)`. Do `k` times: if `h` < 16,
    the state at T has `stage` 0, and T + U is air and inside the build height, grow one block
    at T (GRW-BAM-003) with the height `h`, then let T be T + U and `h` be `h + 1`; otherwise
    the response ends.
- **Edge cases:**
  - Bone meal on a bamboo in the middle of a column grows the top of the column.
  - A column whose top has `stage` 1 rejects the bone meal.
  - The second block of the response can stop the column (GRW-BAM-003 step 3).
- **Conformance tests:** kernel: bone meal on the base of a column of 3 over 100 000 uses.
  Result: the column grows 1 block in half of the uses and 2 in the other half.

Provenance: `source_suggested`. The wiki agrees ("Grows the bamboo by 1–2 stems", "Transforms a
bamboo shoot into a 2-block tall bamboo"). Kind: semantic. Confidence: high.

### GRW-BON-005: Blocks that reject bone meal

- **Behavior:** `sugar_cane`, `cactus`, `cactus_flower`, `attached_melon_stem`,
  `attached_pumpkin_stem`, a crop or a stem at its last age, a dark oak or pale oak sapling
  that is not in a square, a bamboo column whose top has `stage` 1 or is at height 16,
  `mycelium`, `farmland`, `ice`, `snow`, the leaves, and each block that no rule of this
  document or of a later document gives a response, reject bone meal. The item stays.
- **Conformance tests:** kernel: bone meal on stone. Result: no change, and the response is a
  reject.

Provenance: `source_suggested`. The wiki agrees ("Bone meal does not affect: Cactus, Vines,
Nether wart, Sugar cane (Java only), Chorus plants, or other plants not listed").
Kind: semantic. Confidence: high.

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
| GRW-CAN-002 (tick) | ±1 | −1..0 | ±1 | P | item drops |
| GRW-CAC-001 | ±1 | −n..+2 | ±1 | P and P + U | a neighbor update at P + U |
| GRW-CAC-002 (tick) | ±1 | ±1 | ±1 | P | item drops |
| GRW-BAM-001 | 0 | 0..+1 | 0 | P + U | none |
| GRW-BAM-002, GRW-BAM-003 | 0 | −16..+1 | 0 | P + U, P + D, P + 2D | none |
| GRW-SAP-001, GRW-SAP-002 | ±10 | −13..+34 | ±10 | the blocks of one tree feature, and the saplings of the square | none. Note 1 of SIM-MODEL-009: the bound holds for the vanilla features |
| GRW-SPR-002 | ±1 | −3..+2 | ±1 | P, or up to 4 dirt blocks in the box | none |
| GRW-LEF-001 (tick) | ±1 | ±1 | ±1 | P | none |
| GRW-LEF-002 | 0 | 0 | 0 | P | item drops |
| GRW-ICE-001 | 0 | 0 | 0 | P | a neighbor update at P; a game event |
| GRW-ICE-002 (tick) | ±1 | ±1 | ±1 | P and its six neighbors | ticks at the neighbors; a sound; game events |
| GRW-ICE-003 | 0 | 0 | 0 | P | item drops; a game event |
| GRW-FRM-002 | ±4 | 0..+1 | ±4 | P | the rain at P + U; the entities on P move up; a game event |
| GRW-FRM-003 (tick) | 0 | 0..+1 | 0 | P | the entities on P move up; a game event |
| GRW-BON-001 | ±1 | −1..0 | ±1 | P, and one horizontal neighbor for a stem | none |
| GRW-BON-002 | as GRW-SAP-002 | | | | |
| GRW-BON-003 | ±7 | ±7 | ±7 | up to 128 blocks in the box | none |
| GRW-BON-004 | 0 | −16..+18 | 0 | up to 2 new blocks and 2 leaf changes | none |

- **Edge cases:** a tree feature that a data pack adds can reach past the bound of GRW-SAP-002.
  An instance that cannot measure the extent of a feature must treat the handler as a handler
  with no bound (SIM-MODEL-009, Note 1).

Provenance: `derived` from the rules of this document, and consistent with SIM-MODEL-009,
Table 1. GRW-CAC-001 adds +2 for the flower test. GRW-BON-004 reads up to 16 blocks above P.
Kind: semantic. Confidence: high.

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

Each item names the rules that it affects and what decides it.

- **U1: The condition of each randomly ticking state, and its data form** (GRW-TBL-001,
  GRW-TBL-002). The block definitions carry no flag. The conditions of the "later" row of
  Table 1 are not stated. A possible source is the update tool that writes the block
  definitions: it can write the property for each block state from the game, into the
  definitions or into a report under `assets/mcrs/reports/`. The owner decides the source.
  Until then, Table 1 is the source, and a data pack cannot add a randomly ticking block.
- **U3: Crops on a base that is not farmland** (GRW-CRP-002). A crop cannot stay there
  (GRW-CRP-001), so only a command makes the case. Decide by policy: the formula gives `g` from
  1.
- **U5: The data form of the survival predicates** (GRW-CAN-002, GRW-CAC-002, GRW-FRM-003,
  GRW-STM-002). The predicates read P + D plus a horizontal step, and a fluid tag. The data form
  of `block-support.md` (SUP-PRED-002) allows only offsets of one step and no fluid tag.
  Question to the author of `block-support.md`: extend the form, or keep these predicates as
  rules of this document.
- **U14: Frosted ice** (GRW-ICE-002). No experiment is possible at this version: nothing makes
  frosted ice. The rule stays `source_suggested`.
- **U15: Experiments that have not run.** The rates of the rules that E-G2 to E-G8 confirm are
  not measured: the 1/7 of a sapling, the 1/3 of bamboo, the four draws of the spread block.
  These rules have a kernel test and no black-box result at all: GRW-CRP-002 and GRW-CRP-003 (the distribution of the growth time of a lone wheat,
  of a 9 × 9 field, of a dry wheat and of a diagonal pair, over 20 runs of 20 000 ticks at
  `random_tick_speed` 3, with beetroots and the torchflower crop); GRW-STM-001 (the side of the
  fruit over 200 stems); GRW-CAN-001, GRW-CAC-001, GRW-BAM-001 to GRW-BAM-003 (the heights,
  the flower and the leaves over 100 columns); GRW-BON-001 to GRW-BON-004 (the histogram of 200
  dispensers of bone meal on wheat, beetroots, a sapling, a grass block in plains and bamboo,
  and the count of bone meal that stays). Each of these gives a fixture for the exhaustive tier
  of the growth tests when it runs.
- **U16: A square of dark oak that did not grow** (GRW-SAP-002). In experiment E-G5, the square
  of dark oak saplings with the north-west corner (0, −60, 760) did not grow in 405 ticks at
  `random_tick_speed` 1000, while the square of pale oak with the same feature parameters grew.
  In experiment E-G5b, three squares of dark oak at other positions grew in 1 507 ticks, and the
  feature `minecraft:dark_oak` places at a free position. The cause of the first result is not
  known. A repeat of E-G5 at the same position decides whether the position or the run is the
  cause.

---

## 16. Experiments

Each experiment runs on a vanilla dedicated server of the version of
`assets/minecraft/version.json`, with `online-mode=false`, `level-type=minecraft:flat`,
`pause-when-empty-seconds=-1` and `spawn-protection=0`. The commands go to the console. The
default flat world has a grass block at y −61 and air above. Each case stands in its own area,
force-loaded with `forceload add`. `tick freeze` stops the ticks before the scene is built, and
`tick step <n>` runs exactly `n` ticks. `gamerule random_tick_speed <k>` sets the rate. The
result of a step is read with `execute if block <P> <state>`, which prints "Test passed" or
"Test failed" to the console.

A dark room is a box of stone with a roof of two blocks, built so that the sky light inside is 0.
The block light at a position inside comes from a torch at a known Manhattan distance: a torch
gives 14, and the light falls by 1 for each block.

A fixture holds the input (the `setblock` commands, `k`, the number of ticks) and the
observation: a list of `(position, state)` after the ticks, and for a tree a list of
`(dx, dy, dz, block)` relative to the sapling. The fixtures of this section are under
`crates/mcrs_minecraft_simulation/tests/fixtures/vanilla/growth/`.

Each experiment below ran one time. The fixture gives the commands of the scene, the
`random_tick_speed` and the number of ticks that ran, and the state of each position that the
experiment reads.

- **E-G2** (the light of a crop). Build a dark room. Put wheat with `age` 0 on moist farmland at
  four places, with a torch at the Manhattan distance 5, 6, 7, and with no torch. Put wheat on
  moist farmland and an oak sapling under the open sky. Set the time to midnight. Run 2 020
  ticks at `random_tick_speed` 300. Then put stone next to the wheat of the three dark places.
  Then set the time to noon and run 412 ticks at `random_tick_speed` 1000. Result: the wheat at
  light 9 has `age` 7. The wheat at light 8 has `age` 0, and stays after the stone. The wheat at
  light 7 and the wheat in darkness have `age` 0, and are air after the stone. The wheat under
  the open sky has `age` 7 at midnight. The sapling is a sapling at midnight, and a log after
  the run at noon. Fixture: `crates/mcrs_minecraft_simulation/tests/fixtures/vanilla/growth/e-g2.json`.
- **E-G5** (the saplings). Under the open sky at noon, put one sapling of each type of Table 7,
  a square of dark oak, of pale oak, of spruce and of jungle, and an oak sapling under a plate
  of stone at P + 3U. Run 405 ticks at `random_tick_speed` 1000. Count the logs and the leaves in
  the box dx ±12, dy −1..+40, dz ±12 of each sapling. Result: each sapling of a row "no" of
  Table 7 is a log of its wood, with leaves of its wood and no persistent leaf. The poplar
  sapling gives orange poplar leaves. The single dark oak sapling stays. The squares of pale oak,
  spruce and jungle are logs at their four positions, with 43, 81 and 79 logs. The square of
  dark oak stays (U16). The sapling under the plate stays. Fixture: `e-g5.json` in the same
  directory.
- **E-G5b** (the dark oak square). Put three squares of dark oak saplings at three positions and
  a square of pale oak. Place the feature `minecraft:dark_oak` with the command `place feature`
  at a free position. Run 1 507 ticks at `random_tick_speed` 1000. Result: the three squares of
  dark oak are logs at their four positions, with 38 to 40 logs each, as the square of pale oak.
  The command places the feature. Fixture: `e-g5b.json`.
- **E-G6** (the spread block). Under the open sky at noon, put dirt, coarse dirt and rooted dirt
  in the grass surface. Put stone, glass, a water source in a cell of glass, one snow layer, two
  snow layers, a bottom slab, a top slab and persistent leaves on grass blocks. Put a ring of
  mycelium around one dirt, and mycelium under stone. Run 2 016 ticks at `random_tick_speed`
  300. Result: the dirt is a grass block; the coarse dirt and the rooted dirt stay. The grass
  under stone, under the water, under two snow layers and under the bottom slab is dirt. The
  grass under glass, under one snow layer, under the top slab and under the leaves stays. The
  dirt in the ring is mycelium. The mycelium under stone is dirt. Fixture: `e-g6.json`.
- **E-G7** (leaves, ice, snow). Put 20 lone oak leaves with `distance` 7. Put an oak log with
  a line of eight oak leaves with `distance` 7 to its east. Put ice with a torch at the Manhattan
  distance 3, ice with a torch at the distance 4, five snow layers with a torch at the distance
  2, five layers with a torch at the distance 3, and ice with no torch. Run 20 ticks at
  `random_tick_speed` 0, then 2 007 ticks at `random_tick_speed` 300. Result: after the 20 ticks
  the line has the distances 1 to 6, 7 and 7. After the run, none of the 20 lone leaves remains;
  the line has the distances 1 to 6, and its two last blocks are air. The ice at light 11 is
  water, the ice at light 10 is ice, the snow at light 12 is air, the snow at light 11 stays, and
  the ice under the sun is ice. Fixture: `e-g7.json`.
- **E-G8** (farmland). Put a water source in the surface with farmland and wheat at the
  distances 1 to 5 east of it, at its level. Put a second source with a glass block east of it,
  and farmland one level below the surface at the distances 2 to 6, each in a hole with wheat;
  run 2 ticks before the wheat, so that the light settles. Put a third source with farmland one
  level above the surface at the distances 1 to 4. Put farmland with `moisture` 7 and no water
  near it, one with wheat and one without. Run 421 ticks at `random_tick_speed` 300. Result: at
  the level of the water, the farmland at the distances 1 to 4 has `moisture` 7 and the one at 5
  has `moisture` 0. One level below the water, the farmland at the distances 2 to 4 has
  `moisture` 7 and the ones at 5 and 6 have `moisture` 0; all the wheat in the holes stays. One
  level above the water, all the farmland has `moisture` 0. The dry farmland without wheat is a
  grass block, and the one with wheat has `moisture` 0 and keeps the wheat. Fixture:
  `e-g8.json`.
