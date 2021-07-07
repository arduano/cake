use gui::application::{run_application, ApplicationGraphics};
use gui::window::DisplayWindow;
use view::CakeWindow;
use winit::event_loop::EventLoop;

fn main() {
    wgpu_subscriber::initialize_default_subscriber(None);

    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let event_loop = EventLoop::new();

    let main_window = CakeWindow::new(&instance, &event_loop);

    let window = main_window.window_data();
    let graphics = ApplicationGraphics::create(instance, &window);

    run_application(graphics, event_loop, main_window);
}
