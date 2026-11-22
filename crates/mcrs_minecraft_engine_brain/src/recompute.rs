use bevy_ecs::message::MessageReader;
use bevy_ecs::system::Local;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use glam::DVec3;
use mcrs_minecraft_engine_entity_motion::MotionState;
use mcrs_minecraft_engine_physics::PhysicsTables;
use mcrs_minecraft_engine_world::voxel_update::BlockPlaced;
use mcrs_minecraft_entity_key::TileKey;
use mcrs_voxel::BlockPos;
use rustc_hash::FxHashMap;

use crate::MobNavigation;

const BOX_MARGIN: i32 = 1;

pub(crate) fn flag_recomputes(
    mut placed: MessageReader<BlockPlaced>,
    tables: Res<PhysicsTables>,
    mut navigating: Query<(&MotionState, &mut MobNavigation)>,
    mut changes: Local<FxHashMap<TileKey, Vec<BlockPos>>>,
) {
    for list in changes.values_mut() {
        list.clear();
    }
    let table = tables.collision();
    let mut any = false;
    for change in placed.read() {
        let old = table.state(change.old_state);
        let new = table.state(change.new_state);
        if old.class == new.class {
            continue;
        }
        any = true;
        let centre = change.block_pos.as_dvec3() + DVec3::splat(0.5);
        changes
            .entry(TileKey::at(centre))
            .or_default()
            .push(change.block_pos);
    }
    if !any {
        return;
    }
    for (state, mut navigation) in &mut navigating {
        let nav = &navigation.0;
        if nav.recompute_waiting {
            continue;
        }
        let (Some(path), Some(bounds)) = (nav.path.as_ref(), nav.bounds) else {
            continue;
        };
        if path.is_done() || path.nodes.is_empty() {
            continue;
        }
        let Some(end) = path.end() else {
            continue;
        };
        let position = state.0.body.position;
        let midpoint = (position + end.bottom_centre()) / 2.0;
        let remaining = (path.nodes.len() - path.next) as f64;
        let limit = remaining * remaining;
        let tile = TileKey::at(position);
        let near = (-1..=1)
            .flat_map(|dx| (-1..=1).map(move |dz| TileKey::new(tile.x + dx, 0, tile.z + dz)));
        let hit = near.into_iter().any(|key| {
            changes.get(&key).is_some_and(|list| {
                list.iter().any(|&pos| {
                    bounds.touches(pos, BOX_MARGIN)
                        && (pos.as_dvec3() + DVec3::splat(0.5)).distance_squared(midpoint) < limit
                })
            })
        });
        if hit {
            navigation.0.recompute_waiting = true;
        }
    }
}
