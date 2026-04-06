# Configuration

This crate keeps generic camera tuning in explicit data structs and moves action-style behavior into optional adapter components.

## Core Camera Settings

### `ThirdPersonCameraSettings`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `enabled` | `bool` | `true` | Disables all runtime phases for the camera when `false` |
| `orbit` | `OrbitSettings` | `OrbitSettings::default()` | Orbit speed, pitch clamp, and inversion |
| `smoothing` | `SmoothingSettings` | `SmoothingSettings::default()` | Follow, orientation, zoom, and obstruction smoothing |
| `zoom` | `ZoomSettings` | `ZoomSettings::default()` | Designer zoom range and zoom step |
| `anchor` | `AnchorSettings` | `AnchorSettings::default()` | Vertical anchor lift and large-target clearance |
| `screen_framing` | `ScreenSpaceFramingSettings` | `ScreenSpaceFramingSettings::default()` | Screen-space dead-zone and soft-zone follow behavior |
| `collision` | `CollisionSettings` | `CollisionSettings::default()` | Obstruction strategy and minimum camera distance |
| `auto_recenter` | `AutoRecenterSettings` | `AutoRecenterSettings::default()` | Idle recentering policy |

### `ThirdPersonCamera`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `yaw` | `f32` | `0.0` | Current yaw in radians |
| `pitch` | `f32` | `-0.42` | Current pitch in radians |
| `distance` | `f32` | `4.6` | Current designer-facing boom length |
| `target_yaw` | `f32` | `0.0` | Smoothed yaw target |
| `target_pitch` | `f32` | `-0.42` | Smoothed pitch target |
| `target_distance` | `f32` | `4.6` | Smoothed distance target |
| `large_target_radius` | `f32` | `0.0` | Raises anchor clearance and the minimum allowed camera distance for large subjects |
| `home_yaw` / `home_pitch` / `home_distance` | `f32` | match defaults | Recenter destination for orientation and zoom |

### `ThirdPersonCameraInput`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `orbit_delta` | `Vec2` | `(0.0, 0.0)` | Orbit delta for this frame |
| `zoom_delta` | `f32` | `0.0` | Zoom delta for this frame |
| `recenter` | `bool` | `false` | Recenter toward the target reference and home values |

## Core Setting Types

### `AnchorSettings`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `height` | `f32` | `1.35` | Vertical lift from the sampled target anchor |
| `radius_clearance` | `f32` | `0.15` | Extra lift that helps large targets avoid near-clip face shots |

### `OrbitSettings`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `yaw_speed` | `f32` | `1.2` | Multiplier applied to orbit X input |
| `pitch_speed` | `f32` | `1.1` | Multiplier applied to orbit Y input |
| `min_pitch` | `f32` | `-1.25` | Lowest allowed target pitch in radians |
| `max_pitch` | `f32` | `-0.08` | Highest allowed target pitch in radians |
| `invert_x` | `bool` | `false` | Inverts yaw input |
| `invert_y` | `bool` | `false` | Inverts pitch input |

### `SmoothingSettings`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `orientation_smoothing` | `f32` | `16.0` | Yaw and pitch convergence rate |
| `target_follow_smoothing` | `f32` | `18.0` | Pivot follow smoothing rate |
| `zoom_smoothing` | `f32` | `16.0` | Distance convergence rate after zoom input |
| `obstruction_pull_in` | `f32` | `28.0` | Speed for shortening the boom when an obstacle appears |
| `obstruction_release` | `f32` | `10.0` | Speed for restoring boom length after an obstacle clears |

### `ZoomSettings`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `min_distance` | `f32` | `1.15` | Smallest allowed designer zoom distance |
| `max_distance` | `f32` | `10.0` | Largest allowed designer zoom distance |
| `default_distance` | `f32` | `4.6` | Initial and home distance for the default camera |
| `step` | `f32` | `0.8` | Scalar applied to zoom input |

### `ScreenSpaceFramingSettings`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `enabled` | `bool` | `false` | Enables screen-space framing instead of pure world-space follow |
| `dead_zone` | `Vec2` | `(0.18, 0.14)` | Region where the target can move without camera response |
| `soft_zone` | `Vec2` | `(0.42, 0.32)` | Region where the camera eases back before hard follow |
| `screen_offset` | `Vec2` | `(0.0, 0.0)` | Biases the target anchor away from exact screen center |

### `CollisionSettings`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `enabled` | `bool` | `true` | Enables or disables obstruction resolution |
| `strategy` | `CollisionStrategy` | `MultiRay` | Sample pattern used to shorten the boom |
| `probe_radius` | `f32` | `0.28` | Padding around obstacles and offset distance for probe samples |
| `sample_offset_x` | `f32` | `0.30` | Horizontal offset used by `MultiRay` |
| `sample_offset_y` | `f32` | `0.22` | Vertical offset used by `MultiRay` |
| `min_distance_from_target` | `f32` | `0.8` | Hard minimum boom length after collision correction |
| `include_shape_radius` | `bool` | `true` | Expands obstacle padding and sample clearance by `probe_radius` |

### `AutoRecenterSettings`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `enabled` | `bool` | `false` | Enables idle recentering |
| `inactivity_seconds` | `f32` | `2.0` | Manual-input quiet period before recentering begins |
| `follow_alignment` | `FollowAlignment` | `TargetForward` | Chooses the yaw reference used by recentering |

## Optional Adapters

### `ThirdPersonCameraShoulderSettings`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `shoulder_offset` | `f32` | `0.75` | Horizontal offset applied when the shoulder rig is offset |
| `default_side` | `ShoulderSide` | `Right` | Preferred shoulder side for default rigs |
| `aim_enabled` | `bool` | `true` | Allows `Aim` mode and aim-distance scaling |
| `aim_distance_scale` | `f32` | `0.62` | Multiplies camera distance while in aim mode |
| `aim_pitch_offset` | `f32` | `0.10` | Additional pitch applied in aim mode |
| `aim_height_offset` | `f32` | `-0.35` | Vertical offset applied to the look target in aim mode |
| `shoulder_blend_smoothing` | `f32` | `14.0` | Shoulder-side transition rate |
| `aim_blend_smoothing` | `f32` | `20.0` | Center-to-aim transition rate |

### `ThirdPersonCameraShoulderRig`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `shoulder_side` / `target_shoulder_side` | `ShoulderSide` | `Right` | Current and target lateral side |
| `mode` / `target_mode` | `ThirdPersonCameraMode` | `Center` | Current effective mode and persistent target mode |
| `home_shoulder_side` / `home_mode` | enum | match defaults | Home values for shoulder-side or mode resets |

### `ThirdPersonCameraShoulderRuntime`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `shoulder_blend` / `target_shoulder_blend` | `f32` | `0.0` | Current and target shoulder blend |
| `aim_blend` / `target_aim_blend` | `f32` | `0.0` | Current and target aim blend |

### `ThirdPersonCameraLockOnSettings`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `enabled` | `bool` | `false` | Enables lock-on selection and runtime facing behavior |
| `max_distance` | `f32` | `24.0` | Maximum distance from the pivot to candidate targets |
| `focus_bias` | `f32` | `0.35` | Biases the look target blend between the pivot and the selected target |
| `pitch_offset` | `f32` | `0.08` | Pitch offset applied while lock-on is active |
| `blend_smoothing` | `f32` | `20.0` | Lock-on focus blend response rate |

### `ThirdPersonCameraLockOn`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `active_target` | `Option<Entity>` | `None` | Requested lock-on target |

### `ThirdPersonCameraLockOnRuntime`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `focus` | `Vec3` | `(0.0, 0.0, 0.0)` | Blended focus point |
| `blend` / `target_blend` | `f32` | `0.0` | Current and target lock-on blend |
| `active_target` | `Option<Entity>` | `None` | Effective target after validation |

### `ThirdPersonCameraLockOnTarget`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `offset` | `Vec3` | `(0.0, 0.0, 0.0)` | Additional target-local offset for the lock-on anchor |
| `priority` | `f32` | `0.0` | Weight used during candidate ranking and cycling |

### `ThirdPersonCameraCursorController`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `lock_by_default` | `bool` | `true` | Initial cursor-lock state for the active camera |
| `allow_toggle` | `bool` | `true` | Allows runtime toggling |
| `locked` | `bool` | `true` | Current cursor-lock state |

## Target Descriptor

### `ThirdPersonCameraTarget`

| Field | Type | Default | Behavior |
| --- | --- | --- | --- |
| `target` | `Entity` | required | Follow target entity |
| `offset` | `Vec3` | `(0.0, 0.0, 0.0)` | Additional target-local offset before anchor height is applied |
| `follow_rotation` | `bool` | `true` | Uses target forward as the manual recenter reference when `true` |
| `enabled` | `bool` | `true` | Disables target sampling without despawning the component |
| `ignore_children` | `bool` | `true` | Excludes target descendants from obstruction checks |
| `ignored_entities` | `Vec<Entity>` | empty | Explicit per-target ignore set |
| `recenter_on_target_change` | `bool` | `true` | Recenter yaw and distance after retargeting |

## Tuning Advice

- Start with `anchor.height`, `default_distance`, and `min_distance_from_target`.
- Use `ThirdPersonCameraTarget::offset` for per-target anchor shifts, then use `anchor.height` for the shared rig lift.
- If the camera feels sticky, raise `orientation_smoothing` or `target_follow_smoothing`.
- If corners feel twitchy, keep `MultiRay`, increase `probe_radius`, and lower `obstruction_release`.
- Only add the shoulder or lock-on adapters when your game actually wants that interaction model.
