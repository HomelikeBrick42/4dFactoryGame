use crate::{
    Id,
    storage::{Storage, StorageElement},
    texture::{Texture, write_storage_bind_group_layout},
};
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

#[derive(Clone, Copy, NoUninit)]
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

#[derive(Clone, Copy, NoUninit)]
#[repr(C)]
struct ObjectsInfo {
    hyperspheres_count: u32,
}

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
pub struct Hypersphere {
    pub position: Vector4<f32>,
    pub color: Vector3<f32>,
    pub radius: f32,
}

impl StorageElement for Hypersphere {
    type GpuType = Hypersphere;
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    objects_info_buffer: wgpu::Buffer,
    hyperspheres: Storage<Hypersphere>,
    objects_bind_group_layout: wgpu::BindGroupLayout,
    should_recreate_objects_bind_group: bool,
    objects_bind_group: wgpu::BindGroup,

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

        let objects_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Objects Info Buffer"),
            contents: &{
                let mut bytes = [0; size_of::<ObjectsInfo>().next_multiple_of(16)];
                bytes[..size_of::<ObjectsInfo>()].copy_from_slice(bytemuck::bytes_of(
                    &ObjectsInfo {
                        hyperspheres_count: 0,
                    },
                ));
                bytes
            },
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let hyperspheres = Storage::new(&device, "Hypersphere Buffer");
        let objects_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Objects Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
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
            hyperspheres.buffer(),
        );

        let ray_tracing_shader = device.create_shader_module(wgpu::include_spirv!(concat!(
            env!("OUT_DIR"),
            "/shaders/ray_tracing.spv"
        )));
        let ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracing Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&write_storage_bind_group_layout(&device)),
                    Some(&camera_bind_group_layout),
                    Some(&objects_bind_group_layout),
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

            objects_info_buffer,
            hyperspheres,
            objects_bind_group_layout,
            should_recreate_objects_bind_group: false,
            objects_bind_group,

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

    pub fn add_hypersphere(&mut self, hypersphere: Hypersphere) -> Id<Hypersphere> {
        let (id, reallocated) = self
            .hyperspheres
            .add(&self.device, &self.queue, hypersphere);
        self.should_recreate_objects_bind_group |= reallocated;
        id
    }

    pub fn update_hypersphere(&mut self, id: Id<Hypersphere>, hypersphere: Hypersphere) {
        self.hyperspheres.update(&self.queue, id, hypersphere);
    }

    pub fn remove_hypersphere(&mut self, id: Id<Hypersphere>) {
        self.hyperspheres.remove(&self.device, &self.queue, id);
    }

    pub fn render(&mut self, texture: &mut Texture, encoder: &mut wgpu::CommandEncoder) {
        if self.should_recreate_objects_bind_group {
            self.should_recreate_objects_bind_group = false;
            self.objects_bind_group = objects_bind_group(
                &self.device,
                &self.objects_bind_group_layout,
                &self.objects_info_buffer,
                self.hyperspheres.buffer(),
            );
        }

        self.queue.write_buffer(
            &self.objects_info_buffer,
            0,
            bytemuck::bytes_of(&ObjectsInfo {
                hyperspheres_count: self.hyperspheres.len() as _,
            }),
        );

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Ray Tracing Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.ray_tracing_pipeline);
        compute_pass.set_bind_group(0, texture.write_storage_bind_group(), &[]);
        compute_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        compute_pass.set_bind_group(2, &self.objects_bind_group, &[]);
        compute_pass.dispatch_workgroups(
            texture.width().div_ceil(16),
            texture.height().div_ceil(16),
            1,
        );
    }
}

fn objects_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    objects_info_buffer: &wgpu::Buffer,
    hypersphere_buffer: &wgpu::Buffer,
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
                resource: hypersphere_buffer.as_entire_binding(),
            },
        ],
    })
}
