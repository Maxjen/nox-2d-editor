mod collection;
mod scene;
mod mesh;
mod physics;

pub use collection::*;
pub use scene::*;
pub use mesh::*;
pub use physics::*;

use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File, DirEntry},
    path::Path,
};
use ron::de::from_reader;

pub struct DataAccessor {
    pub collections: Vec<Collection>,
    pub collection_indices: HashMap<String, usize>,
}

impl DataAccessor {
    pub fn new() -> Self {
        Self {
            collections: Vec::new(),
            collection_indices: HashMap::new(),
        }
    }

    pub fn load_collection(&mut self, path: &Path) {
        let file = File::open(path).unwrap();
        let mut collection: Collection = match from_reader(file) {
            Ok(x) => x,
            Err(e) => {
                println!("Failed to load ron: {}", e);
                std::process::exit(1);
            }
        };
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        collection.initialize(name.clone());
        self.collection_indices.insert(name, self.collections.len());
        self.collections.push(collection);
    }

    pub fn load_collections_in_directory(&mut self, path: &Path) {
        if path.is_dir() {
            let mut entries = fs::read_dir(path)
                .unwrap()
                .into_iter()
                .collect::<Result<Vec<DirEntry>, _>>()
                .unwrap();
            entries.sort_by(|a, b| a.path().cmp(&b.path()));

            for entry in entries {
                if let Some(extension) = entry.path().extension() {
                    if extension == OsStr::new("ron") {
                        self.load_collection(&entry.path())
                    }
                }
            }
        }
    }

    pub fn get_source_collection_and_component_name(&self, component_path: &String, collection_to_spawn_in: usize) -> Option<(usize, String)> {
        if let Some(index) = component_path.find('/') {
            let (collection_name, component_name) = component_path.split_at(index);
            let component_name = component_name.strip_prefix('/').unwrap().to_string();
            if let Some(index) = self.collection_indices.get(collection_name) {
                return Some((*index, component_name));
            }
            return None;
        }
        Some((collection_to_spawn_in, component_path.clone()))
    }

    fn get_node(&self, name: &String, collection_to_spawn_in: usize) -> Option<(&SceneNode, usize)> {
        if let Some((source_collection, node_name)) = self.get_source_collection_and_component_name(name, collection_to_spawn_in) {
            let collection = &self.collections[source_collection];
            if let Some(node_index) = collection.scene_node_indices.get(&node_name) {
                if let Some(node) = collection.scene_nodes.get(*node_index) {
                    return Some((node, source_collection));
                }
            }
        }
        None
    }

    pub fn build_node(&self, name: &String, collection_to_spawn_in: usize) -> Option<(SceneNode, Vec<(usize, usize)>)> {
        if let Some((mut node, children)) = self.build_prefab_node(name, collection_to_spawn_in, collection_to_spawn_in) {
            let top_node = self.get_node(name, collection_to_spawn_in).unwrap().0;
            node.name = top_node.name.clone();
            if let Some(parent) = top_node.get_parent() {
                node.set_parent(parent.clone());
            }
            node.components.sort_by(scene::compare_components);
            return Some((node, children));
        }
        None
    }

    fn build_prefab_node(&self, name: &String, collection_to_spawn_in: usize, prefab_collection: usize) -> Option<(SceneNode, Vec<(usize, usize)>)> {
        if let Some((node, collection)) = self.get_node(name, prefab_collection) {
            let (mut new_node, mut children) = if !node.prefab.is_empty() {
                self.build_prefab_node(&node.prefab, collection_to_spawn_in, collection).unwrap()
            }
            else {
                (SceneNode::default(), Vec::new())
            };
            new_node.name = name.clone();
            for component in &node.components {
                let adjust_name = |name: &String| {
                    if collection == collection_to_spawn_in {
                        name.clone()
                    }
                    else {
                        let mut prefixed_name = self.collections.get(collection).unwrap().name.clone();
                        prefixed_name.push('/');
                        prefixed_name.push_str(&name);
                        prefixed_name
                    }
                };
                match component {
                    Component::Transform { translation, rotation } => new_node.set_transform(&translation, *rotation),
                    Component::Mesh(name) => new_node.set_mesh(adjust_name(name)),
                    Component::RigidBody(name) => new_node.set_rigid_body(adjust_name(name)),
                    Component::Collider(name) => new_node.set_collider(adjust_name(name)),
                    _ => {}
                }
            }
            for child in &node.children {
                children.push((*child, collection));
            }
            return Some((new_node, children));
        }
        None
    }

    pub fn get_mesh(&self, name: &String, collection_to_spawn_in: usize) -> Option<&MeshData> {
        if let Some((source_collection, mesh_name)) = self.get_source_collection_and_component_name(name, collection_to_spawn_in) {
            let collection = &self.collections[source_collection];
            if let Some(mesh_index) = collection.mesh_indices.get(&mesh_name) {
                return collection.meshes.get(*mesh_index);
            }
        }
        None
    }

    pub fn get_rigid_body(&self, name: &String, collection_to_spawn_in: usize) -> Option<&RigidBody> {
        if let Some((source_collection, rigid_body_name)) = self.get_source_collection_and_component_name(name, collection_to_spawn_in) {
            let collection = &self.collections[source_collection];
            if let Some(rigid_body_index) = collection.rigid_body_indices.get(&rigid_body_name) {
                return collection.rigid_bodies.get(*rigid_body_index);
            }
        }
        None
    }

    pub fn get_collider(&self, name: &String, collection_to_spawn_in: usize) -> Option<&Collider> {
        if let Some((source_collection, collider_name)) = self.get_source_collection_and_component_name(name, collection_to_spawn_in) {
            let collection = &self.collections[source_collection];
            if let Some(collider_index) = collection.collider_indices.get(&collider_name) {
                return collection.colliders.get(*collider_index);
            }
        }
        None
    }
}

/*fn main() -> std::io::Result<()> {
    let mut file = File::open("data/meshes.ron")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let deserialized: Scene = serde_json::from_str(&content)?;
    println!("{:?}", deserialized.meshes[0]);

    let mut file2 = File::create("data/meshes.ron").unwrap();
    let pretty = PrettyConfig::default();
    let ron_string = to_string_pretty(&deserialized, pretty).unwrap();
    file2.write(ron_string.as_bytes()).unwrap();

    Ok(())
}*/
