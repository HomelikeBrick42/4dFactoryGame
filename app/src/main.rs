use crate::camera::Camera;
use math::{NoE2Rotor, Vector2, Vector3, Vector4};
use renderer::{
    Id,
    app::{App, InputState, MouseButton, run_app},
    ray_tracing::{Hypersphere, Renderer},
    texture::Texture,
    ui::{self, Circle, Font, Line, Quad},
};
use std::{collections::VecDeque, f32::consts::TAU, time::Duration};

pub mod camera;

pub struct Game {
    device: wgpu::Device,
    queue: wgpu::Queue,

    camera: Camera,
    selected_tile: Option<Vector3<i32>>,

    ui: ui::Renderer,
    font: Font,

    renderer: Renderer,
    main_texture: Texture,

    last_spawned: VecDeque<Id<Hypersphere>>,
}

impl App for Game {
    const NAME: &str = "4d Factory Game";
    const FEATURES: wgpu::Features = wgpu::Features::empty();
    const PRESENT_MODE: wgpu::PresentMode = wgpu::PresentMode::AutoNoVsync;
    const COLOR_LOAD_OP: wgpu::LoadOp<wgpu::Color> =
        wgpu::LoadOp::DontCare(unsafe { wgpu::LoadOpDontCare::enabled() });
    const DEPTH_LOAD_OP: wgpu::LoadOp<f32> = wgpu::LoadOp::Clear(1.0);
    const FIXED_UPDATE_INTERVAL: Duration = Duration::from_secs(1).checked_div(64).unwrap();

    fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let camera = Camera {
            position: Vector4 {
                x: 0.5,
                y: 2.0,
                z: 0.5,
                w: 0.5,
            },
            base_rotation: NoE2Rotor::identity(),
            xy_rotation: 0.0,
            fov: TAU * 0.25,
        };

        let ui = ui::Renderer::new(device.clone(), queue.clone());
        let font = Font::new(
            include_str!("../fonts/space_mono.fnt"),
            vec![
                {
                    let texture =
                        image::load_from_memory(include_bytes!("../fonts/space_mono_0.png"))
                            .unwrap()
                            .to_rgba8();
                    Texture::new(
                        &device,
                        &queue,
                        texture.width(),
                        texture.height(),
                        wgpu::TextureUsages::TEXTURE_BINDING,
                        wgpu::FilterMode::Linear,
                        wgpu::FilterMode::Linear,
                        Some(bytemuck::cast_slice(&texture)),
                    )
                },
                {
                    let texture =
                        image::load_from_memory(include_bytes!("../fonts/space_mono_1.png"))
                            .unwrap()
                            .to_rgba8();
                    Texture::new(
                        &device,
                        &queue,
                        texture.width(),
                        texture.height(),
                        wgpu::TextureUsages::TEXTURE_BINDING,
                        wgpu::FilterMode::Linear,
                        wgpu::FilterMode::Linear,
                        Some(bytemuck::cast_slice(&texture)),
                    )
                },
            ],
        );

        let renderer = Renderer::new(device.clone(), queue.clone(), camera.to_gpu(None));
        let main_texture = Texture::new(
            &device,
            &queue,
            1,
            1,
            wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::FilterMode::Linear,
            wgpu::FilterMode::Linear,
            None,
        );

        Self {
            device,
            queue,

            camera,
            selected_tile: None,

            ui,
            font,

            renderer,
            main_texture,

            last_spawned: VecDeque::new(),
        }
    }

    fn fixed_update(&mut self, #[expect(unused)] ts: f32) {}

    fn update(&mut self, width: u32, height: u32, input_state: &InputState, dt: f32) {
        self.camera.update(input_state, dt);

        let ray = self
            .camera
            .get_ray(input_state.mouse_position(), width, height);
        let distance = ray.origin.y / -ray.direction.y;
        self.selected_tile = if distance > 0.0
            && let point = ray.origin + ray.direction * distance
            && let tile = (Vector3 {
                x: point.x.floor() as i32,
                y: point.z.floor() as i32,
                z: point.w.floor() as i32,
            })
            && tile.x.abs() <= 16
            && tile.y.abs() <= 16
            && tile.z.abs() <= 16
        {
            Some(tile)
        } else {
            None
        };

        if let Some(selected_tile) = self.selected_tile
            && input_state.mouse_button_just_pressed(MouseButton::Left)
        {
            self.last_spawned
                .push_back(self.renderer.add_hypersphere(Hypersphere {
                    position: Vector4 {
                        x: selected_tile.x as f32 + 0.5,
                        y: 0.5,
                        z: selected_tile.y as f32 + 0.5,
                        w: selected_tile.z as f32 + 0.5,
                    },
                    color: Vector3 {
                        x: rand::random(),
                        y: rand::random(),
                        z: rand::random(),
                    },
                    radius: 0.5,
                }));

            if self.last_spawned.len() > 3 {
                self.renderer
                    .remove_hypersphere(self.last_spawned.pop_front().unwrap());
            }
        }
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
                &self.queue,
                width,
                height,
                wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                wgpu::FilterMode::Linear,
                wgpu::FilterMode::Linear,
                None,
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

        Self::render_compass(&mut frame, self.camera.base_rotation.reverse(), aspect);

        let mut cursor = Vector2 { x: -0.5, y: 0.0 };
        self.font.draw_str(
            &mut frame,
            &mut cursor,
            0.2,
            Vector4 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
                w: 1.0,
            },
            "this is some text",
        );

        |render_pass| {
            frame.render(render_pass);
        }
    }
}

impl Game {
    pub fn render_compass(frame: &mut ui::Frame<'_>, rotation: NoE2Rotor, aspect: f32) {
        frame.push_circle(Circle {
            position: Vector2 {
                x: aspect - 0.25,
                y: 0.75,
            },
            radius: 0.25,
            color: Vector4 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.75,
            },
        });
        frame.push_circle(Circle {
            position: Vector2 {
                x: aspect - 0.25,
                y: 0.75,
            },
            radius: 0.23,
            color: Vector4 {
                x: 0.5,
                y: 0.5,
                z: 0.5,
                w: 0.75,
            },
        });

        let mut lines = [
            (
                rotation.transform_direction(Vector4 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                    w: 0.0,
                }),
                Vector4 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
            ),
            (
                rotation.transform_direction(Vector4 {
                    x: -1.0,
                    y: 0.0,
                    z: 0.0,
                    w: 0.0,
                }),
                Vector4 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
            ),
            (
                rotation.transform_direction(Vector4 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                    w: 0.0,
                }),
                Vector4 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                    w: 1.0,
                },
            ),
            (
                rotation.transform_direction(Vector4 {
                    x: 0.0,
                    y: 0.0,
                    z: -1.0,
                    w: 0.0,
                }),
                Vector4 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                    w: 1.0,
                },
            ),
            (
                rotation.transform_direction(Vector4 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                }),
                Vector4 {
                    x: 1.0,
                    y: 0.0,
                    z: 1.0,
                    w: 1.0,
                },
            ),
            (
                rotation.transform_direction(Vector4 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: -1.0,
                }),
                Vector4 {
                    x: 1.0,
                    y: 0.0,
                    z: 1.0,
                    w: 1.0,
                },
            ),
        ];
        lines.sort_by(|(a, _), (b, _)| a.w.total_cmp(&b.w));
        for (direction, color) in lines {
            let center = Vector2 {
                x: aspect - 0.25,
                y: 0.75,
            };
            frame.push_line(Line {
                a: center,
                b: center
                    + Vector2 {
                        x: direction.z,
                        y: direction.x,
                    } * 0.22,
                width: 0.01,
                color,
            });
        }
    }
}

fn main() {
    run_app::<Game>();
}
