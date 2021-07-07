use imgui::ImColor32;

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
