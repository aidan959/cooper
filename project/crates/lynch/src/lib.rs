mod camera;
mod constants;
pub mod gltf_loader;
pub mod mesh;
pub mod render_graph;
pub mod render_tools;
pub mod renderer;
mod texture;
pub mod vulkan;
pub mod window;
pub use camera::Camera;
pub use texture::Texture;
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
