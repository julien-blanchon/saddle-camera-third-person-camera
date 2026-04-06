use avian3d::prelude::*;
use bevy::{prelude::*, transform::TransformSystems, window::WindowPlugin};
use bevy_enhanced_input::prelude::{
    Action, Axial, Bindings, Cardinal, DeadZone, actions, bindings,
};
use saddle_camera_third_person_camera::{
    ShoulderSide, ThirdPersonCamera, ThirdPersonCameraEnhancedInputPlugin,
    ThirdPersonCameraIgnoreTarget, ThirdPersonCameraLockOnSettings, ThirdPersonCameraMode,
    ThirdPersonCameraPlugin, ThirdPersonCameraRuntime, ThirdPersonCameraSettings,
    ThirdPersonCameraShoulderRig, ThirdPersonCameraShoulderSettings,
    ThirdPersonCameraSystems, ThirdPersonCameraTarget, default_input_bindings,
};
use saddle_camera_third_person_camera_example_common as common;
use saddle_character_controller::{
    CharacterController, CharacterControllerPlugin, CharacterControllerState, CharacterPush,
    CrouchAction, JumpAction, MoveAction, SprintAction,
};
use saddle_pane::prelude::*;

#[derive(Component)]
struct DemoPlayer;

#[derive(Component)]
struct MovingPlatform {
    origin: Vec3,
    axis: Vec3,
    amplitude: f32,
    speed: f32,
}

#[derive(Resource, Clone, Copy)]
struct DemoEntities {
    player: Entity,
    camera: Entity,
}

#[derive(Resource, Pane)]
#[pane(title = "Character Controller", position = "top-left")]
struct ControllerPane {
    #[pane(tab = "Movement", slider, min = 4.0, max = 18.0, step = 0.25)]
    speed: f32,
    #[pane(tab = "Movement", slider, min = 1.0, max = 2.0, step = 0.05)]
    sprint_speed_scale: f32,
    #[pane(tab = "Movement", slider, min = 0.2, max = 1.2, step = 0.05)]
    step_size: f32,
    #[pane(tab = "Jump", slider, min = 1.0, max = 3.2, step = 0.1)]
    jump_height: f32,
    #[pane(tab = "Runtime", monitor)]
    grounded: bool,
    #[pane(tab = "Runtime", monitor)]
    movement_mode: String,
}

impl Default for ControllerPane {
    fn default() -> Self {
        let controller = CharacterController::default();
        Self {
            speed: controller.speed,
            sprint_speed_scale: controller.sprint_speed_scale,
            step_size: controller.step_size,
            jump_height: controller.jump_height,
            grounded: false,
            movement_mode: "Airborne".into(),
        }
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "saddle-camera-third-person-camera character_controller".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        PhysicsPlugins::default(),
        ThirdPersonCameraPlugin::default(),
        ThirdPersonCameraEnhancedInputPlugin::default(),
        CharacterControllerPlugin::always_on(FixedUpdate),
    ));
    common::add_debug_pane(&mut app);
    app.init_resource::<ControllerPane>()
        .register_pane::<ControllerPane>()
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                animate_platforms
                    .before(saddle_character_controller::CharacterControllerSystems::Grounding),
                align_character_to_camera
                    .before(saddle_character_controller::CharacterControllerSystems::Movement),
            ),
        )
        .add_systems(Update, (sync_controller_pane, update_overlay))
        .add_systems(
            PostUpdate,
            (
                common::animate_targets.before(ThirdPersonCameraSystems::UpdateIntent),
                sync_hero_body_height
                    .after(ThirdPersonCameraSystems::ApplyTransform)
                    .before(TransformSystems::Propagate),
            ),
        );
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    common::spawn_reference_world(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Third Person + Character Controller",
        "WASD / left stick moves the controller. RMB aims, F locks on, E/Z cycle targets, C swaps shoulders, Ctrl crouches.\nRun the training lane, jump onto the bridge, and use the pane to retune controller + camera feel live.",
        Color::srgb(0.86, 0.44, 0.18),
    );

    spawn_collision_world(&mut commands);
    spawn_dressing(&mut commands, &mut meshes, &mut materials);

    let player = spawn_player(&mut commands, &mut meshes, &mut materials);
    let camera = spawn_camera(&mut commands, player);

    common::spawn_target(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lock-On Drone Alpha",
        Color::srgb(0.92, 0.38, 0.26),
        Vec3::new(-5.0, 1.2, -18.0),
        common::DemoMotionPath::Circle {
            center: Vec3::new(-5.0, 1.2, -18.0),
            radius: 2.4,
            speed: 0.42,
            phase: 0.0,
        },
    );
    common::spawn_target(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lock-On Drone Beta",
        Color::srgb(0.22, 0.58, 0.88),
        Vec3::new(5.5, 1.4, -26.0),
        common::DemoMotionPath::Hover {
            center: Vec3::new(5.5, 1.4, -26.0),
            amplitude: 0.45,
            speed: 0.8,
        },
    );
    common::spawn_target(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lock-On Drone Gamma",
        Color::srgb(0.90, 0.74, 0.24),
        Vec3::new(0.0, 1.1, -34.0),
        common::DemoMotionPath::Corridor {
            center: Vec3::new(0.0, 1.1, -34.0),
            half_length: 4.8,
            speed: 0.52,
        },
    );

    commands.insert_resource(DemoEntities { player, camera });
}

fn spawn_player(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Entity {
    let body_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.26, 0.34),
        metallic: 0.08,
        perceptual_roughness: 0.34,
        ..default()
    });
    let accent_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.90, 0.44, 0.20),
        emissive: LinearRgba::rgb(0.08, 0.03, 0.01),
        perceptual_roughness: 0.28,
        ..default()
    });

    commands
        .spawn((
            Name::new("Controller Hero"),
            DemoPlayer,
            ThirdPersonCameraIgnoreTarget,
            CharacterController {
                speed: 9.0,
                sprint_speed_scale: 1.35,
                jump_height: 1.85,
                step_size: 0.6,
                standing_view_height: 1.65,
                crouch_view_height: 1.15,
                ..default()
            },
            CharacterPush::default(),
            Transform::from_xyz(0.0, 2.0, 8.0),
            actions!(CharacterController[
                (
                    Action::<MoveAction>::new(),
                    DeadZone::default(),
                    Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
                ),
                (Action::<JumpAction>::new(), bindings![KeyCode::Space, GamepadButton::South]),
                (
                    Action::<SprintAction>::new(),
                    bindings![KeyCode::ShiftLeft, GamepadButton::LeftTrigger2],
                ),
                (
                    Action::<CrouchAction>::new(),
                    bindings![KeyCode::ControlLeft, GamepadButton::East],
                ),
            ]),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Hero Body"),
                Mesh3d(meshes.add(Capsule3d::new(0.42, 1.2).mesh().rings(10).latitudes(14))),
                MeshMaterial3d(body_material),
                Transform::from_xyz(0.0, 0.95, 0.0),
            ));
            parent.spawn((
                Name::new("Hero Accent"),
                Mesh3d(meshes.add(Cuboid::new(0.22, 0.14, 0.62))),
                MeshMaterial3d(accent_material),
                Transform::from_xyz(0.0, 1.1, -0.2),
            ));
        })
        .id()
}

fn spawn_camera(commands: &mut Commands, player: Entity) -> Entity {
    let mut settings = ThirdPersonCameraSettings::default();
    settings.zoom.default_distance = 5.2;
    settings.zoom.min_distance = 2.4;
    settings.zoom.max_distance = 8.0;
    settings.screen_framing.enabled = true;
    settings.screen_framing.dead_zone = Vec2::new(0.10, 0.08);
    settings.screen_framing.soft_zone = Vec2::new(0.30, 0.24);
    settings.anchor.height = 1.45;

    let camera =
        ThirdPersonCamera::looking_at(Vec3::new(0.0, 1.35, 0.0), Vec3::new(0.8, 2.7, 5.6));

    commands
        .spawn((
            Name::new("Controller Follow Camera"),
            camera,
            settings,
            ThirdPersonCameraShoulderRig::default()
                .with_mode(ThirdPersonCameraMode::Shoulder)
                .with_shoulder_side(ShoulderSide::Right),
            ThirdPersonCameraShoulderSettings {
                aim_height_offset: -0.35,
                ..default()
            },
            ThirdPersonCameraLockOnSettings {
                enabled: true,
                max_distance: 34.0,
                ..default()
            },
            ThirdPersonCameraTarget {
                target: player,
                offset: Vec3::Y * 1.35,
                follow_rotation: false,
                enabled: true,
                ignore_children: true,
                ignored_entities: vec![player],
                recenter_on_target_change: true,
            },
            saddle_camera_third_person_camera::ThirdPersonCameraEnhancedInputTarget,
            default_input_bindings(),
        ))
        .id()
}

fn spawn_collision_world(commands: &mut Commands) {
    commands.spawn((
        Name::new("Ground Collider"),
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
    ));

    for (name, translation, size) in [
        (
            "Left Wall Collider",
            Vec3::new(-2.9, 1.6, -8.0),
            Vec3::new(0.4, 3.2, 26.0),
        ),
        (
            "Right Wall Collider",
            Vec3::new(2.9, 1.6, -8.0),
            Vec3::new(0.4, 3.2, 26.0),
        ),
        (
            "Entry Beam Collider",
            Vec3::new(0.0, 2.65, -3.5),
            Vec3::new(5.0, 0.45, 1.0),
        ),
        (
            "Far Gate Collider",
            Vec3::new(0.0, 1.8, -40.0),
            Vec3::new(6.0, 3.6, 0.5),
        ),
    ] {
        commands.spawn((
            Name::new(name),
            RigidBody::Static,
            Collider::cuboid(size.x, size.y, size.z),
            Transform::from_translation(translation),
        ));
    }

    for index in 0..5 {
        let z = -2.5 - index as f32 * 4.5;
        let x = if index % 2 == 0 { -1.55 } else { 1.55 };
        commands.spawn((
            Name::new(format!("Corridor Pillar Collider {}", index + 1)),
            RigidBody::Static,
            Collider::cuboid(0.65, 2.4, 0.65),
            Transform::from_xyz(x, 1.2, z),
        ));
    }
}

fn spawn_dressing(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    commands.spawn((
        Name::new("Moving Bridge"),
        MovingPlatform {
            origin: Vec3::new(0.0, 0.55, -14.0),
            axis: Vec3::X,
            amplitude: 2.4,
            speed: 0.8,
        },
        Mesh3d(meshes.add(Cuboid::new(5.5, 0.24, 3.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.28, 0.38),
            metallic: 0.05,
            perceptual_roughness: 0.72,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.55, -14.0),
        RigidBody::Kinematic,
        Collider::cuboid(5.5, 0.24, 3.0),
        LinearVelocity::ZERO,
    ));

    for (index, position) in [
        Vec3::new(-4.8, 0.65, -9.5),
        Vec3::new(4.5, 0.65, -21.0),
        Vec3::new(-4.2, 0.65, -30.0),
    ]
    .into_iter()
    .enumerate()
    {
        commands.spawn((
            Name::new(format!("Cover Crate {}", index + 1)),
            Mesh3d(meshes.add(Cuboid::new(1.4, 1.3, 1.4))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.34, 0.24, 0.18),
                perceptual_roughness: 0.84,
                ..default()
            })),
            Transform::from_translation(position),
            RigidBody::Static,
            Collider::cuboid(1.4, 1.3, 1.4),
            saddle_camera_third_person_camera::ThirdPersonCameraObstacle::default(),
        ));
    }
}

fn animate_platforms(
    time: Res<Time<Fixed>>,
    mut query: Query<(&MovingPlatform, &mut Transform, &mut LinearVelocity)>,
) {
    let t = time.elapsed_secs();
    for (platform, mut transform, mut velocity) in &mut query {
        let axis = platform.axis.normalize_or_zero();
        let phase = t * platform.speed;
        transform.translation = platform.origin + axis * (platform.amplitude * phase.sin());
        velocity.0 = axis * (platform.amplitude * platform.speed * phase.cos());
    }
}

fn align_character_to_camera(
    camera: Query<&ThirdPersonCamera>,
    mut player: Query<(&mut CharacterControllerState, &mut Transform), With<DemoPlayer>>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    let Ok((mut state, mut transform)) = player.single_mut() else {
        return;
    };
    let facing = Quat::from_rotation_y(camera.yaw);
    state.orientation = facing;
    transform.rotation = facing;
}

fn sync_hero_body_height(
    player: Query<(&CharacterControllerState, &Children), With<DemoPlayer>>,
    mut visuals: Query<&mut Transform, Without<DemoPlayer>>,
) {
    let Ok((state, children)) = player.single() else {
        return;
    };
    let body_height = if state.crouching { 0.72 } else { 0.95 };
    for child in children.iter() {
        let Ok(mut transform) = visuals.get_mut(child) else {
            continue;
        };
        transform.translation.y = body_height;
    }
}

fn sync_controller_pane(
    mut pane: ResMut<ControllerPane>,
    mut controllers: Query<(&mut CharacterController, &CharacterControllerState), With<DemoPlayer>>,
) {
    let Ok((mut controller, state)) = controllers.single_mut() else {
        return;
    };

    if pane.is_changed() && !pane.is_added() {
        controller.speed = pane.speed;
        controller.sprint_speed_scale = pane.sprint_speed_scale;
        controller.step_size = pane.step_size;
        controller.jump_height = pane.jump_height;
    }

    pane.grounded = state.ground.is_some_and(|ground| ground.walkable);
    pane.movement_mode = format!("{:?}", state.movement_mode);
}

fn update_overlay(
    entities: Res<DemoEntities>,
    player: Query<(&CharacterControllerState, &LinearVelocity), With<DemoPlayer>>,
    cameras: Query<(
        &ThirdPersonCamera,
        &ThirdPersonCameraRuntime,
        &ThirdPersonCameraTarget,
        Option<&ThirdPersonCameraShoulderRig>,
        Option<&saddle_camera_third_person_camera::ThirdPersonCameraLockOnRuntime>,
    )>,
    names: Query<&Name>,
    mut overlays: Query<&mut Text, With<common::DemoOverlay>>,
) {
    let Ok((state, velocity)) = player.get(entities.player) else {
        return;
    };
    let Ok((camera, runtime, target, shoulder_rig, lock_on_runtime)) = cameras.get(entities.camera) else {
        return;
    };
    let Ok(mut text) = overlays.single_mut() else {
        return;
    };

    let target_name = lock_on_runtime
        .and_then(|runtime| runtime.active_target)
        .and_then(|entity| names.get(entity).ok().map(Name::as_str))
        .unwrap_or("None");
    let mode = shoulder_rig.map_or(saddle_camera_third_person_camera::ThirdPersonCameraMode::Center, |rig| rig.mode);

    text.0 = format!(
        "Third Person + Character Controller\n\
         mode {:?} | yaw {:.2} pitch {:.2}\n\
         speed {:.2} | grounded {} | movement {:?}\n\
         desired {:.2} corrected {:.2}\n\
         lock target {target_name}\n\
         tracked entity {}",
        mode,
        camera.yaw,
        camera.pitch,
        velocity.0.xz().length(),
        state.ground.is_some_and(|ground| ground.walkable),
        state.movement_mode,
        runtime.desired_distance,
        runtime.corrected_distance,
        target.target.index(),
    );
}
