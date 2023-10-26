use winit::{
    dpi::PhysicalSize,
    event::{Event},
    event_loop::{EventLoop},
    window::{WindowBuilder, Window as WinitWindow},
};
// use super::windows::process_event_windows;
type EventCallback = fn(Event<()>) -> ();
pub struct Window{
    pub window : WinitWindow,
    pub event_loop : EventLoop<()>,
    window_title : String,
    width : f64,
    height : f64,
    cursor_delta: Option<[i32; 2]>,
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
    fn create_event_loop() -> EventLoop<()> {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop
    }
    fn create_window(window_title: &str, width: f64, height: f64, event_loop: &EventLoop<()>) -> WinitWindow{
        WindowBuilder::new()
        .with_title(window_title)
        .with_inner_size(PhysicalSize::new(width, height))
        .build(event_loop)
        .unwrap()
    }

}