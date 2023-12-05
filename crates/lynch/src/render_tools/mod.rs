use crate::{
    render_graph::{RenderGraph, TextureId},
    vulkan::PipelineDesc,
};

static GBUFFER_MAP: phf::OrderedMap<&'static str, vk::Format> = phf_ordered_map! {
    "gbuffer_position" => vk::Format::R32G32B32A32_SFLOAT,
    "gbuffer_albedo"   => vk::Format::R8G8B8A8_UNORM,
};
pub fn create_gbuffer_textures(
    graph: &mut RenderGraph,
    device: Arc<Device>,
    width: u32,
    height: u32,
) -> (TextureId, TextureId) {
    let textures: Vec<TextureId> = GBUFFER_MAP
        .into_iter()
        .map(|(texture_name, format)| -> usize {
            graph.create_texture(
                texture_name,
                device.clone(),
                ImageDesc::new_2d(width, height, *format),
            )
        })
        .collect();
    (textures[0], textures[1])
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
    let (gbuffer_position, gbuffer_albedo) =
        create_gbuffer_textures(graph, device.clone(), width, height);
    graph
        .add_pass_from_desc(
            "gbuffer_pass",
            PipelineDesc::builder()
                .vertex_path("assets/shaders/gbuffer.vert")
                .fragment_path("assets/shaders/gbuffer.frag")
                .default_primitive_vertex_bindings()
                .default_primitive_vertex_attributes(),
        )
        .write(gbuffer_position)
        .write(gbuffer_albedo)
        .external_depth_attachment(renderer.depth_image.clone(), vk::AttachmentLoadOp::CLEAR)
        .record_render(move |device, command_buffer, renderer, pass, resources| {
            let pipeline = resources.pipeline(pass.pipeline_handle);

            renderer.internal_renderer.draw_meshes(
                device,
                *command_buffer,
                pipeline.pipeline_layout,
            );
        })
        .build(graph);
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