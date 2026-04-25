use crate::commands::Command;
use crate::ui_data::{self, UIState};

use crate::ECS::{IDAllocator, World};
use crate::components::*;
use crate::game_data::GameContext;
use crate::ui_data::UIHandler;
use crate::vulkan_data::VulkanContext;
use std::collections::HashSet;
use winit::event;
use winit::keyboard::{KeyCode, PhysicalKey};

type EntityID = u64;
use std::any::TypeId;
#[derive(Default)]
pub struct InputBuffer {
    keyboard_inputs: Vec<winit::event::KeyEvent>,
    keys_held: HashSet<KeyCode>,
}
impl InputBuffer {
    fn clear(&mut self) {
        self.keyboard_inputs.clear();
    }
    pub fn handle_keyboard_event(&mut self, key_event: winit::event::KeyEvent) {
        let PhysicalKey::Code(code) = key_event.physical_key else {
            return;
        };
        match key_event.state {
            event::ElementState::Pressed => {
                self.keys_held.insert(code);
            }
            event::ElementState::Released => {
                self.keys_held.remove(&code);
            }
        }
    }
    pub fn get_key(&self, code: KeyCode) -> bool {
        self.keys_held.contains(&code)
        /*
        for key in self.keyboard_inputs.iter() {
            let winit::keyboard::PhysicalKey::Code(key) = key.physical_key else {
                return false;
            };
            if key == code {
                return true;
            }
        }*/
        //return false;
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
        }
    }

    pub fn add_keyboard_input(&mut self, key_event: winit::event::KeyEvent) {
        self.input_buffer.keyboard_inputs.push(key_event);
    }

    pub fn check_states(&mut self) {
        self.check_ui_state();
    }

    pub fn run_constants(&mut self, window: &winit::window::Window) {
        //Draw call from vulkan
        //record inputs
        let mut ctx = ScriptContext {
            time: &self.game_context.time,
            input: &self.input_buffer,
            id: &mut self.id_allocator,
        };
        let command_buffer = self
            .ecs_world
            .run_update_cycle(&mut ctx, &mut self.vulkan_context);
        self.execute_command_buffer(command_buffer);
        self.vulkan_draw_frame(window);
        self.input_buffer.clear();
    }

    pub fn execute_command_buffer(&mut self, command_buffer: Vec<(EntityID, Command)>) {
        for (target, command) in command_buffer {
            self.handle_command(target, command);
        }
    }

    pub fn handle_command(&mut self, entity: EntityID, command: Command) {
        match command {
            Command::Instantiate(entity_builder) => {
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
            Command::Translate(pos) => {
                let component: &mut Transform =
                    self.ecs_world.get_component_as_mut::<Transform>(entity);
                component.position[0] += pos[0];
                component.position[1] += pos[1];
                component.position[2] += pos[2];
            }
            Command::SetPos(pos) => {
                let component: &mut Transform =
                    self.ecs_world.get_component_as_mut::<Transform>(entity);
                component.position[0] = pos[0];
                component.position[1] = pos[1];
                component.position[2] = pos[2];
            }
            Command::Delete() => {
                //Todo
            }
            Command::Function(func) => {
                (*func)(&mut self.ecs_world, entity);
            }
            Command::SendMessage(message) => {}
            _ => panic!("Unkown Command found in ECB"),
        }
    }

    //pub fn handle_entity_command() {}
    //pub fn handle_camera_command() {}

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
