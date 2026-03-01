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
        //println!("init ui");
        //self.ui_handler.init(window);
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
            self.frame_count += 1;
            if self.frame_count > 1000 {
                self.frame_count = 0;
                self.fps = 1.0 / avg_delta_time;
            }
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
            self.ui_handler.record_ui_data(window, self.fps);
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
    fps: f32,
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
        println!("Loaded window");
        let icon_path = String::from("F-example.jpg");
        let icon = load_icon(&icon_path);
        let mut window_attributes = Window::default_attributes()
            .with_title("Foundry Engine")
            .with_inner_size(self.size);
        window_attributes.window_icon = Some(icon);
        let event_loop = EventLoop::new().unwrap();
        //self.window = Some(event_loop.create_window(window_attributes).unwrap());
        match event_loop.create_window(window_attributes) {
            Ok(window) => {
                self.ui_handler.init(&window);
                self.window = Some(window);
            }
            Err(e) => panic!("{}", e),
        }
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
