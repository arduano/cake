use std::{collections::VecDeque, time::Instant};

pub struct Fps {
    times: VecDeque<Instant>,
    fps: i32,
}

impl Fps {
    pub fn new() -> Self {
        Fps {
            times: VecDeque::new(),
            fps: 0,
        }
    }

    pub fn count_frame(&mut self) {
        self.times.push_front(Instant::now());
        self.fps += 1;
    }

    pub fn fps(&mut self) -> i32 {
        loop {
            match self.times.back() {
                None => break,
                Some(time) => {
                    if time.elapsed().as_secs_f32() > 1.0 {
                        self.times.pop_back();
                        self.fps -= 1;
                    } else {
                        break;
                    }
                }
            }
        }

        self.fps
    }
}
