//--------Tools to use-----------
//Egui:         main brain
//Egui-winit:   taking window events
//Egui-ash:     talking to vulkan

//Egui Winit stuff
use imgui::*;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use winit::window::{self, Window};
#[derive(Default)]
pub struct UIHandler {
    pub context: Option<imgui::Context>,
    pub platform: Option<WinitPlatform>,
}

impl UIHandler {
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
            .size([200.0, 400.0], imgui::Condition::Always)
            .build(|| {
                ui.text("Game Object creation");
                if ui.button("Create new") {
                    println!("Create a game object");
                }
            });
        platform.prepare_render(ui, window);
        context.render();
    }
}
