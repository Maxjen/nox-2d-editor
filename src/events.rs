use legion::*;

pub struct Events<T> {
    pub events: Vec<T>,
}

impl<T> Default for Events<T> {
    fn default() -> Self {
        Self {
            events: Vec::new(),
        }
    }
}

impl<T> Events<T> {
    pub fn send(&mut self, event: T) {
        self.events.push(event);
    }
}

#[system]
pub fn clear_events<T>(#[resource] events: &mut Events::<T>)
where
    T: 'static,
{
    events.events.clear();
}
