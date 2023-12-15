use crate::{window::window::Window, vulkan::Device, Camera};
pub trait Renderer {
    fn create(window: &Window, camera: &Camera) -> Self
    where
        Self: Sized;
    fn begin_frame(self: &mut Self) -> usize;
    fn update_view_to_camera(self: &mut Self, camera: &Camera);
    fn end_frame(self: &mut Self);
    fn add_model(self: &mut Self,  model: crate::gltf_loader::Model, transform: glam::Mat4);
    fn draw_meshes(self: &Self);
    fn draw_frame(self : &mut Self, frame_count: u32);
    fn wait_gpu_idle(self:&Self);
    fn resize(self: &mut Self, resize: winit::dpi::PhysicalSize<u32>);
    fn initialize(self: &mut Self);
    fn device(self:& Self) -> &Device;
}