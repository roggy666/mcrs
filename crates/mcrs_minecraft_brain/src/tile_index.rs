use alloc::vec::Vec;

use glam::DVec3;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::TILE_WIDTH;
use mcrs_minecraft_entity_key::TileKey;

use crate::KindSet;
use crate::MobView;

pub const COLUMN_WIDTH: i32 = 16;
const COLUMNS_PER_SIDE: usize = (TILE_WIDTH.get() / COLUMN_WIDTH as u32) as usize;
const COLUMNS: usize = COLUMNS_PER_SIDE * COLUMNS_PER_SIDE;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TileIndex {
    order: Vec<u32>,
    starts: [u32; COLUMNS + 1],
    by_id: Vec<(EntityId, u32)>,
    kinds: KindSet,
}

impl TileIndex {
    #[must_use]
    pub fn build(tile: TileKey, views: &[MobView]) -> Self {
        let mut keyed: Vec<(usize, u32)> = views
            .iter()
            .enumerate()
            .filter_map(|(index, view)| {
                Some((column_of(tile, view.position), u32::try_from(index).ok()?))
            })
            .collect();
        keyed.sort_unstable();
        let mut starts: [u32; COLUMNS + 1] = [0; COLUMNS + 1];
        for (column, _) in &keyed {
            for start in starts.iter_mut().skip(column.saturating_add(1)) {
                *start = (*start).saturating_add(1);
            }
        }
        let order: Vec<u32> = keyed.into_iter().map(|(_, index)| index).collect();
        let mut by_id: Vec<(EntityId, u32)> = views
            .iter()
            .enumerate()
            .filter_map(|(index, view)| Some((view.mob, u32::try_from(index).ok()?)))
            .collect();
        by_id.sort_unstable();
        let kinds = views
            .iter()
            .fold(KindSet::NONE, |kinds, view| kinds.with(view.kind));
        Self {
            order,
            starts,
            by_id,
            kinds,
        }
    }

    #[must_use]
    pub const fn kinds(&self) -> &KindSet {
        &self.kinds
    }

    pub fn column(&self, column: usize) -> impl Iterator<Item = u32> + '_ {
        let start = self.starts.get(column).copied().unwrap_or(0) as usize;
        let end = self
            .starts
            .get(column.saturating_add(1))
            .copied()
            .unwrap_or(0) as usize;
        self.order
            .get(start..end)
            .unwrap_or_default()
            .iter()
            .copied()
    }

    #[must_use]
    pub fn find(&self, mob: EntityId) -> Option<u32> {
        let at = self.by_id.binary_search_by_key(&mob, |(id, _)| *id).ok()?;
        self.by_id.get(at).map(|(_, index)| *index)
    }

    #[must_use]
    pub const fn columns() -> usize {
        COLUMNS
    }

    pub fn columns_in(tile: TileKey, min: DVec3, max: DVec3) -> impl Iterator<Item = usize> {
        let origin_x = tile.x.saturating_mul(TILE_WIDTH.get() as i32);
        let origin_z = tile.z.saturating_mul(TILE_WIDTH.get() as i32);
        let span = |low: f64, high: f64, origin: i32| {
            let first = floor_div(libm::floor(low) as i32 - origin, COLUMN_WIDTH)
                .clamp(0, COLUMNS_PER_SIDE as i32 - 1);
            let last = floor_div(libm::floor(high) as i32 - origin, COLUMN_WIDTH)
                .clamp(0, COLUMNS_PER_SIDE as i32 - 1);
            first..=last
        };
        let xs = span(min.x, max.x, origin_x);
        let zs = span(min.z, max.z, origin_z);
        xs.flat_map(move |x| {
            zs.clone()
                .map(move |z| x as usize * COLUMNS_PER_SIDE + z as usize)
        })
    }
}

fn column_of(tile: TileKey, position: DVec3) -> usize {
    let origin_x = tile.x.saturating_mul(TILE_WIDTH.get() as i32);
    let origin_z = tile.z.saturating_mul(TILE_WIDTH.get() as i32);
    let x = floor_div(libm::floor(position.x) as i32 - origin_x, COLUMN_WIDTH)
        .clamp(0, COLUMNS_PER_SIDE as i32 - 1);
    let z = floor_div(libm::floor(position.z) as i32 - origin_z, COLUMN_WIDTH)
        .clamp(0, COLUMNS_PER_SIDE as i32 - 1);
    x as usize * COLUMNS_PER_SIDE + z as usize
}

fn floor_div(value: i32, divisor: i32) -> i32 {
    value.div_euclid(divisor)
}
