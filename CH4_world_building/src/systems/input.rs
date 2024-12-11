use crate::components::Protagonist;
use crate::resources::{Animations, SCENES};


use bevy::{
    animation::RepeatAnimation,
    prelude::*,
    pbr::PointLight,
};

use avian3d::prelude::*;
use std::time::Duration;
use bevy::pbr::StandardMaterial;  // Add these imports

// Move constants outside the function scope
const CHARGE_LIGHT_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);  // Bright red color
const CHARGE_LIGHT_INTENSITY: f32 = 100000.0;  // Increased intensity
const CHARGE_LIGHT_RANGE: f32 = 20.0;  // Add range for the light
const CHARGE_LIGHT_RADIUS: f32 = 0.2;  // Add physical size to the light

// Add mesh constants
const CHARGE_MESH_RADIUS: f32 = 0.1;  // Hockey puck radius (adjust as needed)
const CHARGE_MESH_HEIGHT: f32 = 0.05;  // Hockey puck height

// Add these constants near the other light constants
const BACKPACK_LIGHT_COLOR: Color = Color::srgb(1.0, 0.5, 0.0);  // Orange color
const BACKPACK_LIGHT_INTENSITY: f32 = 100000.0;  // Increased from 1000.0
const BACKPACK_LIGHT_RANGE: f32 = 20.0;  // Increased from 5.0

// Add this component
#[derive(Component)]
pub struct TemporaryLight {
    lifetime: Timer,
    initial_intensity: f32,
}

// Add this near the top with other components
#[derive(Component)]
pub struct FallingState {
    was_falling: bool,
}

pub fn keyboard_animation_control(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut impulse_query: Query<&mut ExternalImpulse, With<Protagonist>>,
    mut protagonist_query: Query<(&mut Transform, Has<Collider>), With<Protagonist>>,
    mut velocity_query: Query<&mut LinearVelocity, With<Protagonist>>,
    mut angular_velocity_query: Query<&mut AngularVelocity, With<Protagonist>>,
    mut directional_light_query: Query<&mut DirectionalLight>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animations: Res<Animations>,
    mut current_animation: Local<usize>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut falling_query: Query<&mut FallingState, With<Protagonist>>,
) {
    let turn_speed = 2.0 * time.delta_seconds(); // Rotation speed (radians per second)
    let move_speed = 5.0; // Units per second
    let run_speed = 10.0; // Running speed
    let strafe_speed = 4.0; // Strafing speed
    let underwater_speed = 8.0; // Underwater movement speed

    // Add wall boundaries
    const WALL_MIN_X: f32 = -25.0;  // Left wall
    const WALL_MAX_X: f32 = 25.0;   // Right wall
    const WALL_MIN_Z: f32 = -25.0;  // Front wall
    const WALL_MAX_Z: f32 = 25.0;   // Back wall
    const WATER_LEVEL: f32 = -1.0;  // Water level threshold

    const NORMAL_LIGHT_COLOR: Color = Color::srgb(0.2, 0.2, 0.3);
    const NORMAL_LIGHT_ILLUMINANCE: f32 = 10.0;
    const UNDERWATER_LIGHT_COLOR: Color = Color::srgb(0.0, 0.2, 0.8); // Deep blue color
    const UNDERWATER_LIGHT_ILLUMINANCE: f32 = 50000.0; // Dimmer underwater

    if let Ok((mut protagonist_transform, is_colliding)) = protagonist_query.get_single_mut() {
        // Extract only Y rotation and force upright orientation
        let (yaw, _, _) = protagonist_transform.rotation.to_euler(EulerRot::YXZ);
        protagonist_transform.rotation = Quat::from_rotation_y(yaw);
        
        // Check if character is both below water level AND within walls
        let is_underwater = protagonist_transform.translation.y < WATER_LEVEL
            && protagonist_transform.translation.x > WALL_MIN_X
            && protagonist_transform.translation.x < WALL_MAX_X
            && protagonist_transform.translation.z > WALL_MIN_Z
            && protagonist_transform.translation.z < WALL_MAX_Z;

        // info!("Position: {:?}, Is Underwater: {}", protagonist_transform.translation, is_underwater);
        
        for (mut player, mut transitions) in &mut animation_players {
            if is_underwater {
                // Play SWIM animation when moving forward/backward or TREAD when moving up/down
                if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::KeyS) {
                    let swim = *SCENES.get("SWIM").unwrap();
                    if player.animation(animations.animations[swim]).is_none() {
                        transitions
                            .play(
                                &mut player,
                                animations.animations[swim],
                                Duration::from_millis(250),
                            )
                            .set_repeat(RepeatAnimation::Forever);
                        player.resume_all();
                    }
                } else {
                    // Default to TREAD animation when not swimming forward/backward
                    let tread = *SCENES.get("TREAD").unwrap();
                    if player.animation(animations.animations[tread]).is_none() {
                        transitions
                            .play(
                                &mut player,
                                animations.animations[tread],
                                Duration::from_millis(250),
                            )
                            .set_repeat(RepeatAnimation::Forever);
                        player.resume_all();
                    }
                }
            } else {
                // Check if falling
                if let (Ok(velocity), Ok(mut falling_state)) = (velocity_query.get_single(), falling_query.get_single_mut()) {
                    let is_falling = velocity.0.y < -0.1 && !is_colliding;  // Only consider falling when not touching ground
                    
                    // Play fly animation when falling
                    if is_falling {
                        let fly = *SCENES.get("FLY").unwrap();
                        if !player.is_playing_animation(animations.animations[fly]) {
                            transitions
                                .play(
                                    &mut player,
                                    animations.animations[fly],
                                    Duration::from_millis(250),
                                )
                                .set_repeat(RepeatAnimation::Forever);
                        }
                        
                        // Prevent movement controls while in air
                        for mut linear_velocity in velocity_query.iter_mut() {
                            linear_velocity.0.x = 0.0;
                            linear_velocity.0.z = 0.0;
                        }
                    }
                    
                    falling_state.was_falling = is_falling;
                }
            }

            // Handle turning left (A)
            if keyboard_input.pressed(KeyCode::KeyA) {
                // Apply pure Y-axis rotation
                protagonist_transform.rotation = Quat::from_rotation_y(yaw + turn_speed);

                if !is_underwater && !keyboard_input.pressed(KeyCode::KeyW) && !keyboard_input.pressed(KeyCode::KeyS) {
                    let pivot_left = *SCENES.get("PIVOT_RIGHT").unwrap();
                    transitions
                        .play(
                            &mut player,
                            animations.animations[pivot_left],
                            Duration::from_millis(250),
                        )
                        .set_repeat(RepeatAnimation::Forever);
                }
            }

            if keyboard_input.just_released(KeyCode::KeyA) {
                if !is_underwater && !keyboard_input.pressed(KeyCode::KeyW) && !keyboard_input.pressed(KeyCode::KeyS) {
                    let crouch = *SCENES.get("CROUCH").unwrap();
                    transitions
                        .play(
                            &mut player,
                            animations.animations[crouch],
                            Duration::from_millis(250),
                        )
                        .set_repeat(RepeatAnimation::Count(1));
                }
            }

            // Handle turning right (D)
            if keyboard_input.pressed(KeyCode::KeyD) {
                // Apply pure Y-axis rotation
                protagonist_transform.rotation = Quat::from_rotation_y(yaw - turn_speed);

                if !is_underwater && !keyboard_input.pressed(KeyCode::KeyW) && !keyboard_input.pressed(KeyCode::KeyS) {
                    let pivot_right = *SCENES.get("PIVOT_RIGHT").unwrap();
                    transitions
                        .play(
                            &mut player,
                            animations.animations[pivot_right],
                            Duration::from_millis(250),
                        )
                        .set_repeat(RepeatAnimation::Forever);
                }
            }

            if keyboard_input.just_released(KeyCode::KeyD) {
                if !is_underwater && !keyboard_input.pressed(KeyCode::KeyW) && !keyboard_input.pressed(KeyCode::KeyS) {
                    let crouch = *SCENES.get("CROUCH").unwrap();
                    transitions
                        .play(
                            &mut player,
                            animations.animations[crouch],
                            Duration::from_millis(250),
                        )
                        .set_repeat(RepeatAnimation::Count(1));
                }
            }

            // Handle strafing left (Q)
            if keyboard_input.just_pressed(KeyCode::KeyQ) {
                if !is_underwater {
                    let strafe_left = *SCENES.get("STRAFE_LEFT").unwrap();
                    transitions
                        .play(
                            &mut player,
                            animations.animations[strafe_left],
                            Duration::from_millis(250),
                        )
                        .set_repeat(RepeatAnimation::Forever);
                }
            }

            if keyboard_input.pressed(KeyCode::KeyQ) {
                let right = protagonist_transform.rotation * Vec3::X;
                let strafe_direction = -Vec3::new(right.x, 0.0, right.z).normalize();
                
                for mut linear_velocity in velocity_query.iter_mut() {
                    let current_y = linear_velocity.0.y;
                    linear_velocity.0 = strafe_direction * strafe_speed;
                    linear_velocity.0.y = current_y;
                }
            }

            if keyboard_input.just_released(KeyCode::KeyQ) {
                if !is_underwater {
                    let crouch = *SCENES.get("CROUCH").unwrap();
                    transitions
                        .play(
                            &mut player,
                            animations.animations[crouch],
                            Duration::from_millis(250),
                        )
                        .set_repeat(RepeatAnimation::Count(1));
                }
                
                for mut linear_velocity in velocity_query.iter_mut() {
                    linear_velocity.0 = Vec3::ZERO;
                }
            }

            // Handle strafing right (E)
            if keyboard_input.just_pressed(KeyCode::KeyE) {
                if !is_underwater {
                    let strafe_right = *SCENES.get("STRAFE_RIGHT").unwrap();
                    transitions
                        .play(
                            &mut player,
                            animations.animations[strafe_right],
                            Duration::from_millis(250),
                        )
                        .set_repeat(RepeatAnimation::Forever);
                }
            }

            if keyboard_input.pressed(KeyCode::KeyE) {
                let right = protagonist_transform.rotation * Vec3::X;
                let strafe_direction = Vec3::new(right.x, 0.0, right.z).normalize();
                
                for mut linear_velocity in velocity_query.iter_mut() {
                    let current_y = linear_velocity.0.y;
                    linear_velocity.0 = strafe_direction * strafe_speed;
                    linear_velocity.0.y = current_y;
                }
            }

            if keyboard_input.just_released(KeyCode::KeyE) {
                if !is_underwater {
                    let crouch = *SCENES.get("CROUCH").unwrap();
                    transitions
                        .play(
                            &mut player,
                            animations.animations[crouch],
                            Duration::from_millis(250),
                        )
                        .set_repeat(RepeatAnimation::Count(1));
                }
                
                for mut linear_velocity in velocity_query.iter_mut() {
                    linear_velocity.0 = Vec3::ZERO;
                }
            }

            // Handle forward movement (W)
            if keyboard_input.just_pressed(KeyCode::KeyW) {
                if !is_underwater {
                    let advance = *SCENES.get("LEFT_SHOULDER_ADVANCE").unwrap();
                    transitions
                        .play(
                            &mut player,
                            animations.animations[advance],
                            Duration::from_millis(250),
                        )
                        .set_repeat(RepeatAnimation::Forever);
                } else {
                    let swim = *SCENES.get("SWIM").unwrap();
                    transitions
                        .play(
                            &mut player,
                            animations.animations[swim],
                            Duration::from_millis(250),
                        )
                        .set_repeat(RepeatAnimation::Forever);
                }
            }

            if keyboard_input.pressed(KeyCode::KeyW) {
                if !is_underwater {
                    // Calculate forward vector based on rotation, but remove vertical component
                    let forward = protagonist_transform.rotation * Vec3::Z;
                    let forward_flat = Vec3::new(forward.x, 0.0, forward.z).normalize();
                    let forward_direction = -forward_flat;
                    let current_speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
                        run_speed
                    } else {
                        move_speed
                    };

                    for mut linear_velocity in velocity_query.iter_mut() {
                        // Preserve existing vertical velocity (for gravity)
                        let current_y = linear_velocity.0.y;
                        linear_velocity.0 = forward_direction * current_speed;
                        linear_velocity.0.y = current_y;
                    }
                } else {
                    // Underwater vertical movement
                    for mut linear_velocity in velocity_query.iter_mut() {
                        if keyboard_input.pressed(KeyCode::ShiftLeft) {
                            let forward = protagonist_transform.rotation * Vec3::Z;
                            let forward_direction = -forward.normalize();
                            linear_velocity.0 = forward_direction * underwater_speed;
                        } else {
                            linear_velocity.0 = Vec3::new(0.0, underwater_speed, 0.0);
                        }
                    }
                }
            }

            if keyboard_input.just_released(KeyCode::KeyW) {
                if !is_underwater {
                    let crouch = *SCENES.get("CROUCH").unwrap();
                    transitions
                        .play(
                            &mut player,
                            animations.animations[crouch],
                            Duration::from_millis(250),
                        )
                        .set_repeat(RepeatAnimation::Count(1));
                }

                for mut linear_velocity in velocity_query.iter_mut() {
                    linear_velocity.0 = Vec3::ZERO;
                }
            }

            // Handle backward movement (S)
            if keyboard_input.just_pressed(KeyCode::KeyS) {
                if !is_underwater {
                    let walk_backwards = *SCENES.get("JOG_BACK").unwrap();
                    transitions
                        .play(
                            &mut player,
                            animations.animations[walk_backwards],
                            Duration::from_millis(250),
                        )
                        .set_repeat(RepeatAnimation::Forever);
                }
            }

            if keyboard_input.pressed(KeyCode::KeyS) {
                if !is_underwater {
                    let forward = protagonist_transform.rotation * Vec3::Z;
                    let backward_flat = Vec3::new(forward.x, 0.0, forward.z).normalize();
                    
                    for mut linear_velocity in velocity_query.iter_mut() {
                        // Preserve existing vertical velocity
                        let current_y = linear_velocity.0.y;
                        linear_velocity.0 = backward_flat * move_speed;
                        linear_velocity.0.y = current_y;
                    }
                } else {
                    // Existing underwater downward movement code
                    for mut linear_velocity in velocity_query.iter_mut() {
                        linear_velocity.0 = Vec3::new(0.0, -underwater_speed, 0.0);
                    }
                }
            }

            // Stop movement when S is released
            if keyboard_input.just_released(KeyCode::KeyS) {
                if !is_underwater {
                    let crouch = *SCENES.get("CROUCH").unwrap();
                    transitions
                        .play(
                            &mut player,
                            animations.animations[crouch],
                            Duration::from_millis(250),
                        )
                        .set_repeat(RepeatAnimation::Count(1));
                }

                for mut linear_velocity in velocity_query.iter_mut() {
                    linear_velocity.0 = Vec3::ZERO;
                }
            }

            // Jump (Space)
            if keyboard_input.just_pressed(KeyCode::Space) {
                let light_offset = Vec3::new(0.0, 2.0, 0.0);
                commands.spawn((
                    PointLightBundle {
                        point_light: PointLight {
                            color: BACKPACK_LIGHT_COLOR,
                            intensity: BACKPACK_LIGHT_INTENSITY,
                            range: BACKPACK_LIGHT_RANGE,
                            radius: 2.0,
                            shadows_enabled: true,
                            ..default()
                        },
                        transform: Transform::from_translation(protagonist_transform.translation + light_offset),
                        ..default()
                    },
                    TemporaryLight {
                        lifetime: Timer::from_seconds(1.0, TimerMode::Once),
                        initial_intensity: BACKPACK_LIGHT_INTENSITY,
                    },
                ));
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

            // Teleport the character 10 units down when V is pressed
            if keyboard_input.just_pressed(KeyCode::KeyV) {
                protagonist_transform.translation.y -= 10.0;
                info!("Teleported 10 units down!");
            }

            // Teleport the character 15 units up when B is pressed
            if keyboard_input.just_pressed(KeyCode::KeyB) {
                protagonist_transform.translation.y += 15.0;
                info!("Teleported 15 units up!");
            }

            // Toggle lighting with K for night and L for alarm
            if keyboard_input.just_pressed(KeyCode::KeyK) {
                for mut light in directional_light_query.iter_mut() {
                    // Switch to dark night mode
                    light.illuminance = 10.0;
                    light.color = Color::srgb(0.2, 0.2, 0.3); // Very dark blue night
                }
            }

            if keyboard_input.just_pressed(KeyCode::KeyL) {
                for mut light in directional_light_query.iter_mut() {
                    // Switch to red alarm lights
                    light.illuminance = 1000.0;
                    light.color = Color::srgb(1.0, 0.0, 0.0); // Bright red alarm light
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

            // Handle placing charge (C)
            if keyboard_input.just_pressed(KeyCode::KeyC) {
                info!("C key pressed, is_colliding: {}", is_colliding);  // Debug log

                // Remove the collision check temporarily for testing
                let light_position = protagonist_transform.translation + Vec3::new(0.0, 0.0, 0.0);
                
                info!("Placing charge at position: {:?}", light_position);  // Debug log
                
                // Fix the material creation with proper type conversion
                let charge_material = materials.add(StandardMaterial {
                    emissive: CHARGE_LIGHT_COLOR.into(),  // Convert Color to LinearRgba
                    base_color: Color::BLACK,
                    ..default()
                });

                // Create cylinder mesh
                let charge_mesh = meshes.add(Cylinder {
                    radius: CHARGE_MESH_RADIUS,
                    half_height: CHARGE_MESH_HEIGHT / 2.0,
                    ..default()
                });

                // Spawn the charge
                commands.spawn((
                    PbrBundle {
                        mesh: charge_mesh,
                        material: charge_material,
                        transform: Transform::from_translation(light_position),
                        ..default()
                    },
                    PointLight {
                        color: CHARGE_LIGHT_COLOR,
                        intensity: CHARGE_LIGHT_INTENSITY,
                        range: CHARGE_LIGHT_RANGE,
                        radius: CHARGE_LIGHT_RADIUS,
                        shadows_enabled: true,
                        ..default()
                    },
                    BlinkingLight {
                        timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                    },
                ));

                // Play crouch animation
                let crouch = *SCENES.get("CROUCH").unwrap();
                transitions
                    .play(
                        &mut player,
                        animations.animations[crouch],
                        Duration::from_millis(250),
                    )
                    .set_repeat(RepeatAnimation::Count(1));

                info!("Charge placed successfully");  // Debug log
            }
        }

        // Reset angular velocity if no rotation keys are pressed
        if !keyboard_input.pressed(KeyCode::KeyA) && !keyboard_input.pressed(KeyCode::KeyD) {
            for mut angular_velocity in angular_velocity_query.iter_mut() {
                angular_velocity.0 = Vec3::ZERO;
            }
        }

        // Update lighting based on underwater state
        for mut light in directional_light_query.iter_mut() {
            let in_water_environment = 
                protagonist_transform.translation.x > WALL_MIN_X
                && protagonist_transform.translation.x < WALL_MAX_X
                && protagonist_transform.translation.z > WALL_MIN_Z
                && protagonist_transform.translation.z < WALL_MAX_Z;

            if is_underwater {
                light.illuminance = UNDERWATER_LIGHT_ILLUMINANCE;
                light.color = UNDERWATER_LIGHT_COLOR;
            } else {
                light.illuminance = NORMAL_LIGHT_ILLUMINANCE;
                light.color = NORMAL_LIGHT_COLOR;
            }
        }
    }
}

// Add this component and system for blinking light functionality
#[derive(Component)]
pub struct BlinkingLight {
    timer: Timer,
}

pub fn blink_lights(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut lights: Query<(Entity, &mut PointLight, &mut BlinkingLight, &Handle<StandardMaterial>)>,
) {
    for (_, mut light, mut blink, material_handle) in &mut lights {
        blink.timer.tick(time.delta());
        if blink.timer.just_finished() {
            let is_on = light.intensity > 0.0;
            light.intensity = if is_on { 0.0 } else { CHARGE_LIGHT_INTENSITY };
            
            // Also update the material's emission
            if let Some(material) = materials.get_mut(material_handle) {
                material.emissive = if is_on { 
                    Color::BLACK.into() 
                } else { 
                    CHARGE_LIGHT_COLOR.into()
                };
            }
        }
    }
}

// Add this new system
pub fn handle_temporary_lights(
    mut commands: Commands,
    time: Res<Time>,
    mut lights: Query<(Entity, &mut PointLight, &mut TemporaryLight)>,
) {
    for (entity, mut light, mut temp) in &mut lights {
        temp.lifetime.tick(time.delta());
        let progress = 1.0 - temp.lifetime.fraction_remaining();
        light.intensity = temp.initial_intensity * progress;

        if temp.lifetime.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}