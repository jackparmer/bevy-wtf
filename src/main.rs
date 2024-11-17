//! Loads and renders a glTF file as a scene.

use bevy::{
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};
use std::f32::consts::*;

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, animate_light_direction)
        .add_systems(Update, rotate_camera)
        .run();
}

fn rotate_camera(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    if keys.pressed(KeyCode::KeyQ) || keys.pressed(KeyCode::KeyE) {
        info!("Q or E pressed");
        for mut transform in query.iter_mut() {
            // Rotate the camera around the Y-axis
            let mut rotation_speed = 1.0; // Rotation speed in radians per second
            
            // Reverse rotation for W key
            if keys.pressed(KeyCode::KeyE) {
                rotation_speed *= -1.0;
            }

            let rotation_angle = rotation_speed * time.delta_seconds();
            let rotation = Quat::from_rotation_y(rotation_angle);

            // Apply the rotation
            transform.translation = rotation * transform.translation;

            // Keep the camera looking at the origin
            transform.look_at(Vec3::ZERO, Vec3::Y);
        }        
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.7, 0.7, 10.0)
                .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
            ..default()
        },
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 250.0,
        },
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.0, 0.0),
            ..default()
        },
        // This is a relatively small scene, so use tighter shadow
        // cascade bounds than the default for better quality.
        // We also adjusted the shadow map to be larger since we're
        // only using a single cascade.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .into(),
        ..default()
    });
    commands.spawn(SceneBundle {
        scene: asset_server
            //.load(GltfAssetLabel::Scene(0).from_asset("models/FlightHelmetLowPoly/FlightHelmetLowPoly.gltf")),
            .load(GltfAssetLabel::Scene(0).from_asset("models/ProtagonistLowPoly/ProtagonistLowPoly.glb")),
        ..default()
    });
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