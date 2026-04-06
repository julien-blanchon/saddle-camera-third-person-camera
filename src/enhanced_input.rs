use bevy::{
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};
use bevy_enhanced_input::{
    EnhancedInputPlugin, EnhancedInputSystems,
    context::InputContextAppExt,
    prelude::{
        Action, Axial, Bidirectional, Binding, Bindings, Cancel as InputCancel, Complete, DeadZone,
        Fire, InputAction, Press, Scale, Start, actions, bindings,
    },
    preset::WithBundle,
};

use crate::{
    ThirdPersonCamera, ThirdPersonCameraActionInput, ThirdPersonCameraInput,
    ThirdPersonCameraSettings, ThirdPersonCameraSystems,
};

#[derive(Resource, Default, Clone, Copy)]
struct ActiveInputCamera(Option<Entity>);

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraEnhancedInputContext;

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraEnhancedInputTarget;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub struct OrbitAction;

#[derive(Debug, InputAction)]
#[action_output(f32)]
pub struct ZoomAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct AimAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct ToggleLockOnAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct NextLockOnTargetAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct PreviousLockOnTargetAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct ToggleShoulderAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct ShoulderHoldAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct RecenterAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct CursorLockAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct ForceCenterModeAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct ForceShoulderModeAction;

pub struct ThirdPersonCameraEnhancedInputPlugin {
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl ThirdPersonCameraEnhancedInputPlugin {
    pub fn new(update_schedule: impl ScheduleLabel) -> Self {
        Self {
            update_schedule: update_schedule.intern(),
        }
    }
}

impl Default for ThirdPersonCameraEnhancedInputPlugin {
    fn default() -> Self {
        Self::new(Update)
    }
}

impl Plugin for ThirdPersonCameraEnhancedInputPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EnhancedInputPlugin>() {
            app.add_plugins(EnhancedInputPlugin);
        }

        app.init_resource::<ActiveInputCamera>()
            .add_input_context::<ThirdPersonCameraEnhancedInputContext>()
            .register_type::<ThirdPersonCameraEnhancedInputContext>()
            .register_type::<ThirdPersonCameraEnhancedInputTarget>()
            .add_observer(cache_orbit_delta)
            .add_observer(cache_zoom_delta)
            .add_observer(cache_aim_active)
            .add_observer(cache_lock_on_toggle)
            .add_observer(cache_lock_on_next)
            .add_observer(cache_lock_on_previous)
            .add_observer(cache_shoulder_hold)
            .add_observer(cache_shoulder_toggle)
            .add_observer(cache_recenter)
            .add_observer(cache_cursor_toggle)
            .add_observer(cache_force_center_mode)
            .add_observer(cache_force_shoulder_mode)
            .add_observer(clear_orbit_delta_on_cancel)
            .add_observer(clear_orbit_delta_on_complete)
            .add_observer(clear_zoom_delta_on_cancel)
            .add_observer(clear_zoom_delta_on_complete)
            .add_systems(
                self.update_schedule,
                route_active_input
                    .in_set(ThirdPersonCameraSystems::ReadInput)
                    .after(EnhancedInputSystems::Apply),
            );
    }
}

pub fn default_input_bindings() -> impl Bundle {
    (
        ThirdPersonCameraEnhancedInputContext,
        actions!(ThirdPersonCameraEnhancedInputContext[
            (
                Action::<OrbitAction>::new(),
                Bindings::spawn((
                    Spawn((Binding::mouse_motion(), Scale::splat(0.006))),
                    Axial::right_stick().with((Scale::splat(0.045), DeadZone::default())),
                )),
            ),
            (
                Action::<ZoomAction>::new(),
                Bindings::spawn((
                    Spawn((Binding::mouse_wheel(), Scale::splat(1.0))),
                    Bidirectional::new(GamepadButton::DPadDown, GamepadButton::DPadUp),
                    Bidirectional::new(KeyCode::PageDown, KeyCode::PageUp),
                )),
            ),
            (
                Action::<AimAction>::new(),
                bindings![MouseButton::Right, GamepadButton::LeftTrigger2],
            ),
            (
                Action::<ToggleLockOnAction>::new(),
                Press::default(),
                bindings![KeyCode::KeyF, GamepadButton::RightThumb],
            ),
            (
                Action::<NextLockOnTargetAction>::new(),
                Press::default(),
                bindings![KeyCode::KeyE, GamepadButton::DPadUp],
            ),
            (
                Action::<PreviousLockOnTargetAction>::new(),
                Press::default(),
                bindings![KeyCode::KeyZ, GamepadButton::DPadLeft],
            ),
            (
                Action::<ToggleShoulderAction>::new(),
                Press::default(),
                bindings![KeyCode::KeyC, GamepadButton::DPadRight],
            ),
            (
                Action::<ShoulderHoldAction>::new(),
                bindings![KeyCode::Tab, GamepadButton::LeftTrigger],
            ),
            (
                Action::<RecenterAction>::new(),
                Press::default(),
                bindings![KeyCode::KeyR, GamepadButton::North],
            ),
            (
                Action::<CursorLockAction>::new(),
                Press::default(),
                bindings![KeyCode::KeyQ, GamepadButton::West],
            ),
            (
                Action::<ForceCenterModeAction>::new(),
                Press::default(),
                bindings![KeyCode::Digit1],
            ),
            (
                Action::<ForceShoulderModeAction>::new(),
                Press::default(),
                bindings![KeyCode::Digit2],
            ),
        ]),
    )
}

fn route_active_input(
    mut active_input_camera: ResMut<ActiveInputCamera>,
    mut cameras: ParamSet<(
        Query<
            (Entity, &Camera, &ThirdPersonCameraSettings),
            With<ThirdPersonCameraEnhancedInputTarget>,
        >,
        Query<
            (
                Entity,
                &mut ThirdPersonCameraInput,
                Option<&mut ThirdPersonCameraActionInput>,
            ),
            With<ThirdPersonCamera>,
        >,
    )>,
) {
    let active = cameras
        .p0()
        .iter()
        .filter(|(_, camera_component, settings)| camera_component.is_active && settings.enabled)
        .max_by_key(|(entity, camera_component, _)| (camera_component.order, entity.to_bits()))
        .map(|(entity, _, _)| entity);
    active_input_camera.0 = active;

    for (entity, mut input, action_input) in &mut cameras.p1() {
        if Some(entity) != active {
            input.clear_transient();
            if let Some(mut action_input) = action_input {
                action_input.clear_transient();
            }
        }
    }
}

pub(crate) fn cache_orbit_delta(
    trigger: On<Fire<OrbitAction>>,
    mut cameras: Query<&mut ThirdPersonCameraInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.orbit_delta += trigger.value;
    }
}

pub(crate) fn cache_zoom_delta(
    trigger: On<Fire<ZoomAction>>,
    mut cameras: Query<&mut ThirdPersonCameraInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.zoom_delta += trigger.value;
    }
}

pub(crate) fn cache_aim_active(
    trigger: On<Fire<AimAction>>,
    mut cameras: Query<&mut ThirdPersonCameraActionInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.aim = input.aim || trigger.value;
    }
}

pub(crate) fn cache_lock_on_toggle(
    trigger: On<Start<ToggleLockOnAction>>,
    mut cameras: Query<&mut ThirdPersonCameraActionInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.lock_on_toggle = true;
    }
}

pub(crate) fn cache_lock_on_next(
    trigger: On<Start<NextLockOnTargetAction>>,
    mut cameras: Query<&mut ThirdPersonCameraActionInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.lock_on_next = true;
    }
}

pub(crate) fn cache_lock_on_previous(
    trigger: On<Start<PreviousLockOnTargetAction>>,
    mut cameras: Query<&mut ThirdPersonCameraActionInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.lock_on_previous = true;
    }
}

pub(crate) fn cache_shoulder_hold(
    trigger: On<Fire<ShoulderHoldAction>>,
    mut cameras: Query<&mut ThirdPersonCameraActionInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.shoulder_hold = input.shoulder_hold || trigger.value;
    }
}

pub(crate) fn cache_shoulder_toggle(
    trigger: On<Start<ToggleShoulderAction>>,
    mut cameras: Query<&mut ThirdPersonCameraActionInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.shoulder_toggle = true;
    }
}

pub(crate) fn cache_recenter(
    trigger: On<Start<RecenterAction>>,
    mut cameras: Query<&mut ThirdPersonCameraInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.recenter = true;
    }
}

pub(crate) fn cache_cursor_toggle(
    trigger: On<Start<CursorLockAction>>,
    mut cameras: Query<&mut ThirdPersonCameraActionInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.cursor_lock_toggle = true;
    }
}

pub(crate) fn cache_force_center_mode(
    trigger: On<Start<ForceCenterModeAction>>,
    mut cameras: Query<&mut ThirdPersonCameraActionInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.raw_mode_center = true;
    }
}

pub(crate) fn cache_force_shoulder_mode(
    trigger: On<Start<ForceShoulderModeAction>>,
    mut cameras: Query<&mut ThirdPersonCameraActionInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.raw_mode_shoulder = true;
    }
}

pub(crate) fn clear_orbit_delta_on_cancel(
    trigger: On<InputCancel<OrbitAction>>,
    mut cameras: Query<&mut ThirdPersonCameraInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.orbit_delta = Vec2::ZERO;
    }
}

pub(crate) fn clear_orbit_delta_on_complete(
    trigger: On<Complete<OrbitAction>>,
    mut cameras: Query<&mut ThirdPersonCameraInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.orbit_delta = Vec2::ZERO;
    }
}

pub(crate) fn clear_zoom_delta_on_complete(
    trigger: On<Complete<ZoomAction>>,
    mut cameras: Query<&mut ThirdPersonCameraInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.zoom_delta = 0.0;
    }
}

pub(crate) fn clear_zoom_delta_on_cancel(
    trigger: On<InputCancel<ZoomAction>>,
    mut cameras: Query<&mut ThirdPersonCameraInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.zoom_delta = 0.0;
    }
}
