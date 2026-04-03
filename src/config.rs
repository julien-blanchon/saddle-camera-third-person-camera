use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum FollowAlignment {
    #[default]
    Free,
    TargetForward,
    MovementDirection,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum ShoulderSide {
    Left,
    #[default]
    Right,
}

impl ShoulderSide {
    pub fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    pub fn sign(self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum ThirdPersonCameraMode {
    #[default]
    Center,
    Shoulder,
    Aim,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum ObstacleType {
    #[default]
    Blocker,
    Occluder,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum CollisionStrategy {
    SingleRay,
    #[default]
    MultiRay,
    SphereProbe,
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct OrbitSettings {
    pub yaw_speed: f32,
    pub pitch_speed: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub invert_x: bool,
    pub invert_y: bool,
}

impl Default for OrbitSettings {
    fn default() -> Self {
        Self {
            yaw_speed: 1.2,
            pitch_speed: 1.1,
            min_pitch: -1.25,
            max_pitch: -0.08,
            invert_x: false,
            invert_y: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct SmoothingSettings {
    pub orientation_smoothing: f32,
    pub target_follow_smoothing: f32,
    pub zoom_smoothing: f32,
    pub obstruction_pull_in: f32,
    pub obstruction_release: f32,
    pub shoulder_blend: f32,
    pub aim_blend: f32,
}

impl Default for SmoothingSettings {
    fn default() -> Self {
        Self {
            orientation_smoothing: 16.0,
            target_follow_smoothing: 18.0,
            zoom_smoothing: 16.0,
            obstruction_pull_in: 28.0,
            obstruction_release: 10.0,
            shoulder_blend: 14.0,
            aim_blend: 20.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct ZoomSettings {
    pub min_distance: f32,
    pub max_distance: f32,
    pub default_distance: f32,
    pub step: f32,
}

impl Default for ZoomSettings {
    fn default() -> Self {
        Self {
            min_distance: 1.15,
            max_distance: 10.0,
            default_distance: 4.6,
            step: 0.8,
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct FramingSettings {
    pub shoulder_offset: f32,
    pub shoulder_height: f32,
    pub default_side: ShoulderSide,
    pub aim_enabled: bool,
    pub aim_distance_scale: f32,
    pub aim_pitch_offset: f32,
    pub target_radius_clearance: f32,
}

impl Default for FramingSettings {
    fn default() -> Self {
        Self {
            shoulder_offset: 0.75,
            shoulder_height: 1.35,
            default_side: ShoulderSide::Right,
            aim_enabled: true,
            aim_distance_scale: 0.62,
            aim_pitch_offset: 0.10,
            target_radius_clearance: 0.55,
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct ScreenSpaceFramingSettings {
    pub enabled: bool,
    pub dead_zone: Vec2,
    pub soft_zone: Vec2,
    pub screen_offset: Vec2,
}

impl Default for ScreenSpaceFramingSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            dead_zone: Vec2::new(0.18, 0.14),
            soft_zone: Vec2::new(0.42, 0.32),
            screen_offset: Vec2::ZERO,
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct LockOnSettings {
    pub enabled: bool,
    pub max_distance: f32,
    pub focus_bias: f32,
    pub pitch_offset: f32,
}

impl Default for LockOnSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            max_distance: 24.0,
            focus_bias: 0.35,
            pitch_offset: 0.08,
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct CollisionSettings {
    pub enabled: bool,
    pub strategy: CollisionStrategy,
    pub probe_radius: f32,
    pub sample_offset_x: f32,
    pub sample_offset_y: f32,
    pub min_distance_from_target: f32,
    pub include_shape_radius: bool,
}

impl Default for CollisionSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            strategy: CollisionStrategy::MultiRay,
            probe_radius: 0.28,
            sample_offset_x: 0.30,
            sample_offset_y: 0.22,
            min_distance_from_target: 0.8,
            include_shape_radius: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct AutoRecenterSettings {
    pub enabled: bool,
    pub inactivity_seconds: f32,
    pub follow_alignment: FollowAlignment,
}

impl Default for AutoRecenterSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            inactivity_seconds: 2.0,
            follow_alignment: FollowAlignment::TargetForward,
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct CursorPolicy {
    pub lock_by_default: bool,
    pub allow_toggle: bool,
}

impl Default for CursorPolicy {
    fn default() -> Self {
        Self {
            lock_by_default: true,
            allow_toggle: true,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraSettings {
    pub enabled: bool,
    pub orbit: OrbitSettings,
    pub smoothing: SmoothingSettings,
    pub zoom: ZoomSettings,
    pub framing: FramingSettings,
    pub screen_framing: ScreenSpaceFramingSettings,
    pub collision: CollisionSettings,
    pub lock_on: LockOnSettings,
    pub auto_recenter: AutoRecenterSettings,
    pub cursor: CursorPolicy,
}

impl Default for ThirdPersonCameraSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            orbit: OrbitSettings::default(),
            smoothing: SmoothingSettings::default(),
            zoom: ZoomSettings::default(),
            framing: FramingSettings::default(),
            screen_framing: ScreenSpaceFramingSettings::default(),
            collision: CollisionSettings::default(),
            lock_on: LockOnSettings::default(),
            auto_recenter: AutoRecenterSettings::default(),
            cursor: CursorPolicy::default(),
        }
    }
}

impl ThirdPersonCameraSettings {
    pub fn clamped_zoom(self, value: f32) -> f32 {
        value.clamp(self.zoom.min_distance, self.zoom.max_distance)
    }

    pub fn clamped_pitch(self, value: f32) -> f32 {
        value.clamp(self.orbit.min_pitch, self.orbit.max_pitch)
    }
}
