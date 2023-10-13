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
    todo!();
}
