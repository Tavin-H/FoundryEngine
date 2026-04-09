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

    pub fn check_ui_state(&mut self) {
        match &self.ui_handler.state {
            UIState::InstatiateObject(gameobject) => {
                println!("UI state is create");
                self.game_context
                    .instantiate(gameobject.clone(), &mut self.vulkan_context, true);
                self.ui_handler.game_objects.push(gameobject.clone());

                self.ecs_world.spawn_player();
            }
            UIState::None => {}
        }

        self.ui_handler.state = UIState::None;
    }
}
