use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_pathfind::MobPath;
use mcrs_minecraft_pathfind::Request;
use mcrs_minecraft_pathfind::Searcher;

#[derive(Clone, Debug, PartialEq)]
pub struct SearchSlice {
    pub key: EntityUuid,
    pub version: u64,
    pub mob: MobPath,
    pub request: Request,
    pub searcher: Searcher,
}
