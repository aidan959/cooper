use crate::{
    render_graph::{RenderGraph, TextureId},
    vulkan::PipelineDesc,
};

#[allow(clippy::too_many_arguments)]
pub fn setup_deferred_pass(
    graph: &mut RenderGraph,
    gbuffer_position: TextureId,
    gbuffer_normal: TextureId,
    gbuffer_albedo: TextureId,
    gbuffer_pbr: TextureId,
    shadow_map: TextureId,
    ssao_output: TextureId,
    irradiance_map: TextureId,
    specular_map: TextureId,
    brdf_lut: TextureId,
    cascade_data: ([glam::Mat4; 4], [f32; 4]),
    deferred_output: TextureId,
) {
    graph
        .add_pass_from_desc(
            "deferred_pass",
            PipelineDesc::builder()
                .vertex_path("assets/shaders/fullscreen.vert")
                .fragment_path("assets/shaders/deferred.frag"),
        )
        .read(gbuffer_position)
        .read(gbuffer_normal)
        .read(gbuffer_albedo)
        .read(gbuffer_pbr)
        .read(shadow_map)
        .read(ssao_output)
        .read(irradiance_map)
        .read(specular_map)
        .read(brdf_lut)
        .write(deferred_output)
        .uniforms("shadowmapParams", &(cascade_data))
        .record_render(
            move |device, command_buffer, _renderer, _pass, _resources| unsafe {
                device.device().cmd_draw(*command_buffer, 3, 1, 0, 0);
            },
        )
        .build(graph);
}
