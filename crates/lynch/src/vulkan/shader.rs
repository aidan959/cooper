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
pub struct ShaderReflect {
    pub descriptor_set_reflections: DescriptorSetMap,
    pub push_constant_reflections: Vec<PushConstantInfo>,
    pub binding_mappings: HashMap<String, Binding>,
}

/// TODO Owner MUST clean this up
#[must_use]
pub fn create_layouts_from_reflection(
    device: &ash::Device,
    reflection: &ShaderReflect,
    bindless_descriptor_set_layout: Option<vk::DescriptorSetLayout>,
) -> (
    vk::PipelineLayout,
    Vec<vk::DescriptorSetLayout>,
    Vec<vk::PushConstantRange>,
) {
    let mut descriptor_sets_layouts: Vec<vk::DescriptorSetLayout> = reflection
        .descriptor_set_reflections
        .values()
        .map(|descriptor_set| {
            let descriptor_set_layout_bindings: Vec<vk::DescriptorSetLayoutBinding> =
                descriptor_set
                    .iter()
                    .map(|(binding, descriptor_info)| { // this may be affecting model loader
                        let descriptor_type = match descriptor_info.ty {
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
                            _ => panic!("Unsupported descriptor type"),
                        };

                        let descriptor_set_layout_binding =
                            vk::DescriptorSetLayoutBinding::builder()
                                .binding(*binding)
                                .descriptor_type(descriptor_type)
                                .descriptor_count(1) // descriptor_info.binding_count
                                .stage_flags(vk::ShaderStageFlags::ALL)
                                .build();

                        descriptor_set_layout_binding
                    })
                    .collect();

            let descriptor_sets_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
                .bindings(&descriptor_set_layout_bindings)
                .build();

            unsafe {
                device
                    .create_descriptor_set_layout(&descriptor_sets_layout_info, None)
                    .expect("Error creating descriptor set layout")
            }
        })
        .collect();

    if let Some(bindless_layout) = bindless_descriptor_set_layout {
        descriptor_sets_layouts[0] = bindless_layout;
    }

    let mut push_constant_ranges: Vec<vk::PushConstantRange> = vec![];

    if !reflection.push_constant_reflections.is_empty() {
        push_constant_ranges.push(
            vk::PushConstantRange::builder()
                .size(reflection.push_constant_reflections[0].size)
                .offset(reflection.push_constant_reflections[0].offset)
                .stage_flags(vk::ShaderStageFlags::ALL)
                .build(),
        );
    }

    let pipeline_layout_create_info: vk::PipelineLayoutCreateInfoBuilder =
        if !push_constant_ranges.is_empty() {
            vk::PipelineLayoutCreateInfo::builder()
                .set_layouts(&descriptor_sets_layouts)
                .push_constant_ranges(&push_constant_ranges)
        } else {
            vk::PipelineLayoutCreateInfo::builder().set_layouts(&descriptor_sets_layouts)
        };

    let pipeline_layout = unsafe {
        device
            .create_pipeline_layout(&pipeline_layout_create_info, None)
            .expect("Error creating pipeline layout on device.")
    };
    (
        pipeline_layout,
        descriptor_sets_layouts,
        push_constant_ranges,
    )
}
pub fn create_shader_module(mut spv_file: Cursor<&[u8]>, device: &ash::Device) -> vk::ShaderModule {
    let shader_code = read_spv(&mut spv_file).expect("Failed to read shader spv file");
    let shader_info = vk::ShaderModuleCreateInfo::builder().code(&shader_code);
    unsafe {
        device
            .create_shader_module(&shader_info, None)
            .expect("Error creating shader module")
    }
}