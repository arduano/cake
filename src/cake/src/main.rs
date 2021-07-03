use gui::application::run_application_default;
use gui::window::{DisplayWindow, WindowData};
use imgui::*;
use wgpu::Instance;
use winit::window::WindowBuilder;
use winit::{dpi::LogicalSize, event_loop::EventLoop};

struct CakeData {}

struct CakeWindow {
    window_data: WindowData,
}

enum CakeEvent {
    E,
}

impl CakeWindow {
    pub fn new(instance: &Instance, event_loop: &EventLoop<i32>) -> Self {
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

        CakeWindow {
            window_data: WindowData::new(window, instance),
        }
    }
}

impl<Ev> DisplayWindow<CakeData, Ev> for CakeWindow {
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

    fn render(&mut self, model: &CakeData) {
        todo!()
    }
}

fn main() {
    wgpu_subscriber::initialize_default_subscriber(None);

    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let event_loop = EventLoop::<i32>::with_user_event();

    let main_window = CakeWindow::new(&instance, &event_loop);

    let model = CakeData {};

    run_application_default(
        instance,
        event_loop,
        Box::new(model),
        Box::new(main_window),
        1,
    );
}
