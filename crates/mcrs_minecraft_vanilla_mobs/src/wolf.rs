use mcrs_minecraft_brain::BrainRules;
use mcrs_minecraft_brain::ControlFlags;
use mcrs_minecraft_brain::ItemTest;
use mcrs_minecraft_brain::KindSet;
use mcrs_minecraft_brain::Pursuit;
use mcrs_minecraft_entity::entity_type_keys::EntityType;
use mcrs_minecraft_goals::Alert;
use mcrs_minecraft_goals::AvoidExtra;
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

use crate::WOLF_INTERESTING;
use crate::kind;

const MOVE: ControlFlags = ControlFlags::MOVE;
const LOOK: ControlFlags = ControlFlags::LOOK;
const JUMP: ControlFlags = ControlFlags::JUMP;
const TARGET: ControlFlags = ControlFlags::TARGET;
const MOVE_LOOK: ControlFlags = ControlFlags::MOVE.union(ControlFlags::LOOK);
const JUMP_MOVE: ControlFlags = ControlFlags::JUMP.union(ControlFlags::MOVE);

pub const LLAMA_KIND: KindSet =
    KindSet::of(&[kind(EntityType::Llama), kind(EntityType::TraderLlama)]);
pub const PREY: KindSet = KindSet::of(&[
    kind(EntityType::Sheep),
    kind(EntityType::Rabbit),
    kind(EntityType::Fox),
]);
pub const SKELETON_KIND: KindSet = KindSet::of(&[
    kind(EntityType::Skeleton),
    kind(EntityType::Stray),
    kind(EntityType::WitherSkeleton),
    kind(EntityType::Bogged),
    kind(EntityType::Parched),
]);
pub const TURTLE: KindSet = KindSet::of(&[kind(EntityType::Turtle)]);

pub const WOLF_RULES: BrainRules = BrainRules {
    player_range: 16.0,
    idle_limit: STROLL_IDLE_LIMIT,
    idle_reset_range: IDLE_RESET_RANGE,
};

const fn hunt(set: KindSet, filter: TargetFilter, tame: TameRule, sight: Sight) -> GoalKind {
    GoalKind::NearestTarget {
        set: TargetSet::Kinds(set),
        interval: 10,
        sight,
        filter,
        follow_scale: 1.0,
        tame,
    }
}

pub static WOLF: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(1, JUMP, GoalKind::Float),
        GoalRow::new(1, MOVE, GoalKind::Panic { speed: 1.5 }),
        GoalRow::new(2, JUMP_MOVE, GoalKind::Sit),
        GoalRow::new(
            3,
            MOVE,
            GoalKind::Avoid {
                kinds: LLAMA_KIND,
                distance: 24.0,
                walk: 1.5,
                sprint: 1.5,
                extra: AvoidExtra::LlamaStrength,
            },
        ),
        GoalRow::new(4, JUMP_MOVE, GoalKind::Leap { vertical: 0.4 }),
        GoalRow::new(
            5,
            MOVE_LOOK,
            GoalKind::Melee {
                speed: 1.0,
                pursuit: Pursuit::Unseen,
                interval: 20,
            },
        ),
        GoalRow::new(
            6,
            MOVE_LOOK,
            GoalKind::FollowOwner {
                speed: 1.0,
                start: 10.0,
                stop: 2.0,
            },
        ),
        GoalRow::new(
            7,
            MOVE_LOOK,
            GoalKind::Breed {
                speed: 1.0,
                kinds: Kinds::Own,
                mates: Mates::TameInLove,
            },
        ),
        GoalRow::new(
            8,
            MOVE,
            GoalKind::Stroll {
                speed: 1.0,
                water: Water::Avoid,
            },
        ),
        GoalRow::new(
            9,
            LOOK,
            GoalKind::Beg {
                range: 8.0,
                items: ItemTest::Tag(WOLF_INTERESTING),
            },
        ),
        GoalRow::new(10, LOOK, GoalKind::LookAtPlayer { range: 8.0 }),
        GoalRow::new(10, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(1, TARGET, GoalKind::OwnerHurtBy),
        GoalRow::new(2, TARGET, GoalKind::OwnerHurt),
        GoalRow::new(
            3,
            TARGET,
            GoalKind::HurtBy {
                alert: Alert::SameKind,
                ignored: KindSet::NONE,
            },
        ),
        GoalRow::new(
            4,
            TARGET,
            GoalKind::NearestTarget {
                set: TargetSet::Players,
                interval: 10,
                sight: Sight::MustSee,
                filter: TargetFilter::AngryAt,
                follow_scale: 1.0,
                tame: TameRule::Any,
            },
        ),
        GoalRow::new(
            5,
            TARGET,
            hunt(PREY, TargetFilter::None, TameRule::NotTame, Sight::MustSee),
        ),
        GoalRow::new(
            6,
            TARGET,
            hunt(
                TURTLE,
                TargetFilter::BabyOnLand,
                TameRule::NotTame,
                Sight::MustSee,
            ),
        ),
        GoalRow::new(
            7,
            TARGET,
            hunt(SKELETON_KIND, TargetFilter::None, TameRule::Any, Sight::Any),
        ),
        GoalRow::new(
            8,
            ControlFlags::NONE,
            GoalKind::ResetAnger {
                alert: Alert::SameKind,
            },
        ),
    ],
    rules: WOLF_RULES,
};
