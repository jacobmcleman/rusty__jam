use bevy::{prelude::*, render::camera::OrthographicProjection };

pub struct VisChecker {
    pub radius: f32,
    pub visible: bool,
}

pub struct VisDebug;

pub fn vis_checking_system(
    mut query: Query<(&mut VisChecker, &GlobalTransform)>,
    camera_query: Query<(&Transform, &OrthographicProjection), With<crate::MainCam>, >,
) {
    if let Ok((cam_transform, orthographic_projection)) = camera_query.single() {
        let left_edge = orthographic_projection.left + cam_transform.translation.x;
        let right_edge = orthographic_projection.right + cam_transform.translation.x;
        let top_edge = orthographic_projection.top + cam_transform.translation.y;
        let bottom_edge = orthographic_projection.bottom + cam_transform.translation.y;

        let corner_a = Vec2::new(left_edge, top_edge);
        let corner_b = Vec2::new(right_edge, bottom_edge);

        for (mut vis_check, transform) in query.iter_mut() {
            vis_check.visible = circle_intersect_rect(vis_check.radius, transform.translation.truncate(), corner_a, corner_b);
        }
    }
}

pub fn vis_debug_system(
    query: Query<(&VisChecker, &Transform), With<VisDebug>>
) {
    for (vis_check, transform) in query.iter() {
        println!("{} is {}", 
            transform.translation, 
            if vis_check.visible {"Visible"} else {"Not Visible"});
    }
}

// Given a center point and radius of a circle and two diagonally opposite corners of a rectangle
// Return true if the circle and rectangle overlap, false otherwise
fn circle_intersect_rect(r: f32, center: Vec2, corner_a: Vec2, corner_b: Vec2) -> bool{
    let mut test_x = center.x;
    let mut test_y = center.y;

    let min_x = f32::min(corner_a.x, corner_b.x);
    let max_x = f32::max(corner_a.x, corner_b.x);

    let min_y = f32::min(corner_a.y, corner_b.y);
    let max_y = f32::max(corner_a.y, corner_b.y);

    if center.x < min_x {
        test_x = min_x;
    }
    else if center.x > max_x {
        test_x = max_x;
    }

    if center.y < min_y {
        test_y = min_y;
    }
    else if center.y > max_y {
        test_y = max_y;
    }

    let dist_x = center.x - test_x;
    let dist_y = center.y - test_y;

    let distance_squared = (dist_x * dist_x) + (dist_y * dist_y);

    let intersects = distance_squared <= r * r;

    //println!("circle r {} at {}, intersect with rect between tl {} and br {} - {}", r, center, corner_a, corner_b, intersects);
    return intersects;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_rect_contained_circle() {
        assert_eq!(
            circle_intersect_rect(
                10.0, Vec2::new(0.0, 0.0),
                Vec2::new(10.0, 10.0), Vec2::new(-10.0, -10.0)),
            true
        );
    }

    #[test]
    fn test_circle_rect_contained_rect() {
        assert_eq!(
            circle_intersect_rect(
                50.0, Vec2::new(0.0, 0.0),
                Vec2::new(10.0, 10.0), Vec2::new(-10.0, -10.0)),
            true
        );
    }

    #[test]
    fn test_circle_rect_miss() {
        assert_eq!(
            circle_intersect_rect(
                10.0, Vec2::new(100.0, 0.0),
                Vec2::new(10.0, 10.0), Vec2::new(-10.0, -10.0)),
            false
        );
    }

    #[test]
    fn test_circle_rect_singlepoint_touch() {
        // Test all 4 edges
        assert_eq!(
            circle_intersect_rect(
                10.0, Vec2::new(20.0, 0.0),
                Vec2::new(10.0, 10.0), Vec2::new(-10.0, -10.0)),
            true
        );
        assert_eq!(
            circle_intersect_rect(
                10.0, Vec2::new(-20.0, 0.0),
                Vec2::new(10.0, 10.0), Vec2::new(-10.0, -10.0)),
            true
        );
        assert_eq!(
            circle_intersect_rect(
                10.0, Vec2::new(0.0, 20.0),
                Vec2::new(10.0, 10.0), Vec2::new(-10.0, -10.0)),
            true
        );
        assert_eq!(
            circle_intersect_rect(
                10.0, Vec2::new(0.0, -20.0),
                Vec2::new(10.0, 10.0), Vec2::new(-10.0, -10.0)),
            true
        );
    }
}