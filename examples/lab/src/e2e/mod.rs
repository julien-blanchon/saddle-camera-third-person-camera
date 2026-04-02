use bevy::prelude::*;
use bevy_e2e::{
    E2EPlugin, E2ESet,
    action::Action,
    actions::{assertions, inspect},
    init_scenario,
    scenario::Scenario,
};
use bevy_enhanced_input::prelude::EnhancedInputSystems;
use saddle_camera_third_person_camera::{
    ThirdPersonCamera, ThirdPersonCameraRuntime, ThirdPersonCameraTarget,
};

use crate::{LabAlternateTarget, LabCameraEntity};

pub struct ThirdPersonCameraLabE2EPlugin;

impl Plugin for ThirdPersonCameraLabE2EPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(E2EPlugin);
        app.configure_sets(
            Update,
            (
                E2ESet.before(EnhancedInputSystems::Update),
                E2ESet
                    .before(saddle_camera_third_person_camera::ThirdPersonCameraSystems::ReadInput),
            ),
        );

        let args: Vec<String> = std::env::args().collect();
        let (scenario_name, handoff) = parse_e2e_args(&args);

        if let Some(name) = scenario_name {
            if let Some(mut scenario) = scenario_by_name(&name) {
                if handoff {
                    scenario.actions.push(Action::Handoff);
                }
                init_scenario(app, scenario);
            } else {
                error!(
                    "[saddle_camera_third_person_camera_lab:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
}

fn parse_e2e_args(args: &[String]) -> (Option<String>, bool) {
    let mut scenario_name = None;
    let mut handoff = false;
    for arg in args.iter().skip(1) {
        if arg == "--handoff" {
            handoff = true;
        } else if !arg.starts_with('-') && scenario_name.is_none() {
            scenario_name = Some(arg.clone());
        }
    }
    if !handoff {
        handoff = std::env::var("E2E_HANDOFF").is_ok_and(|value| value == "1" || value == "true");
    }
    (scenario_name, handoff)
}

fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "smoke_launch" => Some(build_smoke_launch()),
        "third_person_camera_smoke" => Some(build_smoke()),
        "third_person_camera_collision_corridor" => Some(build_collision()),
        "third_person_camera_shoulder_swap" => Some(build_shoulder_swap()),
        "third_person_camera_retarget" => Some(build_retarget()),
        _ => None,
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec![
        "smoke_launch",
        "third_person_camera_smoke",
        "third_person_camera_collision_corridor",
        "third_person_camera_shoulder_swap",
        "third_person_camera_retarget",
    ]
}

fn camera_entity(world: &World) -> Option<Entity> {
    world
        .get_resource::<LabCameraEntity>()
        .map(|resource| resource.0)
}

fn runtime(world: &World) -> Option<ThirdPersonCameraRuntime> {
    let entity = camera_entity(world)?;
    world.get::<ThirdPersonCameraRuntime>(entity).copied()
}

#[derive(Resource, Clone, Copy)]
struct CollisionCheckpoint {
    corrected_distance: f32,
}

fn store_collision_checkpoint(world: &mut World) {
    let Some(runtime) = runtime(world) else {
        return;
    };
    world.insert_resource(CollisionCheckpoint {
        corrected_distance: runtime.corrected_distance,
    });
}

fn build_smoke_launch() -> Scenario {
    Scenario::builder("smoke_launch")
        .description(
            "Boot the lab, verify the third-person runtime exists, then capture a baseline frame.",
        )
        .then(Action::WaitFrames(90))
        .then(assertions::entity_exists::<ThirdPersonCamera>(
            "camera entity exists",
        ))
        .then(assertions::component_satisfies::<ThirdPersonCameraRuntime>(
            "runtime initialized",
            |runtime| runtime.desired_distance > 0.0 && runtime.corrected_distance > 0.0,
        ))
        .then(assertions::log_summary("smoke_launch summary"))
        .then(Action::Screenshot("smoke_launch".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_smoke() -> Scenario {
    Scenario::builder("third_person_camera_smoke")
        .description("Exercise orbit and zoom through the real input path, assert yaw and distance changes, then capture before and after screenshots.")
        .then(Action::WaitFrames(60))
        .then(Action::Screenshot("third_person_camera_smoke_before".into()))
        .then(Action::WaitFrames(1))
        .then(Action::MouseMotion {
            delta: Vec2::new(180.0, -60.0),
        })
        .then(Action::WaitFrames(4))
        .then(Action::MouseScroll {
            delta: Vec2::new(0.0, 3.0),
        })
        .then(Action::WaitFrames(8))
        .then(assertions::custom("orbit and zoom changed runtime", |world| {
            let Some(runtime) = runtime(world) else {
                return false;
            };
            let Some(camera_entity) = camera_entity(world) else {
                return false;
            };
            let Some(camera) = world.get::<ThirdPersonCamera>(camera_entity) else {
                return false;
            };
            camera.yaw.abs() > 0.1 && runtime.corrected_distance < 4.6
        }))
        .then(assertions::log_summary("third_person_camera_smoke summary"))
        .then(inspect::dump_component_json::<ThirdPersonCameraRuntime>(
            "third_person_camera_smoke_runtime",
        ))
        .then(Action::Screenshot("third_person_camera_smoke_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_collision() -> Scenario {
    Scenario::builder("third_person_camera_collision_corridor")
        .description("Wait for the corridor motion to shorten the boom against geometry, then assert the camera springs back and capture both checkpoints.")
        .then(Action::WaitFrames(140))
        .then(assertions::component_satisfies::<ThirdPersonCameraRuntime>(
            "camera shortened because of obstruction",
            |runtime| {
                runtime.corrected_distance < runtime.desired_distance - 0.35
                    && runtime.last_hit_point.is_some()
                    && runtime.obstruction_active
            },
        ))
        .then(Action::Custom(Box::new(|world: &mut World| {
            store_collision_checkpoint(world);
        })))
        .then(Action::Screenshot("third_person_camera_collision_active".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(120))
        .then(assertions::custom(
            "camera springs back after obstruction clears",
            |world| {
                let Some(runtime) = runtime(world) else {
                    return false;
                };
                let Some(checkpoint) = world.get_resource::<CollisionCheckpoint>() else {
                    return false;
                };
                !runtime.obstruction_active
                    && runtime.corrected_distance > checkpoint.corrected_distance + 0.4
            },
        ))
        .then(assertions::log_summary("third_person_camera_collision_corridor summary"))
        .then(inspect::dump_component_json::<ThirdPersonCameraRuntime>(
            "third_person_camera_collision_runtime",
        ))
        .then(Action::Screenshot("third_person_camera_collision_recovered".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_shoulder_swap() -> Scenario {
    Scenario::builder("third_person_camera_shoulder_swap")
        .description("Swap shoulders, enter aim mode, and assert both the side and aim blend changed before capturing checkpoints.")
        .then(Action::WaitFrames(60))
        .then(Action::Screenshot("third_person_camera_shoulder_before".into()))
        .then(Action::WaitFrames(1))
        .then(Action::HoldKey {
            key: KeyCode::KeyC,
            frames: 2,
        })
        .then(Action::WaitFrames(16))
        .then(assertions::custom("shoulder side flipped", |world| {
            let Some(camera_entity) = camera_entity(world) else {
                return false;
            };
            let Some(camera) = world.get::<ThirdPersonCamera>(camera_entity) else {
                return false;
            };
            matches!(camera.target_shoulder_side, saddle_camera_third_person_camera::ShoulderSide::Left)
        }))
        .then(Action::PressMouseButton(MouseButton::Right))
        .then(Action::WaitFrames(24))
        .then(assertions::component_satisfies::<ThirdPersonCameraRuntime>(
            "aim blend became active",
            |runtime| runtime.aim_blend > 0.5,
        ))
        .then(Action::Screenshot("third_person_camera_shoulder_aim".into()))
        .then(Action::ReleaseMouseButton(MouseButton::Right))
        .then(Action::WaitFrames(12))
        .then(assertions::log_summary("third_person_camera_shoulder_swap summary"))
        .then(inspect::dump_component_json::<ThirdPersonCameraRuntime>(
            "third_person_camera_shoulder_runtime",
        ))
        .build()
}

fn build_retarget() -> Scenario {
    Scenario::builder("third_person_camera_retarget")
        .description("Switch the camera target at runtime and assert the tracked entity changed and the pivot moved toward it.")
        .then(Action::WaitFrames(60))
        .then(Action::Screenshot("third_person_camera_retarget_before".into()))
        .then(Action::WaitFrames(1))
        .then(Action::HoldKey {
            key: KeyCode::KeyT,
            frames: 2,
        })
        .then(Action::WaitFrames(50))
        .then(assertions::custom("tracked target is the alternate entity", |world| {
            let Some(camera_entity) = world.get_resource::<LabCameraEntity>().map(|resource| resource.0) else {
                return false;
            };
            let Some(alternate) = world.get_resource::<LabAlternateTarget>().map(|resource| resource.0) else {
                return false;
            };
            world
                .get::<ThirdPersonCameraTarget>(camera_entity)
                .is_some_and(|target| target.target == alternate)
        }))
        .then(assertions::component_satisfies::<ThirdPersonCameraRuntime>(
            "pivot moved toward alternate target",
            |runtime| runtime.pivot.x > 1.0,
        ))
        .then(assertions::log_summary("third_person_camera_retarget summary"))
        .then(inspect::dump_component_json::<ThirdPersonCameraRuntime>(
            "third_person_camera_retarget_runtime",
        ))
        .then(Action::Screenshot("third_person_camera_retarget_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}
