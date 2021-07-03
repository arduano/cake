use imgui::Context;

use wgpu::{Instance, Surface, SwapChain};
use winit::window::Window;

use crate::application::ApplicationGraphics;

pub trait DisplayWindow<Model, Ev> {
    fn init_imgui(&mut self, imgui: &mut Context);

    // fn create_window(&self, instance: &Instance, event_loop: &EventLoop<Ev>) -> WindowData;
    fn window_data(&self) -> &WindowData;
    fn window_data_mut(&mut self) -> &mut WindowData;

    fn swapchain_texture_format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Bgra8UnormSrgb
    }

    fn create_swapchain(&self, graphics: &ApplicationGraphics, _model: &Model) -> SwapChain {
        let data = self.window_data();
        let (window, surface) = (&data.window, &data.surface);

        let size = window.inner_size();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: self.swapchain_texture_format(),
            width: size.width as u32,
            height: size.height as u32,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        graphics.device().create_swap_chain(surface, &sc_desc)
    }

    fn create_and_set_swapchain(&mut self, graphics: &ApplicationGraphics, model: &Model) {
        self.window_data_mut().swap_chain = Some(self.create_swapchain(graphics, model))
    }

    fn render(&mut self, model: &Model);
}

pub struct WindowData {
    pub window: Window,
    pub surface: Surface,
    pub swap_chain: Option<SwapChain>,
}

impl WindowData {
    pub fn new(window: Window, instance: &Instance) -> Self {
        let surface = unsafe { instance.create_surface(&window) };
        WindowData {
            surface: surface,
            window,
            swap_chain: None,
        }
    }
}
