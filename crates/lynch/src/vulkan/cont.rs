use ash::{
    extensions::{ext::DebugReport, khr::Surface},
    version::{DeviceV1_0, InstanceV1_0},
    vk, Device, Entry, Instance,
};

pub struct VkContext {
    _entry: Entry,
    instance: Instance,
    debug_report_callback: Option<(DebugReport, vk::DebugReportCallbackEXT)>,
    surface: Surface,
    surface_khr: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    device: Device,
}

impl VkContext {
    pub fn new(
        entry: Entry,
        instance: Instance,
        debug_report_callback: Option<(DebugReport, vk::DebugReportCallbackEXT)>,
        surface: Surface,
        surface_khr: vk::SurfaceKHR,
        physical_device: vk::PhysicalDevice,
        device: Device,
    ) -> Self {
        VkContext {
            _entry: entry,
            instance,
            debug_report_callback,
            surface,
            surface_khr,
            physical_device,
            device,
        }
    }
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    pub fn surface_khr(&self) -> vk::SurfaceKHR {
        self.surface_khr
    }

    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    pub fn device(&self) -> &Device {
        &self.device
    }
    /// get device memory propertiess
    pub fn get_mem_properties(&self) -> vk::PhysicalDeviceMemoryProperties {
        unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device)
        }
    }
    /// Return the maximim sample count supported.
    pub fn get_max_usable_sample_count(&self) -> vk::SampleCountFlags {
        let props = unsafe {
            self.instance
                .get_physical_device_properties(self.physical_device)
        };
        let color_sample_counts = props.limits.framebuffer_color_sample_counts;
        let depth_sample_counts = props.limits.framebuffer_depth_sample_counts;
        let sample_counts = color_sample_counts.min(depth_sample_counts);

        if sample_counts.contains(vk::SampleCountFlags::TYPE_64) {
            vk::SampleCountFlags::TYPE_64
        } else if sample_counts.contains(vk::SampleCountFlags::TYPE_32) {
            vk::SampleCountFlags::TYPE_32
        } else if sample_counts.contains(vk::SampleCountFlags::TYPE_16) {
            vk::SampleCountFlags::TYPE_16
        } else if sample_counts.contains(vk::SampleCountFlags::TYPE_8) {
            vk::SampleCountFlags::TYPE_8
        } else if sample_counts.contains(vk::SampleCountFlags::TYPE_4) {
            vk::SampleCountFlags::TYPE_4
        } else if sample_counts.contains(vk::SampleCountFlags::TYPE_2) {
            vk::SampleCountFlags::TYPE_2
        } else {
            vk::SampleCountFlags::TYPE_1
        }
    }

}

impl Drop for VkContext {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.surface.destroy_surface(self.surface_khr, None);
            if let Some((report, callback)) = self.debug_report_callback.take() {
                report.destroy_debug_report_callback(callback, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}