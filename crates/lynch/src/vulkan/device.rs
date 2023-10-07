use ash::extensions::khr;
use ash::extensions::khr::Surface;
use ash::extensions::khr::Swapchain;
use ash::vk;
use ash::vk::Buffer;



pub struct Device {
    pub ash_device: ash::Device,
    pub physical_device: vk::PhysicalDevice,
    pub cmd_pool: vk::CommandPool,
    pub setup_cmd_buf: vk::CommandBuffer,
    pub queue: vk::Queue,
    pub device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub queue_family_index: u32,
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

            let properties = instance.get_physical_device_properties(physical_device);

            Device {
                ash_device: device,
                physical_device,
                queue: present_queue,
                queue_family_index,
                device_memory_properties,
                cmd_pool,
                setup_cmd_buf,
                debug_utils,
            }
            }
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
        fn check_device_extension_support(
            instance: &ash::Instance,
            device: vk::PhysicalDevice,
        ) -> bool {
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
        fn get_required_device_extensions() -> [&'static std::ffi::CStr; 1] {
            [Swapchain::name()]
        }
        fn is_device_suitable(
            instance: &ash::Instance,
            surface: &Surface,
            surface_khr: vk::SurfaceKHR,
            device: vk::PhysicalDevice,
        ) -> bool { true }
    
    fn check_device_extension_support(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
    ) -> bool {
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
    fn get_required_device_extensions() -> [&'static std::ffi::CStr; 1] {
        [Swapchain::name()]
    }
    fn is_device_suitable(
        instance: &ash::Instance,
        surface: &Surface,
        surface_khr: vk::SurfaceKHR,
        device: vk::PhysicalDevice,
    ) -> bool { true }
}





pub struct CleanupResources {
    buffer_queue: VecDeque<Buffer>,
}
impl CleanupResources {
    pub fn add_buffer(&mut self, buffer: Buffer) {
        self.buffer_queue.push_back(buffer);
    }
    pub fn buffer_queue_pop_front(&mut self) -> Option<ash::vk::Buffer> {
        return self.buffer_queue.pop_front();
    }
}
