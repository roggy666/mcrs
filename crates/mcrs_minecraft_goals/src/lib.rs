#![no_std]

extern crate alloc;

mod avoid;
mod breed;
mod eat_grass;
#[cfg(feature = "testing")]
mod flat_ground;
mod float;
mod follow_adult;
mod gate;
mod goal_kind;
mod goal_state;
mod hurt_by;
mod leap;
mod look_around;
mod look_at_player;
mod melee;
mod nearest_target;
mod owner;
mod panic;
mod parts;
mod path_track;
mod query;
mod random_position;
mod ranged;
mod stroll;
mod table_brain;
mod tempt;

#[cfg(feature = "testing")]
pub use flat_ground::*;
pub use float::*;
pub use gate::*;
pub use goal_kind::*;
pub use goal_state::*;
pub use look_around::*;
pub use look_at_player::*;
pub use melee::in_reach;
pub use panic::*;
pub use parts::*;
pub use path_track::*;
pub use random_position::*;
pub use stroll::*;
pub use table_brain::*;
