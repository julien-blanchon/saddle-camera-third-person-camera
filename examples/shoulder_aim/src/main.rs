use saddle_camera_third_person_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_third_person_camera::{
    ShoulderSide, ThirdPersonCamera, ThirdPersonCameraMode, ThirdPersonCameraPlugin,
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
        "shoulder_aim",
        "Right mouse or LT: aim. C or d-pad right: swap shoulder.\nThe corridor is intentionally tight to show left/right parity.",
        Color::srgb(0.23, 0.63, 0.58),
    );
    let target = common::spawn_target(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Shoulder Target",
        Color::srgb(0.84, 0.38, 0.26),
        Vec3::new(0.0, 1.1, -1.0),
        common::DemoMotionPath::Corridor {
            center: Vec3::new(0.0, 1.1, -6.0),
            half_length: 6.0,
            speed: 0.42,
        },
    );
    common::spawn_camera(
        &mut commands,
        "Shoulder Aim Camera",
        target,
        Vec3::new(0.6, 2.3, 6.0),
        Vec3::new(0.0, 1.5, 0.0),
        ThirdPersonCamera::default()
            .with_mode(ThirdPersonCameraMode::Shoulder)
            .with_shoulder_side(ShoulderSide::Right),
        ThirdPersonCameraSettings {
            auto_recenter: saddle_camera_third_person_camera::AutoRecenterSettings {
                enabled: true,
                ..default()
            },
            ..default()
        },
        true,
    );
}
