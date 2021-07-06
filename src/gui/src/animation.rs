use std::time::Instant;

use crate::util::Lerp;

pub struct VelocityEase {
    pub duration: f32,
    pub slope: f32,
    pub supress: f32,

    pub velocity_power: f32,

    start: f32,
    end: f32,

    pub clamp_to_ends: bool,

    v: f32,
    vm: f32,
    start_time: Instant,
}

impl VelocityEase {
    pub fn new(initial: f32) -> Self {
        VelocityEase {
            duration: 1.0,
            slope: 2.0,
            supress: 1.0,
            start: initial,
            end: initial,
            clamp_to_ends: true,
            velocity_power: 3.0,
            start_time: Instant::now(),
            v: 0.0,
            vm: 1.0,
        }
    }

    fn get_raw_inertial_pos(&self, t: f32) -> f32 {
        f32::powf(t, self.velocity_power) * (1.0 - t)
    }

    fn get_raw_inertial_vel(&self, t: f32) -> f32 {
        let velocity_power = self.velocity_power;
        (1.0 - t) * f32::powf(t, velocity_power - 1.0) * velocity_power
            - f32::powf(t, velocity_power)
    }

    fn get_inertial_pos(&self, t: f32) -> f32 {
        (self.get_raw_inertial_pos(1.0 - t) * self.v) * self.vm
    }

    fn get_inertial_vel(&self, t: f32) -> f32 {
        (-self.get_raw_inertial_vel(1.0 - t) * self.v) * self.vm
    }

    fn get_ease_pos(&self, t: f32) -> f32 {
        let slope = self.slope;
        f32::powf(t, slope) / (f32::powf(1.0 - t, slope) + f32::powf(t, slope))
    }

    fn get_ease_vel(&self, t: f32) -> f32 {
        let slope = self.slope;
        (f32::powf(-(-1.0 + t) * t, slope - 1.0) * slope)
            / f32::powf(f32::powf(1.0 - t, slope) + f32::powf(t, slope), 2.0)
    }

    pub fn value(&self) -> f32 {
        let t = self.start_time.elapsed().as_secs_f32() / self.duration;
        if t > 1.0 {
            return self.end;
        }

        let mut pos = self.get_ease_pos(t) * (self.end - self.start) + self.get_inertial_pos(t);
        pos += self.start;

        if self.clamp_to_ends {
            if self.start < self.end {
                pos = pos.clamp(self.start, self.end);
            } else {
                pos = pos.clamp(self.end, self.start);
            }
        }

        return pos;
    }

    pub fn value_clamped(&self, min: f32, max: f32) -> f32 {
        let val = self.value();
        val.clamp(min, max)
    }

    pub fn set_end(&mut self, e: f32) {
        let mut t = self.start_time.elapsed().as_secs_f32() / self.duration;
        let mut vel;
        let dist = self.end - self.start;
        if t > 1.0 {
            vel = 0.0;
            t = 1.0;
        } else {
            vel = self.get_ease_vel(t) * dist + self.get_inertial_vel(t);
        }
        vel /= self.supress;

        let pos = self.get_ease_pos(t) * dist + self.start + self.get_inertial_pos(t);

        self.start = pos;
        self.end = e;

        self.v = vel;
        self.start_time = Instant::now();
    }

    pub fn force_value(&mut self, v: f32) {
        self.start = v;
        self.end = v;
        self.v = 0.0;
    }
}

pub struct OneWayEase<T: Lerp> {
    start: T,
    end: T,
    delay: f32,
    fade: f32,
    start_time: Option<Instant>,
}

impl<T: Lerp + Copy> OneWayEase<T> {
    pub fn new(start: T, end: T, fade: f32, delay: f32) -> Self {
        OneWayEase {
            start,
            end,
            fade,
            delay,
            start_time: None,
        }
    }

    pub fn new_started(start: T, end: T, fade: f32, delay: f32) -> Self {
        let mut ease = Self::new(start, end, fade, delay);
        ease.start();
        ease
    }

    pub fn started(&self) -> bool {
        self.start_time.is_none()
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn value(&self) -> T {
        match self.start_time {
            None => *&self.start,
            Some(start_time) => {
                let t = start_time.elapsed().as_secs_f32() - self.delay;
                let t = (t / self.fade).clamp(0.0, 1.0);
                self.start.lerp(&self.end, t)
            }
        }
    }

    pub fn ended(&self) -> bool {
        match self.start_time {
            None => false,
            Some(start_time) => {
                let t = start_time.elapsed().as_secs_f32() - self.delay;
                t > self.fade
            }
        }
    }
}
