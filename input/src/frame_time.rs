use std::time::Instant;

#[derive(Clone, Debug)]
pub struct FrameTime {
    delta_seconds: f32,
    seconds: f32,
    frame_time: Instant,
    start_time: Instant,
}

impl FrameTime {
    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let instant = Instant::now();

        Self {
            delta_seconds: 0.0,
            seconds: 0.0,
            frame_time: instant,
            start_time: instant,
        }
    }

    pub fn seconds(&self) -> f32 {
        self.seconds
    }

    pub fn update(&mut self) {
        let frame_time = Instant::now();

        self.delta_seconds =
            frame_time.duration_since(self.frame_time).as_secs_f32();
        self.seconds = frame_time.duration_since(self.start_time).as_secs_f32();
        self.frame_time = frame_time;
    }
}
