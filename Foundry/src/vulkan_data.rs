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

const MAX_GAME_OBJECTS_IN_SCENE: u64 = 1000;
const VALIDATION_LAYERS: &[&str] = &["VK_LAYER_KHRONOS_validation"];
const WANTED_EXTENSION_NAMES: &[&CStr] = &[vk::KHR_SWAPCHAIN_NAME];
const FIRST_PRIORITY: f32 = 1.0;
const MAX_FRAMES_IN_FLIGHT: u32 = 2;

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

    //textures
    texture_image: vk::Image,
    texture_image_view: vk::ImageView,
    texture_image_memory: vk::DeviceMemory,
    texture_sampler: vk::Sampler,

    //Depth Buffering
    depth_image: vk::Image,
    depth_image_memory: vk::DeviceMemory,
    depth_image_view: vk::ImageView,

    //Shader Info
    shader_list: Vec<vk::ShaderModule>,

    //Graphics pipleline
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_set_layouts: Vec<vk::DescriptorSetLayout>, //Used for many images
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
    pipeline_layout: Option<vk::PipelineLayout>,
    render_pass: Option<vk::RenderPass>,
    graphics_pipelines: Vec<vk::Pipeline>,

    //Buffers
    frame_buffers: Vec<vk::Framebuffer>,
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<vk::DeviceMemory>,
    uniform_buffers_mapped: Vec<*mut UniformBufferObject>,

    transform_buffers: Vec<vk::Buffer>,
    transform_buffers_memory: Vec<vk::DeviceMemory>,
    transform_buffers_mapped: Vec<*mut Mat4x4>,

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

impl VulkanContext {
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
        self.create_descriptor_set_layout();
        self.create_graphics_pipeline();
        self.create_command_pool();
        self.create_depth_resources();
        self.create_frame_buffers();
        self.create_texture_image();
        self.create_texture_image_view();
        self.create_texture_sampler();
        //self.load_model();
        self.create_vertex_buffer();
        self.create_index_buffer();
        self.create_transform_storage_buffers();
        self.create_uniform_buffer();
        self.create_descriptor_pool();
        self.create_descriptor_sets();
        self.create_command_buffers();
        self.create_sync_object();

        //self.running = true;
    }
    fn cleanup(&mut self) {
        let Some(instance) = &self.instance else {
            println!("Instance does not exist");
            return;
        };
        let Some(logical_device) = &self.logical_device else {
            panic!("No logical device when cleaning up");
        };
        let Some(entry) = &self.entry else {
            panic!("No entry when cleaning up");
        };

        let swapchain_device = ash::khr::swapchain::Device::new(instance, logical_device);
        let surface_instance = ash::khr::surface::Instance::new(entry, instance);

        unsafe {
            logical_device.destroy_image_view(self.depth_image_view, None);
            logical_device.destroy_image(self.depth_image, None);
            logical_device.free_memory(self.depth_image_memory, None);

            logical_device.destroy_sampler(self.texture_sampler, None);
            logical_device.destroy_image_view(self.texture_image_view, None);
            logical_device.destroy_image(self.texture_image, None);
            logical_device.free_memory(self.texture_image_memory, None);

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                logical_device.destroy_semaphore(self.render_finished_semaphores[i as usize], None);
                logical_device.destroy_semaphore(self.image_available_semaphores[i as usize], None);
                logical_device.destroy_fence(self.in_flight_fences[i as usize], None);
            }

            //Command stuff
            let Some(command_pool) = self.command_pool else {
                panic!("No command_pool when cleaning up");
            };
            logical_device.destroy_command_pool(command_pool, None);
            //Graphics pipleline
            for frame_buffer in self.frame_buffers.iter() {
                logical_device.destroy_framebuffer(*frame_buffer, None);
            }

            //Buffers
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                logical_device.destroy_buffer(self.uniform_buffers[i as usize], None);
                logical_device.free_memory(self.uniform_buffers_memory[i as usize], None);

                logical_device.destroy_buffer(self.transform_buffers[i as usize], None);
                logical_device.free_memory(self.transform_buffers_memory[i as usize], None);
            }
            logical_device.destroy_buffer(self.vertex_buffer, None);
            logical_device.free_memory(self.vertex_buffer_memory, None);

            logical_device.destroy_buffer(self.index_buffer, None);
            logical_device.free_memory(self.index_buffer_memory, None);

            for graphics_pipeline in self.graphics_pipelines.iter() {
                logical_device.destroy_pipeline(*graphics_pipeline, None);
            }
            let Some(render_pass) = self.render_pass else {
                panic!("No render_pass when cleaning up");
            };
            logical_device.destroy_render_pass(render_pass, None);

            //Pipeline Layout
            let Some(pipeline_layout) = self.pipeline_layout else {
                panic!("No pipeline_layout when cleaning up");
            };
            logical_device.destroy_pipeline_layout(pipeline_layout, None);

            logical_device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            logical_device.destroy_descriptor_pool(self.descriptor_pool, None);

            //Graphics pipleline handles
            for image_view in self.swap_chain_image_views.iter() {
                logical_device.destroy_image_view(*image_view, None);
            }
            for shader_module in self.shader_list.iter() {
                logical_device.destroy_shader_module(*shader_module, None);
            }

            //Swapchain
            let Some(swapchain) = self.swap_chain else {
                panic!("No swapchain when cleaning up");
            };
            swapchain_device.destroy_swapchain(swapchain, None);

            //Surface
            let Some(surface) = self.surface else {
                panic!("No surface when cleaning up");
            };
            surface_instance.destroy_surface(surface, None);

            //Device
            logical_device.destroy_device(None);

            //Instance
            let Some(instance) = &self.instance else {
                panic!("No instance when cleaning up");
            };
            instance.destroy_instance(None);

            println!("Everything cleaned up");
        }
    }
}
