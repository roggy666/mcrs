use std::time::Instant;

use bevy_ecs::change_detection::DetectChangesMut;
use bevy_ecs::entity::Entity;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::Query;
use bevy_ecs::world::Mut;
use bevy_tasks::TaskPool;
use bevy_utils::Parallel;
use glam::DVec3;
use mcrs_minecraft_brain::AiDriver;
use mcrs_minecraft_brain::Cadence;
use mcrs_minecraft_brain::Decision;
use mcrs_minecraft_brain::EntityRef;
use mcrs_minecraft_brain::Fact;
use mcrs_minecraft_brain::FactKind;
use mcrs_minecraft_brain::HurtBy;
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
use mcrs_minecraft_brain::Stage;
use mcrs_minecraft_brain::StageConfig;
use mcrs_minecraft_brain::Tables;
use mcrs_minecraft_brain::TileSnapshot;
use mcrs_minecraft_damage::Hit;
use mcrs_minecraft_damage::HitOutcome;
use mcrs_minecraft_engine_entity_motion::MobIntent;
use mcrs_minecraft_engine_entity_motion::MotionId;
use mcrs_minecraft_engine_entity_motion::MotionState;
use mcrs_minecraft_engine_mob_attributes::MobAttributes;
use mcrs_minecraft_engine_physics::MobChanges;
use mcrs_minecraft_engine_physics::MobExtras;
use mcrs_minecraft_engine_physics::MobSpawnRequest;
use mcrs_minecraft_engine_physics::SpawnSetup;
use mcrs_minecraft_engine_physics::hurt_mob;
use mcrs_minecraft_entity::EntityTypeData;
use mcrs_minecraft_entity_key::BirthSource;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_entity_key::TileKey;
use mcrs_minecraft_goal_world::GoalWorld;
use mcrs_minecraft_physics::EntityState;
use mcrs_minecraft_volume::BlockVolume;
use rustc_hash::FxHashMap;

use crate::AiStats;
use crate::BlockGoalWorld;
use crate::BrainState;
use crate::MobHit;
use crate::RangedAttack;
use crate::brain_state_ext::BrainStateExt;
use crate::hit_queue::HitTarget;
use crate::hit_queue::MeleeHit;
use crate::hit_queue::Proposed;
use crate::hit_queue::RangedShot;
use crate::run_tiles::run_tiles;
use crate::targets::Targets;

const BREED_DISTANCE_SQUARED: f64 = 9.0;
const ALERT_HEIGHT: f64 = 10.0;

pub(crate) type Thinking<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static MotionId,
        &'static mut MotionState,
        Option<&'static MobAttributes>,
        &'static mut BrainState,
        &'static mut MobIntent,
    ),
>;

type Entities = FxHashMap<EntityId, Entity>;

#[derive(Resource, Debug)]
pub struct MobAi<B: Send + Sync + 'static> {
    driver: AiDriver<B>,
    brain_for: fn(&'static EntityTypeData) -> Option<B>,
    panic_causes: PanicCauses,
    kinds: KindTable,
    items: ItemTables,
    mob_attack: u32,
    arrivals: Vec<Arrival<B>>,
    departures: Vec<EntityId>,
    due: Vec<Decision>,
    ended: Vec<EntityId>,
    gathered: BrainOut,
    facts: Vec<TiledFact>,
    player_hits: Vec<MobHit>,
    shots: Vec<RangedAttack>,
    spawns: Vec<MobSpawnRequest>,
    births: u32,
    stats: AiStats,
}

#[derive(Debug)]
pub(crate) struct Arrival<B> {
    pub(crate) id: EntityId,
    pub(crate) key: EntityUuid,
    pub(crate) tile: TileKey,
    pub(crate) phase: u8,
    pub(crate) brain: B,
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

struct Pass<'a, B, V> {
    tick: u64,
    players: &'a PlayerTable,
    panic_causes: &'a PanicCauses,
    due: &'a [Decision],
    targets: Targets<'a, B>,
    world: &'a BlockGoalWorld<'a, V>,
}

impl<B: Send + Sync + 'static> MobAi<B> {
    #[must_use]
    pub fn new(
        config: StageConfig,
        brain_for: fn(&'static EntityTypeData) -> Option<B>,
        kinds: KindTable,
    ) -> Self {
        Self {
            driver: AiDriver::new(config),
            brain_for,
            panic_causes: PanicCauses::default(),
            kinds,
            items: ItemTables::default(),
            mob_attack: 0,
            arrivals: Vec::new(),
            departures: Vec::new(),
            due: Vec::new(),
            ended: Vec::new(),
            gathered: BrainOut::default(),
            facts: Vec::new(),
            player_hits: Vec::new(),
            shots: Vec::new(),
            spawns: Vec::new(),
            births: 0,
            stats: AiStats::default(),
        }
    }

    pub fn set_panic_causes(&mut self, panic_causes: PanicCauses) {
        self.panic_causes = panic_causes;
    }

    pub fn set_items(&mut self, items: ItemTables) {
        self.items = items;
    }

    pub fn set_mob_attack(&mut self, damage_type: u32) {
        self.mob_attack = damage_type;
    }

    #[must_use]
    pub const fn stage(&self) -> &Stage<B> {
        self.driver.stage()
    }

    #[must_use]
    pub const fn tick(&self) -> u64 {
        self.driver.tick()
    }

    #[must_use]
    pub const fn stats(&self) -> &AiStats {
        &self.stats
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

    pub(crate) fn brain_for(&self, kind: &'static EntityTypeData) -> Option<B> {
        (self.brain_for)(kind)
    }

    pub(crate) fn admit(&mut self, arrival: Arrival<B>) {
        self.arrivals.push(arrival);
    }

    pub(crate) fn release(&mut self, mob: EntityId) {
        self.arrivals.retain(|arrival| arrival.id != mob);
        self.departures.push(mob);
    }

    pub(crate) fn prepare<V: BlockVolume + Sync>(
        &mut self,
        world: &BlockGoalWorld<'_, V>,
        players: PlayerTable,
        mobs: &mut Thinking<'_, '_>,
        changes: &mut MobChanges,
        threads: &mut Parallel<BrainOut>,
    ) {
        let started = Instant::now();
        let tick = self.driver.begin_tick();
        self.follow_life_cycle(tick);
        let followed = Instant::now();
        self.take_due();
        let pass = Pass {
            tick,
            players: &players,
            panic_causes: &self.panic_causes,
            due: &self.due,
            targets: Targets {
                replica: self.driver.replica(),
                stage: self.driver.stage(),
                players: &players,
                kinds: &self.kinds,
                tick,
            },
            world,
        };
        let shared = &*threads;
        mobs.par_iter_mut().for_each_init(
            || shared.borrow_local_mut(),
            |out, (entity, id, state, attributes, brain, intent)| {
                pass.think(out, entity, id.0, &state.0, attributes, brain, intent);
            },
        );
        let ran = Instant::now();
        self.publish(threads, players, mobs, changes, tick);
        self.stats.life_cycle = followed.duration_since(started);
        self.stats.tile_tasks = ran.duration_since(followed);
        self.stats.publish = ran.elapsed();
        self.stats.prepare = started.elapsed();
    }

    pub fn run_stage<W: GoalWorld + Sync>(&mut self, pool: &TaskPool, world: &W)
    where
        B: MobBrain<W>,
    {
        let started = Instant::now();
        let evaluations = self.driver.stage().evaluations();
        let tables = Tables {
            items: &self.items,
            kinds: &self.kinds,
        };
        self.stats.decisions = self
            .driver
            .run_stage(world, &tables, |work, cx| run_tiles(pool, work, cx));
        self.stats.evaluations = self
            .driver
            .stage()
            .evaluations()
            .saturating_sub(evaluations);
        self.stats.stage = started.elapsed();
    }

    fn follow_life_cycle(&mut self, tick: u64) {
        for mob in self.departures.drain(..) {
            self.driver.despawn(mob);
        }
        self.arrivals.sort_unstable_by_key(|arrival| arrival.key);
        for arrival in self.arrivals.drain(..) {
            if self.driver.stage().tile_of(arrival.id).is_some() {
                continue;
            }
            let cadence = Cadence::new(tick, arrival.phase);
            self.driver.spawn(
                arrival.tile,
                arrival.id,
                arrival.key,
                cadence,
                arrival.brain,
            );
        }
        self.stats.mobs = self.driver.stage().len();
    }

    fn take_due(&mut self) {
        self.due.clear();
        let issued = self.driver.take_due();
        self.stats.waits = self.driver.waits();
        for decisions in issued {
            for decision in decisions.tiles.into_values().flatten() {
                if self.driver.stage().tile_of(decision.mob).is_some() {
                    self.due.push(decision);
                }
            }
        }
        self.due
            .sort_by_key(|decision| (decision.key, decision.issue_tick));
    }

    fn publish(
        &mut self,
        threads: &mut Parallel<BrainOut>,
        players: PlayerTable,
        mobs: &mut Thinking<'_, '_>,
        changes: &mut MobChanges,
        tick: u64,
    ) {
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
        self.driver.relocate_all(
            gathered
                .moves
                .drain(..)
                .map(|moved| (moved.mob, moved.tile)),
        );
        gathered
            .views
            .sort_unstable_by_key(|view| (TileKey::at(view.position), view.key));
        for views in gathered
            .views
            .chunk_by(|a, b| TileKey::at(a.position) == TileKey::at(b.position))
        {
            if let Some(first) = views.first() {
                let tile = TileKey::at(first.position);
                self.driver
                    .publish_tile(tile, TileSnapshot::new(tile, views.to_vec()));
            }
        }
        gathered.views.clear();
        let entities: Entities = gathered.entities.drain(..).collect();
        gathered
            .hits
            .sort_by_key(|hit| (hit.target.key(), hit.attacker_key));
        for hit in gathered.hits.drain(..) {
            self.apply_hit(hit, &entities, mobs, changes, tick, &mut gathered.facts);
        }
        gathered
            .proposals
            .sort_by_key(|proposed| (proposed.issue_tick, proposed.key));
        for proposed in gathered.proposals.drain(..) {
            self.grant(proposed, &entities, mobs, tick, &mut gathered.facts);
        }
        self.shots
            .extend(gathered.shots.drain(..).map(|shot| RangedAttack {
                attacker: shot.attacker,
                target: shot.target,
                power: shot.power,
            }));
        self.facts.clear();
        for mob in self.ended.drain(..) {
            if let (Some(tile), Some(key)) = (
                self.driver.stage().tile_of(mob),
                self.driver.stage().key_of(mob),
            ) {
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
            if let Some(tile) = self.driver.stage().tile_of(fact.mob) {
                self.facts.push(TiledFact { tile, fact });
            }
        }
        self.facts.sort_by_key(|tiled| (tiled.tile, tiled.fact.mob));
        for facts in self.facts.chunk_by(|a, b| a.tile == b.tile) {
            if let Some(first) = facts.first() {
                self.driver
                    .publish_facts(first.tile, facts.iter().map(|tiled| tiled.fact));
            }
        }
        gathered.ended.sort_unstable();
        self.ended.append(&mut gathered.ended);
        self.driver.publish_players(players);
        self.stats.applied = applied;
        self.gathered = gathered;
    }

    fn apply_hit(
        &mut self,
        hit: MeleeHit,
        entities: &Entities,
        mobs: &mut Thinking<'_, '_>,
        changes: &mut MobChanges,
        tick: u64,
        facts: &mut Vec<Fact>,
    ) {
        let Some(amount) = entities
            .get(&hit.attacker)
            .and_then(|entity| mobs.get(*entity).ok())
            .map(|(_, _, _, _, brain, _)| brain.attack_damage)
        else {
            return;
        };
        match hit.target {
            HitTarget::Player(target, _) => self.player_hits.push(MobHit {
                attacker: hit.attacker,
                target,
                amount,
                damage_type: self.mob_attack,
            }),
            HitTarget::Mob(target, key) => {
                let Some(&entity) = entities.get(&target) else {
                    return;
                };
                let Ok((_, _, mut state, _, mut brain, _)) = mobs.get_mut(entity) else {
                    return;
                };
                let outcome = hurt_mob(
                    entity,
                    &mut state.0,
                    changes,
                    Hit::new(amount, self.mob_attack),
                    tick,
                );
                if outcome.is_some_and(|outcome| outcome != HitOutcome::Rejected) {
                    brain.hurt_by = HurtBy::by(hit.attacker, tick as u32);
                    brain.love = 0;
                    facts.push(Fact {
                        mob: target,
                        key,
                        kind: FactKind::Hurt,
                    });
                }
            }
        }
    }

    fn grant(
        &mut self,
        proposed: Proposed,
        entities: &Entities,
        mobs: &mut Thinking<'_, '_>,
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
                    tick,
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
        mobs: &mut Thinking<'_, '_>,
        tick: u64,
    ) {
        let (Some(&mine), Some(&theirs)) = (entities.get(&proposer), entities.get(&partner)) else {
            return;
        };
        let Ok(
            [
                (_, _, my_state, _, mut my_brain, _),
                (_, _, their_state, _, mut their_brain, _),
            ],
        ) = mobs.get_many_mut([mine, theirs])
        else {
            return;
        };
        let alive = |state: &EntityState| state.living.is_some_and(|living| living.is_alive());
        if !alive(&my_state.0) || !alive(&their_state.0) {
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
            self.driver.stage().config().seed().0,
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
        mobs: &mut Thinking<'_, '_>,
        tick: u64,
        facts: &mut Vec<Fact>,
    ) {
        let Some(tile) = self.driver.stage().tile_of(proposer) else {
            return;
        };
        let Some((feet, kind, owner)) = entities
            .get(&proposer)
            .and_then(|entity| mobs.get(*entity).ok())
            .map(|(_, _, state, _, brain, _)| (state.0.body.position, brain.kind, brain.owner))
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
        let replica = self.driver.replica();
        let neighbours = Neighbours::new(tile, |tile| replica.tile(tile, tick));
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
            let Ok((_, _, _, _, mut brain, _)) = mobs.get_mut(entity) else {
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

impl<B, V: BlockVolume> Pass<'_, B, V> {
    #[expect(
        clippy::too_many_arguments,
        reason = "one mob of the parallel query: each component of the query row is its own parameter"
    )]
    fn think(
        &self,
        out: &mut BrainOut,
        entity: Entity,
        id: EntityId,
        state: &EntityState,
        attributes: Option<&MobAttributes>,
        mut brain: Mut<'_, BrainState>,
        mut intent: Mut<'_, MobIntent>,
    ) {
        let mut next = *brain;
        let observed = next.observe(
            self.panic_causes,
            self.players,
            attributes,
            self.tick,
            id,
            state,
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
            next.apply(&decision.intents, state);
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
        if let Some(steered) = next.steer(self.world, &self.targets, self.tick, id, state) {
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
        if self.targets.stage.tile_of(id) != Some(tile) {
            out.moves.push(Move { mob: id, tile });
        }
    }
}
