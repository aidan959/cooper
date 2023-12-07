
use std::sync::Arc;

use ash::vk;
use glam::{Vec2, Vec3, Vec4, Mat4};
use memoffset::offset_of;

use crate::{vulkan::{Buffer, Device, ImageDesc}, Texture}; 

pub const DEFAULT_TEXTURE_MAP: u32 = u32::MAX;

#[derive(Copy, Clone)]
pub enum MaterialType {
    Lambertian = 0,
    Metal = 1,
    Dielectric = 2,
    DiffuseLight = 3,
}

pub struct Material {
    pub diffuse_map: u32,
    pub normal_map: u32,
    pub metallic_roughness_map: u32,
    pub occlusion_map: u32,
    pub base_color_factor: Vec4,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub material_type: MaterialType, 
    pub material_property: f32,
}

pub struct Mesh {
    pub primitive: Primitive,
    pub material: Material,
    pub gpu_mesh: u32,
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub textures: Vec<Texture>,
    pub transforms: Vec<Mat4>,
}

pub fn load_node(
    device: Arc<Device>,
    node: &gltf::Node,
    model: &mut Model,
    buffers: &[gltf::buffer::Data],
    parent_transform: Mat4,
    path: &str,
) {
    let node_transform =
        parent_transform * glam::Mat4::from_cols_array_2d(&node.transform().matrix());

    for child in node.children() {
        load_node(device.clone(), &child, model, buffers, node_transform, path);
    }

    if let Some(mesh) = node.mesh() {
        let primitives = mesh.primitives();

        for primitive in primitives {
            let reader = primitive.reader(|i| Some(&buffers[i.index()]));

            let indices: Vec<_> = reader.read_indices().unwrap().into_u32().collect();
            let positions: Vec<_> = reader.read_positions().unwrap().map(Vec3::from).collect();
            let normals: Vec<_> = reader.read_normals().unwrap().map(Vec3::from).collect();
            let tex_coords = if let Some(tex_coords) = reader.read_tex_coords(0) {
                tex_coords.into_f32().map(Vec2::from).collect()
            } else {
                vec![Vec2::new(0.0, 0.0); positions.len()]
            };

            let tangents = if let Some(tangents) = reader.read_tangents() {
                tangents.map(Vec4::from).collect()
            } else {
                vec![Vec4::new(0.0, 0.0, 0.0, 0.0); positions.len()]
            };

            let colors: Vec<_> = if let Some(colors) = reader.read_colors(0) {
                colors.into_rgba_f32().map(Vec4::from).collect()
            } else {
                vec![Vec4::new(1.0, 1.0, 1.0, 1.0); positions.len()]
            };

            let mut vertices: Vec<Vertex> = vec![];

            for (i, _) in positions.iter().enumerate() {
                vertices.push(Vertex {
                    pos: positions[i].extend(0.0),
                    normal: normals[i].extend(0.0),
                    uv: tex_coords[i],
                    tangent: tangents[i],
                    color: colors[i],
                });
            }

            let material = primitive.material();
            let pbr = material.pbr_metallic_roughness();

            let diffuse_index = pbr
                .base_color_texture()
                .map_or(DEFAULT_TEXTURE_MAP, |texture| {
                    texture.texture().index() as u32
                });

            let normal_index = material
                .normal_texture()
                .map_or(DEFAULT_TEXTURE_MAP, |texture| {
                    texture.texture().index() as u32
                });

            let metallic_roughness_index = pbr
                .metallic_roughness_texture()
                .map_or(DEFAULT_TEXTURE_MAP, |texture| {
                    texture.texture().index() as u32
                });

            let occlusion_index = material
                .occlusion_texture()
                .map_or(DEFAULT_TEXTURE_MAP, |texture| {
                    texture.texture().index() as u32
                });

            let base_color_factor = pbr.base_color_factor();
            let metallic_factor = pbr.metallic_factor();
            let roughness_factor = pbr.roughness_factor();

            model.meshes.push(Mesh {
                primitive: Primitive::new(device.clone(), indices, vertices),
                material: Material {
                    diffuse_map: diffuse_index,
                    normal_map: normal_index,
                    metallic_roughness_map: metallic_roughness_index,
                    occlusion_map: occlusion_index,
                    base_color_factor: Vec4::from(base_color_factor),
                    metallic_factor,
                    roughness_factor,
                    material_type: MaterialType::Lambertian,
                    material_property: 0.0,
                },
                gpu_mesh: 0,
            });

            model
                .meshes
                .last_mut()
                .unwrap()
                .primitive
                .vertex_buffer
                .set_debug_name(format!("vertex_buffer: {}", path).as_str());
            model
                .meshes
                .last_mut()
                .unwrap()
                .primitive
                .index_buffer
                .set_debug_name(format!("index_buffer: {}", path).as_str());

            model.transforms.push(node_transform);
        }
    }
}

pub fn load_gltf(device: Arc<Device>, path: &str) -> Model {
    let (gltf, buffers, mut images) = match gltf::import(path) {
        Ok(result) => result,
        Err(err) => panic!("Loading model {} failed with error: {}", path, err),
    };

    let mut model = Model {
        meshes: vec![],
        transforms: vec![],
        textures: vec![],
    };

    for image in &mut images {
        // Convert images from rgb8 to rgba8
        if image.format == gltf::image::Format::R8G8B8 {
            let dynamic_image = image::DynamicImage::ImageRgb8(
                image::RgbImage::from_raw(
                    image.width,
                    image.height,
                    std::mem::take(&mut image.pixels),
                )
                .unwrap(),
            );

            let rgba_image = dynamic_image.to_rgba();
            image.format = gltf::image::Format::R8G8B8A8;
            image.pixels = rgba_image.into_raw();
        }

        if image.format != gltf::image::Format::R8G8B8A8 {
            panic!("Unsupported image format!");
        }

        let texture = Texture::create(
            device.clone(),
            Some(&image.pixels),
            ImageDesc::new_2d(image.width, image.height, vk::Format::R8G8B8A8_UNORM),
            path,
        );

        model.textures.push(texture);
    }

    for scene in gltf.scenes() {
        for node in scene.nodes() {
            load_node(device.clone(), &node, &mut model, &buffers, Mat4::IDENTITY, path);
        }
    }

    model
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub pos: Vec4,
    pub normal: Vec4,
    pub uv: Vec2,
    pub color: Vec4,
    pub tangent: Vec4,
}
pub struct Primitive {
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
    pub indices: Vec<u32>,
    pub vertices: Vec<Vertex>,
}

impl Primitive {
    pub fn new(device: Arc<Device>, indices: Vec<u32>, vertices: Vec<Vertex>) -> Primitive {
        let index_buffer = Buffer::new(
            device.clone(),
            Some(indices.as_slice()),
            std::mem::size_of_val(&*indices) as u64,
            vk::BufferUsageFlags::INDEX_BUFFER
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
                | vk::BufferUsageFlags::STORAGE_BUFFER,
            gpu_allocator::MemoryLocation::GpuOnly,
            Some(String::from("index_buffer"))
        );

        let vertex_buffer = Buffer::new(
            device,
            Some(vertices.as_slice()),
            std::mem::size_of_val(&*vertices) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
                | vk::BufferUsageFlags::STORAGE_BUFFER,
            gpu_allocator::MemoryLocation::GpuOnly,
            Some(String::from("vertex_buffer"))
        );

        Primitive {
            index_buffer,
            vertex_buffer,
            indices,
            vertices,
        }
    }

    pub fn get_vertex_input_binding_descriptions() -> Vec<vk::VertexInputBindingDescription> {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
        .to_vec()
    }

    pub fn get_vertex_input_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, normal) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, uv) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 3,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 4,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, tangent) as u32,
            },
        ]
        .to_vec()
    }

    pub fn get_vertex_input_create_info() -> vk::PipelineVertexInputStateCreateInfo {
        let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }];
        let vertex_input_attribute_descriptions = [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32,
            },/*
            vk::VertexInputAttributeDescription {
                location: 4,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, tangent) as u32,
            }, */
        ];

        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
            .vertex_binding_descriptions(&vertex_input_binding_descriptions)
            .build();

        vertex_input_state_info
    }
}
