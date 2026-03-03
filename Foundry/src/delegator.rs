use crate::ui_data::{self, UIState};

use crate::game_data::GameContext;
use crate::ui_data::UIHandler;
use crate::vulkan_data::VulkanContext;

pub struct Delagator {
    //Mutable references to other structs
    pub vulkan_context: VulkanContext,
    pub game_context: GameContext,
    pub ui_handler: UIHandler,
}

impl Delagator {
    pub fn new(vulkan: VulkanContext, game: GameContext, ui: UIHandler) -> Self {
        Self {
            vulkan_context: vulkan,
            game_context: game,
            ui_handler: ui,
        }
    }

    pub fn check_states(&mut self) {
        match &self.ui_handler.state {
            UIState::InstatiateObject(gameobject) => {
                println!("UI state is create");
                self.game_context
                    .instantiate(gameobject.clone(), &mut self.vulkan_context, true);
            }
            UIState::None => {}
        }

        self.ui_handler.state = UIState::None;
    }

    fn simple_call_test(self) {
        println!("{}", self.game_context.game_objects.len());
        println!("{}", self.vulkan_context.indices.len());
    }
}
