use bevy::prelude::*;

/// A single additive camera effect layer with translation, rotation offsets, and
/// an optional FOV delta. Each layer has an independent weight and can be
/// enabled/disabled individually.
///
/// # Usage
///
/// External systems create and update layers via [`ThirdPersonCameraCustomEffects`]:
///
/// ```rust,ignore
/// fn update_breathing(
///     time: Res<Time>,
///     mut q: Query<&mut ThirdPersonCameraCustomEffects, With<ThirdPersonCamera>>,
/// ) {
///     for mut custom in &mut q {
///         let t = time.elapsed_secs();
///         custom.set("breathing", CameraEffectLayer::weighted(
///             Vec3::new(0.0, (t * 1.2).sin() * 0.003, 0.0),
///             Vec3::ZERO,
///             0.0,
///             1.0,
///         ));
///     }
/// }
/// ```
#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct CameraEffectLayer {
    /// Camera-local translation offset (right, up, forward).
    pub translation: Vec3,
    /// Camera-local rotation offset in radians (pitch, yaw, roll).
    pub rotation: Vec3,
    /// Additive FOV delta in radians.
    pub fov_delta: f32,
    /// Blend weight for this layer (0 = disabled, 1 = full).
    pub weight: f32,
    /// Whether this layer is active.
    pub enabled: bool,
}

impl CameraEffectLayer {
    /// Create a weighted effect layer.
    pub fn weighted(translation: Vec3, rotation: Vec3, fov_delta: f32, weight: f32) -> Self {
        Self {
            translation,
            rotation,
            fov_delta,
            weight,
            enabled: true,
        }
    }
}

impl Default for CameraEffectLayer {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Vec3::ZERO,
            fov_delta: 0.0,
            weight: 1.0,
            enabled: true,
        }
    }
}

/// A named wrapper around [`CameraEffectLayer`] for storage in the custom
/// effects list.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct NamedEffectLayer {
    pub name: String,
    pub layer: CameraEffectLayer,
}

/// The composited result of all active effect layers.
#[derive(Reflect, Debug, Clone, Default, PartialEq)]
pub struct CameraEffectStack {
    pub translation: Vec3,
    pub rotation: Vec3,
    pub fov_delta: f32,
}

impl CameraEffectStack {
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Vec3::ZERO,
        fov_delta: 0.0,
    };

    pub fn add_layer(&mut self, layer: &CameraEffectLayer) {
        if !layer.enabled || layer.weight <= 0.0 {
            return;
        }
        self.translation += layer.translation * layer.weight;
        self.rotation += layer.rotation * layer.weight;
        self.fov_delta += layer.fov_delta * layer.weight;
    }
}

/// Compose all effect layers into a single stack.
pub fn compose_effect_stack(layers: &[NamedEffectLayer]) -> CameraEffectStack {
    let mut stack = CameraEffectStack::default();
    for named in layers {
        stack.add_layer(&named.layer);
    }
    stack
}

/// Named effect layers that compose additively on the final camera transform.
///
/// Attach this component to a third-person camera entity and use [`set`](Self::set)
/// to insert effects from your own systems. The camera plugin composes all
/// enabled layers after computing the corrected position, applying the
/// combined translation and rotation offsets in camera-local space.
///
/// # Example
///
/// ```rust,ignore
/// fn hit_flinch(
///     mut q: Query<&mut ThirdPersonCameraCustomEffects, With<ThirdPersonCamera>>,
/// ) {
///     for mut custom in &mut q {
///         custom.set("flinch", CameraEffectLayer::weighted(
///             Vec3::ZERO,
///             Vec3::new(0.04, 0.02, 0.0),
///             0.0,
///             1.0,
///         ));
///     }
/// }
/// ```
#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct ThirdPersonCameraCustomEffects {
    pub layers: Vec<NamedEffectLayer>,
}

impl ThirdPersonCameraCustomEffects {
    /// Insert or replace a named layer. If a layer with the given name
    /// already exists it is updated in place; otherwise a new entry is
    /// appended.
    pub fn set(&mut self, name: impl Into<String>, layer: CameraEffectLayer) {
        let name = name.into();
        if let Some(existing) = self.layers.iter_mut().find(|l| l.name == name) {
            existing.layer = layer;
        } else {
            self.layers.push(NamedEffectLayer { name, layer });
        }
    }

    /// Remove a named layer, returning it if it existed.
    pub fn remove(&mut self, name: &str) -> Option<CameraEffectLayer> {
        if let Some(pos) = self.layers.iter().position(|l| l.name == name) {
            Some(self.layers.swap_remove(pos).layer)
        } else {
            None
        }
    }

    /// Get an immutable reference to a named layer.
    pub fn get(&self, name: &str) -> Option<&CameraEffectLayer> {
        self.layers.iter().find(|l| l.name == name).map(|l| &l.layer)
    }

    /// Get a mutable reference to a named layer.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut CameraEffectLayer> {
        self.layers
            .iter_mut()
            .find(|l| l.name == name)
            .map(|l| &mut l.layer)
    }

    /// Number of active (enabled) layers.
    pub fn active_count(&self) -> usize {
        self.layers.iter().filter(|l| l.layer.enabled).count()
    }

    /// Compose all active layers into a single effect stack.
    pub fn compose(&self) -> CameraEffectStack {
        compose_effect_stack(&self.layers)
    }

    /// Iterate over all layers.
    pub fn iter(&self) -> impl Iterator<Item = &NamedEffectLayer> {
        self.layers.iter()
    }
}
