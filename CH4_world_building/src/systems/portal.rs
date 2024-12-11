use bevy::prelude::*;
use avian3d::prelude::*;
use crate::components::Protagonist;

// Marker components for the sensors
#[derive(Component)]
pub struct TopPortalSensor;

#[derive(Component)]
pub struct BottomPortalSensor;

pub fn portal_system(
    mut collision_events: EventReader<CollisionStarted>,
    mut protagonist_query: Query<&mut Transform, With<Protagonist>>,
    top_sensor_query: Query<Entity, With<TopPortalSensor>>,
    bottom_sensor_query: Query<Entity, With<BottomPortalSensor>>,
) {
    for CollisionStarted(e1, e2) in collision_events.read() {
        let top_sensor = top_sensor_query.iter().next();
        let bottom_sensor = bottom_sensor_query.iter().next();
        
        if let Ok(mut transform) = protagonist_query.get_single_mut() {
            // Check top sensor collision
            if let Some(top_entity) = top_sensor {
                if *e1 == top_entity || *e2 == top_entity {
                    info!("Protagonist collided with top portal sensor at y={}", transform.translation.y);
                    // Teleport down into water
                    transform.translation.y -= 20.0;
                    info!("Teleported down to y={}", transform.translation.y);
                }
            }
            
            // Check bottom sensor collision
            if let Some(bottom_entity) = bottom_sensor {
                if *e1 == bottom_entity || *e2 == bottom_entity {
                    info!("Protagonist collided with bottom portal sensor at y={}", transform.translation.y);
                    // Teleport up and over
                    transform.translation.x += 5.0;
                    transform.translation.y += 10.0;
                    info!("Teleported up to y={}", transform.translation.y);
                }
            }
        }
    }
} 