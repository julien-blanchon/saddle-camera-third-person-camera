use saddle_camera_third_person_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_third_person_camera::{
    LockOnSettings, ScreenSpaceFramingSettings, ThirdPersonCamera, ThirdPersonCameraPlugin,
    ThirdPersonCameraSettings, ThirdPersonCameraSystems,
};

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, ThirdPersonCameraPlugin::default()));
    common::add_debug_pane(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(
        PostUpdate,
        common::animate_targets.before(ThirdPersonCameraSystems::UpdateIntent),
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
        "lock_on",
        "F toggles lock-on. E selects the next target. Z selects the previous target.\nThe soft zone keeps the player framed while the focus shifts between threats.",
        Color::srgb(0.84, 0.32, 0.20),
    );

    let player = common::spawn_target(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lock On Hero",
        Color::srgb(0.92, 0.72, 0.28),
        Vec3::new(0.0, 1.1, 2.0),
        common::DemoMotionPath::Hover {
            center: Vec3::new(0.0, 1.1, 2.0),
            amplitude: 0.08,
            speed: 0.9,
        },
    );

    for (index, (position, color, phase)) in [
        (
            Vec3::new(-6.0, 1.1, -9.0),
            Color::srgb(0.86, 0.36, 0.32),
            0.0,
        ),
        (
            Vec3::new(0.0, 1.1, -12.0),
            Color::srgb(0.28, 0.74, 0.64),
            0.7,
        ),
        (
            Vec3::new(6.0, 1.1, -8.0),
            Color::srgb(0.30, 0.54, 0.88),
            1.4,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        common::spawn_target(
            &mut commands,
            &mut meshes,
            &mut materials,
            format!("Lock On Rival {}", index + 1),
            color,
            position,
            common::DemoMotionPath::Circle {
                center: position,
                radius: 1.8 + index as f32 * 0.4,
                speed: 0.25 + index as f32 * 0.06,
                phase,
            },
        );
    }

    common::spawn_camera(
        &mut commands,
        "Lock On Camera",
        player,
        Vec3::new(0.0, 2.6, 7.2),
        Vec3::new(0.0, 1.5, 0.0),
        ThirdPersonCamera::default(),
        ThirdPersonCameraSettings {
            lock_on: LockOnSettings {
                enabled: true,
                max_distance: 22.0,
                focus_bias: 0.42,
                ..default()
            },
            screen_framing: ScreenSpaceFramingSettings {
                enabled: true,
                dead_zone: Vec2::new(0.16, 0.12),
                soft_zone: Vec2::new(0.42, 0.30),
                screen_offset: Vec2::new(0.0, 0.06),
            },
            ..default()
        },
        true,
    );
}
