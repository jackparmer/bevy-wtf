use crate::components::Protagonist;
use crate::resources::Animations;

use avian3d::prelude::*;
use bevy::{
    pbr::{
        CascadeShadowConfigBuilder, 
        FogSettings, 
        FogFalloff
    },
    prelude::*,
};

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

    // Add fog for a hazy effect

    commands.spawn(FogSettings {
        color: Color::srgba(0.2, 0.3, 0.35, 1.0), // Cool, dark fog color
        directional_light_color: Color::WHITE,  // Optional light scattering color
        directional_light_exponent: 15.0,      // How focused the directional light is
        falloff: FogFalloff::Exponential { density: 0.04 }, // Hazy exponential fog
    });

    // Static "floor"

    // Load the texture
    let texture_handle = asset_server.load("textures/ground_texture.png");

    // Create a material with the texture
    let textured_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        ..default()
    });

    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(400.0, 0.1),
        PbrBundle {
            mesh: meshes.add(Cylinder::new(400.0, 0.1)),
            material: textured_material,
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
}