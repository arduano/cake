use __core::fmt::Debug;
use __core::mem::size_of;
use bytemuck::{Pod, Zeroable};
use futures::executor::block_on;
use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig, Texture, TextureConfig};
use midi::data::IntVector4;
use midi::midifile::MIDIFile;
use std::fs::{self, File};
use std::io::Read;
use std::num::NonZeroU32;
use std::time::Instant;
use wgpu::{util::DeviceExt, BlendState, Extent3d};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::WindowBuilder;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

struct CakeApplication {}

fn main() {
    wgpu_subscriber::initialize_default_subscriber(None);

    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        // compatible_surface: Some(&surface),
        compatible_surface: None,
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

    println!("Creating swapchain");

    // Set up window and GPU
    let mut event_loop = EventLoop::new();

    let (window, size, surface) = {
        let version = env!("CARGO_PKG_VERSION");

        let window = WindowBuilder::new()
            .with_transparent(true)
            .build(&event_loop)
            .unwrap();
        window.set_inner_size(LogicalSize {
            width: 1280.0,
            height: 720.0,
        });
        window.set_title(&format!("Cake {}", version));
        let size = window.inner_size();

        let surface = unsafe { instance.create_surface(&window) };

        (window, size, surface)
    };

    let hidpi_factor = window.scale_factor();

    // Set up swap chain
    let sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width as u32,
        height: size.height as u32,
        present_mode: wgpu::PresentMode::Fifo,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

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
        data: include_bytes!("data/OpenSans-Regular.ttf"),
        config: Some(imgui::FontConfig {
            oversample_h: 4,
            pixel_snap_h: true,
            size_pixels: font_size,
            ..Default::default()
        }),
        size_pixels: font_size,
    }]);

    //
    // Set up dear imgui wgpu renderer
    //
    // let clear_color = wgpu::Color {
    //     r: 0.1,
    //     g: 0.2,
    //     b: 0.3,
    //     a: 1.0,
    // };

    let renderer_config = RendererConfig {
        texture_format: sc_desc.format,
        ..Default::default()
    };

    let mut renderer = Renderer::new(&mut imgui, &device, &queue, renderer_config);

    let mut last_frame = Instant::now();

    let mut last_cursor = None;

    // Event loop
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
            } => {
                let size = window.inner_size();

                let sc_desc = wgpu::SwapChainDescriptor {
                    usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    width: size.width as u32,
                    height: size.height as u32,
                    present_mode: wgpu::PresentMode::Mailbox,
                };

                swap_chain = device.create_swap_chain(&surface, &sc_desc);
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
            Event::RedrawRequested(window_id) => {
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
                        if ui.is_window_hovered() {
                            ui.set_mouse_cursor(Some(MouseCursor::Hand));
                        }
                        // imgui::Image::new(example_texture_id, new_example_size.unwrap()).build(&ui);
                    });

                let mut encoder: wgpu::CommandEncoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

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

                renderer
                    .render(draw_data, &queue, &device, &mut rpass)
                    .expect("Rendering failed");

                drop(rpass);

                queue.submit(Some(encoder.finish()));
            }
            _ => (),
        }

        platform.handle_event(imgui.io_mut(), &window, &event);
    });
}
