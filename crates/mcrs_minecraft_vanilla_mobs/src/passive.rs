use mcrs_minecraft_brain::BrainRules;
use mcrs_minecraft_brain::ControlFlags;
use mcrs_minecraft_brain::ItemTag;
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
use mcrs_minecraft_goals::Panicker;
use mcrs_minecraft_goals::STROLL_IDLE_LIMIT;
use mcrs_minecraft_goals::Sight;
use mcrs_minecraft_goals::StrollSearch;
use mcrs_minecraft_goals::TameRule;
use mcrs_minecraft_goals::TargetFilter;
use mcrs_minecraft_goals::TargetSet;

use crate::BEE_FOOD;
use crate::CAT_FOOD;
use crate::HORSE_TEMPT_ITEMS;
use crate::LLAMA_KIND;
use crate::MONSTERS;
use crate::OCELOT_FOOD;
use crate::PANDA_FOOD;
use crate::RABBIT_FOOD;
use crate::STRIDER_TEMPT_ITEMS;
use crate::TURTLE_FOOD;
use crate::TURTLE_KIND;
use crate::WOLF_KIND;
use crate::kind;

const MOVE: ControlFlags = ControlFlags::MOVE;
const LOOK: ControlFlags = ControlFlags::LOOK;
const JUMP: ControlFlags = ControlFlags::JUMP;
const TARGET: ControlFlags = ControlFlags::TARGET;
const MOVE_LOOK: ControlFlags = ControlFlags::MOVE.union(ControlFlags::LOOK);
const JUMP_MOVE: ControlFlags = ControlFlags::JUMP.union(ControlFlags::MOVE);
const NONE: ControlFlags = ControlFlags::NONE;

pub const RABBIT: KindSet = KindSet::of(&[kind(EntityType::Rabbit)]);
pub const CHICKEN_KIND: KindSet = KindSet::of(&[kind(EntityType::Chicken)]);
pub const FOX: KindSet = KindSet::of(&[kind(EntityType::Fox)]);
pub const HORSE_KIND: KindSet = KindSet::of(&[
    kind(EntityType::Horse),
    kind(EntityType::Donkey),
    kind(EntityType::Mule),
    kind(EntityType::SkeletonHorse),
    kind(EntityType::ZombieHorse),
]);
pub const FOX_PREY: KindSet = KindSet::of(&[
    kind(EntityType::Chicken),
    kind(EntityType::Rabbit),
    kind(EntityType::Cod),
    kind(EntityType::Salmon),
    kind(EntityType::TropicalFish),
    kind(EntityType::Pufferfish),
]);
pub const TRADER_THREATS: KindSet = KindSet::of(&[
    kind(EntityType::Zombie),
    kind(EntityType::Evoker),
    kind(EntityType::Vindicator),
    kind(EntityType::Vex),
    kind(EntityType::Pillager),
    kind(EntityType::Illusioner),
    kind(EntityType::Zoglin),
]);
pub const ZOMBIE_VILLAGER: KindSet = KindSet::of(&[kind(EntityType::ZombieVillager)]);

const RULES: BrainRules = BrainRules {
    player_range: 16.0,
    idle_limit: STROLL_IDLE_LIMIT,
    idle_reset_range: IDLE_RESET_RANGE,
};

const FOX_RULES: BrainRules = BrainRules {
    player_range: 24.0,
    idle_limit: STROLL_IDLE_LIMIT,
    idle_reset_range: IDLE_RESET_RANGE,
};

const fn breed(speed: f32, kinds: Kinds, mates: Mates) -> GoalKind {
    GoalKind::Breed {
        speed,
        kinds,
        mates,
    }
}

const fn tempt(tag: ItemTag, speed: f32) -> GoalKind {
    GoalKind::Tempt {
        items: ItemTest::Tag(tag),
        speed,
    }
}

const fn follow(speed: f32, kinds: Kinds) -> GoalKind {
    GoalKind::FollowAdult { speed, kinds }
}

const fn stroll(speed: f32) -> GoalKind {
    GoalKind::Stroll {
        speed,
        search: StrollSearch::Default,
    }
}

const fn look(range: f32) -> GoalKind {
    GoalKind::LookAtPlayer { range }
}

const fn melee(speed: f32, pursuit: Pursuit) -> GoalKind {
    GoalKind::Melee {
        speed,
        pursuit,
        interval: 20,
    }
}

const fn avoid(set: KindSet, distance: f32, walk: f32, sprint: f32) -> GoalKind {
    GoalKind::Avoid {
        set: TargetSet::Kinds(set),
        distance,
        walk,
        sprint,
        extra: AvoidExtra::None,
    }
}

const fn hunt(set: KindSet, filter: TargetFilter, tame: TameRule) -> GoalKind {
    GoalKind::NearestTarget {
        set: TargetSet::Kinds(set),
        interval: 10,
        sight: Sight::MustSee,
        filter,
        follow_scale: 1.0,
        tame,
    }
}

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

pub static CAT_TABLE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(1, JUMP, GoalKind::Float),
        GoalRow::new(
            1,
            MOVE,
            GoalKind::Panic {
                speed: 1.5,
                who: Panicker::Any,
            },
        ),
        GoalRow::new(1, JUMP_MOVE, GoalKind::Sit),
        GoalRow::new(3, MOVE_LOOK, tempt(CAT_FOOD, 0.6)),
        GoalRow::new(
            6,
            MOVE_LOOK,
            GoalKind::FollowOwner {
                speed: 1.0,
                start: 10.0,
                stop: 5.0,
            },
        ),
        GoalRow::new(
            8,
            MOVE,
            GoalKind::Avoid {
                set: TargetSet::Players,
                distance: 16.0,
                walk: 0.8,
                sprint: 1.33,
                extra: AvoidExtra::WildOnly,
            },
        ),
        GoalRow::new(8, MOVE_LOOK, breed(0.8, Kinds::Own, Mates::TameInLove)),
        GoalRow::new(9, MOVE, stroll(0.8)),
        GoalRow::new(10, JUMP_MOVE, GoalKind::Leap { vertical: 0.3 }),
        GoalRow::new(11, MOVE_LOOK, melee(1.0, Pursuit::Seen)),
        GoalRow::new(11, LOOK, look(10.0)),
    ],
    targets: &[
        GoalRow::new(
            1,
            TARGET,
            hunt(RABBIT, TargetFilter::None, TameRule::NotTame),
        ),
        GoalRow::new(
            1,
            TARGET,
            hunt(TURTLE_KIND, TargetFilter::BabyOnLand, TameRule::NotTame),
        ),
    ],
    rules: RULES,
};

pub static OCELOT_TABLE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(1, JUMP, GoalKind::Float),
        GoalRow::new(1, MOVE_LOOK, tempt(OCELOT_FOOD, 0.6)),
        GoalRow::new(3, JUMP_MOVE, GoalKind::Leap { vertical: 0.3 }),
        GoalRow::new(4, MOVE_LOOK, melee(1.0, Pursuit::Seen)),
        GoalRow::new(5, MOVE_LOOK, breed(0.8, Kinds::Own, Mates::InLove)),
        GoalRow::new(6, MOVE, stroll(0.8)),
        GoalRow::new(
            7,
            MOVE,
            GoalKind::Avoid {
                set: TargetSet::Players,
                distance: 16.0,
                walk: 0.8,
                sprint: 1.33,
                extra: AvoidExtra::None,
            },
        ),
        GoalRow::new(9, LOOK, look(10.0)),
    ],
    targets: &[
        GoalRow::new(
            1,
            TARGET,
            hunt(CHICKEN_KIND, TargetFilter::None, TameRule::Any),
        ),
        GoalRow::new(
            1,
            TARGET,
            hunt(TURTLE_KIND, TargetFilter::BabyOnLand, TameRule::Any),
        ),
    ],
    rules: RULES,
};

pub static RABBIT_TABLE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(1, JUMP, GoalKind::Float),
        GoalRow::new(
            1,
            MOVE,
            GoalKind::Panic {
                speed: 2.2,
                who: Panicker::Any,
            },
        ),
        GoalRow::new(2, MOVE_LOOK, breed(0.8, Kinds::Own, Mates::InLove)),
        GoalRow::new(3, MOVE_LOOK, tempt(RABBIT_FOOD, 1.0)),
        GoalRow::new(
            4,
            MOVE,
            GoalKind::Avoid {
                set: TargetSet::Players,
                distance: 8.0,
                walk: 2.2,
                sprint: 2.2,
                extra: AvoidExtra::None,
            },
        ),
        GoalRow::new(4, MOVE, avoid(WOLF_KIND, 10.0, 2.2, 2.2)),
        GoalRow::new(4, MOVE, avoid(MONSTERS, 4.0, 2.2, 2.2)),
        GoalRow::new(6, MOVE, stroll(0.6)),
        GoalRow::new(11, LOOK, look(10.0)),
    ],
    targets: &[],
    rules: RULES,
};

pub static POLAR_BEAR_TABLE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(1, MOVE_LOOK, melee(1.25, Pursuit::Unseen)),
        GoalRow::new(
            1,
            MOVE,
            GoalKind::Panic {
                speed: 2.0,
                who: Panicker::Baby,
            },
        ),
        GoalRow::new(4, NONE, follow(1.25, Kinds::Own)),
        GoalRow::new(5, MOVE, stroll(1.0)),
        GoalRow::new(6, LOOK, look(6.0)),
        GoalRow::new(7, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[
        GoalRow::new(
            1,
            TARGET,
            GoalKind::HurtBy {
                alert: Alert::SameKind,
                ignored: KindSet::NONE,
            },
        ),
        GoalRow::new(3, TARGET, players(TargetFilter::AngryAt)),
        GoalRow::new(4, TARGET, hunt(FOX, TargetFilter::None, TameRule::Any)),
        GoalRow::new(5, NONE, GoalKind::ResetAnger { alert: Alert::None }),
    ],
    rules: BrainRules {
        player_range: 20.0,
        idle_limit: STROLL_IDLE_LIMIT,
        idle_reset_range: IDLE_RESET_RANGE,
    },
};

pub static PANDA_TABLE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(
            2,
            MOVE,
            GoalKind::Panic {
                speed: 2.0,
                who: Panicker::Baby,
            },
        ),
        GoalRow::new(2, MOVE_LOOK, melee(1.0, Pursuit::Seen)),
        GoalRow::new(3, MOVE_LOOK, breed(1.0, Kinds::Own, Mates::InLove)),
        GoalRow::new(5, MOVE_LOOK, tempt(PANDA_FOOD, 1.0)),
        GoalRow::new(7, NONE, follow(1.25, Kinds::Own)),
        GoalRow::new(8, MOVE, stroll(1.0)),
        GoalRow::new(9, LOOK, look(6.0)),
        GoalRow::new(10, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[GoalRow::new(
        1,
        TARGET,
        GoalKind::HurtBy {
            alert: Alert::SameKind,
            ignored: KindSet::NONE,
        },
    )],
    rules: RULES,
};

pub static HORSE_TABLE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(
            1,
            MOVE,
            GoalKind::Panic {
                speed: 1.2,
                who: Panicker::Any,
            },
        ),
        GoalRow::new(
            2,
            MOVE_LOOK,
            breed(1.0, Kinds::Set(HORSE_KIND), Mates::TameAdults),
        ),
        GoalRow::new(4, MOVE_LOOK, tempt(HORSE_TEMPT_ITEMS, 1.25)),
        GoalRow::new(6, NONE, follow(1.0, Kinds::Set(HORSE_KIND))),
        GoalRow::new(7, MOVE, stroll(0.7)),
        GoalRow::new(8, LOOK, look(6.0)),
        GoalRow::new(9, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[],
    rules: RULES,
};

pub static UNDEAD_HORSE_TABLE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(
            1,
            MOVE,
            GoalKind::Panic {
                speed: 1.2,
                who: Panicker::Any,
            },
        ),
        GoalRow::new(6, NONE, follow(1.0, Kinds::Set(HORSE_KIND))),
        GoalRow::new(7, MOVE, stroll(0.7)),
        GoalRow::new(8, LOOK, look(6.0)),
        GoalRow::new(9, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[],
    rules: RULES,
};

pub static TRADER_LLAMA_TABLE: GoalTable = GoalTable {
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
        GoalRow::new(
            3,
            MOVE,
            GoalKind::Panic {
                speed: 1.2,
                who: Panicker::Any,
            },
        ),
        GoalRow::new(
            4,
            MOVE_LOOK,
            breed(1.0, Kinds::Set(LLAMA_KIND), Mates::TameAdults),
        ),
        GoalRow::new(5, MOVE_LOOK, tempt(crate::LLAMA_TEMPT_ITEMS, 1.25)),
        GoalRow::new(6, NONE, follow(1.0, Kinds::Set(LLAMA_KIND))),
        GoalRow::new(7, MOVE, stroll(0.7)),
        GoalRow::new(8, LOOK, look(6.0)),
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
    rules: RULES,
};

pub static WANDERING_TRADER_TABLE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(
            1,
            MOVE,
            GoalKind::Panic {
                speed: 0.5,
                who: Panicker::Any,
            },
        ),
        GoalRow::new(1, MOVE, avoid(TRADER_THREATS, 8.0, 0.5, 0.5)),
        GoalRow::new(1, MOVE, avoid(ZOMBIE_VILLAGER, 8.0, 0.5, 0.5)),
        GoalRow::new(
            2,
            MOVE,
            GoalKind::Stroll {
                speed: 0.35,
                search: StrollSearch::Land,
            },
        ),
        GoalRow::new(2, LOOK, look(8.0)),
    ],
    targets: &[],
    rules: RULES,
};

pub static FOX_TABLE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(0, JUMP, GoalKind::Float),
        GoalRow::new(
            0,
            MOVE,
            GoalKind::Panic {
                speed: 2.2,
                who: Panicker::Any,
            },
        ),
        GoalRow::new(0, MOVE_LOOK, breed(1.0, Kinds::Own, Mates::InLove)),
        GoalRow::new(4, MOVE_LOOK, melee(1.2, Pursuit::Unseen)),
        GoalRow::new(5, NONE, follow(1.25, Kinds::Own)),
        GoalRow::new(8, MOVE, stroll(1.0)),
        GoalRow::new(11, LOOK, look(24.0)),
    ],
    targets: &[
        GoalRow::new(4, TARGET, hunt(FOX_PREY, TargetFilter::None, TameRule::Any)),
        GoalRow::new(
            6,
            TARGET,
            hunt(TURTLE_KIND, TargetFilter::BabyOnLand, TameRule::Any),
        ),
    ],
    rules: FOX_RULES,
};

pub static TURTLE_TABLE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(
            0,
            MOVE,
            GoalKind::Panic {
                speed: 1.2,
                who: Panicker::Any,
            },
        ),
        GoalRow::new(1, MOVE_LOOK, breed(1.0, Kinds::Own, Mates::InLove)),
        GoalRow::new(2, MOVE_LOOK, tempt(TURTLE_FOOD, 1.1)),
        GoalRow::new(8, MOVE, stroll(1.0)),
        GoalRow::new(9, LOOK, look(8.0)),
    ],
    targets: &[],
    rules: RULES,
};

pub static STRIDER_TABLE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(
            1,
            MOVE,
            GoalKind::Panic {
                speed: 1.65,
                who: Panicker::Any,
            },
        ),
        GoalRow::new(2, MOVE_LOOK, breed(1.0, Kinds::Own, Mates::InLove)),
        GoalRow::new(3, MOVE_LOOK, tempt(STRIDER_TEMPT_ITEMS, 1.4)),
        GoalRow::new(5, NONE, follow(1.0, Kinds::Own)),
        GoalRow::new(7, MOVE, stroll(1.0)),
        GoalRow::new(8, LOOK, look(8.0)),
        GoalRow::new(8, MOVE_LOOK, GoalKind::LookAround),
    ],
    targets: &[],
    rules: RULES,
};

pub static PARROT_TABLE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(
            0,
            MOVE,
            GoalKind::Panic {
                speed: 1.25,
                who: Panicker::Any,
            },
        ),
        GoalRow::new(1, JUMP_MOVE, GoalKind::Sit),
        GoalRow::new(
            2,
            MOVE_LOOK,
            GoalKind::FollowOwner {
                speed: 1.0,
                start: 5.0,
                stop: 1.0,
            },
        ),
        GoalRow::new(3, LOOK, look(8.0)),
    ],
    targets: &[],
    rules: RULES,
};

pub static BEE_TABLE: GoalTable = GoalTable {
    goals: &[
        GoalRow::new(1, MOVE_LOOK, breed(1.0, Kinds::Own, Mates::InLove)),
        GoalRow::new(2, MOVE_LOOK, tempt(BEE_FOOD, 1.25)),
        GoalRow::new(5, NONE, follow(1.25, Kinds::Own)),
    ],
    targets: &[],
    rules: RULES,
};
