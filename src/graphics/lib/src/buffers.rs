use getset::Getters;
use gfx_hal::{
    adapter::MemoryType,
    buffer,
    device::Device,
    pso::{self, EntryPoint, Specialization},
    Backend,
    memory,
};
use shaderc::ShaderKind;
use std::{cell::RefCell, mem::size_of, mem::ManuallyDrop, ptr, sync::Arc};

use crate::{device::GDevice, return_option};

pub struct GBuffer<B: Backend> {
    memory: Option<B::Memory>,
    buffer: Option<B::Buffer>,
    device: Arc<B::Device>,
    size: u64,
}

impl<B: Backend> GBuffer<B> {
    fn buffer(&self) -> &B::Buffer {
        return_option!(&self.buffer);
    }

    unsafe fn new<T>(
        gdevice: &GDevice<B>,
        data_source: &[T],
        usage: buffer::Usage,
        memory_types: &[MemoryType],
    ) -> Self
    where
        T: Copy,
    {
        let device = gdevice.logical.clone();

        let mut memory: B::Memory;
        let mut buffer: B::Buffer;
        let size: u64;

        let stride = size_of::<T>();
        let upload_size = data_source.len() * stride;

        {
            buffer = device
                .create_buffer(upload_size as u64, usage, memory::SparseFlags::empty())
                .unwrap();
            let mem_req = device.get_buffer_requirements(&buffer);

            // A note about performance: Using CPU_VISIBLE memory is convenient because it can be
            // directly memory mapped and easily updated by the CPU, but it is very slow and so should
            // only be used for small pieces of data that need to be updated very frequently. For something like
            // a vertex buffer that may be much larger and should not change frequently, you should instead
            // use a DEVICE_LOCAL buffer that gets filled by copying data from a CPU_VISIBLE staging buffer.
            let upload_type = memory_types
                .iter()
                .enumerate()
                .position(|(id, mem_type)| {
                    mem_req.type_mask & (1 << id) != 0
                        && mem_type
                            .properties
                            .contains(memory::Properties::CPU_VISIBLE | memory::Properties::COHERENT)
                })
                .unwrap()
                .into();

            memory = device.allocate_memory(upload_type, mem_req.size).unwrap();
            device.bind_buffer_memory(&memory, 0, &mut buffer).unwrap();
            size = mem_req.size;

            // TODO: check transitions: read/write mapping and vertex buffer read
            let mapping = device.map_memory(&mut memory, memory::Segment::ALL).unwrap();
            ptr::copy_nonoverlapping(data_source.as_ptr() as *const u8, mapping, upload_size);
            device.unmap_memory(&mut memory);
        }

        GBuffer {
            memory: Some(memory),
            buffer: Some(buffer),
            device,
            size,
        }
    }

    fn update_data<T>(&mut self, offset: u64, data_source: &[T])
    where
        T: Copy,
    {
        let stride = size_of::<T>();
        let upload_size = data_source.len() * stride;

        assert!(offset + upload_size as u64 <= self.size);
        let memory = self.memory.as_mut().unwrap();

        unsafe {
            let mapping = self.device
                .map_memory(memory, memory::Segment { offset, size: None })
                .unwrap();
            ptr::copy_nonoverlapping(data_source.as_ptr() as *const u8, mapping, upload_size);
            self.device.unmap_memory(memory);
        }
    }
}
