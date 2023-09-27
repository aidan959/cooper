use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window as WinitWindow},
};

pub struct Window{
    pub window : WinitWindow,
    pub event_loop : EventLoop<()>,
    window_title : String,
    width : f64,
    height : f64,
}

impl Window {

    pub fn create(window_title: &str, width: f64, height: f64) -> Self{
        let event_loop = Self::create_event_loop();
        let window =  Self::create_window(&window_title, width, height, &event_loop);
        Window{
            window,
            event_loop,
            window_title: String::from(window_title),
            width,
            height,
            cursor_delta: None
        }
    }

    fn create_window(window_title: &str, width: f64, height: f64, event_loop: &EventLoop<()>) -> WinitWindow{
        WindowBuilder::new()
        .with_title(window_title)
        .with_inner_size(PhysicalSize::new(width, height))
        .build(event_loop)
        .unwrap()
    }

}