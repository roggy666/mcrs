use mcrs_minecraft_brain::BrainRules;
use mcrs_minecraft_brain::ControlFlags;
use mcrs_minecraft_brain::ItemTest;
use mcrs_minecraft_brain::KindSet;
use mcrs_minecraft_entity::entity_type_keys::EntityType;
use mcrs_minecraft_goals::Alert;
use mcrs_minecraft_goals::GoalKind;
use mcrs_minecraft_goals::GoalRow;
use mcrs_minecraft_goals::GoalTable;
use mcrs_minecraft_goals::IDLE_RESET_RANGE;
use mcrs_minecraft_goals::Kinds;
use mcrs_minecraft_goals::Mates;
use mcrs_minecraft_goals::STROLL_IDLE_LIMIT;
use mcrs_minecraft_goals::Sight;
use mcrs_minecraft_goals::TameRule;
use mcrs_minecraft_goals::TargetFilter;
use mcrs_minecraft_goals::TargetSet;
use mcrs_minecraft_goals::Water;

use crate::LLAMA_KIND;
use crate::LLAMA_TEMPT_ITEMS;
use crate::kind;

const MOVE: ControlFlags = ControlFlags::MOVE;
const LOOK: ControlFlags = ControlFlags::LOOK;
const JUMP: ControlFlags = ControlFlags::JUMP;
const TARGET: ControlFlags = ControlFlags::TARGET;
const MOVE_LOOK: ControlFlags = ControlFlags::MOVE.union(ControlFlags::LOOK);

pub const WOLF_KIND: KindSet = KindSet::of(&[kind(EntityType::Wolf)]);

pub const LLAMA_RULES: BrainRules = BrainRules {
    player_range: 10.0,
    idle_limit: STROLL_IDLE_LIMIT,
    idle_reset_range: IDLE_RESET_RANGE,
};

pub static LLAMA: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(
            3,
            MOVE_LOOK,
            GoalKind::Ranged {
                speed: 1.25,
                min_interval: 40,
                max_interval: 40,
                range: 20.0,
            },
        ),
        GoalRow::new(3, MOVE, GoalKind::Panic { speed: 1.2 }),
        GoalRow::new(
            4,
            MOVE_LOOK,
            GoalKind::Breed {
                speed: 1.0,
                kinds: Kinds::Set(LLAMA_KIND),
                mates: Mates::TameAdultsAtFullHealth,
            },
        ),
        GoalRow::new(
            5,
            MOVE_LOOK,
            GoalKind::Tempt {
                items: ItemTest::Tag(LLAMA_TEMPT_ITEMS),
                speed: 1.25,
            },
        ),
        GoalRow::new(
            6,
            ControlFlags::NONE,
            GoalKind::FollowAdult {
                speed: 1.0,
                kinds: Kinds::Set(LLAMA_KIND),
            },
        ),
        GoalRow::new(
            7,
            MOVE,
            GoalKind::Stroll {
                speed: 0.7,
                water: Water::Avoid,
            },
        ),
        GoalRow::new(8, LOOK, GoalKind::LookAtPlayer { range: 6.0 }),
        GoalRow::new(9, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(
            1,
            TARGET,
            GoalKind::HurtBy {
                alert: Alert::None,
                ignored: KindSet::NONE,
            },
        ),
        GoalRow::new(
            2,
            TARGET,
            GoalKind::NearestTarget {
                set: TargetSet::Kinds(WOLF_KIND),
                interval: 16,
                sight: Sight::Any,
                filter: TargetFilter::NoOwner,
                follow_scale: 0.25,
                tame: TameRule::Any,
            },
        ),
    ],
    rules: LLAMA_RULES,
};
