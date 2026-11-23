use glam::DVec3;
use mcrs_minecraft_path_type::PathType;
use mcrs_voxel::BlockPos;

use crate::Navigation;
use crate::PathWorld;
use crate::ground::normalized;
use crate::world::floor_level;

const STUCK_TICKS: u64 = 100;
const STUCK_FACTOR: f32 = 0.25;
const CORNER_DISTANCE: f64 = 4.0;
const CORNER_CLOSE: f64 = 0.5;
const TIMEOUT_FACTOR: f64 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FollowInput {
    pub position: DVec3,
    pub on_ground: bool,
    pub in_liquid: bool,
    pub passenger: bool,
    pub width: f32,
    pub height: f32,
    pub speed: f32,
    pub time: u64,
    pub flight: Option<Flight>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Flight {
    pub can_float: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PathEnd {
    Done,
    Stuck,
    TimedOut,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FollowStep {
    pub wanted: Option<DVec3>,
    pub speed_modifier: f32,
    pub ended: Option<PathEnd>,
}

pub fn follow<W: PathWorld + ?Sized>(
    navigation: &mut Navigation,
    input: &FollowInput,
    world: &W,
) -> FollowStep {
    navigation.ticks = navigation.ticks.saturating_add(1);
    let speed_modifier = navigation.speed_modifier;
    if !navigation.is_running() {
        return FollowStep {
            wanted: None,
            speed_modifier,
            ended: None,
        };
    }
    let can_update = match input.flight {
        Some(flight) => {
            crate::can_update_path_flying(flight.can_float, input.in_liquid, input.passenger)
        }
        None => crate::can_update_path(input.on_ground, input.in_liquid, input.passenger),
    };
    if can_update {
        advance_on_ground(navigation, input, world);
        if let Some(end) = stuck_check(navigation, input) {
            return ended(navigation, end, speed_modifier);
        }
    } else if input.flight.is_some() {
        advance_in_flight(navigation, input);
    } else {
        advance_in_air(navigation, input);
    }
    if !navigation.is_running() {
        return ended(navigation, PathEnd::Done, speed_modifier);
    }
    if timed_out(navigation, input) {
        navigation.reset_timeout();
        return ended(navigation, PathEnd::TimedOut, speed_modifier);
    }
    let Some(next) = navigation.path.as_ref().and_then(|path| path.next_node()) else {
        return ended(navigation, PathEnd::Done, speed_modifier);
    };
    let point = next.point(input.width);
    let block = BlockPos::from(point);
    let below = world.path_cell(block.below());
    let y = if input.flight.is_some() || below.is_air() {
        point.y
    } else {
        floor_level(world, block, false)
    };
    FollowStep {
        wanted: Some(DVec3::new(point.x, y, point.z)),
        speed_modifier,
        ended: None,
    }
}

fn ended(navigation: &mut Navigation, end: PathEnd, speed_modifier: f32) -> FollowStep {
    navigation.drop_path();
    navigation.stuck_end = end == PathEnd::Stuck;
    FollowStep {
        wanted: None,
        speed_modifier,
        ended: Some(end),
    }
}

fn advance_on_ground<W: PathWorld + ?Sized>(
    navigation: &mut Navigation,
    input: &FollowInput,
    world: &W,
) {
    let position = input.position;
    let Some(path) = navigation.path.as_mut() else {
        return;
    };
    let width = input.width;
    let limit = f64::from(if width > 0.75 {
        width / 2.0
    } else {
        0.75 - width / 2.0
    });
    let mut advanced = false;
    if let Some(next) = path.next_node() {
        let close = (position.x - (f64::from(next.x) + 0.5)).abs() < limit
            && (position.z - (f64::from(next.z) + 0.5)).abs() < limit
            && (position.y - f64::from(next.y)).abs() < 1.0;
        let direct = input.flight.is_some().then(|| {
            let point = next.point(width);
            DVec3::new(point.x, point.y + f64::from(input.height) * 0.5, point.z)
        });
        let clear = |to: DVec3| world.clear_between(position, to);
        if close || corner_rule(path.next_node(), path.after_next(), position, direct, clear) {
            path.advance();
            advanced = true;
        }
    }
    if advanced {
        navigation.reset_stuck(position);
    }
}

fn corner_rule(
    next: Option<&crate::PathNode>,
    after: Option<&crate::PathNode>,
    position: DVec3,
    direct: Option<DVec3>,
    clear: impl Fn(DVec3) -> bool,
) -> bool {
    let (Some(next), Some(after)) = (next, after) else {
        return false;
    };
    if matches!(
        next.kind,
        PathType::FireInNeighbor | PathType::DamagingInNeighbor | PathType::WalkableDoor
    ) {
        return false;
    }
    let c = next.bottom_centre();
    let e = after.bottom_centre();
    if position.distance_squared(c) >= CORNER_DISTANCE {
        return false;
    }
    if direct.is_some_and(clear) {
        return true;
    }
    let u = c - position;
    let v = e - position;
    let u_len = u.length_squared();
    let v_len = v.length_squared();
    if !(v_len < u_len || u_len < CORNER_CLOSE) {
        return false;
    }
    normalized(v).dot(normalized(u)) < 0.0
}

fn advance_in_air(navigation: &mut Navigation, input: &FollowInput) {
    let Some(path) = navigation.path.as_mut() else {
        return;
    };
    let position = input.position;
    let block = BlockPos::from(position);
    let mut advanced = false;
    while let Some(next) = path.next_node() {
        let point = next.point(input.width);
        let point_block = BlockPos::from(point);
        if position.y > f64::from(next.y)
            && !input.on_ground
            && block.x == point_block.x
            && block.z == point_block.z
        {
            path.advance();
            advanced = true;
        } else {
            break;
        }
    }
    if advanced {
        navigation.reset_stuck(position);
    }
}

fn advance_in_flight(navigation: &mut Navigation, input: &FollowInput) {
    let Some(path) = navigation.path.as_mut() else {
        return;
    };
    let position = input.position;
    let block = BlockPos::from(position);
    if let Some(next) = path.next_node()
        && BlockPos::from(next.point(input.width)) == block
    {
        path.advance();
        navigation.reset_stuck(position);
    }
}

fn stuck_check(navigation: &mut Navigation, input: &FollowInput) -> Option<PathEnd> {
    if navigation.ticks.saturating_sub(navigation.stuck.tick) <= STUCK_TICKS {
        return None;
    }
    let speed = input.speed;
    let scale = if speed >= 1.0 { speed } else { speed * speed };
    let reach = f64::from(scale * STUCK_TICKS as f32 * STUCK_FACTOR);
    let moved = input.position.distance_squared(navigation.stuck.position);
    navigation.reset_stuck(input.position);
    (moved < reach * reach).then_some(PathEnd::Stuck)
}

fn timed_out(navigation: &mut Navigation, input: &FollowInput) -> bool {
    let Some(next) = navigation
        .path
        .as_ref()
        .and_then(|path| path.next_node())
        .map(|node| node.pos())
    else {
        return false;
    };
    let timeout = &mut navigation.timeout;
    if timeout.node == Some(next) {
        timeout.timer = timeout
            .timer
            .saturating_add(input.time.saturating_sub(timeout.time));
    } else {
        timeout.node = Some(next);
        timeout.timer = 0;
        let distance = input.position.distance(next.bottom_centre());
        let speed = f64::from(input.speed);
        timeout.limit = if speed > 0.0 {
            distance / speed * 20.0
        } else {
            0.0
        };
    }
    timeout.time = input.time;
    timeout.limit > 0.0 && timeout.timer as f64 > TIMEOUT_FACTOR * timeout.limit
}
