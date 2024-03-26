mod buffer;
mod cont;
mod debug;
mod descriptor;
mod device;
mod image;
mod pipeline;
mod render_pass;
pub mod shader;
mod surface;
mod swapchain;

pub mod renderer;
use ash::vk;
pub use buffer::Buffer;
pub use descriptor::{DescriptorIdentifier, DescriptorSet};
pub use device::Device;
pub use image::{Image, ImageCopyDescBuilder, ImageDesc, ImageType};
pub use pipeline::{Pipeline, PipelineDesc, PipelineDescBuilder, PipelineType};
pub use render_pass::RenderPass;

pub use device::{global_pipeline_barrier, image_pipeline_barrier};

use self::cont::VkContext;

pub(crate) fn create_render_pass_ui(
    device: &Device,
    format: vk::Format,

) -> vk::RenderPass {
    let attachment_descs = [vk::AttachmentDescription::builder()
        .format(format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::LOAD)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .build()];

    let color_attachment_refs = [vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build()];

    let subpass_descs = [vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_attachment_refs)
        .build()];

    let subpass_deps = [vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .build()];

    let render_pass_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachment_descs)
        .subpasses(&subpass_descs)
        .dependencies(&subpass_deps);
    unsafe { device.ash_device.create_render_pass(&render_pass_info, None).unwrap()}
}
pub(crate) fn create_present_render_pass(
    device: &Device,
    attachment_formats: Vec<vk::Format>,
) -> vk::RenderPass {
    log::debug!("Creating vulkan render pass");
    
    let color_attachment_descs = attachment_formats.iter().map(|format| {
        vk::AttachmentDescription::builder()
            .format(*format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build()
    }).collect::<Vec<_>>();



    let mut color_attachment_refs: Vec<vk::AttachmentReference> = Vec::new();
    for i  in 0..attachment_formats.len() {
        color_attachment_refs.push(vk::AttachmentReference::builder()
            .attachment(i as u32)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build());
    }

    let subpass = [vk::SubpassDescription::builder()
    .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
    .color_attachments(&color_attachment_refs)
    .build()];
    let subpass_deps = [vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .dependency_flags(vk::DependencyFlags::BY_REGION)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        ).build()];

    let render_pass_info = vk::RenderPassCreateInfo::builder()
        .attachments(&color_attachment_descs)
        .subpasses(&subpass)
        .dependencies(&subpass_deps);

    unsafe { device.ash_device.create_render_pass(&render_pass_info, None).unwrap() }
}

pub(crate) fn create_render_pass(
    device: &Device,
    attachment_formats: Vec<vk::Format>,
    depth_attachment_format: Option<vk::Format>,
) -> vk::RenderPass {
    log::debug!("Creating vulkan render pass");
    
    let mut color_attachment_descs = attachment_formats.iter().map(|format| {
        vk::AttachmentDescription::builder()
            .format(*format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()
    }).collect::<Vec<_>>();



    let mut color_attachment_refs: Vec<vk::AttachmentReference> = Vec::new();
    for i  in 0..attachment_formats.len() {
        color_attachment_refs.push(vk::AttachmentReference::builder()
            .attachment(i as u32)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build());
    }
    let subpass_descs = match depth_attachment_format {
        Some(depth_attachment) =>{
            let depth_attachment_desc = vk::AttachmentDescription::builder()
                .format(depth_attachment)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build();
                color_attachment_descs.extend_from_slice(&[depth_attachment_desc]);
            let dept_attachment_ref = vk::AttachmentReference::builder()
                .attachment(attachment_formats.len() as u32)
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build();
            [vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&color_attachment_refs)
                .depth_stencil_attachment(&dept_attachment_ref)
                .build()]

        }
        None => {
             [vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs)
            .build()]
        }
    };
    



    let subpass_deps = [vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .dependency_flags(vk::DependencyFlags::BY_REGION)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        ).build()];

    let render_pass_info = vk::RenderPassCreateInfo::builder()
        .attachments(&color_attachment_descs)
        .subpasses(&subpass_descs)
        .dependencies(&subpass_deps);

    unsafe { device.ash_device.create_render_pass(&render_pass_info, None).unwrap() }
}




pub(crate) fn create_vulkan_framebuffers(
    device: &Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    image_views: &Vec<vk::ImageView>,
) -> Vec<vk::Framebuffer> {
    log::debug!("Creating vulkan framebuffers");
    image_views
        .iter()
        .map(|view| [*view])
        .map(|attachments| {
            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);
            unsafe { device.ash_device.create_framebuffer(&framebuffer_info, None) }
        })
        .collect::<Result<Vec<_>, _>>().unwrap()
}

pub(crate) fn create_vulkan_framebuffer(
    device: &Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    image_views: &Vec<vk::ImageView>,
) -> vk::Framebuffer {
    log::debug!("Creating vulkan framebuffers");

    let framebuffer_info = vk::FramebufferCreateInfo::builder()
        .render_pass(render_pass)
        .attachments(&image_views)
        .width(extent.width)
        .height(extent.height)
        .layers(1)
        .build();
    unsafe {
        device.ash_device.create_framebuffer(&framebuffer_info, None)
            .expect("Failed to create framebuffer")
    }


}