use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use backend::CakeBackendModel;
use gui::{
    application::ApplicationGraphics, rgb, util::load_image_texture, window::ImGuiDisplayContext,
};
use imgui::{Context, FontId, FontSource, ImColor32, TextureId};
use imgui_wgpu::Renderer;
use util::fps::Fps;

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

pub struct CakeViewModel {
    pub fps: Fps,
    pub init_time: Instant,
    pub textures: Textures,
    pub fonts: Fonts,
    pub palette: ColorPalette,
    pub paused: bool,
}

impl CakeViewModel {
    pub fn new(textures: Textures, fonts: Fonts) -> Self {
        CakeViewModel {
            fps: Fps::new(),
            paused: true,
            textures,
            fonts,
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

        CakeModel {
            backend: Arc::new(Mutex::new(CakeBackendModel {})),
            view: CakeViewModel::new(textures, fonts),
        }
    }
}
