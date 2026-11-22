use std::collections::VecDeque;
use std::convert::Infallible;
use std::time::Instant;

use bevy_tasks::TaskPool;
use mcrs_engine_link::Stage as LinkStage;
use mcrs_engine_link::Step;
use mcrs_minecraft_brain::Cadence;
use mcrs_minecraft_brain::MobBrain;
use mcrs_minecraft_brain::Replica;
use mcrs_minecraft_brain::Stage;
use mcrs_minecraft_brain::StageConfig;
use mcrs_minecraft_brain::TickContext;
use mcrs_minecraft_nav_projection::NavDelta;
use mcrs_minecraft_nav_projection::NavProjection;
use mcrs_minecraft_nav_projection::NavSection;
use mcrs_minecraft_nav_projection::NavTable;
use mcrs_minecraft_volume::PalettedSection;

use crate::AiInput;
use crate::AiOutput;
use crate::BlockDelta;
use crate::ProjectionWorld;
use crate::RunStats;
use crate::StageTables;
use crate::TickInput;
use crate::lower_priority;
use crate::run_tiles;

pub struct AiStage<B> {
    queue: VecDeque<AiInput<B>>,
    stage: Stage<B>,
    replica: Replica,
    projection: NavProjection,
    tables: Option<Box<StageTables>>,
    pool: TaskPool,
    lowered: bool,
}

impl<B> AiStage<B> {
    #[must_use]
    pub fn new(config: StageConfig, pool: TaskPool) -> Self {
        Self {
            queue: VecDeque::new(),
            stage: Stage::new(config),
            replica: Replica::default(),
            projection: NavProjection::new(NavTable::default(), 0, 0),
            tables: None,
            pool,
            lowered: false,
        }
    }

    #[must_use]
    pub const fn stage(&self) -> &Stage<B> {
        &self.stage
    }

    #[must_use]
    pub const fn replica(&self) -> &Replica {
        &self.replica
    }

    #[must_use]
    pub const fn projection(&self) -> &NavProjection {
        &self.projection
    }

    #[must_use]
    pub fn tables(&self) -> Option<&StageTables> {
        self.tables.as_deref()
    }

    fn set_tables(&mut self, tables: Box<StageTables>) {
        self.projection =
            NavProjection::new(tables.nav.clone(), tables.height.min, tables.height.max);
        self.tables = Some(tables);
    }

    fn follow_life_cycle(&mut self, input: &mut TickInput<B>) {
        for mob in input.departures.drain(..) {
            self.stage.despawn(mob);
        }
        input.arrivals.sort_unstable_by_key(|arrival| arrival.key);
        for arrival in input.arrivals.drain(..) {
            if self.stage.tile_of(arrival.id).is_some() {
                continue;
            }
            let cadence = Cadence::new(input.tick, arrival.phase);
            self.stage.spawn(
                arrival.tile,
                arrival.id,
                arrival.key,
                cadence,
                arrival.brain,
            );
        }
        self.stage.relocate_all(input.moves.drain(..));
    }

    fn publish(&mut self, input: &mut TickInput<B>) {
        let tick = input.tick;
        for (tile, snapshot) in input.tiles.drain(..) {
            self.replica.publish_tile(tick, tile, snapshot);
        }
        input.facts.sort_by_key(|(tile, fact)| (*tile, fact.mob));
        for facts in input.facts.chunk_by(|a, b| a.0 == b.0) {
            if let Some((tile, _)) = facts.first() {
                self.replica
                    .publish_facts(tick, *tile, facts.iter().map(|(_, fact)| *fact));
            }
        }
        self.replica
            .publish_players(tick, std::mem::take(&mut input.players));
    }

    fn project(&mut self, tick: u64, blocks: BlockDelta) {
        let table = self.projection.table();
        let delta = NavDelta {
            landed: blocks
                .landed
                .into_iter()
                .map(|landed| {
                    let section = NavSection::of(&PalettedSection(&landed.blocks.0.0), table);
                    (landed.pos, section)
                })
                .collect(),
            unloaded: blocks.unloaded,
            patches: blocks.patches,
        };
        self.projection.apply(tick, delta);
    }
}

impl<B> AiStage<B>
where
    B: for<'a> MobBrain<ProjectionWorld<'a>> + Send,
{
    fn run_tick(&mut self, mut input: TickInput<B>) -> AiOutput {
        let started = Instant::now();
        let tick = input.tick;
        self.follow_life_cycle(&mut input);
        self.publish(&mut input);
        let blocks = std::mem::take(&mut input.blocks);
        self.project(tick, blocks);
        let before = self.stage.evaluations();
        let max_lag = u64::from(self.stage.config().max_lag());
        let Some(tables) = self.tables.as_deref() else {
            return AiOutput {
                tick,
                decisions: self.stage.collect(tick),
                run: RunStats {
                    stage: started.elapsed(),
                    without_tables: true,
                    ..RunStats::default()
                },
            };
        };
        let world = ProjectionWorld::new(self.projection.at(tick), tables);
        let cx = TickContext {
            tick,
            config: *self.stage.config(),
            replica: &self.replica,
            items: &tables.items,
            kinds: &tables.kinds,
            world: &world,
        };
        let mut work = self.stage.work();
        run_tiles(&self.pool, &mut work, &cx);
        drop(work);
        let decisions = self.stage.collect(tick);
        self.replica.retire(tick.saturating_add(1));
        self.projection.retire(tick.saturating_sub(max_lag));
        AiOutput {
            tick,
            run: RunStats {
                stage: started.elapsed(),
                evaluations: self.stage.evaluations().saturating_sub(before),
                decisions: decisions.len(),
                mobs: self.stage.len(),
                sections: self.projection.sections(),
                projection_bytes: self.projection.heap_bytes(),
                without_tables: false,
            },
            decisions,
        }
    }
}

impl<B> LinkStage for AiStage<B>
where
    B: for<'a> MobBrain<ProjectionWorld<'a>> + Send + 'static,
{
    const NAME: &'static str = "ai";
    type Input = AiInput<B>;
    type Output = AiOutput;
    type TaskOutput = Infallible;

    fn receive(&mut self, input: Self::Input) {
        self.queue.push_back(input);
    }

    fn step(&mut self, step: &mut Step<'_, Self>) {
        if !self.lowered {
            lower_priority();
            self.lowered = true;
        }
        let Some(input) = self.queue.pop_front() else {
            return;
        };
        match input {
            AiInput::Tables(tables) => self.set_tables(tables),
            AiInput::Tick(tick) => {
                let output = self.run_tick(*tick);
                step.emit(output);
            }
        }
    }

    fn has_work(&self) -> bool {
        !self.queue.is_empty()
    }
}
