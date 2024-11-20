use crate::components::Protagonist;
use crate::resources::{Animations, SCENES};


use bevy::{
    animation::{RepeatAnimation},
    prelude::*,
};

use avian3d::prelude::*;
use std::time::Duration;

pub fn keyboard_animation_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut impulse_query: Query<&mut ExternalImpulse, With<Protagonist>>,
    mut protagonist_query: Query<&mut Transform, With<Protagonist>>,
    mut velocity_query: Query<&mut LinearVelocity, With<Protagonist>>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animations: Res<Animations>,
    mut current_animation: Local<usize>,
) {
    let turn_speed = 2.0 * time.delta_seconds(); // Rotation speed (radians per second)
    let move_speed = 5.0; // Units per second

    if let Ok(mut protagonist_transform) = protagonist_query.get_single_mut() {
        for (mut player, mut transitions) in &mut animation_players {

            // Handle turning left (A)
            if keyboard_input.pressed(KeyCode::KeyA) {
                protagonist_transform.rotate(Quat::from_rotation_y(turn_speed));

                if keyboard_input.pressed(KeyCode::KeyW) == false {
                    let turn_left_animation = *SCENES.get("TURN_LEFT").unwrap();
                    transitions
                        .play(
                            &mut player,
                            animations.animations[turn_left_animation],
                            Duration::from_millis(250),
                        )
                        .set_repeat(RepeatAnimation::Count(1));
                }
            }

            // Handle turning right (D)
            if keyboard_input.pressed(KeyCode::KeyD) {
                protagonist_transform.rotate(Quat::from_rotation_y(-turn_speed));

                if keyboard_input.pressed(KeyCode::KeyW) == false {
                    let turn_right_animation = *SCENES.get("TURN_RIGHT").unwrap();
                    transitions
                        .play(
                            &mut player,
                            animations.animations[turn_right_animation],
                            Duration::from_millis(250),
                        )
                        .set_repeat(RepeatAnimation::Forever);
                }
            }

            // Handle forward movement (W)
            if keyboard_input.just_pressed(KeyCode::KeyW) {
                let advance = *SCENES.get("LEFT_SHOULDER_ADVANCE").unwrap();
                info!("Advance!");
                transitions
                    .play(
                        &mut player,
                        animations.animations[advance],
                        Duration::from_millis(250),
                    )
                    .set_repeat(RepeatAnimation::Forever);
            }

            if keyboard_input.pressed(KeyCode::KeyW) {
                // Calculate forward vector based on rotation
                let forward = protagonist_transform.rotation * Vec3::Z; // Forward in local space
                let forward_direction = -forward.normalize(); // Negate Z to move forward in world space

                for mut linear_velocity in velocity_query.iter_mut() {
                    linear_velocity.0 = forward_direction * move_speed; // Apply velocity in the forward direction
                }
            }

            // Stop movement when W is released
            if keyboard_input.just_released(KeyCode::KeyW) {
                let crouch = *SCENES.get("CROUCH").unwrap();
                transitions
                    .play(
                        &mut player,
                        animations.animations[crouch],
                        Duration::from_millis(250),
                    )
                    .set_repeat(RepeatAnimation::Count(1));

                for mut linear_velocity in velocity_query.iter_mut() {
                    linear_velocity.0 = Vec3::ZERO; // Stop the player
                }
            }

            // Jump (Space)
            if keyboard_input.just_pressed(KeyCode::Space) {
                let crouch = *SCENES.get("CROUCH").unwrap();
                transitions
                    .play(
                        &mut player,
                        animations.animations[crouch],
                        Duration::from_millis(250),
                    )
                    .set_repeat(RepeatAnimation::Count(1));
            }

            if keyboard_input.just_released(KeyCode::Space) {
                let legs_up = *SCENES.get("LEGS_UP").unwrap();

                transitions
                    .play(
                        &mut player,
                        animations.animations[legs_up],
                        Duration::from_millis(250),
                    )
                    .set_repeat(RepeatAnimation::Count(1));

                for mut impulse in impulse_query.iter_mut() {
                    impulse.apply_impulse(Vec3::new(0.0, 1.0, 0.0));
                }
            }

            // Switch animations with Tab
            if keyboard_input.just_pressed(KeyCode::Tab) {
                *current_animation = (*current_animation + 1) % animations.animations.len();
                let key = SCENES
                    .iter()
                    .find_map(|(k, &v)| if v == *current_animation { Some(k) } else { None });
                println!(
                    "Scene {}: {}",
                    *current_animation,
                    key.unwrap_or(&"* DISCARDED ANIMATION*")
                );

                transitions
                    .play(
                        &mut player,
                        animations.animations[*current_animation],
                        Duration::from_millis(250),
                    )
                    .repeat();
            }
        }
    }
}