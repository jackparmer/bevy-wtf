use avian3d::prelude::*;
use bevy::{
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};
use std::f32::consts::*;

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        // Enable physics
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, animate_light_direction)
        .add_systems(Update, rotate_camera)
        .add_systems(Update, protagonist_jump)
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

// Marker component for the protagonist
#[derive(Component)]
struct Protagonist;

// System to handle jumping
fn protagonist_jump(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut ExternalImpulse, With<Protagonist>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        for mut impulse in query.iter_mut() {
            impulse.apply_impulse(Vec3::new(0.0, 5.0, 0.0));
        }
    }
}

fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>) {

    // Add Camera and Ambient Lighting

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
        }
    ));

    // Add Directional Lighting

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

    // Static "floor"

    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(4.0, 0.1),
        PbrBundle {
            mesh: meshes.add(Cylinder::new(4.0, 0.1)),
            material: materials.add(Color::BLACK),
            ..default()
        },
    ));

    // GLTF Protagonist

    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 2.0, 1.0),
        AngularVelocity(Vec3::new(2.5, 3.5, 1.5)), 
        ExternalImpulse::default(), // Add ExternalImpulse for jumping
        Protagonist,                // Marker component for the Protagonist        
        SceneBundle {       
            scene: asset_server
                //.load(GltfAssetLabel::Scene(0).from_asset("models/FlightHelmetLowPoly/FlightHelmetLowPoly.gltf")),
                .load(GltfAssetLabel::Scene(0).from_asset("models/ProtagonistLowPoly/ProtagonistLowPoly.glb")),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        },        
    ));

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