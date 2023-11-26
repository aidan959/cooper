use ash::vk;

use crate::{graph::{Graph, TextureId}, vulkan::{Device, renderer::VulkanRenderer, ImageDesc}, ViewUniformData, Camera};

pub mod atmosphere;
pub mod deferred;
pub mod forward;
pub mod gbuffer;
pub mod ibl;
pub mod present;
pub mod shadow;
pub mod ssao;

pub fn create_gbuffer_textures(
    graph: &mut Graph,
    device: &Device,
    width: u32,
    height: u32,
) -> (TextureId, TextureId, TextureId, TextureId) {
    (
        graph.create_texture(
            "gbuffer_position",
            device,
            ImageDesc::new_2d(width, height, vk::Format::R32G32B32A32_SFLOAT),
        ),
        graph.create_texture(
            "gbuffer_normal",
            device,
            ImageDesc::new_2d(width, height, vk::Format::R32G32B32A32_SFLOAT),
        ),
        graph.create_texture(
            "gbuffer_albedo",
            device,
            ImageDesc::new_2d(width, height, vk::Format::R8G8B8A8_UNORM),
        ),
        graph.create_texture(
            "gbuffer_pbr",
            device,
            ImageDesc::new_2d(width, height, vk::Format::R32G32B32A32_SFLOAT),
        ),
    )
}

pub fn create_shadowmap_texture(graph: &mut Graph, device: &Device) -> TextureId {
    graph.create_texture(
        "shadow_map",
        device,
        ImageDesc::new_2d_array(4096, 4096, 4, vk::Format::D32_SFLOAT)
            .aspect(vk::ImageAspectFlags::DEPTH)
            .usage(
                vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
                    | vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::SAMPLED,
            ),
    )
}

pub fn build_render_graph(
    graph: &mut Graph,
    device: &Device,
    base: &VulkanRenderer,
    view_data: &ViewUniformData,
    camera: &Camera,
) {

    let width = base.surface_resolution.width;
    let height = base.surface_resolution.height;

    let (gbuffer_position, gbuffer_normal, gbuffer_albedo, gbuffer_pbr) =
        create_gbuffer_textures(graph, device, width, height);

    let shadow_map = create_shadowmap_texture(graph, device);

    let deferred_output = graph.create_texture(
        "deferred_output",
        device,
        ImageDesc::new_2d(width, height, vk::Format::R32G32B32A32_SFLOAT),
    );

    let ssao_output = graph.create_texture(
        "ssao_output",
        device,
        ImageDesc::new_2d(width, height, vk::Format::R16_UNORM),
    );

    let (cascade_matrices, cascade_depths) = shadow::setup_shadow_pass(
        device,
        graph,
        shadow_map,
        view_data.sun_dir,
        camera,
        view_data.shadows_enabled == 1,
    );


    gbuffer::setup_gbuffer_pass(
        device,
        graph,
        base,
        gbuffer_position,
        gbuffer_normal,
        gbuffer_albedo,
        gbuffer_pbr,
    );

    let (environment_map, irradiance_map, specular_map, brdf_lut) =
        ibl::setup_cubemap_pass(device, graph, &base);

    ssao::setup_ssao_pass(
        device,
        graph,
        gbuffer_position,
        gbuffer_normal,
        ssao_output,
        view_data.ssao_enabled == 1,
    );

    deferred::setup_deferred_pass(
        device,
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

    atmosphere::setup_atmosphere_pass(
        device,
        graph,
        base,
        deferred_output,
        environment_map,
        camera,
        true,
    );
    present::setup_present_pass(device, graph, deferred_output);
}

pub fn build_minimal_forward_render_graph(
    graph: &mut Graph,
    device: &Device,
    renderer: &VulkanRenderer,
    view_data: &ViewUniformData,
    camera: &Camera,
) {

    let width = renderer.surface_resolution.width;
    let height = renderer.surface_resolution.height;
    let rgba32_fmt = vk::Format::R32G32B32A32_SFLOAT;

    // Forward & deferred output textures
    let forward_output = graph.create_texture(
        "forward_output",
        device,
        ImageDesc::new_2d(width, height, rgba32_fmt),
    );
    let shadow_map = create_shadowmap_texture(graph, device);

    let (cascade_matrices, cascade_depths) = shadow::setup_shadow_pass(
        device,
        graph,
        shadow_map,
        view_data.sun_dir,
        camera,
        view_data.shadows_enabled == 1,
    );

    forward::setup_forward_pass(
        device,
        graph,
        renderer,
        forward_output,
        shadow_map,
        (cascade_matrices, cascade_depths),
    );
    let deferred_output = graph.create_texture(
        "deferred_output",
        device,
        ImageDesc::new_2d(width, height, vk::Format::R32G32B32A32_SFLOAT),
    );
    let (environment_map, _irradiance_map, _specular_map, _brdf_lut) =
        ibl::setup_cubemap_pass(device, graph, renderer);
    atmosphere::setup_atmosphere_pass(
        device,
        graph,
        renderer,
        deferred_output,
        environment_map,
        camera,
        true,
    );
    present::setup_present_pass(device, graph, forward_output);
}
pub fn viewport(width: u32, height: u32) -> vk::Viewport {
    vk::Viewport {
        x: 0.0,
        y: height as f32,
        width: width as f32,
        height: -(height as f32),
        min_depth: 0.0,
        max_depth: 1.0,
    }
}
