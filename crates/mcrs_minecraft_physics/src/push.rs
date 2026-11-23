use glam::DVec3;
use mcrs_minecraft_entity_key::KeyedStream;

use crate::BodyView;
use crate::CollisionRule;
use crate::GridFlags;
use crate::GridRow;
use crate::PushTargets;
use crate::Scratch;

pub const MIN_PUSH_DISTANCE: f32 = 0.01;

const STRENGTH: f32 = 0.05;

pub const CROWD_DAMAGE: f32 = 6.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PushRules {
    pub max_entity_cramming: i32,
    pub cap: usize,
}

impl Default for PushRules {
    fn default() -> Self {
        Self {
            max_entity_cramming: 24,
            cap: Self::DEFAULT_CAP,
        }
    }
}

impl PushRules {
    pub const DEFAULT_CAP: usize = 8;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PushOutcome {
    pub pushed: bool,
    pub crowd_damage: bool,
    pub candidates: u32,
    pub free_candidates: u32,
}

#[must_use]
pub fn push_vector(pusher: &GridRow, candidate: &GridRow) -> Option<DVec3> {
    if pusher.root_vehicle == candidate.root_vehicle
        || pusher.flags.contains(GridFlags::NO_PHYSICS)
        || candidate.flags.contains(GridFlags::NO_PHYSICS)
    {
        return None;
    }
    let dx = pusher.position.x - candidate.position.x;
    let dz = pusher.position.z - candidate.position.z;
    let xa = push_axis(dx, dz)?;
    let za = push_axis(dz, dx)?;
    (xa.is_finite() && za.is_finite()).then_some(DVec3::new(xa, 0.0, za))
}

#[must_use]
pub fn push_axis(along: f64, across: f64) -> Option<f64> {
    let largest = along.abs().max(across.abs());
    if largest < f64::from(MIN_PUSH_DISTANCE) {
        return None;
    }
    let root = largest.sqrt();
    Some(along / root * (1.0 / root).min(1.0) * f64::from(STRENGTH))
}

#[must_use]
pub fn pushes(pusher: &GridRow, candidate: &GridRow) -> bool {
    let target = match pusher.push_targets {
        PushTargets::None => false,
        PushTargets::Each => true,
        PushTargets::NotPlayers => !candidate.flags.contains(GridFlags::PLAYER),
    };
    target
        && !candidate.flags.contains(GridFlags::SPECTATOR)
        && candidate.flags.contains(GridFlags::PUSHABLE)
        && team_allows(pusher, candidate)
}

fn team_allows(pusher: &GridRow, candidate: &GridRow) -> bool {
    let (own, other) = (pusher.team.rule, candidate.team.rule);
    if own == CollisionRule::Never || other == CollisionRule::Never {
        return false;
    }
    let same = pusher.team.team.is_some() && pusher.team.team == candidate.team.team;
    if (own == CollisionRule::PushOwnTeam || other == CollisionRule::PushOwnTeam) && same {
        return false;
    }
    if own != CollisionRule::PushOtherTeams && other != CollisionRule::PushOtherTeams {
        return true;
    }
    same
}

fn takes_push(row: &GridRow) -> bool {
    !row.flags.contains(GridFlags::CARRIES_PASSENGER) && row.flags.contains(GridFlags::PUSHABLE)
}

fn staggered_push(of: &GridRow, other: &GridRow) -> Option<DVec3> {
    let offset = of.position - other.position;
    if offset.x.abs().max(offset.z.abs()) >= f64::from(MIN_PUSH_DISTANCE) {
        return None;
    }
    let first = of.key < other.key;
    let (earlier, later) = if first { (of, other) } else { (other, of) };
    let before = GridRow {
        position: later.previous,
        ..*later
    };
    if !pushes(earlier, &before) || !before.bounds().intersects(&earlier.bounds()) {
        return None;
    }
    let push = push_vector(earlier, &before)?;
    Some(if first { push } else { -push })
}

pub fn push_entity<B: BodyView>(
    bodies: &B,
    of: &GridRow,
    rules: &PushRules,
    stream: &KeyedStream,
    velocity: &mut DVec3,
    scratch: &mut Scratch,
) -> PushOutcome {
    let neighbours = &mut scratch.neighbours;
    neighbours.clear();
    let (mut candidates, mut free) = (0u32, 0u32);
    bodies.neighbours_up_to(of, neighbours, rules.cap, |other| {
        if pushes(of, other) {
            candidates = candidates.saturating_add(1);
            if !other.flags.contains(GridFlags::IS_PASSENGER) {
                free = free.saturating_add(1);
            }
        }
    });
    let mut outcome = PushOutcome {
        candidates,
        ..PushOutcome::default()
    };
    let limit = rules.max_entity_cramming;
    if limit > 0 && i64::from(candidates) > i64::from(limit) - 1 && stream.below(4) == 0 {
        outcome.free_candidates = free;
        outcome.crowd_damage = i64::from(free) > i64::from(limit) - 1;
    }
    neighbours.sort_by_key(|other| other.key);
    for other in neighbours.iter() {
        let mut add = |push: DVec3, outcome: &mut PushOutcome| {
            *velocity += push;
            outcome.pushed = true;
        };
        if pushes(other, of)
            && takes_push(of)
            && let Some(push) = push_vector(other, of)
        {
            add(-push, &mut outcome);
        }
        if pushes(of, other)
            && takes_push(of)
            && let Some(push) = push_vector(of, other)
        {
            add(push, &mut outcome);
        }
        if takes_push(of)
            && let Some(push) = staggered_push(of, other)
        {
            add(push, &mut outcome);
        }
    }
    outcome
}
