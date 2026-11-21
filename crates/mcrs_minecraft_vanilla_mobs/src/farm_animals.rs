use mcrs_minecraft_brain::BrainRules;
use mcrs_minecraft_brain::ControlFlags;
use mcrs_minecraft_brain::ItemTest;
use mcrs_minecraft_goals::GoalKind;
use mcrs_minecraft_goals::GoalRow;
use mcrs_minecraft_goals::GoalTable;
use mcrs_minecraft_goals::IDLE_RESET_RANGE;
use mcrs_minecraft_goals::Kinds;
use mcrs_minecraft_goals::Mates;
use mcrs_minecraft_goals::STROLL_IDLE_LIMIT;
use mcrs_minecraft_goals::Water;

use crate::CARROT_ON_A_STICK;
use crate::CHICKEN_FOOD;
use crate::COW_FOOD;
use crate::PIG_FOOD;
use crate::SHEEP_FOOD;

const MOVE: ControlFlags = ControlFlags::MOVE;
const LOOK: ControlFlags = ControlFlags::LOOK;
const JUMP: ControlFlags = ControlFlags::JUMP;
const MOVE_LOOK: ControlFlags = ControlFlags::MOVE.union(ControlFlags::LOOK);
const MOVE_LOOK_JUMP: ControlFlags = MOVE_LOOK.union(ControlFlags::JUMP);
const LOOK_RANGE: f32 = 6.0;

pub const ANIMAL_RULES: BrainRules = BrainRules {
    player_range: 10.0,
    idle_limit: STROLL_IDLE_LIMIT,
    idle_reset_range: IDLE_RESET_RANGE,
};

const fn breed() -> GoalKind {
    GoalKind::Breed {
        speed: 1.0,
        kinds: Kinds::Own,
        mates: Mates::InLove,
    }
}

const fn tempt(items: ItemTest, speed: f32) -> GoalKind {
    GoalKind::Tempt { items, speed }
}

const fn follow(speed: f32) -> GoalKind {
    GoalKind::FollowAdult {
        speed,
        kinds: Kinds::Own,
    }
}

const STROLL: GoalKind = GoalKind::Stroll {
    speed: 1.0,
    water: Water::Avoid,
};

const LOOK_AT_PLAYER: GoalKind = GoalKind::LookAtPlayer { range: LOOK_RANGE };

pub static PIG: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(1, MOVE, GoalKind::Panic { speed: 1.25 }),
        GoalRow::new(3, MOVE_LOOK, breed()),
        GoalRow::new(4, MOVE_LOOK, tempt(CARROT_ON_A_STICK, 1.2)),
        GoalRow::new(4, MOVE_LOOK, tempt(ItemTest::Tag(PIG_FOOD), 1.2)),
        GoalRow::new(5, ControlFlags::NONE, follow(1.1)),
        GoalRow::new(6, MOVE, STROLL),
        GoalRow::new(7, LOOK, LOOK_AT_PLAYER),
        GoalRow::new(8, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[],
    rules: ANIMAL_RULES,
};

pub static COW: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(1, MOVE, GoalKind::Panic { speed: 2.0 }),
        GoalRow::new(2, MOVE_LOOK, breed()),
        GoalRow::new(3, MOVE_LOOK, tempt(ItemTest::Tag(COW_FOOD), 1.25)),
        GoalRow::new(4, ControlFlags::NONE, follow(1.25)),
        GoalRow::new(5, MOVE, STROLL),
        GoalRow::new(6, LOOK, LOOK_AT_PLAYER),
        GoalRow::new(7, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[],
    rules: ANIMAL_RULES,
};

pub static SHEEP: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(1, MOVE, GoalKind::Panic { speed: 1.25 }),
        GoalRow::new(2, MOVE_LOOK, breed()),
        GoalRow::new(3, MOVE_LOOK, tempt(ItemTest::Tag(SHEEP_FOOD), 1.1)),
        GoalRow::new(4, ControlFlags::NONE, follow(1.1)),
        GoalRow::new(5, MOVE_LOOK_JUMP, GoalKind::EatGrass),
        GoalRow::new(6, MOVE, STROLL),
        GoalRow::new(7, LOOK, LOOK_AT_PLAYER),
        GoalRow::new(8, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[],
    rules: ANIMAL_RULES,
};

pub static CHICKEN: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(1, MOVE, GoalKind::Panic { speed: 1.4 }),
        GoalRow::new(2, MOVE_LOOK, breed()),
        GoalRow::new(3, MOVE_LOOK, tempt(ItemTest::Tag(CHICKEN_FOOD), 1.0)),
        GoalRow::new(4, ControlFlags::NONE, follow(1.1)),
        GoalRow::new(5, MOVE, STROLL),
        GoalRow::new(6, LOOK, LOOK_AT_PLAYER),
        GoalRow::new(7, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[],
    rules: ANIMAL_RULES,
};
