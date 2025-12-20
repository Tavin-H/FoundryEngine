//==========================================================================================
//           ______                    _               ______             _
//          |  ____|                  | |             |  ____|           (_)
//          | |__ ___  _   _ _ __   __| |_ __ _   _   | |__   _ __   __ _ _ _ __   ___
//          |  __/ _ \| | | | '_ \ / _` | '__| | | |  |  __| | '_ \ / _` | | '_ \ / _ \
//          | | | (_) | |_| | | | | (_| | |  | |_| |  | |____| | | | (_| | | | | |  __/
//          |_|  \___/ \__,_|_| |_|\__,_|_|   \__, |  |______|_| |_|\__, |_|_| |_|\___|
//                                             __/ |                 __/ |
//                                            |___/                 |___/
//==========================================================================================

//Basic config
#![allow(unused)]

use std::fs::read;

//Imports
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use ash::{Entry, Instance, vk};
use nalgebra_glm as glm;
use std::ffi::CString;

//Setup winit boilerplate
#[derive(Default, Debug)]
struct App {
    window: Option<Window>,
    size: winit::dpi::LogicalSize<f64>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Foundry Engine")
            .with_inner_size(self.size);
        self.window = Some(event_loop.create_window(window_attributes).unwrap())
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

fn create_instance() -> Option<ash::Instance> {
    let entry = unsafe { Entry::load().ok()? };
    let engine_name: CString = CString::new("No Engine").unwrap();
    let app_info = vk::ApplicationInfo {
        api_version: vk::make_api_version(0, 1, 0, 0),
        p_engine_name: engine_name.as_ptr(),

        ..Default::default()
    };
    let create_info = vk::InstanceCreateInfo {
        p_application_info: &app_info,
        enabled_layer_count: 0,
        ..Default::default()
    };
    let instance = unsafe { entry.create_instance(&create_info, None).ok()? };
    return Some(instance);
}

struct HelloTriangleApp;

impl HelloTriangleApp {
    fn run(&mut self, window_width: f64, window_height: f64) {
        HelloTriangleApp::init_window(window_width, window_height);
        HelloTriangleApp::init_vulkan();
        HelloTriangleApp::main_loop();
        HelloTriangleApp::cleanup();
    }
    fn init_vulkan() {
        create_instance();
    }
    fn main_loop() {}
    fn cleanup() {}
    fn init_window(window_width: f64, window_height: f64) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app = App::default();
        app.size = winit::dpi::LogicalSize::new(window_width, window_height);
        //let mut app = App { window: new_window };
        event_loop.run_app(&mut app);
    }
}

fn main() {
    //Vulkan Setup
    let mut app: HelloTriangleApp = HelloTriangleApp;
    app.run(400.0, 300.0);
}
