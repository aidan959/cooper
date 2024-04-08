use ash::util::*;
use ash::vk;
use log::debug;
use rspirv_reflect::DescriptorInfo;
use rspirv_reflect::DescriptorType;
use rspirv_reflect::PushConstantInfo;
use rspirv_reflect::Reflection;
use shaderc::CompilationArtifact;
use shaderc::EnvVersion;
use shaderc::Error;
use shaderc::IncludeCallbackResult;
use shaderc::ResolvedInclude;
use shaderc::ShaderKind;
use shaderc::TargetEnv;

use std::fs::read_to_string;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    io::Cursor,
    path::Path,
};

use rspirv_reflect;
use shaderc;

#[derive(Debug, Clone)]
pub struct Binding {
    pub set: u32,
    pub binding: u32,
    pub info: DescriptorInfo,
}

impl std::ops::Deref for Binding {
    type Target = DescriptorInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

type DescriptorSetMap = BTreeMap<u32, BTreeMap<u32, DescriptorInfo>>;
pub type BindingMap = BTreeMap<String, Binding>;



#[derive(Default)]
pub struct ShaderReflect {
    pub descriptor_set_reflections: DescriptorSetMap,
    pub push_constant_reflections: Vec<PushConstantInfo>,
    pub binding_mappings: HashMap<String, Binding>,
}

const MAIN_ENTRY_POINT: &'static str = r#"main"#;

impl ShaderReflect {
    pub fn new(shader_stages: &[&[u8]]) -> ShaderReflect {
        let mut descriptor_sets_combined: DescriptorSetMap = BTreeMap::new();
        let mut push_constant_ranges: Vec<PushConstantInfo> = vec![];

        for shader_stage in shader_stages {
            let stage_reflection = Reflection::new_from_spirv(shader_stage)
                .expect("Shader reflection failed!");

            let descriptor_sets = stage_reflection.get_descriptor_sets().unwrap();

            for (set, descriptor_set) in descriptor_sets {
                if let Some(existing_descriptor_set) = descriptor_sets_combined.get_mut(&set) {
                    for (binding, descriptor) in descriptor_set {
                        if let Some(existing_descriptor) = existing_descriptor_set.get(&binding) {
                            assert!(
                                (descriptor.ty == existing_descriptor.ty && descriptor.name == existing_descriptor.name),
                                "Set: {} binding: {} inconsistent between shader stages:\n{:#?} {:#?}",
                                set,
                                binding,
                                descriptor,
                                *existing_descriptor,
                            );
                        } else {
                            existing_descriptor_set.insert(binding, descriptor);
                            println!("Set: {} binding: {} does not exist, adding!", set, binding);
                        }
                    }
                } else {
                    descriptor_sets_combined.insert(set, descriptor_set);
                }
            }

            if let Some(push_constant_reflection) =
                stage_reflection.get_push_constant_range().unwrap()
            {
                push_constant_ranges.push(push_constant_reflection);
            }
        }

        let binding_mappings: HashMap<String, Binding> = descriptor_sets_combined
            .iter()
            .flat_map(|(set_key, set_val)| {
                let bindings: HashMap<String, Binding> = set_val
                    .iter()
                    .map(|(binding_key, binding_val)| {
                        (
                            binding_val.name.clone(),
                            Binding {
                                set: *set_key,
                                binding: *binding_key,
                                info: binding_val.clone(),
                            },
                        )
                    })
                    .collect();

                bindings
            })
            .collect();

        ShaderReflect {
            descriptor_set_reflections: descriptor_sets_combined,
            push_constant_reflections: push_constant_ranges,
            binding_mappings,
        }
    }

    pub fn get_set_mappings(&self, set: u32) -> BindingMap {
        self.binding_mappings
            .iter()
            .filter_map(|(key, val)| {
                if val.set == set {
                    Some((key.clone(), val.clone()))
                } else {
                    None
                }
            })
            .collect::<BindingMap>()
    }

    pub fn get_binding(&self, name: &str) -> Binding {
        match self.binding_mappings.get(name) {
            Some(binding) => binding.clone(),
            None => panic!("Binding with \"{}\" name not available", name),
        }
    }
}

#[must_use]
pub fn compile_glsl_shader(path: &str) -> Result<CompilationArtifact, Error> {
    let binding = read_to_string(path);
    let source = match &binding {
        Ok(source) => source,
        Err(err) => panic!(
            "Error compiling shader: Cannot find path: {}.\nOs Error:({})",
            path, err
        ),
    };

    let shader_kind = match path.get(path.len().saturating_sub(5)..) {
        Some(".vert") => ShaderKind::Vertex,
        Some(".frag") => ShaderKind::Fragment,
        Some(".comp") => ShaderKind::Compute,
        _ => todo!("Unsupported."),
    };

    let mut compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.add_macro_definition("EP", Some(MAIN_ENTRY_POINT));
    options.set_target_env(TargetEnv::Vulkan, EnvVersion::Vulkan1_2 as u32);
    options.set_generate_debug_info();
    options.set_include_callback(|include_request, _include_type, _source, _size| -> Result<ResolvedInclude, String> {
        let include_path = Path::new(path).parent().unwrap();
        let mut include_path = include_path.join(include_request);
        if !Path::new(&include_path).exists() {
            include_path = Path::new("assets/shaders").join(include_request);
        }

        let include_source =
            &read_to_string(include_path).expect("Error reading included file")[..];

        IncludeCallbackResult::Ok(ResolvedInclude {
            resolved_name: include_request.to_string(),
            content: include_source.to_string(),
        })
    });

    let binary_result =
        compiler.compile_into_spirv(source, shader_kind, path, MAIN_ENTRY_POINT, Some(&options))?;

    debug_assert_eq!(Some(&119734787), binary_result.as_binary().first());

    let text_result = compiler
        .compile_into_spirv_assembly(source, shader_kind, path, MAIN_ENTRY_POINT, Some(&options))
        .unwrap();

    debug_assert!(text_result.as_text().starts_with("; SPIR-V\n"));

    Ok(binary_result)
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
                    .map(|(binding, descriptor_info)| {
                        let descriptor_type = match descriptor_info.ty {
                            DescriptorType::COMBINED_IMAGE_SAMPLER => {
                                vk::DescriptorType::COMBINED_IMAGE_SAMPLER
                            }
                            DescriptorType::SAMPLED_IMAGE => {
                                vk::DescriptorType::SAMPLED_IMAGE
                            }
                            DescriptorType::STORAGE_IMAGE => {
                                vk::DescriptorType::STORAGE_IMAGE
                            }
                            DescriptorType::UNIFORM_BUFFER => {
                                vk::DescriptorType::UNIFORM_BUFFER
                            }
                            DescriptorType::STORAGE_BUFFER => {
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
// TODO the receiver who takes ownership of this must clean this up vulkan side
// TODO We must cache this to improve launch times
#[must_use]
pub fn create_shader_module(mut spv_file: Cursor<&[u8]>, device: &ash::Device) -> vk::ShaderModule {
    let shader_code = read_spv(&mut spv_file).expect("Failed to read shader SPIR-V shader mod.");
    let shader_info = vk::ShaderModuleCreateInfo::builder().code(&shader_code);
    unsafe {
        device
            .create_shader_module(&shader_info, None)
            .expect("Error creating shader module on device.")
    }
}
