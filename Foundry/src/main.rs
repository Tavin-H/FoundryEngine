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

//Basic IDE config
#![allow(unused)]

//-----------Foundry Engine Modules------------
//Game Data
mod game_data;
use crate::game_data::GameContext;
use crate::game_data::GameObject;
use crate::game_data::MeshAllocation;
use crate::game_data::Transform;

//Vulkan Data
mod vulkan_data;
use crate::vulkan_data::{UniformBufferObject, VulkanContext};

//UI Handler
mod ui_data;
use crate::ui_data::UIHandler;
use egui;
use egui_winit;

//------------------Vulkan----------------------
//Constants
const MAX_GAME_OBJECTS_IN_SCENE: u64 = 1000;
const VALIDATION_LAYERS: &[&str] = &["VK_LAYER_KHRONOS_validation"];
const WANTED_EXTENSION_NAMES: &[&CStr] = &[vk::KHR_SWAPCHAIN_NAME];
const FIRST_PRIORITY: f32 = 1.0;
const MAX_FRAMES_IN_FLIGHT: u32 = 2;

const MAIN_NAME: *const i8 = [109 as i8, 97 as i8, 105 as i8, 110 as i8, 0 as i8].as_ptr();
use ash::amd::texture_gather_bias_lod;
use ash::vk::native::StdVideoAV1Level_STD_VIDEO_AV1_LEVEL_2_2;
//Window
use ash_window;
use image;
use nalgebra_glm::Mat4x4;
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
    CommandBufferUsageFlags, Handle, PFN_vkEnumeratePhysicalDevices,
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
use std::fs::{self, File};
use std::mem::swap;
use std::panic;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::Arc;
use std::thread::current;
use std::time;

//Image
use stb_image;
use stb_image::image::LoadResult;
use std::path;

//Model
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::BufReader;

//Setup winit boilerplate
#[derive(Default)]
struct WinitApp {
    window: Option<Window>,
    size: winit::dpi::LogicalSize<f64>,
    instance: Option<ash::Instance>,
}

impl ApplicationHandler for HelloTriangleApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = &self.window else {
            panic!("");
        };
        self.ui_handler.init(window);
        let Some(context) = &mut self.ui_handler.context else {
            panic!();
        };
        self.vulkan_context.init_vulkan(window, context);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(window) = &self.window else {
            panic!("");
        };
        let Some(platform) = &mut self.ui_handler.platform else {
            panic!();
        };
        let Some(context) = &mut self.ui_handler.context else {
            panic!();
        };
        let event_wrapper: winit::event::Event<()> = winit::event::Event::WindowEvent {
            window_id: id,
            event: event.clone(),
        };
        platform.handle_event(context.io_mut(), window, &event_wrapper);
        //let ui_response = state.on_window_event(&window, &event);
        //Optimization?
        /*
                if (ui_response.consumed) {
                    match event {
                        WindowEvent::KeyboardInput {
                            device_id,
                            event,
                            is_synthetic,
                        } => return,
                        WindowEvent::CursorMoved {
                            device_id,
                            position,
                        } => return,
                        WindowEvent::MouseWheel {
                            device_id,
                            delta,
                            phase,
                        } => return,
                        WindowEvent::MouseInput {
                            device_id,
                            state,
                            button,
                        } => return,
                        _ => (),
                    }
                    return;
        if (ui_response.consumed) {
            return;
        } else {
        */
        match event {
            WindowEvent::CloseRequested => {
                self.closing = true;
                event_loop.exit();
                self.vulkan_context.wait_idle();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap();
            }
            WindowEvent::Resized(size) => {
                if (size.width == 0 && size.height == 0) {
                    self.minimized = true;
                    return;
                }
                //self.window_resized = true;
                self.vulkan_context.window_resized = true;
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                //println!("{:?} {:?}", event.physical_key, event.state);
                match event.state {
                    winit::event::ElementState::Pressed => {}
                    _ => {}
                }
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                println!("winit mouse input");
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // update logic here
        if (!self.closing) {
            let mut avg_delta_time = self.game_context.calculate_delta_time();
            let fps: f32 = 1.0 / avg_delta_time;
            if (!self.vulkan_context.running) {
                avg_delta_time = 0.0;
            }
            if (self.game_context.game_objects[0].transform.position[2] > 1.0) {
                self.rising = false;
            }
            if (self.game_context.game_objects[0].transform.position[2] < -1.0) {
                self.rising = true;
            }
            if (!self.rising) {
                self.game_context.game_objects[0].transform.position[2] -= 1.0 * avg_delta_time;
            } else {
                self.game_context.game_objects[0].transform.position[2] += 1.0 * avg_delta_time;
            }
            //////
            if (self.game_context.game_objects[1].transform.position[2] > 1.0) {
                self.rising2 = false;
            }
            if (self.game_context.game_objects[1].transform.position[2] < -1.0) {
                self.rising2 = true;
            }
            if (!self.rising2) {
                self.game_context.game_objects[1].transform.position[2] -= 3.0 * avg_delta_time;
            } else {
                self.game_context.game_objects[1].transform.position[2] += 3.0 * avg_delta_time;
            }

            //println!("{} {}", delta_time, self.running);

            //LAST THINGS
            let Some(window) = &self.window else {
                panic!("");
            };
            self.ui_handler.record_ui_data(window, fps);
            let Some(ui_context) = &mut self.ui_handler.context else {
                panic!();
            };

            self.vulkan_context
                .draw_frame(&self.game_context.game_objects, ui_context, window);

            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }
}

//----------------Helper functions-----------------

fn convert_vec_to_mat(position: [f32; 3]) -> Mat4x4 {
    if (position.len() != 3) {
        panic!("Position does not have 3 elements");
    }

    let mut transform = glm::Mat4::identity();
    transform[(0, 3)] = position[0];
    transform[(1, 3)] = position[1];
    transform[(2, 3)] = position[2];

    transform
}

fn find_supported_format(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
    candidates: Vec<vk::Format>,
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> vk::Format {
    for format in candidates.iter() {
        unsafe {
            let properties =
                instance.get_physical_device_format_properties(*physical_device, *format);
            if (tiling == vk::ImageTiling::LINEAR
                && (properties.linear_tiling_features & features) == features)
            {
                return *format;
            } else if (tiling == vk::ImageTiling::OPTIMAL
                && (properties.optimal_tiling_features & features) == features)
            {
                return *format;
            }
        }
    }
    //If all else fails
    panic!("No supported format found");
}

fn find_depth_format(instance: &Instance, physical_device: &vk::PhysicalDevice) -> vk::Format {
    let format = find_supported_format(
        instance,
        physical_device,
        vec![
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ],
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    );
    println!("using: {:?} for depth buffering", format);
    return format;
}

fn has_stencil_component(format: vk::Format) -> bool {
    return format == vk::Format::D32_SFLOAT_S8_UINT || format == vk::Format::D24_UNORM_S8_UINT;
}

fn copy_buffer_to_image(
    logical_device: &ash::Device,
    instance: &Instance,
    buffer: vk::Buffer,
    image: vk::Image,
    width: u32,
    height: u32,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
) {
    let command_buffer = begin_single_time_commands(logical_device, command_pool);

    let region = vk::BufferImageCopy {
        buffer_offset: 0,
        buffer_row_length: 0,
        buffer_image_height: 0,
        image_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        },
        image_offset: vk::Offset3D {
            ..Default::default()
        },
        image_extent: vk::Extent3D {
            width: width,
            height: height,
            depth: 1,
        },
        ..Default::default()
    };
    unsafe {
        logical_device.cmd_copy_buffer_to_image(
            command_buffer,
            buffer,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        );
    }
    end_single_time_commands(logical_device, command_buffer, command_pool, graphics_queue);
}

fn create_image(
    logical_device: &ash::Device,
    instance: &ash::Instance,
    physical_device: &vk::PhysicalDevice,
    height: u32,
    width: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
    image_memory: &vk::DeviceMemory,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
) -> (vk::Image, vk::DeviceMemory) {
    let mut image: Option<vk::Image> = None;
    let mut image_memory: Option<vk::DeviceMemory> = None;

    let image_create_info = vk::ImageCreateInfo {
        image_type: vk::ImageType::TYPE_2D,
        extent: vk::Extent2D {
            width: width,
            height: height,
        }
        .into(),
        mip_levels: 1,
        array_layers: 1,
        format: format,
        tiling: tiling,
        initial_layout: vk::ImageLayout::UNDEFINED,
        usage: usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        samples: vk::SampleCountFlags::TYPE_1,

        ..Default::default()
    };
    unsafe {
        match logical_device.create_image(&image_create_info, None) {
            Ok(created_image) => {
                image = Some(created_image);
                println!("Created Texture Successfully");
            }
            Err(e) => panic!("{}", e),
        }
    }

    //Load image into memory
    unsafe {
        let device_memory_properties =
            instance.get_physical_device_memory_properties(*physical_device);

        let Some(image_value) = image else {
            panic!("Image is not a value");
        };
        let memory_requirements = logical_device.get_image_memory_requirements(image_value);
        let alloc_info = vk::MemoryAllocateInfo {
            allocation_size: memory_requirements.size,
            memory_type_index: find_memory_type(
                memory_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                device_memory_properties,
            ),
            ..Default::default()
        };

        match logical_device.allocate_memory(&alloc_info, None) {
            Ok(memory_pointer) => {
                logical_device.bind_image_memory(image_value, memory_pointer, 0);
                image_memory = Some(memory_pointer);
            }
            Err(e) => panic!("{}", e),
        }
    }
    let Some(image_value) = image else {
        panic!("");
    };
    let Some(image_memory_value) = image_memory else {
        panic!("");
    };

    return (image_value, image_memory_value);
}

fn create_image_view(
    image: vk::Image,
    format: vk::Format,
    aspect_flags: vk::ImageAspectFlags,
    logical_device: &ash::Device,
) -> vk::ImageView {
    let view_info = vk::ImageViewCreateInfo {
        image: image,
        view_type: vk::ImageViewType::TYPE_2D,
        format: format,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: aspect_flags,
            base_mip_level: 0,
            base_array_layer: 0,
            level_count: 1,
            layer_count: 1,
            ..Default::default()
        },
        components: vk::ComponentMapping {
            r: vk::ComponentSwizzle::IDENTITY,
            g: vk::ComponentSwizzle::IDENTITY,
            b: vk::ComponentSwizzle::IDENTITY,
            a: vk::ComponentSwizzle::IDENTITY,
        },
        ..Default::default()
    };
    unsafe {
        match logical_device.create_image_view(&view_info, None) {
            Ok(image_view) => {
                return image_view;
            }
            Err(e) => panic!("{}", e),
        }
    }
}

fn begin_single_time_commands(
    logical_device: &ash::Device,
    command_pool: vk::CommandPool,
) -> vk::CommandBuffer {
    let alloc_info = vk::CommandBufferAllocateInfo {
        level: vk::CommandBufferLevel::PRIMARY,
        command_pool: command_pool,
        command_buffer_count: 1,
        ..Default::default()
    };
    let begin_info = vk::CommandBufferBeginInfo {
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        ..Default::default()
    };
    unsafe {
        match logical_device.allocate_command_buffers(&alloc_info) {
            Ok(command_buffers) => {
                println!("allocated command buffer thingy");
                logical_device.begin_command_buffer(command_buffers[0], &begin_info);
                return command_buffers[0]; //FIXME potential dangling pointer if indexing returns a
                //reference and not the value
            }
            Err(e) => panic!("{}", e),
        }
    }
}

fn end_single_time_commands(
    logical_device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
) {
    unsafe {
        logical_device.end_command_buffer(command_buffer);
    }

    let submit_info = vk::SubmitInfo {
        command_buffer_count: 1,
        p_command_buffers: &command_buffer,
        ..Default::default()
    };

    unsafe {
        logical_device.queue_submit(graphics_queue, &[submit_info], vk::Fence::null());
        logical_device.device_wait_idle();

        logical_device.free_command_buffers(command_pool, &[command_buffer]);
    }
}
//Takes creation info and returns a buffer as well as the device memory where the buffer is
//located
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
    game_context: GameContext,
    ui_handler: UIHandler,
    closing: bool,
    running: bool,

    rising: bool,
    rising2: bool,

    minimized: bool,
    window_resized: bool,
    start_time: Option<std::time::Instant>,
    frame_count: u64,
}
//Holds all vulkan objects in a single struct to controll lifetimes more precisely
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
        self.vulkan_context.cleanup();
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
    fn init_window(&mut self, event_loop: EventLoop<()>, window_width: f64, window_height: f64) {
        event_loop.set_control_flow(ControlFlow::Poll);
        self.size = winit::dpi::LogicalSize::new(window_width, window_height);
        event_loop.run_app(self);
    }

    fn instantiate(&mut self, mut gameobject: GameObject) {
        let before_indices = self.vulkan_context.indices.len();
        gameobject._mesh.first_vertex = self.vulkan_context.vertices.len() as i32;
        //println!("RAHHHHHHHHHHHHHH {:?}", gameobject._mesh.first_index);
        self.vulkan_context.load_model();
        let after_indices = self.vulkan_context.indices.len();

        gameobject._mesh.first_index = before_indices as u32;
        gameobject._mesh.index_count = (after_indices - before_indices) as u32;
        self.game_context.game_objects.push(gameobject);
    }
}
fn main() {
    let start_time = std::time::Instant::now();
    let mut test_position = glm::Mat4::identity();
    test_position[(0, 3)] = 1.0;

    let mut gameobject_example = GameObject {
        name: String::from("Example"),
        id: 0,
        _mesh: MeshAllocation {
            index_count: 0, //Hardcoded - change when loading the object
            first_index: 0,
            first_vertex: 0,
        },
        transform: Transform {
            position: [0.0, 0.0, -2.0],
            scale: [1.0, 1.0, 1.0],
        },
        ..Default::default()
    };

    let mut gameobject_example_2 = GameObject {
        name: String::from("Example"),
        id: 1,
        transform: Transform {
            position: [1.0, 0.0, 0.0],
            scale: [1.0, 1.0, 3.0],
        },
        ..Default::default()
    };

    let mut app: HelloTriangleApp = HelloTriangleApp {
        start_time: Some(start_time),
        rising: false,
        rising2: true,
        running: false,
        ..Default::default()
    };
    app.instantiate(gameobject_example_2);
    app.instantiate(gameobject_example);
    app.run(800.0, 800.0);
}

//TODO to expand
//Put all buffers in one buffer with offsets for cache friendly design
//
