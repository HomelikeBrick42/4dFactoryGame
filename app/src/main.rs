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
    frame_time: f32,

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
            ground_view: false,
            ground_view_percentage: 0.0,
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
            frame_time: 0.0,

            renderer,
            main_texture,

            last_spawned: VecDeque::new(),
        }
    }

    fn fixed_update(&mut self, #[expect(unused)] ts: f32) {}

    fn update(&mut self, width: u32, height: u32, input_state: &InputState, dt: f32) {
        self.frame_time += (dt - self.frame_time) * ((4.0 * dt).exp() - 1.0);

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
        frame.push_quad(Quad {
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
            texture: Some(&self.main_texture),
        });

        Self::render_compass(
            &mut frame,
            &self.font,
            self.camera.base_rotation.reverse(),
            aspect,
        );

        {
            let font_size = 0.06;
            let mut line = 0usize;
            let mut max_width = 0.0f32;
            let mut quads = vec![];
            let mut write_text = |s: &str| {
                let mut cursor = Vector2 {
                    x: -aspect,
                    y: 1.0
                        - self.font.base(font_size)
                        - line as f32 * self.font.line_height(font_size),
                };
                let start_cursor = cursor;
                quads.extend(self.font.str_quads(
                    &mut cursor,
                    font_size,
                    Vector4 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                        w: 1.0,
                    },
                    s,
                ));
                line += 1;
                max_width = max_width.max(cursor.x - start_cursor.x);
            };

            write_text(&format!("FPS: {:.1}", 1.0 / self.frame_time));
            write_text(&format!("Frame Time: {:.3}ms", self.frame_time * 1000.0));
            write_text(&format!(
                "Position: {:.2}, {:.2}, {:.2}, {:.2}",
                self.camera.position.x,
                self.camera.position.y,
                self.camera.position.z,
                self.camera.position.w,
            ));
            let rotation = self.camera.rotation();
            let forward = rotation.x();
            write_text(&format!(
                "Forward: {:5.2}, {:5.2}, {:5.2}, {:5.2}",
                forward.x, forward.y, forward.z, forward.w,
            ));
            let up = rotation.y();
            write_text(&format!(
                "Up:      {:5.2}, {:5.2}, {:5.2}, {:5.2}",
                up.x, up.y, up.z, up.w,
            ));
            let right = rotation.z();
            write_text(&format!(
                "Right:   {:5.2}, {:5.2}, {:5.2}, {:5.2}",
                right.x, right.y, right.z, right.w,
            ));
            let ana = rotation.w();
            write_text(&format!(
                "Ana:     {:5.2}, {:5.2}, {:5.2}, {:5.2}",
                ana.x, ana.y, ana.z, ana.w,
            ));

            frame.push_quad(Quad {
                position: Vector2 {
                    x: -aspect + max_width * 0.5,
                    y: 1.0 - line as f32 * self.font.line_height(font_size) * 0.5,
                },
                size: Vector2 {
                    x: max_width,
                    y: line as f32 * self.font.line_height(font_size),
                },
                uv_offset: Vector2 { x: 0.0, y: 0.0 },
                uv_size: Vector2 { x: 1.0, y: 1.0 },
                color: Vector4 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 0.5,
                },
                texture: None,
            });
            frame.push_quads(quads);
        }

        |render_pass| {
            frame.render(render_pass);
        }
    }
}

impl Game {
    pub fn render_compass(
        frame: &mut ui::Frame<'_>,
        font: &Font,
        rotation: NoE2Rotor,
        aspect: f32,
    ) {
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
                "+X",
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
                "-X",
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
                "+Z",
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
                "-Z",
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
                "+W",
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
                "-W",
            ),
        ];
        lines.sort_by(|(a, _, _), (b, _, _)| a.w.total_cmp(&b.w));
        for (direction, color, name) in lines {
            let center = Vector2 {
                x: aspect - 0.25,
                y: 0.75,
            };
            let direction_2d = Vector2 {
                x: direction.z,
                y: direction.x,
            };
            frame.push_line(Line {
                a: center,
                b: center + direction_2d * 0.22,
                width: 0.01,
                color,
            });

            if direction.w > -0.98 {
                let font_size = 0.075;
                let width = font.str_width(font_size, name);
                frame.push_quads(font.str_quads(
                    &mut Vector2 {
                        x: center.x + direction_2d.x * 0.2 - width * 0.5,
                        y: center.y + direction_2d.y * 0.2
                            - (font.line_height(font_size) - font.base(font_size)) * 0.5,
                    },
                    font_size,
                    Vector4 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                        w: 1.0,
                    },
                    name,
                ));
            }
        }
    }
}

fn main() {
    run_app::<Game>();
}
