use bevy_ecs::component::Component;
use mcrs_minecraft_brain::BrainRules;
use mcrs_minecraft_brain::EntityRef;
use mcrs_minecraft_brain::HurtBy;
use mcrs_minecraft_brain::Leap;
use mcrs_minecraft_brain::Panic;
use mcrs_minecraft_brain::PoseBits;
use mcrs_minecraft_damage::LastHit;
use mcrs_minecraft_entity::EntityTypeData;
use mcrs_minecraft_mob_control::ControlInput;
use mcrs_minecraft_mob_control::FlightControl;
use mcrs_minecraft_mob_control::LookDefaults;
use mcrs_minecraft_mob_control::MeleeReflex;
use mcrs_minecraft_mob_control::MobControl;
use mcrs_minecraft_mob_control::RangedReflex;
use mcrs_minecraft_mob_control::Rotations;

pub const BABY_AGE: i32 = -24000;
const BRED_AGE: i32 = 6000;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct BrainState {
    pub kind: &'static EntityTypeData,
    pub rules: &'static BrainRules,
    pub control: MobControl,
    pub travelled: Option<ControlInput>,
    pub turned: Option<Rotations>,
    pub idle_time: u32,
    pub last_hit: Option<LastHit>,
    pub strikes: u16,
    pub hit_tick: Option<u64>,
    pub panic: Panic,
    pub seen_age: Option<u32>,
    pub stalled: bool,
    pub target: EntityRef,
    pub melee: Option<MeleeReflex>,
    pub ranged: Option<RangedReflex>,
    pub leap: Option<Leap>,
    pub pose: PoseBits,
    pub hurt_by: HurtBy,
    pub age: i32,
    pub love: u16,
    pub owner: EntityRef,
    pub follow_range: f32,
    pub attack_damage: f32,
    pub flying_speed: f32,
}

impl BrainState {
    pub const DEFAULT_FOLLOW_RANGE: f32 = 16.0;
    pub const DEFAULT_ATTACK_DAMAGE: f32 = 2.0;
    pub const DEFAULT_FLYING_SPEED: f32 = 0.02;

    #[must_use]
    pub fn new(
        kind: &'static EntityTypeData,
        rules: &'static BrainRules,
        baby: bool,
    ) -> Option<Self> {
        let mob = kind.mob()?;
        Some(Self {
            kind,
            rules,
            control: match mob.flight {
                Some(flight) => MobControl::flying(
                    LookDefaults {
                        turn_speed: mob.head_turn_speed,
                        pitch_limit: mob.pitch_limit,
                    },
                    FlightControl {
                        hovers: flight.hovers,
                        can_float: flight.can_float,
                    },
                ),
                None => MobControl::new(LookDefaults {
                    turn_speed: mob.head_turn_speed,
                    pitch_limit: mob.pitch_limit,
                }),
            },
            travelled: None,
            turned: None,
            idle_time: 0,
            last_hit: None,
            strikes: 0,
            hit_tick: None,
            panic: Panic::Calm,
            seen_age: None,
            stalled: false,
            target: EntityRef::NONE,
            melee: None,
            ranged: None,
            leap: None,
            pose: PoseBits::NONE,
            hurt_by: HurtBy::NONE,
            age: if baby { BABY_AGE } else { 0 },
            love: 0,
            owner: EntityRef::NONE,
            follow_range: Self::DEFAULT_FOLLOW_RANGE,
            attack_damage: Self::DEFAULT_ATTACK_DAMAGE,
            flying_speed: Self::DEFAULT_FLYING_SPEED,
        })
    }

    pub fn bred(&mut self) {
        self.age = BRED_AGE;
        self.love = 0;
    }

    #[must_use]
    pub fn is_baby(&self) -> bool {
        self.age < 0
    }
}
