use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_entity_key::TileKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Presence {
    pub(crate) key: EntityUuid,
    pub(crate) tile: TileKey,
}
