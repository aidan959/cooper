use std::sync::Arc;

use ash::vk;
use glam::{Mat4, Vec3};

use crate::{vulkan::{Device, renderer::VulkanRenderer, PipelineDesc, ImageDesc, ImageCopyDescBuilder}, render_graph::{RenderGraph, TextureId}};

use super::viewport;
const PASS_AMOUNT : u8 = 6;
pub fn setup_cubemap_pass(
    device: Arc<Device>,
    graph: &mut RenderGraph,
    renderer: &VulkanRenderer,
) -> (
    TextureId,
    TextureId,
    TextureId,
    TextureId,
) {

    let (mip0_size, num_mips) = (512, 8);

    let rgba32_fmt = vk::Format::R32G32B32A32_SFLOAT;

    let environment_map = graph.create_texture(
        "environment_map",
        device.clone(),
        ImageDesc::new_cubemap(mip0_size, mip0_size, rgba32_fmt).mip_levels(num_mips),
    );

    let irradiance_map = graph.create_texture(
        "irradiance_map",
        device.clone(),
        ImageDesc::new_cubemap(mip0_size, mip0_size, rgba32_fmt),
    );

    let specular_map = graph.create_texture(
        "specular_map",
        device.clone(),
        ImageDesc::new_cubemap(mip0_size, mip0_size, rgba32_fmt).mip_levels(num_mips),
    );

    let offscreen = graph.create_texture(
        "cubemap_offscreen",
        device.clone(),
        ImageDesc::new_2d(mip0_size, mip0_size, rgba32_fmt),
    );

    let brdf_lut = graph.create_texture(
        "brdf_lut",
        device.clone(),
        ImageDesc::new_2d(512, 512, vk::Format::R16G16_SFLOAT),
    );

    let projection = Mat4::perspective_rh(90.0_f32.to_radians(), 1.0, 0.01, f32::INFINITY);
    let view_matrices = [
        Mat4::look_at_rh(Vec3::ZERO, Vec3::X, -Vec3::Y),
        Mat4::look_at_rh(Vec3::ZERO, -Vec3::X, -Vec3::Y),
        Mat4::look_at_rh(Vec3::ZERO, -Vec3::Y, -Vec3::Z),
        Mat4::look_at_rh(Vec3::ZERO, Vec3::Y, Vec3::Z),
        Mat4::look_at_rh(Vec3::ZERO, Vec3::Z, -Vec3::Y),
        Mat4::look_at_rh(Vec3::ZERO, -Vec3::Z, -Vec3::Y),
    ];
    if !renderer.internal_renderer.need_environment_map_update {
        return (environment_map, irradiance_map, specular_map, brdf_lut);
    }

    for mip in 0..num_mips {
        let size = (mip0_size as f32 * 0.5f32.powf(mip as f32)) as u32;

        for layer in 0..PASS_AMOUNT {
            graph
                .add_pass_from_desc(
                    format!("cubemap_pass_layer_{layer}_mip_{mip}").as_str(),
                    PipelineDesc::builder()
                        .vertex_path("assets/shaders/fullscreen.vert")
                        .fragment_path("assets/shaders/cubemap.frag"),
                )
                .write(offscreen)
                .uniforms("params", &(view_matrices[layer as usize], projection))
                .record_render(move |device, cb, _renderer, _pass, _resources| unsafe {
                    let viewport = [viewport(size, size)];
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

    for layer in 0..PASS_AMOUNT {
        graph
            .add_pass_from_desc(
                format!("irradiance_filter_pass_layer_{layer}").as_str(),
                PipelineDesc::builder()
                    .vertex_path("assets/shaders/fullscreen.vert")
                    .fragment_path("assets/shaders/irradiance_filter.frag"),
            )
            .read(environment_map)
            .write_layer(irradiance_map, layer as u32)
            .uniforms("params", &(view_matrices[layer as usize], projection))
            .record_render(move |device, cb, _renderer, _pass, _resources| unsafe {
                let viewport = [viewport(mip0_size, mip0_size)];
                device.device().cmd_set_viewport(*cb, 0, &viewport);
                device.device().cmd_draw(*cb, 3, 1, 0, 0);
            })
            .build(graph);
    }

    for mip in 0..num_mips {
        let mip_size = (mip0_size as f32 * 0.5f32.powf(mip as f32)) as u32;

        for layer in 0..PASS_AMOUNT {
            graph
                .add_pass_from_desc(
                    format!("specular_filter_pass_layer_{layer}_mip_{mip}").as_str(),
                    PipelineDesc::builder()
                        .vertex_path("assets/shaders/fullscreen_with_pushconst.vert")
                        .fragment_path("assets/shaders/specular_filter.frag"),
                )
                .read(environment_map)
                .write(offscreen)
                .uniforms("params", &(view_matrices[layer as usize], projection))
                .record_render(move |device, cb, _renderer, pass, resources| unsafe {
                    let viewport = [viewport(mip_size, mip_size)];
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
        .write(brdf_lut)
        .record_render(move |device, command_buffer, _, _, _| unsafe {
            device.device().cmd_draw(*command_buffer, 3, 1, 0, 0);
        })
        .build(graph);

    (environment_map, irradiance_map, specular_map, brdf_lut)
}
