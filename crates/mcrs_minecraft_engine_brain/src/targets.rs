use glam::DVec3;
use mcrs_minecraft_brain::KindTable;
use mcrs_minecraft_brain::MobFlags;
use mcrs_minecraft_brain::PlayerTable;
use mcrs_minecraft_brain::Replica;
use mcrs_minecraft_brain::Stage;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;

#[derive(Debug)]
pub struct Targets<'a, B> {
    pub replica: &'a Replica,
    pub stage: &'a Stage<B>,
    pub players: &'a PlayerTable,
    pub kinds: &'a KindTable,
    pub tick: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TargetBody {
    pub key: EntityUuid,
    pub position: DVec3,
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub alive: bool,
    pub player: bool,
}

impl TargetBody {
    #[must_use]
    pub fn eye(&self) -> DVec3 {
        DVec3::new(
            self.position.x,
            self.position.y + f64::from(self.eye_height),
            self.position.z,
        )
    }
}

impl<B> Targets<'_, B> {
    #[must_use]
    pub fn body(&self, id: EntityId) -> Option<TargetBody> {
        if let Some(tile) = self.stage.tile_of(id) {
            let views = self.replica.tile(tile, self.tick)?;
            let index = views.index.find(id)?;
            let view = views.views.get(index as usize)?;
            let dimensions = self.kinds.dimensions(view);
            return Some(TargetBody {
                key: view.key,
                position: view.position,
                width: dimensions.width,
                height: dimensions.height,
                eye_height: dimensions.eye_height,
                alive: view.flags.contains(MobFlags::ALIVE),
                player: false,
            });
        }
        let row = self.players.find(id)?;
        Some(TargetBody {
            key: row.key,
            position: row.position,
            width: row.width,
            height: row.height,
            eye_height: row.eye_height,
            alive: row.is_qualified(),
            player: true,
        })
    }
}
