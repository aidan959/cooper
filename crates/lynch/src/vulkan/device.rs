use ash::extensions::khr;
use ash::extensions::khr::Surface;
use ash::extensions::khr::Swapchain;
use ash::vk;
use ash::vk::Buffer;
use gpu_allocator::vulkan::*;
use gpu_allocator::AllocatorDebugSettings;
use winapi::um::cfgmgr32::BUSNUMBER_DES;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::Image;
use super::swapchain::SwapchainSupportDetails;

pub struct Device {
    pub ash_device: ash::Device,
    pub physical_device: vk::PhysicalDevice,
    pub queue: vk::Queue,
    pub cmd_pool: vk::CommandPool,
    pub setup_cmd_buf: vk::CommandBuffer,
    pub device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub queue_family_index: u32,
    pub gpu_allocator: Arc<Mutex<gpu_allocator::vulkan::Allocator>>,
    pub debug_utils: Option<ash::extensions::ext::DebugUtils>,
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { 
            self.ash_device.destroy_device(None);
            self.ash_device.device_wait_idle().unwrap() };
    }
}

impl Device {
    pub fn new(
        instance: &ash::Instance,
        surface: vk::SurfaceKHR,
        surface_loader: &Surface,
        debug_utils: Option<ash::extensions::ext::DebugUtils>,
    ) -> Device {
        unsafe {
            let physical_devices = 
                instance
                    .enumerate_physical_devices()
                    .expect("ahhh! No graphics card! Ahh! How!?");

            // single device -> gotta select the better one in future
            let physical_device = physical_devices.into_iter().find(|device|{Self::is_device_suitable(instance, surface_loader, surface, *device)}).expect("no suitable device found");

            let queue_family_properties =
                instance.get_physical_device_queue_family_properties(physical_device);
            let queue_family_index = queue_family_properties
                .iter()
                .position(|info| info.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                .expect("Did not find any matching graphics queue");
            let queue_family_index = queue_family_index as u32;
            surface_loader
                .get_physical_device_surface_support(physical_device, queue_family_index, surface).unwrap();

            //println!("Supported extensions:");
            let supported_extension_names: Vec<_> = instance
                .enumerate_device_extension_properties(physical_device)
                .unwrap()
                .iter()
                .map(|extension| {
                    let name = std::ffi::CStr::from_ptr(extension.extension_name.as_ptr())
                        .to_string_lossy()
                        .as_ref()
                        .to_owned();
                    
                    name
                })
                .collect();

            let device_extension_names_raw = vec![
                Swapchain::name().as_ptr(),
                vk::ExtDescriptorIndexingFn::name().as_ptr(),
                vk::KhrDynamicRenderingFn::name().as_ptr(),
                vk::KhrMaintenance1Fn::name().as_ptr(),
                vk::KhrMaintenance2Fn::name().as_ptr(),
                vk::KhrMaintenance3Fn::name().as_ptr(),
            ];



            let mut descriptor_indexing_features =
                vk::PhysicalDeviceDescriptorIndexingFeaturesEXT::default();
            let mut buffer_device_address_features =
                vk::PhysicalDeviceBufferDeviceAddressFeaturesKHR::default();
            let mut scalar_block_layout_features =
                vk::PhysicalDeviceScalarBlockLayoutFeatures::default();
            let mut dynamic_rendering_features =
                vk::PhysicalDeviceDynamicRenderingFeatures::default();

            let mut features2_builder = vk::PhysicalDeviceFeatures2::builder()
                .push_next(&mut descriptor_indexing_features)
                .push_next(&mut buffer_device_address_features)
                .push_next(&mut scalar_block_layout_features)
                .push_next(&mut dynamic_rendering_features);


            let mut features2 = features2_builder.build();

            instance.get_physical_device_features2(physical_device, &mut features2);

            let queue_priorities = [1.0];
            let queue_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&queue_priorities);

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_extension_names_raw)
                .push_next(&mut features2);

            let device: ash::Device = instance
                .create_device(physical_device, &device_create_info, None)
                .expect("Failed to create logical Vulkan device");

            let present_queue = device.get_device_queue(queue_family_index, 0);

            let device_memory_properties =
                instance.get_physical_device_memory_properties(physical_device);

            let (cmd_pool, setup_cmd_buf) =
                Device::create_setup_command_buffer(&device, queue_family_index);

            // println!("{:#?}", rt_pipeline_properties);
            // println!("{:#?}", as_features);

            let mut gpu_allocator = Allocator::new(&AllocatorCreateDesc {
                instance: instance.clone(),
                device: device.clone(),
                physical_device,
                debug_settings: AllocatorDebugSettings {
                    log_leaks_on_shutdown: true,
                    log_memory_information: true,
                    log_allocations: true,
                    log_stack_traces: true,
                    ..Default::default()
                },
                buffer_device_address: true,
            })
            .expect("Failed to create GPU allocator");

            let properties = instance.get_physical_device_properties(physical_device);
            
            Device {
                ash_device: device,
                physical_device,
                queue: present_queue,
                queue_family_index,
                device_memory_properties,
                cmd_pool,
                setup_cmd_buf,
                gpu_allocator: Arc::new(Mutex::new(gpu_allocator)),
                debug_utils,
            }
        }
    }
    pub fn device(&self) -> &ash::Device{
        &self.ash_device
    }
    fn is_device_suitable(
        instance: &ash::Instance,
        surface: &Surface,
        surface_khr: vk::SurfaceKHR,
        device: vk::PhysicalDevice,
    ) -> bool {
        let (graphics, present) = Self::find_queue_families(instance, surface, surface_khr, device);
        let extention_support = Self::check_device_extension_support(instance, device);
        let is_swapchain_adequate = {
            let details = SwapchainSupportDetails::new(device, surface, surface_khr);
            !details.formats.is_empty() && !details.present_modes.is_empty()
        };
        let features = unsafe { instance.get_physical_device_features(device) };
        graphics.is_some()
            && present.is_some()
            && extention_support
            && is_swapchain_adequate
            && features.sampler_anisotropy == vk::TRUE
    }
    fn find_queue_families(
        instance: &ash::Instance,
        surface: &Surface,
        surface_khr: vk::SurfaceKHR,
        device: vk::PhysicalDevice,
    ) -> (Option<u32>, Option<u32>) {
        let (mut graphics, mut present) = (None, None);

        let props = unsafe { instance.get_physical_device_queue_family_properties(device) };

        for (index, family) in props.iter().filter(|f| f.queue_count > 0).enumerate() {
            let index = index as u32;
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) && graphics.is_none() {
                graphics = Some(index);
            }
            if unsafe { surface.get_physical_device_surface_support(device, index, surface_khr).expect("No phyiscal device support") }
                && present.is_none()
            {
                present = Some(index);
            }

            if graphics.is_some() && present.is_some() {
                break;
            };
        }

        (graphics, present)
    }
    fn get_required_device_extensions() -> [&'static std::ffi::CStr; 1] {
        [Swapchain::name()]
    }
    fn check_device_extension_support(instance: &ash::Instance, device: vk::PhysicalDevice) -> bool {
        let required_extentions = Self::get_required_device_extensions();

        let extension_props = unsafe {
            instance
                .enumerate_device_extension_properties(device)
                .unwrap()
        };

        for required in required_extentions.iter() {
            let found = extension_props.iter().any(|ext| {
                let name = unsafe { std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()) };
                required == &name
            });

            if !found {
                return false;
            }
        }

        true
    }
    fn create_setup_command_buffer(
        device: &ash::Device,
        queue_family_index: u32,
    ) -> (vk::CommandPool, vk::CommandBuffer) {
        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);

        let pool = unsafe {
            device
                .create_command_pool(&pool_create_info, None)
                .expect("Failed to create command pool")
        };

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate command buffer")
        };

        (pool, command_buffers[0])
    }

    pub fn execute_and_submit<F: FnOnce(&Device, vk::CommandBuffer)>(&self, recording_function: F) {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            self.ash_device
                .begin_command_buffer(self.setup_cmd_buf, &command_buffer_begin_info)
                .expect("Begin command buffer failed.")
        };

        recording_function(self, self.setup_cmd_buf);

        unsafe {
            self.ash_device
                .end_command_buffer(self.setup_cmd_buf)
                .expect("End commandbuffer failed.")
        };

        let submit_info =
            vk::SubmitInfo::builder().command_buffers(std::slice::from_ref(&self.setup_cmd_buf));

        unsafe {
            self.ash_device
                .queue_submit(self.queue, &[submit_info.build()], vk::Fence::null())
                .expect("Queue submit failed");

            self.ash_device
                .device_wait_idle()
                .expect("Device wait idle failed");
        }
    }

    pub fn find_memory_type_index(
        &self,
        memory_req: &vk::MemoryRequirements,
        flags: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        self.device_memory_properties.memory_types
            [..self.device_memory_properties.memory_type_count as _]
            .iter()
            .enumerate()
            .find(|(index, memory_type)| {
                (1 << index) & memory_req.memory_type_bits != 0
                    && memory_type.property_flags & flags == flags
            })
            .map(|(index, _memory_type)| index as _)
    }

    pub fn cmd_push_constants<T: Copy>(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_layout: vk::PipelineLayout,
        data: T,
    ) {
        unsafe {
            (self.ash_device.fp_v1_0().cmd_push_constants)(
                command_buffer,
                pipeline_layout,
                vk::ShaderStageFlags::ALL,
                0,
                std::mem::size_of_val(&data).try_into().unwrap(),
                &data as *const _ as *const _,
            );
        }
    }

    pub fn set_debug_name(&self, object_handle: u64, object_type: vk::ObjectType, name: &str) {
        if let Some(debug_utils) = &self.debug_utils {
            let name = std::ffi::CString::new(name).unwrap();
            let name_info = vk::DebugUtilsObjectNameInfoEXT::builder()
                .object_handle(object_handle)
                .object_name(&name)
                .object_type(object_type)
                .build();
            unsafe {
                debug_utils
                    .set_debug_utils_object_name(self.device().handle(), &name_info)
                    //.debug_utils_set_object_name(self.ash_device.handle(), &name_info)
                    .expect("Error setting debug name for buffer")
            };
        }
    }
}

pub fn global_pipeline_barrier(
    device: &Device,
    command_buffer: vk::CommandBuffer,
    prev_access: vk_sync::AccessType,
    next_access: vk_sync::AccessType,
) -> vk_sync::AccessType {
    vk_sync::cmd::pipeline_barrier(
        &device.ash_device,
        command_buffer,
        Some(vk_sync::GlobalBarrier {
            previous_accesses: &[prev_access],
            next_accesses: &[next_access],
        }),
        &[],
        &[],
    );

    next_access
}

pub fn image_pipeline_barrier(
    device: &Device,
    command_buffer: vk::CommandBuffer,
    image: &Image,
    prev_access: vk_sync::AccessType,
    next_access: vk_sync::AccessType,
    discard_contents: bool,
) -> vk_sync::AccessType {
    vk_sync::cmd::pipeline_barrier(
        &device.ash_device,
        command_buffer,
        None,
        &[],
        &[vk_sync::ImageBarrier {
            previous_accesses: &[prev_access],
            next_accesses: &[next_access],
            previous_layout: vk_sync::ImageLayout::Optimal,
            next_layout: vk_sync::ImageLayout::Optimal,
            discard_contents,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image: image.image, 
            range: vk::ImageSubresourceRange::builder()
                .aspect_mask(image.desc.aspect_flags)
                .layer_count(vk::REMAINING_ARRAY_LAYERS)
                .level_count(vk::REMAINING_MIP_LEVELS)
                .build(),
        }],
    );

    next_access
}
pub struct CleanupResources {
    buffer_queue: VecDeque<Buffer>
}
impl CleanupResources {
    pub fn add_buffer(&mut self, buffer:  Buffer ){
       self.buffer_queue.push_back(buffer);
    }
    pub fn buffer_queue_pop_front(&mut self) -> Option<ash::vk::Buffer>{
        return self.buffer_queue.pop_front();
    }

}