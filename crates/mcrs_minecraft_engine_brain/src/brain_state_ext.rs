use glam::DVec3;
use mcrs_minecraft_brain::Candidate;
use mcrs_minecraft_brain::DamageCause;
use mcrs_minecraft_brain::DamageRecord;
use mcrs_minecraft_brain::EntityRef;
use mcrs_minecraft_brain::Intents;
use mcrs_minecraft_brain::LookIntent;
use mcrs_minecraft_brain::MeleeIntent;
use mcrs_minecraft_brain::MobFlags;
use mcrs_minecraft_brain::MobKind;
use mcrs_minecraft_brain::MobView;
use mcrs_minecraft_brain::Panic;
use mcrs_minecraft_brain::PanicCauses;
use mcrs_minecraft_brain::PlayerRow;
use mcrs_minecraft_brain::PlayerTable;
use mcrs_minecraft_brain::PoseBits;
use mcrs_minecraft_brain::Progress;
use mcrs_minecraft_brain::RangedIntent;
use mcrs_minecraft_brain::TargetIntent;
use mcrs_minecraft_engine_entity_motion::MobIntent;
use mcrs_minecraft_engine_mob_attributes::MobAttributes;
use mcrs_minecraft_entity::EntityTypeData;
use mcrs_minecraft_entity_attribute::attribute_keys::Attribute;
use mcrs_minecraft_entity_key::EntityId;
use mcrs_minecraft_goal_world::GoalWorld;
use mcrs_minecraft_goals::in_reach;
use mcrs_minecraft_mob_control::BodyData;
use mcrs_minecraft_mob_control::BodyFlags;
use mcrs_minecraft_mob_control::ControlInput;
use mcrs_minecraft_mob_control::LOOK_PITCH_LIMIT;
use mcrs_minecraft_mob_control::LOOK_TURN_SPEED;
use mcrs_minecraft_mob_control::MeleeReflex;
use mcrs_minecraft_mob_control::MeleeStep;
use mcrs_minecraft_mob_control::PathEnd;
use mcrs_minecraft_mob_control::RangedReflex;
use mcrs_minecraft_mob_control::RangedStep;
use mcrs_minecraft_mob_control::Rotations;
use mcrs_minecraft_physics::EntityState;
use mcrs_minecraft_physics::Living;
use mcrs_minecraft_physics::MoveInput;
use mcrs_minecraft_physics::Steering;
use mcrs_minecraft_volume::BlockVolume;
use mcrs_voxel::BlockPos;

use crate::BlockGoalWorld;
use crate::BrainState;
use crate::hit_queue::HitTarget;
use crate::hit_queue::MeleeHit;
use crate::hit_queue::RangedShot;
use crate::sensed::Sensed;
use crate::targets::TargetBody;
use crate::targets::Targets;

const LEAP_TOWARD: f64 = 0.4;
const LEAP_KEEP: f64 = 0.2;
const LOOK_HOLD: u8 = 2;

pub(crate) struct Observed {
    pub(crate) view: MobView,
    pub(crate) struck: bool,
    pub(crate) idle_reset: bool,
    pub(crate) resumed: bool,
}

pub(crate) struct Steered {
    pub(crate) intent: MobIntent,
    pub(crate) ended: Option<PathEnd>,
    pub(crate) hit: Option<MeleeHit>,
    pub(crate) shot: Option<RangedShot>,
}

pub(crate) trait BrainStateExt {
    fn observe(
        &mut self,
        panic_causes: &PanicCauses,
        players: &PlayerTable,
        attributes: Option<&MobAttributes>,
        tick: u64,
        id: EntityId,
        sensed: Sensed<'_>,
    ) -> Observed;

    fn apply(&mut self, intents: &Intents, state: &EntityState);

    fn steer<V: BlockVolume, B>(
        &mut self,
        world: &BlockGoalWorld<'_, V>,
        targets: &Targets<'_, B>,
        tick: u64,
        id: EntityId,
        sensed: Sensed<'_>,
    ) -> Option<Steered>;
}

impl BrainState {
    fn read_attributes(&mut self, attributes: Option<&MobAttributes>) {
        self.follow_range = attribute_of(
            attributes,
            self.kind,
            Attribute::FollowRange,
            Self::DEFAULT_FOLLOW_RANGE,
        );
        self.attack_damage = attribute_of(
            attributes,
            self.kind,
            Attribute::AttackDamage,
            Self::DEFAULT_ATTACK_DAMAGE,
        );
    }

    fn grow(&mut self, alive: bool, stalled: bool) {
        if !alive || stalled {
            return;
        }
        if self.age < 0 {
            self.age = self.age.saturating_add(1);
        } else if self.age > 0 {
            self.age = self.age.saturating_sub(1);
        }
        if self.age == 0 {
            self.love = self.love.saturating_sub(1);
        } else {
            self.love = 0;
        }
    }

    fn attack(
        &mut self,
        body: &TargetBody,
        sees: bool,
        id: EntityId,
        state: &EntityState,
    ) -> (Option<MeleeHit>, Option<RangedShot>) {
        let position = state.body.position;
        let mut hit = None;
        let mut shot = None;
        if self.melee.is_some() || self.ranged.is_some() {
            self.control.look(&LookIntent::At {
                point: body.eye(),
                turn_speed: Some(LOOK_TURN_SPEED),
                pitch_limit: Some(LOOK_PITCH_LIMIT),
                hold: LOOK_HOLD,
            });
        }
        if let Some(reflex) = self.melee.as_mut() {
            let (min, max) = own_box(self.kind.width, self.kind.height, position);
            let reach = in_reach(min, max, &candidate_box(body));
            if reflex.tick(reach, sees) == MeleeStep::Hit {
                hit = self.target.get().map(|target| MeleeHit {
                    attacker: id,
                    attacker_key: state.key,
                    target: if body.player {
                        HitTarget::Player(target, body.key)
                    } else {
                        HitTarget::Mob(target, body.key)
                    },
                });
            }
        }
        if let Some(reflex) = self.ranged.as_mut() {
            let distance = body.position.distance_squared(position);
            if let RangedStep::Attack { power } = reflex.tick(sees, distance) {
                shot = self.target.get().map(|target| RangedShot {
                    attacker: id,
                    attacker_key: state.key,
                    target,
                    power,
                });
            }
        }
        (hit, shot)
    }
}

impl BrainStateExt for BrainState {
    fn observe(
        &mut self,
        panic_causes: &PanicCauses,
        players: &PlayerTable,
        attributes: Option<&MobAttributes>,
        tick: u64,
        id: EntityId,
        sensed: Sensed<'_>,
    ) -> Observed {
        let state = sensed.state;
        let living = state.living.unwrap_or_default();
        let position = state.body.position;
        self.read_attributes(attributes);
        self.turned = self.travelled.take().map(|input| {
            self.control
                .after_travel(&input, position, rotations(&living))
        });
        let hits = sensed.hits.copied().unwrap_or_default();
        let hit = hits.last();
        let strikes = hits.strikes();
        let hurt = hit.is_some() && hit != self.last_hit;
        self.last_hit = hit;
        let struck_now = strikes != self.strikes;
        self.strikes = strikes;
        if hurt {
            self.hit_tick = Some(tick.saturating_sub(1));
            self.love = 0;
        }
        let damage = self.hit_tick.zip(hit).map(|(tick, hit)| DamageRecord {
            tick: tick as u32,
            cause: if panic_causes.contains(u32::from(hit.damage_type)) {
                DamageCause::Panic
            } else {
                DamageCause::Other
            },
        });
        let reach = self.rules.idle_reset_range;
        let near = players.any_within(position, reach, PlayerRow::is_qualified);
        let idle_time = if near || struck_now {
            0
        } else {
            self.idle_time.saturating_add(1)
        };
        let idle_reset = idle_time == 0 && self.idle_time >= self.rules.idle_limit;
        self.idle_time = idle_time;
        let stalled = self.seen_age == Some(state.age);
        let resumed = self.stalled && !stalled;
        self.stalled = stalled;
        self.seen_age = Some(state.age);
        self.grow(sensed.is_alive(), stalled);
        let applied = self.control.applied();
        let marks = [
            (sensed.is_alive(), MobFlags::ALIVE),
            (state.body.flags.on_ground, MobFlags::ON_GROUND),
            (sensed.burning(), MobFlags::ON_FIRE),
            (sensed.exposure().in_sun, MobFlags::IN_SUN),
            (state.contact.in_water(), MobFlags::IN_WATER),
            (self.panic == Panic::Panicking, MobFlags::IN_PANIC),
            (
                applied.progress == Progress::Running,
                MobFlags::PATH_RUNNING,
            ),
            (self.owner.is_some(), MobFlags::TAME),
            (self.pose.contains(PoseBits::SITTING), MobFlags::SITTING),
            (self.pose.contains(PoseBits::SWELLING), MobFlags::SWELLING),
            (self.pose.contains(PoseBits::EATING), MobFlags::EATING),
            (self.is_baby(), MobFlags::BABY),
        ];
        Observed {
            view: MobView {
                mob: id,
                key: state.key,
                position,
                velocity: state.body.velocity.as_vec3(),
                kind: MobKind(self.kind.entity_type.id().number()),
                flags: MobFlags::of(marks),
                health: sensed.vitals.map_or(0.0, |vitals| vitals.health),
                follow_range: self.follow_range,
                age: self.age,
                love: self.love,
                idle_time: u16::try_from(self.idle_time).unwrap_or(u16::MAX),
                damage,
                hurt_by: self.hurt_by,
                target: self.target,
                owner: self.owner,
                path_ticket: applied.ticket,
            },
            struck: struck_now,
            idle_reset,
            resumed,
        }
    }

    fn apply(&mut self, intents: &Intents, state: &EntityState) {
        self.control.apply(intents, state.body.position);
        if let Some(panic) = intents.panic {
            self.panic = panic;
        }
        if let Some(target) = intents.target {
            self.target = match target {
                TargetIntent::Set(target) => EntityRef::of(target),
                TargetIntent::Clear => EntityRef::NONE,
            };
        }
        match intents.melee {
            Some(MeleeIntent::Start {
                speed_modifier,
                interval,
                pursuit,
            }) => self.melee = Some(MeleeReflex::new(speed_modifier, interval, pursuit)),
            Some(MeleeIntent::Stop) => self.melee = None,
            None => {}
        }
        match intents.ranged {
            Some(RangedIntent::Start {
                speed_modifier,
                min_interval,
                max_interval,
                range,
            }) => {
                self.ranged = Some(RangedReflex::new(
                    speed_modifier,
                    min_interval,
                    max_interval,
                    range,
                ));
            }
            Some(RangedIntent::Stop) => self.ranged = None,
            None => {}
        }
        if let Some(leap) = intents.leap {
            self.leap = Some(leap);
        }
        if let Some(speed) = intents.speed {
            self.control.set_speed_modifier(speed);
        }
        if let Some(pose) = intents.pose {
            self.pose =
                PoseBits::of_bits((self.pose.bits() | pose.set.bits()) & !pose.clear.bits());
        }
    }

    fn steer<V: BlockVolume, B>(
        &mut self,
        world: &BlockGoalWorld<'_, V>,
        targets: &Targets<'_, B>,
        tick: u64,
        id: EntityId,
        sensed: Sensed<'_>,
    ) -> Option<Steered> {
        let state = sensed.state;
        let living = state.living.filter(|_| sensed.is_alive())?;
        let head_yaw_limit = self.kind.mob()?.head_yaw_limit;
        let position = state.body.position;
        let turned = self.turned.take();
        let on_ground = state.body.flags.on_ground;
        let target = self.target.get().and_then(|target| targets.body(target));
        let eye = DVec3::new(
            position.x,
            position.y + f64::from(self.kind.eye_height),
            position.z,
        );
        let sees = target
            .as_ref()
            .is_some_and(|body| world.line_of_sight(eye, body.eye()));
        let (hit, shot) = match target.as_ref() {
            Some(body) => self.attack(body, sees, id, state),
            None => (None, None),
        };
        let velocity = self.leap.take().map(|leap| {
            let current = state.body.velocity;
            DVec3::new(
                f64::from(leap.toward_x) * LEAP_TOWARD + current.x * LEAP_KEEP,
                f64::from(leap.vertical),
                f64::from(leap.toward_z) * LEAP_TOWARD + current.z * LEAP_KEEP,
            )
        });
        let input = ControlInput {
            tick,
            position,
            rotations: turned.unwrap_or_else(|| rotations(&living)),
            body: BodyData {
                width: self.kind.width,
                eye_height: self.kind.eye_height,
                step_height: living.attributes.step_height as f32,
                head_yaw_limit,
            },
            flags: if on_ground {
                BodyFlags::ON_GROUND
            } else {
                BodyFlags::NONE
            },
            sideways: living.input.sideways,
            speed: living.speed,
            movement_speed: living.attributes.movement_speed,
            feet_block_top: world.feet_top(BlockPos::from(position)),
        };
        let out = self.control.before_travel(&input);
        self.travelled = Some(input);
        let intent = MobIntent {
            steering: out.steering.map(|steering| Steering {
                input: MoveInput {
                    sideways: steering.sideways,
                    vertical: steering.vertical,
                    forward: steering.forward,
                },
                jump: steering.jump,
                speed: steering.speed,
            }),
            yaw: out.yaw,
            pitch: out.pitch,
            head_yaw: out.head_yaw.or(turned.map(|turned| turned.head_yaw)),
            body_yaw: turned.map(|turned| turned.body_yaw),
            velocity,
        };
        Some(Steered {
            intent,
            ended: out.path_ended,
            hit,
            shot,
        })
    }
}

fn attribute_of(
    attributes: Option<&MobAttributes>,
    kind: &'static EntityTypeData,
    attribute: Attribute,
    default: f32,
) -> f32 {
    let value = match attributes {
        Some(attributes) => attributes.value(kind.entity_type, attribute),
        None => kind
            .attribute_bases()
            .and_then(|bases| bases.base(attribute)),
    };
    value.map_or(default, |value| value as f32)
}

fn rotations(living: &Living) -> Rotations {
    Rotations {
        yaw: living.orientation.yaw,
        pitch: living.orientation.pitch,
        head_yaw: living.orientation.head_yaw,
        body_yaw: living.orientation.body_yaw,
    }
}

fn own_box(width: f32, height: f32, position: DVec3) -> (DVec3, DVec3) {
    let half = f64::from(width) * 0.5;
    (
        DVec3::new(position.x - half, position.y, position.z - half),
        DVec3::new(
            position.x + half,
            position.y + f64::from(height),
            position.z + half,
        ),
    )
}

fn candidate_box(body: &TargetBody) -> Candidate {
    let row = PlayerRow {
        width: body.width,
        height: body.height,
        ..PlayerRow::standing(EntityId(0), body.key, body.position, body.eye_height)
    };
    Candidate::player(&row)
}
