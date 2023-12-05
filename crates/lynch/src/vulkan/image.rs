use std::sync::Arc;

use ash::vk;
use ash::vk::{AccessFlags, ImageLayout, PipelineStageFlags};

use super::Device;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageType {
    Tex1d = 0,
    Tex1dArray = 1,
    Tex2d = 2,
    Tex2dArray = 3,
    Tex3d = 4,
    Cube = 5,
    CubeArray = 6,
}

#[derive(Copy, Clone, Debug)]
pub struct ImageDesc {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub array_layers: u32,
    pub format: vk::Format,
    pub image_type: ImageType,
    pub aspect_flags: vk::ImageAspectFlags,
    pub usage: vk::ImageUsageFlags,
    pub mip_levels: u32,
}

impl ImageDesc {
    fn common_usage_flags() -> vk::ImageUsageFlags {
        vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::SAMPLED
            | vk::ImageUsageFlags::COLOR_ATTACHMENT
    }
    pub fn new_2d(width: u32, height: u32, format: vk::Format) -> Self {
        ImageDesc {
            width,
            height,
            depth: 1,
            array_layers: 1,
            format,
            image_type: ImageType::Tex2d,
            aspect_flags: vk::ImageAspectFlags::COLOR,
            usage: Self::common_usage_flags()
                | vk::ImageUsageFlags::TRANSFER_SRC
            mip_levels: 1,
        }
    }
}


#[derive(Clone)]
pub struct Image {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub layer_views: Vec<vk::ImageView>,
    pub device_memory: vk::DeviceMemory,
    pub current_layout: vk::ImageLayout,
    pub desc: ImageDesc,
    pub debug_name: String,
    pub device: Arc<Device>,
}


impl Image {
    pub fn clean_vk_resources(&self) {
        unsafe {
            self.device.ash_device.device_wait_idle().unwrap();
            self.device
                .ash_device
                .destroy_image_view(self.image_view, None);
            self.device.ash_device.destroy_image(self.image, None);
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn create_image_view(
        device: &Device,
        image: vk::Image,
        format: vk::Format,
        aspect_flags: vk::ImageAspectFlags,
        view_type: vk::ImageViewType,
        base_array_layer: u32,
        layer_count: u32,
        mip_levels: u32,
    ) -> vk::ImageView {
        let image_aspect_flags = vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL;
        let components = match aspect_flags {
            vk::ImageAspectFlags::COLOR => vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            },
            vk::ImageAspectFlags::STENCIL | vk::ImageAspectFlags::DEPTH => {
                vk::ComponentMapping::default()
            }
            n if n == image_aspect_flags => vk::ComponentMapping::default(),
            _ => unimplemented!(),
        };

        let image_view_info = vk::ImageViewCreateInfo {
            view_type,
            format,
            components,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: aspect_flags,
                base_array_layer,
                layer_count,
                base_mip_level: 0,
                level_count: mip_levels,
            },
            image,
            ..Default::default()
        };

        unsafe {
            device
                .ash_device
                .create_image_view(&image_view_info, None)
                .unwrap()
        }
    }
    pub fn new_from_desc(device: Arc<Device>, desc: ImageDesc) -> Image {
        unsafe {
            let initial_layout = vk::ImageLayout::UNDEFINED;
            let image_create_info = vk::ImageCreateInfo {
                image_type: vk::ImageType::TYPE_2D,
                format: desc.format,
                extent: vk::Extent3D {
                    width: desc.width,
                    height: desc.height,
                    depth: 1,
                },
                mip_levels: desc.mip_levels,
                array_layers: desc.array_layers,
                samples: vk::SampleCountFlags::TYPE_1,
                tiling: vk::ImageTiling::OPTIMAL,
                usage: desc.usage,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                initial_layout,
                flags: if desc.image_type == ImageType::Cube
                    || desc.image_type == ImageType::CubeArray
                {
                    vk::ImageCreateFlags::CUBE_COMPATIBLE
                } else {
                    vk::ImageCreateFlags::empty()
                },
                ..Default::default()
            };
            let image = device
                .ash_device
                .create_image(&image_create_info, None)
                .expect("Unable to create image");

            // Memory allocation
            let image_memory_req = device.ash_device.get_image_memory_requirements(image);
            let image_memory_index = device
                .find_memory_type_index(&image_memory_req, vk::MemoryPropertyFlags::DEVICE_LOCAL)
                .expect("Unable to find suitable memory index for image");
            let image_allocate_info = vk::MemoryAllocateInfo {
                allocation_size: image_memory_req.size,
                memory_type_index: image_memory_index,
                ..Default::default()
            };
            let device_memory = device
                .ash_device
                .allocate_memory(&image_allocate_info, None)
                .expect("Unable to allocate image device memory");

            device
                .ash_device
                .bind_image_memory(image, device_memory, 0)
                .expect("Unable to bind device memory to image");

            let view_type = if desc.image_type == ImageType::Tex2d && desc.array_layers == 1 {
                vk::ImageViewType::TYPE_2D
            } else if desc.image_type == ImageType::Tex2dArray && desc.array_layers > 1 {
                vk::ImageViewType::TYPE_2D_ARRAY
            } else if desc.image_type == ImageType::Cube {
                vk::ImageViewType::CUBE
            } else {
                unimplemented!()
            };

            let image_view = Image::create_image_view(
                &device,
                image,
                desc.format,
                desc.aspect_flags,
                view_type,
                0,
                desc.array_layers,
                desc.mip_levels,
            );

            let mut layer_views = vec![];

            if desc.array_layers > 1 {
                for layer in 0..desc.array_layers {
                    let view = Image::create_image_view(
                        &device,
                        image,
                        desc.format,
                        desc.aspect_flags,
                        if desc.image_type == ImageType::Cube {
                            vk::ImageViewType::TYPE_2D
                        } else {
                            view_type
                        },
                        layer,
                        1,
                        desc.mip_levels,
                    );
                    layer_views.push(view);
                }
            }

            Image {
                image,
                image_view,
                layer_views,
                device_memory,
                current_layout: initial_layout,
                desc,
                debug_name: "unnamed_image".to_string(),
                device,
            }
        }
    }
    pub fn new_from_handle(device: Arc<Device>, image: vk::Image, desc: ImageDesc) -> Image {
        let view_type = if desc.image_type == ImageType::Tex2d && desc.array_layers == 1 {
            vk::ImageViewType::TYPE_2D
        } else if desc.image_type == ImageType::Tex2dArray && desc.array_layers > 1 {
            vk::ImageViewType::TYPE_2D_ARRAY
        } else if desc.image_type == ImageType::Cube {
            vk::ImageViewType::CUBE
        } else {
            unimplemented!()
        };

        let image_view = Image::create_image_view(
            &device,
            image,
            desc.format,
            desc.aspect_flags,
            view_type,
            0,
            1,
            desc.mip_levels,
        );

        Image {
            image,
            image_view,
            layer_views: vec![],
            device_memory: vk::DeviceMemory::null(),
            current_layout: vk::ImageLayout::UNDEFINED,
            desc,
            debug_name: "unnamed_image".to_string(),
            device,
        }
    }
    pub fn transition_layout(
        &self,
        device: &Device,
        cb: vk::CommandBuffer,
        new_layout: vk::ImageLayout,
    ) {
        let (src_access_mask, src_stage_mask) = match self.current_layout {
            ImageLayout::UNDEFINED => (AccessFlags::HOST_WRITE, PipelineStageFlags::HOST),
            ImageLayout::PREINITIALIZED => (AccessFlags::HOST_WRITE, PipelineStageFlags::HOST),
            ImageLayout::TRANSFER_DST_OPTIMAL => {
                (AccessFlags::TRANSFER_WRITE, PipelineStageFlags::TRANSFER)
            }
            ImageLayout::TRANSFER_SRC_OPTIMAL => {
                (AccessFlags::TRANSFER_READ, PipelineStageFlags::TRANSFER)
            }
            ImageLayout::GENERAL => (AccessFlags::HOST_WRITE, PipelineStageFlags::HOST),
            ImageLayout::SHADER_READ_ONLY_OPTIMAL => {
                (AccessFlags::HOST_WRITE, PipelineStageFlags::HOST)
            }
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL => (
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ),
            _ => unimplemented!(),
        };

        let (dst_access_mask, dst_stage_mask) = match new_layout {
            ImageLayout::TRANSFER_SRC_OPTIMAL => {
                (AccessFlags::TRANSFER_READ, PipelineStageFlags::TRANSFER)
            }
            ImageLayout::TRANSFER_DST_OPTIMAL => {
                (AccessFlags::TRANSFER_WRITE, PipelineStageFlags::TRANSFER)
            }
            ImageLayout::SHADER_READ_ONLY_OPTIMAL => (
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            ),
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL => (
                AccessFlags::COLOR_ATTACHMENT_WRITE,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ),
            ImageLayout::GENERAL => (
                AccessFlags::SHADER_READ,
                PipelineStageFlags::FRAGMENT_SHADER,
            ),
            _ => unimplemented!(),
        };

        let texture_barrier = vk::ImageMemoryBarrier {
            src_access_mask,
            dst_access_mask,
            new_layout,
            image: self.image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: self.desc.aspect_flags,
                level_count: 1,
                layer_count: self.desc.array_layers,
                ..Default::default()
            },
            ..Default::default()
        };

        unsafe {
            device.ash_device.cmd_pipeline_barrier(
                cb,
                src_stage_mask,
                dst_stage_mask,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[texture_barrier],
            );
        }
    }
}