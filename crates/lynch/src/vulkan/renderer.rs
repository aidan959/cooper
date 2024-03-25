use crate::{
    gltf_loader::Model, render_graph::RenderGraph, render_tools::{self, present}, renderer::Renderer,
    vulkan::{cont::*, debug::*}, window::window::Window, Camera, Texture,
};
use ash::{
    extensions::{
        ext::DebugUtils,
        khr::{Surface, Swapchain},
    },
    vk::Extent2D,
};
use ash::{vk, Entry, Instance};
use gpu_allocator::vulkan::AllocatorCreateDesc;
use imgui::{DrawData, FontConfig, FontGlyphRanges, FontSource};
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use super::{descriptor::DescriptorSet, render_pass, Buffer, Device, Image, ImageDesc};
use glam::{self, Mat4, Vec3, Vec4};
use std::{
    ffi::CString,
    sync::{Arc, Mutex},
    time::Instant,
};
pub const MAX_NUM_GPU_MATERIALS: usize = 1024;
pub const MAX_NUM_GPU_MESHES: usize = 1024;
pub const DESCRIPTOR_SET_INDEX_BINDLESS: u32 = 0;
pub const DESCRIPTOR_SET_INDEX_VIEW: u32 = 1;
pub const DESCRIPTOR_SET_INDEX_INPUT_TEXTURES: u32 = 2;
pub const DEFAULT_TEXTURE: u32 = u32::MAX;

use crate::vulkan::surface::create_surface;
pub struct ModelInstance {
    pub model: Model,
    pub transform: glam::Mat4,
}
pub struct Frame {
    pub command_buffer: vk::CommandBuffer,
    pub command_buffer_reuse_fence: vk::Fence,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
}
pub struct VulkanRenderer {
    pub vk_context: VkContext,
    pub sync_frames: Vec<Frame>,
    pub command_pool: vk::CommandPool,
    pub image_count: u32,
    pub present_images: Vec<Image>,
    pub depth_image: Image,
    pub surface_format: vk::SurfaceFormatKHR,
    pub surface_resolution: vk::Extent2D,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub debug_utils_messenger: Option<vk::DebugUtilsMessengerEXT>,
    pub internal_renderer: RendererInternal,
    pub current_frame: usize,
    pub num_frames_in_flight: u32,
    pub swapchain_recreate_needed: bool,
    pub camera_uniform_buffer: Vec<Buffer>,
    pub view_data: ViewUniformData,
    pub last_frame_end: Instant,
    pub gui: imgui::Context,
    pub gui_renderer: imgui_rs_vulkan_renderer::Renderer,
    pub platform: imgui_winit_support::WinitPlatform,
}
pub struct RendererInternal {
    pub bindless_descriptor_set_layout: vk::DescriptorSetLayout,
    pub bindless_descriptor_set: vk::DescriptorSet,
    pub instances: Vec<ModelInstance>,
    pub gpu_materials_buffer: Buffer,
    pub gpu_meshes_buffer: Buffer,
    gpu_materials: Vec<GpuMaterial>,
    gpu_meshes: Vec<GpuMesh>,
    default_diffuse_map_index: u32,
    default_normal_map_index: u32,
    default_occlusion_map_index: u32,
    default_metallic_roughness_map_index: u32,
    next_bindless_image_index: u32,
    next_bindless_vertex_buffer_index: u32,
    next_bindless_index_buffer_index: u32,
    pub need_environment_map_update: bool,
}

#[derive(Clone, Debug, Copy)]
#[repr(C)]
pub struct ViewUniformData {
    pub view: glam::Mat4,
    pub projection: glam::Mat4,
    pub inverse_view: glam::Mat4,
    pub inverse_projection: glam::Mat4,
    pub eye_pos: glam::Vec3,
    pub sun_dir: glam::Vec3,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub shadows_enabled: u32,
    pub ssao_enabled: u32,
    pub fxaa_enabled: u32,
    pub cubemap_enabled: u32,
    pub ibl_enabled: u32,
}
impl ViewUniformData {
    pub fn new(camera: &Camera, surface_resolution: Extent2D) -> Self {
        Self {
            view: camera.get_view(),
            projection: camera.get_projection(),
            inverse_view: camera.get_view().inverse(),
            inverse_projection: camera.get_projection().inverse(),
            eye_pos: camera.get_position(),
            viewport_width: surface_resolution.width,
            viewport_height: surface_resolution.height,
            sun_dir: Vec3::new(0.0, 0.9, 0.15).normalize(),
            shadows_enabled: 1,
            ssao_enabled: 1,
            fxaa_enabled: 1,
            cubemap_enabled: 1,
            ibl_enabled: 1,
        }
    }
    pub fn create_camera_buffer(&self, vk_context: &VkContext) -> Buffer {
        Buffer::new(
            vk_context.arc_device(),
            Some(std::slice::from_ref(self)),
            std::mem::size_of_val(self) as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            gpu_allocator::MemoryLocation::CpuToGpu,
            Some(String::from("camera_uniform_buffer")),
        )
    }
}
#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct GpuMaterial {
    diffuse_map: u32,
    normal_map: u32,
    metallic_roughness_map: u32,
    occlusion_map: u32,
    base_color_factor: Vec4,
    metallic_factor: f32,
    roughness_factor: f32,
    padding: [f32; 2],
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct GpuMesh {
    vertex_buffer: u32,
    index_buffer: u32,
    material: u32,
}
impl VulkanRenderer {
    pub fn load_model(&self, path: &str) -> Model {
        crate::gltf_loader::load_gltf(self.vk_context.arc_device(), path)
    }
    fn setup_swapchain_images(
        vk_context: &VkContext,
        swapchain: vk::SwapchainKHR,
        swapchain_loader: &Swapchain,
        surface_format: vk::SurfaceFormatKHR,
        surface_resolution: vk::Extent2D,
    ) -> (Vec<Image>, Image) {
        unsafe {
            let present_images = swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Error getting swapchain images");

            let present_images: Vec<Image> = present_images
                .iter()
                .map(|&image| {
                    Image::new_from_handle(
                        vk_context.arc_device(),
                        image,
                        ImageDesc::new_2d(
                            surface_resolution.width,
                            surface_resolution.height,
                            surface_format.format,
                        ),
                    )
                })
                .collect();

            let depth_image = Image::new_from_desc(
                vk_context.arc_device(),
                ImageDesc::new_2d(
                    surface_resolution.width,
                    surface_resolution.height,
                    vk::Format::D32_SFLOAT_S8_UINT,
                )
                .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                .aspect(vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL),
            );

            vk_context.arc_device().execute_and_submit(|cb| {
                for present_image in &present_images {
                    super::image_pipeline_barrier(
                        &vk_context.arc_device(),
                        cb,
                        present_image,
                        vk_sync::AccessType::Nothing,
                        vk_sync::AccessType::Present,
                        true,
                    );
                }

                super::image_pipeline_barrier(
                    &vk_context.arc_device(),
                    cb,
                    &depth_image,
                    vk_sync::AccessType::Nothing,
                    vk_sync::AccessType::DepthStencilAttachmentWrite,
                    true,
                );
            });

            (present_images, depth_image)
        }
    }
    pub fn present_frame(&self, present_index: usize, current_frame: usize) {
        unsafe {
            let wait_semaphores = [self.sync_frames[current_frame].render_finished_semaphore];
            let swapchains = [self.swapchain];
            let image_indices = [present_index as u32];

            self.vk_context.device().execute_and_submit(move |cb| {
                super::image_pipeline_barrier(
                    self.device(),
                    cb,
                    &Image::new_from_handle(
                        self.vk_context.arc_device(),
                        self.swapchain_loader
                            .get_swapchain_images(self.swapchain)
                            .expect("Error getting swapchain images")[present_index],
                        ImageDesc::new_2d(
                            self.surface_resolution.width,
                            self.surface_resolution.height,
                            self.surface_format.format,
                        ),
                    ),
                    vk_sync::AccessType::Nothing,
                    vk_sync::AccessType::Present,
                    false,
                );
            });
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&wait_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            self.swapchain_loader
                .queue_present(self.device().queue, &present_info)
                .unwrap();
        }
    }

    pub fn submit_commands(&self, frame_index: usize) {
        unsafe {
            let command_buffers = [self.sync_frames[frame_index].command_buffer];
            let wait_semaphores = [self.sync_frames[frame_index].image_available_semaphore];
            let signal_semaphores = [self.sync_frames[frame_index].render_finished_semaphore];

            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .signal_semaphores(&signal_semaphores)
                .command_buffers(&command_buffers)
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT]);

            self.device()
                .ash_device
                .queue_submit(
                    self.device().queue,
                    &[submit_info.build()],
                    self.sync_frames[frame_index].command_buffer_reuse_fence,
                )
                .expect("Queue submit failed.");
        }
    }
    fn create_command_pool(vk_context: &VkContext) -> vk::CommandPool {
        let command_pool = unsafe {
            vk_context
                .ash_device()
                .create_command_pool(
                    &vk::CommandPoolCreateInfo::builder()
                        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                        .queue_family_index(vk_context.device().queue_family_index),
                    None,
                )
                .expect("Failed to create command pool")
        };

        command_pool
    }
    fn create_synchronization_frames(
        vk_context: &VkContext,
        command_pool: vk::CommandPool,
        image_count: u32,
    ) -> Vec<Frame> {
        (0..image_count)
            .map(|_| unsafe {
                Frame {
                    command_buffer: vk_context
                        .ash_device()
                        .allocate_command_buffers(
                            &vk::CommandBufferAllocateInfo::builder()
                                .command_buffer_count(1)
                                .command_pool(command_pool)
                                .level(vk::CommandBufferLevel::PRIMARY),
                        )
                        .expect("Failed to allocate command buffer")[0],
                    command_buffer_reuse_fence: vk_context
                        .ash_device()
                        .create_fence(
                            &vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED),
                            None,
                        )
                        .expect("Failed to create fence"),
                    render_finished_semaphore: vk_context
                        .ash_device()
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                        .expect("Failed to create semaphore"),
                    image_available_semaphore: vk_context
                        .ash_device()
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                        .expect("Failed to create semaphore"),
                }
            })
            .collect()
    }
    fn create_instance(entry: &Entry) -> Instance {
        let app_name = CString::new("Cooper").unwrap();
        let engine_name = CString::new("Lynch").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(0)
            .engine_name(&engine_name)
            .engine_version(0)
            .api_version(ash::vk::make_api_version(0, 1, 3, 0));

        let mut extension_names = crate::vulkan::surface::required_extension_names();

        if ENABLE_VALIDATION_LAYERS {
            extension_names.push(DebugUtils::name().as_ptr());
        }

        let (_layer_names, layer_pointers) = get_lay_names_pointers();
        let mut instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);

        if ENABLE_VALIDATION_LAYERS {
            check_validation_layer_support(&entry);
            instance_create_info = instance_create_info.enabled_layer_names(&layer_pointers);
        }

        unsafe { entry.create_instance(&instance_create_info, None).unwrap() }
    }

    fn create_swapchain(
        context: &VkContext,
    ) -> (
        vk::SwapchainKHR,
        Swapchain,
        vk::SurfaceFormatKHR,
        vk::Extent2D,
        u32,
        vk::RenderPass,
    ) {
        let surface_loader = context.surface();
        let physical_device = context.physical_device();
        let surface = context.surface_khr();
        let instance = context.instance();
        unsafe {
            let surface_format = surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .expect("Error getting device surface formats")[0];

            let surface_capabilities = surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .expect("Error getting device surface capabilities");

            let desired_image_count = surface_capabilities.min_image_count + 1;
            let surface_resolution = surface_capabilities.current_extent;
            let desired_transform = vk::SurfaceTransformFlagsKHR::IDENTITY;

            let present_modes = surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .expect("Present modes could not be retrieved.");

            let present_mode = match present_modes
                .iter()
                .cloned()
                .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            {
                Some(a) => a,
                None => vk::PresentModeKHR::FIFO_RELAXED,
            };
            let ash_device = context.ash_device();
            let swapchain_loader = Swapchain::new(instance, ash_device);

            let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(surface)
                .min_image_count(desired_image_count)
                .image_color_space(surface_format.color_space)
                .image_format(surface_format.format)
                .image_extent(surface_resolution)
                .image_usage(
                    vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
                )
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(desired_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .image_array_layers(1);

            let swapchain = swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap();
            let attachment_descs = [vk::AttachmentDescription::builder()
                .format(surface_format.format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .build()];

            let color_attachment_refs = [vk::AttachmentReference::builder()
                .attachment(0)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .build()];

            let subpass_descs = [vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&color_attachment_refs)
                .build()];

            let subpass_deps = [vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                )
                .build()];

            let render_pass_info = vk::RenderPassCreateInfo::builder()
                .attachments(&attachment_descs)
                .subpasses(&subpass_descs)
                .dependencies(&subpass_deps);
            let render_pass = unsafe { context.ash_device().create_render_pass(&render_pass_info, None).unwrap()};


            (
                swapchain,
                swapchain_loader,
                surface_format,
                surface_resolution,
                desired_image_count,
                render_pass
            )
        }
    }

    fn create_debug_utils(
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> (Option<DebugUtils>, Option<vk::DebugUtilsMessengerEXT>) {
        if !ENABLE_VALIDATION_LAYERS {
            return (None, None);
        }
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING,
            )
            .pfn_user_callback(Some(super::debug::vulkan_debug_callback));

        let debug_utils_loader = DebugUtils::new(entry, instance);

        let debug_utils_messenger = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap()
        };

        (Some(debug_utils_loader), Some(debug_utils_messenger))
    }
    pub fn arc_device(self: &Self) -> Arc<Device> {
        self.vk_context.arc_device()
    }
    fn ash_device(self: &Self) -> &ash::Device {
        self.vk_context.ash_device()
    }

    pub fn render(&mut self, graph: &mut RenderGraph, camera: &Camera) -> f32 {
        self.update_view_to_camera(&camera);
        let command_buffer = self.sync_frames[self.current_frame].command_buffer;
        let wait_fence = self.sync_frames[self.current_frame].command_buffer_reuse_fence;
        let present_index = self.begin_frame();
        unsafe {
            {
                self.ash_device()
                    .wait_for_fences(&[wait_fence], true, std::u64::MAX)
                    .expect("Wait for fence failed.");

                self.ash_device()
                    .reset_fences(&[wait_fence])
                    .expect("Reset fences failed.");
            }

            self.ash_device()
                .reset_command_buffer(
                    command_buffer,
                    ash::vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .expect("Reset command buffer failed.");

            let command_buffer_begin_info = ash::vk::CommandBufferBeginInfo::builder()
                .flags(ash::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            self.ash_device()
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Begin command buffer failed.");

            self.camera_uniform_buffer[self.current_frame]
                .update_memory(std::slice::from_ref(&self.view_data));

            graph.new_frame(self.current_frame);
            graph.clear();

            self.internal_renderer.instances[0]
                .transform
                .add_mat4(&Mat4::from_rotation_x(0.01));

            render_tools::build_render_graph(
                graph,
                self.arc_device(),
                &self,
                &self.view_data,
                &camera,
            );

            // render_tools::build_render_graph_gbuffer_only(
            //     graph,
            //     self.arc_device(),
            //     &self,
            // );
            // render_tools::build_render_graph_atmosphere(
            // graph,
            // self.arc_device(),
            // &self,
            // &camera
            // );
            // render_tools::build_render_graph_opt(
            //     graph,
            //     self.arc_device(),
            //     &self,
            //     &self.view_data,
            //     &camera,
            //     true,
            //     false,
            //     false,
            //     false,
            //     true,
            //     true,
            //     true
            // );
            let framebuffer = self.framebuffers[present_index as usize];
            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(self.render_pass)
                
                .framebuffer(framebuffer)
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.surface_resolution,
                })
                .clear_values(&[vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [1.0, 1.0, 1.0, 1.0],
                    },
                }]);

            self.vk_context.ash_device().cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            )
            ;
            graph.prepare(&self);
            let image = self.present_images[present_index].clone();
            graph.render(&command_buffer, &self, &image, present_index);
            self.present_images[self.current_frame].current_layout =
            vk::ImageLayout::PRESENT_SRC_KHR;

            let ui = self.gui.frame();
            let mut a = true;
            ui.show_demo_window(&mut a);
            let draw_data = self.gui.render();
            
            self.gui_renderer
                .cmd_draw(command_buffer, draw_data)
                .unwrap();
            self.vk_context.ash_device().cmd_end_render_pass(command_buffer) ;
            self.ash_device()
                .end_command_buffer(command_buffer)
                .expect("End commandbuffer failed.");

            
            self.submit_commands(self.current_frame);
            self.present_frame(present_index, self.current_frame);
            self.current_frame = (self.current_frame + 1) % self.num_frames_in_flight as usize;
            self.internal_renderer.need_environment_map_update = false;
            graph.current_frame = self.current_frame;
        }
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame_end).as_secs_f32();
        self.last_frame_end = now;
        delta_time
    }
}

impl Renderer for VulkanRenderer {
    fn update_view_to_camera(self: &mut Self, camera: &Camera) {
        self.view_data.view = camera.get_view();
        self.view_data.projection = camera.get_projection();
        self.view_data.inverse_view = camera.get_view().inverse();
        self.view_data.inverse_projection = camera.get_projection().inverse();
        self.view_data.eye_pos = camera.get_position();
    }
    fn initialize(self: &mut Self) {
        self.internal_renderer
            .initialize(self.vk_context.arc_device());
    }
    fn device(self: &Self) -> &Device {
        self.vk_context.device()
    }

    fn add_model(self: &mut Self, model: crate::gltf_loader::Model, transform: glam::Mat4) {
        let device = self.vk_context.device();
        self.internal_renderer.add_model(device, model, transform);
    }
    fn create(window: &Window, camera: &Camera) -> Self {
        let entry = ash::Entry::linked();
        let instance = Self::create_instance(&entry);
        let (debug_utils, debug_utils_messenger) = Self::create_debug_utils(&entry, &instance);
        let surface = Surface::new(&entry, &instance);
        let surface_khr = unsafe { create_surface(&entry, &instance, &window.window) }
            .expect("creating surface failed");

        let device = Device::new(&instance, surface_khr, &surface, debug_utils);

        let vk_context = VkContext::new(entry, instance, surface, surface_khr, device);

        let (swapchain, swapchain_loader, surface_format, surface_resolution, image_count,render_pass) =
            Self::create_swapchain(&vk_context);

        let (present_images, depth_image) = Self::setup_swapchain_images(
            &vk_context,
            swapchain,
            &swapchain_loader,
            surface_format,
            surface_resolution,
        );
        let framebuffers :Vec<vk::Framebuffer> = present_images.iter()
            .map(|view| [view.image_view])
            .map(|attachments| {
                let framebuffer_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(surface_resolution.width)
                    .height(surface_resolution.height)
                    .layers(1);
                unsafe { vk_context.ash_device().create_framebuffer(&framebuffer_info, None) }
            })
        .collect::<Result<Vec<_>, _>>().unwrap();
        let command_pool = Self::create_command_pool(&vk_context);

        let sync_frames =
            Self::create_synchronization_frames(&vk_context, command_pool, image_count);

        let internal_renderer = RendererInternal::new(&vk_context);
        let view_data = ViewUniformData::new(&camera, surface_resolution);
        let camera_uniform_buffer = (0..image_count)
            .map(|_| view_data.create_camera_buffer(&vk_context))
            .collect::<Vec<_>>();

        let (mut gui, platform) = {
            let mut g = imgui::Context::create();
            let mut platform = WinitPlatform::init(&mut g);

            let hidpi_factor = platform.hidpi_factor();
            let font_size = (13.0 * hidpi_factor) as f32;
            g.fonts().add_font(&[
                FontSource::DefaultFontData {
                    config: Some(FontConfig {
                        size_pixels: font_size,
                        ..FontConfig::default()
                    }),
                },
                FontSource::TtfData {
                    data: include_bytes!("../../../../assets/fonts/mplus-1p-regular.ttf"),
                    size_pixels: font_size,
                    config: Some(FontConfig {
                        rasterizer_multiply: 1.75,
                        glyph_ranges: FontGlyphRanges::default(),
                        ..FontConfig::default()
                    }),
                },
            ]);

            g.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
            platform.attach_window(g.io_mut(), &window.window, HiDpiMode::Rounded);
            (g, platform)
        };
        let gui_renderer = {
            let allocator = gpu_allocator::vulkan::Allocator::new(&AllocatorCreateDesc {
                instance: vk_context.instance().clone(),
                device: vk_context.ash_device().clone(),
                physical_device: vk_context.physical_device(),
                debug_settings: Default::default(),
                buffer_device_address: false,
                allocation_sizes: Default::default(),
            })
            .unwrap();

            imgui_rs_vulkan_renderer::Renderer::with_gpu_allocator(
                Arc::new(Mutex::new(allocator)),
                vk_context.ash_device().clone(),
                vk_context.device().queue,
                vk_context.device().cmd_pool,
                imgui_rs_vulkan_renderer::DynamicRendering {
                    color_attachment_format: vk::Format::R32G32B32A32_SFLOAT,
                    depth_attachment_format: None,
                },
                &mut gui,
                Some(imgui_rs_vulkan_renderer::Options {
                    in_flight_frames: image_count as usize,
                    ..Default::default()
                }),
            )
            .unwrap()
        };
        Self {
            vk_context,
            sync_frames,
            command_pool,
            image_count,
            present_images,
            depth_image,
            surface_format,
            surface_resolution,
            swapchain,
            swapchain_loader,
            render_pass,
            debug_utils_messenger,
            internal_renderer,
            current_frame: 0,
            num_frames_in_flight: image_count,
            swapchain_recreate_needed: false,
            camera_uniform_buffer,
            view_data,
            last_frame_end: Instant::now(),
            gui_renderer,
            gui,
            platform,
            framebuffers
        }
    }

    fn begin_frame(self: &mut Self) -> usize {
        unsafe {
            let (present_index, _) = self
                .swapchain_loader
                .acquire_next_image(
                    self.swapchain,
                    std::u64::MAX,
                    self.sync_frames[self.current_frame].image_available_semaphore,
                    vk::Fence::null(),
                )
                .expect("Error acquiring next swapchain image");
            present_index as usize
        }
    }

    fn end_frame(self: &mut Self) {
        todo!()
    }

    fn draw_meshes(self: &Self) {
        todo!()
    }

    fn draw_frame(self: &mut Self, _frame_count: u32) {
        todo!()
    }

    fn wait_gpu_idle(self: &Self) {
        unsafe { self.device().ash_device.device_wait_idle().unwrap() }
    }

    fn resize(self: &mut Self, resize: winit::dpi::PhysicalSize<u32>) {
        self.surface_resolution = Extent2D {
            width: resize.width,
            height: resize.height,
        };
        self.swapchain_recreate_needed = true;
    }
}
impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        let command_buffers: Vec<vk::CommandBuffer> = self
            .sync_frames
            .iter()
            .map(|f| f.command_buffer)
            .clone()
            .collect();
        let hardware_device = &self.device().ash_device;
        unsafe { hardware_device.free_command_buffers(self.command_pool, &command_buffers) };
        unsafe { hardware_device.destroy_command_pool(self.command_pool, None) };
        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None)
        };
        self.depth_image.clean_vk_resources();
        self.present_images
            .iter()
            .for_each(|pi| pi.clean_vk_resources());
    }
}
impl RendererInternal {
    pub fn new(vk_context: &VkContext) -> Self {
        let bindless_descriptor_set_layout =
            create_bindless_descriptor_set_layout(vk_context.device());
        let bindless_descriptor_set =
            create_bindless_descriptor_set(vk_context.device(), bindless_descriptor_set_layout);

        // TODO this should A: Not be so greedy B: Dynamically increase allocation size TO a limit - IE read how much memory we have when this is full, and allocate up to the size

        let gpu_materials_buffer = Buffer::new::<u8>(
            vk_context.arc_device(),
            None,
            (MAX_NUM_GPU_MATERIALS * std::mem::size_of::<GpuMaterial>()) as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            gpu_allocator::MemoryLocation::CpuToGpu,
            Some(String::from("material_buffer")),
        );

        let gpu_meshes_buffer = Buffer::new::<u8>(
            vk_context.arc_device(),
            None,
            (MAX_NUM_GPU_MESHES * std::mem::size_of::<GpuMesh>()) as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            gpu_allocator::MemoryLocation::CpuToGpu,
            Some(String::from("gpu_mesh_buffer")),
        );
        DescriptorSet::write_raw_storage_buffer(
            vk_context.device(),
            bindless_descriptor_set,
            3,
            &gpu_materials_buffer,
        );
        DescriptorSet::write_raw_storage_buffer(
            vk_context.device(),
            bindless_descriptor_set,
            4,
            &gpu_meshes_buffer,
        );

        RendererInternal {
            bindless_descriptor_set_layout,
            bindless_descriptor_set,
            instances: vec![],
            gpu_materials: vec![],
            gpu_meshes: vec![],
            gpu_meshes_buffer,
            gpu_materials_buffer,
            next_bindless_image_index: 0,
            next_bindless_vertex_buffer_index: 0,
            next_bindless_index_buffer_index: 0,
            default_diffuse_map_index: 0,
            default_normal_map_index: 0,
            default_occlusion_map_index: 0,
            default_metallic_roughness_map_index: 0,
            need_environment_map_update: true,
        }
    }

    pub fn initialize(&mut self, device: Arc<Device>) {
        let default_diffuse_map =
            Texture::load(device.clone(), "assets/textures/def/white_texture.png");
        let default_normal_map =
            Texture::load(device.clone(), "assets/textures/def/flat_normal_map.png");
        let default_occlusion_map =
            Texture::load(device.clone(), "assets/textures/def/white_texture.png");
        let default_metallic_roughness_map =
            Texture::load(device.clone(), "assets/textures/def/metallic_roughness.png");

        self.default_diffuse_map_index = self.add_bindless_texture(&device, &default_diffuse_map);
        self.default_normal_map_index = self.add_bindless_texture(&device, &default_normal_map);
        self.default_occlusion_map_index =
            self.add_bindless_texture(&device, &default_occlusion_map);
        self.default_metallic_roughness_map_index =
            self.add_bindless_texture(&device, &default_metallic_roughness_map);
    }

    pub fn add_model(&mut self, device: &Device, mut model: Model, transform: glam::Mat4) {
        for mesh in &mut model.meshes {
            let diffuse_bindless_index = match mesh.material.diffuse_map {
                DEFAULT_TEXTURE => self.default_diffuse_map_index,
                _ => self.add_bindless_texture(
                    device,
                    &model.textures[mesh.material.diffuse_map as usize],
                ),
            };

            let normal_bindless_index = match mesh.material.normal_map {
                DEFAULT_TEXTURE => self.default_normal_map_index,
                _ => self.add_bindless_texture(
                    device,
                    &model.textures[mesh.material.normal_map as usize],
                ),
            };

            let metallic_roughness_bindless_index = match mesh.material.metallic_roughness_map {
                DEFAULT_TEXTURE => self.default_metallic_roughness_map_index,
                _ => self.add_bindless_texture(
                    device,
                    &model.textures[mesh.material.metallic_roughness_map as usize],
                ),
            };

            let occlusion_bindless_index = match mesh.material.occlusion_map {
                DEFAULT_TEXTURE => self.default_occlusion_map_index,
                _ => self.add_bindless_texture(
                    device,
                    &model.textures[mesh.material.occlusion_map as usize],
                ),
            };

            let vertex_buffer_bindless_idx =
                self.add_bindless_vertex_buffer(device, &mesh.primitive.vertex_buffer);
            let index_buffer_bindless_idx =
                self.add_bindless_index_buffer(device, &mesh.primitive.index_buffer);
            let material_index = self.add_material(GpuMaterial {
                diffuse_map: diffuse_bindless_index,
                normal_map: normal_bindless_index,
                metallic_roughness_map: metallic_roughness_bindless_index,
                occlusion_map: occlusion_bindless_index,
                base_color_factor: mesh.material.base_color_factor,
                metallic_factor: mesh.material.metallic_factor,
                roughness_factor: mesh.material.roughness_factor,
                padding: [0.0; 2],
            });

            let mesh_index = self.add_mesh(GpuMesh {
                vertex_buffer: vertex_buffer_bindless_idx,
                index_buffer: index_buffer_bindless_idx,
                material: material_index,
            });

            mesh.gpu_mesh = mesh_index;
        }

        self.gpu_meshes_buffer
            .update_memory(self.gpu_meshes.as_slice());
        self.gpu_materials_buffer
            .update_memory(self.gpu_materials.as_slice());

        self.instances.push(ModelInstance { model, transform });
    }

    fn add_bindless_texture(&mut self, device: &Device, texture: &Texture) -> u32 {
        let new_image_index = self.next_bindless_image_index;

        let descriptor_write = vk::WriteDescriptorSet::builder()
            .dst_set(self.bindless_descriptor_set)
            .dst_binding(0)
            .dst_array_element(new_image_index)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&texture.descriptor_info))
            .build();

        unsafe {
            device
                .device()
                .update_descriptor_sets(std::slice::from_ref(&descriptor_write), &[])
        };

        self.next_bindless_image_index += 1;

        new_image_index
    }

    fn add_bindless_vertex_buffer(&mut self, device: &Device, buffer: &Buffer) -> u32 {
        let new_buffer_index = self.next_bindless_vertex_buffer_index;

        let buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(buffer.buffer)
            .range(buffer.size)
            .build();

        let descriptor_write = vk::WriteDescriptorSet::builder()
            .dst_set(self.bindless_descriptor_set)
            .dst_binding(1)
            .dst_array_element(new_buffer_index)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&buffer_info))
            .build();

        unsafe {
            device
                .device()
                .update_descriptor_sets(std::slice::from_ref(&descriptor_write), &[])
        };

        self.next_bindless_vertex_buffer_index += 1;

        new_buffer_index
    }

    fn add_bindless_index_buffer(&mut self, device: &Device, buffer: &Buffer) -> u32 {
        let new_buffer_index = self.next_bindless_index_buffer_index;

        let buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(buffer.buffer)
            .range(buffer.size)
            .build();

        let descriptor_write = vk::WriteDescriptorSet::builder()
            .dst_set(self.bindless_descriptor_set)
            .dst_binding(2)
            .dst_array_element(new_buffer_index)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&buffer_info))
            .build();

        unsafe {
            device
                .device()
                .update_descriptor_sets(std::slice::from_ref(&descriptor_write), &[])
        };

        self.next_bindless_index_buffer_index += 1;

        new_buffer_index
    }

    fn add_material(&mut self, gpu_material: GpuMaterial) -> u32 {
        let material_index = self.gpu_materials.len() as u32;
        self.gpu_materials.push(gpu_material);

        material_index
    }

    fn add_mesh(&mut self, gpu_mesh: GpuMesh) -> u32 {
        let gpu_index = self.gpu_meshes.len() as u32;
        self.gpu_meshes.push(gpu_mesh);
        println!("{}", self.gpu_meshes.len());
        gpu_index
    }
    fn get_instance(&mut self, _instance_index: u32) {
        //self.instances.get(instance_index).unwrap()
    }
    pub fn draw_meshes(
        &self,
        device: &Device,
        command_buffer: vk::CommandBuffer,
        pipeline_layout: vk::PipelineLayout,
    ) {
        unsafe {
            for instance in &self.instances {
                for (i, mesh) in instance.model.meshes.iter().enumerate() {
                    device.cmd_push_constants(
                        command_buffer,
                        pipeline_layout,
                        (
                            instance.transform * instance.model.transforms[i],
                            glam::Vec4::new(1.0, 0.5, 0.2, 1.0),
                            mesh.gpu_mesh,
                            [0; 3],
                        ),
                    );

                    device.device().cmd_bind_vertex_buffers(
                        command_buffer,
                        0,
                        &[mesh.primitive.vertex_buffer.buffer],
                        &[0],
                    );
                    device.device().cmd_bind_index_buffer(
                        command_buffer,
                        mesh.primitive.index_buffer.buffer,
                        0,
                        vk::IndexType::UINT32,
                    );
                    device.device().cmd_draw_indexed(
                        command_buffer,
                        mesh.primitive.indices.len() as u32,
                        1,
                        0,
                        0,
                        1,
                    );
                }
            }
        }
    }
}
impl Drop for RendererInternal {
    fn drop(&mut self) {
        self.gpu_materials_buffer.clean_vk_resources();
    }
}

pub const MAX_BINDLESS_DESCRIPTOR_COUNT: usize = 512 * 510;

pub fn create_bindless_descriptor_set_layout(device: &Device) -> vk::DescriptorSetLayout {
    let descriptor_set_layout_binding = vec![
        // Texture buffer binding
        vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(MAX_BINDLESS_DESCRIPTOR_COUNT as u32)
            .stage_flags(vk::ShaderStageFlags::ALL)
            .build(),
        // Vertex buffer binding
        vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(MAX_BINDLESS_DESCRIPTOR_COUNT as u32)
            .stage_flags(vk::ShaderStageFlags::ALL)
            .build(),
        // Index buffer binding
        vk::DescriptorSetLayoutBinding::builder()
            .binding(2)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(MAX_BINDLESS_DESCRIPTOR_COUNT as u32)
            .stage_flags(vk::ShaderStageFlags::ALL)
            .build(),
        // Materials buffer binding
        vk::DescriptorSetLayoutBinding::builder()
            .binding(3)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(MAX_BINDLESS_DESCRIPTOR_COUNT as u32) // Hack: actually 1
            .stage_flags(vk::ShaderStageFlags::ALL)
            .build(),
        // Meshes buffer binding
        vk::DescriptorSetLayoutBinding::builder()
            .binding(4)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(MAX_BINDLESS_DESCRIPTOR_COUNT as u32) // Hack: actually 1
            .stage_flags(vk::ShaderStageFlags::ALL)
            .build(),
    ];

    let binding_flags: [vk::DescriptorBindingFlags; 5] = [
        vk::DescriptorBindingFlags::PARTIALLY_BOUND,
        vk::DescriptorBindingFlags::PARTIALLY_BOUND,
        vk::DescriptorBindingFlags::PARTIALLY_BOUND,
        vk::DescriptorBindingFlags::PARTIALLY_BOUND,
        vk::DescriptorBindingFlags::PARTIALLY_BOUND
            | vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT,
    ];

    let mut binding_flags_create_info =
        vk::DescriptorSetLayoutBindingFlagsCreateInfo::builder().binding_flags(&binding_flags);

    let descriptor_sets_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(&descriptor_set_layout_binding)
        .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
        .push_next(&mut binding_flags_create_info)
        .build();

    unsafe {
        device
            .device()
            .create_descriptor_set_layout(&descriptor_sets_layout_info, None)
            .expect("Error creating descriptor set layout")
    }
}

pub fn create_bindless_descriptor_set(
    device: &Device,
    layout: vk::DescriptorSetLayout,
) -> vk::DescriptorSet {
    let descriptor_sizes = [vk::DescriptorPoolSize {
        ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: MAX_BINDLESS_DESCRIPTOR_COUNT as u32,
    }];

    let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(&descriptor_sizes)
        .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
        .max_sets(1);

    let descriptor_pool = unsafe {
        device
            .ash_device
            .create_descriptor_pool(&descriptor_pool_info, None)
            .expect("Error allocating bindless descriptor pool")
    };

    let variable_descriptor_count = MAX_BINDLESS_DESCRIPTOR_COUNT as u32;
    let mut variable_descriptor_count_allocate_info =
        vk::DescriptorSetVariableDescriptorCountAllocateInfo::builder()
            .descriptor_counts(std::slice::from_ref(&variable_descriptor_count))
            .build();

    let descriptor_set = unsafe {
        device
            .ash_device
            .allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::builder()
                    .descriptor_pool(descriptor_pool)
                    .set_layouts(std::slice::from_ref(&layout))
                    .push_next(&mut variable_descriptor_count_allocate_info)
                    .build(),
            )
            .expect("Error allocating bindless descriptor pool")[0]
    };

    descriptor_set
}
