use std::sync::Arc;

use ash::{extensions::khr::Surface, vk, Entry, Instance};

use super::Device;

pub struct VkContext {
    _entry: Entry,
    instance: Instance,
    surface: Surface,
    surface_khr: vk::SurfaceKHR,
    device: Arc<Device>,
}

impl VkContext {
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    pub fn surface_khr(&self) -> vk::SurfaceKHR {
        self.surface_khr
    }

    pub fn ash_device(&self) -> &ash::Device {
        &self.device.ash_device
    }

    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.device.physical_device
    }
    pub fn device(&self) -> &Device {
        &self.device
    }
    pub fn arc_device(&self) -> Arc<Device> {
        self.device.clone()
    }
}

impl VkContext {
    pub fn get_mem_properties(&self) -> vk::PhysicalDeviceMemoryProperties {
        unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device())
        }
    }

    pub fn find_supported_format(
        &self,
        candidates: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> Option<vk::Format> {
        candidates.iter().cloned().find(|candidate| {
            let props = unsafe {
                self.instance
                    .get_physical_device_format_properties(self.physical_device(), *candidate)
            };
            (tiling == vk::ImageTiling::LINEAR && props.linear_tiling_features.contains(features))
                || (tiling == vk::ImageTiling::OPTIMAL
                    && props.optimal_tiling_features.contains(features))
        })
    }

    pub fn get_max_usable_sample_count(&self) -> vk::SampleCountFlags {
        let props = unsafe {
            self.instance
                .get_physical_device_properties(self.physical_device())
        };
        let color_sample_counts = props.limits.framebuffer_color_sample_counts;
        let depth_sample_counts = props.limits.framebuffer_depth_sample_counts;
        let sample_counts = color_sample_counts.min(depth_sample_counts);
        match sample_counts {
            vk::SampleCountFlags::TYPE_64
            | vk::SampleCountFlags::TYPE_32
            | vk::SampleCountFlags::TYPE_16
            | vk::SampleCountFlags::TYPE_8
            | vk::SampleCountFlags::TYPE_4
            | vk::SampleCountFlags::TYPE_2 => sample_counts,
            _ => vk::SampleCountFlags::TYPE_1,
        }
    }
}

impl VkContext {
    pub fn new(
        entry: Entry,
        instance: Instance,
        surface: Surface,
        surface_khr: vk::SurfaceKHR,
        device: Device,
    ) -> Self {
        VkContext {
            _entry: entry,
            instance,
            surface,
            surface_khr,
            device: Arc::new(device),
        }
    }
}

impl Drop for VkContext {
    fn drop(&mut self) {
        unsafe {
            self.surface.destroy_surface(self.surface_khr, None);
            self.instance.destroy_instance(None);
        }
    }
}
