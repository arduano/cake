use gui::application::run_application;
use view::CakeWindow;
use winit::event_loop::EventLoop;

fn main() {
    wgpu_subscriber::initialize_default_subscriber(None);

    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let event_loop = EventLoop::new();

    let main_window = CakeWindow::new(instance, &event_loop);

    run_application(event_loop, main_window);
}
