use std::{
    rc::Rc,
    sync::{Arc, RwLock},
};

use futures::executor::block_on;
use wgpu::{Adapter, Device, Instance, Queue, Surface};
use winit::{event_loop::EventLoop, window::Window};

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

pub fn run_application_default<Model, Ev: 'static>(main_window: Box<dyn DisplayWindow<Model, Ev>>) {
    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

    let mut event_loop = EventLoop::<Ev>::with_user_event();

    let window = main_window.create_window(&instance, &event_loop);
    let graphics = ApplicationGraphics::create(instance, &window);
}

pub fn run_application<Model, Ev>(main_window: Box<dyn DisplayWindow<Model, Ev>>) {}
