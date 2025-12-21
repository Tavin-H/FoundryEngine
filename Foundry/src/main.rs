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

//use ash::vk::{
//    InstanceCreateFlags, KHR_PORTABILITY_ENUMERATION_SPEC_VERSION, KhrPortabilityEnumerationFn,
//    PrimitiveTopology,
//};
//Imports
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use ash::{self, Entry, Instance, vk};
use nalgebra_glm as glm;
use std::ffi::CString;

//Setup winit boilerplate
#[derive(Default)]
struct WinitApp {
    window: Option<Window>,
    size: winit::dpi::LogicalSize<f64>,
    instance: Option<ash::Instance>,
}

impl ApplicationHandler for HelloTriangleApp {
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
                //Cleanup here
                //HelloTriangleApp::cleanup(&self);
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

//Change to entry::linked() if having problems
fn create_instance(entry: ash::Entry) -> Option<ash::Instance> {
    //let entry = unsafe { Entry::load().ok()? };
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
    unsafe {
        match entry.create_instance(&create_info, None) {
            Ok(instance) => {
                print!("yipee");
                return Some(instance);
            }
            Err(result) => {
                if (std::env::consts::OS == "macos"
                    && result == vk::Result::ERROR_INCOMPATIBLE_DRIVER)
                {
                    println!("MoltenVK not setup");
                    //Make a MoltenVK port for mac
                    let instance_flags: vk::InstanceCreateFlags =
                        vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
                    let mut mac_extension_names = Vec::new();
                    mac_extension_names.push(vk::KHR_PORTABILITY_ENUMERATION_NAME.as_ptr());
                    //extension_names.push(vk::KhrGetPhysicalDeviceProperties2Fn::name().as_ptr());
                    let mac_create_info = vk::InstanceCreateInfo {
                        p_application_info: &app_info,
                        enabled_layer_count: 0,
                        flags: instance_flags,
                        pp_enabled_extension_names: mac_extension_names.as_ptr(),
                        enabled_extension_count: 1,
                        ..Default::default()
                    };
                    match entry.create_instance(&mac_create_info, None) {
                        Ok(instance) => {
                            println!("Successfully created mac port");
                            return Some(instance);
                        }
                        Err(result) => {
                            panic!("{:?}", result);
                        }
                    }
                }
            }
        }
    }
    panic!("Failure: No vk instance created");
    return None;
    //let instance = unsafe { entry.create_instance(&create_info, None).ok()? };
    //return Some(instance);
}

#[derive(Default)]
struct HelloTriangleApp {
    window: Option<Window>,
    size: winit::dpi::LogicalSize<f64>,
    instance: Option<ash::Instance>,
}

impl HelloTriangleApp {
    fn run(&mut self, window_width: f64, window_height: f64) {
        let entry = unsafe { Entry::load().ok() };
        let Some(entry) = entry else {
            panic!("no entry created");
        };
        HelloTriangleApp::init_vulkan(self, entry);
        HelloTriangleApp::init_window(self, window_width, window_height);
        HelloTriangleApp::main_loop();
        HelloTriangleApp::cleanup(&self);
    }
    fn init_vulkan(&mut self, entry: ash::Entry) {
        let instance_result: Option<Instance> = create_instance(entry);
        unsafe {
            self.instance = instance_result;
        }
    }
    fn main_loop() {}
    fn cleanup(&self) {
        //Called in WindowEvent::CloseRequested in ApplicationHandler
        let Some(instance) = &self.instance else {
            println!("Instance does not exist");
            return;
        };
        unsafe {
            println!("Destroying instance");
            instance.destroy_instance(None);
            println!("Destroyed instance Successfully");
        }
    }
    fn init_window(&mut self, window_width: f64, window_height: f64) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        /*
        let mut app = WinitApp {
            ..WinitApp::default()
        };
        */
        self.size = winit::dpi::LogicalSize::new(window_width, window_height);
        //app.size = winit::dpi::LogicalSize::new(window_width, window_height);
        event_loop.run_app(self);
    }
}

fn main() {
    //Vulkan Setup
    let mut app: HelloTriangleApp = HelloTriangleApp {
        instance: None,
        ..Default::default()
    };
    app.run(400.0, 300.0);
}
