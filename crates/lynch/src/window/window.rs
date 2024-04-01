use winit::{
    dpi::PhysicalSize, event::Event, event_loop::{EventLoop, EventLoopBuilder}, platform::windows::EventLoopBuilderExtWindows, window::{Window as WinitWindow, WindowBuilder}
};

pub struct Window {
    pub window: WinitWindow,
}
pub type WindowSize = (f64, f64);
impl Window {
    pub fn create(window_title: &str, window_size: WindowSize) -> (Self, EventLoop<()>) {
        let event_loop = Self::create_event_loop();
        let window = Self::create_window(&window_title, window_size, &event_loop);
        (
            Window {
                window,
            },
            event_loop,
        )
    }
    fn create_event_loop() -> EventLoop<()> {
        let event_loop = EventLoopBuilder::new()
            .with_any_thread(true)
            .build();
        event_loop
    }
    fn create_window(
        window_title: &str,
        window_size: WindowSize,
        event_loop: &EventLoop<()>,
    ) -> WinitWindow {
        WindowBuilder::new()
            .with_title(window_title)
            .with_inner_size(PhysicalSize::new(window_size.0, window_size.1))
            .build(event_loop)
            .unwrap()
    }
}

