use crate::{
    render_graph::{RenderGraph, TextureId},
    vulkan::{renderer::VulkanRenderer, PipelineDesc},
};
use ash::vk;

pub fn setup_gbuffer_pass(
    graph: &mut RenderGraph,
    renderer: &VulkanRenderer,
    gbuffer_position: TextureId,
    gbuffer_normal: TextureId,
    gbuffer_albedo: TextureId,
    gbuffer_pbr: TextureId,
) {
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
        .write(gbuffer_normal)
        .write(gbuffer_albedo)
        .write(gbuffer_pbr)
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
}
