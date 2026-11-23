# Server settings

Every setting of the server has a name in the style of a Paper configuration key and an
environment variable of the same meaning: the key `a.b.c-d` is the variable `MCRS_A_B_C_D`.
`src/main.rs` reads the variables into `ServerSettings` (`mcrs_minecraft_server_settings`), hands
each value to the plugin that owns it, and the plugin puts it into a resource of the host or of the
dimension; the hot path reads the resource, never the environment (rule H.9 of the horde plans).

| Key | Variable | Type, default | Meaning |
|---|---|---|---|
| `server.port` | `MCRS_PORT` | 1–65535; the network plugin's default | Listen port. |
| `server.compute-threads` | `MCRS_COMPUTE_THREADS` | positive count; all CPUs | Threads of the Bevy compute pool. |
| `world.seed` | `MCRS_WORLD_SEED` | i64 or u64; 0 | World seed. |
| `world.preset` | `MCRS_WORLD_PRESET` | preset name; `normal` | World preset (`flat`, …). |
| `players.default-game-mode` | `MCRS_DEFAULT_GAMEMODE` | game mode; `creative` | Game mode of a new player. |
| `mobs.ai` | `MCRS_MOB_AI` | flag; on | Whether the mob brains run. |
| `network.lan-announce` | `MCRS_LAN_ANNOUNCE` | flag; on | LAN announcement. |
| `network.compression-threshold` | `MCRS_COMPRESSION_THRESHOLD` | whole number; none | Packet compression threshold (-1 off). |
| `network.accept-transfers` | `MCRS_ACCEPT_TRANSFERS` | flag; off | Accept transfer packets. |
| `network.enable-status` | `MCRS_ENABLE_STATUS` | flag; on | Answer status requests. |
| `network.hide-online-players` | `MCRS_HIDE_ONLINE_PLAYERS` | flag; off | Hide the player list in the status. |
| `network.forwarding` | `MCRS_FORWARDING` | flag; off | Modern proxy forwarding. |
| `network.forwarding-secret` | `MCRS_FORWARDING_SECRET` | string; empty | Secret of the forwarding. |
| `network.proxy-upstreams` | `MCRS_PROXY_UPSTREAMS` | IP list; empty | Proxies allowed to forward. |
| `redstone.preset` | `MCRS_REDSTONE_PRESET` | `java` or `bedrock`; `java` | Redstone preset. |
| `redstone.quasi-connectivity` | `MCRS_REDSTONE_QUASI_CONNECTIVITY` | switch; from the preset | |
| `redstone.sticky-piston-short-pulse` | `MCRS_REDSTONE_STICKY_PISTON_SHORT_PULSE` | switch; from the preset | |
| `redstone.pistons-move-block-entities` | `MCRS_REDSTONE_PISTONS_MOVE_BLOCK_ENTITIES` | switch; from the preset | |
| `redstone.dust-connects-to-pistons` | `MCRS_REDSTONE_DUST_CONNECTS_TO_PISTONS` | switch; from the preset | |
| `redstone.experimental-wire` | `MCRS_REDSTONE_EXPERIMENTAL_WIRE` | switch; from the preset | |
| `entities.collisions.max-per-tick` | `MCRS_ENTITIES_COLLISIONS_MAX_PER_TICK` | 0–65535; 8 | Push neighbours an entity takes in one tick (PHY-CMP-019); 0 = every neighbour, as the vanilla server. The crowd damage keeps the game rule `max_entity_cramming`. |

Values that are constants of the code today and must become settings (rule H.9):

| Where | Value | Today |
|---|---|---|
| `mcrs_minecraft_brain_stage::default_threads` | threads of the AI pool | `available_parallelism / 2` |
| `StageConfig::DEFAULT_PATH_BUDGET`, `DEFAULT_TILE_PATH_BUDGET` | path search pops per run, per dimension and per tile | 65 536 / 4 096 |
| `StageConfig::DEFAULT_TILE_SPLIT` | brains per stage task | 256 |
| `StageConfig::DEFAULT_MAX_LAG`, `AiDelay` | lag of the AI stage | 2, `AtLeast(1)` |
| `SimulationDistance` | entity ticking box around a player | 10 chunks |
