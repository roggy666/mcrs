# Spawn eggs: behavioural specification

| Field | Value |
|-------|-------|
| Version | see `assets/minecraft/version.json` |

This document states what the server does when a player uses a spawn egg on a block. It states
only what a client, the server console and the server files show.

Scope: a spawn egg used on a block that no other use handler takes. The egg on a spawner, the egg
on a mob of its type, the egg on a fluid, the extra tags of the item component and the natural
spawn rules are out of scope (section 5).

## 0. Conventions

### 0.1 Terms

- **Clicked position**, **face**, **adjacent position**, **held item**, **sequence**,
  **acknowledgement**, **open teleport**: as `block-placing.md`, section 0.1, defines them.
- **Spawn egg**: an item whose `minecraft:entity_data` component names a mob type in its `id`.
- **Mob type**: an entity type that has mob data (a brain row, a width and a height).
- **Target position**: the position whose block the mob stands in after the use (EGG-USE-002).
- **First spawn setup**: the setup of `mob-animals.md` ANIM-SPAWN-001 (variant, baby chance,
  follow range bonus, handedness).

### 0.2 Provenance

Each rule carries a label: `documentation` when the public documentation of the game states it,
`design` when the owner decided it for this server, `derived` when it follows from another
specification.

## 1. The use

**EGG-USE-001** (`documentation`, semantic). A `use_item_on` packet whose held item is a spawn
egg spawns one mob of the type that the `id` of the component names. The server refuses the use
when one of these conditions is true:

- the sequence is negative;
- the player is a spectator;
- the player has an open teleport;
- the clicked position is out of reach (`block-breaking.md`, the reach of the player);
- another use handler consumed the request in the same tick (EGG-ORD-001);
- the type is not a mob type (EGG-USE-004).

A refusal spawns nothing and changes nothing. The client then gets the acknowledgement as for
each `use_item_on` packet (`block-placing.md` PLC-SEQ).

**EGG-USE-002** (`documentation`, semantic). The target position is the adjacent position. When
the block at the clicked position is replaceable (`block-placing.md`, the replaceable flag), the
target position is the clicked position. The mob spawns with its feet at the bottom of the target
block, at the horizontal centre of the block: `x + 0.5`, `y`, `z + 0.5`.

**EGG-USE-003** (`documentation`, semantic). In creative mode the egg stays in the stack. In
survival and adventure mode one egg leaves the stack. The player swings the hand that holds the
egg, with the interact animation of the item (`arm-swing.md`).

**EGG-USE-004** (`design`, semantic). A type without mob data refuses the use (EGG-USE-001). The
server has no row for a boat, a minecart, an item frame or an armor stand yet; these are hooks
of section 5.

**EGG-USE-005** (`derived`, semantic). The spawned mob gets the first spawn setup, as
`mob-animals.md` ANIM-SPAWN-001 lists "a spawn egg that a player uses on a block" among the ways
of birth that apply it. Its UUID comes from the birth key of the target position as a block
source, with the sequence of the births of that source in the tick, so that two uses on one
block in one tick give two mobs.

## 2. Order

**EGG-ORD-001** (`design`, semantic). The spawn egg handler runs after the block use handlers
(a lever, a button, a door) and before the block placing. A request that a block use handler
consumed spawns nothing. A request that the spawn egg handler consumed places no block and sends
no refresh.

## 3. Observable result

- One mob entity appears at the target position in the tick after the use; its spawn packet
  reaches each viewer of the column (`entity-tracking.md`).
- The player gets the acknowledgement of the sequence.
- In survival mode the slot of the egg loses one item.

## 4. Experiments

- **E1** (`documentation`). A creative player uses a pig spawn egg on the top face of a stone
  block at (10, 64, 8). A pig stands at (10.5, 65.0, 8.5) two ticks later. The stack keeps 64
  eggs. Test: `egg_use_001_a_spawn_egg_used_on_a_block_spawns_its_mob_on_the_face`.

## 5. Unresolved

- **U1**: the egg on a spawner changes the spawn type of the spawner. Hook.
- **U2**: the egg on a mob of its type spawns a baby that follows the parent for a while. Hook.
- **U3**: the egg used in water or on a waterlogged block. Hook.
- **U4**: the other tags of `minecraft:entity_data` (a custom name, the baby flag) go to the
  spawned mob. Hook.
- **U5**: the y offset of a mob whose box does not fit the target block (the reference lifts
  the mob out of the block below). Not stated.
