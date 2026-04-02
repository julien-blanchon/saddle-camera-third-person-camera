use bevy::prelude::*;

use crate::config::{ObstacleType, ShoulderSide, ThirdPersonCameraMode, ThirdPersonCameraSettings};

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
#[require(
    Camera3d,
    Transform,
    ThirdPersonCameraSettings,
    ThirdPersonCameraRuntime,
    ThirdPersonCameraInput
)]
pub struct ThirdPersonCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target_yaw: f32,
    pub target_pitch: f32,
    pub target_distance: f32,
    pub shoulder_side: ShoulderSide,
    pub target_shoulder_side: ShoulderSide,
    pub mode: ThirdPersonCameraMode,
    pub target_mode: ThirdPersonCameraMode,
    pub large_target_radius: f32,
    pub home_yaw: f32,
    pub home_pitch: f32,
    pub home_distance: f32,
    pub home_shoulder_side: ShoulderSide,
    pub home_mode: ThirdPersonCameraMode,
}

impl ThirdPersonCamera {
    pub fn new(distance: f32, yaw: f32, pitch: f32) -> Self {
        Self {
            yaw,
            pitch,
            distance,
            target_yaw: yaw,
            target_pitch: pitch,
            target_distance: distance,
            shoulder_side: ShoulderSide::Right,
            target_shoulder_side: ShoulderSide::Right,
            mode: ThirdPersonCameraMode::Center,
            target_mode: ThirdPersonCameraMode::Center,
            large_target_radius: 0.0,
            home_yaw: yaw,
            home_pitch: pitch,
            home_distance: distance,
            home_shoulder_side: ShoulderSide::Right,
            home_mode: ThirdPersonCameraMode::Center,
        }
    }

    pub fn looking_at(target: Vec3, eye: Vec3) -> Self {
        let delta = eye - target;
        let horizontal = Vec2::new(delta.x, delta.z);
        let distance = delta.length().max(0.01);
        let yaw = horizontal.x.atan2(horizontal.y);
        let pitch = (-delta.y).atan2(horizontal.length().max(0.001));
        Self::new(distance, yaw, pitch)
    }

    pub fn with_mode(mut self, mode: ThirdPersonCameraMode) -> Self {
        self.mode = mode;
        self.target_mode = mode;
        self.home_mode = mode;
        self
    }

    pub fn with_shoulder_side(mut self, side: ShoulderSide) -> Self {
        self.shoulder_side = side;
        self.target_shoulder_side = side;
        self.home_shoulder_side = side;
        self
    }

    pub fn with_large_target_radius(mut self, radius: f32) -> Self {
        self.large_target_radius = radius.max(0.0);
        self
    }

    pub fn capture_home_from_current(&mut self) {
        self.home_yaw = self.yaw;
        self.home_pitch = self.pitch;
        self.home_distance = self.distance;
        self.home_shoulder_side = self.shoulder_side;
        self.home_mode = self.mode;
    }

    pub fn reset_to_home(&mut self) {
        self.target_yaw = self.home_yaw;
        self.target_pitch = self.home_pitch;
        self.target_distance = self.home_distance;
        self.target_shoulder_side = self.home_shoulder_side;
        self.target_mode = self.home_mode;
    }
}

impl Default for ThirdPersonCamera {
    fn default() -> Self {
        Self::new(
            ThirdPersonCameraSettings::default().zoom.default_distance,
            0.0,
            -0.42,
        )
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraRuntime {
    pub pivot: Vec3,
    pub target_pivot: Vec3,
    pub look_target: Vec3,
    pub desired_distance: f32,
    pub corrected_distance: f32,
    pub obstruction_distance: f32,
    pub obstruction_active: bool,
    pub shoulder_blend: f32,
    pub target_shoulder_blend: f32,
    pub aim_blend: f32,
    pub target_aim_blend: f32,
    pub desired_camera_position: Vec3,
    pub corrected_camera_position: Vec3,
    pub last_hit_point: Option<Vec3>,
    pub last_hit_normal: Vec3,
    pub last_collision_target: Option<Entity>,
    pub idle_seconds: f32,
    pub manual_input_this_frame: bool,
    pub last_target_position: Vec3,
    pub cursor_locked: bool,
}

impl Default for ThirdPersonCameraRuntime {
    fn default() -> Self {
        let distance = ThirdPersonCameraSettings::default().zoom.default_distance;
        Self {
            pivot: Vec3::ZERO,
            target_pivot: Vec3::ZERO,
            look_target: Vec3::ZERO,
            desired_distance: distance,
            corrected_distance: distance,
            obstruction_distance: distance,
            obstruction_active: false,
            shoulder_blend: 0.0,
            target_shoulder_blend: 0.0,
            aim_blend: 0.0,
            target_aim_blend: 0.0,
            desired_camera_position: Vec3::ZERO,
            corrected_camera_position: Vec3::ZERO,
            last_hit_point: None,
            last_hit_normal: Vec3::ZERO,
            last_collision_target: None,
            idle_seconds: 0.0,
            manual_input_this_frame: false,
            last_target_position: Vec3::ZERO,
            cursor_locked: ThirdPersonCameraSettings::default().cursor.lock_by_default,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraTarget {
    pub target: Entity,
    pub offset: Vec3,
    pub follow_rotation: bool,
    pub enabled: bool,
    pub ignore_children: bool,
    pub ignored_entities: Vec<Entity>,
    pub recenter_on_target_change: bool,
}

impl ThirdPersonCameraTarget {
    pub fn new(target: Entity) -> Self {
        Self {
            target,
            offset: Vec3::ZERO,
            follow_rotation: true,
            enabled: true,
            ignore_children: true,
            ignored_entities: Vec::new(),
            recenter_on_target_change: true,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraInput {
    pub orbit_delta: Vec2,
    pub zoom_delta: f32,
    pub shoulder_toggle: bool,
    pub shoulder_hold: bool,
    pub aim: bool,
    pub recenter: bool,
    pub cursor_lock_toggle: bool,
    pub raw_mode_center: bool,
    pub raw_mode_shoulder: bool,
}

impl ThirdPersonCameraInput {
    pub fn clear_transient(&mut self) {
        *self = Self::default();
    }

    pub fn has_manual_motion(&self) -> bool {
        self.orbit_delta.length_squared() > 0.0
            || self.zoom_delta.abs() > f32::EPSILON
            || self.shoulder_toggle
            || self.shoulder_hold
            || self.aim
            || self.recenter
            || self.cursor_lock_toggle
            || self.raw_mode_center
            || self.raw_mode_shoulder
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraDebug {
    pub enabled: bool,
    pub draw_pivot: bool,
    pub draw_desired: bool,
    pub draw_corrected: bool,
    pub draw_hits: bool,
}

impl Default for ThirdPersonCameraDebug {
    fn default() -> Self {
        Self {
            enabled: true,
            draw_pivot: true,
            draw_desired: true,
            draw_corrected: true,
            draw_hits: true,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraObstacle {
    pub kind: ObstacleType,
    pub clearance: f32,
}

impl Default for ThirdPersonCameraObstacle {
    fn default() -> Self {
        Self {
            kind: ObstacleType::Blocker,
            clearance: 0.0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraInputTarget;

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraIgnore;

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraIgnoreTarget;
