use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RigidBody {
    pub name: String,
    pub status: RigidBodyStatus,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RigidBodyStatus {
    Static,
    Dynamic,
    Kinematic,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Collider {
    pub name: String,
    pub shape: Shape,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Shape {
    Cuboid(f32, f32),
    Blubb,
}
