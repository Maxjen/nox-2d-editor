#[macro_use]
extern crate log;

use application::Application;

mod application;
mod wgpu_state;
mod app_state;
mod asset;
mod events;
mod command;
mod camera;
mod mesh;
mod texture;
mod static_data;
mod hierarchy;
mod transform;
mod physics;
mod ui;

fn main() {
    let application = Application::new();
    application.run();
}
