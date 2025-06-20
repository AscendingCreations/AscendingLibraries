use std::time::Instant;

/// Keeps track of Timing useful for Games.
/// it keeps track of Delta Seconds and Seconds since
/// the start of the program.
#[derive(Clone, Copy, Debug)]
pub struct FrameTime {
    /// Time between each update call.
    pub delta_seconds: f32,
    /// Seconds and nano seconds since program start.
    pub seconds: f32,
    /// last Instant::Now() since update()
    pub frame_time: Instant,
    /// time since program started.
    start_time: Instant,
}

impl FrameTime {
    /// Returns Delta Second and Delta Nano Seconds since last update call.
    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }

    /// Creates the FrameTime and sets its Start Timer.
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

    /// Returns Seconds and Nano Seconds since Start of program.
    pub fn seconds(&self) -> f32 {
        self.seconds
    }

    /// Updates the Timer to get the current Seconds and Delta Seconds.
    pub fn update(&mut self) {
        let frame_time = Instant::now();

        self.delta_seconds =
            frame_time.duration_since(self.frame_time).as_secs_f32();
        self.seconds = frame_time.duration_since(self.start_time).as_secs_f32();
        self.frame_time = frame_time;
    }
}
