use legion::*;
use std::{
    sync::Arc,
    collections::HashMap,
};
use crate::asset::handle::Handle;

pub struct Assets<T> {
    assets: HashMap<u64, (T, Arc<u64>)>,
}

impl<T> Assets<T> {
    pub fn new() -> Self {
        Assets {
            assets: HashMap::new(),
        }
    }

    pub fn add(&mut self, asset: T, id: u64) -> Handle<T> {
        if self.assets.contains_key(&id) {
            self.assets.get_mut(&id).unwrap().0 = asset;
            return Handle::new(self.assets.get(&id).unwrap().1.clone());
        }
        self.assets.insert(id, (asset, Arc::new(id)));
        Handle::new(self.assets.get(&id).unwrap().1.clone())
    }

    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        self.assets.get(&*handle.id).and_then(|x| Some(&x.0))
    }

    #[allow(dead_code)]
    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        self.assets.get_mut(&*handle.id).and_then(|x| Some(&mut x.0))
    }

    pub fn get_handle(&self, id: u64) -> Option<Handle<T>> {
        if self.assets.contains_key(&id) {
            return Some(Handle::new(self.assets.get(&id).unwrap().1.clone()));
        }
        None
    }
}

#[system]
pub fn remove_unused_assets<T>(#[resource] assets: &mut Assets<T>)
where
    T: 'static,
{
    let mut to_remove = Vec::new();
    for asset in &assets.assets {
        if Arc::strong_count(&asset.1.1) == 1 {
            to_remove.push(*asset.0);
        }
    }
    for id in &to_remove {
        assets.assets.remove(id);
    }
}
