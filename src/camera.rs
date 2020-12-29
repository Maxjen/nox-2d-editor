use legion::*;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
use crate::{
    application::DeltaTime,
    events::Events,
};

pub struct Camera {
    eye: glam::Vec3,
    target: glam::Vec3,
    pub size: (f32, f32),
    znear: f32,
    zfar: f32,

    is_up_pressed: bool,
    is_down_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            eye: (0.0, 0.0, 20.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            size: (800.0, 600.0),
            znear: 0.1,
            zfar: 100.0,

            is_up_pressed: false,
            is_down_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn build_view_projection_matrix(&self) -> glam::Mat4 {
        let view = glam::Mat4::look_at_rh(self.eye, self.target, glam::Vec3::unit_y());
        let proj = glam::Mat4::orthographic_rh(0.0, self.size.0, 0.0, self.size.1, self.znear, self.zfar);
        proj * view
    }
}

#[system]
pub fn update_camera(
    #[resource] camera: &mut Camera,
    #[resource] delta_time: &mut DeltaTime,
    #[resource] input_events: &mut Events::<KeyboardInput>,
) {
    for event in &input_events.events {
        match event {
            KeyboardInput {
                state,
                virtual_keycode: Some(keycode),
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W => {
                        camera.is_up_pressed = is_pressed;
                    }
                    VirtualKeyCode::A => {
                        camera.is_left_pressed = is_pressed;
                    }
                    VirtualKeyCode::S => {
                        camera.is_down_pressed = is_pressed;
                    }
                    VirtualKeyCode::D => {
                        camera.is_right_pressed = is_pressed;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    let mut direction = glam::Vec3::zero();
    if camera.is_up_pressed {
        direction += glam::Vec3::unit_y();
    }
    if camera.is_down_pressed {
        direction -= glam::Vec3::unit_y();
    }
    if camera.is_left_pressed {
        direction -= glam::Vec3::unit_x();
    }
    if camera.is_right_pressed {
        direction += glam::Vec3::unit_x();
    }
    if direction != glam::Vec3::zero() {
        direction = direction.normalize() * 500.0 * delta_time.0;
    }

    camera.eye += direction;
    camera.target += direction;
}
