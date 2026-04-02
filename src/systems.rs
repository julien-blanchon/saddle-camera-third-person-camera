use std::collections::HashSet;

use bevy::{
    camera::primitives::Aabb,
    prelude::*,
    transform::helper::TransformHelper,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::{
    CollisionStrategy, ShoulderSide, ThirdPersonCamera, ThirdPersonCameraIgnore,
    ThirdPersonCameraIgnoreTarget, ThirdPersonCameraInput, ThirdPersonCameraInputTarget,
    ThirdPersonCameraMode, ThirdPersonCameraObstacle, ThirdPersonCameraRuntime,
    ThirdPersonCameraSettings, ThirdPersonCameraTarget,
    math::{
        CameraPose, camera_pose_from_look_target, forward_from_angles, segment_aabb_hit,
        smooth_angle, smooth_scalar, smooth_vec3, wrap_angle, yaw_from_direction,
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

    for (entity, mut input) in &mut cameras.p1() {
        if Some(entity) != active {
            input.clear_transient();
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
    mut ignore_scratch: Local<HashSet<Entity>>,
    mut primary_window: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut cameras: Query<(
        Entity,
        &mut ThirdPersonCamera,
        &ThirdPersonCameraSettings,
        Option<Ref<'_, ThirdPersonCameraTarget>>,
        &mut ThirdPersonCameraRuntime,
        &ThirdPersonCameraInput,
    )>,
) {
    let dt = time.delta_secs();
    for (camera_entity, mut camera, settings, target, mut runtime, input) in &mut cameras {
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
        runtime.target_pivot = sampled.look_anchor;
        if runtime.pivot == Vec3::ZERO || target_changed {
            runtime.pivot = sampled.look_anchor;
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

        let desired_mode = effective_target_mode(camera.target_mode, input, settings);

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

        let look_target = runtime.pivot
            + right_offset(
                camera.yaw,
                runtime.shoulder_blend,
                settings.framing.shoulder_offset,
            );
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
) -> ThirdPersonCameraMode {
    if settings.framing.aim_enabled && input.aim {
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

fn right_offset(yaw: f32, shoulder_blend: f32, offset: f32) -> Vec3 {
    Quat::from_rotation_y(yaw) * Vec3::X * (offset * shoulder_blend)
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
