pub trait Component {}
#[derive(Default, Debug, Clone)]
pub struct MeshAllocation {
    pub index_count: u32,
    pub first_index: u32,
    pub first_vertex: i32,
}
impl Component for MeshAllocation {}

#[derive(Default, Clone, Copy, Debug)]
pub struct Transform {
    pub position: [f32; 3],
    pub scale: [f32; 3],
}
impl Component for Transform {}

#[derive(Default, Debug, Clone)]
pub struct GameObject {
    pub entity_id: u32,
    pub name: String,
    pub tags: Vec<String>,
}
