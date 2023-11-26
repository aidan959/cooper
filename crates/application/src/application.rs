
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
        self.input_subscribers
            .iter()
            .for_each(|subscriber: &Box<dyn Fn(&GameEvent)>| { subscriber(&event)});
    }

    fn dispatch_update_event(&self, event: &GameEvent) {
        self.update_subscribers
            .iter()
            .for_each(|subscriber: &Box<dyn Fn(&GameEvent)>| { subscriber(&event)});
    }

    fn dispatch_render_event(&self, _event: &GameEvent) {
        self.render_subscribers
            .iter()
            .for_each(|subscriber: &Box<dyn Fn()>| { subscriber()});
    }
}



use ash::vk;
use frost::Input;
use lynch::{Camera, gltf_loader};
use lynch::graph::Graph;
use lynch::vulkan::Buffer;
use lynch::{window::window::Window, renderer::Renderer};
use lynch::vulkan::renderer::VulkanRenderer;
use glam::{Vec3, Mat4};
use winit::
    event::{Event, WindowEvent}
;
use winit::event_loop::EventLoop;

pub struct CooperApplication {
    window: Window,
    pub renderer: VulkanRenderer,
    event_handler: EventHandler,
    graph: Graph,
    view_data: lynch::ViewUniformData,
    camera_uniform_buffer: Vec<Buffer>,
    pub camera: Camera,
    event_loop: EventLoop<()>
}
const WIDTH : f64= 1280.;
const HEIGHT : f64 = 720.;

impl CooperApplication {
    pub fn create() -> CooperApplication {
        let (window,event_loop) = Window::create("Cooper", WIDTH, HEIGHT);
        
        let renderer = VulkanRenderer::create(&window);
        let event_handler = EventHandler::new();

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

        CooperApplication {
            window,
            renderer,
            event_handler,
            graph,
            view_data,
            camera_uniform_buffer,
            camera,
            event_loop
        }
    }
    pub fn run(mut self: Self) -> (){
        //let mut cursor_position = None;
        let _frame_count = 0;
        self.create_scene();
        let mut input : Input = Input::default(); 
        let _events : Vec<WindowEvent> = Vec::new();
        let mut spawned = false;
        self.graph.recompile_all_shaders(self.renderer.device(), Some(self.renderer.internal_renderer.bindless_descriptor_set_layout));
        self.event_loop.run( move
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
                            
                            CooperApplication::record_commands(
                                &self.renderer,
                                self.renderer.sync_frames[self.renderer.current_frame].command_buffer,
                                self.renderer.sync_frames[self.renderer.current_frame].command_buffer_reuse_fence,
                                |command_buffer| {
                                    self.camera_uniform_buffer[self.renderer.current_frame]
                                        .update_memory(&self.renderer.device(), std::slice::from_ref(&self.view_data));
                
                                    self.graph.new_frame(self.renderer.current_frame);
                                    self.graph.clear(self.renderer.device());
                
            
                                    lynch::render_tools::build_render_graph(
                                        &mut self.graph,
                                        &self.renderer.device(),
                                        &self.renderer,
                                        &self.view_data,
                                        &self.camera,
                                    );

                                    self.graph.prepare(&self.renderer);
                                    let image = self.renderer.present_images[present_index].clone();
                                    self.graph.render(
                                        &command_buffer,
                                        &self.renderer,
                                        &[image]
                                    );
            
                                    
                                },
                            );
                            self.renderer.submit_commands(self.graph.current_frame);
                            self.renderer.present_images[present_index].current_layout = vk::ImageLayout::PRESENT_SRC_KHR; 
                            self.renderer.present_frame(present_index, self.graph.current_frame);
                            self.renderer.current_frame = (self.renderer.current_frame + 1 ) % self.renderer.num_frames_in_flight as usize;
                            self.renderer.internal_renderer.need_environment_map_update = false;
                            self.graph.current_frame = self.renderer.current_frame;

                        },
                        WindowEvent::CloseRequested => {
                            self.graph.clear(self.renderer.device());
                                
                            _elwt.exit();
                        },
                        WindowEvent::Resized(resize_value) => {
                            self.renderer.resize(resize_value);
                            //resize_dimensions = Some([width as u32, height as u32]);
                        }
                        WindowEvent::MouseInput {..} | WindowEvent::CursorMoved {..}| WindowEvent::KeyboardInput {..}| WindowEvent::MouseWheel {..} => {
                            input.update(&event);
                        }
                        _ => {
                            
                        }
                    },
                    Event::LoopExiting => self.renderer.wait_gpu_idle(),

                                    
                    _ => {self.window.window.request_redraw();}
                    
                }                
            }
        ).unwrap()
    }
    fn record_commands<F: FnOnce(ash::vk::CommandBuffer)>(
        renderer: &VulkanRenderer,
        command_buffer:  ash::vk::CommandBuffer,
        wait_fence:  ash::vk::Fence,
        render_commands: F,
    ) {
        let device = renderer.device();
        unsafe {
            {
                device
                    .device()
                    .wait_for_fences(&[wait_fence], true, std::u64::MAX)
                    .expect("Wait for fence failed.");
                
                device
                    .device()
                    .reset_fences(&[wait_fence])
                    .expect("Reset fences failed.");
            }

            device
                .device()
                .reset_command_buffer(
                    command_buffer,
                    ash::vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .expect("Reset command buffer failed.");

            let command_buffer_begin_info = ash::vk::CommandBufferBeginInfo::builder()
                .flags(ash::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            device
                .device()
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Begin command buffer failed.");

            render_commands(command_buffer);

            device
                .device()
                .end_command_buffer(command_buffer)
                .expect("End commandbuffer failed.");
        }
    }
    fn create_scene(&mut self) {
        self.renderer.initialize();

        self.build_scene();
    }
    fn build_scene(&mut self ) {
        let mut sphere = lynch::gltf_loader::load_gltf(self.renderer.device(), "models/cube.gltf");
        sphere.meshes[0].material.material_type =
            gltf_loader::MaterialType::Dielectric;
        let translation_matrix = Mat4::from_scale_rotation_translation(
                glam::Vec3::new(1., 1., 1.),
                glam::Quat::IDENTITY ,
                glam::Vec3::new(6., 6., 6.));
        self.renderer.add_model(
            sphere,
            translation_matrix
        );
        let mut sphere = lynch::gltf_loader::load_gltf(self.renderer.device(), "models/sphere.gltf");
        sphere.meshes[0].material.material_type =
            gltf_loader::MaterialType::Dielectric;
        let translation_matrix = Mat4::from_scale_rotation_translation(
                glam::Vec3::new(1., 1., 1.),
                glam::Quat::IDENTITY ,
                glam::Vec3::new(7., 6.5, 6.));
        self.renderer.add_model(
            sphere,
            translation_matrix
        );
        let mut sphere = lynch::gltf_loader::load_gltf(self.renderer.device(), "models/sphere.gltf");
        sphere.meshes[0].material.material_type =
            gltf_loader::MaterialType::Dielectric;
        let translation_matrix = Mat4::from_scale_rotation_translation(
                glam::Vec3::new(1., 10., 1.),
                glam::Quat::IDENTITY ,
                glam::Vec3::new(6.5, 15., 6.));
        self.renderer.add_model(
            sphere,
            translation_matrix
        );

    }
}
