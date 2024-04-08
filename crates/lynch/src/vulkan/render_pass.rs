use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::Arc;

use ash::vk;
use vk_sync::AccessType;

use crate::render_graph::{
    Attachment, BufferId, BufferResource, DepthAttachment, GraphBuffer, GraphResources, GraphTexture, PipelineId, RenderGraph, Resource, TextureCopy, TextureId, TextureResource, TextureResourceType, UniformData, ViewType, MAX_UNIFORMS_SIZE
};

use super::descriptor::{DescriptorIdentifier, DescriptorSet};
use super::renderer::VulkanRenderer;
use super::{Device, Image, Pipeline, PipelineType};

pub struct RenderPass {
    pub name: String,
    pub is_pres_pass: bool,
    pub pipeline_handle: PipelineId,
    pub render_func: Option<
        Box<dyn Fn(&Device, &vk::CommandBuffer, &VulkanRenderer, &RenderPass, &GraphResources)>,
        >,
    pub writes: Vec<Attachment>,
    pub reads: Vec<Resource>,
    pub depth_attachment: Option<DepthAttachment>,
    pub uniforms: HashMap<String, (String, UniformData)>,
    pub read_resources_descriptor_set: Option<DescriptorSet>,
    pub uniform_descriptor_set: Option<DescriptorSet>,
    pub copy_command: Option<TextureCopy>,
    pub uniform_buffer: Option<BufferId>,
    pub extra_barriers: Option<Vec<(BufferId, AccessType)>>,
    device: Arc<Device>,
}

impl RenderPass {
    pub fn new(
        name: String,
        pipeline_handle: PipelineId,
        presentation_pass: bool,
        depth_attachment: Option<DepthAttachment>,
        uniforms: HashMap<String, (String, UniformData)>,
        render_func: Option<
            Box<dyn Fn(&Device, &vk::CommandBuffer, &VulkanRenderer, &RenderPass, &GraphResources)>,
        >,
        copy_command: Option<TextureCopy>,
        extra_barriers: Option<Vec<(BufferId, AccessType)>>,
        device: Arc<Device>,
    ) -> RenderPass {
        RenderPass {
            name,
            pipeline_handle,
            render_func,
            reads: Vec::new(),
            writes: Vec::new(),
            depth_attachment,
            is_pres_pass: presentation_pass,
            read_resources_descriptor_set: None,
            uniforms,
            uniform_buffer: None,
            uniform_descriptor_set: None,
            copy_command,
            extra_barriers,
            device,
        }
    }

    pub fn try_create_read_resources_descriptor_set(
        &mut self,
        pipelines: &[Pipeline],
        textures: &[GraphTexture],
        buffers: &[GraphBuffer],
    ) {
        if !(!self.reads.is_empty() && self.read_resources_descriptor_set.is_none()) {
            return;
        }
        let descriptor_set_read_resources = DescriptorSet::new(
            self.device.clone(),
            pipelines[self.pipeline_handle].descriptor_set_layouts
                [super::renderer::DESCRIPTOR_SET_INDEX_INPUT_TEXTURES as usize],
            pipelines[self.pipeline_handle]
                .reflection
                .get_set_mappings(super::renderer::DESCRIPTOR_SET_INDEX_INPUT_TEXTURES),
        );

        for (idx, &read) in self.reads.iter().enumerate() {
            match read {
                Resource::Texture(read) => {
                    if read.input_type == TextureResourceType::CombinedImageSampler {
                        descriptor_set_read_resources.write_combined_image(
                            &self.device,
                            DescriptorIdentifier::Index(idx as u32),
                            &textures[read.texture].texture,
                        );
                    } else if read.input_type == TextureResourceType::StorageImage {
                        descriptor_set_read_resources.write_storage_image(
                            &self.device,
                            DescriptorIdentifier::Index(idx as u32),
                            &textures[read.texture].texture.image,
                        );
                    }
                }
                Resource::Buffer(read) => {
                    descriptor_set_read_resources.write_storage_buffer(
                        &self.device,
                        DescriptorIdentifier::Index(idx as u32),
                        &buffers[read.buffer].buffer,
                    );
                }
            }
        }

        self.read_resources_descriptor_set
            .replace(descriptor_set_read_resources);
    }

    pub fn try_create_uniform_buffer_descriptor_set(
        &mut self,
        pipelines: &[Pipeline],
        buffers: &[GraphBuffer],
    ) {
        if !self.uniforms.is_empty() && self.uniform_descriptor_set.is_none() {
            let uniform_name = &self.uniforms.values().next().unwrap().0;
            let binding = pipelines[self.pipeline_handle]
                .reflection
                .get_binding(uniform_name);
            let descriptor_set = DescriptorSet::new(
                self.device.clone(),
                pipelines[self.pipeline_handle].descriptor_set_layouts[binding.set as usize],
                pipelines[self.pipeline_handle]
                    .reflection
                    .get_set_mappings(binding.set),
            );
            {
                descriptor_set.write_uniform_buffer(
                    uniform_name.to_string(),
                    &buffers[self.uniform_buffer.unwrap()].buffer,
                );
            }

            self.uniform_descriptor_set.replace(descriptor_set);
        }
    }

    pub fn update_uniform_buffer_memory(&mut self, buffers: &mut [GraphBuffer]) {
        if let Some(buffer_id) = self.uniform_buffer {
            buffers[buffer_id]
                .buffer
                .update_memory(&self.uniforms.values().next().unwrap().1.data)
        }
    }

    pub fn prepare_render(
        &self,
        command_buffer: &vk::CommandBuffer,
        color_attachments: &[(Image, ViewType, vk::AttachmentLoadOp)],
        depth_attachment: Option<(Image, ViewType, vk::AttachmentLoadOp)>,
        extent: vk::Extent2D,
        pipelines: &[Pipeline],
    ) {
        let bind_point = match pipelines[self.pipeline_handle].pipeline_type {
            PipelineType::Graphics => vk::PipelineBindPoint::GRAPHICS,
            PipelineType::Compute => vk::PipelineBindPoint::COMPUTE,
        };

        if bind_point != vk::PipelineBindPoint::GRAPHICS {
            unsafe {
                self.device.ash_device.cmd_bind_pipeline(
                    *command_buffer,
                    bind_point,
                    pipelines[self.pipeline_handle].handle,
                );
            }

            return;
        }

        let color_attachments = color_attachments
            .iter()
            .map(|image| {
                vk::RenderingAttachmentInfo::builder()
                    .image_view(match image.1 {
                        ViewType::Full() => image.0.image_view,
                        ViewType::Layer(layer) => image.0.layer_view(layer),
                    })
                    .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .load_op(image.2)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .clear_value(vk::ClearValue {
                        color: vk::ClearColorValue {
                            float32: [1.0, 1.0, 1.0, 0.0],
                        },
                    })
                    .build()
            })
            .collect::<Vec<_>>();

        let rendering_info = vk::RenderingInfo::builder()
            .view_mask(0)
            .layer_count(1)
            .color_attachments(&color_attachments)
            .depth_attachment(&if let Some(depth_attachment) = depth_attachment {
                vk::RenderingAttachmentInfo::builder()
                    .image_view(match depth_attachment.1 {
                        ViewType::Full() => depth_attachment.0.image_view,
                        ViewType::Layer(layer) => depth_attachment.0.layer_view(layer),
                    })
                    .image_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .load_op(depth_attachment.2)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .clear_value(vk::ClearValue {
                        depth_stencil: vk::ClearDepthStencilValue {
                            depth: 1.0,
                            stencil: 0,
                        },
                    })
                    .build()
            } else {
                vk::RenderingAttachmentInfo::default()
            })
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            })
            .build();

        unsafe {
            self.device
                .device()
                
                .cmd_begin_rendering(*command_buffer, &rendering_info);
            self.device.device().cmd_bind_pipeline(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipelines[self.pipeline_handle].handle,
            );

            let viewports = [vk::Viewport {
                x: 0.0,
                y: extent.height as f32,
                width: extent.width as f32,
                height: -(extent.height as f32),
                min_depth: 0.0,
                max_depth: 1.0,
            }];

            let scissors = [vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            }];

            self.device
                .device()
                .cmd_set_viewport(*command_buffer, 0, &viewports);
            self.device
                .device()
                .cmd_set_scissor(*command_buffer, 0, &scissors);
        }
    }
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
    pub fn layout_in(mut self, resource_id: TextureId) -> Self {
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

    pub fn layout_out(mut self, resource_id: TextureId) -> Self {
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

    pub fn build(self, graph: &mut RenderGraph) {
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

        if let Some(depth) = &pass.depth_attachment {
            match depth {
                DepthAttachment::GraphHandle(write) => {
                    graph.pipeline_descs[pass.pipeline_handle].depth_stencil_attachment_format =
                        graph
                            .resources
                            .texture(write.texture)
                            .texture
                            .image
                            .format()
                }
                DepthAttachment::External(image, _) => {
                    graph.pipeline_descs[pass.pipeline_handle].depth_stencil_attachment_format =
                        image.format()
                }
            }
        }

        if !self.uniforms.is_empty() {
            pass.uniform_buffer.replace(
                graph.get_or_create_buffer(
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
