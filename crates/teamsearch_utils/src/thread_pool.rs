//! Utilities for thread pool management.

/// Construct a thread pool with limited threads.
///
/// This function will create a thread pool with a number of threads that is
/// half of the current number of threads, but at least one thread.
pub fn construct_thread_pool() -> rayon::ThreadPool {
    let num_threads = std::cmp::min(
        rayon::current_num_threads(),
        std::cmp::max(1, rayon::current_num_threads() / 2),
    );

    // Configure a custom thread pool with limited threads
    rayon::ThreadPoolBuilder::new().num_threads(num_threads).build().unwrap()
}
