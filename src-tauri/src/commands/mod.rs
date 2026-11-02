pub mod system;
pub mod benchmarks;
pub mod export;
pub mod extra;

pub use system::{system_summary, thermal_zones};
pub use benchmarks::{
    run_cpu_benchmark, run_memory_benchmark, run_fpu_benchmark, run_cache_benchmark,
    run_disk_benchmark, run_stress_test,
};
pub use export::export_report;
pub use extra::{
    cpu_perf, memory_perf, network_perf, display_details, installed_software, event_log,
    security_posture, cpuid_info, random_disk_benchmark,
};