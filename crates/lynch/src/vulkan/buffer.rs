use std::{mem, ptr::copy_nonoverlapping, sync::Arc};

use super::{device::Device, Image};
use ash::vk::{self, BufferCopy};
use gpu_allocator::{vulkan::*, MemoryLocation};
use log::info;
pub struct Buffer {
    pub vk_buffer: vk::Buffer,
    pub allocation: Allocation,
    pub memory_req: vk::MemoryRequirements,
    pub memory_location: MemoryLocation,
    pub size: u64,
    pub debug_name: String,
    device: Arc<Device>,
}

impl Buffer {
    pub fn create_buffer(
        device: Arc<Device>,
        size: u64, // TODO: infer size from data
        usage_flags: vk::BufferUsageFlags,
        memory_location: MemoryLocation,
        debug_name: Option<String>,
    ) -> Buffer {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage_flags)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        
        unsafe {
            let buffer = device
            .ash_device
            .create_buffer(&buffer_info, None)
            .expect("Failed to create buffer");
    
            let buffer_memory_req = device.ash_device.get_buffer_memory_requirements(buffer);

            let allocation = device
                .gpu_allocator
                .lock()
                .unwrap()
                .allocate(&AllocationCreateDesc {
                    name: "Example allocation",
                    requirements: buffer_memory_req,
                    location: memory_location,
                    linear: true,
                    allocation_scheme: AllocationScheme::GpuAllocatorManaged,
                })
                .unwrap();

            device
                .ash_device
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .unwrap();

            Buffer {
                vk_buffer: buffer,
                allocation,
                memory_req: buffer_memory_req,
                memory_location,
                size,
                debug_name: debug_name.unwrap_or_else(|| String::from("un_buffer")),
                device,
            }
        }
    }

    pub fn new<T: Copy>(
        device: Arc<Device>,
        initial_data: Option<&[T]>,
        size: u64,
        usage_flags: vk::BufferUsageFlags,
        location: MemoryLocation,
        debug_name: Option<String>,
    ) -> Buffer {
        let mut buffer = Buffer::create_buffer(
            device.clone(),
            size,
            usage_flags | vk::BufferUsageFlags::TRANSFER_DST,
            location,
            debug_name.clone(),
        );

        if let Some(initial_data) = initial_data {
            buffer.update_memory(initial_data);
        }
        let debug_name = match debug_name.as_deref() {
            Some(value) => value,
            None => "unnamed_buffer",
        };
        buffer.set_debug_name(&debug_name);
        buffer
    }
    /// WARN THIS IS EXPENSIVE DO NOT USE EVERY FRAME
    pub fn update_memory<T: Copy>(&mut self, data: &[T]) {
        unsafe {
            let src = data.as_ptr() as *const u8;
            let src_bytes = data.len() * mem::size_of::<T>();

            if self.memory_location != MemoryLocation::GpuOnly {
                let dst = self.allocation.mapped_ptr().unwrap().as_ptr() as *mut u8;
                let dst_bytes = self.allocation.size() as usize;
                copy_nonoverlapping(src, dst, std::cmp::min(src_bytes, dst_bytes));
            } else {
                info!(
                    "Creating staging buffer {}",
                    format!("staging_buffer_{:?}", src)
                );
                let staging_buffer = Buffer::create_buffer(
                    self.device.clone(),
                    self.size,
                    vk::BufferUsageFlags::TRANSFER_SRC,
                    MemoryLocation::CpuToGpu,
                    Some(String::from(format!("staging_buffer_{:?}", src))),
                );
                let dst = staging_buffer.allocation.mapped_ptr().unwrap().as_ptr() as *mut u8;
                let dst_bytes = staging_buffer.allocation.size() as usize;
                copy_nonoverlapping(src, dst, std::cmp::min(src_bytes, dst_bytes));

                self.device.execute_and_submit(|cb| {
                    let regions = BufferCopy::builder()
                        .size(self.size)
                        .dst_offset(0)
                        .src_offset(0)
                        .build();

                    self.device.ash_device.cmd_copy_buffer(
                        cb,
                        staging_buffer.vk_buffer,
                        self.vk_buffer,
                        &[regions],
                    );
                });

                self.device
                    .gpu_allocator
                    .lock()
                    .unwrap()
                    .free(staging_buffer.allocation)
                    .unwrap();
                self.device
                    .ash_device
                    .destroy_buffer(staging_buffer.vk_buffer, None);
            }
        }
    }
    pub fn copy_to_buffer(&self, cb: vk::CommandBuffer, dst: &Buffer) {
        let buffer_copy_regions = BufferCopy::builder()
            .size(self.size)
            .src_offset(0)
            .dst_offset(0)
            .build();

        unsafe {
            self.device.ash_device.cmd_copy_buffer(
                cb,
                self.vk_buffer,
                dst.vk_buffer,
                &[buffer_copy_regions],
            );
        }
    }

    pub fn copy_to_image(&self, cb: vk::CommandBuffer, image: &Image) {
        let buffer_copy_regions = vk::BufferImageCopy::builder()
            .image_subresource(
                vk::ImageSubresourceLayers::builder()
                    .aspect_mask(image.desc.aspect_flags)
                    .layer_count(1)
                    .build(),
            )
            .image_extent(vk::Extent3D {
                width: image.width(),
                height: image.height(),
                depth: 1,
            });

        unsafe {
            self.device.ash_device.cmd_copy_buffer_to_image(
                cb,
                self.vk_buffer,
                image.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[buffer_copy_regions.build()],
            );
        }
    }

    pub fn get_device_address(&self, device: &Device) -> vk::DeviceAddress {
        let info = vk::BufferDeviceAddressInfo::builder()
            .buffer(self.vk_buffer)
            .build();

        unsafe { device.ash_device.get_buffer_device_address(&info) }
    }

    pub fn set_debug_name(&mut self, name: &str) {
        self.debug_name = String::from(name);
        self.device.set_debug_name(
            vk::Handle::as_raw(self.vk_buffer),
            vk::ObjectType::BUFFER,
            name,
        );
    }
    pub fn clean_vk_resources(&self) {
        unsafe { self.device.ash_device.destroy_buffer(self.vk_buffer, None) }
    }
}
