use crate::ui_data::{self, UIState};

use crate::ECS::World;
use crate::game_data::GameContext;
use crate::ui_data::UIHandler;
use crate::vulkan_data::VulkanContext;

pub struct Delagator {
    //Mutable references to other structs
    pub vulkan_context: VulkanContext,
    pub game_context: GameContext,
    pub ui_handler: UIHandler,
    pub ecs_world: World,
}

impl Delagator {
    pub fn new(vulkan: VulkanContext, game: GameContext, ui: UIHandler, world: World) -> Self {
        Self {
            vulkan_context: vulkan,
            game_context: game,
            ui_handler: ui,
            ecs_world: world,
        }
    }

    pub fn check_states(&mut self) {
        self.check_ui_state();
    }

    pub fn run_constants(&mut self, window: &winit::window::Window) {
        //Draw call from vulkan
        self.vulkan_draw_frame(window);
    }

    pub fn vulkan_draw_frame(&mut self, window: &winit::window::Window) {
        //Get the UI data
        self.ui_handler.record_ui_data(window, 1000.0);
        let Some(ui_context) = &mut self.ui_handler.context else {
            panic!();
        };
        //Get transform component data
        //
        //Draw the frame
        let render_batches = self.ecs_world.get_render_batches();
        println!("{}", render_batches[0].0.len());
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
                    &mut self.ecs_world,
                    true,
                );
                self.ui_handler.game_objects.push(gameobject.clone());

                self.ecs_world.spawn_player();
            }
            UIState::None => {}
        }

        self.ui_handler.state = UIState::None;
    }
}
