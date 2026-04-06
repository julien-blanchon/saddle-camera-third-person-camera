# Architecture

## Design Split

The crate is organized as a generic camera core plus optional adapters.

### Core Rig

The core runtime owns:

1. follow target sampling
2. anchor height and screen-space framing
3. orbit yaw and pitch
4. zoom distance
5. desired vs corrected camera pose
6. collision pull-in and release smoothing

The core surface is intentionally small:

- `ThirdPersonCamera`
- `ThirdPersonCameraSettings`
- `ThirdPersonCameraTarget`
- `ThirdPersonCameraInput`
- `ThirdPersonCameraRuntime`

### Optional Adapters

Adapters layer archetype-specific behavior on top of the core rig:

- `ThirdPersonCameraShoulderRig` and `ThirdPersonCameraShoulderSettings`
  Adds lateral shoulder framing, aim distance scaling, and aim pitch or height offsets.
- `ThirdPersonCameraLockOn` and `ThirdPersonCameraLockOnSettings`
  Adds target acquisition, target cycling, and look-target bias toward a tracked entity.
- `ThirdPersonCameraCursorController`
  Adds cursor lock ownership for the active camera.
- `ThirdPersonCameraEnhancedInputPlugin`
  Bridges `bevy_enhanced_input` into the core input inbox plus the action adapter inbox.

This split keeps the public default path genre-neutral while preserving the older action-camera feature set as opt-in layers.

## Rig Model

The core rig uses a logical camera model rather than a transform child hierarchy:

1. target origin
2. target-local offset from `ThirdPersonCameraTarget`
3. anchor height from `ThirdPersonCameraSettings::anchor`
4. desired camera point
5. corrected camera point after obstruction

`ThirdPersonCameraTarget` resolves the followed entity and optional target-local offset. The runtime then raises that anchor by `anchor.height + anchor.radius_clearance + camera.large_target_radius`.

Shoulder or aim framing does not live in the core anchor anymore. When the optional shoulder adapter is present, it offsets the look target laterally and adjusts pitch or distance after the core target sample is resolved.

## System Phases

The runtime remains intentionally explicit:

1. `ReadInput`
   Reserved for input adapters. The optional BEI plugin writes to `ThirdPersonCameraInput` and `ThirdPersonCameraActionInput` here.
2. `UpdateIntent`
   Samples the follow target, applies core orbit or zoom or recenter input, updates optional lock-on state, smooths pivot or orientation or zoom, and resolves adapter state such as shoulder or aim blends.
3. `ResolveObstruction`
   Computes the unconstrained pose, tests it against opt-in obstacles, and derives `obstruction_distance`, `corrected_distance`, hit point, and hit normal.
4. `ComposeEffects`
   User-facing seam for custom effect systems. External systems update `ThirdPersonCameraCustomEffects` layers here, after obstruction resolution and before the final transform write.
5. `ApplyTransform`
   Writes the final `Transform`, composing any active custom effect layers (translation and rotation offsets in camera-local space), and clears transient input inboxes.
6. `DebugDraw`
   Draws pivot, desired boom, corrected boom, and hit state for cameras with `ThirdPersonCameraDebug`.

`UpdateIntent` and the later phases run in `PostUpdate` so the camera can follow targets that finish authoritative motion late in the frame.

## Screen-Space Framing

When `ScreenSpaceFramingSettings` is enabled, the core runtime evaluates the target against three regions:

1. dead zone: no camera response
2. soft zone: gentle correction
3. outside the soft zone: direct catch-up

That logic runs before obstruction resolution, so framing and collision share the same desired-vs-corrected pose pipeline.

## Obstruction Strategy

The obstruction path separates desired pose from corrected pose:

- desired pose comes from the anchor, orbit angles, optional shoulder offsets, and designer distance
- corrected pose shortens the boom when sample rays hit an obstacle

Supported strategies:

- `SingleRay`
- `MultiRay`
- `SphereProbe`

Obstacle participation is opt-in:

- only entities with `ThirdPersonCameraObstacle` participate
- `ObstacleType::Blocker` uses full probe padding
- `ObstacleType::Occluder` uses lighter padding
- target-owned exclusions and explicit ignore markers prevent self-occlusion

## Lock-On And Shoulder Adapters

The action adapters are stateful but isolated from the core rig.

### Shoulder Adapter

`ThirdPersonCameraShoulderRig` stores:

- current shoulder side and target shoulder side
- current framing mode and persistent target mode
- home shoulder side and home mode

`ThirdPersonCameraShoulderRuntime` stores the smoothed shoulder and aim blends that the core obstruction stage reads when composing the final look target.

### Lock-On Adapter

`ThirdPersonCameraLockOn` stores the currently requested target entity.

`ThirdPersonCameraLockOnRuntime` stores:

- the blended focus point
- current and target lock-on blend
- the effective active target

The adapter can steer target yaw and pitch without replacing the core orbit or collision logic.

## Cursor Ownership

Cursor lock is no longer a field inside `ThirdPersonCameraSettings`.

If a camera has `ThirdPersonCameraCursorController`, the runtime applies its `locked` state to the active camera with the highest `Camera.order`. Cameras without the cursor adapter never touch the window cursor.

## Ordering Guidance

- If gameplay or physics updates the followed entity in `Update`, no extra ordering is needed.
- If target motion is finalized in `PostUpdate`, schedule that system before `ThirdPersonCameraSystems::UpdateIntent`.
- If you write `ThirdPersonCameraInput` yourself, do it in `Update` or in `ThirdPersonCameraSystems::ReadInput`.
- If another system needs the final camera transform in the same frame, order it after `ThirdPersonCameraSystems::ApplyTransform`.

## Performance Notes

Expected cost scales with:

- number of active camera rigs
- number of entities marked as `ThirdPersonCameraObstacle`
- collision sample count from the selected `CollisionStrategy`
- optional adapter work such as lock-on target ranking

Current tradeoffs:

- ignore-set scratch storage is reused per system
- obstruction samples come from fixed patterns instead of heap-allocated per-frame vectors
- collision uses AABB tests to stay backend-agnostic
- the default path is tuned for one active camera and a moderate obstacle set
