use mcrs_minecraft_brain::BrainRules;
use mcrs_minecraft_brain::ControlFlags;
use mcrs_minecraft_brain::KindSet;
use mcrs_minecraft_brain::Pursuit;
use mcrs_minecraft_entity::entity_type_keys::EntityType;
use mcrs_minecraft_goals::Alert;
use mcrs_minecraft_goals::AvoidExtra;
use mcrs_minecraft_goals::GoalKind;
use mcrs_minecraft_goals::GoalRow;
use mcrs_minecraft_goals::GoalTable;
use mcrs_minecraft_goals::IDLE_RESET_RANGE;
use mcrs_minecraft_goals::STROLL_IDLE_LIMIT;
use mcrs_minecraft_goals::Sight;
use mcrs_minecraft_goals::TameRule;
use mcrs_minecraft_goals::TargetFilter;
use mcrs_minecraft_goals::TargetSet;
use mcrs_minecraft_goals::Water;

use crate::WOLF_KIND;
use crate::kind;

const MOVE: ControlFlags = ControlFlags::MOVE;
const LOOK: ControlFlags = ControlFlags::LOOK;
const JUMP: ControlFlags = ControlFlags::JUMP;
const TARGET: ControlFlags = ControlFlags::TARGET;
const MOVE_LOOK: ControlFlags = ControlFlags::MOVE.union(ControlFlags::LOOK);
const JUMP_MOVE: ControlFlags = ControlFlags::JUMP.union(ControlFlags::MOVE);
const NONE: ControlFlags = ControlFlags::NONE;

pub const VILLAGER: KindSet = KindSet::of(&[kind(EntityType::Villager)]);
pub const IRON_GOLEM: KindSet = KindSet::of(&[kind(EntityType::IronGolem)]);
pub const TURTLE_KIND: KindSet = KindSet::of(&[kind(EntityType::Turtle)]);
pub const ENDERMITE: KindSet = KindSet::of(&[kind(EntityType::Endermite)]);
pub const OCELOT: KindSet = KindSet::of(&[kind(EntityType::Ocelot)]);
pub const CAT: KindSet = KindSet::of(&[kind(EntityType::Cat)]);
pub const PIGLIN_KIND: KindSet =
    KindSet::of(&[kind(EntityType::Piglin), kind(EntityType::PiglinBrute)]);
pub const ZOMBIFIED_PIGLIN: KindSet = KindSet::of(&[kind(EntityType::ZombifiedPiglin)]);
pub const MONSTERS_NOT_CREEPER: KindSet = KindSet::of(&[
    kind(EntityType::Zombie),
    kind(EntityType::Husk),
    kind(EntityType::Drowned),
    kind(EntityType::ZombieVillager),
    kind(EntityType::ZombifiedPiglin),
    kind(EntityType::Skeleton),
    kind(EntityType::Stray),
    kind(EntityType::Bogged),
    kind(EntityType::Parched),
    kind(EntityType::WitherSkeleton),
    kind(EntityType::Spider),
    kind(EntityType::CaveSpider),
    kind(EntityType::Enderman),
    kind(EntityType::Silverfish),
    kind(EntityType::Endermite),
    kind(EntityType::Witch),
    kind(EntityType::Blaze),
    kind(EntityType::Slime),
    kind(EntityType::MagmaCube),
    kind(EntityType::Pillager),
    kind(EntityType::Vindicator),
    kind(EntityType::Evoker),
    kind(EntityType::Illusioner),
    kind(EntityType::Ravager),
    kind(EntityType::Vex),
    kind(EntityType::Phantom),
    kind(EntityType::Guardian),
    kind(EntityType::ElderGuardian),
    kind(EntityType::Piglin),
    kind(EntityType::PiglinBrute),
    kind(EntityType::Hoglin),
    kind(EntityType::Zoglin),
    kind(EntityType::Warden),
    kind(EntityType::Breeze),
    kind(EntityType::Creaking),
    kind(EntityType::Ghast),
    kind(EntityType::Giant),
]);

const MONSTER_RULES: BrainRules = BrainRules {
    player_range: 35.0,
    idle_limit: STROLL_IDLE_LIMIT,
    idle_reset_range: IDLE_RESET_RANGE,
};

const SHORT_RANGE_RULES: BrainRules = BrainRules {
    player_range: 16.0,
    idle_limit: STROLL_IDLE_LIMIT,
    idle_reset_range: IDLE_RESET_RANGE,
};

const fn melee(speed: f32, pursuit: Pursuit) -> GoalKind {
    GoalKind::Melee {
        speed,
        pursuit,
        interval: 20,
    }
}

const fn stroll(speed: f32) -> GoalKind {
    GoalKind::Stroll {
        speed,
        water: Water::Avoid,
    }
}

const LOOK_AT_PLAYER: GoalKind = GoalKind::LookAtPlayer { range: 8.0 };

const fn players(filter: TargetFilter) -> GoalKind {
    GoalKind::NearestTarget {
        set: TargetSet::Players,
        interval: 10,
        sight: Sight::MustSee,
        filter,
        follow_scale: 1.0,
        tame: TameRule::Any,
    }
}

const fn kinds(set: KindSet, sight: Sight, filter: TargetFilter) -> GoalKind {
    GoalKind::NearestTarget {
        set: TargetSet::Kinds(set),
        interval: 10,
        sight,
        filter,
        follow_scale: 1.0,
        tame: TameRule::Any,
    }
}

const fn hurt_by(alert: Alert, ignored: KindSet) -> GoalKind {
    GoalKind::HurtBy { alert, ignored }
}

const fn avoid(set: KindSet) -> GoalKind {
    GoalKind::Avoid {
        kinds: set,
        distance: 6.0,
        walk: 1.0,
        sprint: 1.2,
        extra: AvoidExtra::None,
    }
}

pub static ZOMBIE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(2, MOVE_LOOK, melee(1.0, Pursuit::Unseen)),
        GoalRow::new(7, MOVE, stroll(1.0)),
        GoalRow::new(8, LOOK, LOOK_AT_PLAYER),
        GoalRow::new(8, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(1, TARGET, hurt_by(Alert::SameKind, ZOMBIFIED_PIGLIN)),
        GoalRow::new(2, TARGET, players(TargetFilter::None)),
        GoalRow::new(3, TARGET, kinds(VILLAGER, Sight::Any, TargetFilter::None)),
        GoalRow::new(
            3,
            TARGET,
            kinds(IRON_GOLEM, Sight::MustSee, TargetFilter::None),
        ),
        GoalRow::new(
            5,
            TARGET,
            kinds(TURTLE_KIND, Sight::MustSee, TargetFilter::BabyOnLand),
        ),
    ],
    rules: MONSTER_RULES,
};

pub static ZOMBIFIED_PIGLIN_TABLE: GoalTable = GoalTable {
    goals: ZOMBIE.goals,
    targets: &[
        GoalRow::new(1, TARGET, hurt_by(Alert::SameKind, ZOMBIFIED_PIGLIN)),
        GoalRow::new(2, TARGET, players(TargetFilter::AngryAt)),
        GoalRow::new(
            3,
            NONE,
            GoalKind::ResetAnger {
                alert: Alert::SameKind,
            },
        ),
    ],
    rules: MONSTER_RULES,
};

pub static SKELETON: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(3, MOVE, GoalKind::FleeSun { speed: 1.0 }),
        GoalRow::new(3, MOVE, avoid(WOLF_KIND)),
        GoalRow::new(
            4,
            MOVE_LOOK,
            GoalKind::Ranged {
                speed: 1.0,
                min_interval: 40,
                max_interval: 40,
                range: 15.0,
            },
        ),
        GoalRow::new(5, MOVE, stroll(1.0)),
        GoalRow::new(6, LOOK, LOOK_AT_PLAYER),
        GoalRow::new(6, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(1, TARGET, hurt_by(Alert::None, KindSet::NONE)),
        GoalRow::new(2, TARGET, players(TargetFilter::None)),
        GoalRow::new(
            3,
            TARGET,
            kinds(IRON_GOLEM, Sight::MustSee, TargetFilter::None),
        ),
        GoalRow::new(
            3,
            TARGET,
            kinds(TURTLE_KIND, Sight::MustSee, TargetFilter::BabyOnLand),
        ),
    ],
    rules: SHORT_RANGE_RULES,
};

pub static WITHER_SKELETON: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(3, MOVE, avoid(WOLF_KIND)),
        GoalRow::new(4, MOVE_LOOK, melee(1.2, Pursuit::Seen)),
        GoalRow::new(5, MOVE, stroll(1.0)),
        GoalRow::new(6, LOOK, LOOK_AT_PLAYER),
        GoalRow::new(6, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(1, TARGET, hurt_by(Alert::None, KindSet::NONE)),
        GoalRow::new(2, TARGET, players(TargetFilter::None)),
        GoalRow::new(
            3,
            TARGET,
            kinds(IRON_GOLEM, Sight::MustSee, TargetFilter::None),
        ),
        GoalRow::new(
            3,
            TARGET,
            kinds(PIGLIN_KIND, Sight::MustSee, TargetFilter::None),
        ),
        GoalRow::new(
            3,
            TARGET,
            kinds(TURTLE_KIND, Sight::MustSee, TargetFilter::BabyOnLand),
        ),
    ],
    rules: SHORT_RANGE_RULES,
};

pub static CREEPER: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(1, JUMP, GoalKind::Float),
        GoalRow::new(2, MOVE, GoalKind::Swell),
        GoalRow::new(3, MOVE, avoid(OCELOT)),
        GoalRow::new(3, MOVE, avoid(CAT)),
        GoalRow::new(4, MOVE_LOOK, melee(1.0, Pursuit::Seen)),
        GoalRow::new(5, MOVE, stroll(0.8)),
        GoalRow::new(6, LOOK, LOOK_AT_PLAYER),
        GoalRow::new(6, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(1, TARGET, players(TargetFilter::None)),
        GoalRow::new(2, TARGET, hurt_by(Alert::None, KindSet::NONE)),
    ],
    rules: SHORT_RANGE_RULES,
};

pub static SPIDER: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(1, JUMP, GoalKind::Float),
        GoalRow::new(3, JUMP_MOVE, GoalKind::Leap { vertical: 0.4 }),
        GoalRow::new(4, MOVE_LOOK, melee(1.0, Pursuit::Seen)),
        GoalRow::new(5, MOVE, stroll(0.8)),
        GoalRow::new(6, LOOK, LOOK_AT_PLAYER),
        GoalRow::new(6, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(1, TARGET, hurt_by(Alert::None, KindSet::NONE)),
        GoalRow::new(2, TARGET, players(TargetFilter::None)),
        GoalRow::new(
            3,
            TARGET,
            kinds(IRON_GOLEM, Sight::MustSee, TargetFilter::None),
        ),
    ],
    rules: SHORT_RANGE_RULES,
};

pub static ENDERMAN: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(2, MOVE_LOOK, melee(1.0, Pursuit::Seen)),
        GoalRow::new(7, MOVE, stroll(1.0)),
        GoalRow::new(8, LOOK, LOOK_AT_PLAYER),
        GoalRow::new(8, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(2, TARGET, hurt_by(Alert::None, KindSet::NONE)),
        GoalRow::new(
            3,
            TARGET,
            kinds(ENDERMITE, Sight::MustSee, TargetFilter::None),
        ),
        GoalRow::new(4, NONE, GoalKind::ResetAnger { alert: Alert::None }),
    ],
    rules: BrainRules {
        player_range: 64.0,
        idle_limit: STROLL_IDLE_LIMIT,
        idle_reset_range: IDLE_RESET_RANGE,
    },
};

pub static SILVERFISH: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(1, JUMP, GoalKind::Float),
        GoalRow::new(4, MOVE_LOOK, melee(1.0, Pursuit::Seen)),
        GoalRow::new(7, MOVE, stroll(1.0)),
        GoalRow::new(8, LOOK, LOOK_AT_PLAYER),
        GoalRow::new(8, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(1, TARGET, hurt_by(Alert::SameKind, KindSet::NONE)),
        GoalRow::new(2, TARGET, players(TargetFilter::None)),
    ],
    rules: SHORT_RANGE_RULES,
};

pub static WITCH: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(1, JUMP, GoalKind::Float),
        GoalRow::new(
            2,
            MOVE_LOOK,
            GoalKind::Ranged {
                speed: 1.0,
                min_interval: 60,
                max_interval: 60,
                range: 10.0,
            },
        ),
        GoalRow::new(2, MOVE, stroll(1.0)),
        GoalRow::new(3, LOOK, LOOK_AT_PLAYER),
        GoalRow::new(3, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(1, TARGET, hurt_by(Alert::None, KindSet::NONE)),
        GoalRow::new(2, TARGET, players(TargetFilter::None)),
    ],
    rules: SHORT_RANGE_RULES,
};

pub static BLAZE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(4, MOVE_LOOK, melee(1.0, Pursuit::Seen)),
        GoalRow::new(7, MOVE, stroll(1.0)),
        GoalRow::new(8, LOOK, LOOK_AT_PLAYER),
        GoalRow::new(8, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(1, TARGET, hurt_by(Alert::None, KindSet::NONE)),
        GoalRow::new(2, TARGET, players(TargetFilter::None)),
    ],
    rules: BrainRules {
        player_range: 48.0,
        idle_limit: STROLL_IDLE_LIMIT,
        idle_reset_range: IDLE_RESET_RANGE,
    },
};

pub static IRON_GOLEM_TABLE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(1, MOVE_LOOK, melee(1.0, Pursuit::Unseen)),
        GoalRow::new(4, MOVE, stroll(0.6)),
        GoalRow::new(6, LOOK, GoalKind::LookAtPlayer { range: 6.0 }),
        GoalRow::new(7, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(2, TARGET, hurt_by(Alert::None, KindSet::NONE)),
        GoalRow::new(
            3,
            TARGET,
            GoalKind::NearestTarget {
                set: TargetSet::Kinds(MONSTERS_NOT_CREEPER),
                interval: 10,
                sight: Sight::MustSee,
                filter: TargetFilter::None,
                follow_scale: 1.0,
                tame: TameRule::Any,
            },
        ),
        GoalRow::new(4, NONE, GoalKind::ResetAnger { alert: Alert::None }),
    ],
    rules: SHORT_RANGE_RULES,
};

pub static SNOW_GOLEM: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(
            1,
            MOVE_LOOK,
            GoalKind::Ranged {
                speed: 1.25,
                min_interval: 20,
                max_interval: 20,
                range: 10.0,
            },
        ),
        GoalRow::new(2, MOVE, stroll(1.0)),
        GoalRow::new(3, LOOK, GoalKind::LookAtPlayer { range: 6.0 }),
        GoalRow::new(4, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[GoalRow::new(
        1,
        TARGET,
        kinds(MONSTERS_NOT_CREEPER, Sight::MustSee, TargetFilter::None),
    )],
    rules: SHORT_RANGE_RULES,
};

pub static STILL: GoalTable = GoalTable {
    goals: &[],
    targets: &[],
    rules: SHORT_RANGE_RULES,
};
