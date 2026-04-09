//--------Tools to use-----------
//Egui:         main brain
//Egui-winit:   taking window events
//Egui-ash:     talking to vulkan

//Egui Winit stuff
use imgui::*;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use winit::window::{self, Window};

use crate::game_data::GameObject;
use crate::game_data::Transform;

//-------CONSTANTS--------
//colors
const BACKGROUND: [f32; 4] = [0.05, 0.05, 0.05, 0.6];

//'jobs' that can be done
pub enum UIState {
    None,
    InstatiateObject(GameObject),
}

pub struct UIHandler {
    pub context: Option<imgui::Context>,
    pub platform: Option<WinitPlatform>,
    pub state: UIState,
    x_val: f32,
    game_object_creation: GameObjectCreation,
    pub game_objects: Vec<GameObject>,
}

#[derive(Default)]
struct GameObjectCreation {
    name: String,
    position: [f32; 3],
    scale: [f32; 3],
}

impl Default for UIHandler {
    fn default() -> Self {
        Self {
            context: None,
            platform: None,
            state: UIState::None,
            x_val: 0.0,
            game_object_creation: GameObjectCreation {
                ..Default::default()
            },
            game_objects: Vec::new(),
        }
    }
}

impl UIHandler {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn init(&mut self, window: &Window) {
        let mut context = imgui::Context::create();
        //Style
        let style = context.style_mut();
        style.colors[imgui::StyleColor::WindowBg as usize] = BACKGROUND; // Near black
        //
        context.set_ini_filename(None); // disable imgui.ini file

        let mut platform = WinitPlatform::new(&mut context);
        platform.attach_window(context.io_mut(), window, HiDpiMode::Default);

        self.context = Some(context);
        self.platform = Some(platform);
    }
    pub fn record_ui_data(&mut self, window: &Window, fps: f32) {
        let Some(platform) = &mut self.platform else {
            panic!();
        };
        let Some(context) = &mut self.context else {
            panic!();
        };
        platform.prepare_frame(context.io_mut(), window).unwrap();

        let ui = context.new_frame();

        // 1. Push the style color for WindowBg
        ui.window("Foundry Engine Debug Window")
            .position([0.0, 0.0], imgui::Condition::Always)
            .size([100.0, 20.0], Condition::Always)
            .no_decoration()
            .build(|| {
                ui.text(format!("FPS: {:.1}", fps));
                /*
                                if ui.button("Click me") {
                                    println!("ui clicked!");
                                }
                */
            });
        ui.window("Editor")
            .size([250.0, 200.0], imgui::Condition::Always)
            .position([0.0, 430.0], imgui::Condition::Always)
            .build(|| {
                ui.text("Game Object creation");
                ui.text("Name");
                ui.input_text(" ", &mut self.game_object_creation.name)
                    .build();
                ui.text("Position:");
                ui.set_next_item_width(50.0);
                imgui::Drag::new("x")
                    .speed(0.01)
                    .range(-2.0, 2.0)
                    .build(ui, &mut self.game_object_creation.position[0]);
                ui.same_line();
                ui.set_next_item_width(50.0);
                imgui::Drag::new("y")
                    .speed(0.01)
                    .range(-2.0, 2.0)
                    .build(ui, &mut self.game_object_creation.position[1]);
                ui.same_line();
                ui.set_next_item_width(50.0);
                imgui::Drag::new("z")
                    .speed(0.01)
                    .range(-2.0, 2.0)
                    .build(ui, &mut self.game_object_creation.position[2]);
                if ui.button("Create new") {
                    let x = self.game_object_creation.position[0];
                    let y = self.game_object_creation.position[1];
                    let z = self.game_object_creation.position[2];
                    let mut gameobject = GameObject {
                        name: self.game_object_creation.name.clone(),
                        id: 0,
                        transform: Transform {
                            position: [x, y, z],
                            scale: [1.0, 1.0, 1.0],
                        },
                        ..Default::default()
                    };
                    self.state = UIState::InstatiateObject(gameobject);
                    println!("Set state to create");
                }
            });
        ui.window("Scene")
            .size([250.0, 400.0], imgui::Condition::Always)
            .position([0.0, 30.0], imgui::Condition::Always)
            .build(|| {
                ui.text("Scene Graph");
                for game_object in &self.game_objects {
                    //ui.text(&game_object.name);
                    let _color_token = ui.push_style_color(imgui::StyleColor::Button, BACKGROUND);
                    if ui.button(&game_object.name) {
                        println!("Clicked on {}", game_object.name);
                    };
                }
            });

        platform.prepare_render(ui, window);
        context.render();
    }
}
