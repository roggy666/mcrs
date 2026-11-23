use glam::DVec3;
use mcrs_minecraft_brain::Applied;
use mcrs_minecraft_brain::PathIntent;
use mcrs_minecraft_brain::Progress;
use mcrs_minecraft_pathfind::Acceptance;
use mcrs_minecraft_pathfind::Flight;
use mcrs_minecraft_pathfind::FollowInput;
use mcrs_minecraft_pathfind::FoundPath;
use mcrs_minecraft_pathfind::Navigation;
use mcrs_minecraft_pathfind::PathWorld;
use mcrs_minecraft_pathfind::RequestKind;
use mcrs_minecraft_pathfind::follow;
use mcrs_voxel::BlockPos;

use crate::BodyFlags;
use crate::ControlInput;
use crate::PathEnd;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Follower {
    applied: Applied,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FollowerStep {
    pub wanted: Option<Wanted>,
    pub ended: Option<PathEnd>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wanted {
    pub position: DVec3,
    pub speed_modifier: f32,
}

impl Follower {
    #[must_use]
    pub const fn applied(&self) -> Applied {
        self.applied
    }

    pub fn apply(
        &mut self,
        intent: &PathIntent,
        navigation: &mut Navigation,
        found: Option<&FoundPath>,
        position: DVec3,
        is_cauldron: impl Fn(BlockPos) -> bool,
    ) {
        let ticket = intent.ticket();
        match *intent {
            PathIntent::Walk { speed_modifier, .. } => {
                let acceptance = navigation.accept(
                    RequestKind::Point,
                    found.cloned(),
                    speed_modifier,
                    position,
                    is_cauldron,
                );
                if acceptance == Acceptance::Dropped {
                    navigation.drop_path();
                }
            }
            PathIntent::Clear { .. } => navigation.drop_path(),
        }
        self.applied = Applied {
            ticket,
            progress: if navigation.is_running() {
                Progress::Running
            } else {
                Progress::Ended
            },
        };
    }

    pub fn set_speed_modifier(&mut self, navigation: &mut Navigation, speed_modifier: f32) {
        navigation.keep_current(speed_modifier);
    }

    pub fn step<W: PathWorld + ?Sized>(
        &mut self,
        input: &ControlInput,
        navigation: &mut Navigation,
        world: &W,
        flight: Option<Flight>,
    ) -> FollowerStep {
        let flags = input.flags;
        let step = follow(
            navigation,
            &FollowInput {
                position: input.position,
                on_ground: flags.contains(BodyFlags::ON_GROUND),
                in_liquid: flags.contains(BodyFlags::IN_LIQUID),
                passenger: flags.contains(BodyFlags::PASSENGER),
                width: input.body.width,
                height: input.body.height,
                speed: input.speed,
                time: input.tick,
                flight,
            },
            world,
        );
        if step.ended.is_some() {
            self.applied.progress = Progress::Ended;
        }
        FollowerStep {
            wanted: step.wanted.map(|position| Wanted {
                position,
                speed_modifier: step.speed_modifier,
            }),
            ended: step.ended.map(|end| match end {
                mcrs_minecraft_pathfind::PathEnd::Done => PathEnd::Done,
                mcrs_minecraft_pathfind::PathEnd::Stuck => PathEnd::Stuck,
                mcrs_minecraft_pathfind::PathEnd::TimedOut => PathEnd::TimedOut,
            }),
        }
    }
}
