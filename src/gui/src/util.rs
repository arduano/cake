use imgui::ImColor32;

pub trait Lerp {
    fn lerp(&self, to: &Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(&self, to: &Self, t: f32) -> Self {
        self * (1.0 - t) + to * t
    }
}

impl Lerp for u8 {
    fn lerp(&self, to: &Self, t: f32) -> Self {
        ((*self as f32) * (1.0 - t) + (*to as f32) * t) as u8
    }
}

impl Lerp for ImColor32 {
    fn lerp(&self, to: &Self, t: f32) -> Self {
        ImColor32::from_rgba(
            self.r.lerp(&to.r, t),
            self.g.lerp(&to.g, t),
            self.b.lerp(&to.b, t),
            self.a.lerp(&to.a, t),
        )
    }
}
