use frost::obb::CollisionPoint;
use frost::{Input, RigidBody, Transform, World};
use glam::{Mat4, Vec3};
use imgui::{FontConfig, FontGlyphRanges, FontSource, Ui};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use log::{debug, info};
use lynch::render_graph::RenderGraph;
use lynch::window;
use lynch::{renderer::Renderer, vulkan::renderer::VulkanRenderer, window::window::Window, Camera};
use std::fmt::Debug;
use std::sync::mpsc::{self, Sender};
use std::time::{Duration, Instant};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::engine_callbacks::{EngineCallbacks, GameCallbacks};
use crate::{
    engine_settings, EngineSettings, EngineSettingsBuilder, DEFAULT_FPS_CAP, DEFAULT_MAX_FPS, DEFAULT_UPDATE_RATE
};

pub struct CooperApplication {
    window: Window,
    pub renderer: VulkanRenderer,

    graph: RenderGraph,
    pub camera: Camera,
    event_loop: EventLoop<()>,
    engine_settings: EngineSettings,
}
struct CooperUI {
    pub gui: imgui::Context,
    pub platform: imgui_winit_support::WinitPlatform,
    pub debug_info: DebugInfo

}
impl CooperUI {
    fn new(window: &Window, camera: &Camera, engine_settings: &EngineSettings) -> Self {
        let (mut gui, platform) = {
            let mut g = imgui::Context::create();
            let mut platform = WinitPlatform::init(&mut g);

            let hidpi_factor = platform.hidpi_factor();
            let font_size = (13.0 * hidpi_factor) as f32;
            g.fonts().add_font(&[
                FontSource::DefaultFontData {
                    config: Some(FontConfig {
                        size_pixels: font_size,
                        ..FontConfig::default()
                    }),
                },
                FontSource::TtfData {
                    data: include_bytes!("../../../assets/fonts/mplus-1p-regular.ttf"),
                    size_pixels: font_size,
                    config: Some(FontConfig {
                        rasterizer_multiply: 1.75,
                        glyph_ranges: FontGlyphRanges::default(),
                        ..FontConfig::default()
                    }),
                },
            ]);

            g.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
            platform.attach_window(g.io_mut(), &window.window, HiDpiMode::Rounded);
            (g, platform)
        };
        let debug_info = DebugInfo::new(
            camera.get_position(),
            0.0,
            engine_settings.fixed_update_rate.as_secs_f32(),
            0.0,
            DEFAULT_UPDATE_RATE as f32,
            0);
            Self {
                gui,
                platform,
                debug_info
            }
    }
    fn update_
}
const WIDTH: f64 = 1280.;
const HEIGHT: f64 = 720.;
pub enum GameEvent {
    Input,
    MoveEvent(usize, Mat4),
    Spawn(String),
    NextFrame,
}
pub struct DebugInfo {
    camera_location: Vec3,
    recent_collisions: Vec<CollisionPoint>,
    delta_time: f32,
    fixed_delta_time: f32,
    frame_rate: f32,
    fixed_frame_rate: f32,

    mesh_instances: usize
}
impl DebugInfo {
    pub fn new(
        camera_location: Vec3,
        delta_time: f32,
        fixed_delta_time: f32,
        frame_rate: f32,
        fixed_frame_rate: f32,

        mesh_instances: usize
    ) -> Self{
        Self {
            camera_location,
            recent_collisions: vec![],
            delta_time,
            fixed_delta_time,
            frame_rate,
            fixed_frame_rate,
            mesh_instances,
        }
    }
    pub fn update (&mut self, 
        camera_location: Vec3,
        recent_collisions: Vec<CollisionPoint>,
        delta_time: f32,
        fixed_delta_time: f32,
        frame_rate: f32,
        fixed_frame_rate: f32,
        mesh_instances: usize
        ) -> &Self {
        self.camera_location = camera_location;
        self.recent_collisions.extend(recent_collisions);
        self.delta_time = delta_time;
        self.fixed_delta_time =fixed_delta_time;
        self.frame_rate = frame_rate;
        self.mesh_instances =mesh_instances;
        self.fixed_frame_rate= fixed_frame_rate;
        return self
    }
}

impl CooperApplication {
    pub fn create() -> Self {
        let (window, event_loop) = Window::create("Cooper", WIDTH, HEIGHT);
        let fov_degrees = 90.0;
        let camera = Camera::new(
            Vec3::new(10.0, 0.0, 10.0),
            Vec3::new(10.0, 0.9, 0.0),
            fov_degrees,
            WIDTH / HEIGHT,
            0.01,
            1000.0,
            0.20,
        );
        
        let renderer = VulkanRenderer::create(&window, &camera, &mut gui);
        let graph = RenderGraph::new(
            renderer.vk_context.arc_device(),
            &renderer.camera_uniform_buffer,
            renderer.image_count,
        );
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
            engine_settings,
            gui,
            platform,
            debug_info
        }
    }

    pub fn run<E, F, G, H, J>(
        mut self: Self,
        mut start: E,
        mut update: F,
        mut fixed_update: G,
        mut finally: H,
        mut ui_func: J,
    ) where
        E: FnMut(&Sender<GameEvent>, &mut World) + 'static,
        F: FnMut(&Sender<GameEvent>, f32) + 'static,
        G: FnMut(&Sender<GameEvent>, f32, &World) + 'static,
        H: FnMut(&Sender<GameEvent>) + 'static,
        J: FnMut(&mut bool, &mut Ui, ) + 'static,
    {
        let mut world = World::new();
        let mut frame_count = 0;
        let mut last_fps_time = Instant::now();
        let fps_update_interval = Duration::new(1, 0); // 1 second
        self.create_scene();
        let mut input: Input = Input::default();
        let _events: Vec<WindowEvent> = Vec::new();
        let (event_trasmitter, event_receiver) = mpsc::channel();
        start(&event_trasmitter.clone(), &mut world);
        let update_transmitter = event_trasmitter.clone();
        let fixed_update_transmitter = event_trasmitter.clone();
        let finally_transmitter = event_trasmitter.clone();

        //let cube_hash_map : HashMap<str, usize>  = HashMap::default();
        let mut last_fixed_update = Instant::now();
        let mut lag = 0.0;
        let mut count = 0;
        let mut interval_start = Instant::now();
        let mut last_frame = Instant::now();
        let mut run = true;

        self.event_loop.run(move |event, _elwt, control_flow| {
            self.platform
                .handle_event(self.gui.io_mut(), &self.window.window, &event);
            
            *control_flow = ControlFlow::Poll;
            match event {
                Event::NewEvents(_) => {
                    let now = Instant::now();
                    self.gui
                        .io_mut()
                        .update_delta_time(now.duration_since(last_frame));

                    last_frame = now;
                    
                }
                Event::MainEventsCleared => {
                    let mut ui = self.gui.frame();
                    ui_func(&mut run, &mut ui,);
                    let draw_data = self.gui.render();
                    let delta = self
                        .renderer
                        .render(&mut self.graph, &self.camera, draw_data);
                    let current_time = Instant::now();
                    let elapsed = current_time.duration_since(last_fixed_update);
                    last_fixed_update = current_time;
                    lag += elapsed.as_secs_f32();
                    // user update call
                    update(&update_transmitter, delta);
                    // submit input data to camera
                    self.camera.update(&input, delta);

                    // call fixed_update fixed_update_rate times per second
                    while lag >= self.engine_settings.fixed_update_rate.as_secs_f32() {
                        // user fixed update call
                        count += 1; // Increment the count for each execution

                        fixed_update(
                            &fixed_update_transmitter,
                            self.engine_settings.fixed_update_rate.as_secs_f32(),
                            &mut world,
                        );

                        lag -= self.engine_settings.fixed_update_rate.as_secs_f32();
                        if interval_start.elapsed() >= Duration::new(1, 0) {
                            // Check if one second has passed
                            println!(
                                "Function executed {} times in the last second. ({})",
                                count,
                                self.engine_settings.fixed_update_rate.as_secs_f32()
                            );
                            count = 0; // Reset the count for the next second
                            interval_start = Instant::now(); // Reset the timer for the next second
                        }
                    }

                    input.reset_mouse();
                    // last user definable call
                    finally(&finally_transmitter);
                    loop {
                        let event = event_receiver.recv();
                        match event {
                            Ok(event) => {
                                match event {
                                    GameEvent::Input => {
                                        // TODO DO SOMETHING
                                    }
                                    GameEvent::MoveEvent(instance, matrix) => {
                                        // TODO DO SOMETHING
                                        self.renderer.internal_renderer.instances[instance]
                                            .transform = matrix;
                                    }
                                    GameEvent::Spawn(_path) => {
                                        // TODO handle spawn event
                                        let sphere = self.renderer.load_model(&_path);
                                        let translation = Mat4::default();
                                        self.renderer.add_model(sphere, translation);
                                    }
                                    GameEvent::NextFrame => {
                                        // MARK FRAME COMPLETE
                                        break;
                                    }
                                }
                            }
                            Err(_err) => {}
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
                            std::thread::sleep(
                                self.engine_settings.fps_settings.frame_time - elapsed,
                            );
                        }
                    }
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        self.graph.clear();
                        
                        return;
                    }
                    WindowEvent::Resized(resize_value) => {
                        self.renderer.resize(resize_value);
                    }
                    WindowEvent::MouseInput { .. }
                    | WindowEvent::CursorMoved { .. }
                    | WindowEvent::KeyboardInput { .. }
                    | WindowEvent::MouseWheel { .. } => {
                        input.update(&event);
                    }
                    _ => {}
                },
                _ => {}
            }
        });
    }
    fn debug_ui(&mut self) {
        
    }
    fn create_scene(&mut self) {
        self.renderer.initialize();
        self.build_scene();
    }
    fn build_scene(&mut self) {
        let sphere = self.renderer.load_model("models/cube.gltf");
        let translation = Mat4::from_translation(Vec3::new(10., -100., 10.));
        self.renderer.add_model(sphere, translation);
        // let sphere = self.renderer.load_model("models/sphere.gltf");
        // let translation = Mat4::default();
        // self.renderer.add_model(
        //     sphere,
        //     translation
        // );
        // let sphere = self.renderer.load_model("models/cube.gltf");
        // let translation = Mat4::from_translation(Vec3::new(10.,10.,10.));
        // self.renderer.add_model(
        //     sphere,
        //     translation
        // );
        // let sphere = self.renderer.load_model("models/MetalRoughSpheres.gltf");
        // let translation = Mat4::from_translation(-Vec3::new(10.,10.,10.));
        // self.renderer.add_model(
        //     sphere,
        //     translation
        // );
    }
}
