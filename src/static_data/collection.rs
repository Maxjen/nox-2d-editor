use serde::{Serialize, Deserialize};
use crate::static_data::*;

#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Collection {
    pub scene_nodes: Vec<SceneNode>,
    pub meshes: Vec<MeshData>,
    pub rigid_bodies: Vec<RigidBody>,
    pub colliders: Vec<Collider>,

    #[serde(skip)]
    pub name: String,

    #[serde(skip)]
    pub scene_node_indices: HashMap<String, usize>,

    #[serde(skip)]
    pub mesh_indices: HashMap<String, usize>,

    #[serde(skip)]
    pub rigid_body_indices: HashMap<String, usize>,

    #[serde(skip)]
    pub collider_indices: HashMap<String, usize>,
}

impl Collection {
    pub fn initialize(&mut self, name: String) {
        self.name = name;
        for (i, scene_node) in self.scene_nodes.iter().enumerate() {
            self.scene_node_indices.insert(scene_node.name.clone(), i);
        }
        for (i, mesh) in self.meshes.iter().enumerate() {
            self.mesh_indices.insert(mesh.mesh_name.clone(), i);
        }
        for (i, rigid_body) in self.rigid_bodies.iter().enumerate() {
            self.rigid_body_indices.insert(rigid_body.name.clone(), i);
        }
        for (i, collider) in self.colliders.iter().enumerate() {
            self.collider_indices.insert(collider.name.clone(), i);
        }
        
        for i in 0..self.scene_nodes.len() {
            if let Some(parent_name) = self.scene_nodes[i].get_parent() {
                let parent_index = self.scene_node_indices.get(parent_name).unwrap();
                self.scene_nodes[*parent_index].children.push(i);
            }
            self.scene_nodes[i].components.sort_by(scene::compare_components);
        }
    }
}
