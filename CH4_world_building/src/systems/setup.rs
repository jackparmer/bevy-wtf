use crate::components::Protagonist;
use crate::resources::Animations;
use crate::systems::portal::{TopPortalSensor, BottomPortalSensor};

use avian3d::prelude::*;
use bevy::{
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
};

use rand::Rng;

pub fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {

    // Build the animation graph
    let mut graph = AnimationGraph::new();
    const PROTAGONIST_ANIMATIONS: usize = 44;
    let animations = graph
        .add_clips(
            (0..=PROTAGONIST_ANIMATIONS)
                .map(|i| GltfAssetLabel::Animation(i).from_asset("models/ProtagonistLowPoly/Protagonist.glb"))
                .map(|path| asset_server.load(path)), // Map to load the asset
            1.0,
            graph.root,
        )
        .collect();

    // Insert a resource with the current scene information
    let graph = graphs.add(graph);
    commands.insert_resource(Animations {
        animations,
        graph: graph.clone(),
    });

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
            illuminance: 10.0,
            shadows_enabled: true,
            color: Color::srgb(0.2, 0.2, 0.3),
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

    
    let floor_depth_scale = 0.9;
    let max_floor_layer_count = 50.0;
    let floor_material = materials.add(StandardMaterial {
        perceptual_roughness: 0.9,
        metallic: 0.1,
        base_color_texture: Some(asset_server.load("textures/8k_mars.png")),
        ..default()
    });


    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(90.0, 0.2, 90.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(90.0, 0.2, 90.0)),
            material: floor_material,
            ..default()
        },
        Name::new("Floor"),
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(90.0, 0.2, 90.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(90.0, 0.2, 90.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/ice_texture2.png")),
                metallic: 1.0,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -0.2, 0.0),
            ..default()
        },
        Name::new("SubFloor"),
    ));    

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(90.0, 0.2, 90.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(90.0, 0.2, 90.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/ice_texture2.png")),
                metallic: 1.0,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -80.0, 0.0),
            ..default()
        },
        Name::new("AquifierFloor"),
    ));

    commands.spawn((
        RigidBody::Static,
        ColliderConstructor::TrimeshFromMesh,
        PbrBundle {
            mesh: meshes.add(Extrusion::new(Annulus::new(5.0, 10.0), 10.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/ice_texture3.png")),
                metallic: 0.0,
                ..default()
            }),
            transform: Transform::from_xyz(-10.0, -3.5, 10.0)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ..default()
        },
    ));

    // Top sensor circle
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(5.0, 1.0),
        Sensor,
        TopPortalSensor,
        PbrBundle {
            mesh: meshes.add(Cylinder { 
                radius: 5.0,
                half_height: 0.1,
            }),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/star_well.png")),
                base_color: Color::srgba(0.1, 0.1, 0.3, 0.9),
                metallic: 0.0,
                perceptual_roughness: 1.0,
                unlit: true,
                ..default()
            }),
            transform: Transform::from_xyz(-10.0, 0.6, 10.0),
            ..default()
        },
    ));

    // Bottom sensor circle
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(5.0, 1.0),
        Sensor,
        BottomPortalSensor,
        PbrBundle {
            mesh: meshes.add(Cylinder {
                radius: 5.0,
                half_height: 0.1,
            }),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/star_well.png")),
                base_color: Color::srgba(0.3, 0.1, 0.1, 0.9),
                metallic: 0.0,
                perceptual_roughness: 1.0,
                unlit: true,
                ..default()
            }),
            transform: Transform::from_xyz(-10.0, -5.4, 10.0),
            ..default()
        },
    ));

    // Starship

    let mut rng_starship = rand::thread_rng(); // Random number generator

    for _ in 0..1 {
        // Generate a random position at least 30 units away from the center
        let distance = rng_starship.gen_range(15.0..30.0); // Distance between 15 and 30 units
        let angle = rng_starship.gen_range(0.0..std::f32::consts::TAU); // Random angle in radians
        let x = distance * angle.cos();
        let z = distance * angle.sin();
    
        // Generate a random rotation
        let rotation = Quat::from_rotation_y(rng_starship.gen_range(0.0..std::f32::consts::TAU)); // Random rotation around the Y-axis
    
        // Spawn the entity
        commands.spawn((
            SceneBundle {
                scene: asset_server
                    //.load(GltfAssetLabel::Scene(0).from_asset("models/industrial_building.glb")),
                    .load(GltfAssetLabel::Scene(0).from_asset("models/starhopper.glb")),
                transform: Transform::from_xyz(x, -2.0, z).with_rotation(rotation),
                ..default()
            },
            // ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
            RigidBody::Static,
        ));
    }


    let mut rng_glacier = rand::thread_rng(); // Random number generator

    for _ in 0..3 {
        // Generate a random position at least 30 units away from the center
        let distance = rng_glacier.gen_range(30.0..50.0); // Distance between 30 and 50 units
        let y = rng_glacier.gen_range(0.0..90.0) * -1.0; // Random height between 0 and 90 units
        let angle = rng_glacier.gen_range(0.0..std::f32::consts::TAU); // Random angle in radians
        let x = distance * angle.cos();
        let z = distance * angle.sin();
    
        // Generate a random rotation
        let rotation = Quat::from_rotation_y(rng_glacier.gen_range(0.0..std::f32::consts::TAU)); // Random rotation around the Y-axis
    
        // Spawn the entity
        commands.spawn((
            SceneBundle {
                scene: asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("python/Tall_Monolithic_Rock.glb")),
                transform: Transform::from_xyz(x, y, z).with_rotation(rotation),
                ..default()
            },
            // ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
            RigidBody::Static,
        ));
    }    


    // North wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(100.0, 100.0, 20.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(100.0, 100.0, 20.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/concrete.png")),
                // base_color: Color::rgb(0.0, 0.0, 1.0), // Blue color
                metallic: 1.0,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -40.0, -50.0),
            ..default()
        },
    ));

    // South wall 
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(100.0, 100.0, 20.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(100.0, 100.0, 20.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/concrete.png")),
                // base_color: Color::rgb(0.0, 0.0, 1.0), // Solid color
                metallic: 1.0,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -40.0, 50.0),
            ..default()
        },
    ));

    // East wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(20.0, 95.0, 80.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(20.0, 95.0, 80.0)),
            material: materials.add(StandardMaterial {
                // base_color: Color::rgb(0.0, 1.0, 1.0), // Green color
                base_color_texture: Some(asset_server.load("textures/concrete.png")),
                metallic: 1.0,
                ..default()
            }),
            transform: Transform::from_xyz(40.0, -40.0, 0.0),
            ..default()
        },
    ));

    // West wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(20.0, 95.0, 80.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(20.0, 95.0, 80.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/concrete.png")),
                metallic: 1.0,
                ..default()
            }),
            transform: Transform::from_xyz(-40.0, -40.0, 0.0),
            ..default()
        },
    ));

    // Add ramp along west wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(5.0, 1.0, 80.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(5.0, 1.0, 80.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/ice_texture3.png")),
                metallic: 0.5,
                perceptual_roughness: 0.7,
                ..default()
            }),
            // Position and rotate the ramp
            transform: Transform::from_xyz(30.0, 0.0, 2.0)
                .with_rotation(Quat::from_rotation_x(-0.2)), // About 23 degrees incline
            ..default()
        },
    ));

    // GLTF Protagonist

    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 0.25, 1.0),
        // AngularVelocity(Vec3::new(2.5, 3.5, 1.5)), 
        ExternalImpulse::default(), // Add ExternalImpulse for jumping
        Protagonist,                // Marker component for the Protagonist        
        SceneBundle {       
            scene: asset_server
                .load(GltfAssetLabel::Scene(0)
                .from_asset("models/ProtagonistLowPoly/Protagonist.glb")),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        },        
    ));

    // Load the stars texture
    let stars_texture_handle = asset_server.load("textures/8k_stars.png");
    
    // Create a material with the stars texture
    let sky_material = materials.add(StandardMaterial {
        base_color_texture: Some(stars_texture_handle),
        unlit: true, // Make sure it's unlit so it glows like the sky
        ..default()
    });

    // Spawn the sky dome (large sphere surrounding the scene)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere {
                radius: 1000.0,   // Large enough to enclose the scene
            }),
            material: sky_material,
            transform: Transform::from_translation(Vec3::ZERO)
                .with_scale(Vec3::new(-1.0, 1.0, 1.0)), // Invert normals by scaling on one axis
            ..default()
        },
        Name::new("SkyDome"),
    ));

    // Add large cylindrical terrain with Mars texture
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(1000.0, 1.0),
        PbrBundle {
            mesh: meshes.add(Cylinder {
                radius: 1000.0,
                half_height: 0.5,
            }),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/8k_mars.png")),
                perceptual_roughness: 0.9,
                metallic: 0.1,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -90.0, 0.0),
            ..default()
        },
    ));

    // Random generator
    let mut rng = rand::thread_rng();

    // Metal shipping containers
    let metal_texture_handle = asset_server.load("textures/container_metal.png");

    // Create a material with the stars texture
    let metal_material = materials.add(StandardMaterial {
        base_color_texture: Some(metal_texture_handle),
        metallic: 1.0,
        ..default()
    });

    // Add random metal containers
    for _ in 0..50 {
        let random_position = Vec3::new(
            rng.gen_range(-300.0..300.0),
            rng.gen_range(0.0..50.0),
            rng.gen_range(-300.0..300.0),
        );

        // Ensure the position isn't near the protagonist's start
        if random_position.distance(Vec3::new(0.0, 1.0, 0.0)) > 50.0 {
            commands.spawn((
                RigidBody::Dynamic,
                Collider::cuboid(8.0, 3.0, 3.0),
                PbrBundle {
                    mesh: meshes.add(Cuboid::new(8.0, 3.0, 3.0)),
                    material: materials.add(StandardMaterial {
                        base_color_texture: Some(asset_server.load("textures/container_metal.png")),
                        metallic: 1.0,
                        ..default()
                    }),
                    transform: Transform::from_translation(random_position)
                        .with_rotation(Quat::from_euler(
                            EulerRot::XYZ,
                            rng.gen_range(0.0..std::f32::consts::PI),
                            rng.gen_range(0.0..std::f32::consts::PI),
                            rng.gen_range(0.0..std::f32::consts::PI),
                        )),
                    ..default()
                },
            ));
        }
    }

    // Add invisible floor
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(90.0, 0.2, 90.0),
        Name::new("InvisibleFloor"),
    )).insert(Transform::from_xyz(0.0, -5.2, 0.0));  // 5 units below SubFloor

}