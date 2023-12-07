use std::sync::Arc;

use ash::vk;

use crate::vulkan::Buffer;
use crate::vulkan::Device;
use crate::vulkan::{Image, ImageDesc};

pub struct Texture {
    pub device: Arc<Device>,
    pub image: Image,
    pub sampler: vk::Sampler,
    pub descriptor_info: vk::DescriptorImageInfo,
}

impl Texture {
    pub fn clean_vk_resources(&self) {
        self.image.clean_vk_resources();

        unsafe {
            self.device.ash_device.destroy_sampler(self.sampler,None);
        }
    }
    pub fn load(device: Arc<Device>, path: &str) -> Texture {
        let image = match image::open(path) {
            Ok(image) => image,
            Err(_err) => panic!("Unable to load \"{}\"", path),
        };

        let image = image.to_rgba();
        let (width, height) = (image.width(), image.height());
        let image_data = image.into_raw();

        let mut texture = Texture::create(
            device,
            Some(&image_data),
            ImageDesc::new_2d(width, height, vk::Format::R8G8B8A8_UNORM),
            path,
        );

        texture.image.set_debug_name(path);

        texture
    }

    pub fn create(
        device: Arc<Device>,
        pixels: Option<&[u8]>,
        image_desc: ImageDesc,
        debug_name: &str,
    ) -> Texture {
        let mut image = Image::new_from_desc(device.clone(), image_desc);
        
        image.set_debug_name(debug_name);
        let mut buffer_to_destroy:Vec<Buffer> = vec![];
        device.execute_and_submit(|cb| {
            crate::vulkan::image_pipeline_barrier(
                &device,
                cb,
                &image,
                vk_sync::AccessType::General,
                vk_sync::AccessType::TransferWrite,
                true,
            );

            if let Some(pixels) = pixels {
                let staging_buffer = Buffer::new(
                    device.clone(),
                    Some(pixels),
                    std::mem::size_of_val(pixels) as u64,
                    vk::BufferUsageFlags::TRANSFER_SRC,
                    gpu_allocator::MemoryLocation::CpuToGpu,
                    Some(String::from("staging_buffer"))
                );
                
                staging_buffer.copy_to_image(cb, &image);
                buffer_to_destroy.push(staging_buffer);
            }

            if Image::is_depth_image_fmt(image.desc.format) {
                image.transition_layout(
                    &device,
                    cb,
                    vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
                );
            }
            else {
                crate::vulkan::image_pipeline_barrier(
                    &device,
                    cb,
                    &image,
                    vk_sync::AccessType::TransferWrite,
                    vk_sync::AccessType::AnyShaderReadSampledImageOrUniformTexelBuffer,
                    false,
                );
            }
        });

        let sampler_info = vk::SamplerCreateInfo {
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            address_mode_u: vk::SamplerAddressMode::MIRRORED_REPEAT,
            address_mode_v: vk::SamplerAddressMode::MIRRORED_REPEAT,
            address_mode_w: vk::SamplerAddressMode::MIRRORED_REPEAT,
            max_anisotropy: 1.0,
            border_color: vk::BorderColor::FLOAT_OPAQUE_WHITE,
            compare_op: vk::CompareOp::NEVER,
            min_lod: 0.0,
            max_lod: image_desc.mip_levels as f32,
            ..Default::default()
        };

        let sampler = unsafe {
            device
                .ash_device
                .create_sampler(&sampler_info, None)
                .expect("Unable to create sampler")
        };

        let descriptor_info = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image_view: image.image_view,
            sampler,
        };
          
        while let Some(buffer_to_destroy) = buffer_to_destroy.pop()  {
            unsafe{device.ash_device.destroy_buffer(buffer_to_destroy.buffer, None) }
        }
        Texture {
            device,
            image,
            sampler,
            descriptor_info,
        }
    }
}
