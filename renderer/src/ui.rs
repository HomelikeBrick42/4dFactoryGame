use crate::texture::{Texture, sampler_bind_group_layout};
use bytemuck::NoUninit;
use math::{Vector2, Vector4};

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
pub struct Quad {
    pub position: Vector2<f32>,
    pub size: Vector2<f32>,
    pub uv_offset: Vector2<f32>,
    pub uv_size: Vector2<f32>,
    pub color: Vector4<f32>,
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,

    white_texture: Texture,

    quads_buffer: wgpu::Buffer,
    objects_bind_group_layout: wgpu::BindGroupLayout,
    objects_bind_group: wgpu::BindGroup,

    quads_render_pipeline: wgpu::RenderPipeline,
}

enum Layer {
    Quad {
        start_index: u32,
        end_index: u32,
        texture_bind_group: wgpu::BindGroup,
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
            bytemuck::cast_slice(&[1.0f32, 1.0, 1.0, 1.0]),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: None,
                rows_per_image: None,
            },
            white_texture.texture().size(),
        );

        let quads_buffer = quad_buffer(&device, 0);

        let objects_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Objects Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let objects_bind_group =
            objects_bind_group(&device, &objects_bind_group_layout, &quads_buffer);

        let quads_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/quads.wgsl"
        )));
        let quads_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Quads Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&objects_bind_group_layout),
                    Some(&sampler_bind_group_layout(&device)),
                ],
                immediate_size: 0,
            });
        let quads_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Quads Render Pipeline"),
                layout: Some(&quads_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &quads_shader,
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
                    module: &quads_shader,
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
            });

        Self {
            device,
            queue,

            white_texture,

            quads_buffer,
            objects_bind_group_layout,
            objects_bind_group,

            quads_render_pipeline,
        }
    }

    pub fn begin_frame(&mut self, aspect: f32) -> Frame<'_> {
        Frame {
            ui: self,
            aspect,
            quads: vec![],
            layers: vec![],
        }
    }
}

pub struct Frame<'ui> {
    ui: &'ui mut Renderer,
    aspect: f32,
    quads: Vec<Quad>,
    layers: Vec<Layer>,
}

impl Frame<'_> {
    pub fn push_quad(&mut self, mut quad: Quad, texture: Option<&Texture>) {
        let start_index = self.quads.len();

        quad.position.x /= self.aspect;
        quad.size.x /= self.aspect;
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

    pub fn render(mut self, render_pass: &mut wgpu::RenderPass<'_>) {
        let mut buffer_reallocated = false;

        if size_of_val::<[_]>(&self.quads) > self.ui.quads_buffer.size() as _ {
            buffer_reallocated = true;
            self.ui.quads_buffer = quad_buffer(&self.ui.device, self.quads.len());
        }
        self.ui
            .queue
            .write_buffer(&self.ui.quads_buffer, 0, bytemuck::cast_slice(&self.quads));

        if buffer_reallocated {
            self.ui.objects_bind_group = objects_bind_group(
                &self.ui.device,
                &self.ui.objects_bind_group_layout,
                &self.ui.quads_buffer,
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

fn objects_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    quads_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Objects Bind Group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: quads_buffer.as_entire_binding(),
        }],
    })
}
