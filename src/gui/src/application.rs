use std::{
    collections::{hash_map::Keys, HashMap},
    sync::Arc,
    time::Instant,
};

use futures::executor::block_on;
use imgui::Context;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use wgpu::{Adapter, Device, Instance, Queue};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowId,
};

use crate::window::{DisplayWindow, ImGuiDisplayContext, WindowData};

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

struct OpenDisplayWindow {
    window: Box<dyn DisplayWindow>,
    imgui: ImGuiDisplayContext,
    last_frame: Instant,
}

impl OpenDisplayWindow {
    pub fn new(
        window: Box<dyn DisplayWindow>,
        imgui: &mut Context,
        graphics: &ApplicationGraphics,
    ) -> Self {
        let mut platform = WinitPlatform::init(imgui);
        platform.attach_window(
            imgui.io_mut(),
            &window.window_data().window,
            imgui_winit_support::HiDpiMode::Default,
        );

        let renderer_config = RendererConfig {
            texture_format: window.swapchain_texture_format(),
            ..Default::default()
        };

        let renderer = Renderer::new(imgui, graphics.device(), graphics.queue(), renderer_config);

        let last_frame = Instant::now();

        OpenDisplayWindow {
            window,
            imgui: ImGuiDisplayContext { platform, renderer },
            last_frame,
        }
    }

    pub fn window_id(&self) -> WindowId {
        self.window.window_data().window.id()
    }

    pub fn render(&mut self, graphics: &mut ApplicationGraphics, imgui: &mut Context) {
        let now = Instant::now();
        let delta = now - self.last_frame;
        self.last_frame = now;

        let imgui_context = &mut self.imgui;

        self.window.render(graphics, imgui_context, imgui, delta);
    }

    pub fn handle_platform_event(&mut self, imgui: &mut Context, event: &Event<()>) {
        let imgui_context = &mut self.imgui;
        let platform = &mut imgui_context.platform;

        let window = &self.window;
        let inner_window_data = window.window_data();

        let window = &inner_window_data.window;

        platform.handle_event(imgui.io_mut(), &window, event);
    }
}

struct WindowMap {
    window_map: HashMap<WindowId, OpenDisplayWindow>,
}

impl WindowMap {
    pub fn new() -> Self {
        WindowMap {
            window_map: HashMap::<WindowId, OpenDisplayWindow>::new(),
        }
    }

    pub fn insert(
        &mut self,
        window: Box<dyn DisplayWindow>,
        imgui: &mut Context,
        graphics: &ApplicationGraphics,
    ) {
        let window = OpenDisplayWindow::new(window, imgui, graphics);
        self.window_map.insert(window.window_id(), window);
    }

    // pub fn remove(&mut self, id: &WindowId) {
    //     self.window_map.remove(id);
    // }

    pub fn get(&self, id: &WindowId) -> Option<&OpenDisplayWindow> {
        self.window_map.get(id)
    }

    pub fn get_mut(&mut self, id: &WindowId) -> Option<&mut OpenDisplayWindow> {
        self.window_map.get_mut(id)
    }

    pub fn keys(&self) -> Keys<WindowId, OpenDisplayWindow> {
        self.window_map.keys()
    }
}

pub fn run_application<W: 'static + DisplayWindow>(
    mut graphics: ApplicationGraphics,
    mut event_loop: EventLoop<()>,
    main_window: W,
) {
    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);

    let mut window_map = WindowMap::new();

    window_map.insert(Box::new(main_window), &mut imgui, &graphics);

    // let event_pipe = Arc::new(Mutex::new(event_loop.create_proxy()));

    event_loop.run_return(move |event, _, control_flow| {
        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Poll
        };

        match event {
            Event::UserEvent(_) => {
                println!("User event fired!");
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                window_id,
            } => {
                let window_data = window_map.get_mut(&window_id).unwrap();

                window_data.window.reset_swapchain()
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                for id in window_map.keys() {
                    let window = window_map.get(&id).unwrap();
                    window.window.window_data().window.request_redraw();
                }
            }
            Event::RedrawRequested(window_id) => {
                let window_data = window_map.get_mut(&window_id).unwrap();

                window_data.render(&mut graphics, &mut imgui);
            }
            _ => (),
        }

        match event {
            Event::WindowEvent { window_id, .. } => {
                let window_data = &mut window_map.get_mut(&window_id).unwrap();

                window_data.handle_platform_event(&mut imgui, &event);
            }
            _ => {}
        }
    });
}
