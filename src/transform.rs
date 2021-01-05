#[derive(Clone, Debug)]
pub struct Transform2D {
    pub translation: glam::Vec3,
    pub rotation: f32,
}

impl Transform2D {
    pub fn new(translation: &glam::Vec3, rotation: f32) -> Self {
        Self {
            translation: translation.clone(),
            rotation,
        }
    }

    pub fn build_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_rotation_translation(
            glam::Quat::from_rotation_z(self.rotation.to_radians()),
            self.translation
        )
    }
}
