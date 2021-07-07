use gui::application::{run_application, ApplicationGraphics};
use gui::elements::Element;
use gui::window::{DisplayWindow, ImGuiDisplayContext, WindowData};
use gui::{d, rgb, rgba, size, style};
use imgui::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use stretch::number::Number;
use util::fps::Fps;
use wgpu::Instance;
use winit::window::WindowBuilder;
use winit::{dpi::LogicalSize, event_loop::EventLoop};

struct CakeData {}

struct CakeWindow {
    window_data: WindowData,
    root_element: Box<dyn Element<CakeData>>,
    fps: Fps,
}

impl CakeWindow {
    pub fn new(
        instance: &Instance,
        event_loop: &EventLoop<i32>,
        _model: Arc<Mutex<Box<CakeData>>>,
    ) -> Self {
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

        let root_element = {
            use gui::elements::{RectangleRippleButton, FlexElement};

            FlexElement::<CakeData>::new(
                rgba!(0, 0, 0, 0),
                style!(size => size!(100.0, %; 50.0, px)),
                vec![
                    RectangleRippleButton::new(
                        rgba!(0, 0, 0, 0),
                        style!(size => size!(10, %; 100, %), flex_shrink => 0.0),
                        || {
                            println!("Clicked!");
                        },
                        vec![],
                    ),
                    FlexElement::new(
                        rgb!(255, 0, 0),
                        style!(size => size!(10, %; 100, %), flex_shrink => 0.0),
                        vec![],
                    ),
                    FlexElement::new(
                        rgb!(0, 255, 0),
                        style!(size => size!(10, %; 100, %), flex_shrink => 0.0),
                        vec![],
                    ),
                    FlexElement::new(
                        rgb!(255, 0, 0),
                        style!(size => size!(10, %; 100, %), flex_shrink => 0.0),
                        vec![],
                    ),
                    FlexElement::new(
                        rgb!(0, 0, 255),
                        style!(size => size!(10, %; 100, %), flex_shrink => 0.0),
                        vec![],
                    ),
                    FlexElement::new(
                        rgb!(255, 0, 255),
                        style!(size => size!(500, px; 100, %), flex_shrink => 0.0),
                        vec![],
                    ),
                    FlexElement::new(
                        rgb!(255, 255, 0),
                        style!(size => size!(10, %; 100, %), flex_shrink => 2.0),
                        vec![],
                    ),
                ],
            )
        };

        CakeWindow {
            window_data: WindowData::new(window, instance),
            root_element: root_element,
            fps: Fps::new(),
        }
    }
}

impl DisplayWindow<CakeData, i32> for CakeWindow {
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

    fn render(
        &mut self,
        graphics: &mut ApplicationGraphics,
        imgui_context: &mut ImGuiDisplayContext,
        model: &Arc<Mutex<Box<CakeData>>>,
        imgui: &mut Context,
        delta: Duration,
    ) {
        let platform = &mut imgui_context.platform;
        let renderer = &mut imgui_context.renderer;

        if self.window_data().swap_chain.is_none() {
            self.create_and_set_swapchain(&graphics, model);
        }

        imgui.io_mut().update_delta_time(delta);

        let swap_chain = self.window_data().swap_chain.as_ref().unwrap();

        let frame = match swap_chain.get_current_frame() {
            Ok(frame) => frame,
            Err(e) => {
                eprintln!("dropped frame: {:?}", e);
                return;
            }
        };

        {
            let window = &self.window_data().window;
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
                // ui.get_window_draw_list()
                //     .add_rect([0.0, 0.0], [100.0, 100.0], ImColor32::BLACK)
                //     .filled(true)
                //     .build();
                let mut stretch = stretch::node::Stretch::new();
                let node = self
                    .root_element
                    .layout(&mut stretch, model)
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

                self.root_element.render(&stretch, &ui, model);
            });

        nopadding.pop(&ui);

        imgui::Window::new(im_str!("Cube"))
            .size([512.0, 512.0], Condition::FirstUseEver)
            .build(&ui, || {
                new_example_size = Some(ui.content_region_avail());
                ui.text("Hello World!");
                ui.text(format!("Fps: {}", self.fps.fps()));
                // if ui.is_window_hovered() {
                //     ui.set_mouse_cursor(Some(MouseCursor::Hand));
                // }
                // imgui::Image::new(example_texture_id, new_example_size.unwrap()).build(&ui);
            });

        let mut encoder: wgpu::CommandEncoder = graphics
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let window = &self.window_data().window;
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

        self.fps.count_frame();
    }
}

fn main() {
    wgpu_subscriber::initialize_default_subscriber(None);

    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let event_loop = EventLoop::<i32>::with_user_event();

    let model = CakeData {};
    let model = Arc::new(Mutex::new(Box::new(model)));
    let main_window = CakeWindow::new(&instance, &event_loop, model.clone());

    let window = main_window.window_data();
    let graphics = ApplicationGraphics::create(instance, &window);

    run_application(graphics, event_loop, model, Box::new(main_window));
}
