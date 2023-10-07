use ash::util::*;
use ash::vk;

use std::{collections::{BTreeMap, HashMap},fs,io::Cursor,path::Path};

use rspirv_reflect::{DescriptorInfo, PushConstantInfo};
use shaderc;


type DescriptorSetMap = BTreeMap<u32, BTreeMap<u32, DescriptorInfo>>;
pub type BindingMap = BTreeMap<String, Binding>;

#[derive(Debug, Clone)]
pub struct Binding {
    pub set: u32,
    pub binding: u32,
    pub info: DescriptorInfo,
}

#[derive(Default)]
pub struct Reflection {
    pub descriptor_set_reflections: DescriptorSetMap,
    pub push_constant_reflections: Vec<PushConstantInfo>,
    pub binding_mappings: HashMap<String, Binding>,
}


pub fn create_shader_module(mut spv_file: Cursor<&[u8]>, device: &ash::Device) -> vk::ShaderModule {
    todo!();
    let shader_info = vk::ShaderModuleCreateInfo::builder();

    unsafe {
        device
            .create_shader_module(&shader_info, None)
            .expect("Error creating shader module")
    }
}