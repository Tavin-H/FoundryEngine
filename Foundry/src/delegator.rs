use crate::commands::{
    CameraCommand, Command, CommandBuffer, EntityCommand, MessageCommand, WorldCommand,
};
use crate::ui_data::{self, UIState};

use crate::components::*;
use crate::ecs::{IDAllocator, World};
use crate::game_data::GameContext;
use crate::ui_data::UIHandler;
use crate::vulkan_data::VulkanContext;
use std::collections::HashSet;
use std::panic;
use winit::event;
use winit::keyboard::{KeyCode, PhysicalKey};

type EntityID = u64;
use std::any::TypeId;
#[derive(Default)]
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
    }
    pub fn handle_mouse_movement(&mut self, delta: (f64, f64)) {
        self.mouse_moved = true;
        self.mouse_delta = delta;
    }
}

pub struct Delagator {
    //Mutable references to other structs
    pub vulkan_context: VulkanContext,
    pub game_context: GameContext,
    pub ui_handler: UIHandler,
    pub ecs_world: World,
    pub input_buffer: InputBuffer,
    pub id_allocator: IDAllocator,
    pub broadcaster: BroadCaster,
}

impl Delagator {
    pub fn new(vulkan: VulkanContext, game: GameContext, ui: UIHandler, world: World) -> Self {
        Self {
            vulkan_context: vulkan,
            game_context: game,
            ui_handler: ui,
            ecs_world: world,
            input_buffer: InputBuffer::default(),
            id_allocator: IDAllocator::default(),
            broadcaster: BroadCaster::new(),
        }
    }

    pub fn set_broadcaster(&mut self) {
        let listener_collection = self.ecs_world.compile_broadcast_listener_hash_collection();
        self.broadcaster.broadcast_listener_collection = listener_collection;
    }

    pub fn check_states(&mut self) {
        self.check_ui_state();
    }

    pub fn run_constants(&mut self, window: &winit::window::Window) {
        //Draw call from vulkan
        //record inputs
        let mut ctx = RuntimeContext {
            time: &self.game_context.time,
            input: &self.input_buffer,
            id: &mut self.id_allocator,
            broadcaster: &mut self.broadcaster,
        };
        let command_buffer = self
            .ecs_world
            .run_update_cycle(&mut ctx, &mut self.vulkan_context);
        self.execute_command_buffer(command_buffer);
        self.vulkan_draw_frame(window);
        self.input_buffer.clear_discrete_inputs();
    }

    pub fn execute_command_buffer(&mut self, command_buffer_queue: Vec<CommandBuffer>) {
        for buffer in command_buffer_queue {
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
        }
    }

    pub fn handle_entity_command(&mut self, entity: EntityID, command: EntityCommand) {
        match command {
            EntityCommand::Translate(pos) => {
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
            CameraCommand::Pan(vector) => {
                self.vulkan_context.cam_transform.translate(vector);
            }
            _ => panic!("Unsupported camera command used"),
        }
    }
    pub fn handle_message_command(&mut self, command: MessageCommand) {
        match command {
            MessageCommand::BroadcastMessage(message) => {
                if self
                    .broadcaster
                    .broadcast_listener_collection
                    .contains_key(message)
                {
                    println!(
                        "Calling message {} affecting {} listeners",
                        message,
                        &self.broadcaster.broadcast_listener_collection[message].len()
                    );
                    for function in &self.broadcaster.broadcast_listener_collection[message] {
                        function();
                    }
                } else {
                    println!("Calling message {} affecting 0 listeners", message,);
                }
            }
            _ => {}
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
