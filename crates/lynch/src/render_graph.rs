use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::Arc;

use ash::vk::{self, Framebuffer, ImageView};

use crate::renderer::Renderer;
use crate::vulkan::renderer::{
    VulkanRenderer, DESCRIPTOR_SET_INDEX_BINDLESS, DESCRIPTOR_SET_INDEX_INPUT_TEXTURES,
    DESCRIPTOR_SET_INDEX_VIEW,
};
use crate::vulkan::{
    Buffer, DescriptorSet, Device, Image, ImageDesc, Pipeline, PipelineDesc, PipelineDescBuilder,
    PipelineType, RenderPass,
};
use crate::{vulkan, Texture};

pub type TextureId = usize;
pub type BufferId = usize;
pub type PipelineId = usize;


pub struct GraphTexture {
    pub texture: Texture,
    pub prev_access: vk_sync::AccessType,
}
pub struct GraphBuffer {
    pub buffer: Buffer,
    pub prev_access: vk_sync::AccessType,
}
// TODO These need to be destroyed on release
pub struct GraphResources {
    pub buffers: Vec<GraphBuffer>,
    pub textures: Vec<GraphTexture>,
    pub pipelines: Vec<Pipeline>,
}

pub enum DepthAttachment {
    GraphHandle(Attachment),
    External(Image, vk::AttachmentLoadOp),
}

#[derive(Copy, Clone)]
pub enum ViewType {
    Full(),
    Layer(u32),
}

#[derive(Copy, Clone)]
pub struct Attachment {
    pub texture: TextureId,
    pub view: ViewType,
    pub load_op: vk::AttachmentLoadOp,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TextureResourceType {
    CombinedImageSampler,
    StorageImage,
}

#[derive(Copy, Clone)]
pub struct TextureResource {
    pub texture: TextureId,
    pub input_type: TextureResourceType,
    pub access_type: vk_sync::AccessType,
}

#[derive(Copy, Clone)]
pub struct BufferResource {
    pub buffer: BufferId,
    pub access_type: vk_sync::AccessType,
}

#[derive(Copy, Clone)]
pub enum Resource {
    Texture(TextureResource),
    Buffer(BufferResource),
}
pub struct TextureCopy {
    pub src: TextureId,
    pub dst: TextureId,
    pub copy_desc: vk::ImageCopy,
}
pub struct RenderGraph{
    pub passes: Vec<Vec<RenderPass>>,
    pub render_passes: HashMap<String, vk::RenderPass>,
    pub render_subpasses: HashMap<String, vk::SubpassDependency>,

    pub render_framebuffers: HashMap<String, Vec<vk::Framebuffer>>,

    pub resources: GraphResources,
    pub descriptor_set_camera: Vec<DescriptorSet>,
    pub pipeline_descs: Vec<PipelineDesc>,
    pub current_frame: usize,
    pub device: Arc<Device>,
}
// TODO REMOVE THIS CONSTANT
pub const MAX_UNIFORMS_SIZE: usize = 2048;

// TODO DYNAMIC DESCRIPTOR SETS
#[derive(Copy, Clone)]
pub struct UniformData {
    pub data: [MaybeUninit<u8>; MAX_UNIFORMS_SIZE],
    pub size: u64,
}

pub struct RenderPassBuilder {
    pub name: String,
    pub pipeline_handle: PipelineId,
    pub reads: Vec<Resource>,
    pub writes: Vec<Attachment>,
    pub render_func: Option<
        Box<dyn Fn(&Device, &vk::CommandBuffer, &VulkanRenderer, &RenderPass, &GraphResources)>,
    >,
    pub depth_attachment: Option<DepthAttachment>,
    pub presentation_pass: bool,
    pub uniforms: HashMap<String, (String, UniformData)>,
    pub copy_command: Option<TextureCopy>,
    pub extra_barriers: Option<Vec<(BufferId, vk_sync::AccessType)>>,
    pub device: Arc<Device>,
}

impl RenderPassBuilder {
    pub fn read(mut self, resource_id: TextureId) -> Self {
        self.reads.push(Resource::Texture(TextureResource {
            texture: resource_id,
            input_type: TextureResourceType::CombinedImageSampler,
            access_type: vk_sync::AccessType::AnyShaderReadSampledImageOrUniformTexelBuffer,
        }));
        self
    }

    pub fn image_write(mut self, resource_id: TextureId) -> Self {
        self.reads.push(Resource::Texture(TextureResource {
            texture: resource_id,
            input_type: TextureResourceType::StorageImage,
            access_type: vk_sync::AccessType::AnyShaderWrite,
        }));
        self
    }

    pub fn write_buffer(mut self, resource_id: BufferId) -> Self {
        self.reads.push(Resource::Buffer(BufferResource {
            buffer: resource_id,
            access_type: vk_sync::AccessType::AnyShaderWrite,
        }));
        self
    }

    pub fn read_buffer(mut self, resource_id: BufferId) -> Self {
        self.reads.push(Resource::Buffer(BufferResource {
            buffer: resource_id,
            access_type: vk_sync::AccessType::AnyShaderReadOther,
        }));
        self
    }

    pub fn write(mut self, resource_id: TextureId) -> Self {
        self.writes.push(Attachment {
            texture: resource_id,
            view: ViewType::Full(),
            load_op: vk::AttachmentLoadOp::CLEAR,
        });
        self
    }

    pub fn write_layer(mut self, resource_id: TextureId, layer: u32) -> Self {
        self.writes.push(Attachment {
            texture: resource_id,
            view: ViewType::Layer(layer),
            load_op: vk::AttachmentLoadOp::CLEAR,
        });
        self
    }

    pub fn load_write(mut self, resource_id: TextureId) -> Self {
        self.writes.push(Attachment {
            texture: resource_id,
            view: ViewType::Full(),
            load_op: vk::AttachmentLoadOp::LOAD,
        });
        self
    }

    pub fn record_render(
        mut self,
        render_func: impl Fn(&Device, &vk::CommandBuffer, &VulkanRenderer, &RenderPass, &GraphResources)
            + 'static,
    ) -> Self {
        drop(self.render_func.replace(Box::new(render_func)));
        self
    }

    pub fn dispatch_compute(
        mut self,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ) -> Self {
        self.render_func
            .replace(Box::new(move |device, command_buffer, _, _, _| unsafe {
                device.device().cmd_dispatch(
                    *command_buffer,
                    group_count_x,
                    group_count_y,
                    group_count_z,
                );
            }));
        self
    }

    pub fn copy_image(mut self, src: TextureId, dst: TextureId, copy_desc: vk::ImageCopy) -> Self {
        self.copy_command.replace(TextureCopy {
            src,
            dst,
            copy_desc,
        });
        self
    }

    pub fn presentation_pass(mut self, is_presentation_pass: bool) -> Self {
        self.presentation_pass = is_presentation_pass;
        self
    }
    pub fn depth_attachment(mut self, depth_attachment: TextureId) -> Self {
        self.depth_attachment = Some(DepthAttachment::GraphHandle(Attachment {
            texture: depth_attachment,
            view: ViewType::Full(),
            load_op: vk::AttachmentLoadOp::CLEAR,
        }));
        self
    }

    pub fn depth_attachment_layer(mut self, depth_attachment: TextureId, layer: u32) -> Self {
        self.depth_attachment = Some(DepthAttachment::GraphHandle(Attachment {
            texture: depth_attachment,
            view: ViewType::Layer(layer),
            load_op: vk::AttachmentLoadOp::CLEAR,
        }));
        self
    }

    pub fn external_depth_attachment(
        mut self,
        depth_attachment: Image,
        load_op: vk::AttachmentLoadOp,
    ) -> Self {
        self.depth_attachment = Some(DepthAttachment::External(depth_attachment, load_op));
        self
    }

    pub fn uniforms<T: Copy + std::fmt::Debug>(mut self, name: &str, data: &T) -> Self {
        unsafe {
            let ptr = data as *const _ as *const MaybeUninit<u8>;
            let size = std::mem::size_of::<T>();
            let data_u8 = std::slice::from_raw_parts(ptr, size);

            assert!(data_u8.len() < MAX_UNIFORMS_SIZE);

            let unique_name = self.name.clone() + "_" + name;

            if let Some(entry) = self.uniforms.get_mut(&unique_name) {
                entry.1.data[..data_u8.len()].copy_from_slice(data_u8);
                entry.1.size = size as u64;
            } else {
                let mut new_entry = UniformData {
                    data: [MaybeUninit::zeroed(); MAX_UNIFORMS_SIZE],
                    size: size as u64,
                };
                new_entry.data[..data_u8.len()].copy_from_slice(data_u8);
                self.uniforms
                    .insert(unique_name.to_string(), (name.to_string(), new_entry));
            }
        }
        self
    }

    pub fn build(self, graph: &mut RenderGraph, extent: vk::Extent2D) {
        

        let mut pass = RenderPass::new(
            self.name,
            self.pipeline_handle,
            self.presentation_pass,
            self.depth_attachment,
            self.uniforms.clone(),
            self.render_func,
            self.copy_command,
            self.extra_barriers,
            self.device.clone(),
        );

        for read in &self.reads {
            pass.reads.push(*read);
        }

        for write in &self.writes {
            pass.writes.push(*write);
        }
        graph.pipeline_descs[pass.pipeline_handle].color_attachment_formats = pass
            .writes
            .iter()
            .map(|write| {

                graph
                    .resources
                    .texture(write.texture)
                    .texture
                    .image
                    .format()
            })
            .collect();

    
        let color_attachment_formats = graph.pipeline_descs[pass.pipeline_handle].color_attachment_formats.clone();
        let mut depth_attachment_format = None;
        if let Some(depth) = &pass.depth_attachment {
            match depth {
                DepthAttachment::GraphHandle(write) => {
                    depth_attachment_format = Some(graph
                        .resources
                        .texture(write.texture)
                        .texture
                        .image
                        .format());
                    graph.pipeline_descs[pass.pipeline_handle].depth_stencil_attachment_format =
                        graph
                            .resources
                            .texture(write.texture)
                            .texture
                            .image
                            .format()
                    
                }
                DepthAttachment::External(image, _) => {
                    depth_attachment_format = None;
                    graph.pipeline_descs[pass.pipeline_handle].depth_stencil_attachment_format =
                        image.format();


                }
            }
        };
        
        graph.render_passes.insert(
            pass.name.clone(),
            vulkan::create_render_pass(
                &self.device,
                color_attachment_formats,
                depth_attachment_format
            ),
        );

        let render_pass = graph.render_passes.get(&pass.name).unwrap();
        let image_views = pass
            .writes
            .iter()
            .map(|write| { 
                graph.resources.texture(write.texture).texture.image.image_view.clone()
            }).collect();
        
        
        if !graph.render_framebuffers.contains_key(&pass.name) {
            graph.render_framebuffers.insert(
                pass.name.clone(),
                vec![vulkan::create_vulkan_framebuffer(
                    &graph.device, 
                    *render_pass,
                    extent,
                    &image_views,
                )],
            );
        }
        if !self.uniforms.is_empty() {
            pass.uniform_buffer.replace(
                graph.create_buffer(
                    // Todo: Hack: this is very bad just to get unique buffers for every frame_index
                    format!(
                        "{}_frame_{}",
                        self.uniforms.keys().next().unwrap(),
                        graph.current_frame
                    )
                    .as_str(),
                    self.uniforms.values().next().unwrap().1.size,
                    vk::BufferUsageFlags::UNIFORM_BUFFER,
                    gpu_allocator::MemoryLocation::CpuToGpu,
                ),
            );
        }
        
        graph.passes[graph.current_frame].push(pass);
    }
}

impl GraphResources {
    fn new() -> GraphResources {
        GraphResources {
            buffers: vec![],
            textures: vec![],
            pipelines: vec![],
        }
    }

    pub fn buffer(&self, id: BufferId) -> &GraphBuffer {
        &self.buffers[id]
    }

    pub fn texture(&self, id: TextureId) -> &GraphTexture {
        &self.textures[id]
    }

    pub fn pipeline(&self, id: PipelineId) -> &Pipeline {
        &self.pipelines[id]
    }
}
impl Drop for RenderGraph {
    fn drop(&mut self) {
        self.descriptor_set_camera
            .iter()
            .for_each(|ds| ds.clean_vk_resources());
        self.resources.buffers.iter().for_each(|b| unsafe {
            self.device.ash_device.destroy_buffer(b.buffer.buffer, None);
        });
        self.resources.pipelines.iter().for_each(|b| unsafe {
            self.device
                .ash_device
                .destroy_pipeline_layout(b.pipeline_layout, None);
        });
        self.resources.textures.iter().for_each(|b| {
            b.texture.clean_vk_resources();
        });
    }
}
impl RenderGraph {
    pub fn new(
        device: Arc<Device>,
        camera_uniform_buffer: &Vec<Buffer>,
        num_frames_in_flight: u32,
    ) -> Self {
        RenderGraph {
            passes: (0..num_frames_in_flight).map(|_| vec![]).collect(),
            resources: GraphResources::new(),
            descriptor_set_camera: (*camera_uniform_buffer)
                .iter()
                .map(|buffer| Self::create_camera_descriptor_set(device.clone(), buffer))
                .collect(),
            pipeline_descs: vec![],
            render_passes: HashMap::new(),
            render_framebuffers: HashMap::new(),
            
            render_subpasses: HashMap::new(),
            current_frame: 0,
            device: device,
        }
    }

    pub fn new_frame(&mut self, current_frame: usize) {
        self.current_frame = current_frame;
    }

    pub fn clear(&mut self) {
        for pass in &self.passes[self.current_frame] {
            if let Some(descriptor_set) = &pass.uniform_descriptor_set {
                unsafe {
                    self.device
                        .ash_device
                        .destroy_descriptor_pool(descriptor_set.pool, None)
                };
            }
            if let Some(descriptor_set) = &pass.read_resources_descriptor_set {
                unsafe {
                    self.device
                        .ash_device
                        .destroy_descriptor_pool(descriptor_set.pool, None)
                };
            }
        }

        self.passes[self.current_frame].clear();
    }

    pub fn create_camera_descriptor_set(
        device: Arc<Device>,
        camera_uniform_buffer: &Buffer,
    ) -> DescriptorSet {
        let descriptor_set_layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::ALL)
            .build();

        let descriptor_sets_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&[descriptor_set_layout_binding])
            .build();

        let descriptor_set_layout = unsafe {
            device
                .device()
                .create_descriptor_set_layout(&descriptor_sets_layout_info, None)
                .expect("Error creating descriptor set layout")
        };

        let mut binding_map: vulkan::shader::BindingMap = std::collections::BTreeMap::new();
        binding_map.insert(
            "view".to_string(),
            vulkan::shader::Binding {
                set: DESCRIPTOR_SET_INDEX_VIEW,
                binding: 0,
                info: rspirv_reflect::DescriptorInfo {
                    ty: rspirv_reflect::DescriptorType::UNIFORM_BUFFER,
                    binding_count: rspirv_reflect::BindingCount::One,
                    name: "view".to_string(),
                },
            },
        );

        let descriptor_set_camera =
            DescriptorSet::new(device.clone(), descriptor_set_layout, binding_map);

        descriptor_set_camera.write_uniform_buffer("view".to_string(), camera_uniform_buffer);

        descriptor_set_camera
    }

    fn add_pass(&mut self, name: String, pipeline_handle: PipelineId) -> RenderPassBuilder {

        RenderPassBuilder {
            name,
            pipeline_handle,
            reads: vec![],
            writes: vec![],
            render_func: None,
            depth_attachment: None,
            presentation_pass: false,
            uniforms: HashMap::new(),
            copy_command: None,
            extra_barriers: None,
            device: self.device.clone(),
        }
    }

    pub fn add_pass_from_desc(
        &mut self,
        name: &str,
        desc_builder: PipelineDescBuilder,
    ) -> RenderPassBuilder {
        let desc = desc_builder.build();
        let pipeline_handle = self.create_pipeline(desc);
        
        self.add_pass(name.to_string(), pipeline_handle)
    }

    /// Creates a texture and returns its handle.
    ///
    /// If a texture with the same name already exists, it will be returned instead.
    /// Compared to `graph::create_pipeline`, this function does not defer the creation of the texture.
    pub fn create_texture(
        &mut self,
        debug_name: &str,
        device: Arc<Device>,
        image_desc: ImageDesc,
    ) -> TextureId {
        // Todo: Cannot rely on debug_name being unique
        // Todo: shall use a Hash to include extent and format of the texture
        self.resources
            .textures
            .iter()
            .position(|iter| iter.texture.image.debug_name == debug_name)
            .unwrap_or_else(|| {
                let texture = crate::Texture::create(device, None, image_desc, debug_name);

                self.resources.textures.push(GraphTexture {
                    texture,
                    prev_access: vk_sync::AccessType::Nothing,
                });

                self.resources.textures.len() - 1
            })
    }

    /// Creates a buffer and returns its handle.
    ///
    /// If a buffer with the same name already exists, it will be returned instead.
    /// Compared to `graph::create_pipeline`, this function does not defer the creation of the buffer.
    pub fn create_buffer(
        &mut self,
        debug_name: &str,
        size: u64,
        usage: vk::BufferUsageFlags,
        memory_location: gpu_allocator::MemoryLocation,
    ) -> BufferId {
        self.resources
            .buffers
            .iter()
            .position(|iter| iter.buffer.debug_name == debug_name)
            .unwrap_or_else(|| {
                let mut buffer = Buffer::new::<u8>(
                    self.device.clone(),
                    None,
                    size,
                    usage,
                    memory_location,
                    Some(String::from(debug_name)),
                );

                buffer.set_debug_name(debug_name);

                self.resources.buffers.push(GraphBuffer {
                    buffer,
                    prev_access: vk_sync::AccessType::Nothing,
                });

                self.resources.buffers.len() - 1
            })
    }

    /// Creates a pipeline and returns its handle.
    pub fn create_pipeline(&mut self, pipeline_desc: PipelineDesc) -> PipelineId {
        if let Some(existing_pipeline_id) = self
            .pipeline_descs
            .iter()
            .position(|desc| *desc == pipeline_desc)
        {
            existing_pipeline_id
        } else {
            self.pipeline_descs.push(pipeline_desc);
            self.pipeline_descs.len() - 1
        }
    }

    pub fn prepare(&mut self, renderer: &VulkanRenderer) {
        let device = renderer.device();
        // Todo: shall be possible to create the pipelines using multiple threads
        for (i, desc) in self.pipeline_descs.iter().enumerate() {
            if self.resources.pipelines.len() <= i {
                self.resources.pipelines.push(Pipeline::new(
                    device,
                    desc.clone(),
                    Some(renderer.internal_renderer.bindless_descriptor_set_layout),
                    *self.render_passes.get(self.passes[self.current_frame][i].name.as_str()).unwrap(),
                ));
            }
        }
        for pass in &mut self.passes[self.current_frame] {
            pass.try_create_read_resources_descriptor_set(
                &self.resources.pipelines,
                &self.resources.textures,
                &self.resources.buffers,
                vk::AccelerationStructureKHR::null(),
            );
            pass.try_create_uniform_buffer_descriptor_set(
                &self.resources.pipelines,
                &self.resources.buffers,
            );

            pass.update_uniform_buffer_memory(&mut self.resources.buffers);
        }
    }

    pub fn recompile_all_shaders(
        &mut self,
        bindless_descriptor_set_layout: Option<vk::DescriptorSetLayout>,
    ) {
        for pipeline in &mut self.resources.pipelines {
            pipeline.recreate_pipeline(&self.device, bindless_descriptor_set_layout);
        }
    }

    pub fn recompile_shader(
        &mut self,
        device: &Device,
        bindless_descriptor_set_layout: Option<vk::DescriptorSetLayout>,
        path: std::path::PathBuf,
    ) {
        for pipeline in &mut self.resources.pipelines {
            let desc = &pipeline.pipeline_desc;
            if desc.compute_path.map_or(false, |p| path.ends_with(p))
                || desc.vertex_path.map_or(false, |p| path.ends_with(p))
                || desc.fragment_path.map_or(false, |p| path.ends_with(p))
            {
                pipeline.recreate_pipeline(device, bindless_descriptor_set_layout);
            }
        }
    }

    pub fn render(
        &mut self,
        command_buffer: &vk::CommandBuffer,
        renderer: &VulkanRenderer,
        present_image: &Image,
        present_index: usize,
    ) {
        let device = renderer.device();
        for pass in &self.passes[self.current_frame] {
            let pass_pipeline = &self.resources.pipelines[pass.pipeline_handle];
            for read in &pass.reads {
                match read {
                    Resource::Texture(read) => {
                        let next_access = vulkan::image_pipeline_barrier(
                            device,
                            *command_buffer,
                            &self.resources.textures[read.texture].texture.image,
                            self.resources.textures[read.texture].prev_access,
                            read.access_type,
                            false,
                        );

                        self.resources
                            .textures
                            .get_mut(read.texture)
                            .unwrap()
                            .prev_access = next_access;
                    }

                    Resource::Buffer(read) => {
                        let next_access = vulkan::global_pipeline_barrier(
                            device,
                            *command_buffer,
                            self.resources.buffers[read.buffer].prev_access,
                            read.access_type,
                        );

                        self.resources
                            .buffers
                            .get_mut(read.buffer)
                            .unwrap()
                            .prev_access = next_access;
                    }
                }
            }

            match &pass.extra_barriers {
                Some(extra_barriers) => {
                    for (buffer_id, access_type) in extra_barriers {
                        let next_access = vulkan::global_pipeline_barrier(
                            device,
                            *command_buffer,
                            self.resources.buffers[*buffer_id].prev_access,
                            *access_type,
                        );
                        if let Some(buffer) = self.resources.buffers.get_mut(*buffer_id) {
                            buffer.prev_access = next_access;
                        }
                    }
                }
                None => {}
            }

            let mut writes_for_synch = pass.writes.clone();

            if pass.depth_attachment.is_some() {
                if let DepthAttachment::GraphHandle(depth_attachment) =
                    pass.depth_attachment.as_ref().unwrap()
                {
                    writes_for_synch.push(*depth_attachment);
                }
            }

            for write in &writes_for_synch {
                let next_access = vulkan::image_pipeline_barrier(
                    device,
                    *command_buffer,
                    &self.resources.textures[write.texture].texture.image,
                    self.resources.textures[write.texture].prev_access,
                    if Image::is_depth_image_fmt(
                        self.resources.textures[write.texture]
                            .texture
                            .image
                            .desc
                            .format,
                    ) {
                        vk_sync::AccessType::DepthStencilAttachmentWrite
                    } else {
                        vk_sync::AccessType::ColorAttachmentWrite
                    },
                    false,
                );

                self.resources
                    .textures
                    .get_mut(write.texture)
                    .unwrap()
                    .prev_access = next_access;
            }

            if pass.presentation_pass {
                vulkan::image_pipeline_barrier(
                    device,
                    *command_buffer,
                    present_image,
                    vk_sync::AccessType::Present,
                    vk_sync::AccessType::ColorAttachmentWrite,
                    false,
                );
            }

            let write_attachments: Vec<(Image, ViewType, vk::AttachmentLoadOp)> = pass
                .writes
                .iter()
                .map(|write| {
                    (
                        self.resources.textures[write.texture].texture.image.clone(),
                        write.view,
                        write.load_op,
                    )
                })
                .collect();
            let extent = if !pass.writes.is_empty() {
                vk::Extent2D {
                    width: self.resources.textures[pass.writes[0].texture]
                        .texture
                        .image
                        .width(),
                    height: self.resources.textures[pass.writes[0].texture]
                        .texture
                        .image
                        .height(),
                }
            } else if pass.depth_attachment.is_some() {
                match pass.depth_attachment.as_ref().unwrap() {
                    DepthAttachment::GraphHandle(depth_attachment) => vk::Extent2D {
                        width: self.resources.textures[depth_attachment.texture]
                            .texture
                            .image
                            .width(),
                        height: self.resources.textures[depth_attachment.texture]
                            .texture
                            .image
                            .height(),
                    },
                    DepthAttachment::External(depth_attachment, _) => vk::Extent2D {
                        width: depth_attachment.width(),
                        height: depth_attachment.height(),
                    },
                }
            } else {
                vk::Extent2D {
                    width: 1,
                    height: 1,
                }
            };

            let present_image = [(
                present_image.clone(),
                ViewType::Full(),
                vk::AttachmentLoadOp::CLEAR,
            )];
            let framebuffer = self.render_framebuffers.get(&pass.name).unwrap()[present_index];
            let renderpass = self.render_passes.get(&pass.name).unwrap();
            pass.prepare_render(
                command_buffer,
                if !pass.presentation_pass {
                    write_attachments.as_slice()
                } else {
                    &present_image
                },
                if pass.depth_attachment.is_some() {
                    match pass.depth_attachment.as_ref().unwrap() {
                        DepthAttachment::GraphHandle(depth_attachment) => Some((
                            self.resources.textures[depth_attachment.texture]
                                .texture
                                .image
                                .clone(),
                            depth_attachment.view,
                            depth_attachment.load_op,
                        )),
                        DepthAttachment::External(depth_attachment, load_op) => {
                            Some((depth_attachment.clone(), ViewType::Full(), *load_op))
                        }
                    }
                } else {
                    None
                },
                if !pass.presentation_pass {
                    extent
                } else {
                    vk::Extent2D {
                        width: present_image[0].0.width(),
                        height: present_image[0].0.height(),
                    }
                },
                &self.resources.pipelines,
                *renderpass,
                framebuffer
            );
            unsafe {
                let bind_point = match pass_pipeline.pipeline_type {
                    PipelineType::Graphics => vk::PipelineBindPoint::GRAPHICS,
                    PipelineType::Compute => vk::PipelineBindPoint::COMPUTE,
                };

                device.device().cmd_bind_descriptor_sets(
                    *command_buffer,
                    bind_point,
                    pass_pipeline.pipeline_layout,
                    DESCRIPTOR_SET_INDEX_BINDLESS,
                    &[renderer.internal_renderer.bindless_descriptor_set],
                    &[],
                );

                device.device().cmd_bind_descriptor_sets(
                    *command_buffer,
                    bind_point,
                    pass_pipeline.pipeline_layout,
                    DESCRIPTOR_SET_INDEX_VIEW,
                    &[self.descriptor_set_camera[self.current_frame].handle],
                    &[],
                );

                if let Some(read_textures_descriptor_set) = &pass.read_resources_descriptor_set {
                    device.device().cmd_bind_descriptor_sets(
                        *command_buffer,
                        bind_point,
                        pass_pipeline.pipeline_layout,
                        DESCRIPTOR_SET_INDEX_INPUT_TEXTURES,
                        &[read_textures_descriptor_set.handle],
                        &[],
                    )
                }

                if let Some(uniforms_descriptor_set) = &pass.uniform_descriptor_set {
                    device.device().cmd_bind_descriptor_sets(
                        *command_buffer,
                        bind_point,
                        pass_pipeline.pipeline_layout,
                        pass_pipeline
                            .reflection
                            .get_binding(&pass.uniforms.values().next().unwrap().0)
                            .set,
                        &[uniforms_descriptor_set.handle],
                        &[],
                    )
                }
            };

            if let Some(render_func) = &pass.render_func {
                render_func(device, command_buffer, renderer, pass, &self.resources);
            }

            if pass_pipeline.pipeline_type == PipelineType::Graphics {
                //unsafe { device.device().cmd_end_rendering(*command_buffer) };
                unsafe { device.device().cmd_end_render_pass(*command_buffer) };

            }

            if let Some(copy_command) = &pass.copy_command {
                let src = copy_command.src;
                let dst = copy_command.dst;

                let next_access = vulkan::image_pipeline_barrier(
                    device,
                    *command_buffer,
                    &self.resources.textures[src].texture.image,
                    self.resources.textures[src].prev_access,
                    vk_sync::AccessType::TransferRead,
                    false,
                );
                self.resources.textures.get_mut(src).unwrap().prev_access = next_access;

                let next_access = vulkan::image_pipeline_barrier(
                    device,
                    *command_buffer,
                    &self.resources.textures[dst].texture.image,
                    self.resources.textures[dst].prev_access,
                    vk_sync::AccessType::TransferWrite,
                    false,
                );
                self.resources.textures.get_mut(dst).unwrap().prev_access = next_access;

                let src = &self.resources.textures[src].texture.image;
                let dst = &self.resources.textures[dst].texture.image;

                let mut copy_desc = copy_command.copy_desc;
                copy_desc.src_subresource.aspect_mask = src.desc.aspect_flags;
                copy_desc.dst_subresource.aspect_mask = dst.desc.aspect_flags;

                unsafe {
                    device.device().cmd_copy_image(
                        *command_buffer,
                        src.image,
                        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                        dst.image,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[copy_desc],
                    )
                };
            }
        }
    }
}
