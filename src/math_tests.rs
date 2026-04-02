use super::*;

#[test]
fn wrap_angle_keeps_values_in_pi_range() {
    let wrapped = wrap_angle(std::f32::consts::TAU * 2.5);
    assert!((-std::f32::consts::PI..=std::f32::consts::PI).contains(&wrapped));
}

#[test]
fn shortest_angle_delta_prefers_smallest_arc() {
    let delta = shortest_angle_delta(3.0, -3.0);
    assert!(delta.abs() < 0.4);
}

#[test]
fn camera_pose_points_back_from_forward_vector() {
    let pose = camera_pose_from_look_target(Vec3::ZERO, 0.0, 0.0, 4.0);
    assert_eq!(pose.look_target, Vec3::ZERO);
    assert!((pose.desired_camera_position - Vec3::new(0.0, 0.0, 4.0)).length() < 0.001);
}

#[test]
fn segment_aabb_hit_detects_front_face() {
    let hit = segment_aabb_hit(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, -5.0),
        Vec3::new(-1.0, -1.0, -1.0),
        Vec3::new(1.0, 1.0, 1.0),
    )
    .expect("segment should hit");
    assert!(hit.point.z > 0.9);
    assert_eq!(hit.normal, Vec3::Z);
}

#[test]
fn yaw_from_direction_ignores_vertical_only_vectors() {
    assert!(yaw_from_direction(Vec3::Y).is_none());
    assert!(yaw_from_direction(Vec3::new(1.0, 0.0, 0.0)).is_some());
}
