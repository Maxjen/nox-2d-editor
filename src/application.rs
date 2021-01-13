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
    wgpu_state::{self, WgpuState},
    ui::UiState,
    app_state::{self, AppState},
    transform,
    events::{self, Events},
    command::Command,
    camera::{self, Camera},
    asset::{self, Assets},
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
        self.resources.insert(Events::<Command>::default());
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
            let mut commands = self.resources.get_mut::<Events<Command>>().unwrap();
            commands.send(Command::SetCurrentScene(0));
        }
        self.handle_commands();
    }

    pub fn run(mut self) {
        let mut schedule_1 = Schedule::builder()
            .add_system(app_state::handle_input_system())
            .add_system(camera::update_camera_system())
            .add_system(physics::update_physics_system())
            .add_system(physics::copy_transforms_from_rigid_bodies_system())
            .build();
            //.add_system(mesh_pipeline::update_camera_buffer_system())
        
        let mut schedule_2 = Schedule::builder()
            .add_system(wgpu_state::prepare_frame_system())
            .add_system(mesh::render_meshes_system())
            .add_system(events::clear_events_system::<winit::event::KeyboardInput>())
            .add_system(asset::remove_unused_assets_system::<Texture>())
            .build();

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("2D Editor")
            .with_maximized(true)
            .build(&event_loop)
            .unwrap();
        
        self.setup_world(&window);

        let mut ui_state = {
            let wgpu_state = self.resources.get_mut::<WgpuState>().unwrap();
            UiState::new(&window, &wgpu_state)
        };

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

                    schedule_1.execute(&mut self.world, &mut self.resources);
                    transform::propagate_transforms(&mut self.world, &mut self.resources);
                    schedule_2.execute(&mut self.world, &mut self.resources);
                    ui_state.render_ui(&window, &self.resources);

                    self.handle_commands();

                    {
                        let mut wgpu_state = self.resources.get_mut::<WgpuState>().unwrap();
                        wgpu_state.current_frame = None;
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
            ui_state.platform.handle_event(ui_state.imgui.io_mut(), &window, &event);
        });
    }

    pub fn handle_commands(&mut self) {
        // Handle commands.
        app_state::handle_commands(&mut self.world, &mut self.resources);

        // Clear commands.
        let mut commands = self.resources.get_mut::<Events::<Command>>().unwrap();
        events::clear_events(&mut commands);
    }
}