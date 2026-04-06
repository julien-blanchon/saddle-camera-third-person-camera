use saddle_camera_third_person_camera_example_common as common;

use bevy::{input::mouse::{MouseMotion, MouseWheel}, prelude::*};
use saddle_camera_third_person_camera::{
    AnchorSettings, ThirdPersonCamera, ThirdPersonCameraDebug, ThirdPersonCameraInput,
    ThirdPersonCameraPlugin, ThirdPersonCameraSettings, ThirdPersonCameraTarget,
};

#[derive(Component)]
struct HoverTarget {
    center: Vec3,
    amplitude: f32,
    speed: f32,
}

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, ThirdPersonCameraPlugin::default()));
    common::add_debug_pane(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(Update, (animate_target, drive_camera_input));
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
        "basic_follow",
        "Pure core integration: hold left mouse and drag to orbit, scroll to zoom, R to recenter.\nThis scene does not add the optional shoulder, lock-on, cursor, or enhanced-input adapters.",
        Color::srgb(0.22, 0.58, 0.82),
    );

    let target = commands
        .spawn((
            Name::new("Basic Hover Target"),
            HoverTarget {
                center: Vec3::new(0.0, 1.1, -1.0),
                amplitude: 0.18,
                speed: 1.2,
            },
            Mesh3d(meshes.add(Capsule3d::new(0.45, 1.2).mesh().rings(10).latitudes(14))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.86, 0.34, 0.32),
                metallic: 0.04,
                perceptual_roughness: 0.26,
                ..default()
            })),
            Transform::from_xyz(0.0, 1.1, -1.0),
        ))
        .id();

    commands.spawn((
        Name::new("Basic Follow Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 6.5).looking_at(Vec3::new(0.0, 1.4, -1.0), Vec3::Y),
        ThirdPersonCamera::default(),
        ThirdPersonCameraSettings {
            anchor: AnchorSettings {
                height: 0.55,
                ..default()
            },
            ..default()
        },
        ThirdPersonCameraTarget::new(target),
        ThirdPersonCameraDebug::default(),
    ));
}

fn animate_target(time: Res<Time>, mut targets: Query<(&HoverTarget, &mut Transform)>) {
    for (hover, mut transform) in &mut targets {
        transform.translation = hover.center
            + Vec3::new(
                0.0,
                (time.elapsed_secs() * hover.speed).sin() * hover.amplitude,
                0.0,
            );
    }
}

fn drive_camera_input(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut cameras: Query<&mut ThirdPersonCameraInput, With<ThirdPersonCamera>>,
) {
    let orbit_delta = if buttons.pressed(MouseButton::Left) {
        mouse_motion
            .read()
            .map(|event| event.delta)
            .sum::<Vec2>()
            * 0.006
    } else {
        mouse_motion.clear();
        Vec2::ZERO
    };
    let zoom_delta = mouse_wheel.read().map(|event| event.y).sum::<f32>();
    let recenter = keys.just_pressed(KeyCode::KeyR);

    for mut input in &mut cameras {
        input.orbit_delta += orbit_delta;
        input.zoom_delta += zoom_delta;
        input.recenter |= recenter;
    }
}
