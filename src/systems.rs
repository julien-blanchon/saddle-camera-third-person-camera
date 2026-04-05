use std::collections::HashSet;

use bevy::{
    camera::{Projection, primitives::Aabb},
    prelude::*,
    transform::helper::TransformHelper,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::{
    CollisionStrategy, ShoulderSide, ThirdPersonCamera, ThirdPersonCameraIgnore,
    ThirdPersonCameraIgnoreTarget, ThirdPersonCameraInput, ThirdPersonCameraInputTarget,
    ThirdPersonCameraLockOn, ThirdPersonCameraLockOnTarget, ThirdPersonCameraMode,
    ThirdPersonCameraObstacle, ThirdPersonCameraRuntime, ThirdPersonCameraSettings,
    ThirdPersonCameraTarget,
    math::{
        CameraPose, camera_pose_from_look_target, forward_from_angles, segment_aabb_hit,
        smooth_angle, smooth_scalar, smooth_vec3, wrap_angle, yaw_from_direction,
        yaw_pitch_rotation,
    },
};

#[derive(Resource, Default, Clone, Copy)]
pub(crate) struct ActiveInputCamera(pub Option<Entity>);

pub(crate) fn initialize_added_cameras(
    mut cameras: Query<
        (
            &mut ThirdPersonCamera,
            &ThirdPersonCameraSettings,
            &mut ThirdPersonCameraRuntime,
            &mut Transform,
        ),
        Added<ThirdPersonCamera>,
    >,
) {
    for (mut camera, settings, mut runtime, mut transform) in &mut cameras {
        if settings.framing.default_side == ShoulderSide::Left
            && camera.shoulder_side == ShoulderSide::Right
            && camera.target_shoulder_side == ShoulderSide::Right
            && camera.home_shoulder_side == ShoulderSide::Right
        {
            camera.shoulder_side = ShoulderSide::Left;
            camera.target_shoulder_side = ShoulderSide::Left;
            camera.home_shoulder_side = ShoulderSide::Left;
        }

        let distance = clamp_camera_distance(camera.distance, settings, &camera);
        camera.distance = distance;
        camera.target_distance = clamp_camera_distance(camera.target_distance, settings, &camera);
        camera.home_distance = clamp_camera_distance(camera.home_distance, settings, &camera);
        camera.pitch = settings.clamped_pitch(camera.pitch);
        camera.target_pitch = settings.clamped_pitch(camera.target_pitch);
        camera.home_pitch = settings.clamped_pitch(camera.home_pitch);

        runtime.desired_distance = distance;
        runtime.corrected_distance = distance;
        runtime.obstruction_distance = distance;
        runtime.target_shoulder_blend =
            shoulder_blend_target(camera.target_mode, camera.target_shoulder_side);
        runtime.shoulder_blend = runtime.target_shoulder_blend;
        runtime.target_aim_blend = aim_blend_target(camera.target_mode, settings);
        runtime.aim_blend = runtime.target_aim_blend;
        runtime.cursor_locked = settings.cursor.lock_by_default;
        *transform = Transform::from_xyz(0.0, settings.framing.shoulder_height, distance);
    }
}

pub(crate) fn route_active_input(
    mut active_input_camera: ResMut<ActiveInputCamera>,
    mut cameras: ParamSet<(
        Query<(Entity, &Camera, &ThirdPersonCameraSettings), With<ThirdPersonCameraInputTarget>>,
        Query<(Entity, &mut ThirdPersonCameraInput), With<ThirdPersonCamera>>,
    )>,
) {
    let active = cameras
        .p0()
        .iter()
        .filter(|(_, camera_component, settings)| camera_component.is_active && settings.enabled)
        .max_by_key(|(entity, camera_component, _)| (camera_component.order, entity.to_bits()))
        .map(|(entity, _, _)| entity);
    active_input_camera.0 = active;

    if let Some(active) = active {
        for (entity, mut input) in &mut cameras.p1() {
            if entity != active {
                input.clear_transient();
            }
        }
    }
}

pub(crate) fn update_lock_on_selection(
    helper: TransformHelper,
    candidates: Query<(
        Entity,
        &ThirdPersonCameraLockOnTarget,
        Option<&GlobalTransform>,
    )>,
    mut cameras: Query<(
        &ThirdPersonCamera,
        &ThirdPersonCameraSettings,
        Option<&ThirdPersonCameraTarget>,
        &ThirdPersonCameraInput,
        &mut ThirdPersonCameraLockOn,
        &ThirdPersonCameraRuntime,
    )>,
) {
    for (camera, settings, target, input, mut lock_on, runtime) in &mut cameras {
        if !settings.lock_on.enabled {
            lock_on.active_target = None;
            continue;
        }

        let Some(target) = target else {
            lock_on.active_target = None;
            continue;
        };
        if !target.enabled {
            lock_on.active_target = None;
            continue;
        }

        let Some(follow_anchor) = follow_target_anchor(
            &helper,
            target,
            camera,
            settings,
            runtime.last_target_position,
        ) else {
            lock_on.active_target = None;
            continue;
        };
        let origin = if runtime.pivot == Vec3::ZERO {
            follow_anchor
        } else {
            runtime.pivot
        };

        if !active_lock_on_target_is_valid(
            &helper,
            &candidates,
            lock_on.active_target,
            target.target,
            origin,
            settings.lock_on.max_distance,
        ) {
            lock_on.active_target = None;
        }

        if input.lock_on_toggle {
            lock_on.active_target = if lock_on.active_target.is_some() {
                None
            } else {
                select_best_lock_on_candidate(
                    &helper,
                    &candidates,
                    origin,
                    camera.target_yaw,
                    settings.lock_on.max_distance,
                    target.target,
                )
            };
        } else if input.lock_on_next {
            lock_on.active_target = cycle_lock_on_candidate(
                &helper,
                &candidates,
                origin,
                camera.target_yaw,
                lock_on.active_target,
                settings.lock_on.max_distance,
                target.target,
                true,
            );
        } else if input.lock_on_previous {
            lock_on.active_target = cycle_lock_on_candidate(
                &helper,
                &candidates,
                origin,
                camera.target_yaw,
                lock_on.active_target,
                settings.lock_on.max_distance,
                target.target,
                false,
            );
        }
    }
}

pub(crate) fn update_camera_runtime(
    time: Res<Time>,
    active_input_camera: Res<ActiveInputCamera>,
    helper: TransformHelper,
    children: Query<&Children>,
    ignored: Query<Entity, With<ThirdPersonCameraIgnore>>,
    ignored_targets: Query<Entity, With<ThirdPersonCameraIgnoreTarget>>,
    lock_on_targets: Query<(&ThirdPersonCameraLockOnTarget, Option<&GlobalTransform>)>,
    mut ignore_scratch: Local<HashSet<Entity>>,
    mut primary_window: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut cameras: Query<(
        Entity,
        &Projection,
        &mut ThirdPersonCamera,
        &ThirdPersonCameraSettings,
        Option<Ref<'_, ThirdPersonCameraTarget>>,
        &mut ThirdPersonCameraLockOn,
        &mut ThirdPersonCameraRuntime,
        &ThirdPersonCameraInput,
    )>,
) {
    let dt = time.delta_secs();
    for (
        camera_entity,
        projection,
        mut camera,
        settings,
        target,
        mut lock_on,
        mut runtime,
        input,
    ) in &mut cameras
    {
        if !settings.enabled {
            continue;
        }

        ignore_scratch.clear();
        ignore_scratch.insert(camera_entity);
        for entity in &ignored {
            ignore_scratch.insert(entity);
        }
        for entity in &ignored_targets {
            ignore_scratch.insert(entity);
        }

        let sampled = sample_target(
            &helper,
            &children,
            &camera,
            target.as_deref(),
            &mut ignore_scratch,
            settings,
            &runtime,
        );
        let Some(sampled) = sampled else {
            continue;
        };

        let target_changed = target.as_ref().is_some_and(|target| target.is_changed());
        let desired_pivot = apply_screen_space_framing(
            projection,
            &camera,
            settings,
            runtime.pivot,
            sampled.look_anchor,
        );
        runtime.target_pivot = desired_pivot;
        if runtime.pivot == Vec3::ZERO || target_changed {
            runtime.pivot = desired_pivot;
        }
        runtime.last_target_position = sampled.target_position;

        apply_input_to_camera(
            &mut camera,
            settings,
            &mut runtime,
            input,
            sampled.manual_reference_yaw,
            target_changed
                && target
                    .as_ref()
                    .is_some_and(|value| value.recenter_on_target_change),
        );

        let desired_mode = effective_target_mode(
            camera.target_mode,
            input,
            settings,
            lock_on.active_target.is_some(),
        );

        runtime.idle_seconds = if input.has_manual_motion() {
            0.0
        } else {
            runtime.idle_seconds + dt
        };
        runtime.manual_input_this_frame = input.has_manual_motion();

        if settings.auto_recenter.enabled
            && !runtime.manual_input_this_frame
            && runtime.idle_seconds >= settings.auto_recenter.inactivity_seconds
        {
            if let Some(reference_yaw) = sampled.idle_reference_yaw {
                camera.target_yaw = reference_yaw;
            }
        }

        runtime.pivot = smooth_vec3(
            runtime.pivot,
            runtime.target_pivot,
            settings.smoothing.target_follow_smoothing,
            dt,
        );

        if settings.lock_on.enabled {
            if let Some(lock_target) = lock_on.active_target {
                let valid_anchor = lock_on_target_anchor(&helper, &lock_on_targets, lock_target)
                    .filter(|anchor| {
                        runtime.pivot.distance(*anchor) <= settings.lock_on.max_distance
                    });
                if let Some(anchor) = valid_anchor {
                    runtime.lock_on_focus = runtime
                        .pivot
                        .lerp(anchor, settings.lock_on.focus_bias.clamp(0.0, 1.0));
                    runtime.target_lock_on_blend = 1.0;
                    runtime.active_lock_on_target = Some(lock_target);
                    if let Some(yaw) = yaw_from_direction(anchor - runtime.pivot) {
                        camera.target_yaw = yaw;
                    }
                    camera.target_pitch = settings
                        .clamped_pitch(camera.home_pitch - settings.lock_on.pitch_offset.abs());
                } else {
                    lock_on.active_target = None;
                    runtime.target_lock_on_blend = 0.0;
                    runtime.active_lock_on_target = None;
                }
            } else {
                runtime.target_lock_on_blend = 0.0;
                runtime.active_lock_on_target = None;
            }
        } else {
            lock_on.active_target = None;
            runtime.target_lock_on_blend = 0.0;
            runtime.active_lock_on_target = None;
        }

        camera.target_pitch = settings.clamped_pitch(camera.target_pitch);
        camera.target_distance = clamp_camera_distance(camera.target_distance, settings, &camera);
        camera.yaw = smooth_angle(
            camera.yaw,
            camera.target_yaw,
            settings.smoothing.orientation_smoothing,
            dt,
        );
        camera.pitch = smooth_angle(
            camera.pitch,
            camera.target_pitch,
            settings.smoothing.orientation_smoothing,
            dt,
        );
        camera.distance = smooth_scalar(
            camera.distance,
            camera.target_distance,
            settings.smoothing.zoom_smoothing,
            dt,
        );

        runtime.target_shoulder_blend =
            shoulder_blend_target(desired_mode, camera.target_shoulder_side);
        runtime.shoulder_blend = smooth_scalar(
            runtime.shoulder_blend,
            runtime.target_shoulder_blend,
            settings.smoothing.shoulder_blend,
            dt,
        );
        runtime.target_aim_blend = aim_blend_target(desired_mode, settings);
        runtime.aim_blend = smooth_scalar(
            runtime.aim_blend,
            runtime.target_aim_blend,
            settings.smoothing.aim_blend,
            dt,
        );
        runtime.lock_on_blend = smooth_scalar(
            runtime.lock_on_blend,
            runtime.target_lock_on_blend,
            settings.smoothing.aim_blend,
            dt,
        );
        camera.shoulder_side = if runtime.shoulder_blend < 0.0 {
            ShoulderSide::Left
        } else {
            ShoulderSide::Right
        };
        camera.mode = current_mode(runtime.aim_blend, runtime.shoulder_blend);

        if Some(camera_entity) == active_input_camera.0
            && input.cursor_lock_toggle
            && settings.cursor.allow_toggle
        {
            runtime.cursor_locked = !runtime.cursor_locked;
        }
        if Some(camera_entity) == active_input_camera.0 {
            apply_cursor_lock(runtime.cursor_locked, &mut primary_window);
        }
    }
}

pub(crate) fn resolve_obstruction(
    time: Res<Time>,
    obstacles: Query<(
        Entity,
        &ThirdPersonCameraObstacle,
        Option<&Aabb>,
        &GlobalTransform,
    )>,
    mut cameras: Query<(
        Entity,
        &ThirdPersonCamera,
        &ThirdPersonCameraSettings,
        Option<&ThirdPersonCameraTarget>,
        &mut ThirdPersonCameraRuntime,
    )>,
    children: Query<&Children>,
    ignored: Query<Entity, With<ThirdPersonCameraIgnore>>,
    ignored_targets: Query<Entity, With<ThirdPersonCameraIgnoreTarget>>,
    mut ignore_scratch: Local<HashSet<Entity>>,
) {
    let dt = time.delta_secs();
    for (camera_entity, camera, settings, target, mut runtime) in &mut cameras {
        if !settings.enabled {
            continue;
        }

        let base_look_target = runtime.pivot
            + right_offset(
                camera.yaw,
                runtime.shoulder_blend,
                settings.framing.shoulder_offset,
            )
            + Vec3::Y * (settings.framing.aim_height_offset * runtime.aim_blend);
        let look_target = base_look_target.lerp(runtime.lock_on_focus, runtime.lock_on_blend);
        let aim_pitch = camera.pitch + settings.framing.aim_pitch_offset * runtime.aim_blend;
        let aim_scale = 1.0 - (1.0 - settings.framing.aim_distance_scale) * runtime.aim_blend;
        let minimum_distance =
            minimum_camera_distance(settings, camera).max(settings.zoom.min_distance);
        let desired_distance = (camera.distance * aim_scale)
            .max(minimum_distance)
            .min(settings.zoom.max_distance.max(minimum_distance));
        let pose =
            camera_pose_from_look_target(look_target, camera.yaw, aim_pitch, desired_distance);
        runtime.look_target = pose.look_target;
        runtime.desired_distance = pose.desired_distance;
        runtime.desired_camera_position = pose.desired_camera_position;

        ignore_scratch.clear();
        ignore_scratch.insert(camera_entity);
        for entity in &ignored {
            ignore_scratch.insert(entity);
        }
        for entity in &ignored_targets {
            ignore_scratch.insert(entity);
        }
        if let Some(target) = target {
            ignore_scratch.insert(target.target);
            for entity in &target.ignored_entities {
                ignore_scratch.insert(*entity);
            }
            if target.ignore_children {
                collect_descendants(target.target, &children, &mut ignore_scratch);
            }
        }

        let obstruction = if settings.collision.enabled {
            find_obstruction(&pose, settings, &ignore_scratch, &obstacles)
        } else {
            None
        };

        let obstruction_distance = obstruction
            .map(|hit| hit.distance)
            .unwrap_or(desired_distance)
            .clamp(minimum_distance, desired_distance);
        runtime.obstruction_distance = obstruction_distance;
        runtime.obstruction_active = obstruction.is_some();
        runtime.last_hit_point = obstruction.and_then(|hit| hit.point);
        runtime.last_hit_normal = obstruction.map(|hit| hit.normal).unwrap_or(Vec3::ZERO);
        runtime.last_collision_target = obstruction.map(|hit| hit.entity);

        let smooth_rate = if obstruction_distance < runtime.corrected_distance {
            settings.smoothing.obstruction_pull_in
        } else {
            settings.smoothing.obstruction_release
        };
        runtime.corrected_distance = smooth_scalar(
            runtime.corrected_distance,
            obstruction_distance,
            smooth_rate,
            dt,
        );
        runtime.corrected_camera_position = pose.look_target
            - (forward_from_angles(camera.yaw, aim_pitch) * runtime.corrected_distance);
    }
}

pub(crate) fn apply_camera_transform(
    mut cameras: Query<(&ThirdPersonCameraRuntime, &mut Transform), With<ThirdPersonCamera>>,
) {
    for (runtime, mut transform) in &mut cameras {
        *transform = Transform::from_translation(runtime.corrected_camera_position)
            .looking_at(runtime.look_target, Vec3::Y);
    }
}

pub(crate) fn clear_consumed_input(
    mut cameras: Query<&mut ThirdPersonCameraInput, With<ThirdPersonCamera>>,
) {
    for mut input in &mut cameras {
        input.clear_transient();
    }
}

#[derive(Clone, Copy, Debug)]
struct SampledTarget {
    target_position: Vec3,
    look_anchor: Vec3,
    manual_reference_yaw: Option<f32>,
    idle_reference_yaw: Option<f32>,
}

fn sample_target(
    helper: &TransformHelper,
    children: &Query<&Children>,
    camera: &ThirdPersonCamera,
    target: Option<&ThirdPersonCameraTarget>,
    ignore_set: &mut HashSet<Entity>,
    settings: &ThirdPersonCameraSettings,
    runtime: &ThirdPersonCameraRuntime,
) -> Option<SampledTarget> {
    let target = target?;
    if !target.enabled {
        return None;
    }
    ignore_set.insert(target.target);
    ignore_set.extend(target.ignored_entities.iter().copied());
    if target.ignore_children {
        collect_descendants(target.target, children, ignore_set);
    }

    let global = helper.compute_global_transform(target.target).ok()?;
    let (_, rotation, translation) = global.to_scale_rotation_translation();
    let target_position = translation + rotation * target.offset;
    let look_anchor = target_position
        + Vec3::Y
            * (settings.framing.shoulder_height
                + settings.framing.target_radius_clearance
                + camera.large_target_radius);
    let movement = target_position - runtime.last_target_position;
    let movement_yaw = yaw_from_direction(movement);
    let forward_yaw = yaw_from_direction(*global.forward());
    let forward_reference_yaw = forward_yaw.or(movement_yaw);
    let movement_reference_yaw = movement_yaw.or(forward_yaw);
    let manual_reference_yaw = if target.follow_rotation {
        forward_reference_yaw
    } else {
        movement_reference_yaw
    };
    let idle_reference_yaw = match settings.auto_recenter.follow_alignment {
        crate::FollowAlignment::Free => None,
        crate::FollowAlignment::TargetForward => forward_reference_yaw,
        crate::FollowAlignment::MovementDirection => movement_reference_yaw,
    };
    Some(SampledTarget {
        target_position,
        look_anchor,
        manual_reference_yaw,
        idle_reference_yaw,
    })
}

fn apply_input_to_camera(
    camera: &mut ThirdPersonCamera,
    settings: &ThirdPersonCameraSettings,
    runtime: &mut ThirdPersonCameraRuntime,
    input: &ThirdPersonCameraInput,
    reference_yaw: Option<f32>,
    recenter_on_retarget: bool,
) {
    let orbit_x = if settings.orbit.invert_x { -1.0 } else { 1.0 };
    let orbit_y = if settings.orbit.invert_y { -1.0 } else { 1.0 };

    if input.orbit_delta.length_squared() > 0.0 {
        camera.target_yaw = wrap_angle(
            camera.target_yaw - input.orbit_delta.x * settings.orbit.yaw_speed * orbit_x,
        );
        camera.target_pitch = settings.clamped_pitch(
            camera.target_pitch - input.orbit_delta.y * settings.orbit.pitch_speed * orbit_y,
        );
    }

    if input.zoom_delta.abs() > f32::EPSILON {
        camera.target_distance = clamp_camera_distance(
            camera.target_distance - input.zoom_delta * settings.zoom.step,
            settings,
            camera,
        );
    }

    if input.raw_mode_center {
        camera.target_mode = ThirdPersonCameraMode::Center;
    }
    if input.raw_mode_shoulder {
        camera.target_mode = ThirdPersonCameraMode::Shoulder;
    }
    if input.shoulder_toggle {
        camera.target_shoulder_side = camera.target_shoulder_side.opposite();
        if camera.target_mode == ThirdPersonCameraMode::Center {
            camera.target_mode = ThirdPersonCameraMode::Shoulder;
        }
    }

    if recenter_on_retarget || input.recenter {
        if let Some(reference_yaw) = reference_yaw {
            camera.target_yaw = reference_yaw;
        }
        camera.target_pitch = camera.home_pitch;
        camera.target_distance = camera.home_distance;
    }

    runtime.manual_input_this_frame = input.has_manual_motion();
}

fn effective_target_mode(
    persistent_mode: ThirdPersonCameraMode,
    input: &ThirdPersonCameraInput,
    settings: &ThirdPersonCameraSettings,
    lock_on_active: bool,
) -> ThirdPersonCameraMode {
    if settings.framing.aim_enabled && (input.aim || lock_on_active) {
        ThirdPersonCameraMode::Aim
    } else if input.shoulder_hold {
        ThirdPersonCameraMode::Shoulder
    } else {
        persistent_mode
    }
}

fn current_mode(aim_blend: f32, shoulder_blend: f32) -> ThirdPersonCameraMode {
    if aim_blend > 0.5 {
        ThirdPersonCameraMode::Aim
    } else if shoulder_blend.abs() > 0.5 {
        ThirdPersonCameraMode::Shoulder
    } else {
        ThirdPersonCameraMode::Center
    }
}

fn shoulder_blend_target(mode: ThirdPersonCameraMode, side: ShoulderSide) -> f32 {
    match mode {
        ThirdPersonCameraMode::Center => 0.0,
        ThirdPersonCameraMode::Shoulder | ThirdPersonCameraMode::Aim => side.sign(),
    }
}

fn aim_blend_target(mode: ThirdPersonCameraMode, settings: &ThirdPersonCameraSettings) -> f32 {
    if settings.framing.aim_enabled && mode == ThirdPersonCameraMode::Aim {
        1.0
    } else {
        0.0
    }
}

fn apply_cursor_lock(
    cursor_locked: bool,
    primary_window: &mut Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cursor) = primary_window.single_mut() else {
        return;
    };
    cursor.visible = !cursor_locked;
    cursor.grab_mode = if cursor_locked {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
}

fn apply_screen_space_framing(
    projection: &Projection,
    camera: &ThirdPersonCamera,
    settings: &ThirdPersonCameraSettings,
    current_pivot: Vec3,
    look_anchor: Vec3,
) -> Vec3 {
    if !settings.screen_framing.enabled {
        return look_anchor;
    }

    let reference_pivot = if current_pivot == Vec3::ZERO {
        look_anchor
    } else {
        current_pivot
    };
    let orientation = yaw_pitch_rotation(camera.yaw, camera.pitch);
    let right = orientation * Vec3::X;
    let up = orientation * Vec3::Y;
    let forward = orientation * -Vec3::Z;
    let distance = camera.distance.max(settings.zoom.min_distance).max(0.1);
    let (half_width, half_height) = screen_half_extents(projection, distance);
    let desired_offset = Vec2::new(
        settings.screen_framing.screen_offset.x * half_width,
        settings.screen_framing.screen_offset.y * half_height,
    );
    let anchor = look_anchor - right * desired_offset.x - up * desired_offset.y;
    let delta = anchor - reference_pivot;
    let local = Vec3::new(delta.dot(right), delta.dot(up), delta.dot(forward));

    let dead_zone = settings
        .screen_framing
        .dead_zone
        .max(Vec2::ZERO)
        .min(Vec2::splat(0.95));
    let soft_zone = settings
        .screen_framing
        .soft_zone
        .max(dead_zone)
        .min(Vec2::splat(0.99));
    let dead_world = Vec2::new(half_width * dead_zone.x, half_height * dead_zone.y);
    let soft_world = Vec2::new(half_width * soft_zone.x, half_height * soft_zone.y);
    let correction_x = soft_zone_correction(local.x, dead_world.x, soft_world.x);
    let correction_y = soft_zone_correction(local.y, dead_world.y, soft_world.y);

    reference_pivot + right * correction_x + up * correction_y + forward * local.z
}

fn right_offset(yaw: f32, shoulder_blend: f32, offset: f32) -> Vec3 {
    Quat::from_rotation_y(yaw) * Vec3::X * (offset * shoulder_blend)
}

fn screen_half_extents(projection: &Projection, distance: f32) -> (f32, f32) {
    match projection {
        Projection::Perspective(perspective) => {
            let half_height = (perspective.fov * 0.5).tan() * distance;
            let half_width = half_height * perspective.aspect_ratio.max(0.1);
            (half_width, half_height)
        }
        Projection::Orthographic(orthographic) => (
            (orthographic.area.max.x - orthographic.area.min.x).abs() * 0.5,
            (orthographic.area.max.y - orthographic.area.min.y).abs() * 0.5,
        ),
        Projection::Custom(_) => {
            let half_height = distance * 0.5;
            (half_height, half_height)
        }
    }
}

fn soft_zone_correction(value: f32, dead_zone: f32, soft_zone: f32) -> f32 {
    let magnitude = value.abs();
    let sign = value.signum();
    if magnitude <= dead_zone {
        0.0
    } else if soft_zone <= dead_zone + 0.000_1 {
        sign * (magnitude - dead_zone)
    } else if magnitude <= soft_zone {
        let beyond_dead = magnitude - dead_zone;
        let span = (soft_zone - dead_zone).max(0.000_1);
        let softness = beyond_dead / span;
        sign * beyond_dead * softness
    } else {
        sign * (magnitude - soft_zone)
    }
}

fn collect_descendants(
    entity: Entity,
    children: &Query<&Children>,
    ignore_set: &mut HashSet<Entity>,
) {
    if let Ok(descendants) = children.get(entity) {
        for child in descendants.iter() {
            ignore_set.insert(child);
            collect_descendants(child, children, ignore_set);
        }
    }
}

fn follow_target_anchor(
    helper: &TransformHelper,
    target: &ThirdPersonCameraTarget,
    camera: &ThirdPersonCamera,
    settings: &ThirdPersonCameraSettings,
    fallback_target_position: Vec3,
) -> Option<Vec3> {
    let global = helper.compute_global_transform(target.target).ok();
    let (rotation, target_position) = if let Some(global) = global {
        let (_, rotation, translation) = global.to_scale_rotation_translation();
        (rotation, translation + rotation * target.offset)
    } else {
        (Quat::IDENTITY, fallback_target_position)
    };
    Some(
        target_position
            + rotation * Vec3::ZERO
            + Vec3::Y
                * (settings.framing.shoulder_height
                    + settings.framing.target_radius_clearance
                    + camera.large_target_radius),
    )
}

fn lock_on_target_anchor(
    helper: &TransformHelper,
    candidates: &Query<(&ThirdPersonCameraLockOnTarget, Option<&GlobalTransform>)>,
    entity: Entity,
) -> Option<Vec3> {
    let (candidate, global_transform) = candidates.get(entity).ok()?;
    let global = helper
        .compute_global_transform(entity)
        .ok()
        .or_else(|| global_transform.copied())?;
    let (_, rotation, translation) = global.to_scale_rotation_translation();
    Some(translation + rotation * candidate.offset)
}

fn active_lock_on_target_is_valid(
    helper: &TransformHelper,
    candidates: &Query<(
        Entity,
        &ThirdPersonCameraLockOnTarget,
        Option<&GlobalTransform>,
    )>,
    active_target: Option<Entity>,
    followed_entity: Entity,
    origin: Vec3,
    max_distance: f32,
) -> bool {
    let Some(active_target) = active_target else {
        return false;
    };
    if active_target == followed_entity {
        return false;
    }
    let Ok((_, candidate, global_transform)) = candidates.get(active_target) else {
        return false;
    };
    let Some(global) = helper
        .compute_global_transform(active_target)
        .ok()
        .or_else(|| global_transform.copied())
    else {
        return false;
    };
    let (_, rotation, translation) = global.to_scale_rotation_translation();
    let anchor = translation + rotation * candidate.offset;
    origin.distance(anchor) <= max_distance
}

fn select_best_lock_on_candidate(
    helper: &TransformHelper,
    candidates: &Query<(
        Entity,
        &ThirdPersonCameraLockOnTarget,
        Option<&GlobalTransform>,
    )>,
    origin: Vec3,
    current_yaw: f32,
    max_distance: f32,
    followed_entity: Entity,
) -> Option<Entity> {
    let forward = Vec2::new(current_yaw.sin(), current_yaw.cos()).normalize_or_zero();
    let mut best = None;
    let mut best_score = f32::NEG_INFINITY;

    for (entity, candidate, global_transform) in candidates.iter() {
        if entity == followed_entity {
            continue;
        }
        let Some(global) = helper
            .compute_global_transform(entity)
            .ok()
            .or_else(|| global_transform.copied())
        else {
            continue;
        };
        let (_, rotation, translation) = global.to_scale_rotation_translation();
        let anchor = translation + rotation * candidate.offset;
        let direction = anchor - origin;
        let horizontal = Vec2::new(direction.x, direction.z);
        let horizontal_length = horizontal.length();
        if horizontal_length <= 0.000_1 {
            continue;
        }
        let distance = direction.length();
        if distance > max_distance {
            continue;
        }

        let facing = forward.dot(horizontal / horizontal_length);
        if facing < -0.35 {
            continue;
        }
        let score = facing * 2.0 + candidate.priority - distance / max_distance.max(0.001);
        if score > best_score {
            best_score = score;
            best = Some(entity);
        }
    }

    best
}

fn cycle_lock_on_candidate(
    helper: &TransformHelper,
    candidates: &Query<(
        Entity,
        &ThirdPersonCameraLockOnTarget,
        Option<&GlobalTransform>,
    )>,
    origin: Vec3,
    current_yaw: f32,
    active_target: Option<Entity>,
    max_distance: f32,
    followed_entity: Entity,
    forward_cycle: bool,
) -> Option<Entity> {
    let base_yaw = active_target
        .and_then(|entity| {
            let candidate = candidates.get(entity).ok()?;
            let global = helper
                .compute_global_transform(entity)
                .ok()
                .or_else(|| candidate.2.copied())?;
            let (_, rotation, translation) = global.to_scale_rotation_translation();
            let anchor = translation + rotation * candidate.1.offset;
            yaw_from_direction(anchor - origin)
        })
        .unwrap_or(current_yaw);

    let mut best = None;
    let mut best_delta = f32::INFINITY;

    for (entity, candidate, global_transform) in candidates.iter() {
        if entity == followed_entity || Some(entity) == active_target {
            continue;
        }
        let Some(global) = helper
            .compute_global_transform(entity)
            .ok()
            .or_else(|| global_transform.copied())
        else {
            continue;
        };
        let (_, rotation, translation) = global.to_scale_rotation_translation();
        let anchor = translation + rotation * candidate.offset;
        if origin.distance(anchor) > max_distance {
            continue;
        }
        let Some(candidate_yaw) = yaw_from_direction(anchor - origin) else {
            continue;
        };
        let delta = wrap_angle(candidate_yaw - base_yaw);
        let directional_delta = if forward_cycle {
            if delta <= 0.01 {
                continue;
            }
            delta
        } else {
            if delta >= -0.01 {
                continue;
            }
            -delta
        };

        if directional_delta < best_delta {
            best_delta = directional_delta;
            best = Some(entity);
        }
    }

    best.or_else(|| {
        select_best_lock_on_candidate(
            helper,
            candidates,
            origin,
            current_yaw,
            max_distance,
            followed_entity,
        )
    })
}

#[derive(Clone, Copy, Debug)]
struct ObstructionHit {
    entity: Entity,
    distance: f32,
    point: Option<Vec3>,
    normal: Vec3,
}

fn find_obstruction(
    pose: &CameraPose,
    settings: &ThirdPersonCameraSettings,
    ignore_set: &HashSet<Entity>,
    obstacles: &Query<(
        Entity,
        &ThirdPersonCameraObstacle,
        Option<&Aabb>,
        &GlobalTransform,
    )>,
) -> Option<ObstructionHit> {
    let mut best: Option<ObstructionHit> = None;
    for (entity, obstacle, aabb, global) in obstacles.iter() {
        if ignore_set.contains(&entity) {
            continue;
        }

        let (min, max) = world_aabb(aabb, global, obstacle, settings);
        for_each_obstruction_sample(
            settings.collision.strategy,
            pose.orientation,
            settings.collision.sample_offset_x,
            settings.collision.sample_offset_y,
            settings.collision.probe_radius,
            |sample| {
                let start = pose.look_target;
                let end = pose.desired_camera_position + sample;
                let Some(hit) = segment_aabb_hit(start, end, min, max) else {
                    return;
                };
                let segment_length = start.distance(end);
                let clearance = camera_radius_padding(settings, obstacle.kind);
                let hit_distance = (segment_length * hit.fraction - clearance)
                    .max(settings.collision.min_distance_from_target)
                    .max(0.0);
                let candidate = ObstructionHit {
                    entity,
                    distance: hit_distance.min(pose.desired_distance),
                    point: Some(hit.point),
                    normal: hit.normal,
                };
                if best.is_none_or(|best| candidate.distance < best.distance) {
                    best = Some(candidate);
                }
            },
        );
    }

    if let Some(mut hit) = best {
        if settings.collision.include_shape_radius && hit.distance < pose.desired_distance {
            hit.distance = hit
                .distance
                .max(settings.collision.min_distance_from_target)
                .min(pose.desired_distance);
        }
        return Some(hit);
    }
    None
}

fn world_aabb(
    aabb: Option<&Aabb>,
    global: &GlobalTransform,
    obstacle: &ThirdPersonCameraObstacle,
    settings: &ThirdPersonCameraSettings,
) -> (Vec3, Vec3) {
    let padding = obstacle.clearance + camera_radius_padding(settings, obstacle.kind);

    if let Some(aabb) = aabb {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        let local_min = aabb.min();
        let local_max = aabb.max();
        let matrix = global.to_matrix();
        for x in [local_min.x, local_max.x] {
            for y in [local_min.y, local_max.y] {
                for z in [local_min.z, local_max.z] {
                    let point = matrix.transform_point3(Vec3::new(x, y, z));
                    min = min.min(point);
                    max = max.max(point);
                }
            }
        }
        (min - Vec3::splat(padding), max + Vec3::splat(padding))
    } else {
        let center = global.translation();
        let half = Vec3::splat(0.5 + padding);
        (center - half, center + half)
    }
}

fn for_each_obstruction_sample(
    strategy: CollisionStrategy,
    orientation: Quat,
    offset_x: f32,
    offset_y: f32,
    probe_radius: f32,
    mut visit: impl FnMut(Vec3),
) {
    let right = orientation * Vec3::X;
    let up = orientation * Vec3::Y;
    match strategy {
        CollisionStrategy::SingleRay => visit(Vec3::ZERO),
        CollisionStrategy::MultiRay => {
            for sample in [
                Vec3::ZERO,
                right * offset_x + up * offset_y,
                right * offset_x - up * offset_y,
                -right * offset_x + up * offset_y,
                -right * offset_x - up * offset_y,
            ] {
                visit(sample);
            }
        }
        CollisionStrategy::SphereProbe => {
            for sample in [
                Vec3::ZERO,
                right * probe_radius,
                -right * probe_radius,
                up * probe_radius,
                -up * probe_radius,
                (right + up).normalize_or_zero() * probe_radius,
                (right - up).normalize_or_zero() * probe_radius,
                (-right + up).normalize_or_zero() * probe_radius,
                (-right - up).normalize_or_zero() * probe_radius,
            ] {
                visit(sample);
            }
        }
    }
}

fn minimum_camera_distance(
    settings: &ThirdPersonCameraSettings,
    camera: &ThirdPersonCamera,
) -> f32 {
    settings.collision.min_distance_from_target + camera.large_target_radius
}

fn clamp_camera_distance(
    value: f32,
    settings: &ThirdPersonCameraSettings,
    camera: &ThirdPersonCamera,
) -> f32 {
    let minimum_distance =
        minimum_camera_distance(settings, camera).max(settings.zoom.min_distance);
    value.clamp(
        minimum_distance,
        settings.zoom.max_distance.max(minimum_distance),
    )
}

fn camera_radius_padding(
    settings: &ThirdPersonCameraSettings,
    obstacle_type: crate::ObstacleType,
) -> f32 {
    if !settings.collision.include_shape_radius {
        return 0.0;
    }

    match obstacle_type {
        crate::ObstacleType::Blocker => settings.collision.probe_radius,
        crate::ObstacleType::Occluder => settings.collision.probe_radius * 0.5,
    }
}

#[cfg(test)]
#[path = "systems_tests.rs"]
mod systems_tests;
