use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum FollowAlignment {
    #[default]
    Free,
    TargetForward,
    MovementDirection,
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
}

impl Default for SmoothingSettings {
    fn default() -> Self {
        Self {
            orientation_smoothing: 16.0,
            target_follow_smoothing: 18.0,
            zoom_smoothing: 16.0,
            obstruction_pull_in: 28.0,
            obstruction_release: 10.0,
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
pub struct AnchorSettings {
    pub height: f32,
    pub radius_clearance: f32,
}

impl Default for AnchorSettings {
    fn default() -> Self {
        Self {
            height: 1.35,
            radius_clearance: 0.15,
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

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraSettings {
    pub enabled: bool,
    pub orbit: OrbitSettings,
    pub smoothing: SmoothingSettings,
    pub zoom: ZoomSettings,
    pub anchor: AnchorSettings,
    pub screen_framing: ScreenSpaceFramingSettings,
    pub collision: CollisionSettings,
    pub auto_recenter: AutoRecenterSettings,
}

impl Default for ThirdPersonCameraSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            orbit: OrbitSettings::default(),
            smoothing: SmoothingSettings::default(),
            zoom: ZoomSettings::default(),
            anchor: AnchorSettings::default(),
            screen_framing: ScreenSpaceFramingSettings::default(),
            collision: CollisionSettings::default(),
            auto_recenter: AutoRecenterSettings::default(),
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
