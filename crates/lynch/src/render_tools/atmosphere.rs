use ash::vk;
use glam::{Mat4, Vec3};

use crate::{
    render_graph::{RenderGraph, TextureId},
    vulkan::{renderer::VulkanRenderer, PipelineDesc},
};

pub fn setup_atmosphere_pass(
    graph: &mut RenderGraph,
    base: &VulkanRenderer,
    atmosphere_output: TextureId,
    environment_map: TextureId,
    camera: &crate::camera::Camera,
    enabled: bool,
) {
    let projection = camera.get_projection();
    let world = Mat4::from_scale(Vec3::splat(10000.0));

    graph
        .add_pass_from_desc(
            "atmosphere_pass",
            PipelineDesc::builder()
                .vertex_path("assets/shaders/atmosphere.vert")
                .fragment_path("assets/shaders/atmosphere.frag")
                .default_primitive_vertex_bindings()
                .default_primitive_vertex_attributes(),
        )
        .load_write(atmosphere_output)
        .layout_in(environment_map)
        .uniforms("ubo_constants", &(projection, world))
        .external_depth_attachment(base.depth_image.clone(), vk::AttachmentLoadOp::LOAD)
        .record_render(
            move |device, command_buffer, _renderer, _pass, _resources| unsafe {
                if enabled && _renderer.internal_renderer.instances.len() > 0 { // we should not rely on an instance existing for this
                    device.device().cmd_bind_vertex_buffers(
                        *command_buffer,
                        0,
                        &[_renderer.internal_renderer.instances[0].model.meshes[0]
                            .primitive
                            .vertex_buffer
                            .buffer],
                        &[0],
                    );
                    device.device().cmd_bind_index_buffer(
                        *command_buffer,
                        _renderer.internal_renderer.instances[0].model.meshes[0]
                            .primitive
                            .index_buffer
                            .buffer,
                        0,
                        vk::IndexType::UINT32,
                    );
                    device.device().cmd_draw_indexed(
                        *command_buffer,
                        _renderer.internal_renderer.instances[0].model.meshes[0]
                            .primitive
                            .indices
                            .len() as u32,
                        1,
                        0,
                        0,
                        1,
                    );
                }
            },
        )
        .build(graph);
}
