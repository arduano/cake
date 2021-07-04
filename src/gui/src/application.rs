use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
    time::Instant,
};

use futures::executor::block_on;
use imgui::{im_str, Condition, Context, ImColor32, MouseCursor, StyleVar};
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

struct OpenDisplayWindow<Model, Ev> {
    window: Box<dyn DisplayWindow<Model, Ev>>,
    imgui: ImGuiDisplayContext,
    last_frame: Instant,
    is_open: bool,
}

impl<Model, Ev> OpenDisplayWindow<Model, Ev> {
    pub fn new(window: Box<dyn DisplayWindow<Model, Ev>>, graphics: &ApplicationGraphics) -> Self {
        let mut imgui = imgui::Context::create();
        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            &window.window_data().window,
            imgui_winit_support::HiDpiMode::Default,
        );
        imgui.set_ini_filename(None);

        let renderer_config = RendererConfig {
            texture_format: window.swapchain_texture_format(),
            ..Default::default()
        };

        let renderer = Renderer::new(
            &mut imgui,
            graphics.device(),
            graphics.queue(),
            renderer_config,
        );

        let last_frame = Instant::now();

        OpenDisplayWindow {
            window,
            imgui: ImGuiDisplayContext {
                platform,
                imgui,
                renderer,
            },
            last_frame,
            is_open: true,
        }
    }

    pub fn window_id(&self) -> WindowId {
        self.window.window_data().window.id()
    }

    pub fn render(&mut self, graphics: &mut ApplicationGraphics, model: &mut Model) {
        let now = Instant::now();
        let delta = now - self.last_frame;
        self.last_frame = now;

        let imgui_context = &mut self.imgui;

        self.window.render(graphics, imgui_context, model, delta);
    }

    pub fn handle_platform_event(&mut self, event: &Event<Ev>) {
        let imgui_context = &mut self.imgui;
        let imgui = &mut imgui_context.imgui;
        let platform = &mut imgui_context.platform;

        let window = &self.window;
        let inner_window_data = window.window_data();

        let window = &inner_window_data.window;

        platform.handle_event(imgui.io_mut(), &window, event);
    }
}

struct WindowMap<Model, Ev> {
    window_map: HashMap<WindowId, Mutex<OpenDisplayWindow<Model, Ev>>>,
}

impl<Model, Ev> WindowMap<Model, Ev> {
    pub fn new() -> Self {
        WindowMap {
            window_map: HashMap::<WindowId, Mutex<OpenDisplayWindow<Model, Ev>>>::new(),
        }
    }

    pub fn insert(
        &mut self,
        window: Box<dyn DisplayWindow<Model, Ev>>,
        graphics: &ApplicationGraphics,
    ) {
        let window = OpenDisplayWindow::new(window, graphics);
        self.window_map
            .insert(window.window_id(), Mutex::new(window));
    }

    pub fn remove(&mut self, id: &WindowId) {
        self.window_map.remove(id);
    }

    pub fn get(&mut self, id: &WindowId) -> Option<&Mutex<OpenDisplayWindow<Model, Ev>>> {
        self.window_map.get(id)
    }

    pub fn get_mut(&mut self, id: &WindowId) -> Option<&mut Mutex<OpenDisplayWindow<Model, Ev>>> {
        self.window_map.get_mut(id)
    }
}

pub fn run_application_default<Model, Ev: 'static + Copy + Send>(
    instance: Instance,
    event_loop: EventLoop<Ev>,
    model: Box<Model>,
    main_window: Box<dyn DisplayWindow<Model, Ev>>,
    e: Ev,
) {
    let window = main_window.window_data();
    let graphics = ApplicationGraphics::create(instance, &window);

    run_application(graphics, event_loop, model, main_window, e);
}

pub fn run_application<Model, Ev: 'static + Copy + Send>(
    mut graphics: ApplicationGraphics,
    mut event_loop: EventLoop<Ev>,
    mut model: Box<Model>,
    main_window: Box<dyn DisplayWindow<Model, Ev>>,
    e: Ev,
) {
    let mut window_map = WindowMap::new();

    let main_window_id = main_window.window_data().window.id();

    window_map.insert(main_window, &graphics);

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
                let mut window_data = window_map.get_mut(&window_id).unwrap().lock().unwrap();

                window_data.window.reset_swapchain()
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                let main_window = window_map.get(&main_window_id).unwrap().lock().unwrap();
                main_window.window.window_data().window.request_redraw();
            }
            Event::RedrawRequested(window_id) => {
                let mut window_data = window_map.get_mut(&window_id).unwrap().lock().unwrap();

                window_data.render(&mut graphics, &mut model)
            }
            _ => (),
        }

        match event {
            Event::WindowEvent { window_id, .. } => {
                let window_data = &mut window_map.get_mut(&window_id).unwrap().lock().unwrap();

                window_data.handle_platform_event(&event);
            }
            _ => {}
        }
    });
}
