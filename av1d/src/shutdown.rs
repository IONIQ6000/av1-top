// Graceful shutdown handling
// Handles SIGTERM and SIGINT for clean daemon shutdown

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Shared shutdown flag
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Install signal handlers for graceful shutdown
///
/// Handles SIGTERM and SIGINT (Ctrl+C) to allow current job to complete
pub fn install_signal_handlers() {
    #[cfg(unix)]
    {
        use std::sync::Mutex;
        
        // Set up signal handler using libc
        unsafe {
            libc::signal(libc::SIGTERM, handle_signal as libc::sighandler_t);
            libc::signal(libc::SIGINT, handle_signal as libc::sighandler_t);
        }
    }
}

/// Signal handler function
#[cfg(unix)]
extern "C" fn handle_signal(_signal: i32) {
    SHUTDOWN_REQUESTED.store(true, Ordering::Relaxed);
}

/// Check if shutdown has been requested
pub fn should_shutdown() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::Relaxed)
}

/// Request shutdown (can be called programmatically)
pub fn request_shutdown() {
    SHUTDOWN_REQUESTED.store(true, Ordering::Relaxed);
}

