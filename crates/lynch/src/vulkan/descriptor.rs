use std::sync::Arc;

use ash::vk;

use crate::Texture;


pub enum DescriptorIdentifier {
    Name(String),
    Index(u32),
}


pub struct DescriptorSet {
    pub handle: vk::DescriptorSet,
    pub pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
    binding_map: BindingMap,
    device: Arc<Device>,
}


impl DescriptorSet {
    pub fn new(
        device: Arc<Device>,
        layout: vk::DescriptorSetLayout,
        binding_map: BindingMap,
    ) -> DescriptorSet {
        let descriptor_pool_sizes = binding_map
            .values()
            .map(|val| {
                let descriptor_type = match val.info.ty {
                    rspirv_reflect::DescriptorType::COMBINED_IMAGE_SAMPLER => {
                        vk::DescriptorType::COMBINED_IMAGE_SAMPLER
                    }
                    rspirv_reflect::DescriptorType::SAMPLED_IMAGE => {
                        vk::DescriptorType::SAMPLED_IMAGE
                    }
                    rspirv_reflect::DescriptorType::STORAGE_IMAGE => {
                        vk::DescriptorType::STORAGE_IMAGE
                    }
                    rspirv_reflect::DescriptorType::UNIFORM_BUFFER => {
                        vk::DescriptorType::UNIFORM_BUFFER
                    }
                    rspirv_reflect::DescriptorType::STORAGE_BUFFER => {
                        vk::DescriptorType::STORAGE_BUFFER
                    }
                    _ => unimplemented!(),
                };

                vk::DescriptorPoolSize::builder()
                    .ty(descriptor_type)
                    .descriptor_count(1)
                    .build()
            })
            .collect::<Vec<_>>();
        let descriptor_pool = {
            let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
                .pool_sizes(&descriptor_pool_sizes)
                .flags(
                    vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET
                        | vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND,
                )
                .max_sets(descriptor_pool_sizes.len() as u32);

            unsafe {
                device
                    .device()
                    .create_descriptor_pool(&descriptor_pool_info, None)
                    .expect("Error creating descriptor pool")
            }
        };
        todo!()
    }

}