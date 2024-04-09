use crate::{
    mesh::Vertex, render_graph::{RenderGraph, TextureId}, vulkan::{renderer::VulkanRenderer, PipelineDesc}
};
use ash::vk;

use imgui::{sys::cty::c_short, DrawData};

pub fn setup_gui_pass(
    graph: &mut RenderGraph,
    renderer: &VulkanRenderer,
    deferred_frame: TextureId,
    draw_data: &DrawData,
) {
    let mut verts : Vec<Vertex> = Vec::with_capacity(draw_data.total_vtx_count as usize);

    let mut indices : Vec<c_short> = Vec::with_capacity(draw_data.total_vtx_count as usize);

    for draw_list in draw_data.draw_lists() {
        verts.push(Vertex::from(draw_list.vtx_buffer()[0]));
    };
    for draw_list in draw_data.draw_lists() {
        indices.push(draw_list.idx_buffer()[0] as c_short);
    };
    
    graph
        .add_pass_from_desc(
            "gui_pass",
            PipelineDesc::builder()
                .vertex_path("assets/shaders/imgui.vert")
                .fragment_path("assets/shaders/imgui.frag")
                .default_primitive_vertex_attributes()
                .default_primitive_vertex_bindings(),
        )
        .load_write(deferred_frame)
        .external_depth_attachment(renderer.depth_image.clone(), vk::AttachmentLoadOp::CLEAR)
        .record_render(move |_device, _command_buffer, _renderer, _pass, _resources| {
            

        })
        .build(graph);
}
