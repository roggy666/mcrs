use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::PlayerRow;
use glam::DVec3;
use mcrs_minecraft_entity_key::EntityId;

/// The rows of the players of a dimension, sorted by key, with a map from the column of each
/// position to its rows. A column is 16 by 16 blocks.
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
                .entry(column_of(row.position.x, row.position.z))
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
        let (min_x, min_z) = column_of(position.x - reach, position.z - reach);
        let (max_x, max_z) = column_of(position.x + reach, position.z + reach);
        let limit = reach * reach;
        (min_x..=max_x).any(|x| {
            self.columns
                .range((x, min_z)..=(x, max_z))
                .flat_map(|(_, indices)| indices.iter())
                .filter_map(|index| self.rows.get(*index as usize))
                .any(|row| accept(row) && row.position.distance_squared(position) < limit)
        })
    }

    /// The rows whose position lies in the columns that the box `(min, max)` touches, in the
    /// order of the columns and then of the keys. The box is in blocks; only `x` and `z` count.
    pub fn in_columns(&self, min: DVec3, max: DVec3) -> impl Iterator<Item = &PlayerRow> {
        let (min_x, min_z) = column_of(min.x, min.z);
        let (max_x, max_z) = column_of(max.x, max.z);
        (min_x..=max_x).flat_map(move |x| {
            self.columns
                .range((x, min_z)..=(x, max_z))
                .flat_map(|(_, indices)| indices.iter())
                .filter_map(|index| self.rows.get(*index as usize))
        })
    }

    /// The rows whose body box meets the box `(min, max)`. The body of a player reaches at most
    /// `reach` blocks out of its column; the search widens the columns by that much.
    pub fn bodies_in(
        &self,
        min: DVec3,
        max: DVec3,
        reach: f64,
    ) -> impl Iterator<Item = &PlayerRow> {
        let margin = DVec3::new(reach, 0.0, reach);
        self.in_columns(min - margin, max + margin)
            .filter(move |row| row.meets(min, max))
    }

    pub fn collect_in(&self, min: DVec3, max: DVec3, out: &mut Vec<PlayerRow>) {
        let start = out.len();
        for row in self.in_columns(min, max) {
            let inside = (min.x..=max.x).contains(&row.position.x)
                && (min.z..=max.z).contains(&row.position.z);
            if inside {
                out.push(*row);
            }
        }
        if let Some(found) = out.get_mut(start..) {
            found.sort_by_key(|row| row.key);
        }
    }
}

/// The column of a position: the floor of `x / 16` and of `z / 16`.
#[must_use]
pub fn column_of(x: f64, z: f64) -> (i32, i32) {
    (
        libm::floor(x / COLUMN) as i32,
        libm::floor(z / COLUMN) as i32,
    )
}
