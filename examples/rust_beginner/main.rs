use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::ui::PrepareNextFrameMaterials;
use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;


fn main() {
    let json_file_path = "/Users/rqg/Playground/ProjectX/proj.android/Resources/Animations/barbarian_boss.ExportJson";

    let json_str = std::fs::read_to_string(json_file_path).unwrap();
    let armature: HashMap<String, Value> = serde_json::from_str(&json_str).unwrap();

    println!("{:?}", armature);
}
