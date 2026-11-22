use alloc::vec::Vec;

use crate::MobFlags;
use crate::MobKind;
use crate::MobView;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KindData {
    pub adult: Dimensions,
    pub baby: Dimensions,
    pub flags: KindFlags,
    pub head_turn_speed: f32,
    pub pitch_limit: f32,
    pub head_yaw_limit: f32,
    pub tempt_range: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dimensions {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
}

impl Dimensions {
    #[must_use]
    pub const fn new(width: f32, height: f32, eye_height: f32) -> Self {
        Self {
            width,
            height,
            eye_height,
        }
    }

    #[must_use]
    pub const fn halved(self) -> Self {
        Self {
            width: self.width * 0.5,
            height: self.height * 0.5,
            eye_height: self.eye_height * 0.5,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct KindFlags(u8);

impl KindFlags {
    pub const NONE: Self = Self(0);
    pub const AGEABLE: Self = Self(1);
    pub const ANIMAL: Self = Self(1 << 1);
    pub const FIRE_IMMUNE: Self = Self(1 << 2);
    pub const CAN_BE_TAME: Self = Self(1 << 3);
    pub const PLAYER: Self = Self(1 << 4);
    pub const UNTARGETABLE: Self = Self(1 << 5);
    pub const PANIC_ENVIRONMENTAL_ONLY: Self = Self(1 << 6);

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl KindData {
    pub const DEFAULT_HEAD_TURN_SPEED: f32 = 10.0;
    pub const DEFAULT_PITCH_LIMIT: f32 = 40.0;
    pub const DEFAULT_HEAD_YAW_LIMIT: f32 = 75.0;
    pub const DEFAULT_TEMPT_RANGE: f32 = 10.0;

    #[must_use]
    pub const fn new(adult: Dimensions, flags: KindFlags) -> Self {
        Self {
            adult,
            baby: adult.halved(),
            flags,
            head_turn_speed: Self::DEFAULT_HEAD_TURN_SPEED,
            pitch_limit: Self::DEFAULT_PITCH_LIMIT,
            head_yaw_limit: Self::DEFAULT_HEAD_YAW_LIMIT,
            tempt_range: Self::DEFAULT_TEMPT_RANGE,
        }
    }

    #[must_use]
    pub const fn with_baby(self, baby: Dimensions) -> Self {
        Self { baby, ..self }
    }

    #[must_use]
    pub const fn with_head(self, turn_speed: f32, pitch_limit: f32, yaw_limit: f32) -> Self {
        Self {
            head_turn_speed: turn_speed,
            pitch_limit,
            head_yaw_limit: yaw_limit,
            ..self
        }
    }

    #[must_use]
    pub const fn dimensions(&self, baby: bool) -> Dimensions {
        if baby { self.baby } else { self.adult }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct KindTable {
    rows: Vec<Option<KindData>>,
}

impl KindTable {
    #[must_use]
    pub fn new(rows: impl IntoIterator<Item = (MobKind, KindData)>) -> Self {
        let mut table = Self::default();
        for (kind, data) in rows {
            let at = usize::from(kind.0);
            if table.rows.len() <= at {
                table.rows.resize(at.saturating_add(1), None);
            }
            if let Some(slot) = table.rows.get_mut(at) {
                *slot = Some(data);
            }
        }
        table
    }

    #[must_use]
    pub fn get(&self, kind: MobKind) -> Option<&KindData> {
        self.rows.get(usize::from(kind.0))?.as_ref()
    }

    #[must_use]
    pub fn dimensions(&self, view: &MobView) -> Dimensions {
        self.get(view.kind).map_or(Dimensions::UNKNOWN, |data| {
            data.dimensions(view.flags.contains(MobFlags::BABY))
        })
    }

    #[must_use]
    pub fn has(&self, kind: MobKind, flags: KindFlags) -> bool {
        self.get(kind)
            .is_some_and(|data| data.flags.contains(flags))
    }
}

impl Dimensions {
    pub const UNKNOWN: Self = Self::new(0.6, 1.8, 1.62);
}
