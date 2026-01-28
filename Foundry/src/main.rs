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
const MAX_FRAMES_IN_FLIGHT: u32 = 2;
//static vertices: [Vertex 3];

//const main_name: CString = CString::new("main").expect("failed to load c string");
//const MAIN_NAME: &[i8] = &[109, 97, 105, 110, 0];
const MAIN_NAME: *const i8 = [109 as i8, 97 as i8, 105 as i8, 110 as i8, 0 as i8].as_ptr();
//Window
use ash_window;
use image;
#[allow(deprecated)]
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use vulkan_headers::vulkan::vulkan::VkExternalMemoryTensorCreateInfoARM;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{self, Window, WindowAttributes, WindowId};

//Ash
use ash::ext::{device_memory_report, surface_maintenance1};
use ash::khr::get_physical_device_properties2;
use ash::vk::{
    CommandBufferUsageFlags, PFN_vkEnumeratePhysicalDevices,
    PipelineShaderStageRequiredSubgroupSizeCreateInfoEXT, PresentInfoKHR,
    SamplerCubicWeightsCreateInfoQCOM, SetLatencyMarkerInfoNV,
};
use ash::{self, Entry, Instance, vk};

//Math
use nalgebra_glm::{self as glm, any, log, pi};

//General
use bytemuck::{cast, offset_of};
use num::clamp;
use std::ffi::{CStr, CString};
use std::fs;
use std::hash::Hash;
use std::mem::swap;
use std::panic;
use std::path::PathBuf;
use std::ptr;
use std::sync::Arc;
use std::thread::current;

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
                self.closing = true;
                event_loop.exit();
                let Some(logical_device) = &self.vulkan_context.logical_device else {
                    panic!("AAA");
                };
                unsafe {
                    logical_device.device_wait_idle();
                }
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap();
            }
            WindowEvent::Resized(size) => {
                if (size.width == 0 && size.height == 0) {
                    self.minimized = true;
                    return;
                }
                self.window_resized = true;
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                println!("{:?} {:?}", event.physical_key, event.state);
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // update logic here
        if (!self.closing) {
            self.draw_frame();
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }
}

//----------------Helper functions-----------------

//Takes creation info and returns a buffer as well as the device memory where the buffer is
//located
fn create_buffer(
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    property_flags: vk::MemoryPropertyFlags,
    context: &VulkanContext,
) -> (vk::Buffer, vk::DeviceMemory) {
    let Some(logical_device) = &context.logical_device else {
        panic!("No logical_device when calling create_vertex_buffer");
    };
    let Some(physical_device) = &context.physical_device else {
        panic!("No physical_device when calling create_vertex_buffer");
    };
    let Some(instance) = &context.instance else {
        panic!("No instance when calling create_vertex_buffer");
    };
    let buffer_info = vk::BufferCreateInfo {
        size: size as u64,
        usage: usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };

    //Declare as null (I know this is bad practice but I'm being carefull)
    //At least I'm not using C++
    let mut result: ash::prelude::VkResult<vk::Buffer> = Ok(vk::Buffer::null());
    unsafe {
        result = logical_device.create_buffer(&buffer_info, None);
    }

    let Ok(buffer) = result else {
        panic!("Failed to create vertex buffer");
    };

    unsafe {
        let mem_requirements: vk::MemoryRequirements =
            logical_device.get_buffer_memory_requirements(buffer);
        let device_memory_properties =
            instance.get_physical_device_memory_properties(*physical_device);

        let found_memory_type_index = find_memory_type(
            mem_requirements.memory_type_bits,
            property_flags,
            device_memory_properties,
        );

        let alloc_info = vk::MemoryAllocateInfo {
            allocation_size: mem_requirements.size,
            memory_type_index: found_memory_type_index,
            ..Default::default()
        };

        match logical_device.allocate_memory(&alloc_info, None) {
            Ok(buffer_memory) => {
                //self.vulkan_context.vertex_buffer_memory = buffer_memory;
                logical_device.bind_buffer_memory(buffer, buffer_memory, 0);

                println!("Allocated Vertex Buffer Memory");
                (buffer, buffer_memory)
            }
            Err(e) => panic!("{:?}", e),
        }
    }
}

fn find_memory_type(
    type_filter: u32,
    property_flags: vk::MemoryPropertyFlags,
    device_memory_properties: vk::PhysicalDeviceMemoryProperties,
) -> u32 {
    for i in 0..device_memory_properties.memory_type_count {
        //Check if it's the right memory type
        if (type_filter & (1 << i) != 0) {
            //Check if it has the right property
            if (device_memory_properties.memory_types[i as usize].property_flags & property_flags
                == property_flags)
            {
                return i;
            }
        }
    }
    panic!("Failed to find suitible memory type!");
}

fn debug_call_back(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: vk::DebugUtilsMessengerCallbackDataEXT,
) -> vk::Bool32 {
    println!("Validation layer: {:?}", p_callback_data.p_message);
    return vk::FALSE;
}

fn load_icon(file_path: &String) -> winit::window::Icon {
    let bytes = read_file(file_path);
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(&bytes).unwrap().into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    winit::window::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

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

//DEBUG HELPING FUNCTIONS
fn print_cstring_as_i8(c_string: &CString, size: i8) {
    unsafe {
        for i in 0..(size + 1) {
            println!("Cstring: {:?}", *(c_string.as_ptr().byte_add(i as usize)));
        }
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
    closing: bool,
    vertices: Vec<Vertex>,
    minimized: bool,
    window_resized: bool,
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

    //Graphics pipleline
    pipeline_layout: Option<vk::PipelineLayout>,
    render_pass: Option<vk::RenderPass>,
    graphics_pipelines: Vec<vk::Pipeline>,
    frame_buffers: Vec<vk::Framebuffer>,
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,

    //Command stuff
    command_pool: Option<vk::CommandPool>,
    command_buffers: Vec<vk::CommandBuffer>,

    //Syncronization
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: i32,
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
        let icon_path = String::from("F-example.jpg");
        let icon = load_icon(&icon_path);
        let mut window_attributes = Window::default_attributes()
            .with_title("Foundry Engine")
            .with_inner_size(self.size);
        window_attributes.window_icon = Some(icon);
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
        self.create_render_pass();
        self.create_graphics_pipeline();
        self.create_frame_buffers();
        self.create_command_pool();
        self.create_vertex_buffer();
        self.create_command_buffers();
        self.create_sync_object();
    }
    fn main_loop(&self) {}
    fn cleanup(&mut self) {
        let Some(instance) = &self.vulkan_context.instance else {
            println!("Instance does not exist");
            return;
        };
        let Some(logical_device) = &self.vulkan_context.logical_device else {
            panic!("No logical device when cleaning up");
        };
        let Some(entry) = &self.vulkan_context.entry else {
            panic!("No entry when cleaning up");
        };

        let swapchain_device = ash::khr::swapchain::Device::new(instance, logical_device);
        let surface_instance = ash::khr::surface::Instance::new(entry, instance);

        unsafe {
            //Syncronization
            /*
                        let Some(fence) = self.vulkan_context.in_flight_fence else {
                            panic!("No fence when cleaning up");
                        };
                        logical_device.destroy_fence(fence, None);
                        let Some(image_semaphore) = self.vulkan_context.image_available_semaphore else {
                            panic!("No sephamore when cleaning up");
                        };
                        logical_device.destroy_semaphore(image_semaphore, None);

                        let Some(render_semaphore) = self.vulkan_context.render_finished_semaphore else {
                            panic!("No sephamore when cleaning up");
                        };
                        logical_device.destroy_semaphore(render_semaphore, None);
            */

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                logical_device.destroy_semaphore(
                    self.vulkan_context.render_finished_semaphores[i as usize],
                    None,
                );
                logical_device.destroy_semaphore(
                    self.vulkan_context.image_available_semaphores[i as usize],
                    None,
                );
                logical_device
                    .destroy_fence(self.vulkan_context.in_flight_fences[i as usize], None);
            }

            //Command stuff
            let Some(command_pool) = self.vulkan_context.command_pool else {
                panic!("No command_pool when cleaning up");
            };
            logical_device.destroy_command_pool(command_pool, None);
            //Graphics pipleline
            for frame_buffer in self.vulkan_context.frame_buffers.iter() {
                logical_device.destroy_framebuffer(*frame_buffer, None);
            }

            logical_device.destroy_buffer(self.vulkan_context.vertex_buffer, None);
            logical_device.free_memory(self.vulkan_context.vertex_buffer_memory, None);

            for graphics_pipeline in self.vulkan_context.graphics_pipelines.iter() {
                logical_device.destroy_pipeline(*graphics_pipeline, None);
            }
            let Some(render_pass) = self.vulkan_context.render_pass else {
                panic!("No render_pass when cleaning up");
            };
            logical_device.destroy_render_pass(render_pass, None);

            //Pipeline Layout
            let Some(pipeline_layout) = self.vulkan_context.pipeline_layout else {
                panic!("No pipeline_layout when cleaning up");
            };
            logical_device.destroy_pipeline_layout(pipeline_layout, None);

            //Graphics pipleline handles
            for image_view in self.vulkan_context.swap_chain_image_views.iter() {
                logical_device.destroy_image_view(*image_view, None);
            }
            for shader_module in self.vulkan_context.shader_list.iter() {
                logical_device.destroy_shader_module(*shader_module, None);
            }

            //Swapchain
            let Some(swapchain) = self.vulkan_context.swap_chain else {
                panic!("No swapchain when cleaning up");
            };
            swapchain_device.destroy_swapchain(swapchain, None);

            //Surface
            let Some(surface) = self.vulkan_context.surface else {
                panic!("No surface when cleaning up");
            };
            surface_instance.destroy_surface(surface, None);

            //Device
            logical_device.destroy_device(None);

            //Instance
            let Some(instance) = &self.vulkan_context.instance else {
                panic!("No instance when cleaning up");
            };
            instance.destroy_instance(None);

            println!("Everything cleaned up");
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

    fn cleanup_swapchain(&mut self) {
        let Some(logical_device) = &self.vulkan_context.logical_device else {
            panic!("Cannot fetch logical_device during recreate_swapchain");
        };
        let Some(instance) = &self.vulkan_context.instance else {
            panic!("Cannot fetch instance during cleanup_swapchain");
        };
        let swapchain_device = ash::khr::swapchain::Device::new(instance, logical_device);
        unsafe {
            for image_view in self.vulkan_context.swap_chain_image_views.iter() {
                logical_device.destroy_image_view(*image_view, None);
            }
            for frame_buffer in self.vulkan_context.frame_buffers.iter() {
                logical_device.destroy_framebuffer(*frame_buffer, None);
            }
            let Some(swapchain) = self.vulkan_context.swap_chain else {
                panic!("No swapchain when cleaning up");
            };

            //Clear old lists to avoid dangling pointers
            self.vulkan_context.frame_buffers.clear();
            self.vulkan_context.swap_chain_image_views.clear();

            swapchain_device.destroy_swapchain(swapchain, None);
        }
    }

    fn recreate_swapchain(&mut self) {
        let Some(logical_device) = &self.vulkan_context.logical_device else {
            panic!("Cannot fetch logical_device durring recreate_swapchain");
        };
        unsafe {
            logical_device.device_wait_idle();

            self.cleanup_swapchain();

            self.create_swapchain();
            self.create_image_views();
            self.create_frame_buffers();
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
        //Load shader modules
        let vert_shader_path = String::from("./shaders/vert.spv");
        let frag_shader_path = String::from("./shaders/frag.spv");
        let vert_shader_code: Vec<u8> = read_file(&vert_shader_path);
        let frag_shader_code: Vec<u8> = read_file(&frag_shader_path);

        let vert_shader_module: vk::ShaderModule = self.create_shader_module(vert_shader_code);
        let frag_shader_module: vk::ShaderModule = self.create_shader_module(frag_shader_code);

        self.vulkan_context.shader_list.push(vert_shader_module);
        self.vulkan_context.shader_list.push(frag_shader_module);

        //Make main as a string to avoid a danging pointer
        let main_name: CString = CString::new("main").expect("failed to load c string");

        let vert_shader_create_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::VERTEX,
            module: vert_shader_module,
            p_name: main_name.as_ptr(),
            ..Default::default()
        };
        let frag_shader_create_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: frag_shader_module,
            p_name: main_name.as_ptr(),
            ..Default::default()
        };

        let shader_stages: [vk::PipelineShaderStageCreateInfo; 2] =
            [vert_shader_create_info, frag_shader_create_info];

        //--------FIXED FUNCTIONS--------

        //Vertex Input
        let dynamic_states: Vec<vk::DynamicState> =
            vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

        let dynamic_state = vk::PipelineDynamicStateCreateInfo {
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: dynamic_states.as_ptr(),
            ..Default::default()
        };

        let binding_descriptions_list = Vertex::get_binding_descs();
        let attributes_list = Vertex::get_attribute_descs();
        //let binding_descriptions_list: Vec<vk::VertexInputBindingDescription> = Vec::new();
        //let attributes_list: Vec<vk::VertexInputAttributeDescription> = Vec::new();
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
            vertex_binding_description_count: 1,
            p_vertex_binding_descriptions: binding_descriptions_list.as_ptr(),
            vertex_attribute_description_count: attributes_list.len() as u32,
            p_vertex_attribute_descriptions: attributes_list.as_ptr(),
            ..Default::default()
        };
        vec![vertex_input_info];

        //Input assembly
        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
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
        //let viewport_array: Vec<vk::Viewport> = vec![viewport];

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extent,
        };
        //let scissor_array: Vec<vk::Rect2D> = vec![scissor];
        /*
                let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
                    dynamic_state_count: dynamic_states.len() as u32,
                    p_dynamic_states: dynamic_states.as_ptr(),
                    ..Default::default()
                };
        */

        let viewport_state = vk::PipelineViewportStateCreateInfo {
            viewport_count: 1,
            //p_viewports: &viewport,
            scissor_count: 1,
            //p_scissors: &scissor,
            ..Default::default()
        };

        //Rasterizer
        //Note to self: chaning PolygonMode to be wireframe or points would be good for debugging
        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo {
            depth_clamp_enable: vk::FALSE,
            rasterizer_discard_enable: vk::FALSE,
            polygon_mode: vk::PolygonMode::FILL,
            line_width: 1.0,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::CLOCKWISE,
            depth_bias_enable: vk::FALSE,
            depth_bias_constant_factor: 0.0,
            depth_bias_slope_factor: 0.0,
            depth_bias_clamp: 0.0,
            ..Default::default()
        };

        //Multisampling
        //(Anti-aliasing)
        let sample_mask_list: Vec<vk::SampleMask> = Vec::new();
        let multisampling_info = vk::PipelineMultisampleStateCreateInfo {
            sample_shading_enable: vk::FALSE,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            min_sample_shading: 1.0,
            //p_sample_mask: sample_mask_list.as_ptr(),
            alpha_to_coverage_enable: 0,
            alpha_to_one_enable: vk::FALSE,
            ..Default::default()
        };
        let multisampling_info_list: Vec<vk::PipelineMultisampleStateCreateInfo> =
            vec![multisampling_info];

        //Color blending
        let colour_blend_attatchment = vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B,
            blend_enable: vk::FALSE,
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            ..Default::default()
        };

        let colour_blend_info = vk::PipelineColorBlendStateCreateInfo {
            logic_op_enable: vk::FALSE,
            logic_op: vk::LogicOp::COPY,
            attachment_count: 1,
            p_attachments: &colour_blend_attatchment,
            blend_constants: [0.0, 0.0, 0.0, 0.0],
            ..Default::default()
        };

        //Pipeline Layout
        let set_layouts: Vec<vk::DescriptorSetLayout> = Vec::new();
        let push_constants: Vec<vk::PushConstantRange> = Vec::new();
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            set_layout_count: 0,
            p_set_layouts: set_layouts.as_ptr(),
            push_constant_range_count: 0,
            p_push_constant_ranges: push_constants.as_ptr(),
            ..Default::default()
        };

        let Some(logical_device) = &self.vulkan_context.logical_device else {
            panic!("No logical_device when calling create_graphics_pipeline");
        };
        unsafe {
            match logical_device.create_pipeline_layout(&pipeline_layout_create_info, None) {
                Ok(pipeline_layout) => self.vulkan_context.pipeline_layout = Some(pipeline_layout),
                Err(e) => panic!("Failed to create pipeline_layout"),
            }
        }

        //Render pass
        let Some(pipeline_layout) = self.vulkan_context.pipeline_layout else {
            panic!("No pipeline_layout when calling create_graphics_pipeline");
        };
        let Some(render_pass) = self.vulkan_context.render_pass else {
            panic!("No render_pass when calling create_graphics_pipeline");
        };
        let pipeline_create_info = vk::GraphicsPipelineCreateInfo {
            stage_count: 2,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_input_info,
            p_input_assembly_state: &input_assembly_info,
            p_viewport_state: &viewport_state,
            p_rasterization_state: &rasterizer_info,
            p_multisample_state: &multisampling_info,
            p_color_blend_state: &colour_blend_info,
            p_dynamic_state: &dynamic_state,
            layout: pipeline_layout,
            render_pass: render_pass,
            subpass: 0,
            base_pipeline_index: -1,
            base_pipeline_handle: vk::Pipeline::null(),
            //Base pipeline layout isnt implemented because this pipeline
            //does not inherit from any other pipeline
            //base_pipeline_index is which index to use as a handle for other
            //pipeleine creation
            ..Default::default()
        };

        unsafe {
            match (logical_device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[pipeline_create_info],
                None,
            )) {
                Ok(pipeline) => {
                    println!("Created graphics pipeline");
                    self.vulkan_context.graphics_pipelines = pipeline;
                }
                Err(e) => panic!("{:?}", e),
            }
        }
    }

    fn create_render_pass(&mut self) {
        let Some(swapchain_format) = self.vulkan_context.swap_chain_format else {
            panic!("No swapchain_format when calling create_render_pass");
        };
        let color_attatchment = vk::AttachmentDescription {
            format: swapchain_format.format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        };
        //let colour_attatchment_list: Vec<vk::AttachmentDescription> = vec![color_attatchment];

        let colour_attatchment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };
        let colour_attatchment_ref_list: Vec<vk::AttachmentReference> =
            vec![colour_attatchment_ref];

        let subpass_dependancy = vk::SubpassDependency {
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::BY_REGION,
            ..Default::default()
        };
        let subpass = vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            p_color_attachments: colour_attatchment_ref_list.as_ptr(),
            ..Default::default()
        };
        let subpass_list: Vec<vk::SubpassDescription> = vec![subpass];

        let render_pass_info = vk::RenderPassCreateInfo {
            attachment_count: 1,
            p_attachments: &color_attatchment,
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: 1,
            p_dependencies: &subpass_dependancy,
            ..Default::default()
        };

        let Some(logical_device) = &self.vulkan_context.logical_device else {
            panic!("No logical when calling create_render_pass");
        };
        unsafe {
            match logical_device.create_render_pass(&render_pass_info, None) {
                Ok(render_pass) => self.vulkan_context.render_pass = Some(render_pass),
                Err(e) => panic!("NOOO"),
            }
        }
    }

    fn create_frame_buffers(&mut self) {
        let Some(render_pass) = self.vulkan_context.render_pass else {
            panic!("No render_pass when calling create_frame_buffers");
        };
        let Some(swapchain_extent) = self.vulkan_context.swap_chain_extent_used else {
            panic!("No swap_chain_extent_used when calling create_frame_buffers");
        };
        let Some(logical_device) = &self.vulkan_context.logical_device else {
            panic!("No logical_device when calling create_frame_buffers");
        };
        let image_views = &self.vulkan_context.swap_chain_image_views;
        let swapchain_frame_frame_buffers: Vec<vk::Framebuffer> = Vec::new();
        let frame_buffer_count = image_views.len();
        for i in 0..frame_buffer_count {
            let attatchments: [vk::ImageView; 1] = [image_views[i]];
            let frame_buffer_info = vk::FramebufferCreateInfo {
                render_pass: render_pass,
                attachment_count: 1,
                p_attachments: attatchments.as_ptr(),
                width: swapchain_extent.width,
                height: swapchain_extent.height,
                layers: 1,
                ..Default::default()
            };
            unsafe {
                match logical_device.create_framebuffer(&frame_buffer_info, None) {
                    Ok(frame_buffer) => self.vulkan_context.frame_buffers.push(frame_buffer),
                    Err(e) => panic!("{}", e),
                };
            }
        }
    }
    fn create_command_pool(&mut self) {
        let Some(logical_device) = &self.vulkan_context.logical_device else {
            panic!("No logical_device when calling create_command_buffers");
        };
        let queue_families = &self.vulkan_context.family_indicies;
        let pool_info = vk::CommandPoolCreateInfo {
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: queue_families.graphics_family,
            ..Default::default()
        };
        unsafe {
            match logical_device.create_command_pool(&pool_info, None) {
                Ok(command_pool) => {
                    self.vulkan_context.command_pool = Some(command_pool);
                }
                Err(e) => panic!("{}", e),
            };
        }
    }

    fn create_vertex_buffer(&mut self) {
        let Some(logical_device) = &self.vulkan_context.logical_device else {
            panic!("No logical device when calling create_vertex_buffer");
        };
        let vertices = &self.vertices;
        let vertex_buffer_size = size_of::<Vertex>() * vertices.len();
        let (vertex_buffer, vertex_buffer_memory) = create_buffer(
            vertex_buffer_size as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &self.vulkan_context,
        );
        //MAP MEMORY
        unsafe {
            let map_memory_result = logical_device.map_memory(
                vertex_buffer_memory,
                0,
                vertex_buffer_size as u64,
                vk::MemoryMapFlags::empty(),
            );

            let Ok(memory_pointer) = map_memory_result else {
                panic!("Failed to map memory for vertex buffer");
            };

            ptr::copy_nonoverlapping(
                self.vertices.as_ptr(),
                memory_pointer as *mut Vertex, //Cast void to Vertex Data Type
                self.vertices.len(),
            );
            logical_device.unmap_memory(vertex_buffer_memory);
        }
        self.vulkan_context.vertex_buffer_memory = vertex_buffer_memory;
        self.vulkan_context.vertex_buffer = vertex_buffer;
    }

    fn create_command_buffers(&mut self) {
        let Some(pool) = self.vulkan_context.command_pool else {
            panic!("No command_pool when calling create_command_buffer");
        };
        let Some(logical_device) = &self.vulkan_context.logical_device else {
            panic!("No logical_device when calling create_command_buffer");
        };
        let alloc_info = vk::CommandBufferAllocateInfo {
            command_pool: pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: MAX_FRAMES_IN_FLIGHT,
            ..Default::default()
        };
        unsafe {
            match logical_device.allocate_command_buffers(&alloc_info) {
                Ok(command_buffer_vec) => {
                    self.vulkan_context.command_buffers = command_buffer_vec;
                }
                Err(e) => panic!("{:?}", e),
            }
        }
    }

    fn record_command_buffer(
        logical_device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        render_pass: vk::RenderPass,
        swapchain_extent: vk::Extent2D,
        frame_buffers: &Vec<vk::Framebuffer>,
        image_index: usize,
        pipeline: vk::Pipeline,
        vertex_buffer: vk::Buffer,
    ) {
        let begin_info = vk::CommandBufferBeginInfo {
            ..Default::default()
        };
        unsafe {
            match logical_device.begin_command_buffer(command_buffer, &begin_info) {
                Ok(some) => {
                    ();
                }
                Err(e) => panic!("{:?}", e),
            }
        };
        let clear_color = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };
        let render_pass_info = vk::RenderPassBeginInfo {
            render_pass: render_pass,
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain_extent,
            },
            framebuffer: frame_buffers[image_index],
            clear_value_count: 1,
            p_clear_values: &clear_color,
            ..Default::default()
        };
        unsafe {
            logical_device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
            logical_device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline,
            );
        };

        //Coppied from create_graphics_pipeline if broken check if it matches
        let viewport = vk::Viewport {
            x: 0 as f32,
            y: 0 as f32,
            width: swapchain_extent.width as f32,
            height: swapchain_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
            ..Default::default()
        };

        let viewports = [viewport];
        unsafe {
            logical_device.cmd_set_viewport(command_buffer, 0, &viewports);
        }
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extent,
        };
        let scissors = [scissor];

        unsafe {
            logical_device.cmd_set_scissor(command_buffer, 0, &scissors);

            //Added from Vertex Buffer
            logical_device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline,
            );

            let vertex_buffers = [vertex_buffer];
            let offsets: [vk::DeviceSize; 1] = [0];
            logical_device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
            //

            logical_device.cmd_draw(command_buffer, 3, 1, 0, 0);
            match logical_device.end_command_buffer(command_buffer) {
                Ok(something) => (),
                Err(e) => panic!("{}", e),
            }
        }
    }
    fn create_sync_object(&mut self) {
        let Some(logical_device) = &self.vulkan_context.logical_device else {
            panic!("No logical_device when calling create_sync_object");
        };
        let semaphore_info = vk::SemaphoreCreateInfo {
            ..Default::default()
        };
        let fence_info = vk::FenceCreateInfo {
            flags: vk::FenceCreateFlags::SIGNALED, //It will be active on creation
            ..Default::default()
        };
        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                match logical_device.create_semaphore(&semaphore_info, None) {
                    Ok(semaphore) => {
                        //self.vulkan_context.image_available_semaphores[i as usize] = semaphore
                        self.vulkan_context
                            .image_available_semaphores
                            .push(semaphore);
                    }
                    Err(e) => panic!("{}", e),
                }
                match logical_device.create_semaphore(&semaphore_info, None) {
                    Ok(semaphore) => {
                        //self.vulkan_context.render_finished_semaphores[i] = Some(semaphore)
                        self.vulkan_context
                            .render_finished_semaphores
                            .push(semaphore);
                    }
                    Err(e) => panic!("{}", e),
                }
                match logical_device.create_fence(&fence_info, None) {
                    Ok(fence) => {
                        //self.vulkan_context.in_flight_fences[i] = Some(fence)
                        self.vulkan_context.in_flight_fences.push(fence);
                    }

                    Err(e) => panic!("{}", e),
                }
            }
        }
    }

    fn draw_frame(&mut self) {
        if (self.vulkan_context.frame_buffers.is_empty()) {
            return;
        }
        let Some(instance) = &self.vulkan_context.instance else {
            panic!("No instance when calling draw_frame... somehow..?");
        };
        let Some(logical_device) = &self.vulkan_context.logical_device else {
            panic!("No logical_device when calling create_sync_object");
        };
        let Some(swapchain) = self.vulkan_context.swap_chain else {
            panic!("No swap_chain when calling draw_frame");
        };
        let Some(render_pass) = self.vulkan_context.render_pass else {
            panic!();
        };
        let Some(swapchain_extent) = self.vulkan_context.swap_chain_extent_used else {
            panic!();
        };
        let Some(graphics_queue) = self.vulkan_context.graphics_queue else {
            panic!();
        };
        let swapchain_device = ash::khr::swapchain::Device::new(instance, &logical_device);

        //Get all command buffers and sync objects for current frame
        let current_frame = self.vulkan_context.current_frame as usize;
        let fences = &[self.vulkan_context.in_flight_fences[current_frame]];
        let current_image_available_semaphore =
            self.vulkan_context.image_available_semaphores[current_frame];
        let current_render_finished_semaphore =
            self.vulkan_context.render_finished_semaphores[current_frame];
        let current_fence = self.vulkan_context.in_flight_fences[current_frame];
        let current_command_buffer = self.vulkan_context.command_buffers[current_frame];
        unsafe {
            //Params: list of fences to wait for / should wait for all? / timeout
            logical_device.wait_for_fences(fences, true, u64::MAX);

            let mut image_index: usize = 0;

            //In case device driver doesn't handle ERROR_OUT_OF_DATE_KHR
            if (self.window_resized) {
                self.recreate_swapchain();
                self.window_resized = false;
                return;
            }

            match swapchain_device.acquire_next_image(
                swapchain,
                u64::MAX,
                current_image_available_semaphore,
                vk::Fence::null(),
            ) {
                Ok((index, bool)) => {
                    image_index = index as usize;
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    //panic!("recreate swapchain");

                    println!("\n\n\n recreating \n\n\n");
                    //panic!("LKSJD");
                    self.recreate_swapchain();
                    return;
                }
                Err(e) => {
                    panic!("{}", e);
                }
            }
            logical_device.reset_fences(fences);

            logical_device
                .reset_command_buffer(current_command_buffer, vk::CommandBufferResetFlags::empty()); //MIGHT
            //ERROR
            HelloTriangleApp::record_command_buffer(
                logical_device,
                current_command_buffer,
                render_pass,
                swapchain_extent,
                &self.vulkan_context.frame_buffers,
                image_index,
                self.vulkan_context.graphics_pipelines[0],
                self.vulkan_context.vertex_buffer,
            );
            let wait_semaphores = [current_image_available_semaphore];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let signal_semaphores = [current_render_finished_semaphore];
            let submit_info = vk::SubmitInfo {
                wait_semaphore_count: 1,
                p_wait_semaphores: wait_semaphores.as_ptr(), //CHECK
                //was wait_semaphores
                p_wait_dst_stage_mask: wait_stages.as_ptr(),
                command_buffer_count: 1,
                p_command_buffers: &current_command_buffer,
                signal_semaphore_count: 1,
                p_signal_semaphores: signal_semaphores.as_ptr(),
                ..Default::default()
            };
            let submit_infos = [submit_info];
            match logical_device.queue_submit(graphics_queue, &submit_infos, current_fence) {
                Ok(something) => (),
                Err(e) => panic!("{}", e),
            }

            let image_index_32 = image_index as u32;
            let present_info = vk::PresentInfoKHR {
                swapchain_count: 1,
                p_swapchains: &swapchain,
                p_image_indices: &image_index_32,
                ..Default::default()
            };

            match swapchain_device.queue_present(graphics_queue, &present_info) {
                Ok(is_suboptimal) => {
                    if (is_suboptimal) {
                        self.recreate_swapchain();
                        return;
                    }
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.recreate_swapchain();
                    return;
                }
                Err(e) => {
                    panic!("{:?}", e);
                }
            }
            self.vulkan_context.current_frame =
                (self.vulkan_context.current_frame + 1) % MAX_FRAMES_IN_FLIGHT as i32;
        }
    }
}

#[derive(Default)]
#[repr(C)]
struct Vertex {
    pos: glm::Vec2,
    colour: glm::Vec3,
}

impl Vertex {
    fn get_binding_descs() -> Vec<vk::VertexInputBindingDescription> {
        let vertex_size: u32 = std::mem::size_of::<Vertex>() as u32;
        let binding_desc = vk::VertexInputBindingDescription {
            binding: 0,
            stride: vertex_size,
            input_rate: vk::VertexInputRate::VERTEX,
            ..Default::default()
        };
        vec![binding_desc]
    }

    fn get_attribute_descs() -> Vec<vk::VertexInputAttributeDescription> {
        let position_attribute_desc = vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: offset_of!(Vertex, pos) as u32,
            ..Default::default()
        };
        let colour_attribute_desc = vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: offset_of!(Vertex, colour) as u32,
            ..Default::default()
        };
        let attributes = vec![position_attribute_desc, colour_attribute_desc];
        attributes
    }
}
fn main() {
    let vertecies: Vec<Vertex> = vec![
        Vertex {
            pos: glm::vec2(0.0, -0.5),
            colour: glm::vec3(1.0, 1.0, 1.0),
        },
        Vertex {
            pos: glm::vec2(0.5, 0.5),
            colour: glm::vec3(0.0, 1.0, 0.0),
        },
        Vertex {
            pos: glm::vec2(-0.5, 0.5),
            colour: glm::vec3(0.0, 0.0, 1.0),
        },
    ];

    //Vulkan Setup
    let mut app: HelloTriangleApp = HelloTriangleApp {
        vertices: vertecies,
        ..Default::default()
    };
    app.run(800.0, 600.0);
}
