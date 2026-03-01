//--------Tools to use-----------
//Egui:         main brain
//Egui-winit:   taking window events
//Egui-ash:     talking to vulkan

#![allow(deprecated)]
//Base Egui
use egui;
use egui::ClippedPrimitive;

//Egui Winit stuff
use egui_winit;
use raw_window_handle::{HasDisplayHandle, HasRawDisplayHandle, HasRawWindowHandle};
use winit::window::Theme;
use winit::window::{self, Window, WindowAttributes, WindowId};

#[derive(Default)]
pub struct UIHandler {
    winit_window: Option<Window>,
    pub context: egui::Context,
    pub state: Option<egui_winit::State>,
    pub primitives: Vec<ClippedPrimitive>,
    pub pixels_per_point: f32,
    pub textures_delta: egui::TexturesDelta,
}

impl UIHandler {
    pub fn init_ui(&mut self, display_target: &Window) {
        self.create_context();
        self.create_state(display_target);
    }
    fn create_context(&mut self) {
        let mut context = egui::Context::default();
        self.context = context;
        //Simple function but seperated in case I add feature flags
    }
    fn create_state(&mut self, display_target: &Window) {
        let id = egui::ViewportId::ROOT;
        let state = egui_winit::State::new(
            self.context.clone(),
            id,
            display_target,
            Some(display_target.scale_factor() as f32),
            Some(Theme::Dark),
            None,
        );
        self.state = Some(state);
    }
    pub fn record_ui_data(&mut self, window: &Window) {
        let Some(state) = &mut self.state else {
            panic!("");
        };
        let raw_input = state.take_egui_input(&window);
        /*
        self.context.begin_pass(raw_input);

                egui::Window::new("Test").show(&self.context, |ui| {
                    ui.label("Hello Vulkan!");
                    if ui.button("click me").clicked() {
                        println!("button clicked");
                    }
                });

        let output = self.context.end_pass();
        */
        let output = self.context.run(raw_input, |ctx| {
            egui::Window::new("Test")
                .fixed_pos([100.0, 100.0])
                .show(ctx, |ui| {
                    ui.label("Hello Vulkan!");
                    if ui.button("click me").clicked() {
                        println!("button clicked");
                    }
                });
        });

        let textures = output.textures_delta;
        println!("textures in get data {}", textures.set.len());
        self.textures_delta = textures;
        self.pixels_per_point = output.pixels_per_point;

        state.handle_platform_output(window, output.platform_output);
        let clipped_primitives = self
            .context
            .tessellate(output.shapes, output.pixels_per_point);
        self.primitives = clipped_primitives;
    }
}
