use std::{
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use imgui::{im_str, Condition, Context, FontSource, ImColor32, StyleVar};
use imgui_wgpu::{Renderer, RendererConfig};
use wgpu::{Device, Queue};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::application::ApplicationGraphics;

pub trait DisplayWindow {
    fn init_imgui(&mut self, imgui: &mut Context);

    fn create_window(&self) -> (Window, EventLoop<()>);

    fn render(&mut self);
}

pub fn open_window() {}

pub fn show_window(
    window: Window,
    event_loop: EventLoop<()>,
    mut display_window: Box<dyn DisplayWindow>,
    graphics: &Arc<Mutex<dyn ApplicationGraphics>>,
) {
    let size = window.inner_size();

    let graphics = graphics.clone();

    // Set up swap chain
    let sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width as u32,
        height: size.height as u32,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let (surface, mut swap_chain) = {
        let graphics = graphics.lock().unwrap();

        let surface = unsafe { graphics.instance().create_surface(&window) };
        let swap_chain = graphics.device().create_swap_chain(&surface, &sc_desc);

        (surface, swap_chain)
    };

    let hidpi_factor = window.scale_factor();

    // Set up dear imgui
    let mut imgui = imgui::Context::create();
    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
    platform.attach_window(
        imgui.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Default,
    );
    imgui.set_ini_filename(None);

    let font_size = (16.0 * hidpi_factor) as f32;
    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    // imgui.fonts().add_font(&[FontSource::DefaultFontData {
    //     config: Some(imgui::FontConfig {
    //         oversample_h: 1,
    //         pixel_snap_h: true,
    //         size_pixels: font_size,
    //         ..Default::default()
    //     }),
    // }]);

    imgui.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../../cake/src/data/OpenSans-Regular.ttf"),
        config: Some(imgui::FontConfig {
            oversample_h: 4,
            pixel_snap_h: true,
            size_pixels: font_size,
            ..Default::default()
        }),
        size_pixels: font_size,
    }]);

    let renderer_config = RendererConfig {
        texture_format: sc_desc.format,
        ..Default::default()
    };

    let mut renderer = {
        let graphics = graphics.lock().unwrap();
        Renderer::new(
            &mut imgui,
            graphics.device(),
            graphics.queue(),
            renderer_config,
        )
    };

    let graphics = graphics.clone();

    let mut last_frame = Instant::now();

    let mut last_cursor = None;
    // Event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Poll
        };
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                let size = window.inner_size();

                let sc_desc = wgpu::SwapChainDescriptor {
                    usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    width: size.width as u32,
                    height: size.height as u32,
                    present_mode: wgpu::PresentMode::Mailbox,
                };

                {
                    let graphics = graphics.lock().unwrap();
                    swap_chain = graphics.device().create_swap_chain(&surface, &sc_desc);
                }
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            }
            | Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawEventsCleared => {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;

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
                        // imgui::Image::new(example_texture_id, new_example_size.unwrap()).build(&ui);
                    });

                display_window.render();

                let mut encoder: wgpu::CommandEncoder = {
                    let graphics = graphics.lock().unwrap();
                    graphics
                        .device()
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None })
                };

                if last_cursor != Some(ui.mouse_cursor()) {
                    last_cursor = Some(ui.mouse_cursor());
                    platform.prepare_render(&ui, &window);
                }

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

                {
                    let graphics = graphics.lock().unwrap();

                    renderer
                        .render(draw_data, graphics.queue(), graphics.device(), &mut rpass)
                        .expect("Rendering failed");

                    drop(rpass);

                    graphics.queue().submit(Some(encoder.finish()));
                }
            }
            _ => (),
        }

        platform.handle_event(imgui.io_mut(), &window, &event);
    });
}
