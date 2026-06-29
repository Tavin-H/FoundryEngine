use crate::audio_manager::AudioManager;
use crate::commands::{
    AudioCommand, CameraCommand, Command, CommandBuffer, EntityCommand, MessageCommand,
    WorldCommand,
};
use crate::id_consts::CAMERA;
use crate::ui_data::{self, UIState};

use crate::components::*;
use crate::ecs::{IDAllocator, World};
use crate::game_data::GameContext;
use crate::ui_data::UIHandler;
use crate::vulkan_data::VulkanContext;
use mlua::{FromLua, Lua, UserData, Value};
use std::collections::{HashMap, HashSet};
use std::panic;
use std::sync::{Arc, Mutex};
use winit::event;
use winit::keyboard::{KeyCode, PhysicalKey};

use mlua::{LuaSerdeExt, Result};
//use serde::Deserialize;

use crate::lua_engine::LuaEngine;

type EntityID = uuid::Uuid;
use std::any::TypeId;

#[derive(Default, Clone)]
pub struct InputBuffer {
    key_down_list: HashSet<KeyCode>,
    key_up_list: HashSet<KeyCode>,
    keys_held: HashSet<KeyCode>,
    mouse_delta: (f64, f64),
    mouse_moved: bool,
    mouse_pos: (f64, f64), //Not implemented yet
}
pub enum MouseAxis {
    X,
    Y,
}
impl InputBuffer {
    fn clear_discrete_inputs(&mut self) {
        self.key_down_list.clear();
        self.key_up_list.clear();
    }

    // API
    pub fn get_key(&self, code: KeyCode) -> bool {
        self.keys_held.contains(&code)
    }
    pub fn get_key_down(&self, code: KeyCode) -> bool {
        self.key_down_list.contains(&code)
    }
    pub fn get_key_up(&self, code: KeyCode) -> bool {
        self.key_up_list.contains(&code)
    }
    pub fn get_mouse_axis(&self, axis: MouseAxis) -> f64 {
        match axis {
            // Rust has no ternary operator :(
            MouseAxis::X => {
                if (self.mouse_moved) {
                    return self.mouse_delta.0;
                } else {
                    return 0.0;
                }
            }
            MouseAxis::Y => {
                if (self.mouse_moved) {
                    return self.mouse_delta.1;
                } else {
                    return 0.0;
                }
            }
        }
    }

    // Backend
    pub fn handle_keyboard_event(&mut self, key_event: winit::event::KeyEvent) {
        let PhysicalKey::Code(code) = key_event.physical_key else {
            return;
        };
        match key_event.state {
            event::ElementState::Pressed => {
                self.key_down_list.insert(code);
                self.keys_held.insert(code);
            }
            event::ElementState::Released => {
                self.key_up_list.insert(code);
                self.keys_held.remove(&code);
            }
        }
    }
    pub fn set_mouse_moved(&mut self, moved: bool) {
        self.mouse_moved = moved;
        if (!moved) {
            self.mouse_delta = (0.0, 0.0);
        }
    }
    pub fn handle_mouse_movement(&mut self, delta: (f64, f64)) {
        self.mouse_moved = true;
        self.mouse_delta.0 += delta.0;
        self.mouse_delta.1 += delta.1;
    }
}

pub struct RuntimeContext {
    pub input_buffer_ref: InputBufferRef,
    pub id_allocator_ref: IDAllocatorRef,
}
impl UserData for RuntimeContext {}
impl RuntimeContext {
    pub fn new() -> Self {
        RuntimeContext {
            input_buffer_ref: InputBufferRef(Arc::new(InputBuffer::default())),
            id_allocator_ref: IDAllocatorRef(Arc::new(IDAllocator::default())),
        }
    }
}

// TODO:
// time
// id
// broadcaster
// extract these to a separate file?

#[derive(Clone)]
pub struct InputBufferRef(pub Arc<InputBuffer>);
impl UserData for InputBufferRef {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get_key", |lua, this, key_code_val| {
            let key_code = lua.from_value(key_code_val).expect("Uh Oh spag");
            Ok(this.0.get_key(key_code))
        });
    }
}

impl InputBufferRef {
    fn copy_local(&mut self, other: &InputBuffer) {
        self.0 = Arc::new(other.clone());
    }
    pub fn get_key(&self, code: KeyCode) -> bool {
        self.0.keys_held.contains(&code)
    }
    pub fn get_key_down(&self, code: KeyCode) -> bool {
        self.0.key_down_list.contains(&code)
    }
    pub fn get_key_up(&self, code: KeyCode) -> bool {
        self.0.key_up_list.contains(&code)
    }
}
#[derive(Clone)]
pub struct IDAllocatorRef(pub Arc<IDAllocator>);
impl UserData for IDAllocatorRef {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("reserve_id", |_, mut this, ()| {
            let id = this.0.reserve_id();
            Ok(id.as_u128())
        });
        methods.add_method("this", |_, this, ()| Ok(this.0.this_id.as_u128()));
    }
}
pub struct Test(pub u64);
impl UserData for Test {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("reserve_id", |_, this, ()| {
            let id = this.0 = 10;
            Ok(())
        });
    }
}

impl IDAllocatorRef {
    fn copy_local(&mut self, other: &IDAllocator) {
        self.0 = Arc::new(other.clone());
    }
}

//Mutable references to other structs
pub struct Delagator {
    //Top level structs
    pub vulkan_context: VulkanContext,
    pub game_context: GameContext,

    //Structs for excecuting commands
    pub ui_handler: UIHandler,
    pub ecs_world: World,
    pub lua_engine: LuaEngine,
    pub audio_manager: AudioManager,
    pub broadcaster: BroadCaster,

    // Context structs
    pub runtime_context: RuntimeContext,
    pub input_buffer: InputBuffer,
    pub id_allocator: IDAllocator,
    //pub world (Access other things in the world e.g. search for named object)
}

impl Delagator {
    pub fn new(vulkan: VulkanContext, game: GameContext, ui: UIHandler, world: World) -> Self {
        let mut audio_manager = AudioManager::new();
        let mut lua = LuaEngine::init(4).unwrap();
        lua.add_update_function(std::path::Path::new("src/test.lua"));
        //audio_manager.play("");
        Self {
            vulkan_context: vulkan,
            game_context: game,
            ui_handler: ui,
            ecs_world: world,
            input_buffer: InputBuffer::default(),
            runtime_context: RuntimeContext::new(),
            id_allocator: IDAllocator::default(),
            broadcaster: BroadCaster::new(),
            audio_manager: audio_manager,
            lua_engine: lua,
        }
    }

    pub fn set_broadcaster(&mut self) {
        let listener_collection = self.ecs_world.compile_broadcast_listener_hash_collection();
        self.broadcaster.broadcast_listener_collection = listener_collection;
    }

    pub fn check_states(&mut self) {
        self.check_ui_state();
    }

    pub fn create_runtime_context_snapshot(&mut self) {
        self.runtime_context
            .input_buffer_ref
            .copy_local(&self.input_buffer);
    }

    pub fn run_constants(&mut self, window: &winit::window::Window) {
        //Draw call from vulkan
        //record inputs
        /*
                let mut ctx = RuntimeContext {
                    time: &self.game_context.time,
                    input: &self.input_buffer,
                    id: &mut self.id_allocator,
                    broadcaster: &mut self.broadcaster,
                };
        let command_buffer = self
            .ecs_world
            .run_update_cycle(&mut ctx, &mut self.vulkan_context);
        */
        self.create_runtime_context_snapshot();
        self.lua_engine.run_update_cycle(&self.runtime_context);
        /*
                let mut ctx = RuntimeContext {
                    input_buffer_ref: self.input_buffer_shared.clone(),
                };
        self.lua_engine.batch_context(&self.runtime_context);
        let result = self
            .lua_engine
            .execute_lua_behaviour(0, );
        match result {
            Err(error) => panic!("{}", error),
            Ok(_) => {}
        }
        */
        self.execute_command_buffer_index();
        self.vulkan_draw_frame(window);
        self.input_buffer.clear_discrete_inputs();
    }

    pub fn execute_command_buffer_index(&mut self) {
        /*
                let command_buffer_index = Arc::clone(&self.lua_engine.command_buffer_index);
                let mut map = command_buffer_index.lock().unwrap();
        */
        let buffers: Vec<CommandBuffer> =
            self.lua_engine.command_buffer_storage.drain(..).collect();
        for buffer in buffers {
            self.execute_command_buffer(buffer);
        }
    }

    pub fn execute_command_buffer(&mut self, buffer: CommandBuffer) {
        for (entity, command) in buffer.entity_commands {
            self.handle_entity_command(entity, command);
        }
        for command in buffer.world_commands {
            self.handle_world_command(command);
        }
        for command in buffer.broadcast_commands {
            self.handle_message_command(command);
        }
        for command in buffer.camera_commands {
            self.handle_camera_command(command);
        }
        for command in buffer.audio_commands {
            self.handle_audio_command(command);
        }
    }

    pub fn handle_entity_command(&mut self, entity: EntityID, command: EntityCommand) {
        match command {
            EntityCommand::Translate(pos) => {
                if (entity == CAMERA) {
                    println!("Moving cam {pos}");

                    //self.vulkan_context.cam_transform.translate_local(pos);
                    self.vulkan_context.cam_transform.translate(pos);
                    return;
                }
                let component: &mut Transform =
                    self.ecs_world.get_component_as_mut::<Transform>(entity);
                component.position[0] += pos[0];
                component.position[1] += pos[1];
                component.position[2] += pos[2];
            }
            EntityCommand::SetPos(pos) => {
                let component: &mut Transform =
                    self.ecs_world.get_component_as_mut::<Transform>(entity);
                component.position[0] = pos[0];
                component.position[1] = pos[1];
                component.position[2] = pos[2];
            }
            EntityCommand::Rotate(rot) => {
                if (entity == CAMERA) {
                    self.vulkan_context.cam_transform.rotate(rot);
                    return;
                }
                panic!("");
            }
            other => panic!("Unkown EntityCommand found in ECB: {:?}", other),
        }
    }
    pub fn handle_world_command(&mut self, command: WorldCommand) {
        match command {
            WorldCommand::Instantiate(entity_builder) => {
                //Get mesh_allocation data
                //Build entity_builder
                let has_mesh = entity_builder
                    .signature
                    .contains(&TypeId::of::<MeshAllocation>());
                let id = entity_builder.id;
                entity_builder.build(&mut self.ecs_world);
                if (has_mesh) {
                    let new_mesh_data = self.vulkan_context.create_mesh_data();
                    let mesh_data: &mut MeshAllocation =
                        self.ecs_world.get_component_as_mut::<MeshAllocation>(id);
                    mesh_data.first_vertex = new_mesh_data.first_vertex;
                    mesh_data.first_index = new_mesh_data.first_index;
                    mesh_data.index_count = new_mesh_data.index_count;
                }
                self.vulkan_context.upload_mesh_data();
            }
            WorldCommand::SendMessage(message) => {}
            WorldCommand::Delete() => {
                //Todo
            }
            WorldCommand::Custom(func) => {
                (*func)(&mut self.ecs_world);
            }
            _ => {}
        }
    }
    pub fn handle_camera_command(&mut self, command: CameraCommand) {
        match command {
            CameraCommand::SetFov(fov) => (),
            _ => panic!("Unsupported camera command used"),
        }
    }
    pub fn handle_message_command(&mut self, command: MessageCommand) {
        match command {
            MessageCommand::BroadcastMessage(message) => {
                if self
                    .broadcaster
                    .broadcast_listener_collection
                    .contains_key(message.as_str())
                {
                    println!(
                        "Calling message {} affecting {} listeners",
                        message,
                        &self.broadcaster.broadcast_listener_collection[message.as_str()].len()
                    );
                    for function in
                        &self.broadcaster.broadcast_listener_collection[message.as_str()]
                    {
                        function();
                    }
                } else {
                    println!("Calling message {} affecting 0 listeners", message,);
                }
            }
            _ => {}
        }
    }

    pub fn handle_audio_command(&mut self, command: AudioCommand) {
        match command {
            AudioCommand::Play(path) => {
                self.audio_manager.play(path);
            }
            _ => panic!(),
        }
    }

    pub fn vulkan_draw_frame(&mut self, window: &winit::window::Window) {
        //Get the UI data
        //
        let fps = 1.0 / self.game_context.calculate_delta_time();
        self.ui_handler.record_ui_data(window, fps);
        let Some(ui_context) = &mut self.ui_handler.context else {
            panic!();
        };
        //Get transform component data
        //
        //Draw the frame
        let render_batches = self.ecs_world.get_render_batches();
        self.vulkan_context.draw_frame(
            &self.game_context.game_objects,
            render_batches,
            ui_context,
            window,
        );
        //Reduce game_objects to just uh idk
    }

    pub fn check_ui_state(&mut self) {
        match &self.ui_handler.state {
            UIState::InstatiateObject(gameobject) => {
                println!("UI state is create");
                self.game_context.instantiate(
                    gameobject.clone(),
                    &mut self.vulkan_context,
                    &mut self.id_allocator,
                    &mut self.ecs_world,
                    true,
                    false,
                );
                self.ui_handler.game_objects.push(gameobject.clone());
            }
            UIState::None => {}
        }

        self.ui_handler.state = UIState::None;
    }
}
