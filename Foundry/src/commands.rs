use crate::ECS::{EntityBuilder, World};
type EntityID = u64;
use glam::Vec3;

pub enum Message {
    GameOver,
    GameStart,
}
/*
pub enum Command {
    Instantiate(EntityBuilder),
    Delete(),
    SendMessage(Message),
    Function(Box<dyn Fn(&mut World, EntityID)>), //Use this to make unity-like scripts
    Translate(Vec3),
    Print(String),
    SetPos(Vec3),
}
*/
pub enum Command {
    Entity(EntityID, EntityCommand),
    World(WorldCommand),
    Camera(CameraCommand),
    Message(MessageCommand),
}

pub enum WorldCommand {
    Instantiate(EntityBuilder),
    Delete(),
    SendMessage(Message),
    Custom(Box<dyn Fn(&mut World)>),
}

#[derive(Debug)]
pub enum EntityCommand {
    Translate(Vec3),
    SetPos(Vec3),
}

pub enum CameraCommand {
    LookAt(),
    Custom(),
}

pub enum UICommand {
    ShowUI(),
    HideUI(),
}

pub enum MessageCommand {
    BroadcastMessage(&'static str),
    //BroadcastMessages(Vec<String>),
}

pub struct CommandBuffer {
    //Allow for more features if needed
    pub entity_commands: Vec<(EntityID, EntityCommand)>,
    pub world_commands: Vec<WorldCommand>,
    pub broadcast_commands: Vec<MessageCommand>,
}

impl CommandBuffer {
    pub fn new() -> CommandBuffer {
        CommandBuffer {
            entity_commands: Vec::new(),
            world_commands: Vec::new(),
            broadcast_commands: Vec::new(),
        }
    }
    pub fn push(&mut self, command: Command) {
        match command {
            Command::Entity(target, command) => self.entity_commands.push((target, command)),
            Command::World(command) => self.world_commands.push(command),
            Command::Message(command) => self.broadcast_commands.push(command),
            _ => panic!(""),
        }
    }
}
