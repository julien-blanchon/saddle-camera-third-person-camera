use saddle_camera_third_person_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_third_person_camera::{
    CollisionSettings, CollisionStrategy, ThirdPersonCamera, ThirdPersonCameraPlugin,
    ThirdPersonCameraSettings,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ThirdPersonCameraPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(
            PostUpdate,
            common::animate_targets
                .before(saddle_camera_third_person_camera::ThirdPersonCameraSystems::UpdateIntent),
        )
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
        "collision_corridor",
        "The target moves through pillars, door frames, and a low beam.\nWatch the camera pull in and spring back instead of clipping.",
        Color::srgb(0.70, 0.52, 0.17),
    );
    common::spawn_box_obstacle(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Collision Gate",
        Vec3::new(0.0, 2.3, -0.3),
        Vec3::new(2.8, 1.0, 0.45),
        Color::srgb(0.72, 0.43, 0.16),
    );
    let target = common::spawn_target(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Collision Target",
        Color::srgb(0.25, 0.66, 0.74),
        Vec3::new(0.0, 1.1, 2.0),
        common::DemoMotionPath::Corridor {
            center: Vec3::new(0.0, 1.1, -6.0),
            half_length: 8.0,
            speed: 0.36,
        },
    );
    common::spawn_camera(
        &mut commands,
        "Collision Corridor Camera",
        target,
        Vec3::new(0.0, 2.4, 7.0),
        Vec3::new(0.0, 1.5, 0.0),
        ThirdPersonCamera::default(),
        ThirdPersonCameraSettings {
            collision: CollisionSettings {
                strategy: CollisionStrategy::SphereProbe,
                probe_radius: 0.38,
                sample_offset_x: 0.36,
                sample_offset_y: 0.28,
                ..default()
            },
            ..default()
        },
        true,
    );
}
