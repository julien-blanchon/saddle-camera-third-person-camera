use saddle_camera_third_person_camera_example_common as common;
#[cfg(feature = "e2e")]
mod e2e;

use bevy::prelude::*;
#[cfg(feature = "brp")]
use bevy_brp_extras::BrpExtrasPlugin;
#[cfg(feature = "e2e")]
use saddle_bevy_e2e::E2ESet;
use saddle_camera_third_person_camera::{
    AnchorSettings, CollisionSettings, CollisionStrategy, ThirdPersonCamera,
    ThirdPersonCameraCursorController, ThirdPersonCameraEnhancedInputPlugin,
    ThirdPersonCameraLockOnRuntime, ThirdPersonCameraLockOnSettings, ThirdPersonCameraMode,
    ThirdPersonCameraPlugin, ThirdPersonCameraRuntime, ThirdPersonCameraSettings,
    ThirdPersonCameraShoulderRig, ThirdPersonCameraShoulderRuntime,
    ThirdPersonCameraShoulderSettings, ThirdPersonCameraSystems, ThirdPersonCameraTarget,
};

#[derive(Resource, Clone, Copy)]
pub struct LabCameraEntity(pub Entity);

#[derive(Resource, Clone, Copy)]
pub struct LabPrimaryTarget(pub Entity);

#[derive(Resource, Clone, Copy)]
pub struct LabAlternateTarget(pub Entity);

#[derive(Resource, Clone, Copy)]
pub struct LabReserveTarget(pub Entity);

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "saddle_camera_third_person_camera_lab".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        ThirdPersonCameraPlugin::default(),
        ThirdPersonCameraEnhancedInputPlugin::default(),
    ));
    #[cfg(not(feature = "e2e"))]
    common::add_debug_pane(&mut app);
    #[cfg(feature = "brp")]
    app.add_plugins(BrpExtrasPlugin::default());
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::ThirdPersonCameraLabE2EPlugin);

    app.add_systems(Startup, setup);
    #[cfg(not(feature = "e2e"))]
    app.add_systems(Update, toggle_target);
    #[cfg(feature = "e2e")]
    app.add_systems(Update, toggle_target.after(E2ESet));
    app.add_systems(
        PostUpdate,
        common::animate_targets.before(ThirdPersonCameraSystems::UpdateIntent),
    );
    app.add_systems(Update, update_overlay);
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
        "saddle_camera_third_person_camera_lab",
        "Shared-crate lab: orbit, zoom, aim, shoulder swap, retarget, and obstruction.\nRMB aim, C swap shoulder, T retarget, R recenter, 1/2 center/shoulder, Q cursor toggle.",
        Color::srgb(0.85, 0.48, 0.18),
    );

    let primary = common::spawn_target(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Primary Target",
        Color::srgb(0.86, 0.34, 0.30),
        Vec3::new(0.0, 1.1, -2.0),
        common::DemoMotionPath::Corridor {
            center: Vec3::new(0.0, 1.1, -7.0),
            half_length: 8.5,
            speed: 0.34,
        },
    );
    let alternate = common::spawn_target(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Alternate Target",
        Color::srgb(0.24, 0.58, 0.82),
        Vec3::new(3.4, 1.2, -10.0),
        common::DemoMotionPath::Circle {
            center: Vec3::new(3.4, 1.2, -10.0),
            radius: 1.4,
            speed: -0.36,
            phase: 0.8,
        },
    );
    let reserve = common::spawn_target(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Reserve Lock-On Target",
        Color::srgb(0.52, 0.82, 0.46),
        Vec3::new(-3.2, 1.25, -11.0),
        common::DemoMotionPath::Hover {
            center: Vec3::new(-3.2, 1.25, -11.0),
            amplitude: 0.4,
            speed: 0.9,
        },
    );
    common::spawn_box_obstacle(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Collision Gate",
        Vec3::new(0.0, 2.3, -0.3),
        Vec3::new(2.8, 1.0, 0.45),
        Color::srgb(0.78, 0.45, 0.17),
    );

    let camera = common::spawn_camera(
        &mut commands,
        "Lab Third Person Camera",
        primary,
        Vec3::new(0.6, 2.0, 7.2),
        Vec3::new(0.0, 1.5, 0.0),
        ThirdPersonCamera::default(),
        ThirdPersonCameraSettings {
            anchor: AnchorSettings {
                height: 0.55,
                ..default()
            },
            collision: CollisionSettings {
                strategy: CollisionStrategy::SphereProbe,
                probe_radius: 0.38,
                sample_offset_x: 0.36,
                sample_offset_y: 0.28,
                ..default()
            },
            auto_recenter: saddle_camera_third_person_camera::AutoRecenterSettings {
                enabled: true,
                ..default()
            },
            ..default()
        },
        true,
    );
    commands.entity(camera).insert((
        ThirdPersonCameraShoulderRig::default().with_mode(ThirdPersonCameraMode::Shoulder),
        ThirdPersonCameraShoulderSettings {
            aim_height_offset: -0.25,
            ..default()
        },
        ThirdPersonCameraLockOnSettings {
            enabled: true,
            max_distance: 20.0,
            ..default()
        },
    ));

    commands.insert_resource(LabCameraEntity(camera));
    commands.insert_resource(LabPrimaryTarget(primary));
    commands.insert_resource(LabAlternateTarget(alternate));
    commands.insert_resource(LabReserveTarget(reserve));
}

fn toggle_target(
    keys: Res<ButtonInput<KeyCode>>,
    camera_entity: Res<LabCameraEntity>,
    primary: Res<LabPrimaryTarget>,
    alternate: Res<LabAlternateTarget>,
    mut cameras: Query<&mut ThirdPersonCameraTarget>,
) {
    if !keys.just_pressed(KeyCode::KeyT) {
        return;
    }

    let Ok(mut camera_target) = cameras.get_mut(camera_entity.0) else {
        return;
    };
    camera_target.target = if camera_target.target == primary.0 {
        alternate.0
    } else {
        primary.0
    };
}

fn update_overlay(
    camera_entity: Res<LabCameraEntity>,
    primary: Res<LabPrimaryTarget>,
    alternate: Res<LabAlternateTarget>,
    cameras: Query<(
        &ThirdPersonCamera,
        &ThirdPersonCameraRuntime,
        &ThirdPersonCameraTarget,
        Option<&ThirdPersonCameraShoulderRig>,
        Option<&ThirdPersonCameraShoulderRuntime>,
        Option<&ThirdPersonCameraLockOnRuntime>,
        Option<&ThirdPersonCameraCursorController>,
    )>,
    names: Query<&Name>,
    mut overlays: Query<&mut Text, With<common::DemoOverlay>>,
) {
    let Ok((camera, runtime, target, shoulder_rig, shoulder_runtime, _lock_on_runtime, cursor_controller)) =
        cameras.get(camera_entity.0)
    else {
        return;
    };
    let Ok(mut text) = overlays.single_mut() else {
        return;
    };
    let target_name = names
        .get(target.target)
        .map(Name::as_str)
        .unwrap_or("Unknown Target");

    *text = Text::new(format!(
        "saddle_camera_third_person_camera_lab\n\
         effective mode {:?} | persistent mode {:?}\n\
         yaw {:.2} pitch {:.2}\n\
         desired {:.2} corrected {:.2}\n\
         obstruction {} | cursor locked {}\n\
         shoulder {:.2} aim {:.2}\n\
         tracked target {target_name}\n\
         primary {} alternate {}",
        shoulder_rig.map_or(ThirdPersonCameraMode::Center, |rig| rig.mode),
        shoulder_rig.map_or(ThirdPersonCameraMode::Center, |rig| rig.target_mode),
        camera.yaw,
        camera.pitch,
        runtime.desired_distance,
        runtime.corrected_distance,
        runtime.obstruction_active,
        cursor_controller.is_some_and(|controller| controller.locked),
        shoulder_runtime.map_or(0.0, |runtime| runtime.shoulder_blend),
        shoulder_runtime.map_or(0.0, |runtime| runtime.aim_blend),
        primary.0.index(),
        alternate.0.index(),
    ));
}
