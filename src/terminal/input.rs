use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::warn;
use zond_engine::core::handle::ScanHandle;

pub struct InputGuard {
    running: Arc<AtomicBool>,
}

impl Drop for InputGuard {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        let _ = disable_raw_mode();
    }
}

/// Starts a background thread that listens for raw terminal input.
/// Returns an `InputGuard` that disables raw mode when dropped.
pub fn start_listener(handle: ScanHandle) -> InputGuard {
    if let Err(e) = enable_raw_mode() {
        warn!("Failed to enable raw terminal mode: {}", e);
    }

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    std::thread::spawn(move || {
        while running_clone.load(Ordering::Relaxed) {
            if let Some(Event::Key(KeyEvent {
                code, modifiers, ..
            })) = event::poll(std::time::Duration::from_millis(100))
                .ok()
                .and_then(|ready| if ready { event::read().ok() } else { None })
            {
                match code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        handle.abort();
                        break;
                    }
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        handle.abort();
                        break;
                    }
                    KeyCode::Esc => {
                        handle.abort();
                        break;
                    }
                    _ => {}
                }
            }
        }

        let _ = disable_raw_mode();
    });

    InputGuard { running }
}
