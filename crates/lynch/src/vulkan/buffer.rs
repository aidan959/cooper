use std::sync::Arc;

use super::{device::Device, Image};
use ash::vk;
use gpu_allocator::vulkan::*;
use log::info;
pub struct Buffer {
    pub buffer: vk::Buffer,
    pub allocation: Allocation,
    pub memory_req: vk::MemoryRequirements,
    pub memory_location: gpu_allocator::MemoryLocation,
    pub size: u64,
    pub debug_name: String,
    device: Arc<Device>,
}

impl Buffer {
    pub fn new<T: Copy>(
        device: Arc<Device>,
        initial_data: Option<&[T]>,
        size: u64,
        usage_flags: vk::BufferUsageFlags,
        location: gpu_allocator::MemoryLocation,
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
            None => "unnamed_buffer", // smh lazy work from you
        };
        buffer.set_debug_name(&debug_name);
        buffer
    }
    pub fn create_buffer(
        device: Arc<Device>,
        // TODO: infer
        size: u64,
        usage_flags: vk::BufferUsageFlags,
        memory_location: gpu_allocator::MemoryLocation,
        debug_name: Option<String>,
    ) -> Buffer {
        unsafe {
            let buffer_info = vk::BufferCreateInfo::builder()
                .size(size)
                .usage(usage_flags)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

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
                })
                .unwrap();

            device
                .ash_device
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .unwrap();

            Buffer {
                buffer,
                allocation,
                memory_req: buffer_memory_req,
                memory_location,
                size,
                debug_name: debug_name.unwrap_or_else(|| String::from("un_buffer")),
                device,
            }
        }
    }
    pub fn update_memory<T: Copy>(&mut self, data: &[T]) {
        unsafe {
            let src = data.as_ptr() as *const u8;
            let src_bytes = data.len() * std::mem::size_of::<T>();

            if self.memory_location != gpu_allocator::MemoryLocation::GpuOnly {
                let dst = self.allocation.mapped_ptr().unwrap().as_ptr() as *mut u8;
                let dst_bytes = self.allocation.size() as usize;
                std::ptr::copy_nonoverlapping(src, dst, std::cmp::min(src_bytes, dst_bytes));
            } else {
                info!(
                    "Creating staging buffer {}",
                    format!("staging_buffer_{:?}", src)
                );
                let staging_buffer = Buffer::create_buffer(
                    self.device.clone(),
                    self.size,
                    vk::BufferUsageFlags::TRANSFER_SRC,
                    gpu_allocator::MemoryLocation::CpuToGpu,
                    Some(String::from(format!("staging_buffer_{:?}", src))),
                );
                let dst = staging_buffer.allocation.mapped_ptr().unwrap().as_ptr() as *mut u8;
                let dst_bytes = staging_buffer.allocation.size() as usize;
                std::ptr::copy_nonoverlapping(src, dst, std::cmp::min(src_bytes, dst_bytes));

                self.device.execute_and_submit(|cb| {
                    let regions = vk::BufferCopy::builder()
                        .size(self.size)
                        .dst_offset(0)
                        .src_offset(0)
                        .build();

                    self.device.ash_device.cmd_copy_buffer(
                        cb,
                        staging_buffer.buffer,
                        self.buffer,
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
                    .destroy_buffer(staging_buffer.buffer, None);
            }
        }
    }
}
fn create_device_local_buffer_with_data<A, T: Copy>(
    vk_context: &VkContext,
    command_pool: vk::CommandPool,
    transfer_queue: vk::Queue,
    usage: vk::BufferUsageFlags,
    data: &[T],
) -> (vk::Buffer, vk::DeviceMemory) {
    let size = (data.len() * std::mem::size_of::<T>()) as vk::DeviceSize;
    let (staging_buffer, staging_memory, staging_mem_size) = Self::create_buffer(
        vk_context,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );
    unsafe {
        let data_ptr = vk_context
            .device()
            .map_memory(staging_memory, 0, size, vk::MemoryMapFlags::empty())
            .unwrap();
        let mut align =
            ash::util::Align::new(data_ptr, std::mem::align_of::<A>() as _, staging_mem_size);
        align.copy_from_slice(data);

        vk_context.device().unmap_memory(staging_memory);
    };

    let (buffer, memory, _) = Self::create_buffer(
        vk_context,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | usage,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );
    Self::copy_buffer(
        vk_context.device(),
        command_pool,
        transfer_queue,
        staging_buffer,
        buffer,
        size,
    );
    unsafe {
        vk_context.device().destroy_buffer(staging_buffer, None);
        vk_context.device().free_memory(staging_memory, None);
    };
    (buffer, memory)
}