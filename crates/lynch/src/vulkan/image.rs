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
    pub desc: ImageDesc,
    pub device: Arc<Device>,
}