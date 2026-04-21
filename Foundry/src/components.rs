use std::any::Any;
pub trait Component {}

type EntityID = u64;
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

//--------Custom scripting-----------
pub enum Command {
    Translate([f32; 3]),
    Delete(),
    Print(String),
    SetPos(),
}

pub trait Script: Any {
    fn start(&mut self);
    fn update(&mut self) -> Vec<(EntityID, Command)>;
}

pub struct ScriptComponent {
    pub instance: Box<dyn Script>,
}
impl Component for ScriptComponent {}

pub struct WorldView {
    delta_time: i32,
}
//-------Test----------

pub struct TestScriptInstance {
    pub message: String,
}

impl Script for TestScriptInstance {
    fn start(&mut self) {
        let mut commands: Vec<Command> = Vec::new();
        self.message = String::from("Hello");
    }
    fn update(&mut self) -> Vec<(EntityID, Command)> {
        let mut command_buffer: Vec<(EntityID, Command)> = Vec::new();
        command_buffer.push((1, Command::Translate([0.001, 0.001, 0.0])));
        command_buffer
    }
}
