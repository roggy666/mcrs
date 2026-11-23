use mcrs_minecraft_brain::BrainRules;
use mcrs_minecraft_brain::ControlFlags;
use mcrs_minecraft_brain::GoalSpec;
use mcrs_minecraft_brain::ItemTest;
use mcrs_minecraft_brain::KindSet;
use mcrs_minecraft_brain::Pursuit;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GoalRow {
    pub priority: u8,
    pub flags: ControlFlags,
    pub goal: GoalKind,
}

impl GoalRow {
    #[must_use]
    pub const fn new(priority: u8, flags: ControlFlags, goal: GoalKind) -> Self {
        Self {
            priority,
            flags,
            goal,
        }
    }

    #[must_use]
    pub const fn spec(&self) -> GoalSpec {
        GoalSpec::new(self.priority, self.flags)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GoalTable {
    pub goals: &'static [GoalRow],
    pub targets: &'static [GoalRow],
    pub rules: BrainRules,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GoalKind {
    Float,
    Swell,
    RestrictSun,
    FleeSun {
        speed: f32,
    },
    Panic {
        speed: f32,
        who: Panicker,
    },
    Breed {
        speed: f32,
        kinds: Kinds,
        mates: Mates,
    },
    Tempt {
        items: ItemTest,
        speed: f32,
    },
    FollowAdult {
        speed: f32,
        kinds: Kinds,
    },
    EatGrass,
    Stroll {
        speed: f32,
        search: StrollSearch,
    },
    LookAtPlayer {
        range: f32,
    },
    LookAround,
    Wander {
        speed: f32,
    },
    Melee {
        speed: f32,
        pursuit: Pursuit,
        interval: u8,
    },
    Leap {
        vertical: f32,
    },
    Avoid {
        set: TargetSet,
        distance: f32,
        walk: f32,
        sprint: f32,
        extra: AvoidExtra,
    },
    Ranged {
        speed: f32,
        min_interval: u16,
        max_interval: u16,
        range: f32,
    },
    FollowOwner {
        speed: f32,
        start: f32,
        stop: f32,
    },
    Sit,
    Beg {
        range: f32,
        items: ItemTest,
    },
    NearestTarget {
        set: TargetSet,
        interval: u8,
        sight: Sight,
        filter: TargetFilter,
        follow_scale: f32,
        tame: TameRule,
    },
    HurtBy {
        alert: Alert,
        ignored: KindSet,
    },
    OwnerHurtBy,
    OwnerHurt,
    ResetAnger {
        alert: Alert,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kinds {
    Own,
    Set(KindSet),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mates {
    InLove,
    TameInLove,
    TameAdults,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StrollSearch {
    Default,
    Land,
    Fly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Panicker {
    Any,
    Baby,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AvoidExtra {
    None,
    WildOnly,
    LlamaStrength,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TargetSet {
    Players,
    Kinds(KindSet),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sight {
    MustSee,
    Any,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TargetFilter {
    None,
    BabyOnLand,
    AngryAt,
    NoOwner,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TameRule {
    Any,
    NotTame,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Alert {
    None,
    SameKind,
    Kinds(&'static KindSet),
}
