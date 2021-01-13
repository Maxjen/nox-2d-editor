use legion::*;
use rapier2d::{
    na::Vector2,
    dynamics::{JointSet, RigidBodySet, IntegrationParameters},
    geometry::{BroadPhase, NarrowPhase, ColliderSet},
    pipeline::PhysicsPipeline,
};

use crate::transform::LocalTransform;

pub struct TimeSinceLastPhysicsUpdate(pub f32);

pub struct RigidBodyHandle(pub rapier2d::data::arena::Index);
pub struct ColliderHandle(pub rapier2d::data::arena::Index);

#[system]
pub fn update_physics(
    #[resource] time_since_last_physics_update: &mut TimeSinceLastPhysicsUpdate,
    #[resource] pipeline: &mut PhysicsPipeline,
    #[resource] integration_parameters: &IntegrationParameters,
    #[resource] broad_phase: &mut BroadPhase,
    #[resource] narrow_phase: &mut NarrowPhase,
    #[resource] rigid_body_set: &mut RigidBodySet,
    #[resource] collider_set: &mut ColliderSet,
    #[resource] joint_set: &mut JointSet,
) {
    let interval = 1.0 / 60.0;
    while time_since_last_physics_update.0 > interval {
        time_since_last_physics_update.0 -= interval;

        pipeline.step(
            &Vector2::new(0.0, -9.81),
            integration_parameters,
            broad_phase,
            narrow_phase,
            rigid_body_set,
            collider_set,
            joint_set,
            None,
            None,
            &(),
        );
    }
}

#[system(for_each)]
pub fn copy_transforms_from_rigid_bodies(
    transform: &mut LocalTransform,
    rigid_body_handle: &RigidBodyHandle,
    #[resource] rigid_body_set: &RigidBodySet,
) {
    let rigid_body = rigid_body_set.get(rigid_body_handle.0).unwrap();
    let position = rigid_body.position();
    transform.0.translation = glam::Vec3::new(position.translation.vector.x * 64.0, position.translation.vector.y * 64.0, 0.0);
    transform.0.rotation = position.rotation.angle().to_degrees();
}
