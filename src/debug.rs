use bevy::{color::palettes::css, gizmos::prelude::*, prelude::*};

use crate::{ThirdPersonCamera, ThirdPersonCameraDebug, ThirdPersonCameraRuntime};

pub(crate) fn draw_debug_gizmos(
    mut gizmos: Gizmos,
    cameras: Query<(&ThirdPersonCameraDebug, &ThirdPersonCameraRuntime), With<ThirdPersonCamera>>,
) {
    for (debug, runtime) in &cameras {
        if !debug.enabled {
            continue;
        }

        if debug.draw_pivot {
            gizmos.sphere(runtime.pivot, 0.08, css::AQUA);
            gizmos.sphere(runtime.look_target, 0.06, css::YELLOW);
        }
        if debug.draw_desired {
            gizmos.line(
                runtime.look_target,
                runtime.desired_camera_position,
                css::ORANGE_RED,
            );
            gizmos.sphere(runtime.desired_camera_position, 0.06, css::ORANGE);
        }
        if debug.draw_corrected {
            gizmos.line(
                runtime.look_target,
                runtime.corrected_camera_position,
                css::LIMEGREEN,
            );
            gizmos.sphere(runtime.corrected_camera_position, 0.06, css::LIME);
        }
        if debug.draw_hits {
            if let Some(hit) = runtime.last_hit_point {
                gizmos.sphere(hit, 0.08, css::RED);
                gizmos.arrow(hit, hit + runtime.last_hit_normal * 0.5, css::RED);
            }
        }
    }
}
