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
}


#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
    pub coords: [f32; 2],
}

impl Vertex {
    fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        let position_desc = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(offset_of!(Self, pos) as u32)
            .build();
        let color_desc = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(offset_of!(Self, color) as u32)
            .build();
        let coords_desc = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(offset_of!(Self, coords) as u32)
            .build();
        [position_desc, color_desc, coords_desc]
    }
}

impl Display for Vertex {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "x: {}\ty: {}\tz: {}\ncx: {}\tcy: {}",
            self.pos[0], self.pos[1], self.pos[2], self.coords[0], self.coords[1]
        )
    }
}

struct InFlightFrames {
    sync_objects: Vec<SyncObjects>,
    current_frame: usize,
}

impl InFlightFrames {
    fn new(sync_objects: Vec<SyncObjects>) -> Self {
        Self {
            sync_objects,
            current_frame: 0,
        }
    }

    fn destroy(&self, device: &Device) {
        self.sync_objects.iter().for_each(|o| o.destroy(&device));
    }
}

impl Iterator for InFlightFrames {
    type Item = SyncObjects;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.sync_objects[self.current_frame];

        self.current_frame = (self.current_frame + 1) % self.sync_objects.len();

        Some(next)
    }
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
