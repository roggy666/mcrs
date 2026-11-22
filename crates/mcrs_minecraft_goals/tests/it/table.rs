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
use mcrs_minecraft_brain::KindSet;
use mcrs_minecraft_brain::KindTable;
use mcrs_minecraft_brain::MobBrain;
use mcrs_minecraft_brain::MobFlags;
use mcrs_minecraft_brain::MobKind;
use mcrs_minecraft_brain::MobView;
use mcrs_minecraft_brain::Neighbours;
use mcrs_minecraft_brain::PathIntent;
use mcrs_minecraft_brain::PlayerRow;
use mcrs_minecraft_brain::PlayerTable;
use mcrs_minecraft_brain::Proposal;
use mcrs_minecraft_brain::Receivers;
use mcrs_minecraft_brain::Ticket;
use mcrs_minecraft_brain::Turn;
use mcrs_minecraft_brain::TurnScratch;
use mcrs_minecraft_brain::WorldSeed;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_entity_key::TileKey;
use mcrs_minecraft_goals::Alert;
use mcrs_minecraft_goals::FlatGround;
use mcrs_minecraft_goals::GoalKind;
use mcrs_minecraft_goals::GoalRow;
use mcrs_minecraft_goals::GoalTable;
use mcrs_minecraft_goals::Panicker;
use mcrs_minecraft_goals::StrollSearch;
use mcrs_minecraft_goals::TableBrain;

const ANIMAL: MobKind = MobKind(7);

static TABLE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(
            1,
            ControlFlags::MOVE,
            GoalKind::Panic {
                speed: 1.25,
                who: Panicker::Any,
            },
        ),
        GoalRow::new(
            6,
            ControlFlags::MOVE,
            GoalKind::Stroll {
                speed: 1.0,
                search: StrollSearch::Default,
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

#[test]
fn wake_007_a_pending_stroll_wait_wakes_the_brain_when_the_idle_time_reaches_the_limit() {
    let ground = FlatGround {
        side: 64,
        pond: None,
    };
    let kinds = kinds();
    let items = ItemTables::default();
    let players = PlayerTable::default();
    let neighbours = Neighbours::empty(TileKey::new(1, 0, 1));
    let mut scratch = TurnScratch::default();
    let mut view = view(3, None);
    view.idle_time = 90;
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
    assert!(
        wake > 20 && wake <= 30,
        "the brain wakes no later than the tick in which the idle time reaches 100: {wake}"
    );
}

static ALLIES: KindSet = KindSet::of(&[MobKind(7), MobKind(9)]);

static ALERTING: GoalTable = GoalTable {
    goals: &[],
    targets: &[GoalRow::new(
        1,
        ControlFlags::TARGET,
        GoalKind::HurtBy {
            alert: Alert::Kinds(&ALLIES),
            ignored: KindSet::NONE,
        },
    )],
    rules: BrainRules {
        player_range: 16.0,
        idle_limit: 100,
        idle_reset_range: 32.0,
    },
};

#[test]
fn a_hurt_by_goal_with_a_kind_set_alert_proposes_the_alert_to_that_set() {
    let ground = FlatGround {
        side: 64,
        pond: None,
    };
    let kinds = kinds();
    let items = ItemTables::default();
    let attacker = EntityId(77);
    let players = PlayerTable::new([PlayerRow::standing(
        attacker,
        EntityUuid::from_u128(77 << 64),
        DVec3::new(36.5, 0.0, 32.5),
        1.62,
    )]);
    let neighbours = Neighbours::empty(TileKey::new(1, 0, 1));
    let mut scratch = TurnScratch::default();
    let mut view = view(3, None);
    view.hurt_by = HurtBy::by(attacker, 19);
    let turn = Turn {
        tick: 20,
        seed: WorldSeed(41),
        max_lag: 0,
        cadence: Cadence::new(1, 0),
        view: &view,
        players: players.rows(),
        table: &players,
        neighbours: &neighbours,
        items: &items,
        kinds: &kinds,
        world: &ground,
    };
    let mut brain = TableBrain::new(&ALERTING);
    let mut intents = Intents::default();
    brain.evaluate(&turn, &mut scratch, &mut intents);
    assert_eq!(
        intents.proposal,
        Some(Proposal::Alert {
            target: attacker,
            reach: 16.0,
            receivers: Receivers::Kinds(&ALLIES),
        })
    );
}

static BABY_PANIC: GoalTable = GoalTable {
    goals: &[GoalRow::new(
        1,
        ControlFlags::MOVE,
        GoalKind::Panic {
            speed: 2.0,
            who: Panicker::Baby,
        },
    )],
    targets: &[],
    rules: BrainRules {
        player_range: 6.0,
        idle_limit: 100,
        idle_reset_range: 32.0,
    },
};

fn panic_path_of(age: i32) -> Option<f32> {
    let ground = FlatGround {
        side: 64,
        pond: None,
    };
    let kinds = kinds();
    let items = ItemTables::default();
    let players = PlayerTable::default();
    let neighbours = Neighbours::empty(TileKey::new(1, 0, 1));
    let mut scratch = TurnScratch::default();
    (0..32u128).find_map(|key| {
        let mut view = view(
            key,
            Some(DamageRecord {
                tick: 19,
                cause: DamageCause::Panic,
            }),
        );
        view.age = age;
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
        let mut brain = TableBrain::new(&BABY_PANIC);
        let mut intents = Intents::default();
        brain.evaluate(&turn, &mut scratch, &mut intents);
        match intents.path {
            Some(PathIntent::Walk { speed_modifier, .. }) => Some(speed_modifier),
            _ => None,
        }
    })
}

#[test]
fn a_panic_goal_for_babies_runs_for_a_hurt_baby_and_never_for_an_adult() {
    assert_eq!(panic_path_of(-1000), Some(2.0));
    assert_eq!(panic_path_of(0), None);
}
