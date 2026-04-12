use crate::camera::Camera;
use math::{NoE2Rotor, Vector2, Vector3, Vector4};
use renderer::{
    app::{App, InputState, run_app},
    ray_tracing::Renderer,
    texture::Texture,
    ui::{self, Quad},
};
use std::{f32::consts::TAU, time::Duration};

pub mod camera;

pub struct Game {
    device: wgpu::Device,
    #[expect(unused)]
    queue: wgpu::Queue,

    camera: Camera,
    selected_tile: Option<Vector3<i32>>,

    ui: ui::Renderer,
    renderer: Renderer,
    main_texture: Texture,
}

impl App for Game {
    const NAME: &str = "4d Factory Game";
    const FEATURES: wgpu::Features = wgpu::Features::FLOAT32_FILTERABLE;
    const PRESENT_MODE: wgpu::PresentMode = wgpu::PresentMode::AutoNoVsync;
    const COLOR_LOAD_OP: wgpu::LoadOp<wgpu::Color> =
        wgpu::LoadOp::DontCare(unsafe { wgpu::LoadOpDontCare::enabled() });
    const DEPTH_LOAD_OP: wgpu::LoadOp<f32> = wgpu::LoadOp::Clear(1.0);
    const FIXED_UPDATE_INTERVAL: Duration = Duration::from_secs(1).checked_div(64).unwrap();

    fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let camera = Camera {
            position: Vector4 {
                x: 0.0,
                y: 2.0,
                z: 0.0,
                w: 0.0,
            },
            base_rotation: NoE2Rotor::identity(),
            xy_rotation: 0.0,
            fov: TAU * 0.25,
        };

        let ui = ui::Renderer::new(device.clone(), queue.clone());
        let renderer = Renderer::new(device.clone(), queue.clone(), camera.to_gpu(None));
        let main_texture = Texture::new(
            &device,
            1,
            1,
            wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        );

        Self {
            device,
            queue,

            camera,
            selected_tile: None,

            ui,
            renderer,
            main_texture,
        }
    }

    fn fixed_update(&mut self, #[expect(unused)] ts: f32) {}

    fn update(&mut self, width: u32, height: u32, input_state: &InputState, dt: f32) {
        self.camera.update(input_state, dt);

        let ray = self
            .camera
            .get_ray(input_state.mouse_position(), width, height);
        let distance = ray.origin.y / -ray.direction.y;
        self.selected_tile = if distance > 0.0 {
            let point = ray.origin + ray.direction * distance;
            let tile = Vector3 {
                x: point.x.floor() as i32,
                y: point.z.floor() as i32,
                z: point.w.floor() as i32,
            };
            Some(tile)
        } else {
            None
        };
    }

    fn render<'a>(
        &'a mut self,
        width: u32,
        height: u32,
        encoder: &mut wgpu::CommandEncoder,
    ) -> impl FnOnce(&mut wgpu::RenderPass<'_>) + use<'a> {
        if self.main_texture.width() != width && self.main_texture.height() != height {
            self.main_texture = Texture::new(
                &self.device,
                width,
                height,
                wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            );
        }
        self.renderer
            .set_camera(self.camera.to_gpu(self.selected_tile));
        self.renderer.render(&mut self.main_texture, encoder);

        let aspect = width as f32 / height as f32;
        let mut frame = self.ui.begin_frame(aspect);
        frame.push_quad(
            Quad {
                position: Vector2 { x: 0.0, y: 0.0 },
                size: Vector2 {
                    x: aspect * 2.0,
                    y: 2.0,
                },
                uv_offset: Vector2 { x: 0.0, y: 0.0 },
                uv_size: Vector2 { x: 1.0, y: 1.0 },
                color: Vector4 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                    w: 1.0,
                },
            },
            Some(&self.main_texture),
        );

        |render_pass| {
            frame.render(render_pass);
        }
    }
}

fn main() {
    run_app::<Game>();
}
