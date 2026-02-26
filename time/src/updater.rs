use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use super::instant::*;

/// This updater is based on crate coarsetime updater.
/// A service to periodically call `Instant::update()`
///
#[derive(Debug)]
pub struct Updater {
    running: Arc<AtomicBool>,
    th: Option<thread::JoinHandle<()>>,
}

impl Updater {
    /// Spawns and starts a background task to call `Instant::update()`
    /// This Service internally yields every loop.
    ///
    pub fn new() -> Result<Self, io::Error> {
        let running = Arc::new(AtomicBool::new(true));
        let running_pass = running.clone();

        let th: thread::JoinHandle<()> = thread::Builder::new()
            .name("ascending_time_updater".to_string())
            .spawn(move || {
                while running_pass.load(Ordering::Relaxed) {
                    // we just want to yield it that way we dont eat resource but keep accuracy
                    thread::sleep(Duration::from_millis(0));
                    Instant::update();
                }
            })?;

        Instant::update();
        Ok(Updater {
            running,
            th: Some(th),
        })
    }

    /// Stops the Updater Thread.
    ///
    pub fn stop(mut self) -> Result<(), io::Error> {
        self.running.store(false, Ordering::Relaxed);

        self.th
            .take()
            .expect("The Thread was already unloaded.")
            .join()
            .map_err(|_| {
                io::Error::other("failed to properly stop the updater")
            })
    }
}

/// If we do Drop it without Stopping it we at least want to Set the running to false
/// so the thread will stop running in the background.
///
impl Drop for Updater {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
