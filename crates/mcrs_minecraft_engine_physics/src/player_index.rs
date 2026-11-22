use bevy_ecs::entity::Entity;
use bevy_ecs::query::QueryData;
use bevy_ecs::query::With;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::Query;
use bevy_ecs::system::ResMut;
use glam::Vec3;
use mcrs_minecraft_damage::Vitals;
use mcrs_minecraft_engine::PlayerBody;
use mcrs_minecraft_engine::ShiftKey;
use mcrs_minecraft_engine::client_on_ground::ClientOnGround;
use mcrs_minecraft_engine::network_id::player_network_id;
use mcrs_minecraft_engine::player_motion::PlayerMotion;
use mcrs_minecraft_engine::player_sprinting::PlayerSprinting;
use mcrs_minecraft_engine_item::Selected;
use mcrs_minecraft_engine_item::Slots;
use mcrs_minecraft_engine_world::entity::physics::Transform;
use mcrs_minecraft_engine_world::entity::player::Player;
use mcrs_minecraft_engine_world::session::Owner;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_game_mode::GameMode;
use mcrs_minecraft_game_mode::PlayerGameMode;
use mcrs_minecraft_inventory_player::OFF_HAND;
use mcrs_minecraft_item_stack::Kind;
use mcrs_minecraft_player_index::HurtBy;
use mcrs_minecraft_player_index::ItemKind;
use mcrs_minecraft_player_index::PlayerFlags;
use mcrs_minecraft_player_index::PlayerRow;
use mcrs_minecraft_player_index::PlayerTable;

#[derive(Resource, Clone, Debug, Default)]
pub struct PlayerIndex {
    table: PlayerTable,
    entities: Vec<(EntityId, Entity)>,
}

impl PlayerIndex {
    #[must_use]
    pub fn table(&self) -> &PlayerTable {
        &self.table
    }

    #[must_use]
    pub fn rows(&self) -> &[PlayerRow] {
        self.table.rows()
    }

    #[must_use]
    pub fn find(&self, player: EntityId) -> Option<&PlayerRow> {
        self.table.find(player)
    }

    #[must_use]
    pub fn entities(&self) -> &[(EntityId, Entity)] {
        &self.entities
    }

    #[must_use]
    pub fn entity_of(&self, player: EntityId) -> Option<Entity> {
        let at = self
            .entities
            .binary_search_by_key(&player, |(id, _)| *id)
            .ok()?;
        self.entities.get(at).map(|(_, entity)| *entity)
    }

    pub fn rebuild(&mut self, players: impl IntoIterator<Item = (Entity, PlayerRow)>) {
        self.entities.clear();
        let rows: Vec<PlayerRow> = players
            .into_iter()
            .map(|(entity, row)| {
                self.entities.push((row.player, entity));
                row
            })
            .collect();
        self.entities.sort_unstable_by_key(|(id, _)| *id);
        self.table = PlayerTable::new(rows);
    }
}

#[derive(QueryData)]
pub struct PlayerSource {
    entity: Entity,
    owner: &'static Owner,
    key: &'static EntityUuid,
    transform: &'static Transform,
    mode: Option<&'static PlayerGameMode>,
    body: PlayerBody,
    selected: Option<&'static Selected>,
    slots: Option<&'static Slots>,
    shift: Option<&'static ShiftKey>,
    sprinting: Option<&'static PlayerSprinting>,
    on_ground: Option<&'static ClientOnGround>,
    vitals: Option<&'static Vitals>,
    motion: Option<&'static PlayerMotion>,
}

#[must_use]
pub fn player_id(owner: &Owner) -> EntityId {
    EntityId(u64::from(player_network_id(owner.0).cast_unsigned()))
}

fn hand(slots: Option<&Slots>, slot: Option<usize>, stacks: &Query<&Kind>) -> ItemKind {
    slots
        .zip(slot)
        .and_then(|(slots, slot)| slots.get(slot))
        .and_then(|stack| stacks.get(stack).ok())
        .map_or(ItemKind::EMPTY, |kind| ItemKind(kind.0))
}

fn row_of(player: &PlayerSourceItem<'_, '_>, stacks: &Query<&Kind>) -> PlayerRow {
    let mode = player.mode.map(|mode| mode.0);
    let dimensions = player.body.dimensions();
    let alive = player.vitals.is_none_or(|vitals| vitals.health > 0.0);
    let flags = PlayerFlags::NONE
        .with(PlayerFlags::ALIVE, alive)
        .with(PlayerFlags::SPECTATOR, mode == Some(GameMode::Spectator))
        .with(PlayerFlags::CREATIVE, mode == Some(GameMode::Creative))
        .with(
            PlayerFlags::SNEAKING,
            player.shift.is_some_and(|shift| shift.0),
        )
        .with(
            PlayerFlags::SPRINTING,
            player.sprinting.is_some_and(|sprinting| sprinting.0),
        )
        .with(
            PlayerFlags::ON_GROUND,
            player.on_ground.is_none_or(|on_ground| on_ground.0),
        );
    let motion = player
        .motion
        .map_or(Vec3::ZERO, |motion| motion.delta().as_vec3());
    PlayerRow {
        player: player_id(player.owner),
        key: *player.key,
        position: player.transform.translation,
        width: dimensions.width,
        height: dimensions.height,
        eye_height: dimensions.eye_height,
        flags,
        visibility: 1.0,
        vehicle: None,
        hands: [
            hand(
                player.slots,
                player.selected.map(|selected| usize::from(selected.0)),
                stacks,
            ),
            hand(player.slots, Some(usize::from(OFF_HAND)), stacks),
        ],
        hurt_by: HurtBy::NONE,
        motion,
    }
}

pub(crate) fn index_players(
    players: Query<PlayerSource, With<Player>>,
    stacks: Query<&Kind>,
    mut index: ResMut<PlayerIndex>,
) {
    index.rebuild(
        players
            .iter()
            .map(|player| (player.entity, row_of(&player, &stacks))),
    );
}
