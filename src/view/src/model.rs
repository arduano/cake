use std::sync::{Arc, Mutex};

use backend::CakeBackendModel;
use gui::{application::ApplicationGraphics, util::load_image_texture};
use imgui::TextureId;
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
                load_image_texture(renderer, graphics, include_bytes!($path), image::ImageFormat::$fmt)
            };
        }

        Textures {
            pause_button: load!("./data/pause.png", Png),
            play_button: load!("./data/play.png", Png),
        }
    }
}

pub struct CakeViewModel {
    pub fps: Fps,
    pub textures: Textures,
    pub paused: bool,
}

impl CakeViewModel {
    pub fn new(textures: Textures) -> Self {
        CakeViewModel {
            fps: Fps::new(),
            paused: true,
            textures,
        }
    }
}

pub struct CakeModel {
    pub backend: Arc<Mutex<CakeBackendModel>>,
    pub view: CakeViewModel,
}

impl CakeModel {
    pub fn new(renderer: &mut Renderer, graphics: &mut ApplicationGraphics) -> Self {
        let textures = Textures::load(renderer, graphics);

        CakeModel {
            backend: Arc::new(Mutex::new(CakeBackendModel {})),
            view: CakeViewModel::new(textures),
        }
    }
}
