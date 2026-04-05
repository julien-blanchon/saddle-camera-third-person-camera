# Configuration

This crate keeps camera tuning in explicit configuration structs rather than hidden constants. Defaults target a shoulder-capable action camera with collision pull-in and a conservative zoom range.

## Top-Level Settings

### `ThirdPersonCameraSettings`

| Field | Type | Default | Valid Range | Behavior |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `true` or `false` | Disables all runtime phases for the camera when `false` |
| `orbit` | `OrbitSettings` | `OrbitSettings::default()` | See below | Orbit speed, pitch clamp, and inversion |
| `smoothing` | `SmoothingSettings` | `SmoothingSettings::default()` | See below | Follow, aim, shoulder, zoom, and obstruction smoothing |
| `zoom` | `ZoomSettings` | `ZoomSettings::default()` | See below | Designer zoom range and zoom step |
| `framing` | `FramingSettings` | `FramingSettings::default()` | See below | Shoulder framing, aim framing, and target clearance |
| `screen_framing` | `ScreenSpaceFramingSettings` | `ScreenSpaceFramingSettings::default()` | See below | Screen-space dead-zone and soft-zone follow behavior |
| `lock_on` | `LockOnSettings` | `LockOnSettings::default()` | See below | Action-game lock-on targeting behavior |
| `collision` | `CollisionSettings` | `CollisionSettings::default()` | See below | Obstruction strategy and minimum camera distance |
| `auto_recenter` | `AutoRecenterSettings` | `AutoRecenterSettings::default()` | See below | Idle recentering policy |
| `cursor` | `CursorPolicy` | `CursorPolicy::default()` | See below | Cursor lock defaults and toggle permission |

## Camera Component

### `ThirdPersonCamera`

These are the authored starting values on the camera entity itself rather than fields inside `ThirdPersonCameraSettings`.

| Field | Type | Default | Valid Range | Behavior |
| --- | --- | --- | --- | --- |
| `yaw` | `f32` | `0.0` | Any finite angle | Current yaw in radians |
| `pitch` | `f32` | `-0.42` | Clamped to `OrbitSettings` at runtime | Current pitch in radians |
| `distance` | `f32` | `4.6` | Clamped to zoom plus large-target minimum at runtime | Current designer-facing boom length |
| `target_yaw` | `f32` | `0.0` | Any finite angle | Smoothed yaw target |
| `target_pitch` | `f32` | `-0.42` | Clamped to `OrbitSettings` | Smoothed pitch target |
| `target_distance` | `f32` | `4.6` | Clamped to zoom plus large-target minimum | Smoothed distance target |
| `shoulder_side` | `ShoulderSide` | `Right` | `Left` or `Right` | Current shoulder side after smoothing |
| `target_shoulder_side` | `ShoulderSide` | `Right` | `Left` or `Right` | Desired shoulder side |
| `mode` | `ThirdPersonCameraMode` | `Center` | `Center`, `Shoulder`, `Aim` | Current effective mode after smoothing |
| `target_mode` | `ThirdPersonCameraMode` | `Center` | `Center`, `Shoulder`, `Aim` | Persistent authored mode before temporary aim or shoulder-hold overrides |
| `large_target_radius` | `f32` | `0.0` | `>= 0.0` | Raises look-anchor clearance and the minimum allowed camera distance for large subjects |
| `home_yaw` / `home_pitch` / `home_distance` | `f32` | match defaults | Finite values | Recenter destination for orientation and zoom |
| `home_shoulder_side` / `home_mode` | enum | match defaults | enum variants | Recenter destination for framing |

## Orbit

### `OrbitSettings`

| Field | Type | Default | Valid Range | Behavior |
| --- | --- | --- | --- | --- |
| `yaw_speed` | `f32` | `1.2` | `> 0.0` recommended | Multiplier applied to orbit X input before changing target yaw |
| `pitch_speed` | `f32` | `1.1` | `> 0.0` recommended | Multiplier applied to orbit Y input before changing target pitch |
| `min_pitch` | `f32` | `-1.25` | Less than `max_pitch` | Lowest allowed target pitch in radians |
| `max_pitch` | `f32` | `-0.08` | Greater than `min_pitch` | Highest allowed target pitch in radians |
| `invert_x` | `bool` | `false` | `true` or `false` | Inverts yaw input when `true` |
| `invert_y` | `bool` | `false` | `true` or `false` | Inverts pitch input when `true` |

## Smoothing

### `SmoothingSettings`

All smoothing values are exponential response rates. Higher numbers settle faster.

| Field | Type | Default | Valid Range | Behavior |
| --- | --- | --- | --- | --- |
| `orientation_smoothing` | `f32` | `16.0` | `>= 0.0` | Yaw and pitch convergence rate |
| `target_follow_smoothing` | `f32` | `18.0` | `>= 0.0` | Pivot follow smoothing rate |
| `zoom_smoothing` | `f32` | `16.0` | `>= 0.0` | Distance convergence rate after zoom input |
| `obstruction_pull_in` | `f32` | `28.0` | `>= 0.0` | Speed for shortening the boom when an obstacle appears |
| `obstruction_release` | `f32` | `10.0` | `>= 0.0` | Speed for restoring boom length after an obstacle clears |
| `shoulder_blend` | `f32` | `14.0` | `>= 0.0` | Shoulder-side transition rate |
| `aim_blend` | `f32` | `20.0` | `>= 0.0` | Center-to-aim framing transition rate |

## Zoom

### `ZoomSettings`

| Field | Type | Default | Valid Range | Behavior |
| --- | --- | --- | --- | --- |
| `min_distance` | `f32` | `1.15` | `> 0.0` and `<= max_distance` | Smallest allowed designer zoom distance |
| `max_distance` | `f32` | `10.0` | `>= min_distance` | Largest allowed designer zoom distance |
| `default_distance` | `f32` | `4.6` | Within `[min_distance, max_distance]` | Initial and home distance for the default camera |
| `step` | `f32` | `0.8` | `> 0.0` recommended | Scalar applied to zoom input before clamping |

## Framing

### `FramingSettings`

| Field | Type | Default | Valid Range | Behavior |
| --- | --- | --- | --- | --- |
| `shoulder_offset` | `f32` | `0.75` | `>= 0.0` | Horizontal offset applied when in shoulder or aim mode |
| `shoulder_height` | `f32` | `1.35` | `>= 0.0` | Vertical lift from target origin to look anchor |
| `default_side` | `ShoulderSide` | `Right` | `Left` or `Right` | Preferred shoulder side applied when a newly added camera still uses the crate defaults |
| `aim_enabled` | `bool` | `true` | `true` or `false` | Allows `Aim` mode and aim-distance scaling |
| `aim_distance_scale` | `f32` | `0.62` | `(0.0, 1.0]` recommended | Multiplies camera distance while in aim mode |
| `aim_pitch_offset` | `f32` | `0.10` | Any finite value | Additional pitch applied in aim mode |
| `aim_height_offset` | `f32` | `-0.35` | Any finite value | Vertical offset applied to the look anchor in aim mode, blended by aim blend. Negative values pull the pivot down toward true shoulder level |
| `target_radius_clearance` | `f32` | `0.15` | `>= 0.0` | Extra look-anchor lift that helps large targets avoid near-clip face shots |

## Screen Framing

### `ScreenSpaceFramingSettings`

| Field | Type | Default | Valid Range | Behavior |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `false` | `true` or `false` | Enables screen-space framing rather than pure world-space follow |
| `dead_zone` | `Vec2` | `(0.18, 0.14)` | `0.0..1.0` per axis | Region where the target can move without camera response |
| `soft_zone` | `Vec2` | `(0.42, 0.32)` | Larger than `dead_zone`, typically `< 1.0` | Region where the camera eases back toward the target before hard follow |
| `screen_offset` | `Vec2` | `(0.0, 0.0)` | roughly `-0.5..0.5` | Biases the target anchor away from exact screen center |

Dead zone and soft zone are normalized viewport fractions. A larger soft zone slows visible recentering, while a smaller dead zone makes the camera feel tighter and more responsive.

## Lock On

### `LockOnSettings`

| Field | Type | Default | Valid Range | Behavior |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `false` | `true` or `false` | Enables lock-on selection and runtime facing behavior |
| `max_distance` | `f32` | `24.0` | `> 0.0` | Maximum distance from the camera pivot to candidate targets |
| `focus_bias` | `f32` | `0.35` | `0.0..1.0` | Biases the look-target blend between the player pivot and the lock-on target |
| `pitch_offset` | `f32` | `0.08` | Any finite value | Offsets pitch while lock-on is active to keep both combatants framed |

Candidate targets must carry `ThirdPersonCameraLockOnTarget`.

## Collision

### `CollisionSettings`

| Field | Type | Default | Valid Range | Behavior |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `true` or `false` | Enables or disables obstruction resolution |
| `strategy` | `CollisionStrategy` | `MultiRay` | `SingleRay`, `MultiRay`, `SphereProbe` | Chooses the sample pattern used to shorten the boom |
| `probe_radius` | `f32` | `0.28` | `>= 0.0` | Padding around obstacles and offset distance for probe samples |
| `sample_offset_x` | `f32` | `0.30` | `>= 0.0` | Horizontal offset used by `MultiRay` |
| `sample_offset_y` | `f32` | `0.22` | `>= 0.0` | Vertical offset used by `MultiRay` |
| `min_distance_from_target` | `f32` | `0.8` | `>= 0.0` | Hard minimum boom length after collision correction |
| `include_shape_radius` | `bool` | `true` | `true` or `false` | Expands obstacle padding and sample clearance by the configured probe radius |

### `CollisionStrategy`

| Variant | Meaning |
| --- | --- |
| `SingleRay` | One center ray from look anchor to desired camera position |
| `MultiRay` | Center ray plus four offset samples, good default for shoulder cameras |
| `SphereProbe` | Denser point set approximating a probe volume, useful for tight ceilings and corners |

### `ObstacleType`

| Variant | Meaning |
| --- | --- |
| `Blocker` | Uses full `probe_radius` clearance and behaves like solid blocking geometry |
| `Occluder` | Uses lighter clearance and is better for thin visual occluders |

## Auto Recenter

### `AutoRecenterSettings`

| Field | Type | Default | Valid Range | Behavior |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `false` | `true` or `false` | Enables idle recentering |
| `inactivity_seconds` | `f32` | `2.0` | `>= 0.0` | Manual-input quiet period before recentering begins |
| `follow_alignment` | `FollowAlignment` | `TargetForward` | `Free`, `TargetForward`, `MovementDirection` | Chooses the yaw reference used by recentering |

### `FollowAlignment`

| Variant | Meaning |
| --- | --- |
| `Free` | No automatic yaw recentering |
| `TargetForward` | Recenter behind the followed entity's forward vector |
| `MovementDirection` | Recenter behind recent target motion when available |

## Cursor

### `CursorPolicy`

| Field | Type | Default | Valid Range | Behavior |
| --- | --- | --- | --- | --- |
| `lock_by_default` | `bool` | `true` | `true` or `false` | Initial cursor-lock state for active input cameras |
| `allow_toggle` | `bool` | `true` | `true` or `false` | Allows `CursorLockAction` to flip the lock state |

## Lock-On Components

### `ThirdPersonCameraLockOn`

| Field | Type | Default | Valid Range | Behavior |
| --- | --- | --- | --- | --- |
| `active_target` | `Option<Entity>` | `None` | Live entity or `None` | Currently selected lock-on target |

### `ThirdPersonCameraLockOnTarget`

| Field | Type | Default | Valid Range | Behavior |
| --- | --- | --- | --- | --- |
| `offset` | `Vec3` | `Vec3::ZERO` | Any finite vector | Additional target-local offset used for the lock-on anchor |
| `priority` | `f32` | `0.0` | `>= 0.0` | Weight applied during candidate ranking and cycling |

## Target Descriptor

### `ThirdPersonCameraTarget`

| Field | Type | Default | Valid Range | Behavior |
| --- | --- | --- | --- | --- |
| `target` | `Entity` | none | Live entity | Follow target entity |
| `offset` | `Vec3` | `Vec3::ZERO` | Any finite vector | Additional target-local offset before look-anchor lift |
| `follow_rotation` | `bool` | `true` | `true` or `false` | Uses target forward as the recenter reference when `true`, movement direction when `false` |
| `enabled` | `bool` | `true` | `true` or `false` | Disables target sampling without despawning the component |
| `ignore_children` | `bool` | `true` | `true` or `false` | Excludes target descendants from obstruction checks |
| `ignored_entities` | `Vec<Entity>` | empty | Any entity list | Explicit per-target ignore set for collision and occlusion |
| `recenter_on_target_change` | `bool` | `true` | `true` or `false` | Snaps target yaw or pitch or distance back to home values after retargeting |

## Debug Surface

### `ThirdPersonCameraDebug`

| Field | Type | Default | Valid Range | Behavior |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `true` or `false` | Master toggle for debug drawing |
| `draw_pivot` | `bool` | `true` | `true` or `false` | Draws the pivot and look anchor |
| `draw_desired` | `bool` | `true` | `true` or `false` | Draws the unconstrained boom |
| `draw_corrected` | `bool` | `true` | `true` or `false` | Draws the corrected boom |
| `draw_hits` | `bool` | `true` | `true` or `false` | Draws obstruction hit markers and normals |

## Mode And Shoulder Enums

### `ThirdPersonCameraMode`

| Variant | Meaning |
| --- | --- |
| `Center` | Centered orbit or follow framing |
| `Shoulder` | Horizontal shoulder offset with normal distance |
| `Aim` | Shoulder framing plus aim-distance scaling and pitch offset |

### `ShoulderSide`

| Variant | Meaning |
| --- | --- |
| `Left` | Shoulder offset moves left from the target |
| `Right` | Shoulder offset moves right from the target |

## Recommended Presets

### Action Adventure

```rust
use saddle_camera_third_person_camera::{
    AutoRecenterSettings, FollowAlignment, FramingSettings, ThirdPersonCameraSettings,
};

let settings = ThirdPersonCameraSettings {
    framing: FramingSettings {
        shoulder_offset: 0.68,
        shoulder_height: 1.35,
        aim_height_offset: -0.35,
        aim_distance_scale: 0.7,
        ..default()
    },
    auto_recenter: AutoRecenterSettings {
        enabled: true,
        inactivity_seconds: 1.5,
        follow_alignment: FollowAlignment::TargetForward,
    },
    ..default()
};
```

### Shooter Shoulder Cam

```rust
use saddle_camera_third_person_camera::{FramingSettings, SmoothingSettings, ThirdPersonCameraSettings, ZoomSettings};

let settings = ThirdPersonCameraSettings {
    zoom: ZoomSettings {
        min_distance: 1.0,
        max_distance: 5.0,
        default_distance: 3.4,
        step: 0.55,
    },
    framing: FramingSettings {
        shoulder_offset: 0.95,
        aim_distance_scale: 0.48,
        aim_pitch_offset: 0.06,
        aim_height_offset: -0.40,
        ..default()
    },
    smoothing: SmoothingSettings {
        orientation_smoothing: 20.0,
        obstruction_pull_in: 34.0,
        obstruction_release: 12.0,
        ..default()
    },
    ..default()
};
```

### Platformer Orbit

```rust
use saddle_camera_third_person_camera::{AutoRecenterSettings, FollowAlignment, FramingSettings, ThirdPersonCameraSettings};

let settings = ThirdPersonCameraSettings {
    framing: FramingSettings {
        shoulder_offset: 0.0,
        aim_enabled: false,
        target_radius_clearance: 0.4,
        ..default()
    },
    auto_recenter: AutoRecenterSettings {
        enabled: true,
        inactivity_seconds: 0.75,
        follow_alignment: FollowAlignment::MovementDirection,
    },
    ..default()
};
```

### Vehicle Chase Cam

```rust
use saddle_camera_third_person_camera::{FramingSettings, SmoothingSettings, ThirdPersonCameraSettings, ZoomSettings};

let settings = ThirdPersonCameraSettings {
    zoom: ZoomSettings {
        min_distance: 3.0,
        max_distance: 14.0,
        default_distance: 7.5,
        step: 1.0,
    },
    framing: FramingSettings {
        shoulder_offset: 0.2,
        shoulder_height: 2.0,
        aim_enabled: false,
        target_radius_clearance: 1.2,
        ..default()
    },
    smoothing: SmoothingSettings {
        target_follow_smoothing: 9.0,
        orientation_smoothing: 11.0,
        zoom_smoothing: 9.0,
        ..default()
    },
    ..default()
};
```

## Tuning Advice

- Start by choosing the right `default_distance`, `shoulder_height`, and `target_radius_clearance`. Most bad third-person framing comes from those three values.
- `shoulder_height` is offset from the target entity's transform origin. For characters with transforms at their feet, 1.35 is a good starting point. For capsule-centered targets (origin at capsule midpoint), use a lower value like 0.55.
- If aim mode looks too high, make `aim_height_offset` more negative to pull the camera pivot down to true shoulder level.
- If the camera feels sticky, raise `orientation_smoothing` or `target_follow_smoothing`.
- If corner collisions feel twitchy, keep `MultiRay`, increase `probe_radius`, and lower `obstruction_release`.
- If aim mode feels too invasive, raise `aim_distance_scale` closer to `1.0` and reduce `aim_pitch_offset`.
- If large targets still fill the screen during pull-in, raise both `target_radius_clearance` and `min_distance_from_target`.
