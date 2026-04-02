# Architecture

## Rig Model

The crate uses a logical rig rather than a hierarchy of child entities:

1. target origin
2. look anchor
3. shoulder offset
4. desired camera point
5. corrected camera point

`ThirdPersonCameraTarget` resolves the followed entity and an optional target-local offset. The runtime then raises that position by `framing.shoulder_height + framing.target_radius_clearance + camera.large_target_radius` to produce the look anchor. Center mode keeps the look anchor centered. Shoulder and aim modes add a yaw-relative right offset before computing the camera boom.

The camera stores both current and target state:

- yaw and target yaw
- pitch and target pitch
- distance and target distance
- shoulder side and target shoulder side
- current effective mode and persistent target mode

That split keeps input, recentering, and smoothing explicit rather than hidden in transform math.

Temporary input overrides are layered on top of the persistent mode:

- `target_mode` stays authored or gameplay-driven
- `AimAction` temporarily drives the effective target mode to `Aim`
- `ShoulderHoldAction` temporarily drives the effective target mode to `Shoulder`
- releasing either input returns the camera to the persistent mode

## System Phases

The runtime is intentionally broken into explicit phases:

1. `ReadInput`
   Aggregates the active `bevy_enhanced_input` context into `ThirdPersonCameraInput`. Only the highest-order active camera with `ThirdPersonCameraInputTarget` receives shared input.
2. `UpdateIntent`
   Samples the follow target, updates yaw or pitch or distance intent, applies manual or idle recentering, smooths pivot or orientation or zoom, and updates shoulder or aim blends.
3. `ResolveObstruction`
   Computes the unconstrained pose, tests it against opt-in obstacles, and derives `obstruction_distance`, `corrected_distance`, hit point, and hit normal.
4. `ApplyTransform`
   Writes the final camera `Transform` from the corrected runtime pose, then clears transient input.
5. `DebugDraw`
   Draws pivot, desired boom, corrected boom, and hit state for cameras that carry `ThirdPersonCameraDebug`.

`ReadInput` runs in the plugin's injected update schedule. The remaining phases run in `PostUpdate` so the camera can follow targets that finish authoritative motion late in the frame.

## Ordering Guidance

Typical integrations should use these rules:

- If gameplay or physics updates the followed entity in `Update`, no extra ordering is needed.
- If target motion is finalized in `PostUpdate`, schedule that system before `ThirdPersonCameraSystems::UpdateIntent`.
- If an external system writes `ThirdPersonCameraInput` directly, do that after `ThirdPersonCameraSystems::ReadInput` or write to the active input camera entity.
- If another system needs the final camera transform in the same frame, order it after `ThirdPersonCameraSystems::ApplyTransform` and before `TransformSystems::Propagate` only when that system also works in `PostUpdate`.

The crate does not depend on any specific physics backend. The ordering seam is the public system-set surface, not a physics-specific API.

## Alignment And Recentering

The crate separates two yaw-reference decisions:

- `ThirdPersonCameraTarget::follow_rotation` controls the manual recenter and retarget reference
- `AutoRecenterSettings::follow_alignment` controls idle recentering after input inactivity

That split lets a game keep explicit recenter behavior tied to the tracked actor while still choosing whether passive recentering follows forward, follows motion heading, or stays fully free.

## Obstruction Strategy

The obstruction path separates desired pose from corrected pose:

- desired pose is computed from look anchor, yaw, pitch, shoulder blend, aim blend, and desired distance
- corrected pose is derived by shortening the boom when sample rays or probe offsets hit an obstacle

Supported strategies:

- `SingleRay`: center boom only, cheapest but most likely to leak near walls
- `MultiRay`: center plus four offset rays, default and the best general-purpose tradeoff
- `SphereProbe`: denser offset sample set, more stable around shoulders and ceilings but more expensive

Obstacle policy is opt-in:

- only entities with `ThirdPersonCameraObstacle` participate
- `ObstacleType::Blocker` keeps the camera farther away than `ObstacleType::Occluder`
- explicit ignore markers and target-owned exclusions prevent self-occlusion
- `include_shape_radius` toggles whether probe radius expands obstacle padding and sample clearance

When an obstacle is hit, the runtime uses `obstruction_pull_in` smoothing. When the path clears, it uses the slower `obstruction_release` smoothing to create spring-arm-style recovery.

## Smoothing Strategy

The crate uses exponential smoothing helpers so behavior is frame-rate independent. Different concerns use different rates:

- `target_follow_smoothing` for pivot motion
- `orientation_smoothing` for yaw and pitch
- `zoom_smoothing` for designer-facing distance changes
- `obstruction_pull_in` and `obstruction_release` for collision response
- `shoulder_blend` and `aim_blend` for framing transitions

The runtime exposes both desired and corrected positions in `ThirdPersonCameraRuntime` so shoulder jitter, obstruction oscillation, and recenter behavior can be inspected live.

## Cursor And Input Ownership

The crate owns its default BEI context and cursor-lock policy. Cursor lock is only applied to the active input camera. This avoids one disabled or background camera fighting the primary view.

For UI-heavy games, remove `ThirdPersonCameraInputTarget` from the active camera or disable the camera when a UI layer should own the pointer. The shared crate intentionally does not infer UI hover state.

## Runtime Debugging

`ThirdPersonCameraRuntime` is the main debugging surface:

- `pivot`, `look_target`, `desired_camera_position`, `corrected_camera_position`
- `desired_distance`, `obstruction_distance`, `corrected_distance`
- `obstruction_active`, `last_hit_point`, `last_hit_normal`, `last_collision_target`
- `shoulder_blend`, `aim_blend`, `cursor_locked`

The crate-local lab adds BRP and reproducible E2E scenarios so obstruction, shoulder parity, and retargeting issues can be inspected without involving project sandboxes.

## Performance Notes

Expected cost scales with:

- number of active camera rigs
- number of entities marked as `ThirdPersonCameraObstacle`
- collision sample count from the selected `CollisionStrategy`

Current tradeoffs:

- ignore-set scratch storage is reused per system so the hot path does not heap-allocate per frame for exclusion bookkeeping
- obstruction samples are emitted from fixed patterns instead of allocating temporary sample vectors each frame
- obstruction uses AABB tests rather than physics-engine casts, which keeps the crate backend-agnostic but limits geometric fidelity
- the default path is tuned for one active camera and a moderate obstacle set, which is the common gameplay case

For dense worlds or many simultaneous rigs:

- mark only relevant geometry with `ThirdPersonCameraObstacle`
- prefer `SingleRay` or `MultiRay` before `SphereProbe`
- disable cameras that are not currently presenting to the player
