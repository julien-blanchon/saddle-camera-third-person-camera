use saddle_camera_third_person_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_third_person_camera::{
    ThirdPersonCamera, ThirdPersonCameraMode, ThirdPersonCameraPlugin,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ThirdPersonCameraPlugin::default()))
        .add_systems(Startup, setup)
        .run();
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
        "Right stick: orbit. D-pad up/down: zoom. LT: shoulder hold. RT: aim.\nD-pad right: shoulder swap. North: recenter. West: cursor toggle.",
        Color::srgb(0.30, 0.58, 0.82),
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
    common::spawn_camera(
        &mut commands,
        "Gamepad Camera",
        target,
        Vec3::new(0.0, 2.5, 6.8),
        Vec3::new(0.0, 1.4, -1.0),
        ThirdPersonCamera::default().with_mode(ThirdPersonCameraMode::Shoulder),
        saddle_camera_third_person_camera::ThirdPersonCameraSettings::default(),
        true,
    );
}
