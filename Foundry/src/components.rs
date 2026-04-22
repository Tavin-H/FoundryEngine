use crate::{ECS::World, delegator::InputBuffer};
use std::any::Any;
use winit::keyboard::KeyCode;
pub trait Component {}
use glam::Vec3;

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
    Translate(Vec3),
    Delete(),
    Print(String),
    SetPos(),
}

pub trait Script: Any {
    fn start(&mut self);
    fn update(&mut self, ctx: &ScriptContext) -> Vec<(EntityID, Command)>;
}

pub struct ScriptComponent {
    pub instance: Box<dyn Script>,
}
impl Component for ScriptComponent {}

pub struct WorldView {
    delta_time: i32,
}

//Context struct

pub struct TimeData {
    pub delta_time: f32,
}

pub struct ScriptContext<'a> {
    pub time: &'a TimeData,
    pub input: &'a InputBuffer,
    //Add world later
}
//-------Test----------

pub struct TestScriptInstance {}

impl Script for TestScriptInstance {
    fn start(&mut self) {
        let mut commands: Vec<Command> = Vec::new();
    }

    fn update(&mut self, ctx: &ScriptContext) -> Vec<(EntityID, Command)> {
        //start command buffer
        let ScriptContext { time, input } = ctx;
        let mut command_buffer: Vec<(EntityID, Command)> = Vec::new();

        //Logic
        if input.get_key(KeyCode::KeyS) {
            command_buffer.push((
                1,
                Command::Translate(Vec3::new(1.0, 1.0, 0.0) * time.delta_time),
            ));
        }
        if input.get_key(KeyCode::KeyW) {
            command_buffer.push((
                1,
                Command::Translate(Vec3::new(-1.0, -1.0, 0.0) * time.delta_time),
            ));
        }
        if input.get_key(KeyCode::KeyD) {
            command_buffer.push((
                1,
                Command::Translate(Vec3::new(-1.0, 1.0, 0.0) * time.delta_time),
            ));
        }
        if input.get_key(KeyCode::KeyA) {
            command_buffer.push((
                1,
                Command::Translate(Vec3::new(1.0, -1.0, 0.0) * time.delta_time),
            ));
        }

        //Return command buffer
        command_buffer
    }
}
