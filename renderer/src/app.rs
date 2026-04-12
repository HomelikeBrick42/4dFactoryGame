use math::Vector2;
use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowAttributes, WindowId},
};
pub use winit::{event::MouseButton, keyboard::KeyCode};

pub trait App {
    const NAME: &str;
    const FEATURES: wgpu::Features;
    const PRESENT_MODE: wgpu::PresentMode;
    const COLOR_LOAD_OP: wgpu::LoadOp<wgpu::Color>;
    const DEPTH_LOAD_OP: wgpu::LoadOp<f32>;
    const FIXED_UPDATE_INTERVAL: Duration;

    fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self;

    fn fixed_update(&mut self, #[expect(unused)] ts: f32);
    fn update(
        &mut self,
        #[expect(unused)] width: u32,
        #[expect(unused)] height: u32,
        #[expect(unused)] input_state: &InputState,
        #[expect(unused)] dt: f32,
    );

    fn render<'a>(
        &'a mut self,
        #[expect(unused)] width: u32,
        #[expect(unused)] height: u32,
        #[expect(unused)] encoder: &mut wgpu::CommandEncoder,
    ) -> impl FnOnce(&mut wgpu::RenderPass<'_>) + use<'a, Self>;
}

struct AppState<A: App> {
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    app: A,
    last_time: Option<Instant>,
    dt: Duration,
    fixed_time: Duration,
    window_state: Option<WindowState>,
    input_state: InputState,
}

struct WindowState {
    window: Arc<Window>,
    surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    depth_texture_view: wgpu::TextureView,
}

pub struct InputState {
    keys: HashSet<KeyCode>,
    keys_just_pressed: HashSet<KeyCode>,
    keys_just_released: HashSet<KeyCode>,
    mouse_buttons: HashSet<MouseButton>,
    mouse_buttons_just_pressed: HashSet<MouseButton>,
    mouse_buttons_just_released: HashSet<MouseButton>,
    mouse_position: Vector2<f32>,
    last_mouse_position: Option<Vector2<f32>>,
}

impl InputState {
    fn begin_frame(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.mouse_buttons_just_pressed.clear();
        self.mouse_buttons_just_released.clear();
        self.last_mouse_position = Some(self.mouse_position);
    }

    pub fn key_pressed(&self, key: KeyCode) -> bool {
        self.keys.contains(&key)
    }

    pub fn key_released(&self, key: KeyCode) -> bool {
        !self.keys.contains(&key)
    }

    pub fn key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }

    pub fn key_just_released(&self, key: KeyCode) -> bool {
        self.keys_just_released.contains(&key)
    }

    pub fn mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons.contains(&button)
    }

    pub fn mouse_button_released(&self, button: MouseButton) -> bool {
        !self.mouse_buttons.contains(&button)
    }

    pub fn mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_just_pressed.contains(&button)
    }

    pub fn mouse_button_just_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons_just_released.contains(&button)
    }

    pub fn mouse_position(&self) -> Vector2<f32> {
        self.mouse_position
    }

    pub fn mouse_delta(&self) -> Vector2<f32> {
        self.mouse_position - self.last_mouse_position.unwrap_or(self.mouse_position)
    }
}

impl<A: App> ApplicationHandler for AppState<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.suspended(event_loop);

        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title(A::NAME))
                .unwrap(),
        );
        self.window_state = Some(WindowState {
            window: window.clone(),
            surface_config: wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8Unorm,
                width: 1,
                height: 1,
                present_mode: A::PRESENT_MODE,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: vec![],
            },
            surface: self.instance.create_surface(window).unwrap(),
            depth_texture_view: Self::depth_texture(&self.device, 1, 1),
        });
        self.resize();
    }

    fn suspended(&mut self, #[expect(unused)] event_loop: &ActiveEventLoop) {
        self.last_time = None;
        self.dt = Duration::ZERO;
        self.fixed_time = Duration::ZERO;
        self.window_state = None;
    }

    fn new_events(
        &mut self,
        #[expect(unused)] event_loop: &ActiveEventLoop,
        #[expect(unused)] cause: StartCause,
    ) {
        let time = Instant::now();
        self.dt = time - self.last_time.unwrap_or(time);
        self.last_time = Some(time);

        self.input_state.begin_frame();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(WindowState { window, .. }) = &self.window_state else {
            return;
        };
        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => event_loop.exit(),

            WindowEvent::Resized(_) => {
                self.resize();
                self.render();
            }

            WindowEvent::KeyboardInput {
                device_id: _,
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                is_synthetic: _,
            } => match state {
                ElementState::Pressed => {
                    self.input_state.keys_just_pressed.insert(key);
                    self.input_state.keys.insert(key);
                }

                ElementState::Released => {
                    self.input_state.keys_just_released.insert(key);
                    self.input_state.keys.remove(&key);
                }
            },

            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => match state {
                ElementState::Pressed => {
                    self.input_state.mouse_buttons_just_pressed.insert(button);
                    self.input_state.mouse_buttons.insert(button);
                }

                ElementState::Released => {
                    self.input_state.mouse_buttons_just_released.insert(button);
                    self.input_state.mouse_buttons.remove(&button);
                }
            },

            WindowEvent::CursorMoved {
                device_id: _,
                position: PhysicalPosition { x, y },
            } => {
                self.input_state.mouse_position = Vector2 {
                    x: x as f32,
                    y: y as f32,
                };
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, #[expect(unused)] event_loop: &ActiveEventLoop) {
        'fixed_update: {
            self.fixed_time += self.dt;
            const { assert!(A::FIXED_UPDATE_INTERVAL.as_secs_f32() <= 1.0) };
            for _ in 0..(1.0 / A::FIXED_UPDATE_INTERVAL.as_secs_f32()).ceil() as _ {
                if self.fixed_time < A::FIXED_UPDATE_INTERVAL {
                    break 'fixed_update;
                }

                self.fixed_time -= A::FIXED_UPDATE_INTERVAL;
                self.app
                    .fixed_update(A::FIXED_UPDATE_INTERVAL.as_secs_f32());
            }
            eprintln!(
                "Lagging too far behind, skipping {:.1} fixed updates",
                self.fixed_time.as_secs_f32() / A::FIXED_UPDATE_INTERVAL.as_secs_f32(),
            );
            self.fixed_time = Duration::ZERO;
        }

        let (width, height) = self
            .window_state
            .as_ref()
            .map(|window| {
                let size = window.window.inner_size();
                (size.width.max(1), size.height.max(1))
            })
            .unwrap_or((1, 1));
        self.app
            .update(width, height, &self.input_state, self.dt.as_secs_f32());

        self.render();
    }
}

impl<A: App> AppState<A> {
    fn depth_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&Default::default())
    }

    fn resize(&mut self) {
        let Some(WindowState {
            window,
            surface_config,
            surface,
            depth_texture_view,
        }) = &mut self.window_state
        else {
            return;
        };

        let size = window.inner_size();
        surface_config.width = size.width;
        surface_config.height = size.height;
        if surface_config.width > 0 && surface_config.height > 0 {
            surface.configure(&self.device, surface_config);
            *depth_texture_view =
                Self::depth_texture(&self.device, surface_config.width, surface_config.height);
        }
    }

    fn render(&mut self) {
        let Some(WindowState {
            surface,
            depth_texture_view,
            ..
        }) = &self.window_state
        else {
            return;
        };

        let (surface_texture, suboptimal) = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => (surface_texture, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => (surface_texture, true),

            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return;
            }

            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.resize();
                return;
            }

            wgpu::CurrentSurfaceTexture::Validation => {
                eprintln!("Validation error when trying to get the current texture");
                return;
            }
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let app_rendering = self.app.render(
                surface_texture.texture.width(),
                surface_texture.texture.height(),
                &mut encoder,
            );

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_texture.texture.create_view(&Default::default()),
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: A::COLOR_LOAD_OP,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: A::DEPTH_LOAD_OP,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            app_rendering(&mut render_pass);
        }
        self.queue.submit(core::iter::once(encoder.finish()));

        surface_texture.present();
        if suboptimal {
            self.resize();
        }
    }
}

pub fn run_app<A: App>() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all().with_env(),
        flags: wgpu::InstanceFlags::from_build_config().with_env(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds {
            for_resource_creation: None,
            for_device_loss: None,
        },
        backend_options: wgpu::BackendOptions::default().with_env(),
        display: Some(Box::new(event_loop.owned_display_handle())),
    });

    let (device, queue) = pollster::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Wgpu Device"),
                required_features: A::FEATURES,
                required_limits: adapter.limits(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        (device, queue)
    });

    let app = A::new(device.clone(), queue.clone());
    let mut app_state = AppState {
        instance,
        device,
        queue,
        app,
        last_time: None,
        dt: Duration::ZERO,
        fixed_time: Duration::ZERO,
        window_state: None,
        input_state: InputState {
            keys: HashSet::new(),
            keys_just_pressed: HashSet::new(),
            keys_just_released: HashSet::new(),
            mouse_buttons: HashSet::new(),
            mouse_buttons_just_pressed: HashSet::new(),
            mouse_buttons_just_released: HashSet::new(),
            mouse_position: Vector2 { x: 0.0, y: 0.0 },
            last_mouse_position: None,
        },
    };
    event_loop.run_app(&mut app_state).unwrap();
}
