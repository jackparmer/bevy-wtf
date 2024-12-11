use bevy::prelude::*;
use crate::components::Protagonist;

pub fn rotate_camera(
    time: Res<Time>,
    protagonist_query: Query<&Transform, With<Protagonist>>, // Immutable access to protagonist
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Protagonist>)>, // Exclude protagonist
) {
    if let Ok(protagonist_transform) = protagonist_query.get_single() {
        let protagonist_position = protagonist_transform.translation;
        let protagonist_rotation = protagonist_transform.rotation;

        for mut camera_transform in camera_query.iter_mut() {
            // Define the desired offset for the camera (relative to the protagonist)
            let follow_offset = Vec3::new(0.0, 3.0, 10.0); // Slightly above and behind the protagonist

            // Calculate the new camera position by applying the protagonist's rotation to the offset
            let rotated_offset = protagonist_rotation * follow_offset;
            let new_camera_position = protagonist_position + rotated_offset;

            // Smoothly move the camera to the new position
            camera_transform.translation = camera_transform
                .translation
                .lerp(new_camera_position, time.delta_seconds() * 5.0); // Adjust lerp speed as needed

            // Ensure the camera is always looking at the protagonist
            camera_transform.look_at(protagonist_position, Vec3::Y);
        }
    }
}