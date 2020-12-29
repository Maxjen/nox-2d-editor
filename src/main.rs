#[macro_use]
extern crate log;

use application::Application;

mod application;
mod wgpu_state;
mod app_state;
mod asset;
mod events;
mod camera;
mod mesh_pipeline;
mod mesh;
mod texture;
mod loader;

fn main() {
    let application = Application::new();
    application.run();
}
