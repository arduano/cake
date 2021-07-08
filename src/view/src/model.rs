use std::{
    borrow::Borrow,
    sync::{Arc, Mutex},
    time::Instant,
};

use backend::CakeBackendModel;
use gui::{
    application::ApplicationGraphics, rgb, util::load_image_texture, window::ImGuiDisplayContext,
};
use imgui::{Context, FontId, FontSource, ImColor32, TextureId};
use imgui_wgpu::{Renderer, Texture, TextureConfig};
use util::fps::Fps;
use wgpu::Extent3d;

use crate::renderer::MidiRender;

pub struct Textures {
    pub pause_button: TextureId,
    pub play_button: TextureId,
}

impl Textures {
    pub fn load(renderer: &mut Renderer, graphics: &mut ApplicationGraphics) -> Self {
        macro_rules! load {
            ($path:expr, $fmt:ident) => {
                load_image_texture(
                    renderer,
                    graphics,
                    include_bytes!($path),
                    image::ImageFormat::$fmt,
                )
            };
        }

        Self {
            pause_button: load!("./data/pause.png", Png),
            play_button: load!("./data/play.png", Png),
        }
    }
}

pub struct Fonts {
    pub open_sans_16: FontId,
}

impl Fonts {
    pub fn load(imgui: &mut Context, hidpi_factor: f64) -> Self {
        let font_size = (18.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        Self {
            open_sans_16: imgui.fonts().add_font(&[FontSource::TtfData {
                data: include_bytes!("data/OpenSans-Regular.ttf"),
                config: Some(imgui::FontConfig {
                    oversample_h: 4,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
                size_pixels: font_size,
            }]),
        }
    }
}

pub struct ColorPalette {
    pub primary: ImColor32,
    pub dark: ImColor32,
    pub light: ImColor32,
    pub bg_light: ImColor32,
    pub bg: ImColor32,
}

impl ColorPalette {
    pub fn new() -> Self {
        Self {
            primary: rgb!(0x6a, 0x1b, 0x9a),
            dark: rgb!(0x38, 0x00, 0x6b),
            light: rgb!(0x9c, 0x4d, 0xcc),
            bg_light: rgb!(0x23, 0x23, 0x23),
            bg: rgb!(0x12, 0x12, 0x12),
        }
    }
}

pub struct CakeRenderer {
    pub last_size: [f32; 2],
    pub tex_size: Extent3d,
    pub renderer: MidiRender,
    pub texture_id: TextureId,
}

impl CakeRenderer {
    pub fn new(renderer: &mut Renderer, graphics: &mut ApplicationGraphics) -> Self {
        let tex_size = Extent3d {
            width: 500,
            height: 500,
            ..Default::default()
        };

        let texture = CakeRenderer::make_texture(renderer, graphics, tex_size);

        let texture_id = renderer.textures.insert(texture);

        Self {
            last_size: [500.0, 500.0],
            renderer: MidiRender::init(
                gui::window::WindowData::swapchain_texture_format(),
                graphics.device(),
                graphics.queue(),
            ),
            tex_size,
            texture_id,
        }
    }

    fn make_texture(
        renderer: &mut Renderer,
        graphics: &ApplicationGraphics,
        size: Extent3d,
    ) -> Texture {
        let texture_config = TextureConfig {
            size,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
            ..Default::default()
        };

        Texture::new(&graphics.device(), &renderer, texture_config)
    }

    pub fn borrow_texture<'a>(&self, renderer: &'a Renderer) -> &'a Texture {
        renderer.textures.get(self.texture_id).unwrap()
    }

    pub fn update_to_last_size(
        &mut self,
        renderer: &mut Renderer,
        graphics: &ApplicationGraphics,
        display_framebuffer_scale: &[f32; 2],
    ) {
        let size = &self.last_size;
        let new_size = Extent3d {
            width: (size[0] * display_framebuffer_scale[0]) as u32,
            height: (size[1] * display_framebuffer_scale[1]) as u32,
            ..Default::default()
        };

        if new_size != self.tex_size {
            let new_tex = CakeRenderer::make_texture(renderer, graphics, new_size);
            renderer.textures.replace(self.texture_id, new_tex);
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer, graphics: &ApplicationGraphics) {
        let tex = self.borrow_texture(renderer);
        self.renderer
            .render(tex.view(), graphics.device(), graphics.queue(), &self.last_size);
    }
}

pub struct CakeViewModel {
    pub fps: Fps,
    pub init_time: Instant,
    pub textures: Textures,
    pub fonts: Fonts,
    pub palette: ColorPalette,
    pub renderer: CakeRenderer,
    pub paused: bool,
}

impl CakeViewModel {
    pub fn new(textures: Textures, fonts: Fonts, renderer: CakeRenderer) -> Self {
        CakeViewModel {
            fps: Fps::new(),
            paused: true,
            textures,
            fonts,
            renderer,
            palette: ColorPalette::new(),
            init_time: Instant::now(),
        }
    }
}

pub struct CakeModel {
    pub backend: Arc<Mutex<CakeBackendModel>>,
    pub view: CakeViewModel,
}

impl CakeModel {
    pub fn new(
        imgui: &mut ImGuiDisplayContext,
        graphics: &mut ApplicationGraphics,
        fonts: Fonts,
    ) -> Self {
        let textures = Textures::load(&mut imgui.renderer, graphics);
        let renderer = CakeRenderer::new(&mut imgui.renderer, graphics);

        CakeModel {
            backend: Arc::new(Mutex::new(CakeBackendModel {})),
            view: CakeViewModel::new(textures, fonts, renderer),
        }
    }
}
