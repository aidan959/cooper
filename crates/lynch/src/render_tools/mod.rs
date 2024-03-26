use std::sync::Arc;

use ash::vk;

use crate::{
    render_graph::{RenderGraph, TextureId},
    vulkan::{renderer::VulkanRenderer, Device, ImageDesc},
    Camera, ViewUniformData,
};
use self::{gbuffer::setup_gbuffer_pass, present::setup_present_pass};
// use self::{
//     atmosphere::setup_atmosphere_pass, deferred::setup_deferred_pass, gbuffer::setup_gbuffer_pass, gui::setup_gui_pass, irradiancebasedlighting::{setup_cubemap_pass, setup_cubemap_pass_opt}, present::setup_present_pass, ssao::setup_ssao_pass
// };

//pub mod atmosphere;
//pub mod deferred;
//pub mod forward;
pub mod gbuffer;
//pub mod irradiancebasedlighting;
pub mod present;
//pub mod shadow;
//pub mod ssao;
//pub mod gui;
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
            graph.create_texture(
                texture_name,
                device.clone(),
                ImageDesc::new_2d(width, height, *format),
            )
        })
        .collect();
    (textures[0], textures[1], textures[2], textures[3])
}
pub fn create_gbuffer_textures_opt(
    graph: &mut RenderGraph,
    device: Arc<Device>,
    width: u32,
    height: u32,
) -> (Option<TextureId>, Option<TextureId>, Option<TextureId>, Option<TextureId>) {
    let textures: Vec<TextureId> = GBUFFER_MAP
        .into_iter()
        .map(|(texture_name, format)| -> usize {
            graph.create_texture(
                texture_name,
                device.clone(),
                ImageDesc::new_2d(width, height, *format),
            )
        })
        .collect();
    (Some(textures[0]), Some(textures[1]), Some(textures[2]), Some(textures[3]))
}

pub fn create_shadowmap_texture(graph: &mut RenderGraph, device: Arc<Device>) -> TextureId {
    let image_usage_flags = vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
        | vk::ImageUsageFlags::TRANSFER_DST
        | vk::ImageUsageFlags::SAMPLED;
    graph.create_texture(
        "shadow_map",
        device,
        ImageDesc::new_2d_array(4096, 4096, 4, vk::Format::D32_SFLOAT)
            .aspect(vk::ImageAspectFlags::DEPTH)
            .usage(image_usage_flags),
    )
}
pub fn build_render_graph_gbuffer_only(
    graph: &mut RenderGraph,
    device: Arc<Device>,
    base: &VulkanRenderer,

) {
    let width = base.surface_resolution.width;
    let height = base.surface_resolution.height;

    let (gbuffer_position, gbuffer_normal, gbuffer_albedo, gbuffer_pbr) =
        create_gbuffer_textures(graph, device.clone(), width, height);


    setup_gbuffer_pass(
        graph,
        base,
        gbuffer_position,
        gbuffer_normal,
        gbuffer_albedo,
        gbuffer_pbr,
    );
    setup_present_pass(graph, gbuffer_albedo);
}
/*pub fn build_render_graph_atmosphere(
    graph: &mut RenderGraph,
    device: Arc<Device>,
    base: &VulkanRenderer,
    camera: &Camera,
) {
    let width = base.surface_resolution.width;
    let height = base.surface_resolution.height;

    let (gbuffer_position, gbuffer_normal, gbuffer_albedo, gbuffer_pbr) =
        create_gbuffer_textures(graph, device.clone(), width, height);


    let image_desc = ImageDesc::new_2d(width, height, vk::Format::R32G32B32A32_SFLOAT);

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

    setup_atmosphere_pass(graph, base, gbuffer_albedo, environment_map, camera, true);
    setup_present_pass(graph, gbuffer_albedo);
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
        view_data.shadows_enabled == 1,
    );

    let image_desc = ImageDesc::new_2d(width, height, vk::Format::R32G32B32A32_SFLOAT);

    let deferred_output = graph.create_texture("deferred_output", device.clone(), image_desc);

    let image_desc = ImageDesc::new_2d(width, height, vk::Format::R16_UNORM);
    let ssao_output = graph.create_texture("ssao_output", device.clone(), image_desc);
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
    setup_ssao_pass(
        graph,
        gbuffer_position,
        gbuffer_normal,
        ssao_output,
        view_data.ssao_enabled == 1,
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
    setup_atmosphere_pass(graph, base, deferred_output, environment_map, camera, true);
    setup_present_pass(graph, deferred_output¸);
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


pub fn build_render_graph_opt(
    graph: &mut RenderGraph,
    device: Arc<Device>,
    base: &VulkanRenderer,
    view_data: &ViewUniformData,
    camera: &Camera,
    enable_gbuffer_pass: bool,
    enable_shadow_pass: bool,
    enable_ssao_pass: bool,
    enable_deferred_pass: bool,
    enable_cubemap_pass: bool,
    enable_atmosphere_pass: bool,
    enable_present_pass: bool,
) {
    let width = base.surface_resolution.width;
    let height = base.surface_resolution.height;

    let (gbuffer_position, gbuffer_normal, gbuffer_albedo, gbuffer_pbr) = if enable_gbuffer_pass {
        create_gbuffer_textures_opt(graph, device.clone(), width, height)
    } else {
        (None, None, None, None)
    };

    let shadow_map = if enable_shadow_pass {
        Some(create_shadowmap_texture(graph, device.clone()))
    } else {
        None
    };

    let (cascade_matrices, cascade_depths) = if enable_shadow_pass {
        shadow::setup_shadow_pass(
            graph,
            shadow_map.unwrap(), // Unwrap is safe here because we check enable_shadow_pass
            view_data.sun_dir,
            camera,
            view_data.shadows_enabled == 1,
        )
    } else {
        ([glam::Mat4::default(); 4], [0.0; 4])
    };

    let deferred_output = if enable_deferred_pass  {
        let image_desc = ImageDesc::new_2d(width, height, vk::Format::R32G32B32A32_SFLOAT);
        Some(graph.create_texture("deferred_output", device.clone(), image_desc))
    } else {
        None
    };

    let ssao_output = if enable_ssao_pass {
        let image_desc = ImageDesc::new_2d(width, height, vk::Format::R16_UNORM);
        Some(graph.create_texture("ssao_output", device.clone(), image_desc))
    } else {
        None
    };

    if enable_gbuffer_pass {
        setup_gbuffer_pass(
            graph,
            base,
            gbuffer_position.unwrap(),
            gbuffer_normal.unwrap(),
            gbuffer_albedo.unwrap(),
            gbuffer_pbr.unwrap(),
        );
    }

    let (environment_map, irradiance_map, specular_map, brdf_lut) = if enable_cubemap_pass {
        setup_cubemap_pass_opt(device.clone(), graph, &base)
    } else {
        (None, None, None, None)
    };

    if enable_ssao_pass {
        setup_ssao_pass(
            graph,
            gbuffer_position.unwrap(),
            gbuffer_normal.unwrap(),
            ssao_output.unwrap(),
            view_data.ssao_enabled == 1,
        );
    }

    if enable_deferred_pass {
        setup_deferred_pass(
            graph,
            gbuffer_position.unwrap(),
            gbuffer_normal.unwrap(),
            gbuffer_albedo.unwrap(),
            gbuffer_pbr.unwrap(),
            shadow_map.unwrap(),
            ssao_output.unwrap(),
            irradiance_map.unwrap(),
            specular_map.unwrap(),
            brdf_lut.unwrap(),
            (cascade_matrices, cascade_depths),
            deferred_output.unwrap(),
        );
    }

    if enable_atmosphere_pass {
        let output = deferred_output.or(gbuffer_albedo);
        setup_atmosphere_pass(graph, base, output.unwrap(), environment_map.unwrap(), camera, true);
    }

    if enable_present_pass {
        let output = deferred_output.or(gbuffer_albedo);
        setup_present_pass(graph, output.unwrap());
    }
}*/