pub struct Texture {
    texture: wgpu::Texture,
    sampler_bind_group: Option<wgpu::BindGroup>,
    write_storage_bind_group: Option<wgpu::BindGroup>,
}

impl Texture {
    pub fn new(device: &wgpu::Device, width: u32, height: u32, usage: wgpu::TextureUsages) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&Default::default());

        let sampler_bind_group = if usage.contains(wgpu::TextureUsages::TEXTURE_BINDING) {
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Texture Sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                min_filter: wgpu::FilterMode::Linear,
                mag_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::MipmapFilterMode::Nearest,
                ..Default::default()
            });

            Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Sampler Texture Bind Group"),
                layout: &sampler_bind_group_layout(device),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            }))
        } else {
            None
        };

        let write_storage_bind_group = if usage.contains(wgpu::TextureUsages::STORAGE_BINDING) {
            Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Write Storage Texture Bind Group"),
                layout: &write_storage_bind_group_layout(device),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                }],
            }))
        } else {
            None
        };

        Self {
            texture,
            sampler_bind_group,
            write_storage_bind_group,
        }
    }

    pub fn width(&self) -> u32 {
        self.texture.size().width
    }

    pub fn height(&self) -> u32 {
        self.texture.size().height
    }

    pub(crate) fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub(crate) fn sampler_bind_group(&self) -> &wgpu::BindGroup {
        self.sampler_bind_group.as_ref().expect(
            "the texture should have been created with wgpu::TextureUsages::TEXTURE_BINDING",
        )
    }

    pub(crate) fn write_storage_bind_group(&self) -> &wgpu::BindGroup {
        self.write_storage_bind_group.as_ref().expect(
            "the texture should have been created with wgpu::TextureUsages::STORAGE_BINDING",
        )
    }
}

pub(crate) fn sampler_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Sampler Texture Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

pub(crate) fn write_storage_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Write Storage Texture Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: wgpu::TextureFormat::Rgba32Float,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        }],
    })
}
