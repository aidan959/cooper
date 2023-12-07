use crate::{camera, vulkan::PipelineDesc, render_graph::{RenderGraph, TextureId}};
use glam::{Mat4, Vec3, Vec4Swizzles};

const SHADOW_MAP_CASCADE_COUNT: usize = 4;
pub fn setup_shadow_pass(
    graph: &mut RenderGraph,
    shadow_map: TextureId,
    sun_dir: glam::Vec3,
    camera: &camera::Camera,
    enabled: bool,
) -> ([glam::Mat4; SHADOW_MAP_CASCADE_COUNT], [f32; SHADOW_MAP_CASCADE_COUNT]) {


    let mut out_cascade_matrices : [Mat4; SHADOW_MAP_CASCADE_COUNT] 
        = [glam::Mat4::IDENTITY; SHADOW_MAP_CASCADE_COUNT];
    let mut out_split_depths : [f32; SHADOW_MAP_CASCADE_COUNT] 
     = [0.0; SHADOW_MAP_CASCADE_COUNT];

    if !enabled {
        return (out_cascade_matrices, out_split_depths);
    }

    let mut cascade_splits: [f32; SHADOW_MAP_CASCADE_COUNT] = [0.0; SHADOW_MAP_CASCADE_COUNT];

    let near_clip: f32 = camera.get_near_plane();
    let far_clip: f32 = camera.get_far_plane();
    let clip_range: f32 = far_clip - near_clip;

    let min_z: f32 = near_clip;
    let max_z: f32 = near_clip + clip_range;

    let range: f32 = max_z - min_z;
    let ratio: f32 = max_z / min_z;

    let cascade_split_lambda: f32 = 0.927;
    for i in 0..SHADOW_MAP_CASCADE_COUNT {
        let p: f32 = (i + 1) as f32 / SHADOW_MAP_CASCADE_COUNT as f32;
        let log: f32 = min_z * ratio.powf(p);
        let uniform: f32 = min_z + range * p;
        let d: f32 = cascade_split_lambda * (log - uniform) + uniform;
        cascade_splits[i] = (d - near_clip) / clip_range;
    }
    let mut last_split_dist = 0.0;
    for i in 0..SHADOW_MAP_CASCADE_COUNT {
        let split_dist = cascade_splits[i];

        let mut frustum_corners: [Vec3; 8] = [
            Vec3::new(-1.0, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(-1.0, -1.0, 1.0),
        ];

        let invert_cam: Mat4 = (camera.get_projection() * camera.get_view()).inverse();
        for corner in &mut frustum_corners {
            let inv_corner: glam::Vec4 = invert_cam * corner.extend(1.0);
            *corner = inv_corner.xyz() / inv_corner.w;
        }

        for i in 0..SHADOW_MAP_CASCADE_COUNT {
            let dist: Vec3 = frustum_corners[i + 4] - frustum_corners[i];
            frustum_corners[i + 4] = frustum_corners[i] + (dist * split_dist);
            frustum_corners[i] += dist * last_split_dist;
        }

        let frustum_center: Vec3 = frustum_corners.iter().sum::<Vec3>() / 8.0;

        let mut radius: f32 = 0.0;
        for corner in &frustum_corners {
            let distance: f32 = (*corner - frustum_center).length();
            radius = radius.max(distance);
        }
        radius = f32::ceil(radius * 16.0) / 16.0;

        let max_extents: Vec3 = Vec3::new(radius, radius, radius);
        let min_extents: Vec3 = -max_extents;

        let light_view_matrix: Mat4 = Mat4::look_at_rh(
            frustum_center - sun_dir * min_extents.z,
            frustum_center,
            Vec3::Y,
        );

        let light_ortho_matrix = Mat4::orthographic_rh(
            min_extents.x,
            max_extents.x,
            min_extents.y,
            max_extents.y,
            -(max_extents.z - min_extents.z),
            max_extents.z - min_extents.z,
        );

        let view_projection_matrix = light_ortho_matrix * light_view_matrix;
        out_cascade_matrices[i] = view_projection_matrix;
        out_split_depths[i] = near_clip + split_dist * clip_range;

        last_split_dist = split_dist;

        graph
            .add_pass_from_desc(
                format!("shadow_pass_{i}").as_str(),
                PipelineDesc::builder()
                    .vertex_path("assets/shaders/shadow.vert")
                    .fragment_path("assets/shaders/shadow.frag")
                    .default_primitive_vertex_bindings()
                    .default_primitive_vertex_attributes(),
            )
            .uniforms("cascade_view_projection", &view_projection_matrix)
            .depth_attachment_layer(shadow_map, i as u32 )
            .record_render(move |device, command_buffer, renderer, pass, resources| {
                // Todo: This is a hack to get around the fact that we can't properly disable a pass
                if enabled {
                    let pipeline = resources.pipeline(pass.pipeline_handle);

                    renderer.internal_renderer.draw_meshes(device, *command_buffer, pipeline.pipeline_layout);
                }
            })
            .build(graph);
    }

    (out_cascade_matrices, out_split_depths)
}
