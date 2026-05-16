use crate::{
    delegator::InputBuffer,
    ecs::{EntityBuilder, IDAllocator, World},
};
use std::{any::Any, collections::HashMap};
use winit::keyboard::KeyCode;
pub trait Component {}
use glam::Vec3;
use std::collections::HashSet;

use crate::commands::*;

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

pub trait Script: Any {
    fn start() -> Box<dyn Script>
    where
        Self: Sized;
    fn update(&mut self, ctx: &mut RuntimeContext) -> CommandBuffer;
    fn get_broadcast_listeners(&mut self) -> BroadCasterListenerHash {
        HashMap::new()
    }
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

//Types used for broadcaster
pub type BroadCasterListenerHash = HashMap<&'static str, Box<dyn Fn() -> CommandBuffer>>;
pub type BroadCasterListenerHashCollection =
    HashMap<&'static str, Vec<Box<dyn Fn() -> CommandBuffer>>>;

pub struct BroadCaster {
    pub broadcast_listener_collection: BroadCasterListenerHashCollection,
}

impl BroadCaster {
    pub fn new() -> BroadCaster {
        BroadCaster {
            broadcast_listener_collection: HashMap::new(),
        }
    }
}

pub struct RuntimeContext<'a> {
    pub time: &'a TimeData,
    pub input: &'a InputBuffer,
    pub id: &'a mut IDAllocator,
    pub broadcaster: &'a mut BroadCaster,
    //Add world later
}
//-------Test----------

pub struct TestScriptInstance {
    //Make list of needed variables
    y_velocity: f32,
    timer: f32,
}

use crate::commands::*;
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

    fn update(&mut self, ctx: &mut RuntimeContext) -> CommandBuffer {
        //start command buffer
        let RuntimeContext {
            time,
            input,
            id,
            broadcaster,
        } = ctx;
        let mut command_buffer = CommandBuffer::new();

        //Logic
        if input.get_key(KeyCode::KeyS) {
            command_buffer.push(Command::Entity(
                1,
                EntityCommand::Translate(Vec3::new(1.0, 1.0, 0.0) * time.delta_time),
            ));
        }
        if input.get_key(KeyCode::KeyW) {
            command_buffer.push(Command::Entity(
                1,
                EntityCommand::Translate(Vec3::new(-1.0, -1.0, 0.0) * time.delta_time),
            ));
        }
        if input.get_key(KeyCode::KeyD) {
            command_buffer.push(Command::Entity(
                1,
                EntityCommand::Translate(Vec3::new(-1.0, 1.0, 0.0) * time.delta_time),
            ));
        }
        if input.get_key(KeyCode::KeyA) {
            command_buffer.push(Command::Entity(
                1,
                EntityCommand::Translate(Vec3::new(1.0, -1.0, 0.0) * time.delta_time),
            ));
        }
        if input.get_key_up(KeyCode::KeyC) {
            command_buffer.push(Command::Message(MessageCommand::BroadcastMessage("Test")));
        }
        command_buffer.push(Command::Entity(
            id.camera,
            EntityCommand::Translate(
                Vec3::new(
                    input.get_mouse_axis(crate::delegator::MouseAxis::X) as f32,
                    input.get_mouse_axis(crate::delegator::MouseAxis::Y) as f32,
                    0.0,
                ) * time.delta_time
                    * 10.0,
            ),
        ));

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
            command_buffer.push(Command::World(WorldCommand::Instantiate(test)));
        }
        if (input.get_key(KeyCode::Space)) {
            self.y_velocity = 5.0;
        }
        self.y_velocity -= 9.8 * 4.0 * time.delta_time;
        command_buffer.push(Command::Entity(
            1,
            EntityCommand::Translate(Vec3::new(0.0, 0.0, self.y_velocity) * time.delta_time),
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
            command_buffer.push(Command::World(WorldCommand::Instantiate(test)));
            self.timer = 0.0;
        }

        //Return command buffer
        command_buffer
    }
    fn get_broadcast_listeners(&mut self) -> BroadCasterListenerHash {
        let mut listeners: BroadCasterListenerHash = HashMap::new();
        listeners.insert(
            "Test",
            Box::new(|| {
                println!("Testing the broadcaster");
                CommandBuffer::new()
            }),
        );
        listeners
    }
}

pub struct MoveScriptInstance {}

impl Script for MoveScriptInstance {
    fn start() -> Box<dyn Script> {
        let mut commands: Vec<Command> = Vec::new();
        Box::new(MoveScriptInstance {})
    }

    fn update(&mut self, ctx: &mut RuntimeContext) -> CommandBuffer {
        //start command buffer
        let RuntimeContext {
            time,
            input,
            id,
            broadcaster,
        } = ctx;
        let mut command_buffer = CommandBuffer::new();

        //Logic
        command_buffer.push(Command::Entity(
            id.this,
            EntityCommand::Translate(Vec3::new(1.0, 0.0, 0.0) * time.delta_time),
        ));

        //Return command buffer
        command_buffer
    }
}
