use bevy::prelude::*;
use saddle_camera_third_person_camera::{
    ThirdPersonCamera, ThirdPersonCameraDebug, ThirdPersonCameraInputTarget,
    ThirdPersonCameraLockOn, ThirdPersonCameraLockOnTarget, ThirdPersonCameraObstacle,
    ThirdPersonCameraRuntime, ThirdPersonCameraSettings, ThirdPersonCameraTarget,
    default_input_bindings,
};
use saddle_pane::prelude::*;

#[derive(Component)]
pub struct DemoOverlay;

#[derive(Component, Clone, Copy)]
pub enum DemoMotionPath {
    Circle {
        center: Vec3,
        radius: f32,
        speed: f32,
        phase: f32,
    },
    Corridor {
        center: Vec3,
        half_length: f32,
        speed: f32,
    },
    Hover {
        center: Vec3,
        amplitude: f32,
        speed: f32,
    },
}

#[derive(Component)]
pub struct DemoTarget {
    pub motion: DemoMotionPath,
    pub face_velocity: bool,
}

#[derive(Resource, Pane)]
#[pane(title = "Third Person Camera", position = "top-right")]
pub struct ThirdPersonPane {
    #[pane(tab = "Orbit", slider, min = 0.2, max = 4.0, step = 0.05)]
    pub yaw_speed: f32,
    #[pane(tab = "Orbit", slider, min = 0.2, max = 4.0, step = 0.05)]
    pub pitch_speed: f32,
    #[pane(tab = "Framing", slider, min = 2.0, max = 8.0, step = 0.1)]
    pub default_distance: f32,
    #[pane(tab = "Framing", slider, min = 0.0, max = 2.0, step = 0.05)]
    pub shoulder_offset: f32,
    #[pane(tab = "Framing", slider, min = 0.0, max = 3.0, step = 0.05)]
    pub shoulder_height: f32,
    #[pane(tab = "Framing", slider, min = -1.5, max = 0.5, step = 0.05)]
    pub aim_height_offset: f32,
    #[pane(tab = "Framing", slider, min = 0.0, max = 1.0, step = 0.05)]
    pub target_radius_clearance: f32,
    #[pane(tab = "Framing")]
    pub screen_framing_enabled: bool,
    #[pane(tab = "Framing", slider, min = 0.0, max = 0.8, step = 0.01)]
    pub dead_zone_x: f32,
    #[pane(tab = "Framing", slider, min = 0.0, max = 0.8, step = 0.01)]
    pub dead_zone_y: f32,
    #[pane(tab = "Framing", slider, min = 0.05, max = 0.95, step = 0.01)]
    pub soft_zone_x: f32,
    #[pane(tab = "Framing", slider, min = 0.05, max = 0.95, step = 0.01)]
    pub soft_zone_y: f32,
    #[pane(tab = "Lock On")]
    pub lock_on_enabled: bool,
    #[pane(tab = "Lock On", slider, min = 4.0, max = 60.0, step = 0.5)]
    pub lock_on_max_distance: f32,
    #[pane(tab = "Runtime", monitor)]
    pub obstruction_distance: f32,
    #[pane(tab = "Runtime", monitor)]
    pub obstruction_active: bool,
    #[pane(tab = "Runtime", monitor)]
    pub aim_blend: f32,
    #[pane(tab = "Runtime", monitor)]
    pub shoulder_blend: f32,
    #[pane(tab = "Runtime", monitor)]
    pub lock_target: String,
}

impl Default for ThirdPersonPane {
    fn default() -> Self {
        let settings = ThirdPersonCameraSettings::default();
        Self {
            yaw_speed: settings.orbit.yaw_speed,
            pitch_speed: settings.orbit.pitch_speed,
            default_distance: settings.zoom.default_distance,
            shoulder_offset: settings.framing.shoulder_offset,
            shoulder_height: settings.framing.shoulder_height,
            aim_height_offset: settings.framing.aim_height_offset,
            target_radius_clearance: settings.framing.target_radius_clearance,
            screen_framing_enabled: settings.screen_framing.enabled,
            dead_zone_x: settings.screen_framing.dead_zone.x,
            dead_zone_y: settings.screen_framing.dead_zone.y,
            soft_zone_x: settings.screen_framing.soft_zone.x,
            soft_zone_y: settings.screen_framing.soft_zone.y,
            lock_on_enabled: settings.lock_on.enabled,
            lock_on_max_distance: settings.lock_on.max_distance,
            obstruction_distance: 0.0,
            obstruction_active: false,
            aim_blend: 0.0,
            shoulder_blend: 0.0,
            lock_target: "None".into(),
        }
    }
}

pub fn pane_plugins() -> (
    bevy_flair::FlairPlugin,
    bevy_input_focus::InputDispatchPlugin,
    bevy_ui_widgets::UiWidgetsPlugins,
    bevy_input_focus::tab_navigation::TabNavigationPlugin,
    saddle_pane::PanePlugin,
) {
    (
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        saddle_pane::PanePlugin,
    )
}

pub fn add_debug_pane(app: &mut App) {
    app.add_plugins(pane_plugins())
        .register_pane::<ThirdPersonPane>()
        .add_systems(Update, sync_pane_to_camera);
}

pub fn spawn_reference_world(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    title: &str,
    instructions: &str,
    accent: Color,
) {
    commands.spawn((
        Name::new("Demo Sun"),
        DirectionalLight {
            illuminance: 28_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 18.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Name::new("Demo Fill"),
        PointLight {
            intensity: 90_000.0,
            range: 60.0,
            ..default()
        },
        Transform::from_xyz(-6.0, 8.0, 10.0),
    ));
    commands.spawn((
        Name::new("Demo Floor"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(64.0, 64.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.09, 0.10, 0.13),
            perceptual_roughness: 1.0,
            ..default()
        })),
        ThirdPersonCameraObstacle::default(),
    ));

    commands.spawn(spawn_obstacle(
        meshes,
        materials,
        "Left Wall",
        Vec3::new(-2.6, 1.6, -8.0),
        Vec3::new(0.35, 3.2, 26.0),
        Color::srgb(0.34, 0.20, 0.18),
    ));
    commands.spawn(spawn_obstacle(
        meshes,
        materials,
        "Right Wall",
        Vec3::new(2.6, 1.6, -8.0),
        Vec3::new(0.35, 3.2, 26.0),
        Color::srgb(0.14, 0.18, 0.28),
    ));
    commands.spawn(spawn_obstacle(
        meshes,
        materials,
        "Low Beam",
        Vec3::new(0.0, 2.65, -3.5),
        Vec3::new(5.0, 0.45, 1.0),
        accent,
    ));

    for index in 0..5 {
        let z = -2.5 - index as f32 * 4.5;
        commands.spawn(spawn_obstacle(
            meshes,
            materials,
            format!("Corridor Pillar {}", index + 1),
            Vec3::new(if index % 2 == 0 { -1.55 } else { 1.55 }, 1.2, z),
            Vec3::new(0.65, 2.4, 0.65),
            if index % 2 == 0 {
                accent
            } else {
                Color::srgb(0.26, 0.28, 0.34)
            },
        ));
    }

    commands.spawn((
        Name::new("Demo Overlay"),
        DemoOverlay,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(18.0),
            width: Val::Px(430.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.82)),
        Text::new(format!("{title}\n{instructions}")),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

pub fn spawn_target(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: impl Into<String>,
    color: Color,
    translation: Vec3,
    motion: DemoMotionPath,
) -> Entity {
    commands
        .spawn((
            Name::new(name.into()),
            DemoTarget {
                motion,
                face_velocity: true,
            },
            ThirdPersonCameraLockOnTarget::default(),
            Mesh3d(meshes.add(Capsule3d::new(0.45, 1.2).mesh().rings(10).latitudes(14))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.04,
                perceptual_roughness: 0.26,
                ..default()
            })),
            Transform::from_translation(translation),
            GlobalTransform::from(Transform::from_translation(translation)),
        ))
        .id()
}

pub fn spawn_box_obstacle(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: impl Into<String>,
    translation: Vec3,
    size: Vec3,
    color: Color,
) -> Entity {
    commands
        .spawn(spawn_obstacle(
            meshes,
            materials,
            name,
            translation,
            size,
            color,
        ))
        .id()
}

pub fn spawn_camera(
    commands: &mut Commands,
    name: impl Into<String>,
    target: Entity,
    eye: Vec3,
    look_at: Vec3,
    camera: ThirdPersonCamera,
    settings: ThirdPersonCameraSettings,
    debug: bool,
) -> Entity {
    let mut entity = commands.spawn((
        Name::new(name.into()),
        Camera3d::default(),
        Transform::from_translation(eye).looking_at(look_at, Vec3::Y),
        camera,
        settings,
        ThirdPersonCameraTarget::new(target),
        ThirdPersonCameraInputTarget,
        default_input_bindings(),
    ));
    if debug {
        entity.insert(ThirdPersonCameraDebug::default());
    }
    entity.id()
}

pub fn animate_targets(time: Res<Time>, mut targets: Query<(&DemoTarget, &mut Transform)>) {
    for (target, mut transform) in &mut targets {
        let previous = transform.translation;
        match target.motion {
            DemoMotionPath::Circle {
                center,
                radius,
                speed,
                phase,
            } => {
                let angle = phase + time.elapsed_secs() * speed;
                transform.translation =
                    center + Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);
            }
            DemoMotionPath::Corridor {
                center,
                half_length,
                speed,
            } => {
                let t = (time.elapsed_secs() * speed).sin();
                transform.translation = center + Vec3::new(0.0, 0.0, t * half_length);
            }
            DemoMotionPath::Hover {
                center,
                amplitude,
                speed,
            } => {
                transform.translation =
                    center + Vec3::new(0.0, (time.elapsed_secs() * speed).sin() * amplitude, 0.0);
            }
        }

        if target.face_velocity {
            let velocity = transform.translation - previous;
            let flat = Vec2::new(velocity.x, velocity.z);
            if flat.length_squared() > 0.0001 {
                transform.look_to(velocity.normalize_or_zero(), Vec3::Y);
            }
        }
    }
}

fn sync_pane_to_camera(
    mut pane: ResMut<ThirdPersonPane>,
    mut cameras: Query<
        (
            &mut ThirdPersonCamera,
            &mut ThirdPersonCameraSettings,
            &ThirdPersonCameraRuntime,
            &ThirdPersonCameraLockOn,
        ),
        With<ThirdPersonCamera>,
    >,
) {
    let Some((mut camera, mut settings, runtime, lock_on)) = cameras.iter_mut().next() else {
        return;
    };
    let pane_added = pane.is_added();

    if pane_added {
        let pane = pane.bypass_change_detection();
        pane.yaw_speed = settings.orbit.yaw_speed;
        pane.pitch_speed = settings.orbit.pitch_speed;
        pane.default_distance = settings.zoom.default_distance;
        pane.shoulder_offset = settings.framing.shoulder_offset;
        pane.shoulder_height = settings.framing.shoulder_height;
        pane.aim_height_offset = settings.framing.aim_height_offset;
        pane.target_radius_clearance = settings.framing.target_radius_clearance;
        pane.screen_framing_enabled = settings.screen_framing.enabled;
        pane.dead_zone_x = settings.screen_framing.dead_zone.x;
        pane.dead_zone_y = settings.screen_framing.dead_zone.y;
        pane.soft_zone_x = settings.screen_framing.soft_zone.x;
        pane.soft_zone_y = settings.screen_framing.soft_zone.y;
        pane.lock_on_enabled = settings.lock_on.enabled;
        pane.lock_on_max_distance = settings.lock_on.max_distance;
    }

    if pane.is_changed() && !pane_added {
        settings.orbit.yaw_speed = pane.yaw_speed;
        settings.orbit.pitch_speed = pane.pitch_speed;
        settings.zoom.default_distance = pane.default_distance;
        settings.framing.shoulder_offset = pane.shoulder_offset;
        settings.framing.shoulder_height = pane.shoulder_height;
        settings.framing.aim_height_offset = pane.aim_height_offset;
        settings.framing.target_radius_clearance = pane.target_radius_clearance;
        settings.screen_framing.enabled = pane.screen_framing_enabled;
        settings.screen_framing.dead_zone = Vec2::new(pane.dead_zone_x, pane.dead_zone_y);
        settings.screen_framing.soft_zone = Vec2::new(
            pane.soft_zone_x.max(pane.dead_zone_x),
            pane.soft_zone_y.max(pane.dead_zone_y),
        );
        settings.lock_on.enabled = pane.lock_on_enabled;
        settings.lock_on.max_distance = pane.lock_on_max_distance;
        camera.target_distance = pane.default_distance;
    }

    let pane = pane.bypass_change_detection();
    pane.obstruction_distance = runtime.obstruction_distance;
    pane.obstruction_active = runtime.obstruction_active;
    pane.aim_blend = runtime.aim_blend;
    pane.shoulder_blend = runtime.shoulder_blend;
    pane.lock_target = lock_on
        .active_target
        .map(|entity| format!("{}", entity.to_bits()))
        .unwrap_or_else(|| "None".into());
}

fn spawn_obstacle(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: impl Into<String>,
    translation: Vec3,
    size: Vec3,
    color: Color,
) -> (
    Name,
    Mesh3d,
    MeshMaterial3d<StandardMaterial>,
    Transform,
    ThirdPersonCameraObstacle,
) {
    (
        Name::new(name.into()),
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.88,
            ..default()
        })),
        Transform::from_translation(translation),
        ThirdPersonCameraObstacle::default(),
    )
}
