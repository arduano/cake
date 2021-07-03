use std::{collections::HashMap, sync::Arc, time::Instant};

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

struct OpenDisplayWindow<Model, Ev> {
    window: Box<dyn DisplayWindow<Model, Ev>>,
    platform: WinitPlatform,
    imgui: Context,
    renderer: Renderer,
    last_frame: Instant,
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
            platform,
            imgui,
            renderer,
            last_frame,
        }
    }
}

struct WindowMap<Model, Ev> {
    window_map: HashMap<WindowId, OpenDisplayWindow<Model, Ev>>,
}

impl<Model, Ev> WindowMap<Model, Ev> {
    pub fn new() -> Self {
        WindowMap {
            window_map: HashMap::<WindowId, OpenDisplayWindow<Model, Ev>>::new(),
        }
    }

    pub fn insert(
        &mut self,
        window: Box<dyn DisplayWindow<Model, Ev>>,
        graphics: &ApplicationGraphics,
    ) {
        self.window_map.insert(
            window.window_data().window.id(),
            OpenDisplayWindow::new(window, graphics),
        );
    }

    pub fn remove(&mut self, id: &WindowId) {
        self.window_map.remove(id);
    }

    pub fn get(&mut self, id: &WindowId) -> Option<&OpenDisplayWindow<Model, Ev>> {
        self.window_map.get(id)
    }

    pub fn get_mut(&mut self, id: &WindowId) -> Option<&mut OpenDisplayWindow<Model, Ev>> {
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
    graphics: ApplicationGraphics,
    mut event_loop: EventLoop<Ev>,
    model: Box<Model>,
    main_window: Box<dyn DisplayWindow<Model, Ev>>,
    e: Ev,
) {
    let mut window_map = WindowMap::new();

    let main_window_id = main_window.window_data().window.id();

    window_map.insert(main_window, &graphics);

    // let event_pipe = Arc::new(Mutex::new(event_loop.create_proxy()));

    event_loop.run_return(move |event, _, control_flow| {
        // let window_map = &mut window_map;

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

                window_data
                    .window
                    .create_and_set_swapchain(&graphics, &model);
                // let event_pipe = event_pipe.clone();
                // thread::spawn(move || {
                //     event_pipe.lock().unwrap().send_event(e);
                // });
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                let main_window = window_map.get(&main_window_id).unwrap();
                main_window.window.window_data().window.request_redraw();
            }
            Event::RedrawRequested(window_id) => {
                let window_data = window_map.get_mut(&window_id).unwrap();

                let imgui = &mut window_data.imgui;
                let platform = &mut window_data.platform;
                let renderer = &mut window_data.renderer;

                let window = &mut window_data.window;
                if window.window_data().swap_chain.is_none() {
                    window.create_and_set_swapchain(&graphics, &model);
                }

                let swap_chain = window.window_data().swap_chain.as_ref().unwrap();
                let window = &window.window_data().window;

                let now = Instant::now();
                imgui
                    .io_mut()
                    .update_delta_time(now - window_data.last_frame);
                window_data.last_frame = now;

                let frame = match swap_chain.get_current_frame() {
                    Ok(frame) => frame,
                    Err(e) => {
                        eprintln!("dropped frame: {:?}", e);
                        return;
                    }
                };
                platform
                    .prepare_frame(imgui.io_mut(), &window)
                    .expect("Failed to prepare frame");
                let ui = imgui.frame();

                // Render example normally at background
                let size = ui.io().display_size;

                // Store the new size of Image() or None to indicate that the window is collapsed.
                let mut new_example_size: Option<[f32; 2]> = None;

                let nopadding = ui.push_style_vars(&[
                    StyleVar::WindowPadding([-1.0, -1.0]),
                    StyleVar::WindowBorderSize(0.0),
                ]);

                imgui::Window::new(im_str!("Root"))
                    .no_nav()
                    .title_bar(false)
                    .draw_background(false)
                    .movable(false)
                    .scrollable(false)
                    .bring_to_front_on_focus(false)
                    .collapsible(false)
                    .resizable(false)
                    .collapsed(false, Condition::Always)
                    .always_use_window_padding(false)
                    .size(size, Condition::Always)
                    .position([0.0, 0.0], Condition::Always)
                    .build(&ui, || {
                        new_example_size = Some(ui.content_region_avail());
                        // imgui::Image::new(example_texture_id, new_example_size.unwrap()).build(&ui);
                        ui.get_window_draw_list()
                            .add_rect([0.0, 0.0], [100.0, 100.0], ImColor32::BLACK)
                            .filled(true)
                            .build();
                    });

                nopadding.pop(&ui);

                imgui::Window::new(im_str!("Cube"))
                    .size([512.0, 512.0], Condition::FirstUseEver)
                    .build(&ui, || {
                        new_example_size = Some(ui.content_region_avail());
                        ui.text("Hello World!");
                        if ui.is_window_hovered() {
                            ui.set_mouse_cursor(Some(MouseCursor::Hand));
                        }
                        // imgui::Image::new(example_texture_id, new_example_size.unwrap()).build(&ui);
                    });

                let mut encoder: wgpu::CommandEncoder = graphics
                    .device()
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                platform.prepare_render(&ui, &window);

                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view: &frame.output.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 0.1, // semi-transparent background
                            }),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });

                let draw_data = ui.render();

                renderer
                    .render(draw_data, graphics.queue(), graphics.device(), &mut rpass)
                    .expect("Rendering failed");

                drop(rpass);

                graphics.queue().submit(Some(encoder.finish()));
            }
            _ => (),
        }

        match event {
            Event::WindowEvent { window_id, .. } => {
                let window_data = window_map.get_mut(&window_id).unwrap();

                let imgui = &mut window_data.imgui;
                let platform = &mut window_data.platform;

                let window = &window_data.window;
                let inner_window_data = window.window_data();

                let window = &inner_window_data.window;

                platform.handle_event(imgui.io_mut(), &window, &event);
            }
            _ => {}
        }
    });
}
