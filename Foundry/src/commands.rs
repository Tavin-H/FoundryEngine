use crate::ecs::{EntityBuilder, World};
type EntityID = u64;
use glam::Vec3;

pub enum Message {
    GameOver,
    GameStart,
}

pub enum Command {
    Entity(EntityID, EntityCommand),
    World(WorldCommand),
    Camera(CameraCommand),
    Message(MessageCommand),
}

pub enum WorldCommand {
    Instantiate(EntityBuilder),
    Delete(), //Unsupported
    SendMessage(Message),
    Custom(Box<dyn Fn(&mut World)>),
}

#[derive(Debug)]
pub enum EntityCommand {
    Translate(Vec3),
    SetPos(Vec3),
    LookAt(),       //Unsupported
    RotateAround(), //Unsupported
    SetRotation(),  //Unsupported
}

pub enum CameraCommand {
    SetFov(u32), //Unsupported
    Custom(),
}

pub enum UICommand {
    ShowUI(), //Unsupported
    HideUI(), //Unsupported
}

pub enum MessageCommand {
    BroadcastMessage(&'static str),
    BroadcastMessages(Vec<&'static str>),
}

pub struct CommandBuffer {
    //Allow for more features if needed
    pub entity_commands: Vec<(EntityID, EntityCommand)>,
    pub world_commands: Vec<WorldCommand>,
    pub broadcast_commands: Vec<MessageCommand>,
    pub camera_commands: Vec<CameraCommand>,
}

impl CommandBuffer {
    pub fn new() -> CommandBuffer {
        CommandBuffer {
            entity_commands: Vec::new(),
            world_commands: Vec::new(),
            broadcast_commands: Vec::new(),
            camera_commands: Vec::new(),
        }
    }
    pub fn push(&mut self, command: Command) {
        match command {
            Command::Entity(target, command) => self.entity_commands.push((target, command)),
            Command::World(command) => self.world_commands.push(command),
            Command::Message(command) => self.broadcast_commands.push(command),
            Command::Camera(command) => self.camera_commands.push(command),
            _ => panic!(""),
        }
    }
}
