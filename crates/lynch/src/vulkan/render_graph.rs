use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::Arc;

use ash::vk::{self};
use crate::renderer::Renderer;
use crate::vulkan::renderer::{VulkanRenderer};


pub type TextureId = usize;
pub type BufferId = usize;
pub type PipelineId = usize;

pub struct GraphTexture {
    pub texture: Texture,
    pub prev_access: vk_sync::AccessType,
}
pub struct GraphBuffer {
    pub buffer: Buffer,
    pub prev_access: vk_sync::AccessType,
}


// TODO These need to be destroyed on release
pub struct GraphResources {
    pub buffers: Vec<GraphBuffer>,
    pub textures: Vec<GraphTexture>,
    pub pipelines: Vec<Pipeline>,
}

pub enum DepthAttachment {
    GraphHandle(Attachment),
    External(Image, vk::AttachmentLoadOp),
}


#[derive(Copy, Clone)]
pub struct Attachment {
    pub texture: TextureId,
    pub view: ViewType,
    pub load_op: vk::AttachmentLoadOp,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TextureResourceType {
    CombinedImageSampler,
    StorageImage,
}

#[derive(Copy, Clone)]
pub struct TextureResource {
    pub texture: TextureId,
    pub input_type: TextureResourceType,
    pub access_type: vk_sync::AccessType,
}

#[derive(Copy, Clone)]
pub struct BufferResource {
    pub buffer: BufferId,
    pub access_type: vk_sync::AccessType,
}

#[derive(Copy, Clone)]
pub enum Resource {
    Texture(TextureResource),
    Buffer(BufferResource),
}
pub struct TextureCopy {
    pub src: TextureId,
    pub dst: TextureId,
    pub copy_desc: vk::ImageCopy,
}