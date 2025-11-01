//! Various `teamsearch` utilities.
pub mod fs;
pub mod highlight;
pub mod lines;
pub mod logging;
pub mod stream;
pub mod thread_pool;
pub mod timers;

pub use timers::timed;
