use crate::{vulkan::{Device, PipelineDesc}, graph::{Graph, TextureId}};

pub fn setup_ssao_pass(
    device: &Device,
    graph: &mut Graph,
    gbuffer_position: TextureId,
    gbuffer_normal: TextureId,
    ssao_output: TextureId,
    enabled: bool,
) {
    let radius_bias = glam::Vec4::new(0.1, 0.0, 0.0, 0.0);

    graph
        .add_pass_from_desc(
            "ssao_pass",
            PipelineDesc::builder()
                .vertex_path("assets/shaders/fullscreen.vert")
                .fragment_path("assets/shaders/ssao.frag"),
        )
        .read(gbuffer_position)
        .read(gbuffer_normal)
        .write(ssao_output)
        .uniforms("settings_ubo", &(radius_bias))
        .record_render(
            move |device, command_buffer, _renderer, _pass, _resources| unsafe {
                // Todo: This is a hack to get around the fact that we can't properly disable a pass
                if enabled {
                    device.device().cmd_draw(*command_buffer, 3, 1, 0, 0);
                }
            },
        )
        .build(device, graph);

    // It is common to also have a blur pass for SSAO which can be added here.
    // The SSAO effect looks decent without it, but it should be added here in the future.
}
