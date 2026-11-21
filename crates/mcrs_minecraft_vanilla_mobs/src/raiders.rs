use mcrs_minecraft_brain::ControlFlags;
use mcrs_minecraft_brain::KindSet;
use mcrs_minecraft_brain::Pursuit;
use mcrs_minecraft_entity::entity_type_keys::EntityType;
use mcrs_minecraft_goals::Alert;
use mcrs_minecraft_goals::AvoidExtra;
use mcrs_minecraft_goals::GoalKind;
use mcrs_minecraft_goals::GoalRow;
use mcrs_minecraft_goals::GoalTable;
use mcrs_minecraft_goals::Sight;
use mcrs_minecraft_goals::TargetFilter;
use mcrs_minecraft_goals::TargetSet;

use crate::IRON_GOLEM;
use crate::MONSTER_RULES;
use crate::MOVE_LOOK;
use crate::SHORT_RANGE_RULES;
use crate::hurt_by;
use crate::kind;
use crate::kinds;
use crate::melee;
use crate::players;
use crate::stroll;

const MOVE: ControlFlags = ControlFlags::MOVE;
const LOOK: ControlFlags = ControlFlags::LOOK;
const JUMP: ControlFlags = ControlFlags::JUMP;
const TARGET: ControlFlags = ControlFlags::TARGET;

pub const RAIDERS: KindSet = KindSet::of(&[
    kind(EntityType::Pillager),
    kind(EntityType::Vindicator),
    kind(EntityType::Evoker),
    kind(EntityType::Illusioner),
    kind(EntityType::Ravager),
    kind(EntityType::Witch),
]);

pub const RAIDER_PREY: KindSet = KindSet::of(&[
    kind(EntityType::Villager),
    kind(EntityType::WanderingTrader),
]);

const RAID_ALERT: GoalKind = hurt_by(Alert::Kinds(&RAIDERS), KindSet::NONE);
const VILLAGERS: GoalKind = kinds(RAIDER_PREY, Sight::Any, TargetFilter::None);
const GOLEMS: GoalKind = kinds(IRON_GOLEM, Sight::MustSee, TargetFilter::None);

const fn look(range: f32) -> GoalKind {
    GoalKind::LookAtPlayer { range }
}

pub static PILLAGER: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(
            3,
            MOVE_LOOK,
            GoalKind::Ranged {
                speed: 1.0,
                min_interval: 40,
                max_interval: 40,
                range: 8.0,
            },
        ),
        GoalRow::new(8, MOVE, stroll(0.6)),
        GoalRow::new(9, LOOK, look(15.0)),
        GoalRow::new(10, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(1, TARGET, RAID_ALERT),
        GoalRow::new(2, TARGET, players(TargetFilter::None)),
        GoalRow::new(3, TARGET, VILLAGERS),
        GoalRow::new(3, TARGET, GOLEMS),
    ],
    rules: MONSTER_RULES,
};

pub static VINDICATOR: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(4, MOVE_LOOK, melee(1.0, Pursuit::Seen)),
        GoalRow::new(8, MOVE, stroll(0.6)),
        GoalRow::new(9, LOOK, look(3.0)),
        GoalRow::new(10, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(1, TARGET, RAID_ALERT),
        GoalRow::new(2, TARGET, players(TargetFilter::None)),
        GoalRow::new(3, TARGET, VILLAGERS),
        GoalRow::new(3, TARGET, GOLEMS),
    ],
    rules: SHORT_RANGE_RULES,
};

pub static EVOKER: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(
            2,
            MOVE,
            GoalKind::Avoid {
                set: TargetSet::Players,
                distance: 8.0,
                walk: 0.6,
                sprint: 1.0,
                extra: AvoidExtra::None,
            },
        ),
        GoalRow::new(8, MOVE, stroll(0.6)),
        GoalRow::new(9, LOOK, look(3.0)),
        GoalRow::new(10, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(1, TARGET, RAID_ALERT),
        GoalRow::new(2, TARGET, players(TargetFilter::None)),
        GoalRow::new(3, TARGET, VILLAGERS),
        GoalRow::new(3, TARGET, GOLEMS),
    ],
    rules: SHORT_RANGE_RULES,
};

pub static ILLUSIONER: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(
            6,
            MOVE_LOOK,
            GoalKind::Ranged {
                speed: 0.5,
                min_interval: 20,
                max_interval: 20,
                range: 15.0,
            },
        ),
        GoalRow::new(8, MOVE, stroll(0.6)),
        GoalRow::new(9, LOOK, look(3.0)),
        GoalRow::new(10, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(1, TARGET, RAID_ALERT),
        GoalRow::new(2, TARGET, players(TargetFilter::None)),
        GoalRow::new(3, TARGET, VILLAGERS),
        GoalRow::new(3, TARGET, GOLEMS),
    ],
    rules: SHORT_RANGE_RULES,
};

pub static RAVAGER: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(4, MOVE_LOOK, melee(1.0, Pursuit::Unseen)),
        GoalRow::new(5, MOVE, stroll(0.4)),
        GoalRow::new(6, LOOK, look(6.0)),
        GoalRow::new(10, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(2, TARGET, RAID_ALERT),
        GoalRow::new(3, TARGET, players(TargetFilter::None)),
        GoalRow::new(4, TARGET, VILLAGERS),
        GoalRow::new(4, TARGET, GOLEMS),
    ],
    rules: MONSTER_RULES,
};
