use glam;
use nalgebra_glm::{self as glm, rotate_x_vec3, rotate_y_vec3, rotate_z_vec3};

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

    let x = yaw.cos() * pitch.cos();
    let y = yaw.sin() * pitch.cos();
    let z = pitch.sin();

    let target_position = glm::vec3(x, y, z) + position;
    target_position
}

pub fn convert_vector_to_local(
    direction: glm::Vec3,
    rotation: glm::Vec3,
    yaw: f32,
    pitch: f32,
) -> glm::Vec3 {
    /*
    Letting Y = yaw, P = pitch.

    First to get yaw you want:

    tan(Y) = x/(-y)
    Now to get pitch:

    tan(P) = sqrt(x^2 + y^2)/z
    */

    //println!("rotation = {:?}", rotation);
    //let yaw = rotation.x.atan2(-rotation.y);
    println!("yaw = {:?}", yaw);

    //let yaw = rotation.x;
    let identity = glm::identity::<f32, 4>();

    //let yaw_matrix: glm::Mat4 = glm::rotate_z(&identity, yaw);
    //let direction_expanded: glm::Vec4 = glm::Vec4::new(direction.x, direction.y, direction.z, 0.0);

    //let result = yaw_matrix * direction_expanded;
    //let one = rotate_y_vec3(&direction, pitch);
    let result = rotate_z_vec3(&direction, yaw - (std::f32::consts::PI / 4.0));

    //let ret = glm::Vec3::new(result.x, result.y, result.z);
    //println!("after = {}", ret);
    result
}
