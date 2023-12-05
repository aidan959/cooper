use std::sync::Arc;

use ash::vk;

use crate::Texture;

use super::{shader::BindingMap, Device, Buffer, Image};

pub struct DescriptorSet {
    pub handle: vk::DescriptorSet,
    pub pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
    binding_map: BindingMap,
    device: Arc<Device>
}

pub enum DescriptorIdentifier {
    Name(String),
    Index(u32),
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

        // Todo: Every descriptor should not have its own pool
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

        let descriptor_sets = {
            let descriptor_alloc_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(descriptor_pool)
                .set_layouts(&[layout])
                .build();

            unsafe {
                device
                    .device()
                    .allocate_descriptor_sets(&descriptor_alloc_info)
                    .expect("Error allocating descriptor sets")
            }
        };

        DescriptorSet {
            handle: descriptor_sets[0],
            pool: descriptor_pool,
            binding_map,
            device,
            layout
        }
    }

    pub fn write_uniform_buffer(&self, name: String, buffer: &Buffer) {
        let buffer_info = vk::DescriptorBufferInfo::builder()
            .offset(0)
            .range(buffer.size)
            .buffer(buffer.buffer)
            .build();

        let binding = match self.binding_map.get(&name) {
            Some(binding) => binding,
            None => panic!("No descriptor binding found with name: \"{}\"", name),
        };
        let descriptor_writes = vk::WriteDescriptorSet::builder()
            .dst_set(self.handle)
            .dst_binding(binding.binding)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(&[buffer_info])
            .build();
        unsafe {
            self.device
                .device()
                .update_descriptor_sets(&[descriptor_writes], &[])
        };
    }

    pub fn write_storage_buffer(
        &self,
        device: &Device,
        name: DescriptorIdentifier,
        buffer: &Buffer,
    ) {
        let binding = match name {
            DescriptorIdentifier::Name(name) => match self.binding_map.get(&name) {
                Some(binding) => binding.binding,
                None => panic!("No descriptor binding found with name: \"{}\"", name),
            },
            DescriptorIdentifier::Index(index) => index,
        };

        let buffer_info = vk::DescriptorBufferInfo::builder()
            .offset(0)
            .range(buffer.size)
            .buffer(buffer.buffer)
            .build();

        let descriptor_writes = vk::WriteDescriptorSet::builder()
            .dst_set(self.handle)
            .dst_binding(binding)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER) // todo
            .buffer_info(&[buffer_info])
            .build();

        unsafe {
            device
                .device()
                .update_descriptor_sets(&[descriptor_writes], &[])
        };
    }

    pub fn write_combined_image(
        &self,
        device: &Device,
        name: DescriptorIdentifier,
        texture: &Texture,
    ) {
        let binding = match name {
            DescriptorIdentifier::Name(name) => match self.binding_map.get(&name) {
                Some(binding) => binding.binding,
                None => panic!("No descriptor binding found with name: \"{}\"", name),
            },
            DescriptorIdentifier::Index(index) => index,
        };

        let descriptor_writes = vk::WriteDescriptorSet::builder()
            .dst_set(self.handle)
            .dst_binding(binding)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&[texture.descriptor_info])
            .build();

        unsafe {
            device
                .device()
                .update_descriptor_sets(&[descriptor_writes], &[])
        };
    }

    pub fn write_storage_image(&self, device: &Device, name: DescriptorIdentifier, image: &Image) {
        let binding = match name {
            DescriptorIdentifier::Name(name) => match self.binding_map.get(&name) {
                Some(binding) => binding.binding,
                None => panic!("No descriptor binding found with name: \"{}\"", name),
            },
            DescriptorIdentifier::Index(index) => index,
        };

        let descriptor_info = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::GENERAL,
            image_view: image.image_view,
            sampler: vk::Sampler::null(),
        };

        let descriptor_writes = vk::WriteDescriptorSet::builder()
            .dst_set(self.handle)
            .dst_binding(binding)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .image_info(&[descriptor_info])
            .build();

        unsafe {
            device
                .device()
                .update_descriptor_sets(&[descriptor_writes], &[])
        };
    }

    pub fn write_acceleration_structure(
        &self,
        device: &Device,
        name: DescriptorIdentifier,
        acceleration_structure: vk::AccelerationStructureKHR,
    ) {
        let binding = match name {
            DescriptorIdentifier::Name(name) => match self.binding_map.get(&name) {
                Some(binding) => binding.binding,
                None => panic!("No descriptor binding found with name: \"{}\"", name),
            },
            DescriptorIdentifier::Index(index) => index,
        };

        let mut descriptor_info = vk::WriteDescriptorSetAccelerationStructureKHR::builder()
            .acceleration_structures(std::slice::from_ref(&acceleration_structure))
            .build();

        let mut descriptor_writes = vk::WriteDescriptorSet::builder()
            .dst_set(self.handle)
            .dst_binding(binding)
            .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
            .push_next(&mut descriptor_info)
            .build();
        descriptor_writes.descriptor_count = 1;

        unsafe {
            device
                .device()
                .update_descriptor_sets(&[descriptor_writes], &[])
        };
    }


    pub fn write_raw_storage_buffer(
        device: &Device,
        descriptor_set: vk::DescriptorSet,
        binding: u32,
        buffer: &Buffer,
    ) {
        let buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(buffer.buffer)
            .range(buffer.size)
            .build();

        let descriptor_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_binding(binding)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&buffer_info))
            .build();

        unsafe {
            device
                .device()
                .update_descriptor_sets(std::slice::from_ref(&descriptor_write), &[])
        };
    }

    pub fn get_set_index(&self) -> u32 {
        self.binding_map
            .iter()
            .next()
            .expect("Empty DescriptorSet")
            .1
            .set
    }
    pub fn clean_vk_resources(&self) {
        unsafe {
            self.device.device().destroy_descriptor_pool(self.pool, None);
            self.device.device().destroy_descriptor_set_layout(self.layout, None);
        };
    }
}