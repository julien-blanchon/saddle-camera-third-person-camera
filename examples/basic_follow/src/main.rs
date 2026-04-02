use saddle_camera_third_person_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_third_person_camera::{ThirdPersonCamera, ThirdPersonCameraPlugin, ThirdPersonCameraSettings};

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
        "basic_follow",
        "Mouse move or right stick: orbit. Scroll or d-pad up/down: zoom.\nR recenter. Q cursor toggle.",
        Color::srgb(0.89, 0.55, 0.18),
    );
    let target = common::spawn_target(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Basic Target",
        Color::srgb(0.86, 0.34, 0.32),
        Vec3::new(0.0, 1.1, 1.0),
        common::DemoMotionPath::Hover {
            center: Vec3::new(0.0, 1.1, 1.0),
            amplitude: 0.18,
            speed: 1.2,
        },
    );
    common::spawn_camera(
        &mut commands,
        "Basic Follow Camera",
        target,
        Vec3::new(0.0, 2.4, 6.5),
        Vec3::new(0.0, 1.5, 0.0),
        ThirdPersonCamera::default(),
        ThirdPersonCameraSettings::default(),
        false,
    );
}
