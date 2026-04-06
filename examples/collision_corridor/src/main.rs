use saddle_camera_third_person_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_third_person_camera::{
    AnchorSettings, CollisionSettings, CollisionStrategy, ThirdPersonCamera,
    ThirdPersonCameraEnhancedInputPlugin, ThirdPersonCameraPlugin, ThirdPersonCameraSettings,
    ThirdPersonCameraShoulderSettings,
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
        "collision_corridor",
        "The target moves through pillars, door frames, and a low beam.\n\
         Watch the camera pull in and spring back instead of clipping.\n\
         Right mouse: aim | C: swap shoulder | Mouse: orbit | Scroll: zoom\n\
         R: recenter | Q: toggle cursor lock",
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
    // Extra narrow doorway to demonstrate tight-space collision
    common::spawn_box_obstacle(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Narrow Doorway Left",
        Vec3::new(-1.1, 1.4, -8.0),
        Vec3::new(0.35, 2.8, 0.6),
        Color::srgb(0.52, 0.36, 0.20),
    );
    common::spawn_box_obstacle(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Narrow Doorway Right",
        Vec3::new(1.1, 1.4, -8.0),
        Vec3::new(0.35, 2.8, 0.6),
        Color::srgb(0.52, 0.36, 0.20),
    );
    common::spawn_box_obstacle(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Narrow Doorway Lintel",
        Vec3::new(0.0, 2.5, -8.0),
        Vec3::new(2.6, 0.35, 0.6),
        Color::srgb(0.52, 0.36, 0.20),
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
    let camera = common::spawn_camera(
        &mut commands,
        "Collision Corridor Camera",
        target,
        Vec3::new(0.0, 2.0, 7.0),
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
            ..default()
        },
        true,
    );
    commands.entity(camera).insert(ThirdPersonCameraShoulderSettings {
        aim_height_offset: -0.25,
        ..default()
    });
}
