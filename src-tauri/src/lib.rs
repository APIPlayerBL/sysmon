pub mod commands;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::system::system_summary,
            commands::system::thermal_zones,
            commands::benchmarks::run_cpu_benchmark,
            commands::benchmarks::run_memory_benchmark,
            commands::benchmarks::run_fpu_benchmark,
            commands::benchmarks::run_cache_benchmark,
            commands::benchmarks::run_disk_benchmark,
            commands::benchmarks::run_stress_test,
            commands::extra::random_disk_benchmark,
            commands::export::export_report,
            commands::extra::cpu_perf,
            commands::extra::memory_perf,
            commands::extra::network_perf,
            commands::extra::display_details,
            commands::extra::installed_software,
            commands::extra::event_log,
            commands::extra::security_posture,
            commands::extra::cpuid_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running sysmon")
}

