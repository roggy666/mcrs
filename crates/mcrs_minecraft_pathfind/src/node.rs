use glam::DVec3;
use mcrs_minecraft_path_type::PathType;
use mcrs_voxel::BlockPos;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Node {
    pub(crate) pos: BlockPos,
    pub(crate) g: f32,
    pub(crate) h: f32,
    pub(crate) f: f32,
    pub(crate) walked: f32,
    pub(crate) kind: PathType,
    pub(crate) malus: f32,
    pub(crate) previous: Option<u32>,
    pub(crate) closed: bool,
    pub(crate) heap: Option<u32>,
}

impl Node {
    pub(crate) const fn new(pos: BlockPos) -> Self {
        Self {
            pos,
            g: 0.0,
            h: 0.0,
            f: 0.0,
            walked: 0.0,
            kind: PathType::Blocked,
            malus: 0.0,
            previous: None,
            closed: false,
            heap: None,
        }
    }

    pub(crate) fn distance_to(&self, other: BlockPos) -> f32 {
        distance(self.pos, other)
    }

    pub(crate) fn manhattan_to(&self, other: BlockPos) -> i32 {
        (self.pos.x - other.x).abs() + (self.pos.y - other.y).abs() + (self.pos.z - other.z).abs()
    }
}

#[must_use]
pub(crate) fn distance(a: BlockPos, b: BlockPos) -> f32 {
    let dx = (a.x - b.x) as f32;
    let dy = (a.y - b.y) as f32;
    let dz = (a.z - b.z) as f32;
    libm::sqrtf(dx * dx + dy * dy + dz * dz)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PathNode {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub kind: PathType,
}

const _: () = assert!(size_of::<PathNode>() <= 16);

impl PathNode {
    #[must_use]
    pub fn new(pos: BlockPos, kind: PathType) -> Self {
        Self {
            x: pos.x,
            y: pos.y,
            z: pos.z,
            kind,
        }
    }

    #[must_use]
    pub const fn pos(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }

    #[must_use]
    pub fn bottom_centre(&self) -> DVec3 {
        DVec3::new(
            f64::from(self.x) + 0.5,
            f64::from(self.y),
            f64::from(self.z) + 0.5,
        )
    }

    #[must_use]
    pub fn point(&self, width: f32) -> DVec3 {
        let cells = (width + 1.0) as i32;
        let offset = f64::from(cells) * 0.5;
        DVec3::new(
            f64::from(self.x) + offset,
            f64::from(self.y),
            f64::from(self.z) + offset,
        )
    }
}
