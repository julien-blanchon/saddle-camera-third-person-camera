# Saddle Camera Third Person Camera

Reusable third-person camera rig for Bevy with orbit, zoom, shoulder framing, aim mode, lock-on targeting, screen-space framing, cursor lock, obstruction handling, and crate-local lab verification.

The crate is built for generic third-person play rather than one specific genre. It can cover over-the-shoulder action cameras, centered platformer orbit cameras, inspection cameras with recentering disabled, and follow cameras for moving targets that finish motion late in the frame.

## Quick Start

```toml
[dependencies]
saddle-camera-third-person-camera = { git = "https://github.com/julien-blanchon/saddle-camera-third-person-camera" }
bevy = "0.18"
```

```rust,no_run
use bevy::prelude::*;
use saddle_camera_third_person_camera::{
    ThirdPersonCamera, ThirdPersonCameraInputTarget, ThirdPersonCameraPlugin,
    ThirdPersonCameraSettings, ThirdPersonCameraTarget, default_input_bindings,
};

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DemoState {
    #[default]
    Gameplay,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ThirdPersonCameraPlugin::new(
                OnEnter(DemoState::Gameplay),
                OnExit(DemoState::Gameplay),
                Update,
            ),
        ))
        .init_state::<DemoState>()
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let target = commands
        .spawn((
            Name::new("Camera Target"),
            Mesh3d(meshes.add(Capsule3d::new(0.4, 1.2))),
            MeshMaterial3d(materials.add(Color::srgb(0.77, 0.34, 0.28))),
            Transform::from_xyz(0.0, 1.0, 0.0),
        ))
        .id();

    commands.spawn((
        Name::new("Third Person Camera"),
        Camera3d::default(),
        ThirdPersonCamera::looking_at(Vec3::new(0.0, 1.35, 0.0), Vec3::new(0.0, 2.6, 4.6)),
        ThirdPersonCameraSettings::default(),
        ThirdPersonCameraTarget::new(target),
        ThirdPersonCameraInputTarget,
        default_input_bindings(),
    ));
}
```

For always-on tools and crate-local examples, `ThirdPersonCameraPlugin::default()` is the simplest constructor.

## Public API

| Type | Purpose |
| --- | --- |
| `ThirdPersonCameraPlugin` | Registers the runtime with injectable activate, deactivate, and update schedules |
| `ThirdPersonCameraSystems` | Public ordering hooks: `ReadInput`, `UpdateIntent`, `ResolveObstruction`, `ApplyTransform`, `DebugDraw` |
| `ThirdPersonCamera` | Main camera state: yaw, pitch, distance, persistent mode, shoulder side, large-target radius, and stored home values |
| `ThirdPersonCameraSettings` | Top-level tuning surface for orbit, smoothing, zoom, framing (including `aim_height_offset`), collision, recentering, and cursor policy |
| `ThirdPersonCameraTarget` | Follow-target descriptor: tracked entity, target-local offset, ignore rules, and retarget behavior |
| `ThirdPersonCameraRuntime` | Readable runtime state for debugging and external systems: pivot, desired vs corrected distance, hit data, blends, and effective camera positions |
| `ThirdPersonCameraInput` | Public input inbox for external systems that want to drive the camera directly |
| `ThirdPersonCameraInputTarget` | Opt-in marker for the camera that should consume the shared BEI action context |
| `ThirdPersonCameraLockOn` / `ThirdPersonCameraLockOnTarget` | Opt-in lock-on state and candidate marker for action-game targeting flows |
| `ThirdPersonCameraObstacle` | Opt-in obstruction marker for entities that should shorten or block the camera boom |
| `ThirdPersonCameraIgnore` / `ThirdPersonCameraIgnoreTarget` | Opt-in exclusions for camera collision and occlusion checks |
| `ThirdPersonCameraDebug` | Per-camera debug drawing toggles |
| `default_input_bindings()` | Default BEI action bundle for orbit, zoom, aim, recenter, cursor toggle, shoulder controls, and lock-on switching |

## Input Model

The crate owns a camera-oriented `bevy_enhanced_input` context. The default binding bundle exposes:

- `OrbitAction`: mouse delta and right stick orbit
- `ZoomAction`: wheel, D-pad, and page up/down zoom
- `AimAction`: right mouse or gamepad trigger hold
- `ToggleShoulderAction`: shoulder swap
- `ShoulderHoldAction`: temporary shoulder mode that falls back to the persistent mode when released
- `RecenterAction`: snap back to reference yaw and home pitch or distance
- `CursorLockAction`: toggle cursor grab when enabled by policy
- `ForceCenterModeAction` / `ForceShoulderModeAction`: explicit runtime mode switching
- `ToggleLockOnAction`, `NextLockOnTargetAction`, `PreviousLockOnTargetAction`: action-game lock-on controls

Only entities marked with `ThirdPersonCameraInputTarget` participate in shared input routing. If several active cameras have that marker, the highest `Camera.order` wins.

`ThirdPersonCamera::target_mode` is the persistent authored mode. Aim and shoulder-hold input layer temporary overrides on top of it instead of latching the camera into a new state.

## Ordering And Integration

- `ThirdPersonCameraSystems::ReadInput` runs in the plugin's injected update schedule after `EnhancedInputSystems::Apply`.
- `ThirdPersonCameraSystems::UpdateIntent`, `ResolveObstruction`, `ApplyTransform`, and `DebugDraw` run in `PostUpdate`.
- If your followed target finishes authoritative motion late in the frame, place that target-motion system before `ThirdPersonCameraSystems::UpdateIntent` in `PostUpdate`.
- If you write `ThirdPersonCameraInput` manually instead of using the default BEI bindings, write it after `ReadInput` or on the active input camera so it is not cleared by input routing.

## Obstruction Model

The runtime keeps desired camera pose separate from corrected camera pose. Obstruction handling is opt-in and AABB-based:

- mark blockers or occluders with `ThirdPersonCameraObstacle`
- provide a mesh-derived `Aabb` for accurate bounds when available
- otherwise the runtime falls back to a simple padded cube around the obstacle transform
- the target entity, optional target children, explicit ignored entities, and camera-side ignore markers are excluded from the cast

The default `MultiRay` strategy samples the boom center plus four near-plane-style offsets. `SphereProbe` adds more sample points for tighter shoulder and low-ceiling cases, while `SingleRay` is the cheap fallback.

`CollisionSettings::include_shape_radius` controls whether obstacle padding and sample clearance include the configured probe radius. Disable it only when you explicitly want point-like casts.

## Framing And Lock-On

- `ScreenSpaceFramingSettings` lets the target drift inside a dead zone, then gently recenters inside a larger soft zone before hard follow kicks in.
- `ThirdPersonCameraLockOn` keeps an active target entity, while nearby `ThirdPersonCameraLockOnTarget` markers provide candidates for toggle and cycle controls.
- When lock-on is active, the runtime blends the look target toward the selected target and can bias pitch and framing for soulslike or action-combat camera behavior.

## Examples

| Example | Purpose | Run |
| --- | --- | --- |
| `basic_follow` | Minimal follow, orbit, zoom, and pitch clamp | `cargo run -p saddle-camera-third-person-camera-example-basic-follow` |
| `character_controller` | Third-person gameplay lane integrated with `saddle-character-controller`, lock-on drones, and a live controller pane | `cargo run -p saddle-camera-third-person-camera-example-character-controller` |
| `lock_on` | Combat-style target lock, target cycling, and screen-space framing | `cargo run -p saddle-camera-third-person-camera-example-lock-on` |
| `shoulder_aim` | Shoulder framing, aim transitions, and shoulder swap parity | `cargo run -p saddle-camera-third-person-camera-example-shoulder-aim` |
| `collision_corridor` | Corridor, pillars, and beam obstruction pull-in or release | `cargo run -p saddle-camera-third-person-camera-example-collision-corridor` |
| `physics_target` | Late target motion ordered before camera intent in `PostUpdate` | `cargo run -p saddle-camera-third-person-camera-example-physics-target` |
| `gamepad` | Gamepad-focused orbit, zoom, and aim configuration | `cargo run -p saddle-camera-third-person-camera-example-gamepad` |
| `runtime_retarget` | Runtime target switching between multiple tracked entities | `cargo run -p saddle-camera-third-person-camera-example-runtime-retarget` |

Every example now includes a live `saddle-pane` panel so orbit, framing, and lock-on settings can be tuned while the scene is running.

## Crate-Local Lab

The richer verification app lives inside the crate at `shared/camera/saddle-camera-third-person-camera/examples/lab`:

```bash
cargo run -p saddle-camera-third-person-camera-lab
```

With E2E:

```bash
cargo run -p saddle-camera-third-person-camera-lab --features e2e -- third_person_camera_smoke
cargo run -p saddle-camera-third-person-camera-lab --features e2e -- third_person_camera_collision_corridor
cargo run -p saddle-camera-third-person-camera-lab --features e2e -- third_person_camera_lock_on
```

With BRP:

```bash
uv run --project .codex/skills/bevy-brp/script brp app launch saddle-camera-third-person-camera-lab
```

The lab itself supports interactive retargeting on `T`, lock-on on `F`, target cycling on `E` and `Z`, and cursor toggling on `Q`.

## More Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
