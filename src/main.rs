mod util;

use ash::{
    extensions::ext::DebugReport,
    version::{EntryV1_0, InstanceV1_0}, vk::DeviceGroupBindSparseInfo,
};
use ash::{vk, Entry, Instance};
use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_void},
};

struct VulkanApp {
    _entry: Entry,
    instance: Instance,
    debug_report_callback: Option<(DebugReport, vk::DebugReportCallbackEXT)>,
    _physical_device: vk::PhysicalDevice,
}

impl VulkanApp {
    fn new() -> Self {
        log::debug!("Creating application.");

        let entry = ash::Entry::new().expect("Failed to create entry.");
        let instance = Self::create_instance(&entry);
        let debug_report_callback = Self::setup_debug_messenger(&entry, &instance);
        let _physical_device = Self::get_physical_device(&instance);
        Self {
            _entry: entry,
            instance,
            debug_report_callback,
            _physical_device,
        }
    }

    fn create_instance(entry: &Entry) -> Instance {
        let app_name = CString::new("Vulkan Application").unwrap();
        let engine_name = CString::new("No Engine").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(app_name.as_c_str())
            .application_version(ash::vk_make_version!(0, 1, 0))
            .engine_name(engine_name.as_c_str())
            .engine_version(ash::vk_make_version!(0, 1, 0))
            .api_version(ash::vk_make_version!(1, 0, 0))
            .build();

        let mut extension_names = util::required_extension_names();
        if ENABLE_VALIDATION_LAYERS {
            extension_names.push(DebugReport::name().as_ptr());
        }

        let layer_names = REQUIRED_LAYERS
            .iter()
            .map(|name| CString::new(*name).expect("Failed to build CString"))
            .collect::<Vec<_>>();
        let layer_names_ptrs = layer_names
            .iter()
            .map(|name| name.as_ptr())
            .collect::<Vec<_>>();

        let mut instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);
        if ENABLE_VALIDATION_LAYERS {
            Self::check_validation_layer_support(&entry);
            instance_create_info = instance_create_info.enabled_layer_names(&layer_names_ptrs);
        }

        unsafe { entry.create_instance(&instance_create_info, None).unwrap() }
    }

    fn check_validation_layer_support(entry: &Entry) {
        for required in REQUIRED_LAYERS.iter() {
            let found = entry
                .enumerate_instance_layer_properties()
                .unwrap()
                .iter()
                .any(|layer| {
                    let name = unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) };
                    let name = name.to_str().expect("Failed to get layer name pointer");
                    required == &name
                });

            if !found {
                panic!("Validation layer not supported: {}", required);
            }
        }
    }

    fn setup_debug_messenger(
        entry: &Entry,
        instance: &Instance,
    ) -> Option<(DebugReport, vk::DebugReportCallbackEXT)> {
        if !ENABLE_VALIDATION_LAYERS {
            return None;
        }
        let create_info = vk::DebugReportCallbackCreateInfoEXT::builder()
            .flags(vk::DebugReportFlagsEXT::all())
            .pfn_callback(Some(vulkan_debug_callback))
            .build();
        let debug_report = DebugReport::new(entry, instance);
        let debug_report_callback = unsafe {
            debug_report
                .create_debug_report_callback(&create_info, None)
                .unwrap()
        };
        Some((debug_report, debug_report_callback))
    }
    fn get_physical_device(instance: &Instance) -> vk::PhysicalDevice {
        let physical_device_handle  = unsafe { instance.enumerate_physical_devices().unwrap() };
        if physical_device_handle.len() == 0 {
            panic!("ahhh!");
        }
        let device = physical_device_handle.
            into_iter().
            find(|device | Self::is_device_suitable(instance, *device)).
            expect("no suitable device found");
        let props = unsafe {instance.get_physical_device_properties(device)};
        // i really need to not use c bindings smh
        log::debug!("Selected device: {:?}", unsafe {
            CStr::from_ptr(props.device_name.as_ptr())
        });
        device


    }
    fn is_device_suitable(instance: &Instance, device: vk::PhysicalDevice) -> bool{ 
        Self::find_queue_families(instance, device).is_some()
    }
    fn find_queue_families(instance: &Instance,  device: vk::PhysicalDevice) -> Option<usize> {
        let props = unsafe { instance.get_physical_device_queue_family_properties(device)};
        props. iter().enumerate().find(|(_, family)|{
            family.queue_count > 0 && family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        }).map(|(index, _ )| index)
    }
    fn run(&mut self) {
        log::debug!("Running application.");
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        log::debug!("Dropping application.");
        unsafe {
            if let Some((report, callback)) = self.debug_report_callback.take() {
                report.destroy_debug_report_callback(callback, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

fn main() {
    env_logger::init();
    VulkanApp::new().run()
}