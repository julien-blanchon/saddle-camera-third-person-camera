//! Third-person camera gamepad-focused example.
//!
//! Starts in **Shoulder** mode to show the over-the-shoulder offset. The target
//! moves on a circular path so the camera tracks a moving subject. Debug gizmos
//! visualise the orbit and obstruction volumes.
//!
//! Controls: right stick orbit, d-pad up/down zoom, LT shoulder hold, RT aim,
//! d-pad right shoulder swap, North recenter, West cursor toggle.

use bevy::prelude::*;
use saddle_camera_third_person_camera::{
    ThirdPersonCamera, ThirdPersonCameraDebug, ThirdPersonCameraInputTarget, ThirdPersonCameraMode,
    ThirdPersonCameraObstacle, ThirdPersonCameraPlugin, ThirdPersonCameraSettings,
    ThirdPersonCameraTarget, default_input_bindings,
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

    // -- Target entity (capsule on a circular path) --
    let start = Vec3::new(0.0, 1.1, -4.0);
    let target = commands
        .spawn((
            Name::new("Gamepad Target"),
            CircleTarget {
                center: start,
                radius: 2.2,
                speed: 0.28,
                phase: 0.2,
            },
            Mesh3d(meshes.add(Capsule3d::new(0.45, 1.2).mesh().rings(10).latitudes(14))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.84, 0.32, 0.30),
                metallic: 0.04,
                perceptual_roughness: 0.26,
                ..default()
            })),
            Transform::from_translation(start),
        ))
        .id();

    // -- Camera in Shoulder mode --
    //
    // `ThirdPersonCameraMode::Shoulder` applies a lateral offset for the
    // over-the-shoulder framing common in action games. The default
    // input bindings include shoulder-hold (LT), aim (RT), swap (d-pad
    // right), recenter (North), and cursor toggle (West).
    let camera = ThirdPersonCamera::default().with_mode(ThirdPersonCameraMode::Shoulder);
    // shoulder_height is tuned for capsule-centered targets.
    let settings = ThirdPersonCameraSettings {
        framing: saddle_camera_third_person_camera::FramingSettings {
            shoulder_height: 0.55,
            aim_height_offset: -0.25,
            ..default()
        },
        ..default()
    };

    commands.spawn((
        Name::new("Gamepad Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.5, 6.8).looking_at(Vec3::new(0.0, 1.4, -1.0), Vec3::Y),
        camera,
        settings,
        ThirdPersonCameraTarget::new(target),
        ThirdPersonCameraInputTarget,
        default_input_bindings(),
        // Debug gizmos visualise the orbit sphere and obstruction rays
        ThirdPersonCameraDebug::default(),
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
            "gamepad\n\
             Right stick: orbit. D-pad up/down: zoom. LT: shoulder hold. RT: aim.\n\
             D-pad right: shoulder swap. North: recenter. West: cursor toggle.",
        ),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

// ---------------------------------------------------------------------------
// Circular motion for the target
// ---------------------------------------------------------------------------

#[derive(Component)]
struct CircleTarget {
    center: Vec3,
    radius: f32,
    speed: f32,
    phase: f32,
}

fn animate_target(time: Res<Time>, mut targets: Query<(&CircleTarget, &mut Transform)>) {
    for (circle, mut transform) in &mut targets {
        let previous = transform.translation;
        let angle = circle.phase + time.elapsed_secs() * circle.speed;
        transform.translation = circle.center
            + Vec3::new(
                angle.cos() * circle.radius,
                0.0,
                angle.sin() * circle.radius,
            );

        // Face the direction of movement
        let velocity = transform.translation - previous;
        let flat = Vec2::new(velocity.x, velocity.z);
        if flat.length_squared() > 0.0001 {
            transform.look_to(velocity.normalize_or_zero(), Vec3::Y);
        }
    }
}
