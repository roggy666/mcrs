use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;

use crate::Aabb;
use crate::GridRow;

pub trait BodyView {
    fn colliders(&self, query: Aabb, mover: EntityUuid, out: &mut Vec<EntityCollider>);

    fn neighbours(&self, of: &GridRow, out: &mut Vec<GridRow>);

    fn neighbours_up_to<F: FnMut(&GridRow)>(
        &self,
        of: &GridRow,
        out: &mut Vec<GridRow>,
        limit: usize,
        mut each: F,
    ) {
        self.neighbours(of, out);
        let bounds = of.bounds();
        out.retain(|row| row.key != of.key && row.bounds().intersects(&bounds));
        out.sort_by_key(|row| row.key);
        for row in out.iter() {
            each(row);
        }
        if limit > 0 {
            out.truncate(limit);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntityCollider {
    pub bounds: Aabb,
    pub id: EntityId,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoBodies;

impl BodyView for NoBodies {
    fn colliders(&self, _query: Aabb, _mover: EntityUuid, _out: &mut Vec<EntityCollider>) {}

    fn neighbours(&self, _of: &GridRow, _out: &mut Vec<GridRow>) {}
}

#[cfg(feature = "testing")]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ListedBodies {
    pub colliders: Vec<EntityCollider>,
    pub rows: Vec<GridRow>,
}

#[cfg(feature = "testing")]
impl BodyView for ListedBodies {
    fn colliders(&self, query: Aabb, _mover: EntityUuid, out: &mut Vec<EntityCollider>) {
        out.extend(
            self.colliders
                .iter()
                .filter(|found| found.bounds.intersects(&query)),
        );
    }

    fn neighbours(&self, of: &GridRow, out: &mut Vec<GridRow>) {
        let bounds = of.bounds();
        out.extend(
            self.rows
                .iter()
                .filter(|row| row.bounds().intersects(&bounds)),
        );
    }
}
