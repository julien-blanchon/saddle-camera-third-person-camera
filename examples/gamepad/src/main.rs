use saddle_camera_third_person_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_third_person_camera::{
    AnchorSettings, ShoulderSide, ThirdPersonCamera, ThirdPersonCameraEnhancedInputPlugin,
    ThirdPersonCameraMode, ThirdPersonCameraPlugin, ThirdPersonCameraSettings,
    ThirdPersonCameraShoulderRig, ThirdPersonCameraShoulderSettings,
};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        ThirdPersonCameraPlugin::default(),
        ThirdPersonCameraEnhancedInputPlugin::default(),
    ));
    common::add_debug_pane(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(
        PostUpdate,
        common::animate_targets
            .before(saddle_camera_third_person_camera::ThirdPersonCameraSystems::UpdateIntent),
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
        "gamepad",
        "Right stick: orbit | D-pad up/down: zoom | LT: shoulder hold | RT: aim\nD-pad right: shoulder swap | North: recenter | West: cursor toggle",
        Color::srgb(0.76, 0.32, 0.26),
    );

    let target = common::spawn_target(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Gamepad Target",
        Color::srgb(0.84, 0.32, 0.30),
        Vec3::new(0.0, 1.1, -4.0),
        common::DemoMotionPath::Circle {
            center: Vec3::new(0.0, 1.1, -4.0),
            radius: 2.2,
            speed: 0.28,
            phase: 0.2,
        },
    );

    let camera = common::spawn_camera(
        &mut commands,
        "Gamepad Camera",
        target,
        Vec3::new(0.0, 2.5, 6.8),
        Vec3::new(0.0, 1.4, -1.0),
        ThirdPersonCamera::default(),
        ThirdPersonCameraSettings {
            anchor: AnchorSettings {
                height: 0.55,
                ..default()
            },
            ..default()
        },
        true,
    );

    commands.entity(camera).insert((
        ThirdPersonCameraShoulderRig::default()
            .with_mode(ThirdPersonCameraMode::Shoulder)
            .with_shoulder_side(ShoulderSide::Right),
        ThirdPersonCameraShoulderSettings {
            aim_height_offset: -0.25,
            ..default()
        },
    ));
}
