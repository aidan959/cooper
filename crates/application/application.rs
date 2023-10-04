
enum GameEvent {
    InputEvent,
    UpdateEvent,
    RenderEvent
}
impl EventHandler {
    fn new() -> Self {
        EventHandler {
            input_subscribers: Vec::new(),
            update_subscribers: Vec::new(),
            render_subscribers: Vec::new(),
            // ...
        }
    }

    fn subscribe_input_event(&mut self, callback: Box<dyn Fn(&GameEvent) -> ()>) {
        self.input_subscribers.push(callback);
    }

    fn subscribe_update_event(&mut self, callback: Box<dyn Fn(&GameEvent) -> ()>) {
        self.update_subscribers.push(callback);
    }

    fn subscribe_render_event(&mut self, callback: Box<dyn Fn() -> ()>) {
        self.render_subscribers.push(callback);
    }

    fn dispatch_input_event(&self, event: &GameEvent) {
        &self.input_subscribers
            .iter()
            .for_each(|subscriber: &Box<dyn Fn(&GameEvent)>| { subscriber(&event)});
    }

    fn dispatch_update_event(&self, event: &GameEvent) {
        &self.update_subscribers
            .iter()
            .for_each(|subscriber: &Box<dyn Fn(&GameEvent)>| { subscriber(&event)});
    }

    fn dispatch_render_event(&self, event: &GameEvent) {
        &self.render_subscribers
            .iter()
            .for_each(|subscriber: &Box<dyn Fn()>| { subscriber()});
    }
}
pub struct EventHandler {
    input_subscribers: Vec<Box<dyn Fn(&GameEvent) -> ()>>,
    update_subscribers: Vec<Box<dyn Fn(&GameEvent) -> ()>>, 
    render_subscribers: Vec<Box<dyn Fn() -> ()>>,
}

use lynch::{window::window::Window, renderer::Renderer};

pub struct CooperApplication {
    window: Window,
    renderer: Box<dyn Renderer>,
    event_handler: EventHandler
}

impl CooperApplication {
    pub fn create() -> Self {
        let window = Window::create("Cooper", 1280., 720.);
        
        Self {
            window,
            todo!(),
            todo!()
        }
    }

    pub fn run(mut self) -> () {
        let mut cursor_position  = None;
        let mut wheel_delta = None;

        let event_loop = self.window.event_loop;
        

        event_loop.run( move
            |event, elwt|{
                match event{
                    Event::WindowEvent {event, .. } => match event {
                        WindowEvent::RedrawRequested=> {
                            println!("Redraw Requested");
                            self.renderer.draw_frame();
                        }
                        WindowEvent::CloseRequested => {return},
                        WindowEvent::Resized(PhysicalSize { width, height }) => {
                            //resize_dimensions = Some([width as u32, height as u32]);
                        }
                        WindowEvent::MouseInput {
                            button: MouseButton::Left,
                            state,
                            ..
                        } => {

                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            let position: (i32, i32) = position.into();
                            cursor_position = Some([position.0, position.1]);
                        }
                        WindowEvent::MouseWheel {
                            delta: MouseScrollDelta::LineDelta(_, v_lines),
                            ..
                        } => {
                            wheel_delta = Some(v_lines);
                        }
                    //
                        _ => {}
                    },
                    _ => {}
                }
            }
        ).unwrap()
    }
}