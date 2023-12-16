use frost::Input;
use lynch::render_graph::RenderGraph;
use lynch::{window::window::Window, renderer::Renderer, vulkan::renderer::VulkanRenderer, Camera};
use glam::{Vec3, Mat4};
use winit::
    event::{Event, WindowEvent}
;
use winit::event_loop::EventLoop;
use log::{debug, info};
use std::sync::mpsc::{self, Sender};
use std::time::{Instant, Duration};

use crate::{EngineSettings, EngineSettingsBuilder, DEFAULT_FPS_CAP, DEFAULT_UPDATE_RATE, DEFAULT_MAX_FPS};
use crate::engine_callbacks::{EngineCallbacks, GameCallbacks};

pub struct CooperApplication
{
    window: Window,
    pub renderer: VulkanRenderer,
    graph: RenderGraph,
    pub camera: Camera,
    event_loop: EventLoop<()>,
    engine_settings: EngineSettings
}
const WIDTH : f64= 1280.;
const HEIGHT : f64 = 720.;
pub enum GameEvent {
    Input,
    MoveEvent(usize, Mat4),
    Spawn(String),
    NextFrame
}

impl CooperApplication
{
    pub fn create() -> Self {
        let (window,event_loop) = Window::create("Cooper", WIDTH, HEIGHT);
        let fov_degrees = 90.0;
        let camera = Camera::new(
            Vec3::default(),
            Vec3::default(),
            fov_degrees,
            WIDTH / HEIGHT,
            0.01,
            1000.0,
            0.20,
        );
        let renderer = VulkanRenderer::create(&window, &camera);
        let graph = RenderGraph::new(renderer.vk_context.arc_device(), &renderer.camera_uniform_buffer, renderer.image_count);
        let engine_settings = EngineSettingsBuilder::new()
                                .fps_cap(Some(DEFAULT_MAX_FPS))
                                .update_rate_hz(DEFAULT_UPDATE_RATE)
                                .build();
        CooperApplication {
            window,
            renderer,
            graph,
            camera,
            event_loop,
            engine_settings
        }
    }
    
    pub fn run<E, F, G, H>(mut self: Self, mut start: E, mut update: F, mut fixed_update: G, mut finally: H)
    where
        E: FnMut( &Sender<GameEvent>),
        F: FnMut( &Sender<GameEvent>, f32),
        G: FnMut( &Sender<GameEvent>, f32),
        H: FnMut( &Sender<GameEvent>),
    {
        let mut frame_count = 0;
        let mut last_fps_time = Instant::now();
        let fps_update_interval = Duration::new(1, 0); // 1 second
        self.create_scene();
        let mut input : Input = Input::default(); 
        let _events : Vec<WindowEvent> = Vec::new();
        let (event_trasmitter,event_receiver) = mpsc::channel();
        start(&event_trasmitter.clone());
        let update_transmitter = event_trasmitter.clone();
        let fixed_update_transmitter = event_trasmitter.clone();
        let finally_transmitter = event_trasmitter.clone();

        //let cube_hash_map : HashMap<str, usize>  = HashMap::default();  
        let mut last_fixed_update = Instant::now();
        let mut lag = 0.0;

        self.event_loop.run( 
            |event, _elwt|{
                match event{
                    Event::WindowEvent {event, .. } => match event {
                        WindowEvent::RedrawRequested=> {
                            let delta = self.renderer.render(&mut self.graph, &self.camera);
                            let current_time = Instant::now();
                            let elapsed = current_time.duration_since(last_fixed_update);
                            last_fixed_update = current_time;
                            lag += elapsed.as_secs_f32();
                            // user update call
                            update( &update_transmitter, delta);
                            // submit input data to camera
                            self.camera.update(&input,delta);
                            
                            // call fixed_update fixed_update_rate times per second
                            while lag >= self.engine_settings.fixed_update_rate.as_secs_f32() {
                                // user fixed update call
                                fixed_update(&fixed_update_transmitter, self.engine_settings.fixed_update_rate.as_secs_f32());
                                lag -= self.engine_settings.fixed_update_rate.as_secs_f32();
                            }

                            input.reset_mouse();
                            // last user definable call
                            finally(&finally_transmitter);
                            loop{
                                let event = event_receiver.recv();
                                match event {
                                    Ok(event) => {
                                        match event {
                                            GameEvent::Input=>{
                                                // TODO DO SOMETHING
                                            },
                                            GameEvent::MoveEvent(instance, matrix)=>{
                                                // TODO DO SOMETHING
                                                self.renderer.internal_renderer.instances[instance].transform = matrix;
                                            },
                                            GameEvent::Spawn(_path) =>{
                                                // TODO handle spawn event
                                            }
                                            GameEvent::NextFrame=>{
                                                // MARK FRAME COMPLETE
                                                break
                                            },
                                        }
                                    },
                                    Err(_err) => {},
                                }
                            }

                            frame_count += 1;
                            // print frame per second (every second!)
                            if last_fps_time.elapsed() >= fps_update_interval {
                                println!("FPS: {}", frame_count);
                                frame_count = 0;
                                last_fps_time = Instant::now();
                            }
                            // apply fps limit //TODO explore why this is limiting the fps to HALF the rate (? how)
                            if self.engine_settings.fps_settings.limit {
                                let elapsed = last_fixed_update.elapsed();
                                if elapsed < self.engine_settings.fps_settings.frame_time {
                                    std::thread::sleep(self.engine_settings.fps_settings.frame_time - elapsed);
                                }
                            }  
                        },
                        WindowEvent::CloseRequested => {
                            self.graph.clear();
                            _elwt.exit();
                        },
                        WindowEvent::Resized(resize_value) => {
                            self.renderer.resize(resize_value);
                        },
                        WindowEvent::MouseInput {..} | WindowEvent::CursorMoved {..}| WindowEvent::KeyboardInput {..}| WindowEvent::MouseWheel {..} => {
                            input.update(&event);
                        },
                        _ => { 
                        }
                    },
                    Event::LoopExiting => 
                        info!("Main program loop exiting."),
                    Event::AboutToWait => {
                        self.window.window.request_redraw();
                    },
                    _ => {}
                }                
            }
        ).unwrap()
    }

    fn create_scene(&mut self) {
        self.renderer.initialize();
        self.build_scene();
    }
    fn build_scene(&mut self ) {
        
        let sphere = self.renderer.load_model("models/cube.gltf");
        let translation = Mat4::default();
        self.renderer.add_model(
            sphere,
            translation
        );
        let sphere = self.renderer.load_model("models/sphere.gltf");
        let translation = Mat4::default();
        self.renderer.add_model(
            sphere,
            translation
        );
    }
}