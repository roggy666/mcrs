use glam::DVec3;
use mcrs_minecraft_entity_key::EntityUuid;
use mcrs_minecraft_physics::Body;
use mcrs_minecraft_physics::EntityState;
use mcrs_minecraft_physics::MotionRow;
use mcrs_minecraft_physics::NoBodies;
use mcrs_minecraft_physics::Rest;
use mcrs_minecraft_physics::Side;
use mcrs_minecraft_physics::TestFixture;
use mcrs_minecraft_physics::TickOutcome;
use mcrs_minecraft_physics::View;
use mcrs_minecraft_physics::tick_entity;
use mcrs_voxel::BlockPos;

const SERVER: Side = Side::Server { authority: true };

fn tick(fixture: &mut TestFixture, row: &MotionRow, entity: &mut EntityState) -> TickOutcome {
    fixture.actions.clear();
    let outcome = tick_entity(
        View::new(&fixture.world, &NoBodies, &fixture.tables),
        SERVER,
        row,
        entity,
        false,
        &mut fixture.actions,
        &mut fixture.scratch,
    );
    entity.age = entity.age.wrapping_add(1);
    outcome
}

fn projectile(position: DVec3, velocity: DVec3) -> EntityState {
    let body = Body {
        position,
        velocity,
        ..Body::default()
    };
    EntityState::new(EntityUuid::from_u128(7), body, None)
}

#[test]
fn prj_flt_001_an_arrow_moves_with_the_start_velocity_then_drags_then_falls() {
    let mut fixture = TestFixture::new();
    let mut arrow = projectile(DVec3::new(0.5, 10.0, 0.5), DVec3::new(3.0, 0.0, 0.0));
    let outcome = tick(&mut fixture, &MotionRow::arrow(), &mut arrow);
    assert_eq!(outcome, TickOutcome::default());
    assert_eq!(arrow.body.position, DVec3::new(3.5, 10.0, 0.5));
    assert_eq!(arrow.body.velocity.x, 3.0 * f64::from(0.99_f32));
    assert_eq!(arrow.body.velocity.y, -0.05);
    tick(&mut fixture, &MotionRow::arrow(), &mut arrow);
    assert_eq!(
        arrow.body.position.y,
        10.0 - 0.05,
        "the second move uses the fallen velocity"
    );
}

#[test]
fn prj_flt_002_a_snowball_falls_and_drags_before_its_move() {
    let mut fixture = TestFixture::new();
    let mut snowball = projectile(DVec3::new(0.5, 10.0, 0.5), DVec3::new(1.5, 0.0, 0.0));
    tick(&mut fixture, &MotionRow::snowball(), &mut snowball);
    let drag = f64::from(0.99_f32);
    assert_eq!(
        snowball.body.velocity,
        DVec3::new(1.5 * drag, -0.03 * drag, 0.0)
    );
    assert_eq!(
        snowball.body.position,
        DVec3::new(0.5 + 1.5 * drag, 10.0 - 0.03 * drag, 0.5)
    );
}

#[test]
fn prj_blk_001_an_arrow_stops_before_the_first_collider_and_rests() {
    let mut fixture = TestFixture::new();
    let stone = fixture.blocks.stone;
    fixture
        .world
        .fill(BlockPos::new(4, 8, -1), BlockPos::new(4, 12, 1), stone);
    let mut arrow = projectile(DVec3::new(0.5, 10.0, 0.5), DVec3::new(3.0, 0.0, 0.0));
    let first = tick(&mut fixture, &MotionRow::arrow(), &mut arrow);
    assert!(!first.landed, "the first move ends in the air");
    let second = tick(&mut fixture, &MotionRow::arrow(), &mut arrow);
    assert!(second.landed, "the second move meets the wall");
    assert_eq!(arrow.body.velocity, DVec3::ZERO);
    let back_off = f64::from(0.05_f32);
    assert!(
        (arrow.body.position.x - (4.0 - back_off)).abs() < 1.0e-9,
        "the arrow stops a back-off before the wall face: {}",
        arrow.body.position.x
    );
    let entry = 0.5 / (3.0 * f64::from(0.99_f32));
    assert!(
        (arrow.body.position.y - (10.0 - 0.05 * entry + back_off)).abs() < 1.0e-9,
        "the hit point takes the fallen velocity, then the back-off moves against it: {}",
        arrow.body.position.y
    );
    let third = tick(&mut fixture, &MotionRow::arrow(), &mut arrow);
    assert!(!third.landed);
    assert_eq!(
        arrow.body.velocity,
        DVec3::ZERO,
        "an arrow in the ground does not fall"
    );
    tick(&mut fixture, &MotionRow::arrow(), &mut arrow);
    assert!(
        matches!(arrow.rest, Rest::Resting(_)),
        "a stopped arrow rests until a block near it changes"
    );
}

#[test]
fn prj_blk_001_an_arrow_inside_a_collider_at_the_start_of_a_tick_stops() {
    let mut fixture = TestFixture::new();
    let stone = fixture.blocks.stone;
    fixture.world.set(BlockPos::new(0, 10, 0), stone);
    let mut arrow = projectile(DVec3::new(0.5, 10.5, 0.5), DVec3::new(1.0, 0.0, 0.0));
    let outcome = tick(&mut fixture, &MotionRow::arrow(), &mut arrow);
    assert!(outcome.stuck_in_block);
    assert_eq!(arrow.body.position, DVec3::new(0.5, 10.5, 0.5));
    assert_eq!(arrow.body.velocity, DVec3::ZERO);
}

#[test]
fn prj_flt_001_an_arrow_in_water_uses_the_water_drag_before_its_move() {
    let mut fixture = TestFixture::new();
    let water = fixture.blocks.water;
    fixture
        .world
        .fill(BlockPos::new(-2, 9, -2), BlockPos::new(4, 11, 2), water);
    let mut arrow = projectile(DVec3::new(0.5, 10.0, 0.5), DVec3::new(1.0, 0.0, 0.0));
    tick(&mut fixture, &MotionRow::arrow(), &mut arrow);
    let drag = f64::from(0.6_f32);
    assert_eq!(arrow.body.position.x, 0.5 + drag);
    assert_eq!(arrow.body.velocity, DVec3::new(drag, -0.05, 0.0));
}

#[test]
fn prj_flt_002_a_snowball_in_water_uses_the_water_drag() {
    let mut fixture = TestFixture::new();
    let water = fixture.blocks.water;
    fixture
        .world
        .fill(BlockPos::new(-2, 9, -2), BlockPos::new(4, 11, 2), water);
    let mut snowball = projectile(DVec3::new(0.5, 10.0, 0.5), DVec3::new(1.0, 0.0, 0.0));
    tick(&mut fixture, &MotionRow::snowball(), &mut snowball);
    let drag = f64::from(0.8_f32);
    assert_eq!(snowball.body.velocity, DVec3::new(drag, -0.03 * drag, 0.0));
}
