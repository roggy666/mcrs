use glam::DVec3;
use glam::Vec3;
use mcrs_minecraft_brain::AiDelay;
use mcrs_minecraft_brain::AiDriver;
use mcrs_minecraft_brain::Applied;
use mcrs_minecraft_brain::Cadence;
use mcrs_minecraft_brain::Decision;
use mcrs_minecraft_brain::EntityRef;
use mcrs_minecraft_brain::HurtBy;
use mcrs_minecraft_brain::ItemTables;
use mcrs_minecraft_brain::MeleeIntent;
use mcrs_minecraft_brain::MobFlags;
use mcrs_minecraft_brain::MobView;
use mcrs_minecraft_brain::PathIntent;
use mcrs_minecraft_brain::PlayerRow;
use mcrs_minecraft_brain::PlayerTable;
use mcrs_minecraft_brain::Progress;
use mcrs_minecraft_brain::RangedIntent;
use mcrs_minecraft_brain::StageConfig;
use mcrs_minecraft_brain::Tables;
use mcrs_minecraft_brain::TargetIntent;
use mcrs_minecraft_brain::TileSnapshot;
use mcrs_minecraft_brain::WorldSeed;
use mcrs_minecraft_entity::entity_type_keys::EntityType;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_entity_key::TileKey;
use mcrs_minecraft_goals::FlatGround;
use mcrs_minecraft_goals::GoalTable;
use mcrs_minecraft_goals::TableBrain;
use mcrs_minecraft_vanilla_mobs::kind;
use mcrs_minecraft_vanilla_mobs::kind_table;
use mcrs_minecraft_vanilla_mobs::run_tiles;
use mcrs_minecraft_vanilla_mobs::table_of;
use std::collections::BTreeMap;

struct Mob {
    key: EntityUuid,
    kind: EntityType,
    position: DVec3,
    walk: Option<(DVec3, f64)>,
    path: Applied,
    target: EntityRef,
    melee: bool,
    ranged: bool,
    follow_range: f32,
}

struct Yard {
    driver: AiDriver<TableBrain>,
    ground: FlatGround,
    kinds: mcrs_minecraft_brain::KindTable,
    mobs: Vec<Mob>,
    players: Vec<PlayerRow>,
}

const PLAYER_EYE_HEIGHT: f32 = 1.62;
const FIRST_PLAYER: u64 = 1000;

const SPEED: f64 = 0.3;
const GRASS_WALK: f64 = 2.2026;

impl Yard {
    fn new() -> Self {
        let config = StageConfig::new(WorldSeed(7))
            .with_delay(2, AiDelay::Fixed(1))
            .unwrap();
        Self {
            driver: AiDriver::new(config),
            ground: FlatGround {
                side: 128,
                pond: None,
            },
            kinds: kind_table(),
            mobs: Vec::new(),
            players: Vec::new(),
        }
    }

    fn add(&mut self, kind: EntityType, position: DVec3, follow_range: f32) -> usize {
        let index = self.mobs.len();
        let key = EntityUuid::from_u128((index as u128 + 1) << 64);
        let table: &'static GoalTable = table_of(kind).unwrap();
        self.driver.spawn(
            TileKey::at(position),
            EntityId(index as u64),
            key,
            Cadence::new(1, 0),
            TableBrain::new(table),
        );
        self.mobs.push(Mob {
            key,
            kind,
            position,
            walk: None,
            path: Applied::default(),
            target: EntityRef::NONE,
            melee: false,
            ranged: false,
            follow_range,
        });
        index
    }

    fn add_player(&mut self, position: DVec3) -> EntityId {
        let id = EntityId(FIRST_PLAYER + self.players.len() as u64);
        let key = EntityUuid::from_u128(u128::from(id.0) << 64);
        self.players
            .push(PlayerRow::standing(id, key, position, PLAYER_EYE_HEIGHT));
        id
    }

    fn view(&self, index: usize, mob: &Mob) -> MobView {
        let marks = [(
            mob.path.progress == Progress::Running,
            MobFlags::PATH_RUNNING,
        )];
        MobView {
            mob: EntityId(index as u64),
            key: mob.key,
            position: mob.position,
            velocity: Vec3::ZERO,
            yaw: 0.0,
            kind: kind(mob.kind),
            flags: MobFlags::ALIVE
                .union(MobFlags::ON_GROUND)
                .union(MobFlags::of(marks)),
            health: 8.0,
            follow_range: mob.follow_range,
            age: 0,
            love: 0,
            idle_time: 0,
            damage: None,
            hurt_by: HurtBy::NONE,
            target: mob.target,
            owner: EntityRef::NONE,
            path_ticket: mob.path.ticket,
        }
    }

    fn step(&mut self) {
        self.driver.begin_tick();
        let mut tiles: BTreeMap<TileKey, Vec<MobView>> = BTreeMap::new();
        for (index, mob) in self.mobs.iter().enumerate() {
            tiles
                .entry(TileKey::at(mob.position))
                .or_default()
                .push(self.view(index, mob));
        }
        for (tile, views) in tiles {
            self.driver
                .publish_tile(tile, TileSnapshot::new(tile, views));
        }
        self.driver
            .publish_players(PlayerTable::new(self.players.iter().copied()));
        let items = ItemTables::default();
        let tables = Tables {
            items: &items,
            kinds: &self.kinds,
        };
        self.driver.run_stage(&self.ground, &tables, run_tiles);
        let due = self.driver.take_due();
        for issued in due {
            for decision in issued.tiles.values().flatten() {
                self.apply(decision);
            }
        }
        let mut moves = Vec::new();
        for (index, mob) in self.mobs.iter_mut().enumerate() {
            let from = TileKey::at(mob.position);
            if let Some((target, per_tick)) = mob.walk {
                let delta = target - mob.position;
                let flat = DVec3::new(delta.x, 0.0, delta.z);
                if flat.length() <= per_tick {
                    mob.position = target;
                    mob.walk = None;
                    mob.path.progress = Progress::Ended;
                } else {
                    mob.position += flat.normalize() * per_tick;
                }
            }
            let to = TileKey::at(mob.position);
            if to != from {
                moves.push((EntityId(index as u64), to));
            }
        }
        self.driver.relocate_all(moves);
    }

    fn apply(&mut self, decision: &Decision) {
        let Some(mob) = self.mobs.get_mut(decision.mob.0 as usize) else {
            return;
        };
        match decision.intents.path {
            Some(PathIntent::Walk {
                ticket,
                target,
                speed_modifier,
            }) => {
                let speed = f64::from(speed_modifier) * SPEED;
                mob.walk = Some((target, speed * speed * GRASS_WALK));
                mob.path = Applied {
                    ticket,
                    progress: Progress::Running,
                };
            }
            Some(PathIntent::Clear { ticket }) => {
                mob.walk = None;
                mob.path = Applied {
                    ticket,
                    progress: Progress::Ended,
                };
            }
            None => {}
        }
        match decision.intents.target {
            Some(TargetIntent::Set(target)) => mob.target = EntityRef::of(target),
            Some(TargetIntent::Clear) => mob.target = EntityRef::NONE,
            None => {}
        }
        match decision.intents.melee {
            Some(MeleeIntent::Start { .. }) => mob.melee = true,
            Some(MeleeIntent::Stop) => mob.melee = false,
            None => {}
        }
        match decision.intents.ranged {
            Some(RangedIntent::Start { .. }) => mob.ranged = true,
            Some(RangedIntent::Stop) => mob.ranged = false,
            None => {}
        }
    }
}

#[test]
fn a_wild_wolf_hunts_the_sheep_it_sees_and_runs_at_it() {
    let mut yard = Yard::new();
    let wolf = yard.add(EntityType::Wolf, DVec3::new(40.5, 0.0, 40.5), 16.0);
    let sheep = yard.add(EntityType::Sheep, DVec3::new(48.5, 0.0, 40.5), 16.0);
    let mut targeted_at = None;
    let mut attacking = false;
    for tick in 1..=200 {
        yard.step();
        if yard.mobs[wolf].target.is(EntityId(sheep as u64)) && targeted_at.is_none() {
            targeted_at = Some(tick);
        }
        attacking |= yard.mobs[wolf].melee;
    }
    let targeted = targeted_at.expect("the wolf selects the sheep as its target within 200 ticks");
    assert!(
        targeted <= 120,
        "the hunt gate of 1/5 per evaluation selects the sheep soon: tick {targeted}"
    );
    assert!(attacking, "the melee goal starts after the target is set");
    let distance = yard.mobs[wolf].position.distance(yard.mobs[sheep].position);
    assert!(
        distance < 8.0,
        "the wolf closes in on the sheep: {distance} blocks left"
    );
}

#[test]
fn a_pig_never_selects_a_target() {
    let mut yard = Yard::new();
    let pig = yard.add(EntityType::Pig, DVec3::new(40.5, 0.0, 40.5), 16.0);
    yard.add(EntityType::Sheep, DVec3::new(44.5, 0.0, 40.5), 16.0);
    for _ in 0..200 {
        yard.step();
    }
    assert!(yard.mobs[pig].target.is_none());
    assert!(!yard.mobs[pig].melee);
}

#[test]
fn a_zombie_hunts_the_player_it_sees_and_closes_in() {
    let mut yard = Yard::new();
    let zombie = yard.add(EntityType::Zombie, DVec3::new(40.5, 0.0, 40.5), 16.0);
    let player = yard.add_player(DVec3::new(48.5, 0.0, 40.5));
    let mut targeted_at = None;
    let mut attacking = false;
    for tick in 1..=200 {
        yard.step();
        if yard.mobs[zombie].target.is(player) && targeted_at.is_none() {
            targeted_at = Some(tick);
        }
        attacking |= yard.mobs[zombie].melee;
    }
    let targeted = targeted_at.expect("the zombie selects the player within 200 ticks");
    assert!(
        targeted <= 40,
        "the player goal of the zombie has no hunt gate: tick {targeted}"
    );
    assert!(
        attacking,
        "the melee goal of the zombie starts after the target is set"
    );
    let distance = yard.mobs[zombie]
        .position
        .distance(DVec3::new(48.5, 0.0, 40.5));
    assert!(
        distance < 8.0,
        "the zombie closes in on the player: {distance} blocks left"
    );
}

#[test]
fn a_skeleton_draws_its_bow_at_the_player_in_range() {
    let mut yard = Yard::new();
    let skeleton = yard.add(EntityType::Skeleton, DVec3::new(40.5, 0.0, 40.5), 16.0);
    let player = yard.add_player(DVec3::new(50.5, 0.0, 40.5));
    let mut shooting = false;
    for _ in 1..=200 {
        yard.step();
        shooting |= yard.mobs[skeleton].ranged;
    }
    assert!(yard.mobs[skeleton].target.is(player));
    assert!(
        shooting,
        "the ranged goal of the skeleton starts on the player"
    );
    assert!(
        !yard.mobs[skeleton].melee,
        "the skeleton table has no melee goal"
    );
}

#[test]
fn a_rabbit_runs_from_a_wolf_faster_than_the_wolf_follows() {
    let mut yard = Yard::new();
    let start = DVec3::new(40.5, 0.0, 40.5);
    let rabbit = yard.add(EntityType::Rabbit, start, 16.0);
    let wolf = yard.add(EntityType::Wolf, DVec3::new(45.5, 0.0, 40.5), 16.0);
    for _ in 1..=100 {
        yard.step();
    }
    let fled = yard.mobs[rabbit].position.distance(start);
    let gap = yard.mobs[rabbit]
        .position
        .distance(yard.mobs[wolf].position);
    assert!(fled >= 8.0, "the rabbit left its start: {fled} blocks");
    assert!(gap > 5.0, "the gap to the wolf grew from 5 blocks: {gap}");
    assert!(yard.mobs[rabbit].target.is_none());
}

#[test]
fn a_vindicator_charges_the_player_and_a_pillager_draws_its_crossbow() {
    let mut yard = Yard::new();
    let vindicator = yard.add(EntityType::Vindicator, DVec3::new(40.5, 0.0, 40.5), 12.0);
    let pillager = yard.add(EntityType::Pillager, DVec3::new(40.5, 0.0, 44.5), 32.0);
    let player = yard.add_player(DVec3::new(47.5, 0.0, 42.5));
    let mut charging = false;
    let mut drawing = false;
    for _ in 1..=200 {
        yard.step();
        charging |= yard.mobs[vindicator].melee;
        drawing |= yard.mobs[pillager].ranged;
    }
    assert!(yard.mobs[vindicator].target.is(player));
    assert!(yard.mobs[pillager].target.is(player));
    assert!(
        charging,
        "the melee goal of the vindicator starts on the player"
    );
    assert!(
        drawing,
        "the ranged goal of the pillager starts on the player"
    );
    assert!(
        !yard.mobs[pillager].melee,
        "the pillager table has no melee goal"
    );
}

#[test]
fn an_evoker_backs_away_from_a_player_within_eight_blocks() {
    let mut yard = Yard::new();
    let start = DVec3::new(40.5, 0.0, 40.5);
    let evoker = yard.add(EntityType::Evoker, start, 12.0);
    let at = DVec3::new(45.5, 0.0, 40.5);
    let player = yard.add_player(at);
    for _ in 1..=100 {
        yard.step();
    }
    let gap = yard.mobs[evoker].position.distance(at);
    assert!(gap > 8.0, "the evoker keeps more than 8 blocks: {gap}");
    assert!(yard.mobs[evoker].target.is(player));
    assert!(
        !yard.mobs[evoker].melee,
        "the evoker table has no melee goal"
    );
}
