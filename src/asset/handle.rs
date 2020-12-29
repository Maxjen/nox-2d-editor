use std::{
    marker::PhantomData,
    sync::Arc,
};

#[derive(Debug)]
pub struct Handle<T> {
    pub id: Arc<u64>,
    _marker: PhantomData<T>,
}

impl<T> Handle<T> {
    pub fn new(id: Arc<u64>) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}
