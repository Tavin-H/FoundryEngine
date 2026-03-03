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
}

impl Default for UIHandler {
    fn default() -> Self {
        Self {
            context: None,
            platform: None,
            state: UIState::None,
            x_val: 0.0,
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
            .size([250.0, 400.0], imgui::Condition::Always)
            .build(|| {
                ui.text("Game Object creation");
                if ui.button("Create new") {
                    let rng = rand::rng();
                    let x = rand::random_range(-2.0..2.0);
                    let y = rand::random_range(-2.0..2.0);
                    let mut gameobject = GameObject {
                        name: String::from("Example"),
                        id: 0,
                        transform: Transform {
                            position: [x, y, 0.0],
                            scale: [1.0, 1.0, 1.0],
                        },
                        ..Default::default()
                    };
                    self.state = UIState::InstatiateObject(gameobject);
                    println!("Set state to create");
                }
                ui.set_next_item_width(50.0);
                imgui::Drag::new("x")
                    .speed(0.01)
                    .range(-2.0, 2.0)
                    .build(ui, &mut self.x_val);
                ui.same_line();
                ui.set_next_item_width(50.0);
                imgui::Drag::new("y")
                    .speed(0.01)
                    .range(-2.0, 2.0)
                    .build(ui, &mut self.x_val);
                ui.same_line();
                ui.set_next_item_width(50.0);
                imgui::Drag::new("z")
                    .speed(0.01)
                    .range(-2.0, 2.0)
                    .build(ui, &mut self.x_val);
            });
        platform.prepare_render(ui, window);
        context.render();
    }
}
