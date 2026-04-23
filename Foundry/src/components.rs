use crate::{
    ECS::{EntityBuilder, IDAllocator, World},
    delegator::InputBuffer,
};
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
    Instantiate(EntityBuilder),
    Delete(EntityID),
    Translate(Vec3),
    Print(String),
    SetPos(),
}

pub trait Script: Any {
    fn start() -> Box<dyn Script>
    where
        Self: Sized;
    fn update(&mut self, ctx: &mut ScriptContext) -> Vec<(EntityID, Command)>;
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
    pub id: &'a mut IDAllocator,
    //Add world later
}
//-------Test----------

pub struct TestScriptInstance {
    //Make list of needed variables
    y_velocity: f32,
    timer: f32,
}

impl Script for TestScriptInstance {
    fn start() -> Box<dyn Script> {
        //Return instance
        Box::new(TestScriptInstance {
            //Initialize variables
            //1
            //2
            //...
            y_velocity: 0.0,
            timer: 0.0,
        })
    }

    fn update(&mut self, ctx: &mut ScriptContext) -> Vec<(EntityID, Command)> {
        //start command buffer
        let ScriptContext { time, input, id } = ctx;
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

        if input.get_key(KeyCode::KeyK) {
            let test_id = id.reserve_id();
            println!("{}", test_id);
            let test = EntityBuilder::spawn(test_id)
                .with::<MeshAllocation>(MeshAllocation::default())
                .with::<Transform>(Transform {
                    position: [0.0, 0.0, 0.0],
                    scale: [1.0, 1.0, 1.0],
                })
                .with::<ScriptComponent>(ScriptComponent {
                    instance: Box::new(MoveScriptInstance {}),
                });
            command_buffer.push((id.this, Command::Instantiate(test)));
        }
        if (input.get_key(KeyCode::Space)) {
            self.y_velocity = 5.0;
        }
        self.y_velocity -= 9.8 * 4.0 * time.delta_time;
        command_buffer.push((
            1,
            Command::Translate(Vec3::new(0.0, 0.0, self.y_velocity) * time.delta_time),
        ));

        //Spawning
        self.timer += time.delta_time;
        if (self.timer > 2.0) {
            let test_id = id.reserve_id();
            println!("{}", test_id);
            let test = EntityBuilder::spawn(test_id)
                .with::<MeshAllocation>(MeshAllocation::default())
                .with::<Transform>(Transform {
                    position: [0.0, 0.0, 0.0],
                    scale: [1.0, 1.0, 1.0],
                })
                .with::<ScriptComponent>(ScriptComponent {
                    instance: Box::new(MoveScriptInstance {}),
                });
            command_buffer.push((id.this, Command::Instantiate(test)));
            self.timer = 0.0;
        }
        //Return command buffer
        command_buffer
    }
}

pub struct MoveScriptInstance {}

impl Script for MoveScriptInstance {
    fn start() -> Box<dyn Script> {
        let mut commands: Vec<Command> = Vec::new();
        Box::new(MoveScriptInstance {})
    }

    fn update(&mut self, ctx: &mut ScriptContext) -> Vec<(EntityID, Command)> {
        //start command buffer
        let ScriptContext { time, input, id } = ctx;
        let mut command_buffer: Vec<(EntityID, Command)> = Vec::new();

        //Logic
        command_buffer.push((
            id.this,
            Command::Translate(Vec3::new(1.0, 0.0, 0.0) * time.delta_time),
        ));

        //Return command buffer
        command_buffer
    }
}
