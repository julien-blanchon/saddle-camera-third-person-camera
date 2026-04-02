use saddle_camera_third_person_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_third_person_camera::{
    ThirdPersonCamera, ThirdPersonCameraPlugin, ThirdPersonCameraTarget,
};

#[derive(Resource)]
struct RetargetState {
    camera: Entity,
    targets: [Entity; 3],
    timer: Timer,
    index: usize,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ThirdPersonCameraPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, advance_retarget)
        .add_systems(PostUpdate, common::animate_targets)
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
        "runtime_retarget",
        "The camera switches between three moving targets every three seconds.\nThis example exercises the stable public API for runtime target changes.",
        Color::srgb(0.22, 0.68, 0.46),
    );

    let targets = [
        common::spawn_target(
            &mut commands,
            &mut meshes,
            &mut materials,
            "Retarget Alpha",
            Color::srgb(0.85, 0.38, 0.30),
            Vec3::new(-5.0, 1.1, -4.0),
            common::DemoMotionPath::Circle {
                center: Vec3::new(-5.0, 1.1, -4.0),
                radius: 1.6,
                speed: 0.35,
                phase: 0.0,
            },
        ),
        common::spawn_target(
            &mut commands,
            &mut meshes,
            &mut materials,
            "Retarget Beta",
            Color::srgb(0.26, 0.58, 0.84),
            Vec3::new(0.0, 1.1, -7.0),
            common::DemoMotionPath::Hover {
                center: Vec3::new(0.0, 1.1, -7.0),
                amplitude: 0.45,
                speed: 1.6,
            },
        ),
        common::spawn_target(
            &mut commands,
            &mut meshes,
            &mut materials,
            "Retarget Gamma",
            Color::srgb(0.84, 0.62, 0.22),
            Vec3::new(5.0, 1.1, -3.0),
            common::DemoMotionPath::Circle {
                center: Vec3::new(5.0, 1.1, -3.0),
                radius: 1.9,
                speed: -0.30,
                phase: 1.1,
            },
        ),
    ];

    let camera = common::spawn_camera(
        &mut commands,
        "Retarget Camera",
        targets[0],
        Vec3::new(0.0, 2.6, 8.0),
        Vec3::new(0.0, 1.4, 0.0),
        ThirdPersonCamera::default(),
        saddle_camera_third_person_camera::ThirdPersonCameraSettings::default(),
        true,
    );

    commands.insert_resource(RetargetState {
        camera,
        targets,
        timer: Timer::from_seconds(3.0, TimerMode::Repeating),
        index: 0,
    });
}

fn advance_retarget(
    time: Res<Time>,
    mut state: ResMut<RetargetState>,
    mut cameras: Query<&mut ThirdPersonCameraTarget>,
) {
    if !state.timer.tick(time.delta()).just_finished() {
        return;
    }

    state.index = (state.index + 1) % state.targets.len();
    let Ok(mut target) = cameras.get_mut(state.camera) else {
        return;
    };
    target.target = state.targets[state.index];
}
