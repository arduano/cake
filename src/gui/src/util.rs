use std::sync::atomic::{AtomicUsize, Ordering};

use image::ImageFormat;
use imgui::{ImColor32, TextureId};
use imgui_wgpu::Renderer;

use crate::application::ApplicationGraphics;

pub trait Lerp {
    // Called lerpv for Lerp Value because "lerp" conflicts with an existing unstable
    // float interpolation feature.
    fn lerpv(&self, to: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerpv(&self, to: Self, t: f32) -> Self {
        self * (1.0 - t) + to * t
    }
}

impl Lerp for u8 {
    fn lerpv(&self, to: Self, t: f32) -> Self {
        ((*self as f32) * (1.0 - t) + (to as f32) * t) as u8
    }
}

impl Lerp for ImColor32 {
    fn lerpv(&self, to: Self, t: f32) -> Self {
        ImColor32::from_rgba(
            self.r.lerpv(to.r, t),
            self.g.lerpv(to.g, t),
            self.b.lerpv(to.b, t),
            self.a.lerpv(to.a, t),
        )
    }
}

static IMGUI_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
pub fn rand_im_id() -> imgui::Id<'static> {
    let val = IMGUI_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    imgui::Id::Int(val as i32)
}

pub fn load_image_texture(
    renderer: &mut Renderer,
    graphics: &mut ApplicationGraphics,
    bytes: &[u8],
    format: ImageFormat
) -> TextureId {
    use imgui_wgpu::{Texture, TextureConfig};
    use wgpu::Extent3d;

    let image =
        image::load_from_memory_with_format(bytes, format).expect("invalid image");
    let image = image.to_bgra8();
    let (width, height) = image.dimensions();
    let raw_data = image.into_raw();

    let texture_config = TextureConfig {
        size: Extent3d {
            width,
            height,
            ..Default::default()
        },
        label: Some("lenna texture"),
        ..Default::default()
    };

    let texture = Texture::new(&graphics.device(), &renderer, texture_config);

    texture.write(&graphics.queue(), &raw_data, width, height);
    let texture_id = renderer.textures.insert(texture);

    texture_id
}

pub trait ToImColor {
    fn to_imcolor(&self) -> ImColor32;
}

impl ToImColor for color::Rgba {
    fn to_imcolor(&self) -> ImColor32 {
        ImColor32::from_rgba(self.c.r, self.c.g, self.c.b, self.a)
    }
}

impl ToImColor for color::Rgb {
    fn to_imcolor(&self) -> ImColor32 {
        ImColor32::from_rgba(self.r, self.g, self.b, 255)
    }
}