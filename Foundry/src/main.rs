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
const VALIDATION_LAYERS: &[&str] = &["VK_LAYER_KHRONOS_validation"];

use ash::khr::get_physical_device_properties2;
use ash::vk::PFN_vkEnumeratePhysicalDevices;
//Imports
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use ash::{self, Entry, Instance, vk};
use nalgebra_glm as glm;
use std::ffi::{CStr, CString};
use std::hash::Hash;

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
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

//----------------Helper funtions-----------------
//Change to entry::linked() if having problems
fn create_instance(context: &mut VulkanContext) -> Option<ash::Instance> {
    let Some(entry) = &context.entry else {
        panic!("Sent invalid entry to create_instance");
    };

    let engine_name: CString = CString::new("No Engine").unwrap();
    let app_info = vk::ApplicationInfo {
        api_version: vk::make_api_version(0, 1, 0, 0),
        p_engine_name: engine_name.as_ptr(),

        ..Default::default()
    };
    let mut layer_count: u32 = 0;
    let mut enabled_layer_names: Vec<*const i8> = Vec::new();

    let validation = vec![
        CString::new("VK_LAYER_KHRONOS_validation")
            .unwrap()
            .as_ptr(),
    ];

    if cfg!(debug_assertions) {
        //Save Cstrings in vulkan_context and make a list of pointers to those
        layer_count = VALIDATION_LAYERS.len() as u32;
        for item in VALIDATION_LAYERS.iter() {
            println!("{:?}", item);
            context
                .validation_layer_names
                .push(CString::new(*item).expect("ih on"));
        }
        for item in context.validation_layer_names.iter() {
            enabled_layer_names.push(item.as_ptr());
        }
    }
    let mut create_info = vk::InstanceCreateInfo {
        p_application_info: &app_info,
        enabled_layer_count: layer_count,
        pp_enabled_layer_names: enabled_layer_names.as_ptr(),
        ..Default::default()
    };
    unsafe {
        match entry.create_instance(&create_info, None) {
            Ok(instance) => {
                print!("Created Vulkan Instance");
                return Some(instance);
            }
            Err(result) => {
                //Check if there is a driver issue on mac (likely need to port MoltenVK)
                if (std::env::consts::OS == "macos"
                    && result == vk::Result::ERROR_INCOMPATIBLE_DRIVER)
                {
                    println!("Error: Incompatible driver, making a port");
                    //Make a MoltenVK port for mac
                    let instance_flags: vk::InstanceCreateFlags =
                        vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
                    let mut mac_extension_names = Vec::new();
                    mac_extension_names.push(vk::KHR_PORTABILITY_ENUMERATION_NAME.as_ptr());
                    create_info.pp_enabled_layer_names = enabled_layer_names.as_ptr();
                    create_info.pp_enabled_extension_names = mac_extension_names.as_ptr();
                    create_info.enabled_extension_count = 1;
                    create_info.flags = instance_flags;

                    match entry.create_instance(&create_info, None) {
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
}

fn is_device_stable(instance: &ash::Instance, device: &vk::PhysicalDevice) -> bool {
    unsafe {
        let properties = instance.get_physical_device_properties(*device);
        let features = instance.get_physical_device_features(*device);
        let name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }.to_str();
        return (properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU);
    };
}

//Vulkan app struct that ties everything together (winit, vulkan, and game engine stuff in the
//future)
#[derive(Default)]
struct HelloTriangleApp {
    window: Option<Window>,
    size: winit::dpi::LogicalSize<f64>,
    vulkan_context: VulkanContext,
}
//Holds all vulkan objects in a single struct to controll lifetimes more precisely
#[derive(Default)]
struct VulkanContext {
    instance: Option<ash::Instance>,
    entry: Option<ash::Entry>,
    validation_layers_enabaled: bool,
    validation_layer_names: Vec<CString>,
    physical_device: Option<vk::PhysicalDevice>,
    family_indicies: QueueFamilyIndices,
}

#[derive(Default)]
struct QueueFamilyIndices {
    graphics_family: u32,
}

impl HelloTriangleApp {
    fn run(&mut self, window_width: f64, window_height: f64) {
        HelloTriangleApp::init_vulkan(self);
        HelloTriangleApp::pick_physical_device(self);

        HelloTriangleApp::find_queue_families(self);
        HelloTriangleApp::init_window(self, window_width, window_height);
        HelloTriangleApp::main_loop();
        HelloTriangleApp::cleanup(&self);
        println!("Shutdown complete");
    }

    fn init_vulkan(&mut self) {
        unsafe {
            match Entry::load() {
                Err(result) => {
                    panic!("Failed to load an entry");
                }
                Ok(entry) => {
                    self.vulkan_context.entry = Some(entry);
                }
            }
        }
        if (!self.check_validation_layers()) {
            panic!("uh oh");
        }

        let instance_result: Option<Instance> = create_instance(&mut self.vulkan_context);
        unsafe {
            self.vulkan_context.instance = instance_result;
        }
    }
    fn main_loop() {}
    fn cleanup(&self) {
        let Some(instance) = &self.vulkan_context.instance else {
            println!("Instance does not exist");
            return;
        };
        unsafe {
            instance.destroy_instance(None);
            println!("Destroyed instance Successfully");
        }
    }
    fn init_window(&mut self, window_width: f64, window_height: f64) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        self.size = winit::dpi::LogicalSize::new(window_width, window_height);
        event_loop.run_app(self);
    }

    fn check_validation_layers(&mut self) -> bool {
        if !cfg!(debug_assertions) {
            return true;
        }
        let Some(entry) = &self.vulkan_context.entry else {
            panic!("Entry is invalid when seting in setup_validation_layers");
        };
        match unsafe { entry.enumerate_instance_layer_properties() } {
            Ok(available_layers) => {
                //println!("found layers {:?}", layer_properties_vec.len());
                for wanted_layer in VALIDATION_LAYERS {
                    let mut layer_found = false;
                    for available_layer in &available_layers {
                        unsafe {
                            let available_layer_name =
                                CStr::from_ptr(available_layer.layer_name.as_ptr());
                            let string_converstion_result = available_layer_name.to_str();
                            let Ok(a_layer_name) = string_converstion_result else {
                                panic!("invalid string converstion");
                            };
                            if wanted_layer == &a_layer_name {
                                layer_found = true;
                            }
                        };
                    }
                    if (!layer_found) {
                        println!("Couldn't find layer {:?}", wanted_layer);
                        return false;
                    }
                }
                return true;
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    fn pick_physical_device(&mut self) -> vk::PhysicalDevice {
        //physical_device: vk::PhysicalDevice;
        let Some(instance) = &self.vulkan_context.instance else {
            panic!("invalid instance when getting physical devices");
        };
        unsafe {
            match instance.enumerate_physical_devices() {
                Ok(physical_device_list) => {
                    if (physical_device_list.len() == 0) {
                        panic!("found no physical devices");
                    }
                    let Some(instance) = &self.vulkan_context.instance else {
                        panic!("No instance in pick_physical_device()");
                    };
                    let mut physical_device: vk::PhysicalDevice = vk::PhysicalDevice::null();
                    for device in physical_device_list.iter() {
                        if (is_device_stable(&instance, device)) {
                            println!("yay");
                            physical_device = *device;
                            break;
                        }
                    }
                    if (physical_device == vk::PhysicalDevice::null()) {
                        panic!("No stable devices found");
                    }

                    self.vulkan_context.physical_device = Some(physical_device);
                    return physical_device_list[0];
                }
                Err(e) => {
                    panic!("{:?}", e);
                }
            }
        }
    }

    fn find_queue_families(&mut self) -> QueueFamilyIndices {
        let Some(device) = self.vulkan_context.physical_device else {
            panic!("No physical_device when finding families");
        };
        let Some(instance) = &self.vulkan_context.instance else {
            panic!("No instance when finding families");
        };
        let mut indices = QueueFamilyIndices {
            ..Default::default()
        };
        unsafe {
            let queue_families: Vec<vk::QueueFamilyProperties> =
                instance.get_physical_device_queue_family_properties(device);
            println!("{:?}", queue_families);
            for family in queue_families.iter() {
                println!("{:?}", family.queue_flags);
            }
        }
        return indices;
    }
}

fn main() {
    //Vulkan Setup
    let mut app: HelloTriangleApp = HelloTriangleApp {
        ..Default::default()
    };
    app.run(400.0, 300.0);
}
