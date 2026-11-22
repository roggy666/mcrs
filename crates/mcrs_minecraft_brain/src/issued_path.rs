use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_pathfind::FoundPath;

use crate::Ticket;

#[derive(Clone, Debug, PartialEq)]
pub struct IssuedPath {
    pub mob: EntityId,
    pub ticket: Ticket,
    pub path: FoundPath,
}

impl IssuedPath {
    #[must_use]
    pub const fn key(&self) -> (EntityId, Ticket) {
        (self.mob, self.ticket)
    }
}

pub fn sort_paths(paths: &mut [IssuedPath]) {
    paths.sort_by_key(IssuedPath::key);
}

#[must_use]
pub fn find_path(paths: &[IssuedPath], mob: EntityId, ticket: Ticket) -> Option<&FoundPath> {
    let at = paths
        .binary_search_by_key(&(mob, ticket), IssuedPath::key)
        .ok()?;
    paths.get(at).map(|issued| &issued.path)
}
