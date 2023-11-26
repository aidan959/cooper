use crate::{vulkan::{Device, PipelineDesc}, graph::{Graph, TextureId}};


pub fn setup_present_pass(
    device: &Device,
    graph: &mut Graph,
    color_output: TextureId,
) {

    let fxaa_threshold = 0.45;

    graph
        .add_pass_from_desc(
            "present_pass",
            PipelineDesc::builder()
                .vertex_path("assets/shaders/fullscreen.vert")
                .fragment_path("assets/shaders/present.frag"),
        )
        .read(color_output)
        .uniforms(
            "settings_fxaa",
            &(glam::Vec4::new(1.0, 0.0, fxaa_threshold, 0.0)),
        )
        .presentation_pass(true)
        .record_render(
            move |device, command_buffer, _renderer, _pass, _pipeline_cache| unsafe {
                device.device().cmd_draw(*command_buffer, 3, 1, 0, 0);
            },
        )
        .build(device, graph);
}
