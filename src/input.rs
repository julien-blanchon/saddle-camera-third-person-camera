use bevy::prelude::*;
use bevy_enhanced_input::prelude::{
    Action, Axial, Bidirectional, Binding, Bindings, Cancel as InputCancel, Complete, DeadZone,
    Fire, InputAction, Press, Scale, Start, actions, bindings,
};
use bevy_enhanced_input::preset::WithBundle;

use crate::{ThirdPersonCamera, ThirdPersonCameraInput};

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ThirdPersonCameraInputContext;

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

pub fn default_input_bindings() -> impl Bundle {
    (
        ThirdPersonCameraInputContext,
        actions!(ThirdPersonCameraInputContext[
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
    mut cameras: Query<&mut ThirdPersonCameraInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.aim = input.aim || trigger.value;
    }
}

pub(crate) fn cache_shoulder_hold(
    trigger: On<Fire<ShoulderHoldAction>>,
    mut cameras: Query<&mut ThirdPersonCameraInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.shoulder_hold = input.shoulder_hold || trigger.value;
    }
}

pub(crate) fn cache_shoulder_toggle(
    trigger: On<Start<ToggleShoulderAction>>,
    mut cameras: Query<&mut ThirdPersonCameraInput, With<ThirdPersonCamera>>,
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
    mut cameras: Query<&mut ThirdPersonCameraInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.cursor_lock_toggle = true;
    }
}

pub(crate) fn cache_force_center_mode(
    trigger: On<Start<ForceCenterModeAction>>,
    mut cameras: Query<&mut ThirdPersonCameraInput, With<ThirdPersonCamera>>,
) {
    if let Ok(mut input) = cameras.get_mut(trigger.context) {
        input.raw_mode_center = true;
    }
}

pub(crate) fn cache_force_shoulder_mode(
    trigger: On<Start<ForceShoulderModeAction>>,
    mut cameras: Query<&mut ThirdPersonCameraInput, With<ThirdPersonCamera>>,
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
