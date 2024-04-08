use crate::{
    render_graph::{RenderGraph, TextureId},
    vulkan::PipelineDesc,
};

const SSAO_BIAS : glam::Vec2 = glam::const_vec2!([0.35, 0.3]);
pub fn setup_ssao_pass(
    graph: &mut RenderGraph,
    gbuffer_position: TextureId,
    gbuffer_normal: TextureId,
    ssao_output: TextureId,
    enabled: bool,
) {

    graph
        .add_pass_from_desc(
            "ssao_pass",
            PipelineDesc::builder()
                .vertex_path("assets/shaders/fullscreen.vert")
                .fragment_path("assets/shaders/ssao.frag"),
        )
        .layout_in(gbuffer_position)
        .layout_in(gbuffer_normal)
        .layout_out(ssao_output)
        .uniforms("settings_ubo", &(SSAO_BIAS))
        .record_render(
            move |device, command_buffer, _renderer, _pass, _resources| unsafe {
                if enabled {
                    device.device().cmd_draw(*command_buffer, 3, 1, 0, 0);
                }
            },
        )
        .build(graph);
}
