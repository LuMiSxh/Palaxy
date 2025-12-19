use common::ResourceBudget;

/// Initialize performance optimizations for the application.
pub fn init() {
    let budget = ResourceBudget::calculate();

    // Stack size for image processing (4MB per thread)
    const STACK_SIZE: usize = 4 * 1024 * 1024;

    match rayon::ThreadPoolBuilder::new()
        .num_threads(budget.thread_pool_size)
        .stack_size(STACK_SIZE)
        .thread_name(|idx| format!("palaxy-worker-{}", idx))
        .build_global()
    {
        Ok(_) => {
            log::info!(
                "Performance initialized - {} threads ({}MB stack), {} concurrent volumes, rayon_chunk={}, memory_batch={}",
                budget.thread_pool_size,
                STACK_SIZE / (1024 * 1024),
                budget.max_concurrent_volumes,
                budget.rayon_chunk_size,
                budget.memory_batch_size
            );
        }
        Err(_) => {
            log::warn!("Rayon thread pool already initialized - using default configuration.");
        }
    }
}
