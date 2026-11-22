use glam::DVec3;

use crate::adapter::PlayerMode;

pub const FLOOR_Y: i32 = -60;
const FORCELOAD_SIDE_CHUNKS: i32 = 16;
const FORCELOAD_MARGIN: i32 = 16;
pub const SAMPLE_PREFIX: &str = "mbs";
pub const WATCHER_NAME: &str = "mobbench";

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mob {
    Pig,
    Zombie,
}

impl Mob {
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Mob::Pig => "minecraft:pig",
            Mob::Zombie => "minecraft:zombie",
        }
    }

    #[must_use]
    pub fn kind(self) -> mcrs_minecraft_entity::entity_type_keys::EntityType {
        match self {
            Mob::Pig => mcrs_minecraft_entity::entity_type_keys::EntityType::Pig,
            Mob::Zombie => mcrs_minecraft_entity::entity_type_keys::EntityType::Zombie,
        }
    }
}

impl std::str::FromStr for Mob {
    type Err = anyhow::Error;

    fn from_str(name: &str) -> anyhow::Result<Self> {
        Ok(match name {
            "pig" => Mob::Pig,
            "zombie" => Mob::Zombie,
            other => anyhow::bail!("unknown mob {other}; pig or zombie"),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Layout {
    Spread,
    Point,
}

impl std::str::FromStr for Layout {
    type Err = anyhow::Error;

    fn from_str(name: &str) -> anyhow::Result<Self> {
        Ok(match name {
            "spread" => Layout::Spread,
            "point" => Layout::Point,
            other => anyhow::bail!("unknown layout {other}; spread or point"),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ai {
    On,
    Off,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct Area {
    pub min_x: i32,
    pub min_z: i32,
    pub size: i32,
}

impl Area {
    #[must_use]
    pub fn center(self) -> DVec3 {
        DVec3::new(
            f64::from(self.min_x) + f64::from(self.size) / 2.0,
            f64::from(FLOOR_Y),
            f64::from(self.min_z) + f64::from(self.size) / 2.0,
        )
    }
}

pub const PEN: Area = Area {
    min_x: -125,
    min_z: -125,
    size: 250,
};

pub const ROAM: Area = Area {
    min_x: 380,
    min_z: -320,
    size: 640,
};

pub const TILE: Area = Area {
    min_x: 0,
    min_z: 0,
    size: 32,
};

pub const PEN_WATCH: DVec3 = DVec3::new(-130.5, -58.0, 0.5);

#[must_use]
pub fn settings(mob: Mob) -> Vec<String> {
    let time = match mob {
        Mob::Pig => "time set noon",
        Mob::Zombie => "time set midnight",
    };
    [
        "gamerule spawn_mobs false",
        "gamerule spawn_monsters false",
        "gamerule spawn_patrols false",
        "gamerule spawn_wandering_traders false",
        "gamerule advance_time false",
        "gamerule advance_weather false",
        "gamerule max_entity_cramming 0",
        "gamerule mob_drops false",
        time,
        "weather clear",
    ]
    .map(str::to_owned)
    .to_vec()
}

#[must_use]
pub fn forceload(area: Area) -> Vec<String> {
    let low = |min: i32| (min - FORCELOAD_MARGIN).div_euclid(16);
    let high = |min: i32| (min + area.size - 1 + FORCELOAD_MARGIN).div_euclid(16);
    let mut lines = Vec::new();
    let mut chunk_x = low(area.min_x);
    while chunk_x <= high(area.min_x) {
        let end_x = (chunk_x + FORCELOAD_SIDE_CHUNKS - 1).min(high(area.min_x));
        let mut chunk_z = low(area.min_z);
        while chunk_z <= high(area.min_z) {
            let end_z = (chunk_z + FORCELOAD_SIDE_CHUNKS - 1).min(high(area.min_z));
            lines.push(format!(
                "forceload add {} {} {} {}",
                chunk_x * 16,
                chunk_z * 16,
                end_x * 16 + 15,
                end_z * 16 + 15
            ));
            chunk_z = end_z + 1;
        }
        chunk_x = end_x + 1;
    }
    lines
}

#[must_use]
pub fn fence(area: Area) -> Vec<String> {
    let (low_x, low_z) = (area.min_x - 1, area.min_z - 1);
    let (high_x, high_z) = (area.min_x + area.size, area.min_z + area.size);
    let y = FLOOR_Y;
    [
        (low_x, low_z, high_x, low_z),
        (low_x, high_z, high_x, high_z),
        (low_x, low_z, low_x, high_z),
        (high_x, low_z, high_x, high_z),
    ]
    .map(|(x1, z1, x2, z2)| format!("fill {x1} {y} {z1} {x2} {y} {z2} minecraft:oak_fence"))
    .to_vec()
}

const UUID_HIGH: u32 = 0x006d_6273;
const STIR_STEP: usize = 7919;

#[must_use]
pub fn pig_uuid(generation: u32, index: usize) -> String {
    format!(
        "{UUID_HIGH:08x}-{:04x}-{:04x}-0000-0000{:08x}",
        generation >> 16,
        generation & 0xffff,
        index as u32
    )
}

#[must_use]
pub fn sample_index(count: usize, samples: usize, sample: usize) -> usize {
    let stride = count
        .checked_div(samples)
        .map_or(usize::MAX, |stride| stride.max(1));
    sample * stride + stride / 2
}

#[must_use]
pub fn summons(
    area: Area,
    generation: u32,
    count: usize,
    samples: usize,
    ai: Ai,
    mob: Mob,
    layout: Layout,
) -> Vec<String> {
    let id = mob.id();
    let centre = area.center();
    let side = (count as f64).sqrt().ceil().max(1.0) as usize;
    let spacing = f64::from(area.size) / side as f64;
    let named: Vec<usize> = (0..samples)
        .map(|sample| sample_index(count, samples, sample))
        .collect();
    (0..count)
        .map(|index| {
            let (x, z) = match layout {
                Layout::Spread => (
                    f64::from(area.min_x) + ((index % side) as f64 + 0.5) * spacing,
                    f64::from(area.min_z) + ((index / side) as f64 + 0.5) * spacing,
                ),
                Layout::Point => (centre.x, centre.z),
            };
            let no_ai = match ai {
                Ai::On => "",
                Ai::Off => ",NoAI:1b",
            };
            let uuid = format!(
                "UUID:[I;{UUID_HIGH},{},0,{}]{no_ai}",
                generation as i32,
                index as u32 as i32
            );
            match named.iter().position(|named| *named == index) {
                Some(sample) => format!(
                    "summon {id} {x:.2} {FLOOR_Y} {z:.2} {{{uuid},CustomName:\"{SAMPLE_PREFIX}{sample}\"}}"
                ),
                None => format!("summon {id} {x:.2} {FLOOR_Y} {z:.2} {{{uuid}}}"),
            }
        })
        .collect()
}

#[must_use]
pub fn kill(mob: Mob) -> String {
    format!("kill @e[type={}]", mob.id())
}

#[must_use]
pub fn count(mob: Mob) -> String {
    format!("execute if entity @e[type={}]", mob.id())
}

const GUARD_RADIUS: i32 = 3;
const GUARD_HEIGHT: i32 = 2;

#[must_use]
pub fn guard(at: DVec3) -> Vec<String> {
    let (cx, cz) = (at.x.floor() as i32, at.z.floor() as i32);
    let (low_x, low_z) = (cx - GUARD_RADIUS, cz - GUARD_RADIUS);
    let (high_x, high_z) = (cx + GUARD_RADIUS, cz + GUARD_RADIUS);
    let mut lines = Vec::new();
    for layer in 0..GUARD_HEIGHT {
        let y = FLOOR_Y + layer;
        for (x1, z1, x2, z2) in [
            (low_x, low_z, high_x, low_z),
            (low_x, high_z, high_x, high_z),
            (low_x, low_z, low_x, high_z),
            (high_x, low_z, high_x, high_z),
        ] {
            lines.push(format!(
                "fill {x1} {y} {z1} {x2} {y} {z2} minecraft:oak_fence"
            ));
        }
    }
    lines
}

#[must_use]
pub fn resistance(player: &str) -> String {
    format!("effect give {player} minecraft:resistance infinite 255 true")
}
pub const COUNT_SPECTATORS: &str = "execute if entity @a[gamemode=spectator]";

#[must_use]
pub fn panic_hits(generation: u32, round: usize, hit: usize, count: usize) -> Vec<String> {
    (0..hit.min(count))
        .map(|offset| {
            let index = ((round * hit + offset) * STIR_STEP) % count.max(1);
            format!("damage {} 0 minecraft:cactus", pig_uuid(generation, index))
        })
        .collect()
}

#[must_use]
pub fn sample_query(generation: u32, count: usize, samples: usize, sample: usize) -> String {
    format!(
        "data get entity {} Pos",
        pig_uuid(generation, sample_index(count, samples, sample))
    )
}

#[must_use]
pub fn game_mode(player: &str, mode: PlayerMode) -> String {
    format!("gamemode {} {player}", mode.as_str())
}

#[must_use]
pub fn teleport(player: &str, at: DVec3) -> String {
    format!("tp {player} {} {} {}", at.x, at.y, at.z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forceload_covers_the_area_and_its_margin_within_the_chunk_limit() {
        let lines = forceload(ROAM);
        let mut chunks = 0;
        for line in &lines {
            let numbers: Vec<i32> = line
                .trim_start_matches("forceload add ")
                .split(' ')
                .map(|part| part.parse().unwrap())
                .collect();
            let [x1, z1, x2, z2] = numbers[..] else {
                panic!("{line}");
            };
            let area = ((x2 - x1 + 1) / 16) * ((z2 - z1 + 1) / 16);
            assert!(area <= 256, "{line} names {area} chunks");
            chunks += area;
        }
        assert_eq!(chunks, 43 * 42);
    }

    #[test]
    fn the_panic_rounds_hit_every_pig_once_before_any_twice() {
        let mut hit = std::collections::HashSet::new();
        for round in 0..25 {
            for line in panic_hits(1, round, 2000, 50_000) {
                assert!(
                    hit.insert(line),
                    "a pig was hit twice in the first 50,000 hits"
                );
            }
        }
    }

    #[test]
    fn the_summons_stay_in_the_area_and_keep_the_sample_and_uuid_formats() {
        let lines = summons(PEN, 1, 50_000, 32, Ai::On, Mob::Pig, Layout::Spread);
        assert_eq!(lines.len(), 50_000);
        let named = lines
            .iter()
            .filter(|line| line.contains("CustomName"))
            .count();
        assert_eq!(named, 32);
        for line in &lines {
            let parts: Vec<f64> = line
                .split(' ')
                .skip(2)
                .take(3)
                .map(|part| part.parse().unwrap())
                .collect();
            assert!((-125.0..125.0).contains(&parts[0]), "{line}");
            assert!((-125.0..125.0).contains(&parts[2]), "{line}");
        }
        assert_eq!(pig_uuid(2, 1234), "006d6273-0000-0002-0000-0000000004d2");
    }
}
