use std::collections::HashMap;
use std::sync::Arc;

use ash::vk;

use crate::render_graph::{
    Attachment, BufferId, DepthAttachment, GraphBuffer, GraphResources, GraphTexture, PipelineId,
    Resource, TextureCopy, TextureResourceType, UniformData, ViewType,
};

use super::descriptor::{DescriptorIdentifier, DescriptorSet};
use super::renderer::VulkanRenderer;
use super::{Device, Image, Pipeline, PipelineType};

pub struct RenderPass {
    pub pipeline_handle: PipelineId,
    pub render_func: Option<
        Box<dyn Fn(&Device, &vk::CommandBuffer, &VulkanRenderer, &RenderPass, &GraphResources)>,
    >,
    pub writes: Vec<Attachment>,
    pub depth_attachment: Option<DepthAttachment>,
    pub presentation_pass: bool,
    pub reads: Vec<Resource>,
    pub name: String,
    pub uniforms: HashMap<String, (String, UniformData)>,
    pub read_resources_descriptor_set: Option<DescriptorSet>,
    pub uniform_descriptor_set: Option<DescriptorSet>,
    pub copy_command: Option<TextureCopy>,
    pub uniform_buffer: Option<BufferId>,
    pub extra_barriers: Option<Vec<(BufferId, vk_sync::AccessType)>>,
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
        extra_barriers: Option<Vec<(BufferId, vk_sync::AccessType)>>,
        device: Arc<Device>,
    ) -> RenderPass {
        RenderPass {
            pipeline_handle,
            render_func,
            reads: Vec::new(),
            writes: Vec::new(),
            depth_attachment,
            presentation_pass,
            read_resources_descriptor_set: None,
            name,
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
        _tlas: vk::AccelerationStructureKHR,
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