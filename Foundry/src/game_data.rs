#[derive(Default)]
pub struct MeshAllocation {
    pub index_count: u32,
    pub first_index: u32,
    pub first_vertex: i32,
}

#[derive(Default)]
pub struct GameObject {
    pub id: u32,
    pub name: String,
    pub _mesh: MeshAllocation,
}
