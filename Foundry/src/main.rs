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
const WANTED_EXTENSION_NAMES: &[&CStr] = &[vk::KHR_SWAPCHAIN_NAME];
const FIRST_PRIORITY: f32 = 1.0;

//Window
use ash_window;
#[allow(deprecated)]
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

//Ash
use ash::ext::surface_maintenance1;
use ash::khr::get_physical_device_properties2;
use ash::vk::{PFN_vkEnumeratePhysicalDevices, SamplerCubicWeightsCreateInfoQCOM};
use ash::{self, Entry, Instance, vk};

//Math
use nalgebra_glm::{self as glm, any, log};

//General
use bytemuck::cast;
use num::clamp;
use std::ffi::{CStr, CString};
use std::fs;
use std::hash::Hash;
use std::mem::swap;
use std::panic;

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
    let mut extension_names: Vec<*const i8> = Vec::new();
    extension_names.push(ash::khr::surface::NAME.as_ptr());
    if (std::env::consts::OS == "windows") {
        extension_names.push(ash::vk::KHR_WIN32_SURFACE_NAME.as_ptr());
    }
    let mut create_info = vk::InstanceCreateInfo {
        p_application_info: &app_info,
        enabled_layer_count: layer_count,
        pp_enabled_layer_names: enabled_layer_names.as_ptr(),
        pp_enabled_extension_names: extension_names.as_ptr(),
        enabled_extension_count: extension_names.len() as u32,

        ..Default::default()
    };

    unsafe {
        match entry.create_instance(&create_info, None) {
            Ok(instance) => {
                println!("Created Vulkan Instance");
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
                if (std::env::consts::OS == "windows") {
                    println!("Windows problem");
                    panic!("{}", result);
                }
            }
        }
    }
    panic!("Failure: No vk instance created");
    return None;
}

fn is_device_stable(
    entry: &ash::Entry,
    instance: &ash::Instance,
    device: &vk::PhysicalDevice,
    surface: &vk::SurfaceKHR,
) -> bool {
    unsafe {
        let properties = instance.get_physical_device_properties(*device);
        let features = instance.get_physical_device_features(*device);
        let name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }.to_str();

        let extension_supported: bool = check_extenstion_support(instance, &device);

        let mut swapchain_supported: bool = false;
        if (extension_supported) {
            let swapchain_support_details =
                query_swapchain_support(entry, instance, device, surface);

            swapchain_supported = swapchain_support_details.formats.len() != 0
                && swapchain_support_details.present_modes.len() != 0;
        }
        return (properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
            && extension_supported
            && swapchain_supported);
    };
}

fn check_extenstion_support(instance: &ash::Instance, device: &vk::PhysicalDevice) -> bool {
    unsafe {
        let available_extensions: Vec<vk::ExtensionProperties> = instance
            .enumerate_device_extension_properties(*device)
            .expect("failed to enumerate extensions in check_extenstion_support");
        let mut extensions_supported = 0;
        for extension in available_extensions.iter() {
            let name = extension.extension_name;
            for extension in WANTED_EXTENSION_NAMES.iter() {
                if (CStr::from_ptr(name.as_ptr()) == CStr::from_ptr(extension.as_ptr())) {
                    extensions_supported += 1;
                }
            }
        }
        return WANTED_EXTENSION_NAMES.len() == extensions_supported;
    }
}

fn query_swapchain_support(
    entry: &Entry,
    instance: &Instance,
    device: &vk::PhysicalDevice,
    surface: &vk::SurfaceKHR,
) -> SwapChainSupportDetails {
    unsafe {
        let surface_instance = ash::khr::surface::Instance::new(entry, instance);

        // get the capabilities
        let capability_details = surface_instance
            .get_physical_device_surface_capabilities(*device, *surface)
            .expect("failed");

        // find formats
        let format_details: Vec<vk::SurfaceFormatKHR> = surface_instance
            .get_physical_device_surface_formats(*device, *surface)
            .expect("failed");

        // find present modes
        let present_mode_details: Vec<vk::PresentModeKHR> = surface_instance
            .get_physical_device_surface_present_modes(*device, *surface)
            .expect("failed");

        return SwapChainSupportDetails {
            capabilities: capability_details,
            formats: format_details,
            present_modes: present_mode_details,
        };
    }
}

fn choose_swap_surface_format(
    available_formats: &Vec<vk::SurfaceFormatKHR>,
) -> vk::SurfaceFormatKHR {
    for available_format in available_formats.iter() {
        if (available_format.format == vk::Format::B8G8R8A8_SRGB
            && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
        {
            return *available_format;
        }
    }

    return available_formats[0];
}

fn choose_swap_surface_mode(available_modes: &Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
    for available_mode in available_modes.iter() {
        if (*available_mode == vk::PresentModeKHR::MAILBOX) {
            println!("Found mailbox");
            return *available_mode;
        }
    }
    println!("Did not find mailbox present mode, using FIFO as default");
    return vk::PresentModeKHR::FIFO;
}

fn choose_swap_extent(
    surface_capabilities: vk::SurfaceCapabilitiesKHR,
    window: &Window,
) -> vk::Extent2D {
    if (surface_capabilities.current_extent.width != u32::MAX) {
        return surface_capabilities.current_extent;
    } else {
        let actual_size = window.inner_size();
        let return_size = vk::Extent2D {
            width: clamp(
                actual_size.width,
                surface_capabilities.min_image_extent.width,
                surface_capabilities.max_image_extent.width,
            ),
            height: clamp(
                actual_size.height,
                surface_capabilities.min_image_extent.height,
                surface_capabilities.max_image_extent.height,
            ),
        };
        return return_size;
    }
}

fn read_file(file_path: &String) -> Vec<u8> {
    //let contents = fs::read(ffile_path).expect("Failed to read file");
    match fs::read(file_path) {
        Ok(file) => {
            return file;
        }
        Err(e) => {
            panic!("{}", e);
        }
    }
}

fn convert_u8_to_u32_vec(bytes: &Vec<u8>) -> Vec<u32> {
    //let bytes_as_u32: Vec<u32> = bytes.into_iter().map(|x| x as u32).collect();
    let converted_bytes: &[u8] = &bytes;
    let bytes_as_u32: &[u32] = bytemuck::cast_slice(converted_bytes);
    return bytes_as_u32.to_vec();
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
    //Basics
    instance: Option<ash::Instance>,
    entry: Option<ash::Entry>,
    validation_layers_enabaled: bool,
    validation_layer_names: Vec<CString>,
    //Devices
    physical_device: Option<vk::PhysicalDevice>,
    logical_device: Option<ash::Device>,
    //Queue Indices Struct
    family_indicies: QueueFamilyIndices,
    //Queue Handles
    graphics_queue: Option<vk::Queue>,
    present_queue: Option<vk::Queue>,
    //Surface
    surface: Option<vk::SurfaceKHR>,
    swap_chain_details: Option<SwapChainSupportDetails>,
    swap_chain: Option<ash::vk::SwapchainKHR>,
    swap_chain_images: Vec<ash::vk::Image>,
    swap_chain_format: Option<vk::SurfaceFormatKHR>,
    swap_chain_extent_used: Option<vk::Extent2D>,
    swap_chain_image_views: Vec<vk::ImageView>,

    //Shader Info
    shader_list: Vec<vk::ShaderModule>,
}
struct SwapChainSupportDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

#[derive(Default)]
struct QueueFamilyIndices {
    graphics_family: u32,
    present_family: u32,
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
        self.create_swapchain();
        self.create_image_views();
        self.create_graphics_pipeline();
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
        let Some(swapchain) = self.vulkan_context.swap_chain else {
            panic!("No swapchain when cleaning up");
        };
        unsafe {
            let swapchain_device = ash::khr::swapchain::Device::new(instance, logical_device);
            let surface_instance = ash::khr::surface::Instance::new(entry, instance);
            for image_view in self.vulkan_context.swap_chain_image_views.iter() {
                logical_device.destroy_image_view(*image_view, None);
            }
            for shader_module in self.vulkan_context.shader_list.iter() {
                logical_device.destroy_shader_module(*shader_module, None);
            }
            swapchain_device.destroy_swapchain(swapchain, None);
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
        let Some(entry) = &self.vulkan_context.entry else {
            panic!("No entry in pick_physical_device");
        };
        let Some(surface) = &self.vulkan_context.surface else {
            panic!("No surface in pick_physical_device");
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
                        if (is_device_stable(entry, &instance, device, surface)) {
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
            let mut found_graphics_index: bool = false;
            let mut found_present_index: bool = false;
            let queue_families: Vec<vk::QueueFamilyProperties> =
                instance.get_physical_device_queue_family_properties(device);
            let mut i = 0;
            for family in queue_families.iter() {
                if (family.queue_flags.contains(vk::QueueFlags::GRAPHICS) && !found_graphics_index)
                {
                    indices.graphics_family = i as u32;
                    found_graphics_index = true;
                }
                let present_support = surface_instance
                    .get_physical_device_surface_support(device, i, surface)
                    .expect("Failed to check surface support");
                if (!found_present_index && present_support) {
                    indices.present_family = i as u32;
                    found_present_index = true;
                }
                i += 1;
            }
            if (!found_present_index && !found_graphics_index) {
                panic!("Did not find all needed indices");
            }
        }
        self.vulkan_context.family_indicies = indices;
    }

    #[allow(deprecated)]
    fn create_logical_device(&mut self) {
        let indices = &self.vulkan_context.family_indicies;
        //Remember to add indices to this list
        //BUG
        let indices_vec = vec![indices.graphics_family, indices.present_family];
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

        for extension in WANTED_EXTENSION_NAMES.iter() {
            device_extensions.push(extension.as_ptr());
        }
        //Port to MoltenVK if needed
        if (std::env::consts::OS == "macos") {
            device_extensions.push(vk::KHR_PORTABILITY_SUBSET_NAME.as_ptr());
        }

        let mut create_info: vk::DeviceCreateInfo = vk::DeviceCreateInfo {
            p_queue_create_infos: queue_create_infos.as_ptr(),
            queue_create_info_count: 1,
            p_enabled_features: &device_features,
            pp_enabled_extension_names: device_extensions.as_ptr(),
            enabled_extension_count: device_extensions.len() as u32,
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
        unsafe {
            let device_queue: vk::Queue = device.get_device_queue(indices.graphics_family, 0);
            self.vulkan_context.graphics_queue = Some(device_queue);

            let present_queue: vk::Queue = device.get_device_queue(indices.present_family, 0);
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
            IN CASE I WANT TO DO THIS MANUALLY LATER
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
            let surface: Result<vk::SurfaceKHR, ash::vk::Result> = unsafe {
                ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)
            };
            let Ok(surface) = surface else {
                panic!("metal surface creation failed");
            };
            self.vulkan_context.surface = Some(surface);
        }
    }
    fn create_swapchain(&mut self) {
        let Some(instance) = &self.vulkan_context.instance else {
            panic!("No instance when calling create_swapchain");
        };
        let Some(entry) = &self.vulkan_context.entry else {
            panic!("No entry when calling create_swapchain");
        };
        let Some(surface) = &self.vulkan_context.surface else {
            panic!("No surface when calling create_swapchain");
        };
        let Some(device) = &self.vulkan_context.physical_device else {
            panic!("No physical_device when calling create_swapchain");
        };
        let Some(window) = &self.window else {
            panic!("No window when calling create_swapchain");
        };

        let swapchain_support: SwapChainSupportDetails =
            query_swapchain_support(entry, instance, device, surface);
        let surface_format: vk::SurfaceFormatKHR =
            choose_swap_surface_format(&swapchain_support.formats);
        let present_mode: vk::PresentModeKHR =
            choose_swap_surface_mode(&swapchain_support.present_modes);
        let extent: vk::Extent2D = choose_swap_extent(swapchain_support.capabilities, window);
        let mut image_count = swapchain_support.capabilities.min_image_count + 1;
        if (swapchain_support.capabilities.max_image_count > 0
            && image_count > swapchain_support.capabilities.max_image_count)
        {
            image_count = swapchain_support.capabilities.max_image_count;
        }
        let indices = &self.vulkan_context.family_indicies;

        //Assume values are same
        //TUTORAL MENTIONS HOW TO USE POST PROCESSING KINDA
        let mut create_info = vk::SwapchainCreateInfoKHR {
            surface: *surface,
            min_image_count: image_count,
            image_color_space: surface_format.color_space,
            image_extent: extent,
            //WARNING THIS IS ANNOYING
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            pre_transform: swapchain_support.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode: present_mode,
            image_format: surface_format.format,
            clipped: vk::TRUE,
            old_swapchain: vk::SwapchainKHR::null(),
            ..Default::default()
        };

        if (indices.graphics_family == indices.present_family) {
            create_info.queue_family_index_count = 0;
            create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
        } else {
            println!("Not same");
            let indices = [indices.graphics_family, indices.present_family];
            create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            create_info.queue_family_index_count = 2;
            create_info.queue_family_indices(&indices);
        }
        let Some(logical_device) = &self.vulkan_context.logical_device else {
            panic!("No logical device in create_swapchain");
        };
        let swapchain_device = ash::khr::swapchain::Device::new(instance, logical_device);
        unsafe {
            match swapchain_device.create_swapchain(&create_info, None) {
                Ok(swapchain) => {
                    println!("Created Swapchain");
                    let swap_chain_images: Vec<ash::vk::Image> = swapchain_device
                        .get_swapchain_images(swapchain)
                        .expect("Failed to get swap_chain images");

                    //Store variables
                    self.vulkan_context.swap_chain_details = Some(swapchain_support);
                    self.vulkan_context.swap_chain = Some(swapchain);
                    self.vulkan_context.swap_chain_images = swap_chain_images;
                    self.vulkan_context.swap_chain_format = Some(surface_format);
                    self.vulkan_context.swap_chain_extent_used = Some(extent);
                }
                Err(e) => {
                    panic!("{:?}", e);
                }
            }
        }
    }

    fn create_image_views(&mut self) {
        let swap_chain_images = &self.vulkan_context.swap_chain_images;
        let Some(surface_format) = self.vulkan_context.swap_chain_format else {
            panic!("No format when calling create_image_views");
        };
        let Some(logical_device) = &self.vulkan_context.logical_device else {
            panic!("No logical_device when calling create_image_views");
        };
        let mut swap_chain_image_views: Vec<vk::ImageView> = Vec::new();
        let image_components = vk::ComponentMapping {
            r: vk::ComponentSwizzle::IDENTITY,
            g: vk::ComponentSwizzle::IDENTITY,
            b: vk::ComponentSwizzle::IDENTITY,
            a: vk::ComponentSwizzle::IDENTITY,
        };
        let image_subresource_range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };
        for image in swap_chain_images.iter() {
            let create_info = vk::ImageViewCreateInfo {
                image: *image,
                format: surface_format.format,
                view_type: vk::ImageViewType::TYPE_2D,
                components: image_components,
                subresource_range: image_subresource_range,
                ..Default::default()
            };
            unsafe {
                match logical_device.create_image_view(&create_info, None) {
                    Ok(image_view) => {
                        swap_chain_image_views.push(image_view);
                    }
                    Err(e) => {
                        panic!("{}", e);
                    }
                }
            }
        }
        self.vulkan_context.swap_chain_image_views = swap_chain_image_views;
    }

    fn create_shader_module(&mut self, bytes: Vec<u8>) -> vk::ShaderModule {
        let bytes_as_u32: Vec<u32> = convert_u8_to_u32_vec(&bytes);
        let create_info = vk::ShaderModuleCreateInfo {
            code_size: bytes.len(),
            p_code: bytes_as_u32.as_ptr(),
            ..Default::default()
        };
        let Some(logical_device) = &self.vulkan_context.logical_device else {
            panic!("No logical_device when calling create_shader_module");
        };
        unsafe {
            match logical_device.create_shader_module(&create_info, None) {
                Ok(shader_module) => shader_module,
                Err(e) => panic!("Failed to create_shader_module"),
            }
        }
    }

    fn create_graphics_pipeline(&mut self) {
        let vert_shader_path = String::from("./shaders/vert.spv");
        let frag_shader_path = String::from("./shaders/frag.spv");
        let vert_shader_code: Vec<u8> = read_file(&vert_shader_path);
        let frag_shader_code: Vec<u8> = read_file(&frag_shader_path);

        let vert_shader_module: vk::ShaderModule = self.create_shader_module(vert_shader_code);
        let frag_shader_module: vk::ShaderModule = self.create_shader_module(frag_shader_code);

        self.vulkan_context.shader_list.push(vert_shader_module);
        self.vulkan_context.shader_list.push(frag_shader_module);

        let vert_shader_create_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::VERTEX,
            module: vert_shader_module,
            p_name: "main".as_ptr() as *const i8,
            ..Default::default()
        };
        let frag_shader_create_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::VERTEX,
            module: frag_shader_module,
            p_name: "main".as_ptr() as *const i8,
            ..Default::default()
        };

        let shader_stages: Vec<vk::PipelineShaderStageCreateInfo> =
            vec![vert_shader_create_info, frag_shader_create_info];

        //--------FIXED FUNCTIONS--------

        //Vertex Input
        let dynamic_states: Vec<vk::DynamicState> =
            vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

        let dynamic_state = vk::PipelineDynamicStateCreateInfo {
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: dynamic_states.as_ptr(),
            ..Default::default()
        };

        let binding_descriptions_list: Vec<vk::VertexInputBindingDescription> = Vec::new();
        let attributes_list: Vec<vk::VertexInputAttributeDescription> = Vec::new();
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
            vertex_binding_description_count: 0,
            p_vertex_binding_descriptions: binding_descriptions_list.as_ptr(),
            vertex_attribute_description_count: 0,
            p_vertex_attribute_descriptions: attributes_list.as_ptr(),
            ..Default::default()
        };

        //Input assembly
        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::TRUE,
            ..Default::default()
        };

        //Viewport
        let Some(swapchain_extent) = self.vulkan_context.swap_chain_extent_used else {
            panic!("No swapchain_extent when calling create_graphics_pipeline");
        };
        let viewport = vk::Viewport {
            x: 0 as f32,
            y: 0 as f32,
            width: swapchain_extent.width as f32,
            height: swapchain_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
            ..Default::default()
        };
        let viewport_array: Vec<vk::Viewport> = vec![viewport];

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extent,
        };
        let scissor_array: Vec<vk::Rect2D> = vec![scissor];

        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: dynamic_states.as_ptr(),
            ..Default::default()
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo {
            viewport_count: 1,
            p_viewports: viewport_array.as_ptr(),
            scissor_count: 1,
            p_scissors: scissor_array.as_ptr(),
            ..Default::default()
        };
    }
}

fn main() {
    //Vulkan Setup
    let mut app: HelloTriangleApp = HelloTriangleApp {
        ..Default::default()
    };
    app.run(800.0, 600.0);
}
