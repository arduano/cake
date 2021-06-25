use gfx_hal::{
    adapter::MemoryType,
    prelude::{PhysicalDevice, QueueFamily},
    queue::QueueGroup,
    window::Surface,
    Backend, Limits,
};

pub struct GDevice<B: Backend> {
    pub logical: B::Device,
    pub physical: B::PhysicalDevice,
    pub queues: QueueGroup<B>,
    pub sparsely_bound: bool,
}

impl<B: Backend> GDevice<B> {
    pub fn new(adapter: gfx_hal::adapter::Adapter<B>, surface: &B::Surface) -> GDevice<B> {
        // Build a new device and associated command queues
        let family = adapter
            .queue_families
            .iter()
            .find(|family| {
                surface.supports_queue_family(family) && family.queue_type().supports_graphics()
            })
            .expect("No queue family supports presentation");

        let physical = adapter.physical_device;
        let sparsely_bound = physical.features().contains(
            gfx_hal::Features::SPARSE_BINDING | gfx_hal::Features::SPARSE_RESIDENCY_IMAGE_2D,
        );
        let mut gpu = unsafe {
            physical
                .open(
                    &[(family, &[1.0])],
                    if sparsely_bound {
                        gfx_hal::Features::SPARSE_BINDING
                            | gfx_hal::Features::SPARSE_RESIDENCY_IMAGE_2D
                    } else {
                        gfx_hal::Features::empty()
                    },
                )
                .unwrap()
        };
        let queues = gpu.queue_groups.pop().unwrap();
        let logical = gpu.device;

        GDevice {
            logical,
            physical,
            queues,
            sparsely_bound,
        }
    }
}
