use glam::DVec3;
use glam::Vec3;
use mcrs_minecraft_brain::BrainRules;
use mcrs_minecraft_brain::Cadence;
use mcrs_minecraft_brain::ControlFlags;
use mcrs_minecraft_brain::DamageCause;
use mcrs_minecraft_brain::DamageRecord;
use mcrs_minecraft_brain::Dimensions;
use mcrs_minecraft_brain::EntityRef;
use mcrs_minecraft_brain::HurtBy;
use mcrs_minecraft_brain::Intents;
use mcrs_minecraft_brain::ItemTables;
use mcrs_minecraft_brain::KindData;
use mcrs_minecraft_brain::KindFlags;
use mcrs_minecraft_brain::KindTable;
use mcrs_minecraft_brain::MobBrain;
use mcrs_minecraft_brain::MobFlags;
use mcrs_minecraft_brain::MobKind;
use mcrs_minecraft_brain::MobView;
use mcrs_minecraft_brain::Neighbours;
use mcrs_minecraft_brain::PathIntent;
use mcrs_minecraft_brain::PlayerTable;
use mcrs_minecraft_brain::Ticket;
use mcrs_minecraft_brain::Turn;
use mcrs_minecraft_brain::TurnScratch;
use mcrs_minecraft_brain::WorldSeed;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_entity_key::TileKey;
use mcrs_minecraft_goals::FlatGround;
use mcrs_minecraft_goals::GoalKind;
use mcrs_minecraft_goals::GoalRow;
use mcrs_minecraft_goals::GoalTable;
use mcrs_minecraft_goals::TableBrain;
use mcrs_minecraft_goals::Water;

const ANIMAL: MobKind = MobKind(7);

static TABLE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(1, ControlFlags::MOVE, GoalKind::Panic { speed: 1.25 }),
        GoalRow::new(
            6,
            ControlFlags::MOVE,
            GoalKind::Stroll {
                speed: 1.0,
                water: Water::Avoid,
            },
        ),
        GoalRow::new(7, ControlFlags::LOOK, GoalKind::LookAtPlayer { range: 6.0 }),
        GoalRow::new(
            8,
            ControlFlags::MOVE.union(ControlFlags::LOOK),
            GoalKind::LookAround,
        ),
    ],
    targets: &[],
    rules: BrainRules {
        player_range: 6.0,
        idle_limit: 100,
        idle_reset_range: 32.0,
    },
};

fn view(key: u128, damage: Option<DamageRecord>) -> MobView {
    MobView {
        mob: EntityId(0),
        key: EntityUuid::from_u128(key << 64),
        position: DVec3::new(32.5, 0.0, 32.5),
        velocity: Vec3::ZERO,
        kind: ANIMAL,
        flags: MobFlags::ALIVE.union(MobFlags::ON_GROUND),
        health: 10.0,
        follow_range: 16.0,
        age: 0,
        love: 0,
        idle_time: 0,
        damage,
        hurt_by: HurtBy::NONE,
        target: EntityRef::NONE,
        owner: EntityRef::NONE,
        path_ticket: Ticket::default(),
    }
}

fn kinds() -> KindTable {
    KindTable::new([(
        ANIMAL,
        KindData::new(Dimensions::new(0.9, 0.9, 0.765), KindFlags::ANIMAL),
    )])
}

#[test]
fn a_hurt_animal_of_the_table_requests_a_panic_path_at_the_panic_speed() {
    let ground = FlatGround {
        side: 64,
        pond: None,
    };
    let kinds = kinds();
    let items = ItemTables::default();
    let players = PlayerTable::default();
    let neighbours = Neighbours::empty(TileKey::new(1, 0, 1));
    let mut scratch = TurnScratch::default();
    let speed = (0..32u128).find_map(|key| {
        let view = view(
            key,
            Some(DamageRecord {
                tick: 19,
                cause: DamageCause::Panic,
            }),
        );
        let turn = Turn {
            tick: 20,
            seed: WorldSeed(41),
            max_lag: 0,
            cadence: Cadence::new(1, 0),
            view: &view,
            players: &[],
            table: &players,
            neighbours: &neighbours,
            items: &items,
            kinds: &kinds,
            world: &ground,
        };
        let mut brain = TableBrain::new(&TABLE);
        let mut intents = Intents::default();
        brain.evaluate(&turn, &mut scratch, &mut intents);
        assert!(brain.panics() || intents.path.is_none());
        match intents.path {
            Some(PathIntent::Walk { speed_modifier, .. }) => Some(speed_modifier),
            _ => None,
        }
    });
    assert_eq!(speed, Some(1.25));
}

#[test]
fn an_animal_with_no_hit_does_not_panic_and_sleeps_until_a_gate_is_due() {
    let ground = FlatGround {
        side: 64,
        pond: None,
    };
    let kinds = kinds();
    let items = ItemTables::default();
    let players = PlayerTable::default();
    let neighbours = Neighbours::empty(TileKey::new(1, 0, 1));
    let mut scratch = TurnScratch::default();
    let view = view(3, None);
    let turn = Turn {
        tick: 20,
        seed: WorldSeed(41),
        max_lag: 0,
        cadence: Cadence::new(1, 0),
        view: &view,
        players: &[],
        table: &players,
        neighbours: &neighbours,
        items: &items,
        kinds: &kinds,
        world: &ground,
    };
    let mut brain = TableBrain::new(&TABLE);
    let mut intents = Intents::default();
    let wake = brain.evaluate(&turn, &mut scratch, &mut intents);
    assert!(!brain.panics());
    assert!(
        wake > 21,
        "a mob with only gated goals sleeps until the first wait is due: {wake}"
    );
}
