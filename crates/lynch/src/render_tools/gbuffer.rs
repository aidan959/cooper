use ash::vk;
use crate::{vulkan::{Device, renderer::VulkanRenderer, PipelineDesc}, graph::{Graph, TextureId}};
#[allow(dead_code)]
struct PushConstants {
    world: glam::Mat4,
    color: glam::Vec4,
    mesh_index: u32,
    pad: [u32; 3],
}

pub fn setup_gbuffer_pass(
    device: &Device,
    graph: &mut Graph,
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
        //.depth_attachment(depth_image)
        .external_depth_attachment(renderer.depth_image.clone(), vk::AttachmentLoadOp::CLEAR) // Todo: create own Depth image
        .record_render(move |device, command_buffer, renderer, pass, resources| {
            let pipeline = resources.pipeline(pass.pipeline_handle);

            renderer.internal_renderer.draw_meshes(device, *command_buffer, pipeline.pipeline_layout);
        })
        .build(device, graph);
}
