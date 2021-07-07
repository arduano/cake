use std::{sync::Arc, time::Instant};

use futures::executor::block_on;
use wgpu::{Adapter, Device, Instance, Queue};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};

use crate::window::{DisplayWindow, WindowData};

pub struct ApplicationGraphics {
    adapter: Adapter,
    device: Arc<Device>,
    queue: Arc<Queue>,
    instance: Arc<Instance>,
}

impl ApplicationGraphics {
    pub fn create(instance: Instance, window: &WindowData) -> Self {
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&window.surface),
        }))
        .unwrap();

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))
        .unwrap();

        ApplicationGraphics {
            adapter: adapter,
            device: Arc::new(device),
            queue: Arc::new(queue),
            instance: Arc::new(instance),
        }
    }

    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }
    pub fn queue(&self) -> &Arc<Queue> {
        &self.queue
    }
    pub fn instance(&self) -> &Arc<Instance> {
        &self.instance
    }
}

pub fn run_application<W: 'static + DisplayWindow>(
    mut event_loop: EventLoop<()>,
    mut main_window: W,
) {
    let mut last_frame = Instant::now();

    event_loop.run_return(move |event, _, control_flow| {
        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Poll
        };

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => main_window.reset_swapchain(),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                main_window.window_data().window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let curr_frame = Instant::now();
                main_window.render(last_frame.elapsed());
                last_frame = curr_frame;
            }
            _ => (),
        }

        match event {
            Event::WindowEvent { .. } => {
                main_window.handle_platform_event(&event);
            }
            _ => {}
        }
    });
}
