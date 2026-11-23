use glam::DVec3;
use mcrs_voxel::BlockPos;

use crate::MalusTable;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NavFlags(u8);

impl NavFlags {
    pub const NONE: Self = Self(0);
    pub const CAN_OPEN_DOORS: Self = Self(1);
    pub const CAN_PASS_DOORS: Self = Self(1 << 1);
    pub const CAN_FLOAT: Self = Self(1 << 2);
    pub const CAN_WALK_OVER_FENCES: Self = Self(1 << 3);
    pub const DEFAULT: Self = Self::CAN_PASS_DOORS;

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NavKind {
    Ground,
    Flying {
        starts: [BlockPos; crate::fly::START_CANDIDATES],
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MobPath {
    pub position: DVec3,
    pub min: DVec3,
    pub max: DVec3,
    pub on_ground: bool,
    pub in_liquid: bool,
    pub width: f32,
    pub height: f32,
    pub step_height: f32,
    pub malus: MalusTable,
    pub fall_limit: i32,
    pub flags: NavFlags,
    pub nav: NavKind,
}

impl MobPath {
    pub const DEFAULT_FALL_LIMIT: i32 = 3;

    #[must_use]
    pub fn standing(position: DVec3, width: f32, height: f32, step_height: f32) -> Self {
        let half = f64::from(width) / 2.0;
        Self {
            position,
            min: DVec3::new(position.x - half, position.y, position.z - half),
            max: DVec3::new(
                position.x + half,
                position.y + f64::from(height),
                position.z + half,
            ),
            on_ground: true,
            in_liquid: false,
            width,
            height,
            step_height,
            malus: MalusTable::default(),
            fall_limit: Self::DEFAULT_FALL_LIMIT,
            flags: NavFlags::DEFAULT,
            nav: NavKind::Ground,
        }
    }

    #[must_use]
    pub fn block_pos(&self) -> BlockPos {
        BlockPos::from(self.position)
    }

    #[must_use]
    pub fn cells_wide(&self) -> i32 {
        libm::floorf(self.width + 1.0) as i32
    }

    #[must_use]
    pub fn cells_high(&self) -> i32 {
        libm::floorf(self.height + 1.0) as i32
    }

    #[must_use]
    pub const fn flies(&self) -> bool {
        matches!(self.nav, NavKind::Flying { .. })
    }

    #[must_use]
    pub fn can(&self, flag: NavFlags) -> bool {
        self.flags.contains(flag)
    }

    #[must_use]
    pub fn jump_height(&self) -> f64 {
        f64::from(self.step_height).max(1.125)
    }
}
