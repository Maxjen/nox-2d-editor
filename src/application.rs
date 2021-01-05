use std::time::Instant;
use legion::*;
use physics::TimeSinceLastPhysicsUpdate;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use rapier2d::{
    dynamics::{JointSet, RigidBodySet, IntegrationParameters},
    geometry::{BroadPhase, NarrowPhase, ColliderSet},
    pipeline::PhysicsPipeline,
};
use crate::{
    wgpu_state::WgpuState,
    app_state,
    app_state::AppState,
    events,
    events::Events,
    camera,
    camera::Camera,
    asset,
    asset::Assets,
    mesh,
    texture::Texture,
    physics,
};

pub struct DeltaTime(pub f32);

pub struct Application {
    world: World,
    resources: Resources,
}

impl Application {
    pub fn new() -> Self {
        env_logger::init();

        Self {
            world: World::default(),
            resources: Resources::default(),
        }
    }

    fn setup_world(&mut self, window: &Window) {
        self.resources.insert(futures::executor::block_on(WgpuState::new(window)));
        self.resources.insert(AppState::new());
        self.resources.insert(Events::<winit::event::KeyboardInput>::default());
        self.resources.insert(Camera::new());
        self.resources.insert(Assets::<Texture>::new());
        self.resources.insert(DeltaTime(0.0));

        {
            self.resources.insert(PhysicsPipeline::new());
            self.resources.insert(IntegrationParameters::default());
            self.resources.insert(BroadPhase::new());
            self.resources.insert(NarrowPhase::new());
            self.resources.insert(RigidBodySet::new());
            self.resources.insert(ColliderSet::new());
            self.resources.insert(JointSet::new());
            self.resources.insert(physics::TimeSinceLastPhysicsUpdate(0.0));
        }

        let mesh_pipeline = {
            let wgpu_state = self.resources.get_mut::<WgpuState>().unwrap();
            mesh::Pipeline::new(&wgpu_state)
        };
        self.resources.insert(mesh_pipeline);

        {
            let mut wgpu_state = self.resources.get_mut::<WgpuState>().unwrap();
            let mut app_state = self.resources.get_mut::<AppState>().unwrap();
            let pipeline = self.resources.get::<mesh::Pipeline>().unwrap();
            let mut textures = self.resources.get_mut::<Assets<Texture>>().unwrap();
            let mut rigid_body_set = self.resources.get_mut::<RigidBodySet>().unwrap();
            let mut collider_set = self.resources.get_mut::<ColliderSet>().unwrap();
            let mut command_buffer = legion::systems::CommandBuffer::new(&self.world);
            //app_state.set_current_mesh_index(Some(0), &mut wgpu_state, &pipeline, &mut textures, &mut command_buffer);
            app_state.spawn_scene(&mut wgpu_state, &pipeline, &mut textures, &mut rigid_body_set, &mut collider_set, &mut command_buffer);
            command_buffer.flush(&mut self.world);
        }
    }

    pub fn run(mut self) {
        let mut schedule = Schedule::builder()
            .add_system(app_state::handle_input_system())
            .add_system(camera::update_camera_system())
            .add_system(physics::update_physics_system())
            .add_system(physics::copy_transforms_from_rigid_bodies_system())
            //.add_system(mesh_pipeline::update_camera_buffer_system())
            .add_system(mesh::render_meshes_system())
            .add_system(events::clear_events_system::<winit::event::KeyboardInput>())
            .add_system(asset::remove_unused_assets_system::<Texture>())
            .build();

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
            .build(&event_loop)
            .unwrap();
        
        self.setup_world(&window);

        let mut start = Instant::now();
        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {

                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => {
                            let mut keyboard_events = self.resources.get_mut::<Events::<KeyboardInput>>().unwrap();
                            keyboard_events.send(input.clone());
                        }
                        /*WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        },*/
                        WindowEvent::Resized(physical_size) => {
                            let mut wgpu_state = self.resources.get_mut::<WgpuState>().unwrap();
                            let mut camera = self.resources.get_mut::<Camera>().unwrap();
                            wgpu_state.resize(*physical_size);
                            camera.size = wgpu_state.size.into();
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            let mut wgpu_state = self.resources.get_mut::<WgpuState>().unwrap();
                            wgpu_state.resize(**new_inner_size);
                        }
                        _ => {}
                    }

                }
                Event::RedrawRequested(_) => {
                    {
                        let mut delta_time = self.resources.get_mut::<DeltaTime>().unwrap();
                        delta_time.0 = start.elapsed().as_secs_f32();
                        start = Instant::now();

                        let mut time_since_last_physics_update = self.resources.get_mut::<TimeSinceLastPhysicsUpdate>().unwrap();
                        time_since_last_physics_update.0 += delta_time.0;
                    }
                    schedule.execute(&mut self.world, &mut self.resources);
                    {
                        let mut wgpu_state = self.resources.get_mut::<WgpuState>().unwrap();
                        match &wgpu_state.render_result {
                            Ok(_) => {}
                            //Err(wgpu::SwapChainError::Lost) => wgpu_state.resize(wgpu_state.size),
                            Err(wgpu::SwapChainError::Lost) => {
                                let size = wgpu_state.size;
                                wgpu_state.resize(size);
                            }
                            Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                _ => {}
            }
        });
    }
}