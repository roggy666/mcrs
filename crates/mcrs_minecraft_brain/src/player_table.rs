use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::PlayerRow;
use glam::DVec3;
use mcrs_minecraft_entity_key::EntityId;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayerTable {
    rows: Vec<PlayerRow>,
    by_id: Vec<(EntityId, u32)>,
    columns: BTreeMap<(i32, i32), Vec<u32>>,
}

const COLUMN: f64 = 16.0;

impl PlayerTable {
    #[must_use]
    pub fn new(rows: impl IntoIterator<Item = PlayerRow>) -> Self {
        let mut rows: Vec<PlayerRow> = rows.into_iter().collect();
        rows.sort_by_key(|row| row.key);
        let mut columns: BTreeMap<(i32, i32), Vec<u32>> = BTreeMap::new();
        let mut by_id: Vec<(EntityId, u32)> = Vec::with_capacity(rows.len());
        for (index, row) in rows.iter().enumerate() {
            let Ok(index) = u32::try_from(index) else {
                break;
            };
            by_id.push((row.player, index));
            columns
                .entry(column(row.position.x, row.position.z))
                .or_default()
                .push(index);
        }
        by_id.sort_unstable();
        Self {
            rows,
            by_id,
            columns,
        }
    }

    #[must_use]
    pub fn rows(&self) -> &[PlayerRow] {
        &self.rows
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    #[must_use]
    pub fn find(&self, player: EntityId) -> Option<&PlayerRow> {
        let at = self
            .by_id
            .binary_search_by_key(&player, |(id, _)| *id)
            .ok()?;
        let (_, index) = self.by_id.get(at)?;
        self.rows.get(*index as usize)
    }

    #[must_use]
    pub fn any_within(
        &self,
        position: DVec3,
        reach: f64,
        accept: impl Fn(&PlayerRow) -> bool,
    ) -> bool {
        let (min_x, min_z) = column(position.x - reach, position.z - reach);
        let (max_x, max_z) = column(position.x + reach, position.z + reach);
        let limit = reach * reach;
        (min_x..=max_x).any(|x| {
            self.columns
                .range((x, min_z)..=(x, max_z))
                .flat_map(|(_, indices)| indices.iter())
                .filter_map(|index| self.rows.get(*index as usize))
                .any(|row| accept(row) && row.position.distance_squared(position) < limit)
        })
    }

    pub fn collect_in(&self, min: DVec3, max: DVec3, out: &mut Vec<PlayerRow>) {
        let start = out.len();
        let (min_x, min_z) = column(min.x, min.z);
        let (max_x, max_z) = column(max.x, max.z);
        for x in min_x..=max_x {
            for (_, indices) in self.columns.range((x, min_z)..=(x, max_z)) {
                for &index in indices {
                    let Some(row) = self.rows.get(index as usize) else {
                        continue;
                    };
                    let inside = (min.x..=max.x).contains(&row.position.x)
                        && (min.z..=max.z).contains(&row.position.z);
                    if inside {
                        out.push(*row);
                    }
                }
            }
        }
        if let Some(found) = out.get_mut(start..) {
            found.sort_by_key(|row| row.key);
        }
    }
}

fn column(x: f64, z: f64) -> (i32, i32) {
    (
        libm::floor(x / COLUMN) as i32,
        libm::floor(z / COLUMN) as i32,
    )
}
