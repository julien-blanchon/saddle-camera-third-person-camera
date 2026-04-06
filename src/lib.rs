mod action;
mod components;
mod config;
mod debug;
#[cfg(feature = "enhanced-input")]
mod enhanced_input;
mod math;
mod systems;

pub use action::{
    ShoulderSide, ThirdPersonCameraActionInput, ThirdPersonCameraCursorController,
    ThirdPersonCameraLockOn, ThirdPersonCameraLockOnRuntime, ThirdPersonCameraLockOnSettings,
    ThirdPersonCameraLockOnTarget, ThirdPersonCameraMode, ThirdPersonCameraShoulderRig,
    ThirdPersonCameraShoulderRuntime, ThirdPersonCameraShoulderSettings,
};
pub use components::{
    ThirdPersonCamera, ThirdPersonCameraDebug, ThirdPersonCameraIgnore,
    ThirdPersonCameraIgnoreTarget, ThirdPersonCameraInput, ThirdPersonCameraObstacle,
    ThirdPersonCameraRuntime, ThirdPersonCameraTarget,
};
pub use config::{
    AnchorSettings, AutoRecenterSettings, CollisionSettings, CollisionStrategy, FollowAlignment,
    ObstacleType, OrbitSettings, ScreenSpaceFramingSettings, SmoothingSettings,
    ThirdPersonCameraSettings, ZoomSettings,
};
#[cfg(feature = "enhanced-input")]
pub use enhanced_input::{
    AimAction, CursorLockAction, ForceCenterModeAction, ForceShoulderModeAction,
    NextLockOnTargetAction, OrbitAction, PreviousLockOnTargetAction, RecenterAction,
    ShoulderHoldAction, ThirdPersonCameraEnhancedInputContext,
    ThirdPersonCameraEnhancedInputPlugin, ThirdPersonCameraEnhancedInputTarget, ToggleLockOnAction,
    ToggleShoulderAction, ZoomAction, default_input_bindings,
};
pub use math::{
    CameraPose, SegmentHit, camera_pose_from_look_target, forward_from_angles, segment_aabb_hit,
    shortest_angle_delta, smooth_angle, smooth_factor, smooth_scalar, smooth_vec3, wrap_angle,
    yaw_from_direction, yaw_pitch_rotation,
};

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    gizmos::{config::DefaultGizmoConfigGroup, gizmos::GizmoStorage},
    prelude::*,
    transform::TransformSystems,
};

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ThirdPersonCameraSystems {
    ReadInput,
    UpdateIntent,
    ResolveObstruction,
    ApplyTransform,
    DebugDraw,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

#[derive(Resource, Default)]
struct ThirdPersonCameraRuntimeActive(bool);

pub struct ThirdPersonCameraPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl ThirdPersonCameraPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for ThirdPersonCameraPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for ThirdPersonCameraPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.init_resource::<ThirdPersonCameraRuntimeActive>()
            .register_type::<AnchorSettings>()
            .register_type::<AutoRecenterSettings>()
            .register_type::<CollisionSettings>()
            .register_type::<CollisionStrategy>()
            .register_type::<FollowAlignment>()
            .register_type::<ObstacleType>()
            .register_type::<OrbitSettings>()
            .register_type::<ScreenSpaceFramingSettings>()
            .register_type::<ShoulderSide>()
            .register_type::<SmoothingSettings>()
            .register_type::<ThirdPersonCamera>()
            .register_type::<ThirdPersonCameraActionInput>()
            .register_type::<ThirdPersonCameraCursorController>()
            .register_type::<ThirdPersonCameraDebug>()
            .register_type::<ThirdPersonCameraIgnore>()
            .register_type::<ThirdPersonCameraIgnoreTarget>()
            .register_type::<ThirdPersonCameraInput>()
            .register_type::<ThirdPersonCameraLockOn>()
            .register_type::<ThirdPersonCameraLockOnRuntime>()
            .register_type::<ThirdPersonCameraLockOnSettings>()
            .register_type::<ThirdPersonCameraLockOnTarget>()
            .register_type::<ThirdPersonCameraMode>()
            .register_type::<ThirdPersonCameraObstacle>()
            .register_type::<ThirdPersonCameraRuntime>()
            .register_type::<ThirdPersonCameraSettings>()
            .register_type::<ThirdPersonCameraShoulderRig>()
            .register_type::<ThirdPersonCameraShoulderRuntime>()
            .register_type::<ThirdPersonCameraShoulderSettings>()
            .register_type::<ThirdPersonCameraTarget>()
            .register_type::<ZoomSettings>()
            .add_systems(self.activate_schedule, activate_runtime)
            .add_systems(self.deactivate_schedule, deactivate_runtime)
            .add_systems(
                self.update_schedule,
                (
                    systems::initialize_added_cameras,
                    action::initialize_action_adapters,
                ),
            )
            .configure_sets(self.update_schedule, ThirdPersonCameraSystems::ReadInput)
            .configure_sets(
                PostUpdate,
                (
                    ThirdPersonCameraSystems::UpdateIntent,
                    ThirdPersonCameraSystems::ResolveObstruction,
                    ThirdPersonCameraSystems::ApplyTransform,
                    ThirdPersonCameraSystems::DebugDraw,
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                (
                    systems::initialize_added_cameras,
                    action::initialize_action_adapters,
                    systems::update_lock_on_selection,
                    systems::update_camera_runtime,
                )
                    .chain()
                    .in_set(ThirdPersonCameraSystems::UpdateIntent)
                    .run_if(runtime_is_active),
            )
            .add_systems(
                PostUpdate,
                systems::resolve_obstruction
                    .in_set(ThirdPersonCameraSystems::ResolveObstruction)
                    .run_if(runtime_is_active),
            )
            .add_systems(
                PostUpdate,
                (
                    systems::apply_camera_transform,
                    systems::clear_consumed_input,
                    action::clear_consumed_action_input,
                )
                    .chain()
                    .in_set(ThirdPersonCameraSystems::ApplyTransform)
                    .before(TransformSystems::Propagate)
                    .run_if(runtime_is_active),
            )
            .add_systems(
                PostUpdate,
                debug::draw_debug_gizmos
                    .in_set(ThirdPersonCameraSystems::DebugDraw)
                    .run_if(resource_exists::<GizmoStorage<DefaultGizmoConfigGroup, ()>>)
                    .run_if(runtime_is_active),
            );
    }
}

fn activate_runtime(mut runtime: ResMut<ThirdPersonCameraRuntimeActive>) {
    runtime.0 = true;
}

fn deactivate_runtime(mut runtime: ResMut<ThirdPersonCameraRuntimeActive>) {
    runtime.0 = false;
}

fn runtime_is_active(runtime: Res<ThirdPersonCameraRuntimeActive>) -> bool {
    runtime.0
}

#[cfg(test)]
#[path = "plugin_tests.rs"]
mod plugin_tests;
