# Saddle Camera Third Person Camera

Reusable Bevy 0.18 camera rig for follow, orbit, zoom, smoothing, screen-space framing, and obstruction handling.

The crate now has a small generic core plus opt-in adapters:

- core rig: `ThirdPersonCamera`, `ThirdPersonCameraSettings`, `ThirdPersonCameraTarget`, `ThirdPersonCameraInput`
- optional shoulder/aim adapter: `ThirdPersonCameraShoulderRig`, `ThirdPersonCameraShoulderSettings`
- optional lock-on adapter: `ThirdPersonCameraLockOn`, `ThirdPersonCameraLockOnSettings`, `ThirdPersonCameraLockOnTarget`
- optional cursor adapter: `ThirdPersonCameraCursorController`
- optional `bevy_enhanced_input` adapter: `ThirdPersonCameraEnhancedInputPlugin`, `ThirdPersonCameraEnhancedInputTarget`, `default_input_bindings()`

## Quick Start

Core-only integration does not need the optional action adapters or the enhanced-input plugin.

```toml
[dependencies]
bevy = "0.18"
saddle-camera-third-person-camera = { git = "https://github.com/julien-blanchon/saddle-camera-third-person-camera", default-features = false }
```

```rust,no_run
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use saddle_camera_third_person_camera::{
    AnchorSettings, ThirdPersonCamera, ThirdPersonCameraInput, ThirdPersonCameraPlugin,
    ThirdPersonCameraSettings, ThirdPersonCameraTarget,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ThirdPersonCameraPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, drive_camera_input)
        .run();
}

fn setup(mut commands: Commands) {
    let target = commands
        .spawn((
            Name::new("Camera Target"),
            Transform::from_xyz(0.0, 1.0, 0.0),
            GlobalTransform::default(),
        ))
        .id();

    commands.spawn((
        Name::new("Third Person Camera"),
        Camera3d::default(),
        ThirdPersonCamera::looking_at(Vec3::new(0.0, 1.55, 0.0), Vec3::new(0.0, 2.6, 4.6)),
        ThirdPersonCameraSettings {
            anchor: AnchorSettings {
                height: 0.55,
                ..default()
            },
            ..default()
        },
        ThirdPersonCameraTarget::new(target),
    ));
}

fn drive_camera_input(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut cameras: Query<&mut ThirdPersonCameraInput>,
) {
    let orbit_delta = if buttons.pressed(MouseButton::Left) {
        mouse_motion.read().map(|event| event.delta).sum::<Vec2>() * 0.006
    } else {
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
```

If you want the old action-camera behavior, add the enhanced-input plugin and the optional adapter components to the camera entity.

## Public API

### Core

| Type | Purpose |
| --- | --- |
| `ThirdPersonCameraPlugin` | Registers the runtime with injectable activate, deactivate, and update schedules |
| `ThirdPersonCameraSystems` | Public ordering hooks: `ReadInput`, `UpdateIntent`, `ResolveObstruction`, `ApplyTransform`, `DebugDraw` |
| `ThirdPersonCamera` | Core orbit state: yaw, pitch, distance, home values, and large-target radius |
| `ThirdPersonCameraSettings` | Generic tuning surface for orbit, anchor height, smoothing, zoom, screen framing, collision, and idle recentering |
| `ThirdPersonCameraTarget` | Follow-target descriptor: tracked entity, target-local offset, ignore rules, and retarget behavior |
| `ThirdPersonCameraRuntime` | Runtime debug surface for pivot, desired vs corrected distance, hit data, and effective camera positions |
| `ThirdPersonCameraInput` | Generic transient input inbox: orbit delta, zoom delta, and recenter |
| `ThirdPersonCameraObstacle` | Opt-in obstruction marker for entities that should shorten or block the camera boom |
| `ThirdPersonCameraIgnore` / `ThirdPersonCameraIgnoreTarget` | Opt-in exclusions for camera collision and occlusion checks |
| `ThirdPersonCameraDebug` | Per-camera debug drawing toggles |

### Optional Adapters

| Type | Purpose |
| --- | --- |
| `ThirdPersonCameraShoulderRig` / `ThirdPersonCameraShoulderSettings` | Opt-in shoulder and aim framing state plus tuning |
| `ThirdPersonCameraLockOn` / `ThirdPersonCameraLockOnSettings` / `ThirdPersonCameraLockOnTarget` | Opt-in target lock and target cycling |
| `ThirdPersonCameraCursorController` | Opt-in cursor lock ownership for the active camera |
| `ThirdPersonCameraEnhancedInputPlugin` | Optional `bevy_enhanced_input` bridge for the crate’s demo bindings |
| `ThirdPersonCameraEnhancedInputTarget` | Marker for the camera that should consume the shared BEI context |
| `default_input_bindings()` | Default BEI action bundle for orbit, zoom, aim, shoulder controls, cursor toggle, and lock-on switching |

## Integration Model

- The core camera does not assume an action-game vocabulary.
- External systems write transient values into `ThirdPersonCameraInput`.
- Shoulder/aim, lock-on, and cursor ownership only activate when their adapter components are present.
- The enhanced-input adapter is optional and lives behind the crate’s default `enhanced-input` feature. Disable default features if you want the smallest core-only dependency surface.

## Ordering

- `ThirdPersonCameraSystems::ReadInput` is reserved for input adapters such as the optional BEI bridge.
- `ThirdPersonCameraSystems::UpdateIntent`, `ResolveObstruction`, `ApplyTransform`, and `DebugDraw` run in `PostUpdate`.
- If your followed target finishes authoritative motion late in the frame, order that system before `ThirdPersonCameraSystems::UpdateIntent`.

## Obstruction Model

The runtime keeps desired camera pose separate from corrected camera pose.

- mark blockers or occluders with `ThirdPersonCameraObstacle`
- provide a mesh-derived `Aabb` for accurate bounds when available
- otherwise the runtime falls back to a padded cube around the obstacle transform
- target-owned entities and explicit ignore markers are excluded from the cast

`MultiRay` remains the default. `SphereProbe` is the tighter but more expensive option for shoulders, ceilings, and narrow corridors.

## Examples

| Example | Purpose | Run |
| --- | --- | --- |
| `basic_follow` | Pure core integration path with manual Bevy input wiring and no action adapters | `cargo run -p saddle-camera-third-person-camera-example-basic-follow` |
| `gamepad` | Enhanced-input adapter plus shoulder framing for a gamepad-first camera | `cargo run -p saddle-camera-third-person-camera-example-gamepad` |
| `shoulder_aim` | Shoulder framing, aim transitions, and shoulder swap parity | `cargo run -p saddle-camera-third-person-camera-example-shoulder-aim` |
| `lock_on` | Lock-on target selection, target cycling, and screen-space framing | `cargo run -p saddle-camera-third-person-camera-example-lock-on` |
| `collision_corridor` | Corridor, pillars, and beam obstruction pull-in and release | `cargo run -p saddle-camera-third-person-camera-example-collision-corridor` |
| `physics_target` | Late target motion ordered before camera intent in `PostUpdate` | `cargo run -p saddle-camera-third-person-camera-example-physics-target` |
| `runtime_retarget` | Runtime target switching between multiple tracked entities | `cargo run -p saddle-camera-third-person-camera-example-runtime-retarget` |
| `character_controller` | Cross-crate controller lane using the action adapters and a live pane | `cargo run -p saddle-camera-third-person-camera-example-character-controller` |

Every example includes a live `saddle-pane` panel and on-screen instructions.

## Crate-Local Lab

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

## More Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
