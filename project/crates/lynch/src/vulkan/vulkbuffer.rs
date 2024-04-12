use ash::{vk, Device, Entry, Instance};


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