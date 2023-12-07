mod constants;
mod camera;
pub mod mesh;
mod texture;
pub mod window;
pub mod vulkan;
pub mod renderer;
pub mod render_graph;
pub mod gltf_loader;
pub mod render_tools;
pub use texture::Texture;
pub use camera::Camera;
pub use vulkan::renderer::ViewUniformData;
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(add(2, 2), 4);
    }
}
