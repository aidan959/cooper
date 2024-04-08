use std::sync::Arc;

use ash::vk;

use crate::{
    render_graph::{RenderGraph, TextureId},
    vulkan::{renderer::VulkanRenderer, Device, ImageDesc},
    Camera, ViewUniformData,
};

use self::{
    atmosphere::setup_atmosphere_pass, deferred::setup_deferred_pass, gbuffer::setup_gbuffer_pass, gui::setup_gui_pass, irradiancebasedlighting::setup_cubemap_pass, present::setup_present_pass, ssao::setup_ssao_pass
};

pub mod atmosphere;
pub mod deferred;
pub mod forward;
pub mod gbuffer;
pub mod irradiancebasedlighting;
pub mod present;
pub mod shadow;
pub mod ssao;
pub mod gui;
use phf::phf_ordered_map;

static GBUFFER_MAP: phf::OrderedMap<&'static str, vk::Format> = phf_ordered_map! {
    "gbuffer_position" => vk::Format::R32G32B32A32_SFLOAT,
    "gbuffer_normal"   => vk::Format::R32G32B32A32_SFLOAT,
    "gbuffer_albedo"   => vk::Format::R8G8B8A8_UNORM,
    "gbuffer_pbr"      => vk::Format::R32G32B32A32_SFLOAT,
};
pub fn create_gbuffer_textures(
    graph: &mut RenderGraph,
    device: Arc<Device>,
    width: u32,
    height: u32,
) -> (TextureId, TextureId, TextureId, TextureId) {
    let textures: Vec<TextureId> = GBUFFER_MAP
        .into_iter()
        .map(|(texture_name, format)| -> usize {
            graph.get_or_create_texture(
                texture_name,
                device.clone(),
                ImageDesc::new_2d(width, height, *format),
            )
        })
        .collect();
    (textures[0], textures[1], textures[2], textures[3])
}
pub fn create_shadowmap_texture(graph: &mut RenderGraph, device: Arc<Device>) -> TextureId {
    let image_usage_flags = vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
        | vk::ImageUsageFlags::TRANSFER_DST
        | vk::ImageUsageFlags::SAMPLED;
    graph.get_or_create_texture(
        "shadow_map",
        device,
        ImageDesc::new_2d_array(4096, 4096, shadow::SHADOW_TEXTURE_CASCADE_NO as u32, vk::Format::D32_SFLOAT)
            .aspect(vk::ImageAspectFlags::DEPTH)
            .usage(image_usage_flags),
    )
}


pub fn build_render_graph(
    graph: &mut RenderGraph,
    device: Arc<Device>,
    base: &VulkanRenderer,
    view_data: &ViewUniformData,
    camera: &Camera,
) {
    let width = base.surface_resolution.width;
    let height = base.surface_resolution.height;

    let (gbuffer_position, gbuffer_normal, gbuffer_albedo, gbuffer_pbr) =
        create_gbuffer_textures(graph, device.clone(), width, height);

    let shadow_map = create_shadowmap_texture(graph, device.clone());
    let (cascade_matrices, cascade_depths) = shadow::setup_shadow_pass(
        graph,
        shadow_map,
        view_data.sun_dir,
        camera,
        view_data.shadows_enabled,
    );

    let image_desc = ImageDesc::new_2d(width, height, vk::Format::R32G32B32A32_SFLOAT);

    let deferred_output = graph.get_or_create_texture("deferred_output", device.clone(), image_desc);

    let image_desc = ImageDesc::new_2d(width, height, vk::Format::R16_UNORM);
    setup_gbuffer_pass(
        graph,
        base,
        gbuffer_position,
        gbuffer_normal,
        gbuffer_albedo,
        gbuffer_pbr,
    );
    let (environment_map, irradiance_map, specular_map, brdf_lut) =
        setup_cubemap_pass(device.clone(), graph, &base);
    let ssao_output = graph.get_or_create_texture("ssao_output", device.clone(), image_desc);
    
    setup_ssao_pass(
        graph,
        gbuffer_position,
        gbuffer_normal,
        ssao_output,
        true,
    );
    setup_deferred_pass(
        graph,
        gbuffer_position,
        gbuffer_normal,
        gbuffer_albedo,
        gbuffer_pbr,
        shadow_map,
        ssao_output,
        irradiance_map,
        specular_map,
        brdf_lut,
        (cascade_matrices, cascade_depths),
        deferred_output,
    );
    setup_atmosphere_pass(graph, base, deferred_output, environment_map, camera);
    setup_present_pass(graph, deferred_output);
}


 