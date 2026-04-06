use bevy::prelude::*;

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

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraActionInput {
    pub shoulder_toggle: bool,
    pub shoulder_hold: bool,
    pub aim: bool,
    pub lock_on_toggle: bool,
    pub lock_on_next: bool,
    pub lock_on_previous: bool,
    pub cursor_lock_toggle: bool,
    pub raw_mode_center: bool,
    pub raw_mode_shoulder: bool,
}

impl ThirdPersonCameraActionInput {
    pub fn clear_transient(&mut self) {
        *self = Self::default();
    }

    pub fn has_manual_motion(&self) -> bool {
        self.shoulder_toggle
            || self.shoulder_hold
            || self.aim
            || self.lock_on_toggle
            || self.lock_on_next
            || self.lock_on_previous
            || self.cursor_lock_toggle
            || self.raw_mode_center
            || self.raw_mode_shoulder
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraShoulderSettings {
    pub shoulder_offset: f32,
    pub default_side: ShoulderSide,
    pub aim_enabled: bool,
    pub aim_distance_scale: f32,
    pub aim_pitch_offset: f32,
    pub aim_height_offset: f32,
    pub shoulder_blend_smoothing: f32,
    pub aim_blend_smoothing: f32,
}

impl Default for ThirdPersonCameraShoulderSettings {
    fn default() -> Self {
        Self {
            shoulder_offset: 0.75,
            default_side: ShoulderSide::Right,
            aim_enabled: true,
            aim_distance_scale: 0.62,
            aim_pitch_offset: 0.10,
            aim_height_offset: -0.35,
            shoulder_blend_smoothing: 14.0,
            aim_blend_smoothing: 20.0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraShoulderRuntime {
    pub shoulder_blend: f32,
    pub target_shoulder_blend: f32,
    pub aim_blend: f32,
    pub target_aim_blend: f32,
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
#[require(
    ThirdPersonCameraActionInput,
    ThirdPersonCameraShoulderSettings,
    ThirdPersonCameraShoulderRuntime
)]
pub struct ThirdPersonCameraShoulderRig {
    pub shoulder_side: ShoulderSide,
    pub target_shoulder_side: ShoulderSide,
    pub mode: ThirdPersonCameraMode,
    pub target_mode: ThirdPersonCameraMode,
    pub home_shoulder_side: ShoulderSide,
    pub home_mode: ThirdPersonCameraMode,
}

impl ThirdPersonCameraShoulderRig {
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

    pub fn capture_home_from_current(&mut self) {
        self.home_shoulder_side = self.shoulder_side;
        self.home_mode = self.mode;
    }

    pub fn reset_to_home(&mut self) {
        self.target_shoulder_side = self.home_shoulder_side;
        self.target_mode = self.home_mode;
    }
}

impl Default for ThirdPersonCameraShoulderRig {
    fn default() -> Self {
        Self {
            shoulder_side: ShoulderSide::Right,
            target_shoulder_side: ShoulderSide::Right,
            mode: ThirdPersonCameraMode::Center,
            target_mode: ThirdPersonCameraMode::Center,
            home_shoulder_side: ShoulderSide::Right,
            home_mode: ThirdPersonCameraMode::Center,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraLockOnSettings {
    pub enabled: bool,
    pub max_distance: f32,
    pub focus_bias: f32,
    pub pitch_offset: f32,
    pub blend_smoothing: f32,
}

impl Default for ThirdPersonCameraLockOnSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            max_distance: 24.0,
            focus_bias: 0.35,
            pitch_offset: 0.08,
            blend_smoothing: 20.0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraLockOnRuntime {
    pub focus: Vec3,
    pub blend: f32,
    pub target_blend: f32,
    pub active_target: Option<Entity>,
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
#[require(
    ThirdPersonCameraActionInput,
    ThirdPersonCameraLockOnSettings,
    ThirdPersonCameraLockOnRuntime
)]
pub struct ThirdPersonCameraLockOn {
    pub active_target: Option<Entity>,
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraLockOnTarget {
    pub offset: Vec3,
    pub priority: f32,
}

impl Default for ThirdPersonCameraLockOnTarget {
    fn default() -> Self {
        Self {
            offset: Vec3::ZERO,
            priority: 0.0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
#[require(ThirdPersonCameraActionInput)]
pub struct ThirdPersonCameraCursorController {
    pub lock_by_default: bool,
    pub allow_toggle: bool,
    pub locked: bool,
}

impl ThirdPersonCameraCursorController {
    pub const fn new(lock_by_default: bool, allow_toggle: bool) -> Self {
        Self {
            lock_by_default,
            allow_toggle,
            locked: lock_by_default,
        }
    }
}

impl Default for ThirdPersonCameraCursorController {
    fn default() -> Self {
        Self::new(true, true)
    }
}

pub(crate) fn initialize_action_adapters(
    mut shoulder_cameras: Query<
        (
            &mut ThirdPersonCameraShoulderRig,
            &ThirdPersonCameraShoulderSettings,
            &mut ThirdPersonCameraShoulderRuntime,
        ),
        Added<ThirdPersonCameraShoulderRig>,
    >,
    mut cursor_cameras: Query<
        &mut ThirdPersonCameraCursorController,
        Added<ThirdPersonCameraCursorController>,
    >,
) {
    for (mut rig, settings, mut runtime) in &mut shoulder_cameras {
        if settings.default_side == ShoulderSide::Left
            && rig.shoulder_side == ShoulderSide::Right
            && rig.target_shoulder_side == ShoulderSide::Right
            && rig.home_shoulder_side == ShoulderSide::Right
        {
            rig.shoulder_side = ShoulderSide::Left;
            rig.target_shoulder_side = ShoulderSide::Left;
            rig.home_shoulder_side = ShoulderSide::Left;
        }

        runtime.target_shoulder_blend =
            shoulder_blend_target(rig.target_mode, rig.target_shoulder_side);
        runtime.shoulder_blend = runtime.target_shoulder_blend;
        runtime.target_aim_blend = aim_blend_target(rig.target_mode, settings);
        runtime.aim_blend = runtime.target_aim_blend;
    }

    for mut cursor in &mut cursor_cameras {
        cursor.locked = cursor.lock_by_default;
    }
}

pub(crate) fn clear_consumed_action_input(mut cameras: Query<&mut ThirdPersonCameraActionInput>) {
    for mut input in &mut cameras {
        input.clear_transient();
    }
}

pub(crate) fn effective_target_mode(
    persistent_mode: ThirdPersonCameraMode,
    input: &ThirdPersonCameraActionInput,
    settings: &ThirdPersonCameraShoulderSettings,
    lock_on_active: bool,
) -> ThirdPersonCameraMode {
    if settings.aim_enabled && (input.aim || lock_on_active) {
        ThirdPersonCameraMode::Aim
    } else if input.shoulder_hold {
        ThirdPersonCameraMode::Shoulder
    } else {
        persistent_mode
    }
}

pub(crate) fn current_mode(aim_blend: f32, shoulder_blend: f32) -> ThirdPersonCameraMode {
    if aim_blend > 0.5 {
        ThirdPersonCameraMode::Aim
    } else if shoulder_blend.abs() > 0.5 {
        ThirdPersonCameraMode::Shoulder
    } else {
        ThirdPersonCameraMode::Center
    }
}

pub(crate) fn shoulder_blend_target(mode: ThirdPersonCameraMode, side: ShoulderSide) -> f32 {
    match mode {
        ThirdPersonCameraMode::Center => 0.0,
        ThirdPersonCameraMode::Shoulder | ThirdPersonCameraMode::Aim => side.sign(),
    }
}

pub(crate) fn aim_blend_target(
    mode: ThirdPersonCameraMode,
    settings: &ThirdPersonCameraShoulderSettings,
) -> f32 {
    if settings.aim_enabled && mode == ThirdPersonCameraMode::Aim {
        1.0
    } else {
        0.0
    }
}
