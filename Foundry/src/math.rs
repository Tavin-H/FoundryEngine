use glam;
use nalgebra_glm::{self as glm};

pub fn convert_glam_to_glm3(vector: glam::Vec3) -> glm::Vec3 {
    let array: [f32; 3] = vector.into();
    let glm_vec = glm::Vec3::from(array);
    glm_vec
}

// Vec3(yaw, pitch, 0)
pub fn calculate_rotation_target(position: glm::Vec3, rotation: glm::Vec3) -> glm::Vec3 {
    //Use a unit sphere to calculate
    let yaw: f32 = rotation.x;
    let pitch: f32 = rotation.y;

    let x = yaw.sin() * pitch.cos();
    let y = yaw.sin() * pitch.sin();
    let z = yaw.cos();

    let target_position = glm::vec3(x, y, z) + position;
    target_position
}
