use crate::window::window::Window;
pub trait Renderer {
    fn create(window: &Window) -> Self
    where
        Self: Sized;
    fn draw_frame(self : &mut Self, frame_count: u32);
    fn wait_gpu_idle(self:&Self);
    fn resize(self: &mut Self, resize: winit::dpi::PhysicalSize<u32>);
}