use frost::obb::CollisionPoint;
use frost::physics::math::physics_system;
use frost::{Input, RigidBody, Search, SearchIter, Transform, World};
use glam::{Mat4, Vec3};
use imgui::{Condition, DragRange, FontConfig, FontGlyphRanges, FontSource, ImString, Ui, UiBuffer};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use log::{debug, info};
use lynch::render_graph::RenderGraph;

use lynch::vulkan::renderer::RenderStatistics;
use lynch::{renderer::Renderer, vulkan::renderer::VulkanRenderer, window::window::Window, Camera};

use crate::engine_callbacks::{EngineCallbacks, GameCallbacks};
use crate::{
    engine_settings, EngineSettings, EngineSettingsBuilder, DEFAULT_FPS_CAP, DEFAULT_MAX_FPS,
    DEFAULT_UPDATE_RATE,
};
use frost::System;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
pub struct GfxLocation(pub usize);

pub struct CooperApplication {
    window: Window,
    pub renderer: VulkanRenderer,

    graph: RenderGraph,
    pub camera: Camera,
    event_loop: EventLoop<()>,
    engine_settings: EngineSettings,
    ui: CooperUI,
}
struct UIContext<'a> {
    pub ui_frame: Option<Mutex<Box<&'a mut Ui>>>,
}
struct CooperUI {
    pub gui: imgui::Context,
    pub platform: imgui_winit_support::WinitPlatform,
}

struct PhysicsControl {
    paused: bool,
    step: bool,
}
impl PhysicsControl {
    fn new() -> Self {
        PhysicsControl {
            paused: false,
            step: false,
        }
    }

    fn pause(&mut self) {
        self.paused = true;
    }

    fn resume(&mut self) {
        self.paused = false;
    }

    fn step(&mut self) {
        if self.paused {
            self.step = true;
        }
    }

    fn should_update_physics(&mut self) -> bool {
        if self.paused {
            if self.step {
                self.step = false; // Reset step to wait for the next manual step trigger
                true // Allow physics update for this frame
            } else {
                false // Skip physics update while paused
            }
        } else {
            true // Always update physics if not paused
        }
    }
}
impl CooperUI {
    fn new(window: &Window) -> Self {
        let (gui, platform) = {
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

        Self { gui, platform }
    }
    fn mut_ui(&mut self) -> &mut imgui::Context {
        &mut self.gui
    }
    fn update_ui(&self, guiframe: &mut Ui) {}
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
    mesh_instances: usize,
    opened: bool,
}
impl DebugInfo {
    pub fn new(
        camera_location: Vec3,
        delta_time: f32,
        fixed_delta_time: f32,
        frame_rate: f32,
        fixed_frame_rate: f32,

        mesh_instances: usize,
    ) -> Self {
        Self {
            camera_location,
            recent_collisions: vec![],
            delta_time,
            fixed_delta_time,
            frame_rate,
            fixed_frame_rate,
            mesh_instances,
            opened: true,
        }
    }
    pub fn update(
        &mut self,
        camera_location: Vec3,
        recent_collisions: Vec<CollisionPoint>,
        delta_time: f32,
        fixed_delta_time: f32,
        frame_rate: f32,
        fixed_frame_rate: f32,
        mesh_instances: usize,
    ) -> &Self {
        self.camera_location = camera_location;
        self.recent_collisions.extend(recent_collisions);
        self.delta_time = delta_time;
        self.fixed_delta_time = fixed_delta_time;
        self.frame_rate = frame_rate;
        self.mesh_instances = mesh_instances;
        self.fixed_frame_rate = fixed_frame_rate;
        return self;
    }
}

impl CooperApplication {
    pub fn create() -> Self {
        let (window, event_loop) = Window::create("Cooper", WIDTH, HEIGHT);
        let engine_settings = EngineSettingsBuilder::new()
            .fps_cap(Some(DEFAULT_MAX_FPS))
            .update_rate_hz(DEFAULT_UPDATE_RATE)
            .build();

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

        let mut ui = CooperUI::new(&window);
        let renderer = VulkanRenderer::create(&window, &camera, ui.mut_ui());
        let graph = RenderGraph::new(
            renderer.vk_context.arc_device(),
            &renderer.camera_uniform_buffer,
            renderer.image_count,
        );

        CooperApplication {
            window,
            renderer,
            graph,
            camera,
            event_loop,
            engine_settings,
            ui,
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
        J: FnMut(&mut bool, &mut Ui) + 'static,
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

        let mut physics_control = PhysicsControl {
            paused: false,
            step: false,
        };

        let mut debug_info = DebugInfo::new(
            self.camera.get_position(),
            0.0,
            self.engine_settings.fixed_update_rate.as_secs_f32(),
            0.0,
            DEFAULT_UPDATE_RATE as f32,
            0,
        );
        //let cube_hash_map : HashMap<str, usize>  = HashMap::default();
        let mut last_fixed_update = Instant::now();
        let mut lag = 0.0;
        let mut count = 0;
        let mut interval_start = Instant::now();
        let mut last_frame = Instant::now();
        let mut run = true;
        let mut rigidbody_list: Vec<RigidBody> = vec![];
        let mut render_statistics = RenderStatistics::default();
        self.event_loop.run(move |event, _elwt, control_flow| {
            self.ui
                .platform
                .handle_event(self.ui.gui.io_mut(), &self.window.window, &event);

            *control_flow = ControlFlow::Poll;
            match event {
                Event::NewEvents(_) => {
                    let now = Instant::now();
                    self.ui
                        .gui
                        .io_mut()
                        .update_delta_time(now.duration_since(last_frame));

                    last_frame = now;
                }
                Event::MainEventsCleared => {
                    // call fixed_update fixed_update_rate times per second
                    while lag >= self.engine_settings.fixed_update_rate.as_secs_f32()
                        || physics_control.step
                    {
                        // user fixed update call
                        count += 1; // Increment the count for each execution
                        if physics_control.should_update_physics() {
                            physics_system
                                .run(&world, self.engine_settings.fixed_update_rate.as_secs_f32())
                                .unwrap();
                            let mut rb_search =
                                world.search::<(&GfxLocation, &RigidBody)>().unwrap();
                            rb_search.iter().for_each(|(gfx, rb)| {
                                let new_location = Mat4::from_scale_rotation_translation(
                                    rb.transform.scale,
                                    rb.transform.rotation,
                                    rb.transform.position,
                                );
                                self.renderer.internal_renderer.instances[gfx.0].transform =
                                    new_location;
                            });
                        }

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
                        let mut search = world.search::<(&RigidBody,)>().unwrap();
                        for (i, rb) in search.iter().enumerate() {
                            if i == rigidbody_list.len() {
                                rigidbody_list.push(rb.clone())
                            } else {
                                rigidbody_list[i] = rb.clone()
                            }
                        }
                    }

                    let mut gui_frame = self.ui.gui.frame();
                    Self::debug_ui(
                        gui_frame,
                        &mut debug_info,
                        &rigidbody_list,
                        &mut physics_control,
                        &input,
                        &mut world,
                    );
                    ui_func(&mut run, &mut gui_frame);

                    let draw_data = self.ui.gui.render();
                    render_statistics.full_render_time = self.renderer.render(
                        &mut self.graph,
                        &self.camera,
                        draw_data,
                        &mut render_statistics,
                    );

                    let current_time = Instant::now();
                    let elapsed = current_time.duration_since(last_fixed_update);
                    last_fixed_update = current_time;
                    lag += elapsed.as_secs_f32();
                    // user update call
                    update(&update_transmitter, render_statistics.full_render_time);
                    // submit input data to camera
                    self.camera
                        .update(&input, render_statistics.full_render_time);

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
                    // if self.engine_settings.fps_settings.limit {
                    // let elapsed = last_fixed_update.elapsed();
                    // if elapsed < self.engine_settings.fps_settings.frame_time {
                    // std::thread::sleep(
                    // self.engine_settings.fps_settings.frame_time - elapsed,
                    // );
                    // }
                    // }
                    debug_info.update(
                        self.camera.get_position(),
                        vec![],
                        render_statistics.full_render_time,
                        self.engine_settings.fixed_update_rate.as_secs_f32(),
                        1. / render_statistics.full_render_time,
                        count as f32,
                        self.renderer.internal_renderer.instances.len(),
                    );
                    input.end_frame();
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
    fn debug_ui(
        gui_frame: &mut Ui,
        debug_info: &mut DebugInfo,
        rigidbody_list: &Vec<RigidBody>,
        physics_control: &mut PhysicsControl,
        input: &Input,
        world: &mut World,
    ) {
        if input.key_pressed(winit::event::VirtualKeyCode::M) {
            println!("M was pressed");
            debug_info.opened = !debug_info.opened;
        }
        if !debug_info.opened {return};
        let w = gui_frame
            .window("Debug Menu")
            .opened(&mut debug_info.opened)
            .position([1000.0, 20.0], Condition::Appearing)
            .size([200.0, 100.0], Condition::Appearing);

        w.build(|| {
            gui_frame.text(format!("FPS: {}", debug_info.frame_rate));
            gui_frame.text(format!("Delta Time: {}", debug_info.delta_time));

            gui_frame.text(format!("Camera location: {:?}", debug_info.camera_location));
        });

        gui_frame
            .window("Rigidbody Debug")
            .size([50., 100.], Condition::FirstUseEver)
            .position([10., 10.], Condition::FirstUseEver)
            .build(|| {
                imgui::ListClipper::new(rigidbody_list.len() as i32)
                    .items_height(gui_frame.current_font_size() * 2.0)
                    .begin(gui_frame);
                let mut rbs = world.search::<(&mut RigidBody,)>().unwrap();
                let input_width = 70.0;
                for (i, rigidbody) in rbs.iter().enumerate() {
                    let pos = &mut rigidbody.transform.position;
                


                    // Unique identifiers for each component's drag control
                    let x_id = format!("X:##{}", i);
                    let y_id = format!("Y:##{}", i);
                    let z_id = format!("Z:##{}", i);

                    let drag_speed = 0.1; // The rate of change when dragging
                    let drag_min = -std::f32::MAX; // Minimum value
                    let drag_max = std::f32::MAX; // Maximum value

                    // Display text box for the x-component
                    gui_frame.set_next_item_width(input_width);
                    imgui::Drag::new(x_id)
                        .range(drag_min, drag_max)
                        .speed(drag_speed)
                        .build(&gui_frame, &mut pos.x);
                
                    gui_frame.same_line(); 
                    gui_frame.set_next_item_width(input_width);
                
                    imgui::Drag::new(y_id)
                        .range(drag_min, drag_max)
                        .speed(drag_speed)
                        .build(&gui_frame, &mut pos.y);
                
                
                    gui_frame.same_line(); 
                    gui_frame.set_next_item_width(input_width);
                
                    imgui::Drag::new(z_id)
                        .range(drag_min, drag_max)
                        .speed(drag_speed)
                        .build(&gui_frame, &mut pos.z);
                
                    gui_frame.separator();
                }
            });
        gui_frame
            .window("Physics Control")
            .size([50., 50.], Condition::FirstUseEver)
            .position([10., 110.], Condition::FirstUseEver)
            .build(|| {
                gui_frame.checkbox("Do Physics? ", &mut physics_control.paused);
                physics_control.step = gui_frame.button("Step");
            });
    }
    fn create_scene(&mut self) {
        self.renderer.initialize();
        self.build_scene();
    }
    fn build_scene(&mut self) {
        let mut sphere = self.renderer.load_model("models/sphere.gltf");
        let translation = Mat4::from_translation(Vec3::new(0., 0., 0.));
        self.renderer.add_model(sphere, translation);
    }
}


struct Vec3Control {
    vec: Vec3,
    x_input: ImString,
    y_input: ImString,
    z_input: ImString,
}
