use mcrs_minecraft_brain::ItemTables;
use mcrs_minecraft_brain::KindTable;
use mcrs_minecraft_goal_world::BuildHeight;
use mcrs_minecraft_nav_projection::NavTable;
use mcrs_minecraft_physics::Tables;

#[derive(Clone, Debug)]
pub struct StageTables {
    pub physics: Tables,
    pub nav: NavTable,
    pub items: ItemTables,
    pub kinds: KindTable,
    pub height: BuildHeight,
}

impl StageTables {
    #[must_use]
    pub fn new(physics: Tables, items: ItemTables, kinds: KindTable, height: BuildHeight) -> Self {
        let nav = NavTable::new(&physics.collision);
        Self {
            physics,
            nav,
            items,
            kinds,
            height,
        }
    }
}
