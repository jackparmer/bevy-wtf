use bevy::prelude::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;


#[derive(Resource)]
pub struct Animations {
    pub animations: Vec<AnimationNodeIndex>,
    #[allow(dead_code)]
    pub graph: Handle<AnimationGraph>,
}

// Define the global `scenes` variable
pub static SCENES: Lazy<HashMap<&'static str, usize>> = Lazy::new(|| {
    [
        ("STRAFE_JOG_RIGHT",0),
        ("ADVANCE", 1),
        ("TURN_RIGHT", 2),
        ("STRAFE_RIGHT", 6),
        ("STRAFE_LEFT", 7),
        ("PIVOT_RIGHT", 13),
        ("HEARD_SOUND", 14),
        ("WALK_BACK", 16),
        ("DEATH", 16),
        ("LEGS_UP", 18),
        ("LOOK_AROUND", 19),
        ("TREAD", 20),
        ("JOG_BACK", 22),
        ("SIDE_STEP_LEFT", 23),
        ("TURN_LEFT", 24),
        ("IDLE_STRETCH", 25),
        ("CRAWL", 28),
        ("QUARTER_LEFT", 29),
        ("JUMP_LAND", 30),
        ("LEFT_SHOULDER_ADVANCE", 31),
        ("FLY", 32),
        ("CROUCH", 33),
        ("SIDE_STEP_RIGHT", 34),
        ("CRAWL_BACKWARDS", 35),
        ("STRAFE_JOG_LEFT", 36), 
        ("CLIMB", 37),
        ("JUMP", 38),
        ("TPOSE", 39),
        ("TRACK_JUMP", 40), 
        ("SWIM", 41),   
        ("QUARTER_RIGHT", 42),
        ("IDLE_FALL", 44), 
    ]
    .iter()
    .cloned()
    .collect()
});