use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use gui::{
    application::ApplicationGraphics,
    elements::Element,
    window::{DisplayWindow, ImGuiDisplayContext, WindowData},
};
use imgui::{im_str, Condition, StyleVar};
use model::CakeModel;
use stretch::number::Number;
use wgpu::Instance;
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

use crate::{model::Fonts, windows::main::MainWindowElement};

mod macros;
mod model;
mod renderer;
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

        let hidpi_factor = window_data.window.scale_factor();

        let (mut imgui, fonts) = ImGuiDisplayContext::new(
            &graphics,
            &window_data,
            wgpu::TextureFormat::Bgra8UnormSrgb,
            |mut imgui| Fonts::load(&mut imgui, hidpi_factor),
        );

        let model = Arc::new(Mutex::new(CakeModel::new(&mut imgui, &mut graphics, fonts)));

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

        let swap_chain = self.window_data.swap_chain.as_ref().unwrap();

        let imgui = &mut self.imgui.imgui;

        imgui.io_mut().update_delta_time(delta);

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

        let mut model_locked = self.model.lock().unwrap();
        let main_window_element = &mut self.main_window_element;

        let default_font = ui.push_font(model_locked.view.fonts.open_sans_16);

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
                let mut stretch = stretch::node::Stretch::new();
                let node = main_window_element
                    .layout(&mut stretch, &mut model_locked)
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
                main_window_element.render([0.0, 0.0], &stretch, &ui, &mut model_locked);
            });

        {
            let renderer = &mut model_locked.view.renderer;
            renderer.update_to_last_size(
                &mut self.imgui.renderer,
                &self.graphics,
                &ui.io().display_framebuffer_scale,
            );
            renderer.render(&mut self.imgui.renderer, &self.graphics);
        }

        nopadding.pop(&ui);

        imgui::Window::new(im_str!("Cube"))
            .size([512.0, 512.0], Condition::FirstUseEver)
            .build(&ui, || {
                new_example_size = Some(ui.content_region_avail());
                ui.text("Hello World!");
                ui.text(format!("Fps: {}", model_locked.view.fps.fps()));
                // if ui.is_window_hovered() {
                //     ui.set_mouse_cursor(Some(MouseCursor::Hand));
                // }
                // imgui::Image::new(example_texture_id, new_example_size.unwrap()).build(&ui);
            });

        default_font.pop(&ui);

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
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
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

        model_locked.view.fps.count_frame();
    }

    fn handle_platform_event(&mut self, event: &winit::event::Event<()>) {
        self.imgui
            .platform
            .handle_event(self.imgui.imgui.io_mut(), &self.window_data.window, event)
    }
}
