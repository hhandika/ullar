//! Map reads to reference using minimap2.

use crate::helper::common::SystemInfo;
use minimap2::{Aligner, Built};
use std::path::Path;

const MINIMAP_THREAD_MEMORY: u64 = 500_000; // in KB (200 MB per thread)
const BUFFER_MEMORY: u64 = 1000_000; // in KB (200 MB buffer)
/// Minimap2 mapper
/// Requires:
/// reference_path - path to the reference genome
/// query_path - path to the reads to be mapped
pub struct MinimapMapping<'a> {
    pub reference_path: &'a Path,
    pub query_path: &'a Path,
}

impl<'a> MinimapMapping<'a> {
    /// Create a new MinimapMapping instance
    pub fn new(reference_path: &'a Path, query_path: &'a Path) -> Self {
        Self {
            reference_path,
            query_path,
        }
    }

    /// Generate the minimap2 command to map reads to the reference
    pub fn build_aligner(&self) -> Aligner<Built> {
        let available_cpu_threads = self.get_cpu_threads();
        Aligner::builder()
            .sr()
            .with_index_threads(available_cpu_threads)
            .with_cigar()
            .with_index(self.reference_path, None)
            .expect("Failed to build minimap2 aligner")
    }

    // Get CPU counts based on available memory
    fn get_cpu_threads(&self) -> usize {
        let mut sysinfo = SystemInfo::new();
        sysinfo.get();

        let available_memory =
            if sysinfo.available_memory == 0 && sysinfo.available_memory < MINIMAP_THREAD_MEMORY {
                sysinfo.total_memory
            } else {
                sysinfo.available_memory
            };
        let max_threads_by_memory = (available_memory / MINIMAP_THREAD_MEMORY) - BUFFER_MEMORY;
        let sys_threads = sysinfo.threads as u64;
        let threads = std::cmp::min(sys_threads, max_threads_by_memory) as usize;
        // We want at least 1 thread to run minimap2 just in case the calculation gives 0
        std::cmp::max(1, threads)
    }
}
