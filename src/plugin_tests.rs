use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use crate::{ThirdPersonCamera, ThirdPersonCameraPlugin};

#[derive(States, Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
enum DemoState {
    #[default]
    Active,
}

#[test]
fn plugin_registers_runtime_defaults() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin));
    app.init_state::<DemoState>();
    app.add_plugins(ThirdPersonCameraPlugin::new(
        OnEnter(DemoState::Active),
        OnExit(DemoState::Active),
        Update,
    ));

    app.world_mut().spawn((
        ThirdPersonCamera::default(),
        Name::new("Plugin Test Camera"),
    ));
    app.update();

    let mut query = app.world_mut().query::<(
        &crate::ThirdPersonCameraRuntime,
        &crate::ThirdPersonCameraInput,
        &crate::ThirdPersonCameraSettings,
        &Name,
    )>();
    assert_eq!(query.iter(app.world()).count(), 1);
}
