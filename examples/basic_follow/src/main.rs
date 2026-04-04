//! Basic third-person camera follow example.
//!
//! The simplest possible setup: a single target entity gently hovering in
//! place, orbited by a third-person camera with default settings and the
//! built-in `default_input_bindings()`.
//!
//! Controls: mouse move or right stick to orbit, scroll or d-pad up/down to
//! zoom, R to recenter, Q to toggle cursor lock.

use bevy::prelude::*;
use saddle_camera_third_person_camera::{
    ThirdPersonCamera, ThirdPersonCameraInputTarget, ThirdPersonCameraObstacle,
    ThirdPersonCameraPlugin, ThirdPersonCameraSettings, ThirdPersonCameraTarget,
    default_input_bindings,
};

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, ThirdPersonCameraPlugin::default()));
    app.add_systems(Startup, setup);
    app.add_systems(Update, animate_target);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // -- Lights --
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 28_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 18.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Name::new("Fill"),
        PointLight {
            intensity: 90_000.0,
            range: 60.0,
            ..default()
        },
        Transform::from_xyz(-6.0, 8.0, 10.0),
    ));

    // -- Floor --
    commands.spawn((
        Name::new("Floor"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(64.0, 64.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.09, 0.10, 0.13),
            perceptual_roughness: 1.0,
            ..default()
        })),
        ThirdPersonCameraObstacle::default(),
    ));

    // -- Target entity (capsule that gently hovers) --
    let target = commands
        .spawn((
            Name::new("Basic Target"),
            HoverTarget {
                center: Vec3::new(0.0, 1.1, 1.0),
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
            Transform::from_xyz(0.0, 1.1, 1.0),
        ))
        .id();

    // -- Camera --
    //
    // `ThirdPersonCameraTarget::new(target)` tells the camera which entity to
    // follow. `default_input_bindings()` wires mouse + gamepad via
    // bevy_enhanced_input so orbit, zoom, recenter, and cursor lock work
    // out of the box.
    let camera = ThirdPersonCamera::default();
    let settings = ThirdPersonCameraSettings::default();

    commands.spawn((
        Name::new("Basic Follow Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.4, 6.5).looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
        camera,
        settings,
        ThirdPersonCameraTarget::new(target),
        ThirdPersonCameraInputTarget,
        default_input_bindings(),
    ));

    // -- HUD --
    commands.spawn((
        Name::new("Overlay"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(18.0),
            width: Val::Px(430.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.82)),
        Text::new(
            "basic_follow\n\
             Mouse move or right stick: orbit. Scroll or d-pad up/down: zoom.\n\
             R recenter. Q cursor toggle.",
        ),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

// ---------------------------------------------------------------------------
// Simple hover animation for the target
// ---------------------------------------------------------------------------

#[derive(Component)]
struct HoverTarget {
    center: Vec3,
    amplitude: f32,
    speed: f32,
}

fn animate_target(time: Res<Time>, mut targets: Query<(&HoverTarget, &mut Transform)>) {
    for (hover, mut transform) in &mut targets {
        transform.translation =
            hover.center + Vec3::new(0.0, (time.elapsed_secs() * hover.speed).sin() * hover.amplitude, 0.0);
    }
}
