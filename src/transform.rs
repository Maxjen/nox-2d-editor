use legion::*;
use crate::{
    app_state::AppState,
    hierarchy::Children,
};

#[derive(Clone, Debug)]
pub struct Transform2D {
    pub translation: glam::Vec3,
    pub rotation: f32,
}

impl Transform2D {
    pub fn new(translation: &glam::Vec3, rotation: f32) -> Self {
        Self {
            translation: translation.clone(),
            rotation,
        }
    }

    pub fn build_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_rotation_translation(
            glam::Quat::from_rotation_z(self.rotation.to_radians()),
            self.translation
        )
    }

    pub fn multiply(&self, transform: &Transform2D) -> Self {
        let mut rotation = ((self.rotation + transform.rotation) + 180.0) % 360.0;
        if rotation < 0.0 {
            rotation += 360.0;
        }
        rotation -= 180.0;

        let x = transform.translation.x * rotation.to_radians().cos() - transform.translation.y * rotation.to_radians().sin();
        let y = transform.translation.x * rotation.to_radians().sin() + transform.translation.y * rotation.to_radians().cos();
        let translation = self.translation + glam::Vec3::new(x, y, 0.0);

        Self { translation, rotation }
    }
}

pub struct LocalTransform(pub Transform2D);
pub struct GlobalTransform(pub Transform2D);

pub fn propagate_transforms(world: &mut World, resources: &mut Resources) {
    let app_state = resources.get::<AppState>().unwrap();

    /*let mut query =
        <(&mut GlobalTransform, &LocalTransform, &Children)>::query()
            .filter(maybe_changed::<LocalTransform>());
    
    for (global_transform, local_transform, children) in query.iter_mut(world) {
        global_transform.0 = local_transform.0.clone();
        for child in &children.0 {
            propagate_recursive(*child, &global_transform.0, world);
        }
    }*/

    let root_entities = app_state.root_entities.clone();
    for entity in &root_entities {
        let mut entry = world.entry(*entity).unwrap();
        let local_transform = entry.get_component::<LocalTransform>().unwrap().0.clone();
        let global_transform = entry.get_component_mut::<GlobalTransform>().unwrap();
        global_transform.0 = local_transform.clone();
        if let Ok(children) = entry.get_component::<Children>() {
            let children = children.0.clone();
            for child in children {
                propagate_recursive(child, local_transform.clone(), world);
            }
        }
    }
}

fn propagate_recursive(entity: Entity, parent_transform: Transform2D, world: &mut World) {
    let mut entry = world.entry(entity).unwrap();
    let transform = parent_transform.multiply(&entry.get_component::<LocalTransform>().unwrap().0);
    let global_transform = entry.get_component_mut::<GlobalTransform>().unwrap();
    global_transform.0 = transform.clone();

    if let Ok(children) = entry.get_component::<Children>() {
        let children = children.0.clone();
        for child in children {
            propagate_recursive(child, transform.clone(), world);
        }
    }
}
