use std::fmt::{Display, Formatter, Debug};
use winit::{
    dpi::PhysicalSize,
    event_loop::{EventLoop, EventLoopBuilder},
    platform::windows::EventLoopBuilderExtWindows,
    window::Window as WinitWindow,
};
const DEFAULT_WINDOW_NAME: &'static str = "Lynch Window";
const DEFAULT_WINDOW_SIZE: WindowSize = (1280., 720.);
pub struct Window {
    pub window: WinitWindow,
}
pub type WindowSize = (f64, f64);
impl Window {
    pub fn create(window_title: &str, window_size: WindowSize) -> (Self, EventLoop<()>) {
        let event_loop = Self::create_event_loop();
        let window = Self::create_window(&window_title, window_size, &event_loop);
        (Window { window }, event_loop)
    }
    fn create_event_loop() -> EventLoop<()> {
        let event_loop = EventLoopBuilder::new().with_any_thread(true).build();
        event_loop
    }
    fn create_window(
        window_title: &str,
        window_size: WindowSize,
        event_loop: &EventLoop<()>,
    ) -> WinitWindow {
        winit::window::WindowBuilder::new()
            .with_title(window_title)
            .with_inner_size(PhysicalSize::new(window_size.0, window_size.1))
            .build(event_loop)
            .unwrap()
    }
    pub fn builder() -> WindowBuilder {
        WindowBuilder::new()
    }
}
pub struct WindowBuilder {
    window_title: &'static str,
    window_size: WindowSize,
}
impl WindowBuilder {
    fn new() -> Self {
        Self {
            window_title: DEFAULT_WINDOW_NAME,
            window_size: DEFAULT_WINDOW_SIZE,
        }
    }
    pub fn width<T>(mut self, width: T) -> Result<Self, WindowBuilderError<<T as TryInto<f64>>::Error>>
    where
        T: TryInto<f64> + Display + Copy,
        <T as TryInto<f64>>::Error:Debug,
    { 
        match width.try_into() {
            Ok (n) => self.window_size.0 = n,
            Err(e) => return Err(WindowBuilderError::CouldNotConvertF64(e))
        }
        Ok(self)
    }
    pub fn height<T>(mut self, width: T) -> Result<Self, WindowBuilderError<<T as TryInto<f64>>::Error>>
    where
        T: TryInto<f64> + Display + Copy,
        <T as TryInto<f64>>::Error:Debug,
    { 
        match width.try_into() {
            Ok (n) => self.window_size.1 = n,
            Err(e) => return Err(WindowBuilderError::CouldNotConvertF64(e))
        }
        Ok(self)
    }
    pub fn set_window_size<T>(mut self, width: T, height: T) -> Result<Self, WindowBuilderError<<T as TryInto<f64>>::Error>>
    where
        T: TryInto<f64> + Display + Copy,
        <T as TryInto<f64>>::Error:Debug,
    {
        
        let _width  = 
            match width.try_into() {
                Ok(n) => n,
                Err(e) => {
                    return Err(WindowBuilderError::CouldNotConvertF64(e))        
                }
            };
        let _height = 
        match height.try_into() {
            Ok(n) => n,
            Err(e) => {
                return Err(WindowBuilderError::CouldNotConvertF64(e))        
            }
        };
        self.window_size = (_width, _height); 
        Ok(self)
    }
    pub fn window_name(mut self, window_title: &'static str) -> Self {
        self.window_title = window_title;
        self
    }
    pub fn build(self) -> (Window, EventLoop<()>) {
        Window::create(self.window_title, self.window_size)
    }
}
#[derive(Debug)]
pub enum WindowBuilderError<E> 
{
    CouldNotConvertF64(E)

}
impl<E: Debug> Display for WindowBuilderError<E>
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            WindowBuilderError::CouldNotConvertF64(e) => write!(f, "Could not convert provided value {:?} to type.", e),
        }
    }
}

impl<E: Debug> std::error::Error for WindowBuilderError<E> {}