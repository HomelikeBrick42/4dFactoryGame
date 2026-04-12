use crate::texture::{Texture, write_storage_bind_group_layout};
use bytemuck::NoUninit;
use math::{Vector3, Vector4};
use wgpu::util::DeviceExt;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub position: Vector4<f32>,
    pub forward: Vector4<f32>,
    pub up: Vector4<f32>,
    pub right: Vector4<f32>,
    pub fov: f32,
    pub hovered_tile: Option<Vector3<i32>>,
}

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
struct GpuCamera {
    position: Vector4<f32>,
    forward: Vector4<f32>,
    up: Vector4<f32>,
    right: Vector4<f32>,
    hovered_tile: Vector3<i32>,
    is_hovering: u32,
    fov: f32,
}

impl From<Camera> for GpuCamera {
    fn from(
        Camera {
            position,
            forward,
            up,
            right,
            fov,
            hovered_tile,
        }: Camera,
    ) -> Self {
        Self {
            position,
            forward,
            up,
            right,
            hovered_tile: hovered_tile.unwrap_or(Vector3 { x: 0, y: 0, z: 0 }),
            is_hovering: hovered_tile.is_some() as _,
            fov,
        }
    }
}

pub struct Renderer {
    #[expect(unused)]
    device: wgpu::Device,
    queue: wgpu::Queue,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    ray_tracing_pipeline: wgpu::ComputePipeline,
}

impl Renderer {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, camera: Camera) -> Self {
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: &{
                let mut bytes = [0; size_of::<GpuCamera>().next_multiple_of(16)];
                bytes[..size_of::<GpuCamera>()]
                    .copy_from_slice(bytemuck::bytes_of(&GpuCamera::from(camera)));
                bytes
            },
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let ray_tracing_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/ray_tracing.wgsl"
        )));
        let ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracing Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&write_storage_bind_group_layout(&device)),
                    Some(&camera_bind_group_layout),
                ],
                immediate_size: 0,
            });
        let ray_tracing_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Ray Tracing Pipeline"),
                layout: Some(&ray_tracing_pipeline_layout),
                module: &ray_tracing_shader,
                entry_point: Some("trace_rays"),
                compilation_options: Default::default(),
                cache: None,
            });

        Self {
            device,
            queue,

            camera_buffer,
            camera_bind_group,

            ray_tracing_pipeline,
        }
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&GpuCamera::from(camera)),
        );
    }

    pub fn render(&mut self, texture: &mut Texture, encoder: &mut wgpu::CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Ray Tracing Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.ray_tracing_pipeline);
        compute_pass.set_bind_group(0, texture.write_storage_bind_group(), &[]);
        compute_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        compute_pass.dispatch_workgroups(
            texture.width().div_ceil(16),
            texture.height().div_ceil(16),
            1,
        );
    }
}
