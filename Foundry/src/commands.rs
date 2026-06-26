use std::any::TypeId;

use crate::ecs::{EntityBuilder, World};
type EntityID = uuid::Uuid;
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
    Audio(AudioCommand),
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
    Rotate(Vec3),
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
    BroadcastMessage(String),
    BroadcastMessages(Vec<&'static str>),
}

pub enum AudioCommand {
    Play(&'static str),
}

pub struct CommandBuffer {
    //Allow for more features if needed
    pub entity_commands: Vec<(EntityID, EntityCommand)>,
    pub world_commands: Vec<WorldCommand>,
    pub broadcast_commands: Vec<MessageCommand>,
    pub camera_commands: Vec<CameraCommand>,
    pub audio_commands: Vec<AudioCommand>,
}
unsafe impl Send for CommandBuffer {}

impl CommandBuffer {
    pub fn new() -> CommandBuffer {
        CommandBuffer {
            entity_commands: Vec::new(),
            world_commands: Vec::new(),
            broadcast_commands: Vec::new(),
            camera_commands: Vec::new(),
            audio_commands: Vec::new(),
        }
    }
    pub fn push(&mut self, command: Command) {
        match command {
            Command::Entity(target, command) => self.entity_commands.push((target, command)),
            Command::World(command) => self.world_commands.push(command),
            Command::Message(command) => self.broadcast_commands.push(command),
            Command::Camera(command) => self.camera_commands.push(command),
            Command::Audio(command) => self.audio_commands.push(command),
            _ => panic!(""),
        }
    }
    /*
    pub fn push<T: 'static>(&mut self, command: T) {
        let audio = TypeId::of::<AudioCommand>();
        match TypeId::of::<T>() {
            audio => {
                self.audio_commands.push(command as AudioCommand);
            }
            _ => panic!(),
        }
    }
    */
}
