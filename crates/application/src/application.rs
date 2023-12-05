
enum GameEvent {
    InputEvent,
    UpdateEvent,
    RenderEvent
}

pub struct EventHandler {
    input_subscribers: Vec<Box<dyn Fn(&GameEvent) -> ()>>,
    update_subscribers: Vec<Box<dyn Fn(&GameEvent) -> ()>>, 
    render_subscribers: Vec<Box<dyn Fn() -> ()>>,
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

    fn dispatch_render_event(&self, _event: &GameEvent) {
        &self.render_subscribers
            .iter()
            .for_each(|subscriber: &Box<dyn Fn()>| { subscriber()});
    }
}
use lynch::{window::window::Window, renderer::Renderer};
use lynch::{Camera};

use lynch::vulkan::renderer::VulkanRenderer;
use ash::vk;
use glam::{Vec3, Mat4};
use winit::
    event::{Event, WindowEvent}
;
use winit::event_loop::EventLoop;
const WIDTH : f64= 1280.;
const HEIGHT : f64 = 720.;
pub struct CooperApplication {
    window: Window,
    renderer: VulkanRenderer,
    graph: RenderGraph,
    event_handler: EventHandler,
    event_loop: EventLoop<()>,
    pub camera: Camera,
    view_data: lynch::ViewUniformData,
    camera_uniform_buffer: Vec<Buffer>,
}
impl CooperApplication {
    pub fn create() -> CooperApplication {
        let (window,event_loop) = Window::create("Cooper", WIDTH, HEIGHT);
        
        let renderer = VulkanRenderer::create(&window);


        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(6.0, 6.0, 6.0),
            60.0,
            WIDTH / HEIGHT,
            0.01,
            1000.0,
            0.20,
        );
        let view_data = lynch::ViewUniformData::new(&camera, WIDTH, HEIGHT);

        let camera_uniform_buffer = (0..renderer.image_count)
            .map(|_| { 
                view_data.create_camera_buffer(&renderer.vk_context)
            })
            .collect::<Vec<_>>();

            let graph = lynch::graph::Graph::new(renderer.vk_context.arc_device(), &camera_uniform_buffer, renderer.image_count);
        let event_handler = EventHandler::new();
        CooperApplication {
            window,
            renderer,
            event_handler,
            camera,
            graph,
            camera_uniform_buffer,
            event_loop,
        }
    }
    pub fn run(mut self) -> (){
        let mut cursor_position = None;
        let mut wheel_delta = None;
        let event_loop = self.window.event_loop;
        self.graph.recompile_all_shaders(self.renderer.device(), Some(self.renderer.internal_renderer.bindless_descriptor_set_layout));
        event_loop.run( move
            |event, _elwt|{
                match event{
                    
                    Event::WindowEvent {event, .. } => match event {
                        WindowEvent::RedrawRequested=> {
                            let present_index = self.renderer.begin_frame();
                            self.camera.update(&input);

                            self.view_data.view = self.camera.get_view();
                            self.view_data.projection = self.camera.get_projection();
                            self.view_data.inverse_view = self.camera.get_view().inverse();
                            self.view_data.inverse_projection = self.camera.get_projection().inverse();
                            self.view_data.eye_pos = self.camera.get_position();

                            self.renderer.submit_commands(self.graph.current_frame);
                            self.renderer.present_images[present_index].current_layout = vk::ImageLayout::PRESENT_SRC_KHR; 
                            self.renderer.present_frame(present_index, self.graph.current_frame);
                            self.renderer.current_frame = (self.renderer.current_frame + 1 ) % self.renderer.num_frames_in_flight as usize;
                            self.renderer.internal_renderer.need_environment_map_update = false;
                            self.graph.current_frame = self.renderer.current_frame;
                        }
                        WindowEvent::CloseRequested => {
                            print!("close requested");
                            
                            _elwt.exit();
                            },
                        WindowEvent::Resized(PhysicalSize { width: _, height: _ }) => {
                            //resize_dimensions = Some([width as u32, height as u32]);
                        }
                        WindowEvent::MouseInput {
                            button: MouseButton::Left,
                            state: _,
                            ..
                        } => {
                            //if state == ElementState::Pressed {
                            //    is_left_clicked = Some(true);
                            //} else {
                            //    is_left_clicked = Some(false);
                            //}
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
                    Event::LoopExiting => self.renderer.wait_gpu_idle(),
                    _ => {}
                }
            }
        ).unwrap()
    }
}