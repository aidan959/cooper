use crate::{
    render_graph::{RenderGraph, TextureId},
    vulkan::PipelineDesc,
};

pub fn setup_present_pass(graph: &mut RenderGraph, color_output: TextureId) {

    graph
        .add_pass_from_desc(
            "present_pass",
            PipelineDesc::builder()
                .vertex_path("assets/shaders/fullscreen.vert")
                .fragment_path("assets/shaders/present.frag"),
        )
        .read(color_output)
        .presentation_pass(true)
        .record_render(
            move |device, command_buffer, _renderer, _pass, _pipeline_cache| unsafe {
                device.device().cmd_draw(*command_buffer, 3, 1, 0, 0);
            },
        )
        .build(graph);
}


pub fn build_render_graph(
    render_graph: &mut RenderGraph,
    device: Arc<Device>,
    base: &VulkanRenderer,
    view_data: &ViewUniformData,
    camera: &Camera,
) {
    let width = base.surface_resolution.width;
    let height = base.surface_resolution.height;
    let image_desc = ImageDesc::new_2d(width, height, vk::Format::R32G32B32A32_SFLOAT);

    let present_output = graph.create_texture("present_output", device.clone(), image_desc);

    setup_present_pass(graph, present_output);
}