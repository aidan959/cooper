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


