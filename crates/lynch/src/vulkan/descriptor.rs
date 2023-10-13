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
        todo!()
    }

}