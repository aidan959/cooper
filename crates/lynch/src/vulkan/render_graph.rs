use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::Arc;

use ash::vk::{self};
use crate::renderer::Renderer;
use crate::vulkan::renderer::{VulkanRenderer};


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
pub struct RenderGraphResources {
    pub buffers: Vec<GraphBuffer>,
    pub textures: Vec<GraphTexture>,
    pub pipelines: Vec<Pipeline>,
}

pub enum DepthAttachment {
    GraphHandle(Attachment),
    External(Image, vk::AttachmentLoadOp),
}


#[derive(Copy, Clone)]
pub struct Attachment {
    pub texture: TextureId,
    pub view: ViewType,
    pub load_op: vk::AttachmentLoadOp,
}





pub struct RenderGraph {
    pub passes: Vec<Vec<RenderPass>>,
    pub resources: GraphResources,
    pub descriptor_set_camera: Vec<DescriptorSet>,
    pub pipeline_descs: Vec<PipelineDesc>,
    pub current_frame: usize,
    pub device: Arc<Device>,
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
            current_frame: 0,
            device: device.clone(),
        }
    }
    pub fn render(
        &mut self,
        command_buffer: &vk::CommandBuffer,
        renderer: &VulkanRenderer,
        present_image: &[Image], // Todo: pass single value
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

            if let Some(extra_barriers) = &pass.extra_barriers {
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
            let mut writes_for_synch = pass.writes.clone();
            // If the depth attachment is owned by the graph make sure it gets a barrier as well
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
                    &present_image[0],
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
    
                assert_eq!(present_image.len(), 1);
                let present_image = [(
                    present_image[0].clone(),
                    ViewType::Full(),
                    vk::AttachmentLoadOp::CLEAR,
                )];
        }
        todo!(); // complete
    }
    pub fn recompile_all_shaders(
        &mut self,
        device: &Device,
        bindless_descriptor_set_layout: Option<vk::DescriptorSetLayout>,
    ) {
        for pipeline in &mut self.resources.pipelines {
            pipeline.recreate_pipeline(device, bindless_descriptor_set_layout);
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
        device: &Device,
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
            DescriptorSet::new(device, descriptor_set_layout, binding_map);

        descriptor_set_camera.write_uniform_buffer(
            device,
            "view".to_string(),
            camera_uniform_buffer,
        );

        descriptor_set_camera
    } 
    
}


pub struct RenderPassBuilder {
    pub name: String,
    pub pipeline_handle: PipelineId,
    pub reads: Vec<Resource>,
    pub writes: Vec<Attachment>,
    pub render_func:
        Option<Box<dyn Fn(&Device, &vk::CommandBuffer, &VulkanRenderer, &RenderPass, &GraphResources)>>,
    pub depth_attachment: Option<DepthAttachment>,
    pub presentation_pass: bool,
    pub uniforms: HashMap<String, (String, UniformData)>,
    pub copy_command: Option<TextureCopy>,
    pub extra_barriers: Option<Vec<(BufferId, vk_sync::AccessType)>>,
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
        self.render_func.replace(Box::new(render_func));
        self
    }
    pub fn dispatch(mut self, group_count_x: u32, group_count_y: u32, group_count_z: u32) -> Self {
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
    pub fn presentation_pass(mut self, is_presentation_pass: bool) -> Self {
        self.presentation_pass = is_presentation_pass;
        self
    }
    pub fn depth_attachment(mut self, depth_attachment: TextureId) -> Self {
        self.depth_attachment = Some(DepthAttachment::GraphHandle(Attachment {
            texture: depth_attachment,
            view: ViewType::Full(),
            load_op: vk::AttachmentLoadOp::CLEAR, // Todo
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
    pub fn build(self, graph: &mut RenderGraph) {
        let mut pass = RenderPass::new(
            self.name,
            self.pipeline_handle,
            self.render_func,
            self.device.clone(),
        );

        for read in &self.reads {
            pass.reads.push(*read);
        }

        for write in &self.writes {
            pass.writes.push(*write);
        }


        graph.passes[graph.current_frame].push(pass);
    }
}
impl RenderGraphResources {
    fn new() -> RenderGraphResources {
        RenderGraphResources {
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