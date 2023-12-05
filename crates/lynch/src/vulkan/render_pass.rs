use ash::vk;


pub struct RenderPass {
    pub name: String,
    pub pipeline_handle: PipelineId,
    pub render_func: Option<
        Box<dyn Fn(&Device, &vk::CommandBuffer, &VulkanRenderer, &RenderPass, &GraphResources)>,
    >,
    pub reads: Vec<Resource>,

    pub writes: Vec<Attachment>,
}
impl RenderPass {
    pub fn new(
        name: String,
        pipeline_handle: PipelineId,
        render_func: Option<
            Box<dyn Fn(&Device, &vk::CommandBuffer, &VulkanRenderer, &RenderPass, &GraphResources)>,
        >,
    ) -> RenderPass {
        RenderPass {
            name,
            pipeline_handle,
            render_func,
            reads: Vec::new(),
            writes: Vec::new(),
        }
    }
    pub fn prepare_render(
        &self,
        device: &Device,
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
                device.ash_device.cmd_bind_pipeline(
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
            //.flags(vk::RenderingFlags::RESUMING)
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
            device
                .device()
                .cmd_begin_rendering(*command_buffer, &rendering_info);

            device
                .device()
                .cmd_bind_pipeline(
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

            device
                .device()
                .cmd_set_viewport(*command_buffer, 0, &viewports);
            device
                .device()
                .cmd_set_scissor(*command_buffer, 0, &scissors);
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
    pub fn update_uniform_buffer_memory(&mut self, buffers: &mut [GraphBuffer]) {
        if let Some(buffer_id) = self.uniform_buffer {
            buffers[buffer_id]
                .buffer
                .update_memory(&self.uniforms.values().next().unwrap().1.data)
        }
    }

}


