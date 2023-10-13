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