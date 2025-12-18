use num_cpus;
use serde::Serialize;
use specta::Type;
use sysinfo::System;

// --- Configuration Constants ---

/// Memory required per concurrent volume in GB.
const GB_PER_VOLUME: u64 = 2;

/// Minimum concurrent volumes allowed.
const MIN_CONCURRENT_VOLUMES: usize = 1;

/// Maximum concurrent volumes to prevent I/O thrashing.
const MAX_CONCURRENT_VOLUMES: usize = 4;

/// CPU cores divisor for concurrency limit calculation.
const CPU_CORES_DIVISOR: usize = 4;

/// Minimum thread pool size.
const MIN_THREAD_POOL_SIZE: usize = 2;

/// Maximum thread pool size to prevent excessive context switching.
const MAX_THREAD_POOL_SIZE: usize = 16;

/// Chunk size for systems with ≤4GB RAM.
const CHUNK_SIZE_TINY: usize = 25;

/// Chunk size for systems with 5-8GB RAM.
const CHUNK_SIZE_SMALL: usize = 50;

/// Chunk size for systems with 9-16GB RAM.
const CHUNK_SIZE_MEDIUM: usize = 100;

/// Chunk size for systems with >16GB RAM.
const CHUNK_SIZE_LARGE: usize = 150;

const RAM_THRESHOLD_SMALL: u64 = 4;
const RAM_THRESHOLD_MEDIUM: u64 = 8;
const RAM_THRESHOLD_LARGE: u64 = 16;

// --- Resource Budget ---

/// Resource budget for concurrent operations.
#[derive(Debug, Clone, Serialize, Type)]
pub struct ResourceBudget {
    pub max_concurrent_volumes: usize,
    pub chunk_size: usize,
    pub thread_pool_size: usize,
}

impl ResourceBudget {
    /// Calculates optimal resource budget based on system resources.
    pub fn calculate() -> Self {
        let mut sys = System::new();

        // Only refresh memory - faster than new_all()
        sys.refresh_memory();

        let total_memory_bytes = sys.total_memory();
        let total_memory_gb = total_memory_bytes / 1024 / 1024 / 1024;

        let cpu_cores = num_cpus::get();

        let max_concurrent_volumes = Self::calculate_concurrency(total_memory_gb, cpu_cores);

        let chunk_size = Self::calculate_chunk_size(total_memory_gb);

        let thread_pool_size = cpu_cores.clamp(MIN_THREAD_POOL_SIZE, MAX_THREAD_POOL_SIZE);

        log::info!(
            "System: {}GB RAM, {} cores -> Budget: {} concurrent volumes, {} chunk size, {} threads",
            total_memory_gb,
            cpu_cores,
            max_concurrent_volumes,
            chunk_size,
            thread_pool_size
        );

        Self {
            max_concurrent_volumes,
            chunk_size,
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

    /// Calculates optimal chunk size from available RAM.
    #[inline]
    fn calculate_chunk_size(total_memory_gb: u64) -> usize {
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

    /// Creates custom resource budget with explicit values.
    #[inline]
    pub fn custom(
        max_concurrent_volumes: usize,
        chunk_size: usize,
        thread_pool_size: usize,
    ) -> Self {
        Self {
            max_concurrent_volumes: max_concurrent_volumes
                .clamp(MIN_CONCURRENT_VOLUMES, MAX_CONCURRENT_VOLUMES),
            chunk_size,
            thread_pool_size: thread_pool_size.clamp(MIN_THREAD_POOL_SIZE, MAX_THREAD_POOL_SIZE),
        }
    }
}

impl Default for ResourceBudget {
    fn default() -> Self {
        Self::calculate()
    }
}
