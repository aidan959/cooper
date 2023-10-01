use ash::{
    extensions::{
        ext::DebugReport,
        khr::{Surface, Swapchain},
    },
    version::{DeviceV1_0, EntryV1_0, InstanceV1_0},
};
use ash::{vk, Device, Entry, Instance};
use crate::vulkan::cont::*;
use crate::vulkan::window::*;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;


pub struct VulkanRenderer {
    resize_dimensions: Option<[u32; 2]>,
    vk_context: VkContext,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    swapchain: Swapchain,
    swapchain_khr: vk::SwapchainKHR,
    swapchain_properties: SwapchainProperties,
    images: Vec<vk::Image>,
    msaa_samples: vk::SampleCountFlags,
    swapchain_image_views: Vec<vk::ImageView>,
}

impl VulkanRenderer {
    fn create_instance(entry: &Entry) -> Instance {
        let app_name = CString::new("Cooper").unwrap();
        let engine_name = CString::new("Lynch").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(app_name.as_c_str())
            .application_version(ash::vk_make_version!(0, 1, 0))
            .engine_name(engine_name.as_c_str())
            .engine_version(ash::vk_make_version!(0, 1, 0))
            .api_version(ash::vk_make_version!(1, 0, 0))
            .build();
        
        let mut extension_names = crate::vulkan::surface::required_extension_names();

        if ENABLE_VALIDATION_LAYERS {
            extension_names.push(DebugReport::name().as_ptr());
        }

        let (_layer_names, layer_pointers) = get_lay_names_pointers();

        let mut instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);

        if ENABLE_VALIDATION_LAYERS {
            check_validation_layer_support(&entry);
            instance_create_info = instance_create_info.enabled_layer_names(&layer_pointers);
        }

        unsafe { entry.create_instance(&instance_create_info, None).unwrap() }            
    }


    fn get_physical_device(
        instance: &Instance,
        surface: &Surface,
        surface_khr: &vk::SurfaceKHR
    ) -> (vk::PhysicalDevice, QueueFamiliesIndices) {
        let physical_device_handle = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("ahhh! No graphics card! Ahh! How!?")
        };

        let device = physical_device_handle
            .into_iter()
            .find(|device| Self::is_device_suitable(instance, surface, surface_khr, *device))
            .expect("no suitable device found");

        let props = unsafe { instance.get_physical_device_properties(device) };
        // i really need to not use c bindings smh
        log::debug!("Selected device: {:?}", unsafe {
            CStr::from_ptr(props.device_name.as_ptr())
        });

        let (graphics, present) = Self::find_queue_families(instance, surface, surface_khr, device);
        let queue_families_indices = QueueFamiliesIndices {
            graphics_index: graphics.unwrap(),
            present_index: present.unwrap(),
        };
        (device, queue_families_indices)
    }
    fn is_device_suitable(
        instance: &Instance,
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
    fn check_device_extension_support(instance: &Instance, device: vk::PhysicalDevice) -> bool {
        let required_extentions = Self::get_required_device_extensions();

        let extension_props = unsafe {
            instance
                .enumerate_device_extension_properties(device)
                .unwrap()
        };

        for required in required_extentions.iter() {
            let found = extension_props.iter().any(|ext| {
                let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                required == &name
            });

            if !found {
                return false;
            }
        }

        true
    }
    fn find_queue_families(
        instance: &Instance,
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
            if unsafe { surface.get_physical_device_surface_support(device, index, surface_khr) }
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
    fn get_required_device_extensions() -> [&'static CStr; 1] {
        [Swapchain::name()]
    }
    fn get_logical_device_queue(
        instance: &Instance,
        device: vk::PhysicalDevice,
        queue_families_indices: QueueFamiliesIndices,
    ) -> (Device, vk::Queue, vk::Queue) {
        let graphics_family_index = queue_families_indices.graphics_index;
        let present_family_index = queue_families_indices.present_index;

        let queue_priorities: [f32; 1] = [1.];

        let queue_create_infos = {
            let mut indices = vec![graphics_family_index, present_family_index];
            indices.dedup();

            indices
                .iter()
                .map(|index| {
                    vk::DeviceQueueCreateInfo::builder()
                        .queue_priorities(&queue_priorities)
                        .queue_family_index(*index)
                        .build()
                })
                .collect::<Vec<_>>()
        };
        
        let device_extensions = Self::get_required_device_extensions();
        let device_extensions_ptrs = device_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();
        let device_features = vk::PhysicalDeviceFeatures::builder()
            .sampler_anisotropy(true)
            .build();
        let mut device_info_builder = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extesions_ptrs)
            .enabled_features(&device_features);

        let (_layer_names, layer_pointers) = get_lay_names_pointers();
        if ENABLE_VALIDATION_LAYERS {
            device_info_builder = device_info_builder.enabled_layer_names(&layer_pointers);
        }
        let device_create_info = device_info_builder.build();

        let device = unsafe {
            instance
                .create_device(device, &device_create_info, None)
                .expect("Logical device could not be created")
        };

        let graphics_queue = unsafe { device.get_device_queue(graphics_family_index, 0) };
        let present_queue = unsafe { device.get_device_queue(present_family_index, 0) };

        (device, graphics_queue, present_queue)
    }

    fn create_swapchain_and_images(
        vk_context: &VkContext,
        queue_families_indices: QueueFamiliesIndices,
        dimensions: [u32; 2],
    ) -> (
        Swapchain,
        vk::SwapchainKHR,
        SwapchainProperties,
        Vec<vk::Image>,
    ) {
        let details = SwapchainSupportDetails::new(
            vk_context.physical_device(),
            vk_context.surface(),
            vk_context.surface_khr(),
        );
        let properties = details.get_ideal_swapchain_properties(dimensions);

        let format = properties.format;
        let present_mode = properties.present_mode;
        let extent = properties.extent;
        let image_count = {
            let max = details.capabilities.max_image_count;
            let mut preferred = details.capabilities.min_image_count + 1;
            if max > 0 && preferred > max {
                preferred = max;
            }
            preferred
        };
        log::debug!(
            "Creating swapchain.\n\tFormat: {}\n\tColorSpace: {}\n\tPresentMode: {}\n\tExtent: {:?}\n\tImageCount: {}",
            format.format,
            format.color_space,
            present_mode,
            extent,
            image_count,
        );
        let graphics = queue_families_indices.graphics_index;
        let present = queue_families_indices.present_index;

        let families_indices = [graphics, present];
        let create_info = {
            let mut builder = vk::SwapchainCreateInfoKHR::builder()
                .surface(vk_context.surface_khr())
                .min_image_count(image_count)
                .image_format(format.format)
                .image_color_space(format.color_space)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

            builder = if graphics != present {
                builder
                    .image_sharing_mode(vk::SharingMode::CONCURRENT)
                    .queue_family_indices(&families_indices)
            } else {
                builder.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            };

            builder
                .pre_transform(details.capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .build()
        };
        let swapchain = Swapchain::new(vk_context.instance(), vk_context.device());
        let swapchain_khr = 
            unsafe {
                swapchain.create_swapchain(&create_info, None).unwrap()
            };
        let images = 
            unsafe {
                swapchain.get_swapchain_images(swapchain_khr).unwrap()
            };
        (swapchain, swapchain_khr, properties, images)
    }
    fn find_depth_format(vk_context: &VkContext) -> vk::Format {
        let candidates = vec![
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ];
        vk_context
            .find_supported_format(
                &candidates,
                vk::ImageTiling::OPTIMAL,
                vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
            )
            .expect("Failed to find a supported depth format")
    }

    fn create_swapchain_image_views(
        device: &Device,
        swapchain_images: &[vk::Image],
        swapchain_properties: SwapchainProperties,
    ) -> Vec<vk::ImageView> {
        swapchain_images
            .iter()
            .map(|image| {
                Self::create_image_view(
                    device,
                    *image,
                    swapchain_properties.format.format,
                    1,
                    vk::ImageAspectFlags::COLOR,
                )
            })
            .collect::<Vec<_>>()
    }
}



impl Renderer for VulkanRenderer {
    fn create(window: &Window) -> Self {
        log::debug!("Creating application");
        let entry = ash::Entry::new().expect("Failed to create entry.");
        let instance = Self::create_instance(&entry);

        let surface = Surface::new(&entry, &instance);

        let surface_khr = 
            unsafe {
                unsafe { create_surface(&entry, &instance, &window.window) }.expect("creating surface failed");
            };

        let debug_report_callback = setup_debug_messenger(&entry, &instance);

        let (physical_device, queue_family_indices) = 
            Self::get_physical_device(&instance, &surface, surface_khr);
        

        let (logical_device, graphics_queue, present_queue) = 
            Self::get_logical_device_queue(&instance,physical_device, queue_families_indices);
        
        let vk_context = VkContext::new(
            entry,
            instance,
            debug_report_callback,
            surface,
            surface_khr,
            physical_device,
            logical_device,
        );
        let (swapchain, swapchain_khr, properties, images) = 
            Self::create_swapchain_and_images(vk_context,queue_families_indices, [WIDTH, HEIGHT]);

        let swapchain_image_views = 
            Self::create_swapchain_image_views(vk_context.device(), &images, properties);
        let msaa_samples = vk_context.get_max_usable_sample_count();
        let depth_format = Self::find_depth_format(&vk_context);
        Self {
            resize_dimensions:None,
            vk_context,
            graphics_queue,
            present_queue,
            images,
            msaa_samples,
            swapchain,
            swapchain_khr,
            swapchain_properties,
            swapchain_image_views
        }
    }
}