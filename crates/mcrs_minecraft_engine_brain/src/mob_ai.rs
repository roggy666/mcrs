use std::time::Instant;

use bevy_ecs::change_detection::DetectChangesMut;
use bevy_ecs::entity::Entity;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::ParamSet;
use bevy_ecs::system::Query;
use bevy_ecs::world::Mut;
use bevy_utils::Parallel;
use glam::DVec3;
use mcrs_engine_link::Link;
use mcrs_engine_link::Placement;
use mcrs_minecraft_brain::Decision;
use mcrs_minecraft_brain::DecisionPipeline;
use mcrs_minecraft_brain::EntityRef;
use mcrs_minecraft_brain::Fact;
use mcrs_minecraft_brain::FactKind;
use mcrs_minecraft_brain::IssuedDecisions;
use mcrs_minecraft_brain::IssuedPath;
use mcrs_minecraft_brain::ItemTables;
use mcrs_minecraft_brain::KindSet;
use mcrs_minecraft_brain::KindTable;
use mcrs_minecraft_brain::MobBrain;
use mcrs_minecraft_brain::MobKind;
use mcrs_minecraft_brain::MobView;
use mcrs_minecraft_brain::Neighbours;
use mcrs_minecraft_brain::Panic;
use mcrs_minecraft_brain::PanicCauses;
use mcrs_minecraft_brain::PlayerTable;
use mcrs_minecraft_brain::Proposal;
use mcrs_minecraft_brain::Receivers;
use mcrs_minecraft_brain::StageConfig;
use mcrs_minecraft_brain::TileSnapshot;
use mcrs_minecraft_brain::TileViews;
use mcrs_minecraft_brain::find_path;
use mcrs_minecraft_brain::sort_paths;
use mcrs_minecraft_brain_stage::AiInput;
use mcrs_minecraft_brain_stage::AiStage;
use mcrs_minecraft_brain_stage::Arrival;
use mcrs_minecraft_brain_stage::BlockDelta;
use mcrs_minecraft_brain_stage::ProjectionWorld;
use mcrs_minecraft_brain_stage::StageTables;
use mcrs_minecraft_brain_stage::TickInput;
use mcrs_minecraft_brain_stage::ai_pool;
use mcrs_minecraft_damage::DamageSource;
use mcrs_minecraft_damage::Exposure;
use mcrs_minecraft_damage::Fire;
use mcrs_minecraft_damage::Hit;
use mcrs_minecraft_damage::HitRecord;
use mcrs_minecraft_damage::Vitals;
use mcrs_minecraft_engine_damage::DamageRequest;
use mcrs_minecraft_engine_damage::PendingHits;
use mcrs_minecraft_engine_entity_motion::MobIntent;
use mcrs_minecraft_engine_entity_motion::MotionId;
use mcrs_minecraft_engine_entity_motion::MotionState;
use mcrs_minecraft_engine_mob_attributes::MobAttributes;
use mcrs_minecraft_engine_physics::MobExtras;
use mcrs_minecraft_engine_physics::MobSpawnRequest;
use mcrs_minecraft_engine_physics::SpawnSetup;
use mcrs_minecraft_entity::EntityTypeData;
use mcrs_minecraft_entity_key::BirthSource;
use mcrs_minecraft_entity_key::Cause;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_entity_key::TileKey;
use mcrs_minecraft_entity_key::WriterKey;
use mcrs_minecraft_goal_world::BuildHeight;
use mcrs_minecraft_nav_projection::NavTable;
use mcrs_minecraft_physics::Tables;
use mcrs_minecraft_volume::BlockVolume;
use rustc_hash::FxHashMap;

use crate::AiStats;
use crate::BlockGoalWorld;
use crate::BrainState;
use crate::MobHit;
use crate::MobNavigation;
use crate::RangedAttack;
use crate::brain_state_ext::BrainStateExt;
use crate::hit_queue::HitTarget;
use crate::hit_queue::MeleeHit;
use crate::hit_queue::Proposed;
use crate::hit_queue::RangedShot;
use crate::presence::Presence;
use crate::sensed::Sensed;
use crate::targets::Targets;

const BREED_DISTANCE_SQUARED: f64 = 9.0;
const ALERT_HEIGHT: f64 = 10.0;

pub(crate) type Thinking<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static MotionId,
        &'static MotionState,
        Option<&'static MobAttributes>,
        Option<&'static Vitals>,
        Option<&'static HitRecord>,
        Option<&'static Fire>,
        Option<&'static Exposure>,
        &'static mut BrainState,
        &'static mut MobIntent,
        &'static mut MobNavigation,
    ),
>;

pub(crate) type Hitting<'w, 's> = Query<
    'w,
    's,
    (
        &'static MotionState,
        &'static mut BrainState,
        Option<&'static Vitals>,
    ),
>;

type Entities = FxHashMap<EntityId, Entity>;

pub(crate) type PlayerEntities = [(EntityId, Entity)];

pub(crate) type Snapshots = Vec<(TileKey, TileSnapshot)>;

#[derive(Resource)]
pub struct MobAi<B>
where
    B: for<'a> MobBrain<ProjectionWorld<'a>> + Send + Sync + 'static,
{
    link: Link<AiStage<B>>,
    pipeline: DecisionPipeline,
    config: StageConfig,
    tick: u64,
    brain_for: fn(&'static EntityTypeData) -> Option<B>,
    panic_causes: PanicCauses,
    kinds: KindTable,
    items: ItemTables,
    mob_attack: u32,
    nav: NavTable,
    resend_sections: bool,
    present: FxHashMap<EntityId, Presence>,
    snapshots: Snapshots,
    input: TickInput<B>,
    due: Vec<Decision>,
    due_paths: Vec<IssuedPath>,
    ended: Vec<EntityId>,
    gathered: BrainOut,
    facts: Vec<TiledFact>,
    player_hits: Vec<MobHit>,
    shots: Vec<RangedAttack>,
    spawns: Vec<MobSpawnRequest>,
    struck: Vec<(Entity, EntityId)>,
    births: u32,
    stats: AiStats,
}

#[derive(Debug, Default)]
pub(crate) struct BrainOut {
    views: Vec<MobView>,
    facts: Vec<Fact>,
    ended: Vec<EntityId>,
    moves: Vec<Move>,
    entities: Vec<(EntityId, Entity)>,
    hits: Vec<MeleeHit>,
    shots: Vec<RangedShot>,
    proposals: Vec<Proposed>,
    applied: usize,
}

#[derive(Clone, Copy, Debug)]
struct Move {
    mob: EntityId,
    tile: TileKey,
}

#[derive(Clone, Copy, Debug)]
struct TiledFact {
    tile: TileKey,
    fact: Fact,
}

struct Pass<'a, V> {
    tick: u64,
    players: &'a PlayerTable,
    panic_causes: &'a PanicCauses,
    due: &'a [Decision],
    due_paths: &'a [IssuedPath],
    targets: Targets<'a>,
    world: &'a BlockGoalWorld<'a, V>,
}

struct Publishing<'a, 'w, 's> {
    players: PlayerTable,
    player_entities: &'a PlayerEntities,
    mobs: &'a mut Hitting<'w, 's>,
    pending: Option<&'a mut PendingHits>,
    tick: u64,
}

pub(crate) fn tile_views(snapshots: &Snapshots, tile: TileKey) -> Option<TileViews<'_>> {
    let index = snapshots
        .binary_search_by_key(&tile, |(tile, _)| *tile)
        .ok()?;
    let (_, snapshot) = snapshots.get(index)?;
    Some(TileViews {
        tile,
        views: &snapshot.views,
        index: &snapshot.index,
    })
}

impl<B> MobAi<B>
where
    B: for<'a> MobBrain<ProjectionWorld<'a>> + Send + Sync + 'static,
{
    #[must_use]
    pub fn new(
        config: StageConfig,
        brain_for: fn(&'static EntityTypeData) -> Option<B>,
        kinds: KindTable,
        placement: Placement,
        threads: usize,
    ) -> Self {
        let stage = AiStage::new(config, ai_pool(threads));
        Self {
            link: Link::new(stage, placement),
            pipeline: DecisionPipeline::new(&config),
            config,
            tick: 0,
            brain_for,
            panic_causes: PanicCauses::default(),
            kinds,
            items: ItemTables::default(),
            mob_attack: 0,
            nav: NavTable::default(),
            resend_sections: false,
            present: FxHashMap::default(),
            snapshots: Vec::new(),
            input: TickInput::new(1),
            due: Vec::new(),
            due_paths: Vec::new(),
            ended: Vec::new(),
            gathered: BrainOut::default(),
            facts: Vec::new(),
            player_hits: Vec::new(),
            shots: Vec::new(),
            spawns: Vec::new(),
            struck: Vec::new(),
            births: 0,
            stats: AiStats::default(),
        }
    }

    pub fn configure(
        &mut self,
        physics: &Tables,
        items: ItemTables,
        panic_causes: PanicCauses,
        mob_attack: u32,
        height: BuildHeight,
    ) {
        self.items = items.clone();
        self.panic_causes = panic_causes;
        self.mob_attack = mob_attack;
        let tables = StageTables::new(physics.clone(), items, self.kinds.clone(), height);
        self.nav = tables.nav.clone();
        self.resend_sections = true;
        self.link.send(AiInput::Tables(Box::new(tables)));
    }

    pub fn settle(&mut self) {
        self.collect_outputs();
    }

    #[cfg(feature = "testing")]
    pub fn slow_stage(&mut self, duration: std::time::Duration) {
        self.link.send(AiInput::Slow(duration));
    }

    #[must_use]
    pub fn inline_stage(&self) -> Option<&AiStage<B>> {
        self.link.inline_stage()
    }

    #[must_use]
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    #[must_use]
    pub const fn stats(&self) -> &AiStats {
        &self.stats
    }

    #[must_use]
    pub fn present(&self) -> usize {
        self.present.len()
    }

    #[must_use]
    pub fn is_present(&self, mob: EntityId) -> bool {
        self.present.contains_key(&mob)
    }

    #[must_use]
    pub const fn nav_table(&self) -> &NavTable {
        &self.nav
    }

    pub fn take_resend(&mut self) -> bool {
        std::mem::take(&mut self.resend_sections)
    }

    pub fn blocks(&mut self) -> &mut BlockDelta {
        &mut self.input.blocks
    }

    pub fn take_player_hits(&mut self) -> Vec<MobHit> {
        std::mem::take(&mut self.player_hits)
    }

    pub fn take_shots(&mut self) -> Vec<RangedAttack> {
        std::mem::take(&mut self.shots)
    }

    pub fn take_spawns(&mut self) -> Vec<MobSpawnRequest> {
        std::mem::take(&mut self.spawns)
    }

    #[must_use]
    pub fn attacker_of(&self, target: Entity) -> Option<EntityId> {
        self.struck
            .binary_search_by_key(&target, |(struck, _)| *struck)
            .ok()
            .and_then(|index| self.struck.get(index))
            .map(|(_, attacker)| *attacker)
    }

    pub(crate) fn brain_for(&self, kind: &'static EntityTypeData) -> Option<B> {
        (self.brain_for)(kind)
    }

    pub(crate) fn admit(&mut self, arrival: Arrival<B>) {
        self.input.arrivals.push(arrival);
    }

    pub(crate) fn release(&mut self, mob: EntityId) {
        self.input.arrivals.retain(|arrival| arrival.id != mob);
        self.input.departures.push(mob);
    }

    fn tile_of(&self, mob: EntityId) -> Option<TileKey> {
        self.present.get(&mob).map(|presence| presence.tile)
    }

    fn key_of(&self, mob: EntityId) -> Option<EntityUuid> {
        self.present.get(&mob).map(|presence| presence.key)
    }

    pub(crate) fn prepare<V: BlockVolume + Sync>(
        &mut self,
        world: &BlockGoalWorld<'_, V>,
        players: PlayerTable,
        player_entities: &PlayerEntities,
        mobs: &mut ParamSet<'_, '_, (Thinking<'_, '_>, Hitting<'_, '_>)>,
        pending: Option<&mut PendingHits>,
        threads: &mut Parallel<BrainOut>,
    ) {
        let started = Instant::now();
        self.tick = self.tick.saturating_add(1);
        let tick = self.tick;
        self.pipeline.issue(tick);
        self.collect_outputs();
        self.follow_life_cycle();
        let followed = Instant::now();
        self.take_due(tick);
        let waited = Instant::now();
        let pass = Pass {
            tick,
            players: &players,
            panic_causes: &self.panic_causes,
            due: &self.due,
            due_paths: &self.due_paths,
            targets: Targets {
                snapshots: &self.snapshots,
                present: &self.present,
                players: &players,
                kinds: &self.kinds,
            },
            world,
        };
        let shared = &*threads;
        mobs.p0().par_iter_mut().for_each_init(
            || shared.borrow_local_mut(),
            |out,
             (
                entity,
                id,
                state,
                attributes,
                vitals,
                hits,
                fire,
                exposure,
                brain,
                intent,
                navigation,
            )| {
                let sensed = Sensed {
                    state: &state.0,
                    vitals,
                    hits,
                    fire,
                    exposure,
                    recompute_waiting: navigation.0.recompute_waiting,
                };
                pass.think(
                    out, entity, id.0, sensed, attributes, brain, intent, navigation,
                );
            },
        );
        let ran = Instant::now();
        self.publish(
            threads,
            Publishing {
                players,
                player_entities,
                mobs: &mut mobs.p1(),
                pending,
                tick,
            },
        );
        self.stats.life_cycle = followed.duration_since(started);
        self.stats.wait = waited.duration_since(followed);
        self.stats.tile_tasks = ran.duration_since(waited);
        self.stats.publish = ran.elapsed();
        self.stats.prepare = started.elapsed();
        self.stats.mobs = self.present.len();
    }

    fn collect_outputs(&mut self) {
        for output in self.link.outputs() {
            self.stats.stage = output.run.stage;
            self.stats.evaluations = output.run.evaluations;
            self.stats.evaluations_total = self
                .stats
                .evaluations_total
                .saturating_add(output.run.evaluations);
            self.stats.decisions = output.run.decisions;
            self.stats.projection_sections = output.run.sections;
            self.stats.projection_bytes = output.run.projection_bytes;
            self.stats.without_tables = output.run.without_tables;
            self.pipeline.ready(output.decisions);
        }
    }

    fn follow_life_cycle(&mut self) {
        for mob in &self.input.departures {
            self.present.remove(mob);
        }
        self.input
            .arrivals
            .sort_unstable_by_key(|arrival| arrival.key);
        let present = &mut self.present;
        self.input.arrivals.retain(|arrival| {
            if present.contains_key(&arrival.id) {
                return false;
            }
            present.insert(
                arrival.id,
                Presence {
                    key: arrival.key,
                    tile: arrival.tile,
                },
            );
            true
        });
    }

    fn take_due(&mut self, tick: u64) {
        self.due.clear();
        self.due_paths.clear();
        let issued = loop {
            match self.pipeline.take_due(tick) {
                Ok(due) => break due,
                Err(wait) => {
                    if self.stats.stage_lost {
                        self.pipeline.ready(IssuedDecisions {
                            tick: wait.issue_tick,
                            ..IssuedDecisions::default()
                        });
                        continue;
                    }
                    self.stats.waits = self.stats.waits.saturating_add(1);
                    if self.link.is_answered() || !self.link.wait_output() {
                        self.collect_outputs();
                        if self.pipeline.take_due(tick).is_ok() {
                            continue;
                        }
                        tracing::error!(
                            tick,
                            issue_tick = wait.issue_tick,
                            "the AI stage gave no decisions for a tick it was sent; the mobs go on without new decisions"
                        );
                        self.stats.stage_lost = true;
                        continue;
                    }
                    self.collect_outputs();
                }
            }
        };
        self.stats.lag_max = 0;
        for mut decisions in issued {
            self.due_paths.append(&mut decisions.paths);
            let lag = tick.saturating_sub(decisions.tick);
            if !decisions.is_empty() {
                self.stats.lag_max = self.stats.lag_max.max(lag);
                self.stats.lag_max_seen = self.stats.lag_max_seen.max(lag);
            }
            for decision in decisions.tiles.into_values().flatten() {
                if self.present.contains_key(&decision.mob) {
                    self.due.push(decision);
                }
            }
        }
        sort_paths(&mut self.due_paths);
        self.due
            .sort_by_key(|decision| (decision.key, decision.issue_tick));
    }

    fn publish(
        &mut self,
        threads: &mut Parallel<BrainOut>,
        mut publishing: Publishing<'_, '_, '_>,
    ) {
        let tick = publishing.tick;
        let mut gathered = std::mem::take(&mut self.gathered);
        let mut applied: usize = 0;
        for out in threads.iter_mut() {
            gathered.views.append(&mut out.views);
            gathered.facts.append(&mut out.facts);
            gathered.ended.append(&mut out.ended);
            gathered.moves.append(&mut out.moves);
            gathered.entities.append(&mut out.entities);
            gathered.hits.append(&mut out.hits);
            gathered.shots.append(&mut out.shots);
            gathered.proposals.append(&mut out.proposals);
            applied = applied.saturating_add(out.applied);
            out.applied = 0;
        }
        gathered.moves.sort_unstable_by_key(|moved| moved.mob);
        for moved in gathered.moves.drain(..) {
            if let Some(presence) = self.present.get_mut(&moved.mob) {
                presence.tile = moved.tile;
                self.input.moves.push((moved.mob, moved.tile));
            }
        }
        gathered
            .views
            .sort_unstable_by_key(|view| (TileKey::at(view.position), view.key));
        self.snapshots.clear();
        for views in gathered
            .views
            .chunk_by(|a, b| TileKey::at(a.position) == TileKey::at(b.position))
        {
            if let Some(first) = views.first() {
                let tile = TileKey::at(first.position);
                self.snapshots
                    .push((tile, TileSnapshot::new(tile, views.to_vec())));
            }
        }
        gathered.views.clear();
        let entities: Entities = gathered.entities.drain(..).collect();
        gathered
            .hits
            .sort_by_key(|hit| (hit.target.key(), hit.attacker_key));
        self.struck.clear();
        for hit in gathered.hits.drain(..) {
            self.request_hit(hit, &entities, &mut publishing);
        }
        self.struck.sort_unstable_by_key(|(target, _)| *target);
        gathered
            .proposals
            .sort_by_key(|proposed| (proposed.issue_tick, proposed.key));
        for proposed in gathered.proposals.drain(..) {
            self.grant(
                proposed,
                &entities,
                publishing.mobs,
                tick,
                &mut gathered.facts,
            );
        }
        gathered
            .shots
            .sort_by_key(|shot| (shot.attacker_key, shot.target));
        self.shots
            .extend(gathered.shots.drain(..).map(|shot| RangedAttack {
                attacker: shot.attacker,
                target: shot.target,
                power: shot.power,
            }));
        self.facts.clear();
        let ended = std::mem::take(&mut self.ended);
        for mob in ended {
            if let (Some(tile), Some(key)) = (self.tile_of(mob), self.key_of(mob)) {
                self.facts.push(TiledFact {
                    tile,
                    fact: Fact {
                        mob,
                        key,
                        kind: FactKind::PathEnded,
                    },
                });
            }
        }
        for fact in gathered.facts.drain(..) {
            if let Some(tile) = self.tile_of(fact.mob) {
                self.facts.push(TiledFact { tile, fact });
            }
        }
        self.facts.sort_by_key(|tiled| (tiled.tile, tiled.fact.mob));
        gathered.ended.sort_unstable();
        self.ended.append(&mut gathered.ended);
        self.stats.applied = applied;
        self.gathered = gathered;
        self.send(tick, publishing.players);
    }

    fn send(&mut self, tick: u64, players: PlayerTable) {
        let mut input = std::mem::replace(&mut self.input, TickInput::new(tick.saturating_add(1)));
        input.tick = tick;
        input.tiles = self.snapshots.clone();
        input
            .facts
            .extend(self.facts.iter().map(|tiled| (tiled.tile, tiled.fact)));
        input.players = players;
        self.stats.blocks_sent = input.blocks.landed.len() + input.blocks.patches.len();
        self.link.send(AiInput::Tick(Box::new(input)));
    }

    fn request_hit(
        &mut self,
        hit: MeleeHit,
        entities: &Entities,
        publishing: &mut Publishing<'_, '_, '_>,
    ) {
        let Some((amount, position)) = entities
            .get(&hit.attacker)
            .and_then(|entity| publishing.mobs.get(*entity).ok())
            .map(|(state, brain, _)| (brain.attack_damage, state.0.body.position))
        else {
            return;
        };
        let target = match hit.target {
            HitTarget::Player(target, _) => {
                self.player_hits.push(MobHit {
                    attacker: hit.attacker,
                    target,
                    amount,
                    damage_type: self.mob_attack,
                });
                publishing
                    .player_entities
                    .binary_search_by_key(&target, |(player, _)| *player)
                    .ok()
                    .and_then(|index| publishing.player_entities.get(index))
                    .map(|(_, entity)| *entity)
            }
            HitTarget::Mob(target, _) => entities.get(&target).copied(),
        };
        let (Some(target), Some(pending)) = (target, publishing.pending.as_deref_mut()) else {
            return;
        };
        let mut source = DamageSource::of(self.mob_attack);
        source.cause = Cause {
            causing: Some(hit.attacker_key),
            direct: Some(hit.attacker_key),
        };
        source.position = Some(position);
        pending.push(DamageRequest::new(
            target,
            Hit::new(amount, source),
            WriterKey::Entity {
                key: hit.attacker_key,
            },
            publishing.tick,
            0,
        ));
        if matches!(hit.target, HitTarget::Mob(..)) {
            self.struck.push((target, hit.attacker));
        }
    }

    fn grant(
        &mut self,
        proposed: Proposed,
        entities: &Entities,
        mobs: &mut Hitting<'_, '_>,
        tick: u64,
        facts: &mut Vec<Fact>,
    ) {
        match proposed.proposal {
            Proposal::Breed { partner } => {
                self.grant_breed(proposed.proposer, partner, entities, mobs, tick);
            }
            Proposal::Alert {
                target,
                reach,
                receivers,
            } => {
                self.grant_alert(
                    proposed.proposer,
                    target,
                    reach,
                    receivers,
                    entities,
                    mobs,
                    facts,
                );
            }
            Proposal::CaravanJoin { .. } | Proposal::CaravanLeave => {}
        }
    }

    fn grant_breed(
        &mut self,
        proposer: EntityId,
        partner: EntityId,
        entities: &Entities,
        mobs: &mut Hitting<'_, '_>,
        tick: u64,
    ) {
        let (Some(&mine), Some(&theirs)) = (entities.get(&proposer), entities.get(&partner)) else {
            return;
        };
        let Ok(
            [
                (my_state, mut my_brain, my_vitals),
                (their_state, mut their_brain, their_vitals),
            ],
        ) = mobs.get_many_mut([mine, theirs])
        else {
            return;
        };
        let alive = |vitals: Option<&Vitals>| vitals.is_none_or(Vitals::is_alive);
        if !alive(my_vitals) || !alive(their_vitals) {
            return;
        }
        if my_state
            .0
            .body
            .position
            .distance_squared(their_state.0.body.position)
            >= BREED_DISTANCE_SQUARED
        {
            return;
        }
        let ready = |brain: &BrainState| brain.love > 0 && brain.panic == Panic::Calm;
        if !ready(&my_brain) || !ready(&their_brain) {
            return;
        }
        let kind = my_brain.kind;
        let (position, key) = (my_state.0.body.position, my_state.0.key);
        my_brain.bred();
        their_brain.bred();
        self.births = self.births.wrapping_add(1);
        let child = EntityUuid::born(
            self.config.seed().0,
            "",
            tick,
            &BirthSource::Entity(key),
            self.births,
        );
        self.spawns.push(MobSpawnRequest {
            kind,
            position,
            uuid: child.as_u128(),
            setup: SpawnSetup::None,
            extras: MobExtras {
                no_ai: false,
                custom_name: None,
                tags: Vec::new(),
                baby: true,
            },
        });
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the grant reads the proposer, the hit and the host maps; a struct would hold borrows of all of them"
    )]
    fn grant_alert(
        &self,
        proposer: EntityId,
        target: EntityId,
        reach: f32,
        receivers: Receivers,
        entities: &Entities,
        mobs: &mut Hitting<'_, '_>,
        facts: &mut Vec<Fact>,
    ) {
        let Some(tile) = self.tile_of(proposer) else {
            return;
        };
        let Some((feet, kind, owner)) = entities
            .get(&proposer)
            .and_then(|entity| mobs.get(*entity).ok())
            .map(|(state, brain, _)| (state.0.body.position, brain.kind, brain.owner))
        else {
            return;
        };
        let reach = f64::from(reach);
        let min = DVec3::new(feet.x - reach, feet.y - ALERT_HEIGHT, feet.z - reach);
        let max = DVec3::new(
            feet.x + 1.0 + reach,
            feet.y + 1.0 + ALERT_HEIGHT,
            feet.z + 1.0 + reach,
        );
        let kinds = match receivers {
            Receivers::SameKind => KindSet::NONE.with(MobKind(kind.entity_type.id().number())),
            Receivers::Kinds(kinds) => *kinds,
        };
        let neighbours = Neighbours::new(tile, |tile| tile_views(&self.snapshots, tile));
        let mut found: Vec<(EntityId, EntityUuid)> = Vec::new();
        neighbours.for_each_in(min, max, &kinds, |view| {
            if view.mob != proposer && view.mob != target && view.target.is_none() {
                found.push((view.mob, view.key));
            }
        });
        for (receiver, key) in found {
            let Some(&entity) = entities.get(&receiver) else {
                continue;
            };
            let Ok((_, mut brain, _)) = mobs.get_mut(entity) else {
                continue;
            };
            if brain.target.is_none() && brain.owner == owner {
                brain.target = EntityRef::of(target);
                facts.push(Fact {
                    mob: receiver,
                    key,
                    kind: FactKind::Alerted,
                });
            }
        }
    }
}

impl<V: BlockVolume> Pass<'_, V> {
    #[expect(
        clippy::too_many_arguments,
        reason = "one mob of the parallel query: each component of the query row is its own parameter"
    )]
    fn think(
        &self,
        out: &mut BrainOut,
        entity: Entity,
        id: EntityId,
        sensed: Sensed<'_>,
        attributes: Option<&MobAttributes>,
        mut brain: Mut<'_, BrainState>,
        mut intent: Mut<'_, MobIntent>,
        mut navigation: Mut<'_, MobNavigation>,
    ) {
        let state = sensed.state;
        let mut next = *brain;
        let navigation = &mut navigation.bypass_change_detection().0;
        let observed = next.observe(
            self.panic_causes,
            self.players,
            attributes,
            self.tick,
            id,
            sensed,
        );
        let kinds = [
            (observed.struck, FactKind::Hurt),
            (observed.idle_reset, FactKind::IdleReset),
            (observed.resumed, FactKind::Resumed),
        ];
        for (set, kind) in kinds {
            if set {
                out.facts.push(Fact {
                    mob: id,
                    key: state.key,
                    kind,
                });
            }
        }
        out.views.push(observed.view);
        out.entities.push((id, entity));
        let first = self
            .due
            .partition_point(|decision| decision.key < state.key);
        let due = self.due.get(first..).unwrap_or_default();
        for decision in due.iter().take_while(|decision| decision.key == state.key) {
            if decision.mob != id {
                continue;
            }
            let found = decision
                .intents
                .path
                .and_then(|path| find_path(self.due_paths, id, path.ticket()));
            next.apply(&decision.intents, state, navigation, found, self.world);
            out.applied = out.applied.saturating_add(1);
            if let Some(proposal) = decision.intents.proposal {
                out.proposals.push(Proposed {
                    proposer: id,
                    key: state.key,
                    issue_tick: decision.issue_tick,
                    proposal,
                });
            }
        }
        if let Some(steered) =
            next.steer(self.world, &self.targets, self.tick, id, sensed, navigation)
        {
            intent.set_if_neq(steered.intent);
            if steered.ended.is_some() {
                out.ended.push(id);
            }
            if let Some(hit) = steered.hit {
                out.hits.push(hit);
            }
            if let Some(shot) = steered.shot {
                out.shots.push(shot);
            }
        }
        brain.set_if_neq(next);
        let tile = TileKey::at(state.body.position);
        if self.targets.present.get(&id).map(|presence| presence.tile) != Some(tile) {
            out.moves.push(Move { mob: id, tile });
        }
    }
}
