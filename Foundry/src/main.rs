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

const FIRST_PRIORITY: f32 = 1.0;

use ash::ext::surface_maintenance1;
use ash::khr::get_physical_device_properties2;
use ash::vk::PFN_vkEnumeratePhysicalDevices;
use ash_window;
#[allow(deprecated)]
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
//Imports
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use ash::{self, Entry, Instance, vk};
use nalgebra_glm::{self as glm, any};
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
        self.init_vulkan();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap();
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
                    mac_extension_names.push(vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_NAME.as_ptr());
                    mac_extension_names.push(ash::khr::surface::NAME.as_ptr());
                    mac_extension_names.push(ash::ext::metal_surface::NAME.as_ptr());
                    //MAKE SURE TO INCREASE COUNT
                    create_info.pp_enabled_layer_names = enabled_layer_names.as_ptr();
                    create_info.pp_enabled_extension_names = mac_extension_names.as_ptr();
                    create_info.enabled_extension_count = 4;
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
        let extension_supported: bool = check_extenstion_support(instance, &device);
        return (properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
            && extension_supported);
    };
}

fn check_extenstion_support(instance: &ash::Instance, device: &vk::PhysicalDevice) -> bool {
    let wanted_extension_names: Vec<&CStr> = vec![vk::KHR_SWAPCHAIN_NAME];
    unsafe {
        let available_extensions: Vec<vk::ExtensionProperties> = instance
            .enumerate_device_extension_properties(*device)
            .expect("failed to enumerate extensions in check_extenstion_support");
        let mut extensions_supported = 0;
        for extension in available_extensions.iter() {
            let name = extension.extension_name;
            println!("{:?}", CStr::from_ptr(name.as_ptr()).to_string_lossy());
            for extension in wanted_extension_names.iter() {
                if (CStr::from_ptr(name.as_ptr()) == CStr::from_ptr(extension.as_ptr())) {
                    println!("Found!!!!");
                    extensions_supported += 1;
                }
            }
        }
        return wanted_extension_names.len() == extensions_supported;
    }
}

//Vulkan app struct that ties everything together (winit, vulkan, and game engine stuff in the
//future)
#[derive(Default)]
struct HelloTriangleApp {
    window: Option<Window>,
    size: winit::dpi::LogicalSize<f64>,
    event_loop: Option<EventLoop<()>>,
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
    logical_device: Option<ash::Device>,
    family_indicies: QueueFamilyIndices,
    graphics_queue: Option<vk::Queue>,
    present_queue: Option<vk::Queue>,
    surface: Option<vk::SurfaceKHR>,
}

#[derive(Default)]
struct QueueFamilyIndices {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
}

impl QueueFamilyIndices {
    fn is_complete(&self) -> bool {
        println!("Graphics index: {:?}", self.graphics_family);
        println!("Preseent index: {:?}", self.present_family);
        return self.graphics_family.is_some() && self.present_family.is_some();
    }
}

impl HelloTriangleApp {
    fn run(&mut self, window_width: f64, window_height: f64) {
        self.size = winit::dpi::LogicalSize::new(window_width, window_height);
        let event_loop = self.load_window_early();
        self.init_window(event_loop, window_width, window_height);
        self.main_loop();
        self.cleanup();
        println!("Shutdown complete");
    }

    #[allow(deprecated)]
    //Depreciated code is using EventLoop<> instead of ActiveEventLoop
    fn load_window_early(&mut self) -> EventLoop<()> {
        let window_attributes = Window::default_attributes()
            .with_title("Foundry Engine")
            .with_inner_size(self.size);
        let event_loop = EventLoop::new().unwrap();
        self.window = Some(event_loop.create_window(window_attributes).unwrap());
        event_loop
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
        self.create_surface();
        self.pick_physical_device();
        self.find_queue_families();
        self.create_logical_device();
        self.retrieve_queue_handles();
    }
    fn main_loop(&self) {}
    fn cleanup(&self) {
        let Some(instance) = &self.vulkan_context.instance else {
            println!("Instance does not exist");
            return;
        };
        let Some(logical_device) = &self.vulkan_context.logical_device else {
            panic!("No logical device when cleaning up");
        };
        let Some(surface) = self.vulkan_context.surface else {
            panic!("No surface when cleaning up");
        };
        let Some(entry) = &self.vulkan_context.entry else {
            panic!("No entry when cleaning up");
        };

        let Some(instance) = &self.vulkan_context.instance else {
            panic!("No instance when cleaning up");
        };
        unsafe {
            let surface_instance = ash::khr::surface::Instance::new(entry, instance);
            surface_instance.destroy_surface(surface, None);
            logical_device.destroy_device(None);
            instance.destroy_instance(None);
            println!("Destroyed instance Successfully");
        }
    }
    fn init_window(&mut self, event_loop: EventLoop<()>, window_width: f64, window_height: f64) {
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

    fn pick_physical_device(&mut self) {
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
                            physical_device = *device;
                            break;
                        }
                    }
                    if (physical_device == vk::PhysicalDevice::null()) {
                        panic!("No stable devices found");
                    }

                    self.vulkan_context.physical_device = Some(physical_device);
                }
                Err(e) => {
                    panic!("{:?}", e);
                }
            }
        }
    }

    fn find_queue_families(&mut self) {
        let Some(device) = self.vulkan_context.physical_device else {
            panic!("No physical_device when finding families");
        };
        let Some(instance) = &self.vulkan_context.instance else {
            panic!("No instance when finding families");
        };
        let Some(entry) = &self.vulkan_context.entry else {
            panic!("No entry when calling find_queue_families");
        };
        let Some(surface) = self.vulkan_context.surface else {
            panic!("No surface when calling find_queue_families");
        };
        let mut indices = QueueFamilyIndices {
            ..Default::default()
        };
        let surface_instance = ash::khr::surface::Instance::new(entry, instance);
        unsafe {
            let queue_families: Vec<vk::QueueFamilyProperties> =
                instance.get_physical_device_queue_family_properties(device);
            let mut i = 0;
            for family in queue_families.iter() {
                if (family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                    && !indices.graphics_family.is_some())
                {
                    indices.graphics_family = Some(i as u32);
                }
                let present_support = surface_instance
                    .get_physical_device_surface_support(device, i, surface)
                    .expect("Failed to check surface support");
                if (!indices.present_family.is_some() && present_support) {
                    indices.present_family = Some(i as u32);
                }
                i += 1;
            }
        }
        if (!indices.is_complete()) {
            panic!("Did not find all needed indices");
        }
        self.vulkan_context.family_indicies = indices;
        return;
    }

    #[allow(deprecated)]
    fn create_logical_device(&mut self) {
        let indices = &self.vulkan_context.family_indicies;
        //Remember to add indices to this list
        //BUG
        let Some(graphics_index) = indices.graphics_family else {
            panic!("");
        };
        let Some(present_index) = indices.present_family else {
            panic!("");
        };
        let indices_vec = vec![graphics_index, present_index];
        let mut queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = Vec::new();

        for queue in indices_vec.iter() {
            let queue_create_info = vk::DeviceQueueCreateInfo {
                queue_count: 1,
                queue_family_index: *queue,
                p_queue_priorities: &FIRST_PRIORITY,
                ..Default::default()
            };
            queue_create_infos.push(queue_create_info);
        }

        let device_features: vk::PhysicalDeviceFeatures = vk::PhysicalDeviceFeatures {
            ..Default::default()
        };
        let mut device_extensions: Vec<*const i8> = Vec::new();

        //Port to MoltenVK if needed
        if (std::env::consts::OS == "macos") {
            device_extensions.push(vk::KHR_PORTABILITY_SUBSET_NAME.as_ptr());
        }

        let mut create_info: vk::DeviceCreateInfo = vk::DeviceCreateInfo {
            p_queue_create_infos: queue_create_infos.as_ptr(),
            queue_create_info_count: 1,
            p_enabled_features: &device_features,
            pp_enabled_extension_names: device_extensions.as_ptr(),
            enabled_extension_count: 1,
            ..Default::default()
        };
        let mut enabled_layer_names: Vec<*const i8> = Vec::new();
        for item in self.vulkan_context.validation_layer_names.iter() {
            enabled_layer_names.push(item.as_ptr());
        }
        if self.vulkan_context.validation_layers_enabaled {
            create_info.enabled_layer_count =
                self.vulkan_context.validation_layer_names.len() as u32;
            create_info.pp_enabled_layer_names = enabled_layer_names.as_ptr();
        } else {
            create_info.enabled_layer_count = 0;
        }
        let Some(instance) = &self.vulkan_context.instance else {
            panic!("No instance when creating logical device");
        };
        let Some(physical_device) = self.vulkan_context.physical_device else {
            panic!("No physical_device when creating logical device");
        };
        unsafe {
            match instance.create_device(physical_device, &create_info, None) {
                Ok(logical_device) => {
                    self.vulkan_context.logical_device = Some(logical_device);
                }
                Err(e) => {
                    panic!("{:?}", e);
                }
            }
        }
    }
    fn retrieve_queue_handles(&mut self) {
        let Some(device) = &self.vulkan_context.logical_device else {
            panic!("No physical_device when calling retrieve_queue_handles");
        };
        let indices = &self.vulkan_context.family_indicies;
        let Some(graphics_index) = indices.graphics_family else {
            panic!();
        };
        let Some(present_index) = indices.present_family else {
            panic!();
        };
        unsafe {
            let device_queue: vk::Queue = device.get_device_queue(graphics_index, 0);
            self.vulkan_context.graphics_queue = Some(device_queue);

            let present_queue: vk::Queue = device.get_device_queue(present_index, 0);
            self.vulkan_context.present_queue = Some(present_queue);
        };
    }

    #[allow(deprecated)]
    fn create_surface(&mut self) {
        let Some(instance) = &self.vulkan_context.instance else {
            panic!("No instance when calling create_surface");
        };
        let Some(entry) = &self.vulkan_context.entry else {
            panic!("No entry when calling create_surface");
        };
        let Some(window) = &self.window else {
            panic!("No window when calling create_surface");
        };
        let display_handle = window
            .raw_display_handle()
            .expect("failed to get display_handle");
        let window_handle = window
            .raw_window_handle()
            .expect("failed to get window_handle");
        if (std::env::consts::OS == "macos") {
            //Mac implementation
            let surface: Result<vk::SurfaceKHR, ash::vk::Result> = unsafe {
                ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)
            };
            /*
                        let metal_surface_loader = ash::ext::metal_surface::Instance::new(entry, instance);
                        let surface_loader = ash::khr::surface::Instance::new(entry, instance);
                        unsafe {
                            let create_info = vk::MetalSurfaceCreateInfoEXT {
                                ..Default::default()
                            };
                            let metal_surface = metal_surface_loader.create_metal_surface(&create_info, None);

                            let Ok(surface) = metal_surface else {
                                panic!("metal surface creation failed");
                            };
                            self.vulkan_context.surface = Some(surface);
                        }
            */
            let Ok(surface) = surface else {
                panic!("metal surface creation failed");
            };

            self.vulkan_context.surface = Some(surface);

            println!("Created SurfaceKHR Successfully");
        } else {
            panic!(
                "No implementation for creating a window for {:?}",
                std::env::consts::OS
            )
        }
    }
}

fn main() {
    //Vulkan Setup
    let mut app: HelloTriangleApp = HelloTriangleApp {
        ..Default::default()
    };
    app.run(800.0, 600.0);
}
