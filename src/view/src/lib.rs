use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use gui::{
    application::ApplicationGraphics,
    elements::Element,
    window::{DisplayWindow, ImGuiDisplayContext, WindowData},
};
use imgui::{im_str, Condition, Context, FontSource, StyleVar};
use model::CakeModel;
use stretch::number::Number;
use wgpu::Instance;
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

use crate::windows::main::MainWindowElement;

mod model;
mod windows;

pub struct CakeWindow {
    window_data: WindowData,
    main_window_element: MainWindowElement,
    model: Arc<Mutex<CakeModel>>,
    imgui: ImGuiDisplayContext,
    graphics: ApplicationGraphics,
}

impl CakeWindow {
    pub fn new(instance: Instance, event_loop: &EventLoop<()>) -> Self {
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

        let window_data = WindowData::new(window, &instance);

        let mut graphics = ApplicationGraphics::create(instance, &window_data);

        let mut imgui =
            ImGuiDisplayContext::new(&graphics, &window_data, wgpu::TextureFormat::Bgra8UnormSrgb);

        let model = Arc::new(Mutex::new(CakeModel::new(
            &mut imgui.renderer,
            &mut graphics,
        )));

        let main_window_element = MainWindowElement::new(&model);

        CakeWindow {
            window_data,
            main_window_element,
            model,
            graphics,
            imgui,
        }
    }
}

impl DisplayWindow for CakeWindow {
    fn init_imgui(&mut self, imgui: &mut Context) {
        let hidpi_factor = self.window_data.window.scale_factor();
        let font_size = (16.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

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
    }

    fn window_data(&self) -> &WindowData {
        &self.window_data
    }

    fn window_data_mut(&mut self) -> &mut WindowData {
        &mut self.window_data
    }

    fn render(&mut self, delta: Duration) {
        if self.window_data().swap_chain.is_none() {
            self.window_data.create_and_set_swapchain(&self.graphics);
        }

        let imgui = &mut self.imgui.imgui;

        imgui.io_mut().update_delta_time(delta);

        let swap_chain = self.window_data.swap_chain.as_ref().unwrap();

        let frame = match swap_chain.get_current_frame() {
            Ok(frame) => frame,
            Err(e) => {
                eprintln!("dropped frame: {:?}", e);
                return;
            }
        };

        {
            let platform = &mut self.imgui.platform;
            let window = &self.window_data.window;
            platform
                .prepare_frame(imgui.io_mut(), &window)
                .expect("Failed to prepare frame");
        }

        let ui = imgui.frame();

        // Render example normally at background
        let size = ui.io().display_size;

        // Store the new size of Image() or None to indicate that the window is collapsed.
        let mut new_example_size: Option<[f32; 2]> = None;

        let nopadding = ui.push_style_vars(&[
            StyleVar::WindowPadding([-1.0, -1.0]),
            StyleVar::WindowBorderSize(0.0),
        ]);

        let mut model_locked = self.model.lock().unwrap();
        let main_window_element = &mut self.main_window_element;

        let view_model = &mut model_locked.view;

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
                let mut stretch = stretch::node::Stretch::new();
                let node = main_window_element
                    .layout(&mut stretch, view_model)
                    .expect("Failed to retreive layout!");
                let window_size = ui.window_size();
                stretch
                    .compute_layout(
                        node,
                        stretch::geometry::Size {
                            width: Number::Defined(window_size[0]),
                            height: Number::Defined(window_size[1]),
                        },
                    )
                    .expect("Failed to compute layout!");
                main_window_element.render([0.0, 0.0], &stretch, &ui, view_model);
            });

        nopadding.pop(&ui);

        imgui::Window::new(im_str!("Cube"))
            .size([512.0, 512.0], Condition::FirstUseEver)
            .build(&ui, || {
                new_example_size = Some(ui.content_region_avail());
                ui.text("Hello World!");
                ui.text(format!("Fps: {}", view_model.fps.fps()));
                // if ui.is_window_hovered() {
                //     ui.set_mouse_cursor(Some(MouseCursor::Hand));
                // }
                // imgui::Image::new(example_texture_id, new_example_size.unwrap()).build(&ui);
            });

        let mut encoder: wgpu::CommandEncoder = self
            .graphics
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let platform = &mut self.imgui.platform;
            let window = &mut self.window_data.window;
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
            let renderer = &mut self.imgui.renderer;
            renderer
                .render(
                    draw_data,
                    self.graphics.queue(),
                    self.graphics.device(),
                    &mut rpass,
                )
                .expect("Rendering failed");
        }

        drop(rpass);

        self.graphics.queue().submit(Some(encoder.finish()));

        view_model.fps.count_frame();
    }

    fn handle_platform_event(&mut self, event: &winit::event::Event<()>) {
        self.imgui
            .platform
            .handle_event(self.imgui.imgui.io_mut(), &self.window_data.window, event)
    }
}
