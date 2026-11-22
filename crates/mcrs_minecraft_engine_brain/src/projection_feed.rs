use bevy_ecs::entity::Entity;
use bevy_ecs::message::MessageReader;
use bevy_ecs::query::With;
use bevy_ecs::system::Local;
use bevy_ecs::system::Query;
use bevy_ecs::system::ResMut;
use bevy_ecs::system::Single;
use mcrs_minecraft_brain::MobBrain;
use mcrs_minecraft_brain_stage::Landed;
use mcrs_minecraft_brain_stage::ProjectionWorld;
use mcrs_minecraft_engine_world::block_update::BlockPlaced;
use mcrs_minecraft_engine_world::world::dimension::Dimension;
use mcrs_minecraft_engine_world::world::lifecycle::stage::SectionStageChanged;
use mcrs_minecraft_engine_world::world::storage::section::SectionIndex;
use mcrs_minecraft_engine_world::world::storage::view::WorldView;
use mcrs_minecraft_nav_projection::NavPatch;
use mcrs_minecraft_volume::BlockVolume;
use mcrs_voxel::BlockPos;
use rustc_hash::FxHashSet;

use crate::MobAi;

pub(crate) fn feed_projection<B>(
    dimension: Single<Entity, With<Dimension>>,
    world: WorldView,
    indices: Query<&SectionIndex>,
    mut stages: MessageReader<SectionStageChanged>,
    mut placed: MessageReader<BlockPlaced>,
    mut ai: ResMut<MobAi<B>>,
    mut seen: Local<FxHashSet<BlockPos>>,
) where
    B: for<'a> MobBrain<ProjectionWorld<'a>> + Send + Sync + 'static,
{
    let dimension = *dimension;
    let index = indices.get(dimension).ok();
    let resend = ai.take_resend();
    let ai = &mut *ai;
    let nav = ai.nav_table().clone();
    let delta = ai.blocks();
    if resend && let Some(index) = index {
        delta.landed.clear();
        delta.unloaded.clear();
        for (pos, _, blocks) in index.loaded() {
            delta.landed.push(Landed {
                pos,
                blocks: blocks.clone(),
            });
        }
    }
    for change in stages.read() {
        if change.dim != dimension {
            continue;
        }
        if change.left() {
            delta.unloaded.push(change.pos);
        } else if change.landed()
            && let Some(blocks) = index.and_then(|index| index.blocks(change.pos))
        {
            delta.landed.push(Landed {
                pos: change.pos,
                blocks: blocks.clone(),
            });
        }
    }
    let Some(blocks) = world.get(dimension) else {
        return;
    };
    seen.clear();
    for change in placed.read() {
        if nav.of_state(change.old_state) == nav.of_state(change.new_state) {
            continue;
        }
        if !seen.insert(change.block_pos) {
            continue;
        }
        if let Ok(state) = blocks.block(change.block_pos) {
            delta.patches.push(NavPatch {
                pos: change.block_pos,
                nav: nav.of_state(state),
            });
        }
    }
}
