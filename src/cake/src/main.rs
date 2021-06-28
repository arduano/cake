extern crate gfx_backend as back;

use gfx_hal::{
    buffer, command, display,
    format::{self as f, Format},
    format::{AsFormat, ChannelType, Rgba8Srgb as ColorFormat, Swizzle},
    image as i, memory as m, pass,
    pass::Subpass,
    pool,
    prelude::*,
    pso,
    pso::{PipelineStage, ShaderStageFlags, VertexInputRate},
    queue::QueueGroup,
    window,
};
use graphics::{
    device::GDevice,
    pipeline::{GDescriptorSetLayout, GPipeline, GPipelineBuilder, GPipelineLayout},
    render_pass::{GRenderPass, GRenderPassBuilder},
};
use shaderc::ShaderKind;

use std::{
    borrow::{Borrow, BorrowMut},
    io::Cursor,
    iter,
    mem::{self, ManuallyDrop},
    ptr,
    rc::Rc,
    sync::Arc,
};

#[cfg_attr(rustfmt, rustfmt_skip)]
const DIMS: window::Extent2D = window::Extent2D { width: 1024, height: 768 };

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
struct Vertex {
    a_Pos: [f32; 2],
    a_Uv: [f32; 2],
}

#[cfg_attr(rustfmt, rustfmt_skip)]
const QUAD: [Vertex; 6] = [
    Vertex { a_Pos: [ 0f32, 1f32 ], a_Uv: [0.0, 1.0] },
    Vertex { a_Pos: [ 1f32, 1f32 ], a_Uv: [1.0, 1.0] },
    Vertex { a_Pos: [ 1f32, 0f32 ], a_Uv: [1.0, 0.0] },

    Vertex { a_Pos: [ 0f32, 1f32 ], a_Uv: [0.0, 1.0] },
    Vertex { a_Pos: [ 1f32, 0f32 ], a_Uv: [1.0, 0.0] },
    Vertex { a_Pos: [ 0f32, 0f32 ], a_Uv: [0.0, 0.0] },
];

struct IntVector4 {
    val1: i32,
    val2: i32,
    val3: i32,
    val4: i32,
}

struct UniformData {
    width: i32,
    height: i32,
    start: i32,
    end: i32,
    key_count: i32,
}

fn main() {
    let instance = back::Instance::create("gfx-rs quad", 1).expect("Failed to create an instance!");

    let adapter = {
        let mut adapters = instance.enumerate_adapters();
        for adapter in &adapters {
            println!("{:?}", adapter.info);
        }
        adapters.remove(0)
    };

    let event_loop = winit::event_loop::EventLoop::new();

    let wb = winit::window::WindowBuilder::new()
        .with_min_inner_size(winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
            64.0, 64.0,
        )))
        .with_inner_size(winit::dpi::Size::Physical(winit::dpi::PhysicalSize::new(
            DIMS.width,
            DIMS.height,
        )))
        .with_title("quad".to_string());

    // instantiate backend
    let window = wb.build(&event_loop).unwrap();

    let surface = unsafe {
        instance
            .create_surface(&window)
            .expect("Failed to create a surface!")
    };

    let mut renderer = Renderer::new(instance, surface, adapter);

    renderer.render();

    use std::sync::{Mutex, RwLock};
    use std::thread;

    let stopped = Arc::new(RwLock::new(false));

    let rend = Arc::new(Mutex::new(renderer));

    let stopped_listener = stopped.clone();
    let rend2 = rend.clone();
    thread::spawn(move || loop {
        let mut r = rend2.lock().unwrap();
        r.render();
        {
            let stopped = *stopped_listener.read().unwrap();
            if stopped {
                break;
            }
        }
    });

    // It is important that the closure move captures the Renderer,
    // otherwise it will not be dropped when the event loop exits.
    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    {
                        let mut s = stopped.write().unwrap();
                        *s = true;
                    }
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
                winit::event::WindowEvent::KeyboardInput {
                    input:
                        winit::event::KeyboardInput {
                            virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = winit::event_loop::ControlFlow::Exit,
                winit::event::WindowEvent::Resized(dims) => {
                    println!("resized to {:?}", dims);
                    let mut r = rend.lock().unwrap();

                    r.dimensions = window::Extent2D {
                        width: dims.width,
                        height: dims.height,
                    };
                    r.recreate_swapchain();
                }
                _ => {}
            },
            winit::event::Event::RedrawEventsCleared => {
                // let mut r = rend.lock().unwrap();
                // r.render();
            }
            _ => {}
        }
    });
}

struct Renderer<B: gfx_hal::Backend> {
    desc_pool: ManuallyDrop<B::DescriptorPool>,
    surface: ManuallyDrop<B::Surface>,
    format: gfx_hal::format::Format,
    dimensions: window::Extent2D,
    viewport: pso::Viewport,
    render_pass: Arc<GRenderPass<B>>,
    framebuffer: ManuallyDrop<B::Framebuffer>,
    pipeline: GPipeline<B>,
    desc_set: Option<B::DescriptorSet>,
    submission_complete_semaphores: Vec<B::Semaphore>,
    submission_complete_fences: Vec<B::Fence>,
    cmd_pools: Vec<B::CommandPool>,
    cmd_buffers: Vec<B::CommandBuffer>,
    vertex_buffer: ManuallyDrop<B::Buffer>,
    buffer_memory: ManuallyDrop<B::Memory>,
    frames_in_flight: usize,
    frame: u64,
    // These members are dropped in the declaration order.
    gdevice: GDevice<B>,
    instance: B::Instance,
}

impl<B> Renderer<B>
where
    B: gfx_hal::Backend,
{
    fn new(
        instance: B::Instance,
        mut surface: B::Surface,
        adapter: gfx_hal::adapter::Adapter<B>,
    ) -> Renderer<B> {
        let memory_types = adapter.physical_device.memory_properties().memory_types;
        let limits = adapter.physical_device.properties().limits;

        let gdevice = GDevice::new(adapter, &surface);
        let device = gdevice.logical.clone();

        let command_pool = unsafe {
            device.create_command_pool(gdevice.queues.family, pool::CommandPoolCreateFlags::empty())
        }
        .expect("Can't create command pool");

        // Setup renderpass and pipeline
        let set_layout = Arc::new(GDescriptorSetLayout::new(&gdevice, vec![].into_iter()));

        // Descriptors
        let mut desc_pool = ManuallyDrop::new(
            unsafe {
                device.create_descriptor_pool(
                    1, // sets
                    vec![
                        pso::DescriptorRangeDesc {
                            ty: pso::DescriptorType::Image {
                                ty: pso::ImageDescriptorType::Sampled {
                                    with_sampler: false,
                                },
                            },
                            count: 1,
                        },
                        pso::DescriptorRangeDesc {
                            ty: pso::DescriptorType::Sampler,
                            count: 1,
                        },
                    ]
                    .into_iter(),
                    pso::DescriptorPoolCreateFlags::empty(),
                )
            }
            .expect("Can't create descriptor pool"),
        );
        let desc_set = unsafe { desc_pool.allocate_one(&set_layout.layout()) }.unwrap();

        // Buffer allocations
        println!("Memory types: {:?}", memory_types);
        let non_coherent_alignment = limits.non_coherent_atom_size as u64;

        let buffer_stride = mem::size_of::<Vertex>() as u64;
        let buffer_len = QUAD.len() as u64 * buffer_stride;
        assert_ne!(buffer_len, 0);
        let padded_buffer_len = ((buffer_len + non_coherent_alignment - 1)
            / non_coherent_alignment)
            * non_coherent_alignment;

        let mut vertex_buffer = ManuallyDrop::new(
            unsafe {
                device.create_buffer(
                    padded_buffer_len,
                    buffer::Usage::VERTEX,
                    m::SparseFlags::empty(),
                )
            }
            .unwrap(),
        );

        let buffer_req = unsafe { device.get_buffer_requirements(&vertex_buffer) };

        let upload_type = memory_types
            .iter()
            .enumerate()
            .position(|(id, mem_type)| {
                // type_mask is a bit field where each bit represents a memory type. If the bit is set
                // to 1 it means we can use that type for our buffer. So this code finds the first
                // memory type that has a `1` (or, is allowed), and is visible to the CPU.
                buffer_req.type_mask & (1 << id) != 0
                    && mem_type.properties.contains(m::Properties::CPU_VISIBLE)
            })
            .unwrap()
            .into();

        // TODO: check transitions: read/write mapping and vertex buffer read
        let buffer_memory = unsafe {
            let mut memory = device
                .allocate_memory(upload_type, buffer_req.size)
                .unwrap();
            device
                .bind_buffer_memory(&memory, 0, &mut vertex_buffer)
                .unwrap();
            let mapping = device.map_memory(&mut memory, m::Segment::ALL).unwrap();
            ptr::copy_nonoverlapping(QUAD.as_ptr() as *const u8, mapping, buffer_len as usize);
            device
                .flush_mapped_memory_ranges(iter::once((&memory, m::Segment::ALL)))
                .unwrap();
            device.unmap_memory(&mut memory);
            ManuallyDrop::new(memory)
        };

        let caps = surface.capabilities(&gdevice.physical);
        let formats = surface.supported_formats(&gdevice.physical);
        println!("formats: {:?}", formats);
        let format = formats.map_or(f::Format::Rgba8Srgb, |formats| {
            formats
                .iter()
                .find(|format| format.base_format().1 == ChannelType::Srgb)
                .map(|format| *format)
                .unwrap_or(formats[0])
        });

        let swap_config = window::SwapchainConfig::from_caps(&caps, format, DIMS);
        let fat = swap_config.framebuffer_attachment();
        println!("{:?}", swap_config);
        let extent = swap_config.extent;
        unsafe {
            surface
                .configure_swapchain(&device, swap_config)
                .expect("Can't configure swapchain");
        };

        let render_pass = Arc::new(GRenderPassBuilder::new(format).build(&gdevice));

        let swap_config = window::SwapchainConfig::from_caps(&caps, format, DIMS);
        let framebuffer = ManuallyDrop::new(unsafe {
            device
                .create_framebuffer(
                    &render_pass.render_pass(),
                    iter::once(fat),
                    swap_config.extent.to_extent(),
                )
                .unwrap()
        });

        // Define maximum number of frames we want to be able to be "in flight" (being computed
        // simultaneously) at once
        let frames_in_flight = 3;

        // The number of the rest of the resources is based on the frames in flight.
        let mut submission_complete_semaphores = Vec::with_capacity(frames_in_flight);
        let mut submission_complete_fences = Vec::with_capacity(frames_in_flight);
        // Note: We don't really need a different command pool per frame in such a simple demo like this,
        // but in a more 'real' application, it's generally seen as optimal to have one command pool per
        // thread per frame. There is a flag that lets a command pool reset individual command buffers
        // which are created from it, but by default the whole pool (and therefore all buffers in it)
        // must be reset at once. Furthermore, it is often the case that resetting a whole pool is actually
        // faster and more efficient for the hardware than resetting individual command buffers, so it's
        // usually best to just make a command pool for each set of buffers which need to be reset at the
        // same time (each frame). In our case, each pool will only have one command buffer created from it,
        // though.
        let mut cmd_pools = Vec::with_capacity(frames_in_flight);
        let mut cmd_buffers = Vec::with_capacity(frames_in_flight);

        cmd_pools.push(command_pool);
        for _ in 1..frames_in_flight {
            unsafe {
                cmd_pools.push(
                    device
                        .create_command_pool(
                            gdevice.queues.family,
                            pool::CommandPoolCreateFlags::empty(),
                        )
                        .expect("Can't create command pool"),
                );
            }
        }

        for i in 0..frames_in_flight {
            submission_complete_semaphores.push(
                device
                    .create_semaphore()
                    .expect("Could not create semaphore"),
            );
            submission_complete_fences
                .push(device.create_fence(true).expect("Could not create fence"));
            cmd_buffers.push(unsafe { cmd_pools[i].allocate_one(command::Level::Primary) });
        }

        let pipeline_cache_path = "quad_pipeline_cache";

        let previous_pipeline_cache_data = std::fs::read(pipeline_cache_path);

        if let Err(error) = previous_pipeline_cache_data.as_ref() {
            println!("Error loading the previous pipeline cache data: {}", error);
        }

        let pipeline_layout = Arc::new(GPipelineLayout::new(&gdevice, set_layout));
        let pipeline = {
            use graphics::shaders::GShaderModule;

            let vs_module = GShaderModule::<B>::new(
                &gdevice,
                include_str!("./data/quad.vert"),
                ShaderKind::Vertex,
            );
            let fs_module = GShaderModule::<B>::new(
                &gdevice,
                include_str!("./data/quad.frag"),
                ShaderKind::Fragment,
            );

            let spec = gfx_hal::spec_const_list![0.8f32];

            let vertex_buffers = vec![pso::VertexBufferDesc {
                binding: 0,
                stride: mem::size_of::<Vertex>() as u32,
                rate: VertexInputRate::Vertex,
            }];

            let attributes = vec![
                pso::AttributeDesc {
                    location: 0,
                    binding: 0,
                    element: pso::Element {
                        format: f::Format::Rg32Sfloat,
                        offset: 0,
                    },
                },
                pso::AttributeDesc {
                    location: 1,
                    binding: 0,
                    element: pso::Element {
                        format: f::Format::Rg32Sfloat,
                        offset: 8,
                    },
                },
            ];

            GPipelineBuilder::new(
                &vertex_buffers,
                &attributes,
                vs_module.entrypoint_with(None, Some(spec)),
                fs_module.entrypoint(),
                pipeline_layout,
                render_pass.clone(),
            )
            .build(&gdevice)
        };

        // Rendering setup
        let viewport = pso::Viewport {
            rect: pso::Rect {
                x: 0,
                y: 0,
                w: extent.width as _,
                h: extent.height as _,
            },
            depth: 0.0..1.0,
        };

        Renderer {
            instance,
            gdevice,
            desc_pool,
            surface: ManuallyDrop::new(surface),
            format,
            dimensions: DIMS,
            viewport,
            render_pass,
            framebuffer,
            pipeline,
            desc_set: Some(desc_set),
            submission_complete_semaphores,
            submission_complete_fences,
            cmd_pools,
            cmd_buffers,
            vertex_buffer,
            buffer_memory,
            frames_in_flight,
            frame: 0,
        }
    }

    fn recreate_swapchain(&mut self) {
        let caps = self.surface.capabilities(&self.gdevice.physical);
        let swap_config = window::SwapchainConfig::from_caps(&caps, self.format, self.dimensions);
        println!("{:?}", swap_config);

        let extent = swap_config.extent.to_extent();
        self.viewport.rect.w = extent.width as _;
        self.viewport.rect.h = extent.height as _;

        let device = &self.gdevice.logical;
        unsafe {
            device.wait_idle().unwrap();
            device.destroy_framebuffer(ManuallyDrop::into_inner(ptr::read(&self.framebuffer)));
            self.framebuffer = ManuallyDrop::new(
                device
                    .create_framebuffer(
                        &self.render_pass.render_pass(),
                        iter::once(swap_config.framebuffer_attachment()),
                        extent,
                    )
                    .unwrap(),
            )
        };

        unsafe {
            self.surface
                .configure_swapchain(device, swap_config)
                .expect("Can't create swapchain");
        }
    }

    fn render(&mut self) {
        // Start a RenderDoc capture, which allows analyzing the rendering pipeline
        self.gdevice.logical.start_capture();

        let surface_image = unsafe {
            match self.surface.acquire_image(!0) {
                Ok((image, _)) => image,
                Err(_) => {
                    self.recreate_swapchain();
                    return;
                }
            }
        };

        // Compute index into our resource ring buffers based on the frame number
        // and number of frames in flight. Pay close attention to where this index is needed
        // versus when the swapchain image index we got from acquire_image is needed.
        let frame_idx = self.frame as usize % self.frames_in_flight;

        // Wait for the fence of the previous submission of this frame and reset it; ensures we are
        // submitting only up to maximum number of frames_in_flight if we are submitting faster than
        // the gpu can keep up with. This would also guarantee that any resources which need to be
        // updated with a CPU->GPU data copy are not in use by the GPU, so we can perform those updates.
        // In this case there are none to be done, however.
        unsafe {
            let device = &self.gdevice.logical;
            let fence = &mut self.submission_complete_fences[frame_idx];
            device
                .wait_for_fence(fence, !0)
                .expect("Failed to wait for fence");
            device.reset_fence(fence).expect("Failed to reset fence");
            self.cmd_pools[frame_idx].reset(false);
        }

        // Rendering
        let cmd_buffer = &mut self.cmd_buffers[frame_idx];
        unsafe {
            cmd_buffer.begin_primary(command::CommandBufferFlags::ONE_TIME_SUBMIT);

            cmd_buffer.set_viewports(0, iter::once(self.viewport.clone()));
            cmd_buffer.set_scissors(0, iter::once(self.viewport.rect));
            cmd_buffer.bind_graphics_pipeline(&self.pipeline.pipeline());
            cmd_buffer.bind_graphics_descriptor_sets(
                &self.pipeline.layout().layout(),
                0,
                self.desc_set.as_ref().into_iter(),
                iter::empty(),
            );
            cmd_buffer.bind_vertex_buffers(
                0,
                iter::once((&*self.vertex_buffer, buffer::SubRange::WHOLE)),
            );

            cmd_buffer.begin_render_pass(
                &self.render_pass.render_pass(),
                &self.framebuffer,
                self.viewport.rect,
                iter::once(command::RenderAttachmentInfo {
                    image_view: surface_image.borrow(),
                    clear_value: command::ClearValue {
                        color: command::ClearColor {
                            float32: [0.8, 0.8, 0.8, 1.0],
                        },
                    },
                }),
                command::SubpassContents::Inline,
            );
            cmd_buffer.draw(0..6, 0..1);
            cmd_buffer.end_render_pass();
            cmd_buffer.finish();

            self.gdevice.queues.queues[0].submit(
                iter::once(&*cmd_buffer),
                iter::empty(),
                iter::once(&self.submission_complete_semaphores[frame_idx]),
                Some(&mut self.submission_complete_fences[frame_idx]),
            );

            // present frame
            let result = self.gdevice.queues.queues[0].present(
                &mut self.surface,
                surface_image,
                Some(&mut self.submission_complete_semaphores[frame_idx]),
            );

            if result.is_err() {
                self.recreate_swapchain();
            }
        }

        // Increment our frame
        self.frame += 1;

        // End the RenderDoc capture
        self.gdevice.logical.stop_capture();
    }
}

impl<B> Drop for Renderer<B>
where
    B: gfx_hal::Backend,
{
    fn drop(&mut self) {
        let device = &self.gdevice.logical;

        device.wait_idle().unwrap();
        unsafe {
            // TODO: When ManuallyDrop::take (soon to be renamed to ManuallyDrop::read) is stabilized we should use that instead.
            let _ = self.desc_set.take();
            device.destroy_descriptor_pool(ManuallyDrop::into_inner(ptr::read(&self.desc_pool)));

            device.destroy_buffer(ManuallyDrop::into_inner(ptr::read(&self.vertex_buffer)));
            for p in self.cmd_pools.drain(..) {
                device.destroy_command_pool(p);
            }
            for s in self.submission_complete_semaphores.drain(..) {
                device.destroy_semaphore(s);
            }
            for f in self.submission_complete_fences.drain(..) {
                device.destroy_fence(f);
            }
            device.destroy_framebuffer(ManuallyDrop::into_inner(ptr::read(&self.framebuffer)));
            self.surface.unconfigure_swapchain(&device);
            device.free_memory(ManuallyDrop::into_inner(ptr::read(&self.buffer_memory)));
            let surface = ManuallyDrop::into_inner(ptr::read(&self.surface));
            self.instance.destroy_surface(surface);
        }
        println!("DROPPED!");
    }
}
