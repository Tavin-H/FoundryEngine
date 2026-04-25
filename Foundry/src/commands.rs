use crate::ECS::{EntityBuilder, World};
type EntityID = u64;
use glam::Vec3;

pub enum Message {
    GameOver,
    GameStart,
}
pub enum Command {
    Instantiate(EntityBuilder),
    Delete(),
    SendMessage(Message),
    Function(Box<dyn Fn(&mut World, EntityID)>), //Use this to make unity-like scripts
    Translate(Vec3),
    Print(String),
    SetPos(Vec3),
}
/*
pub enum Command {
    Entity(EntityID, EntityCommand),
    Cammera(CameraCommand),
}
*/

pub enum EntityCommand {
    Instantiate(EntityBuilder),
    Delete(),
    Translate(Vec3),
    SetPos(Vec3),
}

pub enum CameraCommand {
    LookAt(),
}

pub struct CommandBuffer {
    entity_commands: Vec<EntityCommand>,
    camera_commands: Vec<CameraCommand>,
}
