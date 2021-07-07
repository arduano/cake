use std::time::Duration;

use imgui::Context;

use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use wgpu::{Instance, Surface, SwapChain, TextureFormat};
use winit::window::Window;

use crate::application::ApplicationGraphics;

pub trait DisplayWindow {
    fn init_imgui(&mut self, imgui: &mut Context);

    // fn create_window(&self, instance: &Instance, event_loop: &EventLoop) -> WindowData;
    fn window_data(&self) -> &WindowData;
    fn window_data_mut(&mut self) -> &mut WindowData;

    fn reset_swapchain(&mut self) {
        self.window_data_mut().swap_chain = None;
    }

    fn render(&mut self, delta: Duration);

    fn handle_platform_event(&mut self, event: &winit::event::Event<()>);
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

    pub fn swapchain_texture_format() -> wgpu::TextureFormat {
        wgpu::TextureFormat::Bgra8UnormSrgb
    }

    pub fn create_swapchain(&self, graphics: &ApplicationGraphics) -> SwapChain {
        let (window, surface) = (&self.window, &self.surface);

        let size = window.inner_size();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: WindowData::swapchain_texture_format(),
            width: size.width as u32,
            height: size.height as u32,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        graphics.device().create_swap_chain(surface, &sc_desc)
    }

    pub fn create_and_set_swapchain(&mut self, graphics: &ApplicationGraphics) {
        self.swap_chain = Some(self.create_swapchain(graphics));
    }
}

pub struct ImGuiDisplayContext {
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    pub imgui: Context,
}

impl ImGuiDisplayContext {
    pub fn new(
        graphics: &ApplicationGraphics,
        window: &WindowData,
        swapchain_format: TextureFormat,
    ) -> Self {
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            &window.window,
            imgui_winit_support::HiDpiMode::Default,
        );

        let renderer_config = RendererConfig {
            texture_format: swapchain_format,
            ..Default::default()
        };

        let renderer = Renderer::new(
            &mut imgui,
            graphics.device(),
            graphics.queue(),
            renderer_config,
        );

        ImGuiDisplayContext {
            imgui,
            platform,
            renderer,
        }
    }
}
