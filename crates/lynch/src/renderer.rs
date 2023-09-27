use crate::window::window::Window;

pub trait Renderer {
    fn create(window: &Window) -> Self
    where
        Self: Sized;
    fn draw_frame(self : &mut Self);
}