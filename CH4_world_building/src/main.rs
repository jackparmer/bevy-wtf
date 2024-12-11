mod systems;
mod components;
mod resources;

use crate::components::Protagonist;
use crate::resources::{Animations, SCENES};

use systems::portal::portal_system;
use systems::setup::setup;
use systems::camera::rotate_camera;
use systems::input::{
    keyboard_animation_control, 
    blink_lights, 
    handle_temporary_lights,
};

use avian3d::prelude::*;
use bevy::{
    animation::animate_targets,
    pbr::DirectionalLightShadowMap,
    prelude::*,
};

use std::f32::consts::*;
use std::time::Duration;


fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        // Enable physics
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, animate_light_direction)
        .add_systems(Update, rotate_camera)
        .add_systems(Update, setup_scene_once_loaded.before(animate_targets))
        .add_systems(Update, keyboard_animation_control)
        .add_systems(Update, reset_game_on_command_r) // Add reset system
        .add_systems(Update, update_gravity_and_light)
        .add_systems(Update, portal_system) 
        .add_systems(Update, blink_lights)
        .add_systems(Update, handle_temporary_lights)        
        .run();
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * PI / 5.0,
            -FRAC_PI_4,
        );
    }
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
        let stretch = *SCENES.get("IDLE_STRETCH").unwrap();
        info!("starting pose: {}", stretch);
        // Make sure to start the animation via the `AnimationTransitions`
        // component. The `AnimationTransitions` component wants to manage all
        // the animations and will get confused if the animations are started
        // directly via the `AnimationPlayer`.
        transitions
            .play(&mut player, 
                animations.animations[stretch], 
                Duration::from_millis(1000),
            ).repeat();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }
}

/// System to reset the game when Command-R or Ctrl-R is pressed
fn reset_game_on_command_r(
    mut commands: Commands,
    camera_query: Query<Entity, With<Camera>>,
    protagonist_query: Query<Entity, With<Protagonist>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Check for Command-R (Mac) or Ctrl-R (Other systems)
    let is_command_r = keyboard_input.pressed(KeyCode::SuperLeft) && keyboard_input.just_pressed(KeyCode::KeyR);
    let is_ctrl_r = keyboard_input.pressed(KeyCode::ControlLeft) && keyboard_input.just_pressed(KeyCode::KeyR);

    if is_command_r || is_ctrl_r {
        println!("Resetting the game...");

        for entity in camera_query.iter() {
            println!("Despawning camera entity: {:?}", entity);
            commands.entity(entity).despawn();
        }

        for entity in protagonist_query.iter() {
            commands.entity(entity).despawn();
        }        

        // Re-run the setup logic to reinitialize the game
        setup(commands, asset_server, meshes, materials, graphs);
    }
}

fn update_gravity_and_light(
    mut gravity: ResMut<Gravity>,
    protagonist_query: Query<&Transform, With<Protagonist>>,
    mut light_query: Query<&mut DirectionalLight>,
) {
    if let Ok(protagonist_transform) = protagonist_query.get_single() {
        let y_position = protagonist_transform.translation.y;

        // Update gravity
        let outside_walls = protagonist_transform.translation.x > 40.0 || 
                          protagonist_transform.translation.x < -40.0 ||
                          protagonist_transform.translation.z > 50.0 ||
                          protagonist_transform.translation.z < -50.0;
                          
        let underwater = protagonist_transform.translation.y < 0.0 && 
                        protagonist_transform.translation.x >= -40.0 &&
                        protagonist_transform.translation.x <= 40.0 &&
                        protagonist_transform.translation.z >= -50.0 &&
                        protagonist_transform.translation.z <= 50.0;

        if outside_walls || (underwater == false) {
            gravity.0 = Vec3::new(0.0, -9.81, 0.0); // Enable gravity outside walls and above water
        } else {
            gravity.0 = Vec3::ZERO; // Disable gravity inside walls when underwater
        }

        // Update light color
        for mut directional_light in light_query.iter_mut() {
            directional_light.color = if y_position > 0.0 {
                Color::srgb(1.0, 0.0, 0.0) // Original red
            } else {
                Color::srgb(0.24, 0.62, 0.92) // Blue below the y-plane
            };
        }
    }
}