use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use crate::{
    shortest_angle_delta, AutoRecenterSettings, CollisionStrategy, FollowAlignment, LockOnSettings,
    ScreenSpaceFramingSettings, ShoulderSide, SmoothingSettings, ThirdPersonCamera,
    ThirdPersonCameraIgnore, ThirdPersonCameraInput, ThirdPersonCameraInputTarget,
    ThirdPersonCameraLockOn, ThirdPersonCameraLockOnTarget, ThirdPersonCameraObstacle,
    ThirdPersonCameraPlugin, ThirdPersonCameraRuntime, ThirdPersonCameraSettings,
    ThirdPersonCameraSystems, ThirdPersonCameraTarget,
};

#[derive(States, Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
enum DemoState {
    #[default]
    Active,
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin, AssetPlugin::default()));
    app.init_state::<DemoState>();
    app.add_plugins(ThirdPersonCameraPlugin::new(
        OnEnter(DemoState::Active),
        OnExit(DemoState::Active),
        Update,
    ));
    app
}

fn spawn_target(app: &mut App, name: &str, transform: Transform) -> Entity {
    app.world_mut()
        .spawn((
            Name::new(name.to_owned()),
            transform,
            GlobalTransform::from(transform),
        ))
        .id()
}

fn set_target_transform(app: &mut App, entity: Entity, transform: Transform) {
    *app.world_mut().get_mut::<Transform>(entity).unwrap() = transform;
    *app.world_mut().get_mut::<GlobalTransform>(entity).unwrap() = GlobalTransform::from(transform);
}

fn assert_angle_close(actual: f32, expected: f32) {
    assert!(
        shortest_angle_delta(actual, expected).abs() < 0.12,
        "expected angle {expected}, got {actual}",
    );
}

#[test]
fn retarget_changes_runtime_pivot() {
    let mut app = test_app();
    let target_a = spawn_target(&mut app, "Target A", Transform::from_xyz(0.0, 0.0, 0.0));
    let target_b = spawn_target(&mut app, "Target B", Transform::from_xyz(6.0, 0.0, 0.0));
    let camera = app
        .world_mut()
        .spawn((
            ThirdPersonCamera::default(),
            ThirdPersonCameraTarget::new(target_a),
            ThirdPersonCameraInput::default(),
        ))
        .id();

    app.update();
    let first_pivot = app
        .world()
        .get::<ThirdPersonCameraRuntime>(camera)
        .expect("runtime exists")
        .pivot;
    assert!(first_pivot.distance(Vec3::new(0.0, 1.9, 0.0)) < 0.5);

    app.world_mut()
        .get_mut::<ThirdPersonCameraTarget>(camera)
        .expect("target exists")
        .target = target_b;
    app.update();
    let second_pivot = app
        .world()
        .get::<ThirdPersonCameraRuntime>(camera)
        .expect("runtime exists")
        .pivot;
    assert!(second_pivot.x > first_pivot.x + 0.5);
}

#[test]
fn shoulder_toggle_swaps_target_side() {
    let mut app = test_app();
    let target = spawn_target(&mut app, "Target", Transform::default());
    let camera = app
        .world_mut()
        .spawn((
            ThirdPersonCamera::default(),
            ThirdPersonCameraTarget::new(target),
            ThirdPersonCameraInputTarget,
            ThirdPersonCameraInput {
                shoulder_toggle: true,
                ..default()
            },
        ))
        .id();

    app.update();
    let camera_state = app.world().get::<ThirdPersonCamera>(camera).unwrap();
    assert_eq!(camera_state.target_shoulder_side, ShoulderSide::Left);
}

#[test]
fn obstruction_shortens_distance() {
    let mut app = test_app();
    let target = spawn_target(&mut app, "Target", Transform::default());
    let camera = app
        .world_mut()
        .spawn((
            ThirdPersonCamera::default(),
            ThirdPersonCameraTarget::new(target),
            ThirdPersonCameraSettings {
                collision: crate::CollisionSettings {
                    strategy: CollisionStrategy::SingleRay,
                    ..default()
                },
                ..default()
            },
        ))
        .id();
    app.world_mut().spawn((
        ThirdPersonCameraObstacle::default(),
        bevy::camera::primitives::Aabb::from_min_max(
            Vec3::new(-0.8, -0.8, -0.8),
            Vec3::new(0.8, 0.8, 0.8),
        ),
        Transform::from_xyz(0.0, 2.8, 2.2),
        GlobalTransform::from(Transform::from_xyz(0.0, 2.8, 2.2)),
    ));

    app.update();

    let runtime = app.world().get::<ThirdPersonCameraRuntime>(camera).unwrap();
    assert!(runtime.obstruction_active);
    assert!(runtime.obstruction_distance < runtime.desired_distance);
}

#[test]
fn plugin_exposes_public_system_sets() {
    let mut app = test_app();
    app.configure_sets(
        PostUpdate,
        ThirdPersonCameraSystems::ResolveObstruction.after(ThirdPersonCameraSystems::UpdateIntent),
    );
}

#[test]
fn aim_hold_uses_temporary_mode_override() {
    let mut app = test_app();
    let target = spawn_target(&mut app, "Target", Transform::default());
    let camera = app
        .world_mut()
        .spawn((
            ThirdPersonCamera::default(),
            ThirdPersonCameraTarget::new(target),
            ThirdPersonCameraInputTarget,
            ThirdPersonCameraInput {
                aim: true,
                ..default()
            },
        ))
        .id();

    app.update();

    let camera_state = app.world().get::<ThirdPersonCamera>(camera).unwrap();
    let runtime = app.world().get::<ThirdPersonCameraRuntime>(camera).unwrap();
    assert_eq!(
        camera_state.target_mode,
        crate::ThirdPersonCameraMode::Center
    );
    assert_eq!(runtime.target_aim_blend, 1.0);

    app.update();

    let camera_state = app.world().get::<ThirdPersonCamera>(camera).unwrap();
    let runtime = app.world().get::<ThirdPersonCameraRuntime>(camera).unwrap();
    assert_eq!(
        camera_state.target_mode,
        crate::ThirdPersonCameraMode::Center
    );
    assert_eq!(runtime.target_aim_blend, 0.0);
}

#[test]
fn shoulder_hold_is_temporary() {
    let mut app = test_app();
    let target = spawn_target(&mut app, "Target", Transform::default());
    let camera = app
        .world_mut()
        .spawn((
            ThirdPersonCamera::default(),
            ThirdPersonCameraTarget::new(target),
            ThirdPersonCameraInputTarget,
            ThirdPersonCameraInput {
                shoulder_hold: true,
                ..default()
            },
        ))
        .id();

    app.update();

    let camera_state = app.world().get::<ThirdPersonCamera>(camera).unwrap();
    let runtime = app.world().get::<ThirdPersonCameraRuntime>(camera).unwrap();
    assert_eq!(
        camera_state.target_mode,
        crate::ThirdPersonCameraMode::Center
    );
    assert_eq!(runtime.target_shoulder_blend, 1.0);

    app.update();

    let camera_state = app.world().get::<ThirdPersonCamera>(camera).unwrap();
    let runtime = app.world().get::<ThirdPersonCameraRuntime>(camera).unwrap();
    assert_eq!(
        camera_state.target_mode,
        crate::ThirdPersonCameraMode::Center
    );
    assert_eq!(runtime.target_shoulder_blend, 0.0);
}

#[test]
fn auto_recenter_target_forward_uses_alignment_setting() {
    let mut app = test_app();
    let target = spawn_target(
        &mut app,
        "Facing Target",
        Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
    );
    let camera = app
        .world_mut()
        .spawn((
            ThirdPersonCamera::default(),
            ThirdPersonCameraTarget::new(target),
            ThirdPersonCameraSettings {
                auto_recenter: AutoRecenterSettings {
                    enabled: true,
                    inactivity_seconds: 0.0,
                    follow_alignment: FollowAlignment::TargetForward,
                },
                ..default()
            },
        ))
        .id();

    app.update();

    let camera_state = app.world().get::<ThirdPersonCamera>(camera).unwrap();
    assert_angle_close(camera_state.target_yaw, -std::f32::consts::FRAC_PI_2);
}

#[test]
fn auto_recenter_movement_alignment_uses_motion_heading() {
    let mut app = test_app();
    let target = spawn_target(&mut app, "Moving Target", Transform::default());
    let camera = app
        .world_mut()
        .spawn((
            ThirdPersonCamera::default(),
            ThirdPersonCameraTarget::new(target),
            ThirdPersonCameraSettings {
                auto_recenter: AutoRecenterSettings {
                    enabled: true,
                    inactivity_seconds: 0.0,
                    follow_alignment: FollowAlignment::MovementDirection,
                },
                ..default()
            },
        ))
        .id();

    app.update();
    set_target_transform(&mut app, target, Transform::from_xyz(4.0, 0.0, 0.0));
    app.update();

    let camera_state = app.world().get::<ThirdPersonCamera>(camera).unwrap();
    assert_angle_close(camera_state.target_yaw, std::f32::consts::FRAC_PI_2);
}

#[test]
fn ignored_obstacles_do_not_shorten_distance() {
    let mut app = test_app();
    let target = spawn_target(&mut app, "Target", Transform::default());
    let camera = app
        .world_mut()
        .spawn((
            ThirdPersonCamera::default(),
            ThirdPersonCameraTarget::new(target),
            ThirdPersonCameraSettings {
                collision: crate::CollisionSettings {
                    strategy: CollisionStrategy::SingleRay,
                    ..default()
                },
                ..default()
            },
        ))
        .id();
    app.world_mut().spawn((
        ThirdPersonCameraObstacle::default(),
        ThirdPersonCameraIgnore,
        bevy::camera::primitives::Aabb::from_min_max(
            Vec3::new(-0.8, -0.8, -0.8),
            Vec3::new(0.8, 0.8, 0.8),
        ),
        Transform::from_xyz(0.0, 2.8, 2.2),
        GlobalTransform::from(Transform::from_xyz(0.0, 2.8, 2.2)),
    ));

    app.update();

    let runtime = app.world().get::<ThirdPersonCameraRuntime>(camera).unwrap();
    assert!(!runtime.obstruction_active);
    assert_eq!(runtime.corrected_distance, runtime.desired_distance);
}

#[test]
fn large_target_radius_raises_minimum_distance() {
    let mut app = test_app();
    let target = spawn_target(&mut app, "Large Target", Transform::default());
    let camera = app
        .world_mut()
        .spawn((
            ThirdPersonCamera::new(0.5, 0.0, -0.42).with_large_target_radius(1.0),
            ThirdPersonCameraTarget::new(target),
        ))
        .id();

    app.update();

    let runtime = app.world().get::<ThirdPersonCameraRuntime>(camera).unwrap();
    assert!(runtime.desired_distance >= 1.79);
}

#[test]
fn lock_on_toggle_selects_forward_candidate() {
    let mut app = test_app();
    let target = spawn_target(&mut app, "Player", Transform::default());
    let front_target = app
        .world_mut()
        .spawn((
            Name::new("Front Target"),
            ThirdPersonCameraLockOnTarget::default(),
            Transform::from_xyz(0.0, 0.0, 8.0),
            GlobalTransform::from(Transform::from_xyz(0.0, 0.0, 8.0)),
        ))
        .id();
    app.world_mut().spawn((
        Name::new("Rear Target"),
        ThirdPersonCameraLockOnTarget::default(),
        Transform::from_xyz(0.0, 0.0, -8.0),
        GlobalTransform::from(Transform::from_xyz(0.0, 0.0, -8.0)),
    ));
    let camera = app
        .world_mut()
        .spawn((
            ThirdPersonCamera::default(),
            ThirdPersonCameraTarget::new(target),
            ThirdPersonCameraInput {
                lock_on_toggle: true,
                ..default()
            },
            ThirdPersonCameraSettings {
                lock_on: LockOnSettings {
                    enabled: true,
                    max_distance: 20.0,
                    ..default()
                },
                smoothing: SmoothingSettings {
                    orientation_smoothing: 0.0,
                    target_follow_smoothing: 0.0,
                    aim_blend: 0.0,
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    app.update();

    let lock_on = app.world().get::<ThirdPersonCameraLockOn>(camera).unwrap();
    let runtime = app.world().get::<ThirdPersonCameraRuntime>(camera).unwrap();
    assert_eq!(lock_on.active_target, Some(front_target));
    assert_eq!(runtime.active_lock_on_target, Some(front_target));
    assert!(runtime.lock_on_blend > 0.9);
}

#[test]
fn lock_on_cycle_moves_to_the_next_target() {
    let mut app = test_app();
    let target = spawn_target(&mut app, "Player", Transform::default());
    let front_target = app
        .world_mut()
        .spawn((
            Name::new("Front Target"),
            ThirdPersonCameraLockOnTarget::default(),
            Transform::from_xyz(0.0, 0.0, 8.0),
            GlobalTransform::from(Transform::from_xyz(0.0, 0.0, 8.0)),
        ))
        .id();
    let right_target = app
        .world_mut()
        .spawn((
            Name::new("Right Target"),
            ThirdPersonCameraLockOnTarget::default(),
            Transform::from_xyz(6.0, 0.0, 6.0),
            GlobalTransform::from(Transform::from_xyz(6.0, 0.0, 6.0)),
        ))
        .id();
    let camera = app
        .world_mut()
        .spawn((
            ThirdPersonCamera::default(),
            ThirdPersonCameraTarget::new(target),
            ThirdPersonCameraSettings {
                lock_on: LockOnSettings {
                    enabled: true,
                    max_distance: 20.0,
                    ..default()
                },
                smoothing: SmoothingSettings {
                    orientation_smoothing: 0.0,
                    target_follow_smoothing: 0.0,
                    aim_blend: 0.0,
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    app.world_mut()
        .get_mut::<ThirdPersonCameraInput>(camera)
        .unwrap()
        .lock_on_toggle = true;
    app.update();
    assert_eq!(
        app.world()
            .get::<ThirdPersonCameraLockOn>(camera)
            .unwrap()
            .active_target,
        Some(front_target)
    );

    app.world_mut()
        .get_mut::<ThirdPersonCameraInput>(camera)
        .unwrap()
        .lock_on_next = true;
    app.update();

    assert_eq!(
        app.world()
            .get::<ThirdPersonCameraLockOn>(camera)
            .unwrap()
            .active_target,
        Some(right_target)
    );
}

#[test]
fn screen_framing_dead_zone_absorbs_small_motion() {
    let mut app = test_app();
    let target = spawn_target(&mut app, "Player", Transform::default());
    let camera = app
        .world_mut()
        .spawn((
            ThirdPersonCamera::default(),
            ThirdPersonCameraTarget::new(target),
            ThirdPersonCameraSettings {
                smoothing: SmoothingSettings {
                    target_follow_smoothing: 0.0,
                    ..default()
                },
                screen_framing: ScreenSpaceFramingSettings {
                    enabled: true,
                    dead_zone: Vec2::new(0.35, 0.25),
                    soft_zone: Vec2::new(0.55, 0.4),
                    screen_offset: Vec2::ZERO,
                },
                ..default()
            },
        ))
        .id();

    app.update();
    let initial_pivot = app
        .world()
        .get::<ThirdPersonCameraRuntime>(camera)
        .unwrap()
        .pivot;

    set_target_transform(&mut app, target, Transform::from_xyz(0.2, 0.0, 0.0));
    app.update();
    let small_motion_pivot = app
        .world()
        .get::<ThirdPersonCameraRuntime>(camera)
        .unwrap()
        .pivot;
    assert!((small_motion_pivot.x - initial_pivot.x).abs() < 0.05);

    set_target_transform(&mut app, target, Transform::from_xyz(2.6, 0.0, 0.0));
    app.update();
    let large_motion_pivot = app
        .world()
        .get::<ThirdPersonCameraRuntime>(camera)
        .unwrap()
        .pivot;
    assert!(large_motion_pivot.x > small_motion_pivot.x + 0.3);
}
