use saddle_camera_third_person_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_third_person_camera::{ThirdPersonCamera, ThirdPersonCameraPlugin, ThirdPersonCameraSystems};

#[derive(Component)]
struct AuthoritativeMotion {
    center: Vec3,
    radius: f32,
    speed: f32,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ThirdPersonCameraPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(
            PostUpdate,
            move_authoritative_target.before(ThirdPersonCameraSystems::UpdateIntent),
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
        "physics_target",
        "This target is updated in PostUpdate before ThirdPersonCameraSystems::UpdateIntent.\nUse the same ordering when a physics backend writes transforms late in the frame.",
        Color::srgb(0.74, 0.32, 0.26),
    );
    let target = common::spawn_target(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Authoritative Target",
        Color::srgb(0.88, 0.48, 0.24),
        Vec3::new(2.5, 1.1, -3.0),
        common::DemoMotionPath::Hover {
            center: Vec3::new(2.5, 1.1, -3.0),
            amplitude: 0.0,
            speed: 0.0,
        },
    );
    commands.entity(target).insert(AuthoritativeMotion {
        center: Vec3::new(0.0, 1.1, -6.0),
        radius: 5.0,
        speed: 0.55,
    });
    common::spawn_camera(
        &mut commands,
        "Physics Friendly Camera",
        target,
        Vec3::new(0.0, 2.6, 7.5),
        Vec3::new(0.0, 1.4, 0.0),
        ThirdPersonCamera::default(),
        saddle_camera_third_person_camera::ThirdPersonCameraSettings::default(),
        true,
    );
}

fn move_authoritative_target(
    time: Res<Time>,
    mut query: Query<(&AuthoritativeMotion, &mut Transform)>,
) {
    for (motion, mut transform) in &mut query {
        let angle = time.elapsed_secs() * motion.speed;
        let previous = transform.translation;
        transform.translation = motion.center
            + Vec3::new(angle.cos() * motion.radius, 0.0, angle.sin() * motion.radius);
        let velocity = transform.translation - previous;
        if velocity.length_squared() > 0.0001 {
            transform.look_to(velocity.normalize_or_zero(), Vec3::Y);
        }
    }
}
