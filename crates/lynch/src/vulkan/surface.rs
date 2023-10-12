use ash::{vk, Entry, Instance};
use ash::extensions::khr::{Surface, Win32Surface};
use winit::raw_window_handle::{HasWindowHandle};
use std::{os::raw::c_void, ptr};
use winapi::{shared::windef::HWND, um::libloaderapi::GetModuleHandleW};
use winit::{ window::Window};

/// Get required instance extensions.
/// This is windows specific.
pub fn required_extension_names() -> Vec<*const i8> {
    vec![Surface::name().as_ptr(), Win32Surface::name().as_ptr()]
}

/// Create the surface.
pub unsafe fn create_surface(
    entry: &Entry,
    instance: &Instance,
    window: &Window,
) -> Result<vk::SurfaceKHR, vk::Result> {
    let a = window.window_handle().unwrap().as_raw();
    match a {
        winit::raw_window_handle::RawWindowHandle::Win32(raw_win32) => {
            let hwnd = raw_win32.hwnd.get() as HWND;
            let hinstance = GetModuleHandleW(ptr::null()) as *const c_void;
            let win32_create_info = vk::Win32SurfaceCreateInfoKHR {
                s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
                p_next: ptr::null(),
                flags: Default::default(),
                hinstance,
                hwnd: hwnd as *const c_void,
            };
            let win32_surface_loader = Win32Surface::new(entry, instance);
            return win32_surface_loader.create_win32_surface(&win32_create_info, None);
        },
        winit::raw_window_handle::RawWindowHandle::UiKit(_) => todo!("UiKit is Unimplmeneted"),
        winit::raw_window_handle::RawWindowHandle::AppKit(_) => todo!("AppKit is Unimplmeneted"),
        winit::raw_window_handle::RawWindowHandle::Orbital(_) => todo!("Orbital is Unimplmeneted"),
        winit::raw_window_handle::RawWindowHandle::Xlib(_) => todo!("Xlib is Unimplmeneted"),
        winit::raw_window_handle::RawWindowHandle::Xcb(_) => todo!("Xcb is Unimplmeneted"),
        winit::raw_window_handle::RawWindowHandle::Wayland(_) => todo!("Wayland is Unimplmeneted"),
        winit::raw_window_handle::RawWindowHandle::Drm(_) => todo!("Drm is Unimplmeneted"),
        winit::raw_window_handle::RawWindowHandle::Gbm(_) => todo!("Gbm is Unimplmeneted"),
        winit::raw_window_handle::RawWindowHandle::WinRt(_) => todo!("WinRt is Unimplmeneted"),
        winit::raw_window_handle::RawWindowHandle::Web(_) => todo!("Web is Unimplmeneted"),
        winit::raw_window_handle::RawWindowHandle::WebCanvas(_) => todo!("WebCanvas is Unimplmeneted"),
        winit::raw_window_handle::RawWindowHandle::WebOffscreenCanvas(_) => todo!("WebOffscreenCanvas is Unimplmeneted"),
        winit::raw_window_handle::RawWindowHandle::AndroidNdk(_) => todo!("AndroidNdk is Unimplmeneted"),
        winit::raw_window_handle::RawWindowHandle::Haiku(_) => todo!("Haiku is Unimplmeneted"),
        _ => panic!("Unsupported operating system used to create window."),
    };
    
}