use std::sync::Arc;

use ash::vk;
use glam::{Mat4, Vec3};

use crate::{
    render_graph::{RenderGraph, TextureId},
    vulkan::{renderer::VulkanRenderer, Device, ImageCopyDescBuilder, ImageDesc, PipelineDesc},
};

pub fn vk_viewport(width: u32, height: u32) -> vk::Viewport {
    vk::Viewport {
        x: 0.0,
        y: height as f32,
        width: width as f32,
        height: -(height as f32),
        min_depth: 0.0,
        max_depth: 1.0,
    }
}
const PASS_NO: u8 = 6;
const LUT_TEXTURE_SIZE: u32 = 512;
pub fn  setup_cubemap_pass(
    device: Arc<Device>,
    graph: &mut RenderGraph,
    renderer: &VulkanRenderer,
) -> (TextureId, TextureId, TextureId, TextureId) {
    let (mip0_size, num_mips) = (512, 8);

    let rgba32_fmt = vk::Format::R32G32B32A32_SFLOAT;

    let environment_map = graph.get_or_create_texture(
        "environment_map",
        device.clone(),
        ImageDesc::new_cubemap(mip0_size, mip0_size, rgba32_fmt).mip_levels(num_mips),
    );

    let irradiance_map = graph.get_or_create_texture(
        "irradiance_map",
        device.clone(),
        ImageDesc::new_cubemap(mip0_size, mip0_size, rgba32_fmt),
    );

    let specular_map = graph.get_or_create_texture(
        "specular_map",
        device.clone(),
        ImageDesc::new_cubemap(mip0_size, mip0_size, rgba32_fmt).mip_levels(num_mips),
    );

    let offscreen = graph.get_or_create_texture(
        "cubemap_offscreen",
        device.clone(),
        ImageDesc::new_2d(mip0_size, mip0_size, rgba32_fmt),
    );

    let brdf_lut = graph.get_or_create_texture(
        "brdf_lut",
        device.clone(),
        ImageDesc::new_2d(LUT_TEXTURE_SIZE, LUT_TEXTURE_SIZE, vk::Format::R16G16_SFLOAT),
    );


    if !renderer.internal_renderer.recreate_environment {
        return (environment_map, irradiance_map, specular_map, brdf_lut);
    }

    let projection = Mat4::perspective_rh(90.0_f32.to_radians(), 1.0, 0.01, f32::INFINITY);
    let view_matrices = [
        Mat4::look_at_rh(Vec3::ZERO, Vec3::X, -Vec3::Y),
        Mat4::look_at_rh(Vec3::ZERO, -Vec3::X, -Vec3::Y),
        Mat4::look_at_rh(Vec3::ZERO, -Vec3::Y, -Vec3::Z),
        Mat4::look_at_rh(Vec3::ZERO, Vec3::Y, Vec3::Z),
        Mat4::look_at_rh(Vec3::ZERO, Vec3::Z, -Vec3::Y),
        Mat4::look_at_rh(Vec3::ZERO, -Vec3::Z, -Vec3::Y),
    ];

    print!("Updating environment map\n");
    for mip in 0..num_mips {
        let size = (mip0_size as f32 * 0.5f32.powf(mip as f32)) as u32;

        for layer in 0..PASS_NO {
            graph
                .add_pass_from_desc(
                    format!("cubemap_pass_layer_{layer}_mip_{mip}").as_str(),
                    PipelineDesc::builder()
                        .vertex_path("assets/shaders/fullscreen.vert")
                        .fragment_path("assets/shaders/cubemap.frag"),
                )
                .layout_out(offscreen)
                .uniforms("params", &(view_matrices[layer as usize], projection))
                .record_render(move |device, cb, _renderer, _pass, _resources| unsafe {
                    let viewport = [vk_viewport(size, size)];
                    device.device().cmd_set_viewport(*cb, 0, &viewport);
                    device.device().cmd_draw(*cb, 3, 1, 0, 0);
                })
                .copy_image(
                    offscreen,
                    environment_map,
                    ImageCopyDescBuilder::new(size, size)
                        .dst_base_array_layer(layer as u32)
                        .dst_mip_level(mip)
                        .build(),
                )
                .build(graph);
        }
    }

    for layer in 0..PASS_NO {
        graph
            .add_pass_from_desc(
                format!("irradiance_filter_pass_layer_{layer}").as_str(),
                PipelineDesc::builder()
                    .vertex_path("assets/shaders/fullscreen.vert")
                    .fragment_path("assets/shaders/irradiance_filter.frag"),
            )
            .layout_in(environment_map)
            .write_layer(irradiance_map, layer as u32)
            .uniforms("params", &(view_matrices[layer as usize], projection))
            .record_render(move |device, cb, _, _, _| unsafe {
                let viewport = [vk_viewport(mip0_size, mip0_size)];
                device.device().cmd_set_viewport(*cb, 0, &viewport);
                device.device().cmd_draw(*cb, 3, 1, 0, 0);
            })
            .build(graph);
    }

    for mip in 0..num_mips {
        let mip_size = (mip0_size as f32 * 0.5f32.powf(mip as f32)) as u32;

        for layer in 0..PASS_NO {
            graph
                .add_pass_from_desc(
                    format!("specular_filter_pass_layer_{layer}_mip_{mip}").as_str(),
                    PipelineDesc::builder()
                        .vertex_path("assets/shaders/fullscreen_with_pushconst.vert")
                        .fragment_path("assets/shaders/specular_filter.frag"),
                )
                .layout_in(environment_map)
                .layout_out(offscreen)
                .uniforms("params", &(view_matrices[layer as usize], projection))
                .record_render(move |device, cb, _, pass, resources| unsafe {
                    let viewport = [vk_viewport(mip_size, mip_size)];
                    device.device().cmd_set_viewport(*cb, 0, &viewport);

                    let roughness = mip as f32 / (num_mips - 1) as f32;

                    device.cmd_push_constants(
                        *cb,
                        resources.pipeline(pass.pipeline_handle).pipeline_layout,
                        roughness,
                    );

                    device.device().cmd_draw(*cb, 3, 1, 0, 0);
                })
                .copy_image(
                    offscreen,
                    specular_map,
                    ImageCopyDescBuilder::new(mip_size, mip_size)
                        .dst_base_array_layer(layer as u32)
                        .dst_mip_level(mip)
                        .build(),
                )
                .build(graph);
        }
    }

    graph
        .add_pass_from_desc(
            "brdf_lut_pass",
            PipelineDesc::builder()
                .vertex_path("assets/shaders/fullscreen.vert")
                .fragment_path("assets/shaders/brdf_lut.frag"),
        )
        .layout_out(brdf_lut)
        .record_render(move |device, command_buffer, _, _, _| unsafe {
            device.device().cmd_draw(*command_buffer, 3, 1, 0, 0);
        })
        .build(graph);

    (environment_map, irradiance_map, specular_map, brdf_lut)
}
