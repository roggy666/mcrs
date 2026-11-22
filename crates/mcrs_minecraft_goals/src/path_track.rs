use glam::DVec3;
use mcrs_minecraft_brain::Intents;
use mcrs_minecraft_brain::MobView;
use mcrs_minecraft_brain::PathIntent;
use mcrs_minecraft_brain::Progress;
use mcrs_minecraft_brain::Ticket;
use mcrs_minecraft_brain::Turn;
use mcrs_minecraft_goal_world::GoalWorld;
use mcrs_voxel::BlockPos;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PathTrack {
    issued: Ticket,
    route: Route,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
enum Route {
    #[default]
    None,
    Walking,
}

impl PathTrack {
    #[must_use]
    pub fn is_running(&self, view: &MobView) -> bool {
        match self.route {
            Route::None => false,
            Route::Walking => {
                let applied = view.path();
                applied.ticket != self.issued || applied.progress == Progress::Running
            }
        }
    }

    pub fn request<W: GoalWorld + ?Sized>(
        &mut self,
        turn: &Turn<'_, W>,
        target: DVec3,
        speed_modifier: f32,
        intents: &mut Intents,
    ) {
        match turn.world.route(turn.view.position, BlockPos::from(target)) {
            Some(end) => self.walk(end, speed_modifier, intents),
            None => self.clear(intents),
        }
    }

    pub fn walk(&mut self, end: DVec3, speed_modifier: f32, intents: &mut Intents) {
        self.issued = self.issued.next();
        self.route = Route::Walking;
        intents.path = Some(PathIntent::Walk {
            ticket: self.issued,
            target: end,
            speed_modifier,
        });
    }

    pub fn clear(&mut self, intents: &mut Intents) {
        self.issued = self.issued.next();
        self.route = Route::None;
        intents.path = Some(PathIntent::Clear {
            ticket: self.issued,
        });
    }
}
