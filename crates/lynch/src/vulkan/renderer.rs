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
            
        Self {
            None,
            todo!()
        }
    }
}