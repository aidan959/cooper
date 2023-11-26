use ash::vk;

use crate::{vulkan::{Device, renderer::VulkanRenderer, PipelineDesc}, graph::{Graph, TextureId}};

#[allow(dead_code)]
struct PushConstants {
    world: glam::Mat4,
    color: glam::Vec4,
    mesh_index: u32,
    pad: [u32; 3],
}

pub fn setup_forward_pass(
    device: &Device,
    graph: &mut Graph,
    base: &VulkanRenderer,
    forward_output: TextureId,
    shadow_map: TextureId,
    cascade_data: ([glam::Mat4; 4], [f32; 4]),
) {

    graph
        .add_pass_from_desc(
            "forward_pass",
            PipelineDesc::builder()
                .vertex_path("assets/shaders/forward.vert")
                .fragment_path("assets/shaders/forward.frag")
                .default_primitive_vertex_bindings()
                .default_primitive_vertex_attributes(),
        )
        .read(shadow_map)
        .write(forward_output)
        .uniforms("shadowmapParams", &(cascade_data))
        .external_depth_attachment(base.depth_image.clone(), vk::AttachmentLoadOp::CLEAR) // Todo: create own Depth image
        .record_render(move |device, command_buffer, renderer, pass, resources| {
            let pipeline = resources.pipeline(pass.pipeline_handle);

            renderer.internal_renderer.draw_meshes(device, *command_buffer, pipeline.pipeline_layout);
        })
        .build(device, graph);
}
