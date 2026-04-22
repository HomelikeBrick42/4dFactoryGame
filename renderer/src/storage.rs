use bytemuck::NoUninit;
use std::{fmt::Debug, marker::PhantomData};

pub trait StorageElement: Sized {
    type GpuType: From<Self> + NoUninit;
}

pub struct Id<T: StorageElement>(usize, PhantomData<T>);

impl<T: StorageElement> Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Id<{}>({})", std::any::type_name::<T>(), self.0)
    }
}

impl<T: StorageElement> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: StorageElement> Copy for Id<T> {}

pub struct Storage<T: StorageElement> {
    label: Box<str>,
    indices: Vec<usize>,
    ids: Vec<Id<T>>,
    len: usize,
    data: wgpu::Buffer,
}

impl<T: StorageElement> Storage<T> {
    pub fn new(device: &wgpu::Device, label: impl Into<Box<str>>) -> Self {
        let label = label.into();
        let data = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&label),
            size: size_of::<T::GpuType>() as _,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            label,
            indices: vec![],
            ids: vec![],
            len: 0,
            data,
        }
    }

    /// returns (id, was_reallocated)
    pub fn add(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, value: T) -> (Id<T>, bool) {
        if let Some(&id) = self.ids.get(self.len) {
            self.len += 1;
            self.update(queue, id, value);
            (id, false)
        } else {
            self.indices.push(self.len);
            let id = Id(self.len, PhantomData);
            self.ids.push(id);
            self.len += 1;

            let mut was_reallocated = false;
            if (self.len * size_of::<T::GpuType>()) > self.data.size() as _ {
                was_reallocated = true;
                let old_data = std::mem::replace(
                    &mut self.data,
                    device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some(&self.label),
                        size: ((self.len + (self.len >> 1)) * size_of::<T::GpuType>()) as _,
                        usage: wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::COPY_SRC
                            | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }),
                );

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Buffer Copy"),
                });
                encoder.copy_buffer_to_buffer(&old_data, 0, &self.data, 0, old_data.size());
                queue.submit(std::iter::once(encoder.finish()));
            }

            self.update(queue, id, value);
            (id, was_reallocated)
        }
    }

    pub fn remove(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, id: Id<T>) {
        assert!(
            self.indices[id.0] < self.len,
            "invalid id, {} was >= than {}",
            id.0,
            self.len
        );

        let last_id = self.ids[self.len - 1];

        let temp_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size_of::<T::GpuType>() as _,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Buffer Copy"),
        });
        encoder.copy_buffer_to_buffer(
            &self.data,
            (self.indices[last_id.0] * size_of::<T::GpuType>()) as _,
            &temp_buffer,
            0,
            temp_buffer.size(),
        );
        encoder.copy_buffer_to_buffer(
            &temp_buffer,
            0,
            &self.data,
            (self.indices[id.0] * size_of::<T::GpuType>()) as _,
            temp_buffer.size(),
        );
        queue.submit(std::iter::once(encoder.finish()));

        self.ids.swap(self.indices[id.0], self.indices[last_id.0]);
        self.indices.swap(id.0, last_id.0);
        self.len -= 1;
    }

    pub fn update(&mut self, queue: &wgpu::Queue, id: Id<T>, value: T) {
        assert!(
            self.indices[id.0] < self.len,
            "invalid id, {} was >= than {}",
            id.0,
            self.len
        );
        queue.write_buffer(
            &self.data,
            (self.indices[id.0] * size_of::<T::GpuType>()) as _,
            bytemuck::bytes_of(&T::GpuType::from(value)),
        );
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.data
    }
}
