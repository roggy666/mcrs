use crate::JukeboxSongId;
use crate::entity_form::EntityForm;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub struct JukeboxPlayable(pub JukeboxSongId);

impl EntityForm for JukeboxPlayable {}
