use crate::texture::{Texture, sampler_bind_group_layout};
use bytemuck::NoUninit;
use math::{Vector2, Vector3, Vector4};

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,

    white_texture: Texture,

    objects_info_buffer: wgpu::Buffer,
    quads_buffer: wgpu::Buffer,
    circles_buffer: wgpu::Buffer,
    lines_buffer: wgpu::Buffer,
    objects_bind_group_layout: wgpu::BindGroupLayout,
    objects_bind_group: wgpu::BindGroup,

    quads_render_pipeline: wgpu::RenderPipeline,
    circles_render_pipeline: wgpu::RenderPipeline,
    lines_render_pipeline: wgpu::RenderPipeline,
}

enum Layer {
    Quad {
        start_index: u32,
        end_index: u32,
        texture_bind_group: wgpu::BindGroup,
    },
    Circle {
        start_index: u32,
        end_index: u32,
    },
    Line {
        start_index: u32,
        end_index: u32,
    },
}

impl Renderer {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let white_texture = Texture::new(
            &device,
            1,
            1,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        queue.write_texture(
            white_texture.texture().as_image_copy(),
            &[255, 255, 255, 255],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: None,
                rows_per_image: None,
            },
            white_texture.texture().size(),
        );

        let objects_info_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Objects Info Buffer"),
            size: size_of::<GpuObjectsInfo>().next_multiple_of(16) as _,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let quads_buffer = quad_buffer(&device, 0);
        let circles_buffer = circle_buffer(&device, 0);
        let lines_buffer = line_buffer(&device, 0);

        let objects_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Objects Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let objects_bind_group = objects_bind_group(
            &device,
            &objects_bind_group_layout,
            &objects_info_buffer,
            &quads_buffer,
            &circles_buffer,
            &lines_buffer,
        );

        macro_rules! pipeline {
            {
                $shader_name:literal,
                $name:literal,
                [
                    $($bind_group_layouts:expr,)*
                ],
            } => {{
                let shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
                    env!("OUT_DIR"),
                    "/shaders/",
                    $shader_name,
                    ".wgsl"
                )));
                let render_pipeline_layout =
                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some(concat!($name, " Render Pipeline Layout")),
                        bind_group_layouts: &[$($bind_group_layouts,)*],
                        immediate_size: 0,
                    });
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some(concat!($name, " Render Pipeline")),
                        layout: Some(&render_pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: Some("vertex"),
                            compilation_options: Default::default(),
                            buffers: &[],
                        },
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleStrip,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Cw,
                            cull_mode: None,
                            unclipped_depth: false,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            conservative: false,
                        },
                        depth_stencil: Some(wgpu::DepthStencilState {
                            format: wgpu::TextureFormat::Depth32Float,
                            depth_write_enabled: Some(false),
                            depth_compare: Some(wgpu::CompareFunction::Always),
                            stencil: wgpu::StencilState::default(),
                            bias: wgpu::DepthBiasState::default(),
                        }),
                        multisample: wgpu::MultisampleState {
                            count: 1,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: Some("fragment"),
                            compilation_options: Default::default(),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: wgpu::TextureFormat::Bgra8Unorm,
                                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                write_mask: wgpu::ColorWrites::all(),
                            })],
                        }),
                        multiview_mask: None,
                        cache: None,
                    })
            }};
        }

        let quads_render_pipeline = pipeline! {
            "quads",
            "Quads",
            [
                Some(&objects_bind_group_layout),
                Some(&sampler_bind_group_layout(&device)),
            ],
        };
        let circles_render_pipeline = pipeline! {
            "circles",
            "Circles",
            [
                Some(&objects_bind_group_layout),
            ],
        };
        let lines_render_pipeline = pipeline! {
            "lines",
            "Lines",
            [
                Some(&objects_bind_group_layout),
            ],
        };

        Self {
            device,
            queue,

            white_texture,

            objects_info_buffer,
            quads_buffer,
            circles_buffer,
            lines_buffer,
            objects_bind_group_layout,
            objects_bind_group,

            quads_render_pipeline,
            circles_render_pipeline,
            lines_render_pipeline,
        }
    }

    pub fn begin_frame(&mut self, aspect: f32) -> Frame<'_> {
        Frame {
            ui: self,
            aspect,
            quads: vec![],
            circles: vec![],
            lines: vec![],
            layers: vec![],
        }
    }
}

pub struct Frame<'ui> {
    ui: &'ui mut Renderer,
    aspect: f32,
    quads: Vec<Quad>,
    circles: Vec<GpuCircle>,
    lines: Vec<GpuLine>,
    layers: Vec<Layer>,
}

impl Frame<'_> {
    pub fn push_quad(&mut self, quad: Quad, texture: Option<&Texture>) {
        let start_index = self.quads.len();
        self.quads.push(quad);

        let texture = texture.unwrap_or(&self.ui.white_texture);
        if let Some(Layer::Quad {
            start_index: _,
            end_index,
            texture_bind_group,
        }) = self.layers.last_mut()
            && *texture_bind_group == *texture.sampler_bind_group()
        {
            *end_index = self.quads.len() as _;
        } else {
            self.layers.push(Layer::Quad {
                start_index: start_index as _,
                end_index: self.quads.len() as _,
                texture_bind_group: texture.sampler_bind_group().clone(),
            });
        }
    }

    pub fn push_circle(&mut self, circle: Circle) {
        let start_index = self.circles.len();
        self.circles.push(GpuCircle::from(circle));

        if let Some(Layer::Circle {
            start_index: _,
            end_index,
        }) = self.layers.last_mut()
        {
            *end_index = self.circles.len() as _;
        } else {
            self.layers.push(Layer::Circle {
                start_index: start_index as _,
                end_index: self.circles.len() as _,
            });
        }
    }

    pub fn push_line(&mut self, line: Line) {
        let start_index = self.lines.len();
        self.lines.push(GpuLine::from(line));

        if let Some(Layer::Line {
            start_index: _,
            end_index,
        }) = self.layers.last_mut()
        {
            *end_index = self.lines.len() as _;
        } else {
            self.layers.push(Layer::Line {
                start_index: start_index as _,
                end_index: self.lines.len() as _,
            });
        }
    }

    pub fn render(mut self, render_pass: &mut wgpu::RenderPass<'_>) {
        let mut buffer_reallocated = false;

        self.ui.queue.write_buffer(
            &self.ui.objects_info_buffer,
            0,
            bytemuck::bytes_of(&GpuObjectsInfo {
                aspect: self.aspect,
            }),
        );

        if size_of_val::<[_]>(&self.quads) > self.ui.quads_buffer.size() as _ {
            buffer_reallocated = true;
            self.ui.quads_buffer = quad_buffer(&self.ui.device, self.quads.len());
        }
        self.ui
            .queue
            .write_buffer(&self.ui.quads_buffer, 0, bytemuck::cast_slice(&self.quads));

        if size_of_val::<[_]>(&self.circles) > self.ui.circles_buffer.size() as _ {
            buffer_reallocated = true;
            self.ui.circles_buffer = circle_buffer(&self.ui.device, self.circles.len());
        }
        self.ui.queue.write_buffer(
            &self.ui.circles_buffer,
            0,
            bytemuck::cast_slice(&self.circles),
        );

        if size_of_val::<[_]>(&self.lines) > self.ui.lines_buffer.size() as _ {
            buffer_reallocated = true;
            self.ui.lines_buffer = line_buffer(&self.ui.device, self.lines.len());
        }
        self.ui
            .queue
            .write_buffer(&self.ui.lines_buffer, 0, bytemuck::cast_slice(&self.lines));

        if buffer_reallocated {
            self.ui.objects_bind_group = objects_bind_group(
                &self.ui.device,
                &self.ui.objects_bind_group_layout,
                &self.ui.objects_info_buffer,
                &self.ui.quads_buffer,
                &self.ui.circles_buffer,
                &self.ui.lines_buffer,
            );
        }

        render_pass.set_bind_group(0, &self.ui.objects_bind_group, &[]);
        for layer in self.layers.drain(..) {
            match layer {
                Layer::Quad {
                    start_index,
                    end_index,
                    texture_bind_group,
                } => {
                    render_pass.set_pipeline(&self.ui.quads_render_pipeline);
                    render_pass.set_bind_group(1, &texture_bind_group, &[]);
                    render_pass.draw(0..4, start_index..end_index);
                }

                Layer::Circle {
                    start_index,
                    end_index,
                } => {
                    render_pass.set_pipeline(&self.ui.circles_render_pipeline);
                    render_pass.draw(0..4, start_index..end_index);
                }

                Layer::Line {
                    start_index,
                    end_index,
                } => {
                    render_pass.set_pipeline(&self.ui.lines_render_pipeline);
                    render_pass.draw(0..4, start_index..end_index);
                }
            }
        }
    }
}

fn quad_buffer(device: &wgpu::Device, length: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Quad Buffer"),
        size: (length.max(1) * size_of::<Quad>()) as _,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn circle_buffer(device: &wgpu::Device, length: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Circle Buffer"),
        size: (length.max(1) * size_of::<GpuCircle>()) as _,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn line_buffer(device: &wgpu::Device, length: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Line Buffer"),
        size: (length.max(1) * size_of::<GpuLine>()) as _,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn objects_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    objects_info_buffer: &wgpu::Buffer,
    quads_buffer: &wgpu::Buffer,
    circles_buffer: &wgpu::Buffer,
    lines_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Objects Bind Group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: objects_info_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: quads_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: circles_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: lines_buffer.as_entire_binding(),
            },
        ],
    })
}

#[derive(Clone, Copy, NoUninit)]
#[repr(C)]
struct GpuObjectsInfo {
    aspect: f32,
}

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
pub struct Quad {
    pub position: Vector2<f32>,
    pub size: Vector2<f32>,
    pub uv_offset: Vector2<f32>,
    pub uv_size: Vector2<f32>,
    pub color: Vector4<f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct Circle {
    pub position: Vector2<f32>,
    pub radius: f32,
    pub color: Vector4<f32>,
}

#[derive(Clone, Copy, NoUninit)]
#[repr(C)]
pub struct GpuCircle {
    pub position: Vector2<f32>,
    pub radius: f32,
    pub _padding: f32,
    pub color: Vector4<f32>,
}

impl From<Circle> for GpuCircle {
    fn from(
        Circle {
            position,
            radius,
            color,
        }: Circle,
    ) -> Self {
        Self {
            position,
            radius,
            _padding: 0.0,
            color,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub a: Vector2<f32>,
    pub b: Vector2<f32>,
    pub width: f32,
    pub color: Vector4<f32>,
}

#[derive(Clone, Copy, NoUninit)]
#[repr(C)]
pub struct GpuLine {
    pub a: Vector2<f32>,
    pub b: Vector2<f32>,
    pub _padding: Vector3<f32>,
    pub width: f32,
    pub color: Vector4<f32>,
}

impl From<Line> for GpuLine {
    fn from(Line { a, b, width, color }: Line) -> Self {
        Self {
            a,
            b,
            _padding: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            width,
            color,
        }
    }
}
