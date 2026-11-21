use glam::DVec3;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::TILE_WIDTH;
use mcrs_minecraft_entity_key::TileKey;

use crate::KindSet;
use crate::MobView;
use crate::TileIndex;

pub const REACH_TILES: i32 = 2;
const SIDE: usize = (REACH_TILES * 2 + 1) as usize;
const TILES: usize = SIDE * SIDE;

#[derive(Clone, Copy, Debug)]
pub struct TileViews<'a> {
    pub tile: TileKey,
    pub views: &'a [MobView],
    pub index: &'a TileIndex,
}

#[derive(Clone, Copy, Debug)]
pub struct Neighbours<'a> {
    centre: TileKey,
    tiles: [Option<TileViews<'a>>; TILES],
}

impl<'a> Neighbours<'a> {
    #[must_use]
    pub fn new(centre: TileKey, lookup: impl Fn(TileKey) -> Option<TileViews<'a>>) -> Self {
        let mut tiles = [None; TILES];
        for (slot, tile) in tiles.iter_mut().zip(around(centre)) {
            *slot = lookup(tile);
        }
        Self { centre, tiles }
    }

    #[must_use]
    pub const fn empty(centre: TileKey) -> Self {
        Self {
            centre,
            tiles: [None; TILES],
        }
    }

    #[must_use]
    pub const fn centre(&self) -> TileKey {
        self.centre
    }

    pub fn tiles(&self) -> impl Iterator<Item = TileViews<'a>> + '_ {
        self.tiles.iter().flatten().copied()
    }

    pub fn for_each_in(
        &self,
        min: DVec3,
        max: DVec3,
        kinds: &KindSet,
        mut visit: impl FnMut(&'a MobView),
    ) {
        let width = f64::from(TILE_WIDTH.get());
        for held in self.tiles.iter().flatten() {
            let origin_x = f64::from(held.tile.x) * width;
            let origin_z = f64::from(held.tile.z) * width;
            let outside = max.x < origin_x
                || min.x >= origin_x + width
                || max.z < origin_z
                || min.z >= origin_z + width;
            if outside {
                continue;
            }
            for column in TileIndex::columns_in(held.tile, min, max) {
                for index in held.index.column(column) {
                    let Some(view) = held.views.get(index as usize) else {
                        continue;
                    };
                    if !kinds.contains(view.kind) {
                        continue;
                    }
                    let inside = (min.x..=max.x).contains(&view.position.x)
                        && (min.y..=max.y).contains(&view.position.y)
                        && (min.z..=max.z).contains(&view.position.z);
                    if inside {
                        visit(view);
                    }
                }
            }
        }
    }

    #[must_use]
    pub fn find(&self, mob: EntityId) -> Option<&'a MobView> {
        self.tiles.iter().flatten().find_map(|held| {
            let index = held.index.find(mob)?;
            held.views.get(index as usize)
        })
    }

    #[must_use]
    pub fn has_kind(&self, kinds: &KindSet) -> bool {
        self.tiles
            .iter()
            .flatten()
            .any(|held| held.views.iter().any(|view| kinds.contains(view.kind)))
    }
}

fn around(centre: TileKey) -> impl Iterator<Item = TileKey> {
    (-REACH_TILES..=REACH_TILES).flat_map(move |dx| {
        (-REACH_TILES..=REACH_TILES).map(move |dz| {
            TileKey::new(
                centre.x.saturating_add(dx),
                centre.y,
                centre.z.saturating_add(dz),
            )
        })
    })
}
