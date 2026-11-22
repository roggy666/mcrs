use bevy_app::App;
use bevy_app::FixedUpdate;
use bevy_app::Plugin;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::observer::On;
use bevy_ecs::query::Changed;
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::system::Commands;
use bevy_ecs::system::Query;
use mcrs_minecraft_damage::Fire;
use mcrs_minecraft_engine_world::tick::TickSet;

use crate::persistence::autosave_players;
use crate::player_state::PlayerFileState;

pub(crate) struct PlayerFirePlugin;

impl Plugin for PlayerFirePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(restore_fire);
        app.add_systems(
            FixedUpdate,
            mirror_fire.in_set(TickSet::Output).before(autosave_players),
        );
    }
}

pub(crate) fn restore_fire(
    insert: On<Insert<PlayerFileState>>,
    files: Query<&PlayerFileState>,
    mut commands: Commands,
) {
    let entity = insert.event().entity;
    if let Ok(file) = files.get(entity) {
        commands
            .entity(entity)
            .insert(Fire::of(i32::from(file.fire)));
    }
}

pub(crate) fn mirror_fire(mut players: Query<(&Fire, &mut PlayerFileState), Changed<Fire>>) {
    for (fire, mut file) in &mut players {
        let ticks = i16::try_from(fire.ticks).unwrap_or(0);
        if file.fire != ticks {
            file.fire = ticks;
        }
    }
}
