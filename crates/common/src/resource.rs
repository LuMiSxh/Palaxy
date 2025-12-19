use num_cpus;
use serde::Serialize;
use specta::Type;
use sysinfo::System;

// --- Configuration Constants ---

/// Memory required per concurrent volume in GB (conservative estimate).
const GB_PER_VOLUME: u64 = 2;

/// Minimum concurrent volumes allowed (always process at least 1 volume).
const MIN_CONCURRENT_VOLUMES: usize = 1;

/// Maximum concurrent volumes to prevent I/O thrashing and excessive memory usage.
const MAX_CONCURRENT_VOLUMES: usize = 4;

/// CPU cores divisor for concurrency limit calculation.
/// We use 1/4 of cores for volume-level concurrency, leaving resources for rayon's
/// image-level parallelism within each volume.
const CPU_CORES_DIVISOR: usize = 4;

/// Minimum thread pool size for Rayon global thread pool.
const MIN_THREAD_POOL_SIZE: usize = 2;

/// Maximum thread pool size to prevent excessive context switching.
/// Higher values don't help much due to diminishing returns and cache contention.
const MAX_THREAD_POOL_SIZE: usize = 16;

/// Chunk size for image batch processing on systems with ≤4GB RAM.
const CHUNK_SIZE_TINY: usize = 25;

/// Chunk size for image batch processing on systems with 5-8GB RAM.
const CHUNK_SIZE_SMALL: usize = 50;

/// Chunk size for image batch processing on systems with 9-16GB RAM.
const CHUNK_SIZE_MEDIUM: usize = 100;

/// Chunk size for image batch processing on systems with >16GB RAM.
const CHUNK_SIZE_LARGE: usize = 150;

/// RAM threshold for "small" systems (4GB).
const RAM_THRESHOLD_SMALL: u64 = 4;

/// RAM threshold for "medium" systems (8GB).
const RAM_THRESHOLD_MEDIUM: u64 = 8;

/// RAM threshold for "large" systems (16GB).
const RAM_THRESHOLD_LARGE: u64 = 16;

// --- Resource Budget ---

/// Resource budget for concurrent operations.
///
/// This struct calculates and holds optimal resource allocations based on system capabilities:
/// - `max_concurrent_volumes`: How many volumes can be converted in parallel
/// - `rayon_chunk_size`: Chunk size for Rayon's parallel iterators (optimized for work distribution)
/// - `memory_batch_size`: Batch size for memory management (prevents OOM on low-RAM systems)
/// - `thread_pool_size`: Total number of worker threads for Rayon
#[derive(Debug, Clone, Serialize, Type)]
pub struct ResourceBudget {
    /// Maximum number of volumes to convert concurrently (1-4).
    pub max_concurrent_volumes: usize,

    /// Optimal chunk size for Rayon parallelism (4-32 images per chunk).
    /// Used with rayon's `with_min_len()` for work-stealing load balancing.
    pub rayon_chunk_size: usize,

    /// Maximum batch size for memory management (25-150 images per batch).
    /// Used for streaming/batching to prevent OOM on low-RAM systems.
    pub memory_batch_size: usize,

    /// Total number of worker threads for the Rayon thread pool (2-16).
    pub thread_pool_size: usize,
}

impl ResourceBudget {
    /// Calculates optimal resource budget based on current system resources.
    ///
    /// This analyzes available RAM and CPU cores to determine the best settings
    /// for parallel conversion operations. The calculation is done once at startup.
    pub fn calculate() -> Self {
        let mut sys = System::new();

        // Only refresh memory - faster than new_all()
        sys.refresh_memory();

        let total_memory_bytes = sys.total_memory();
        let total_memory_gb = total_memory_bytes / 1024 / 1024 / 1024;

        let cpu_cores = num_cpus::get();

        let max_concurrent_volumes = Self::calculate_concurrency(total_memory_gb, cpu_cores);

        let thread_pool_size = cpu_cores.clamp(MIN_THREAD_POOL_SIZE, MAX_THREAD_POOL_SIZE);

        // Rayon chunk size: optimized for parallelism (small chunks for work distribution)
        let rayon_chunk_size = Self::calculate_rayon_chunk_size(thread_pool_size);

        // Memory batch size: optimized for memory efficiency (larger batches based on RAM)
        let memory_batch_size = Self::calculate_memory_batch_size(total_memory_gb);

        log::info!(
            "System: {}GB RAM, {} cores -> Budget: {} concurrent volumes, rayon_chunk={}, memory_batch={}, {} threads",
            total_memory_gb,
            cpu_cores,
            max_concurrent_volumes,
            rayon_chunk_size,
            memory_batch_size,
            thread_pool_size
        );

        Self {
            max_concurrent_volumes,
            rayon_chunk_size,
            memory_batch_size,
            thread_pool_size,
        }
    }

    /// Calculates maximum concurrent volumes from RAM and CPU constraints.
    #[inline]
    fn calculate_concurrency(total_memory_gb: u64, cpu_cores: usize) -> usize {
        let ram_limit = ((total_memory_gb / GB_PER_VOLUME) as usize).max(MIN_CONCURRENT_VOLUMES);
        let cpu_limit = (cpu_cores / CPU_CORES_DIVISOR).max(MIN_CONCURRENT_VOLUMES);

        ram_limit.min(cpu_limit).min(MAX_CONCURRENT_VOLUMES)
    }

    /// Calculates optimal chunk size for Rayon's parallel iterator.
    ///
    /// This is optimized for work distribution and load balancing, not memory.
    /// The formula creates enough chunks to keep all threads busy with work-stealing.
    #[inline]
    fn calculate_rayon_chunk_size(thread_pool_size: usize) -> usize {
        // Create 2-4 chunks per thread for good load balancing
        // More chunks = better load balancing but more overhead
        // Formula: aim for 3x the thread count as total chunks
        let target_total_chunks = thread_pool_size * 3;

        // For typical volume (200 images), this gives:
        // 16 threads -> chunk_size = 200 / (16*3) = 4-5
        // 8 threads -> chunk_size = 200 / (8*3) = 8-9
        // 4 threads -> chunk_size = 200 / (4*3) = 16-17

        // Clamp between 4 (min for overhead) and 32 (max for parallelism)
        4.max(200 / target_total_chunks).min(32)
    }

    /// Calculates optimal batch size for memory management.
    ///
    /// This determines how many images to keep in memory at once.
    /// Larger batches are more efficient but use more RAM.
    #[inline]
    fn calculate_memory_batch_size(total_memory_gb: u64) -> usize {
        if total_memory_gb <= RAM_THRESHOLD_SMALL {
            CHUNK_SIZE_TINY
        } else if total_memory_gb <= RAM_THRESHOLD_MEDIUM {
            CHUNK_SIZE_SMALL
        } else if total_memory_gb <= RAM_THRESHOLD_LARGE {
            CHUNK_SIZE_MEDIUM
        } else {
            CHUNK_SIZE_LARGE
        }
    }

    /// Creates custom resource budget with explicit values (clamped to safe ranges).
    ///
    /// This is useful for testing or when you want to override auto-detection.
    #[inline]
    pub fn custom(
        max_concurrent_volumes: usize,
        rayon_chunk_size: usize,
        memory_batch_size: usize,
        thread_pool_size: usize,
    ) -> Self {
        Self {
            max_concurrent_volumes: max_concurrent_volumes
                .clamp(MIN_CONCURRENT_VOLUMES, MAX_CONCURRENT_VOLUMES),
            rayon_chunk_size: rayon_chunk_size.clamp(4, 32),
            memory_batch_size: memory_batch_size.clamp(25, 150),
            thread_pool_size: thread_pool_size.clamp(MIN_THREAD_POOL_SIZE, MAX_THREAD_POOL_SIZE),
        }
    }

    /// Returns the optimal number of threads for image processing within a single volume.
    ///
    /// This is typically the full thread pool size, as Rayon will handle the parallelism.
    /// Use this value when configuring parallel image processing operations.
    #[inline]
    pub fn image_processing_threads(&self) -> usize {
        self.thread_pool_size
    }
}

impl Default for ResourceBudget {
    fn default() -> Self {
        Self::calculate()
    }
}
