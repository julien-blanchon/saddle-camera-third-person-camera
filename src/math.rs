use bevy::prelude::*;

pub fn wrap_angle(mut angle: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    angle = (angle + std::f32::consts::PI).rem_euclid(tau) - std::f32::consts::PI;
    if angle <= -std::f32::consts::PI {
        angle += tau;
    }
    angle
}

pub fn shortest_angle_delta(current: f32, target: f32) -> f32 {
    wrap_angle(target - current)
}

pub fn smooth_factor(rate: f32, dt: f32) -> f32 {
    if rate <= 0.0 {
        1.0
    } else {
        1.0 - (-rate * dt).exp()
    }
}

pub fn smooth_scalar(current: f32, target: f32, rate: f32, dt: f32) -> f32 {
    current + (target - current) * smooth_factor(rate, dt)
}

pub fn smooth_angle(current: f32, target: f32, rate: f32, dt: f32) -> f32 {
    current + shortest_angle_delta(current, target) * smooth_factor(rate, dt)
}

pub fn smooth_vec3(current: Vec3, target: Vec3, rate: f32, dt: f32) -> Vec3 {
    current.lerp(target, smooth_factor(rate, dt))
}

pub fn yaw_pitch_rotation(yaw: f32, pitch: f32) -> Quat {
    Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0)
}

pub fn forward_from_angles(yaw: f32, pitch: f32) -> Vec3 {
    yaw_pitch_rotation(yaw, pitch) * -Vec3::Z
}

pub fn yaw_from_direction(direction: Vec3) -> Option<f32> {
    let flat = Vec2::new(direction.x, direction.z);
    if flat.length_squared() <= 0.000_001 {
        None
    } else {
        Some(flat.x.atan2(flat.y))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraPose {
    pub look_target: Vec3,
    pub desired_camera_position: Vec3,
    pub orientation: Quat,
    pub desired_distance: f32,
}

pub fn camera_pose_from_look_target(
    look_target: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
) -> CameraPose {
    let orientation = yaw_pitch_rotation(yaw, pitch);
    let forward = orientation * -Vec3::Z;
    CameraPose {
        look_target,
        desired_camera_position: look_target - forward * distance,
        orientation,
        desired_distance: distance,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SegmentHit {
    pub fraction: f32,
    pub point: Vec3,
    pub normal: Vec3,
}

pub fn segment_aabb_hit(start: Vec3, end: Vec3, min: Vec3, max: Vec3) -> Option<SegmentHit> {
    let direction = end - start;
    let mut t_min: f32 = 0.0;
    let mut t_max: f32 = 1.0;
    let mut normal = Vec3::ZERO;

    for axis in 0..3 {
        let origin = start[axis];
        let delta = direction[axis];
        let axis_min = min[axis];
        let axis_max = max[axis];

        if delta.abs() <= 0.000_001 {
            if origin < axis_min || origin > axis_max {
                return None;
            }
            continue;
        }

        let inv_delta = 1.0 / delta;
        let mut t1 = (axis_min - origin) * inv_delta;
        let mut t2 = (axis_max - origin) * inv_delta;
        let axis_normal = match axis {
            0 => Vec3::X,
            1 => Vec3::Y,
            _ => Vec3::Z,
        };
        let near_normal = -axis_normal * delta.signum();

        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }

        if t1 > t_min {
            t_min = t1;
            normal = near_normal;
        }
        t_max = t_max.min(t2);
        if t_min > t_max {
            return None;
        }
    }

    if !(0.0..=1.0).contains(&t_min) {
        return None;
    }

    let point = start + direction * t_min;
    Some(SegmentHit {
        fraction: t_min,
        point,
        normal,
    })
}

#[cfg(test)]
#[path = "math_tests.rs"]
mod math_tests;
