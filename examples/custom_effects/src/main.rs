use saddle_camera_third_person_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_third_person_camera::{
    AnchorSettings, CameraEffectLayer, ThirdPersonCamera, ThirdPersonCameraCustomEffects,
    ThirdPersonCameraEnhancedInputPlugin, ThirdPersonCameraPlugin, ThirdPersonCameraSettings,
    ThirdPersonCameraSystems,
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
        (
            common::animate_targets.before(ThirdPersonCameraSystems::UpdateIntent),
            (update_breathing, update_hit_flinch, update_landing_shake)
                .in_set(ThirdPersonCameraSystems::ComposeEffects),
        ),
    );
    app.add_systems(Update, trigger_effects);
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
        "custom_effects",
        "Custom camera effects demo. Press H: hit flinch, L: landing shake.\n\
         Breathing sway is always active. Mouse or right stick: orbit, scroll: zoom.",
        Color::srgb(0.58, 0.32, 0.72),
    );
    let target = common::spawn_target(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Effects Target",
        Color::srgb(0.72, 0.44, 0.86),
        Vec3::new(0.0, 1.1, -1.0),
        common::DemoMotionPath::Hover {
            center: Vec3::new(0.0, 1.1, -1.0),
            amplitude: 0.18,
            speed: 1.2,
        },
    );
    let camera = common::spawn_camera(
        &mut commands,
        "Effects Camera",
        target,
        Vec3::new(0.6, 2.0, 6.0),
        Vec3::new(0.0, 1.5, 0.0),
        ThirdPersonCamera::default(),
        ThirdPersonCameraSettings {
            anchor: AnchorSettings {
                height: 0.55,
                ..default()
            },
            ..default()
        },
        true,
    );
    commands.entity(camera).insert((
        ThirdPersonCameraCustomEffects::default(),
        EffectsState::default(),
    ));
}

#[derive(Component, Default)]
struct EffectsState {
    flinch_timer: f32,
    shake_timer: f32,
}

fn trigger_effects(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut EffectsState, With<ThirdPersonCamera>>,
) {
    for mut state in &mut query {
        if keys.just_pressed(KeyCode::KeyH) {
            state.flinch_timer = 0.35;
        }
        if keys.just_pressed(KeyCode::KeyL) {
            state.shake_timer = 0.5;
        }
    }
}

fn update_breathing(
    time: Res<Time>,
    mut query: Query<&mut ThirdPersonCameraCustomEffects, With<ThirdPersonCamera>>,
) {
    let t = time.elapsed_secs();
    for mut custom in &mut query {
        custom.set(
            "breathing",
            CameraEffectLayer::weighted(
                Vec3::new(0.0, (t * 1.2).sin() * 0.008, 0.0),
                Vec3::new((t * 0.9).sin() * 0.002, (t * 0.7).cos() * 0.001, 0.0),
                0.0,
                1.0,
            ),
        );
    }
}

fn update_hit_flinch(
    time: Res<Time>,
    mut query: Query<
        (&mut ThirdPersonCameraCustomEffects, &mut EffectsState),
        With<ThirdPersonCamera>,
    >,
) {
    let dt = time.delta_secs();
    for (mut custom, mut state) in &mut query {
        if state.flinch_timer > 0.0 {
            state.flinch_timer = (state.flinch_timer - dt).max(0.0);
            let intensity = state.flinch_timer / 0.35;
            let t = time.elapsed_secs();
            custom.set(
                "flinch",
                CameraEffectLayer::weighted(
                    Vec3::ZERO,
                    Vec3::new(
                        intensity * 0.06 * (t * 28.0).sin(),
                        intensity * 0.03 * (t * 19.0).cos(),
                        intensity * 0.02 * (t * 23.0).sin(),
                    ),
                    0.0,
                    1.0,
                ),
            );
        } else {
            custom.remove("flinch");
        }
    }
}

fn update_landing_shake(
    time: Res<Time>,
    mut query: Query<
        (&mut ThirdPersonCameraCustomEffects, &mut EffectsState),
        With<ThirdPersonCamera>,
    >,
) {
    let dt = time.delta_secs();
    for (mut custom, mut state) in &mut query {
        if state.shake_timer > 0.0 {
            state.shake_timer = (state.shake_timer - dt).max(0.0);
            let intensity = state.shake_timer / 0.5;
            custom.set(
                "landing_shake",
                CameraEffectLayer::weighted(
                    Vec3::new(0.0, -intensity * 0.04, 0.0),
                    Vec3::ZERO,
                    0.0,
                    1.0,
                ),
            );
        } else {
            custom.remove("landing_shake");
        }
    }
}
