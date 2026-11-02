use serde_json::json;
use std::time::Instant;

pub fn run_cpu_benchmark_sync() -> Result<serde_json::Value, String> {
    // Integer: sieve of Eratosthenes up to N
    let n: usize = 5_000_000;
    let t0 = Instant::now();
    let mut sieve = vec![true; n + 1];
    sieve[0] = false;
    if n >= 1 { sieve[1] = false; }
    let mut i = 2usize;
    while i * i <= n {
        if sieve[i] {
            let mut j = i * i;
            while j <= n {
                sieve[j] = false;
                j += i;
            }
        }
        i += 1;
    }
    let prime_count: usize = sieve.iter().filter(|b| **b).count();
    let int_ms = t0.elapsed().as_millis();

    // Floating point: sqrt loop
    let t1 = Instant::now();
    let mut acc = 0.0f64;
    let iters: u64 = 50_000_000;
    for k in 0..iters {
        acc += ((k as f64) + 1.0).sqrt();
    }
    let fpu_ms = t1.elapsed().as_millis();

    Ok(json!({
        "sieve_limit": n,
        "prime_count": prime_count,
        "int_elapsed_ms": int_ms,
        "fpu_iterations": iters,
        "fpu_elapsed_ms": fpu_ms,
        "fpu_accumulator": acc
    }))
}

pub fn run_memory_benchmark_sync() -> Result<serde_json::Value, String> {
    let size: usize = 16 * 1024 * 1024; // 16M u64 = 128MB
    let t0 = Instant::now();
    let src: Vec<u64> = vec![1u64; size];
    let mut dst: Vec<u64> = vec![0u64; size];
    dst.copy_from_slice(&src);
    // simple transform to defeat optimization
    for v in dst.iter_mut() {
        *v = v.wrapping_mul(31).wrapping_add(7);
    }
    let elapsed_ms = t0.elapsed().as_millis().max(1);
    let bytes = (size as u64) * 8 * 2; // read + write approx
    let mb = bytes as f64 / (1024.0 * 1024.0);
    let bandwidth_mb_s = mb / (elapsed_ms as f64 / 1000.0);
    Ok(json!({
        "elements": size,
        "bytes_moved_approx": bytes,
        "elapsed_ms": elapsed_ms,
        "bandwidth_mb_s": bandwidth_mb_s
    }))
}


#[tauri::command]
pub async fn run_cpu_benchmark() -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(run_cpu_benchmark_sync).await.map_err(|e| format!("task join error: {}", e))?
}

#[tauri::command]
pub async fn run_memory_benchmark() -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(run_memory_benchmark_sync).await.map_err(|e| format!("task join error: {}", e))?
}


pub fn run_fpu_benchmark_sync() -> Result<serde_json::Value, String> {
    let iters: u64 = 25_000_000;
    let t0 = Instant::now();
    let mut acc = 0.0f64;
    for k in 0..iters {
        let x = k as f64 + 1.0;
        acc += x.sin().cos().tan().sqrt();
    }
    let elapsed_ms = t0.elapsed().as_millis().max(1);
    let ops_per_sec = (iters as f64) / (elapsed_ms as f64 / 1000.0);
    // prevent optimization
    let _ = std::hint::black_box(acc);
    Ok(json!({
        "iterations": iters,
        "elapsed_ms": elapsed_ms,
        "operations_per_second": ops_per_sec,
        "accumulator": acc
    }))
}

pub fn run_cache_benchmark_sync() -> Result<serde_json::Value, String> {
    #[derive(serde::Serialize)]
    struct Level { name: String, size_bytes: usize, passes: usize, elapsed_ms: u128, bandwidth_mb_s: f64 }

    let levels = [
        ("L1", 32usize * 1024),
        ("L2", 256usize * 1024),
        ("L3", 4usize * 1024 * 1024),
    ];
    let passes = 64usize;
    let mut out = Vec::new();
    for (name, size_bytes) in levels {
        let words = size_bytes / 8;
        let buf: Vec<u64> = (0..words as u64).collect();
        // warm up
        let mut acc = 0u64;
        for &v in &buf { acc = acc.wrapping_add(v); }
        let t0 = Instant::now();
        for _ in 0..passes {
            for &v in &buf { acc = acc.wrapping_add(v); }
        }
        let elapsed_ms = t0.elapsed().as_millis().max(1);
        let total_bytes = size_bytes as u64 * passes as u64;
        let bandwidth_mb_s = (total_bytes as f64 / (1024.0 * 1024.0)) / (elapsed_ms as f64 / 1000.0);
        let _ = std::hint::black_box(acc);
        out.push(Level {
            name: name.to_string(),
            size_bytes,
            passes,
            elapsed_ms,
            bandwidth_mb_s,
        });
    }
    Ok(json!({ "levels": out }))
}

#[tauri::command]
pub async fn run_fpu_benchmark() -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(run_fpu_benchmark_sync).await.map_err(|e| format!("task join error: {}", e))?
}

#[tauri::command]
pub async fn run_cache_benchmark() -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(run_cache_benchmark_sync).await.map_err(|e| format!("task join error: {}", e))?
}



pub fn run_disk_benchmark_sync() -> Result<serde_json::Value, String> {
    use std::fs::{File, remove_file};
    use std::io::{BufWriter, BufReader, Read, Write};
    use std::time::Instant;

    let size_mb: usize = 64;
    let bytes = size_mb * 1024 * 1024;
    let path = std::env::temp_dir().join(format!("sysmon_disk_bench_{}.tmp", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let path_str = path.to_string_lossy().to_string();

    // Sequential write
    let t0 = Instant::now();
    {
        let f = File::create(&path).map_err(|e| format!("create temp file: {e}"))?;
        let mut w = BufWriter::with_capacity(1024 * 1024, f);
        let chunk = vec![0x5Au8; 1024 * 1024];
        let mut written = 0usize;
        while written < bytes {
            let n = (bytes - written).min(chunk.len());
            w.write_all(&chunk[..n]).map_err(|e| format!("write: {e}"))?;
            written += n;
        }
        w.flush().map_err(|e| format!("flush: {e}"))?;
    }
    let write_ms = t0.elapsed().as_millis().max(1);
    let write_mb_s = (bytes as f64 / (1024.0 * 1024.0)) / (write_ms as f64 / 1000.0);

    // Sequential read
    let t1 = Instant::now();
    {
        let f = File::open(&path).map_err(|e| format!("open temp file: {e}"))?;
        let mut r = BufReader::with_capacity(1024 * 1024, f);
        let mut buf = vec![0u8; 1024 * 1024];
        let mut total = 0usize;
        loop {
            let n = r.read(&mut buf).map_err(|e| format!("read: {e}"))?;
            if n == 0 { break; }
            total += n;
            // touch data to prevent dead-code elimination
            let _ = std::hint::black_box(buf[0]);
        }
        if total != bytes {
            return Err(format!("short read: expected {bytes}, got {total}"));
        }
    }
    let read_ms = t1.elapsed().as_millis().max(1);
    let read_mb_s = (bytes as f64 / (1024.0 * 1024.0)) / (read_ms as f64 / 1000.0);

    let _ = remove_file(&path);
    Ok(json!({
        "size_mb": size_mb,
        "path": path_str,
        "write_elapsed_ms": write_ms,
        "write_mb_s": write_mb_s,
        "read_elapsed_ms": read_ms,
        "read_mb_s": read_mb_s
    }))
}

pub fn run_stress_test_sync(seconds: u64) -> Result<serde_json::Value, String> {
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    let secs = seconds.clamp(2, 30);
    let threads = thread::available_parallelism().map(|x| x.get()).unwrap_or(4).min(16);
    let stop = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::new();
    let mut counters = Vec::new();

    for _ in 0..threads {
        let stop = Arc::clone(&stop);
        let counter = Arc::new(AtomicU64::new(0));
        counters.push(Arc::clone(&counter));
        handles.push(thread::spawn(move || {
            let mut acc = 0u64;
            while !stop.load(Ordering::Relaxed) {
                // mixed small workload to load CPU and FPU
                for i in 0..1_000u64 {
                    acc = acc.wrapping_add(i.wrapping_mul(31).wrapping_add(7));
                    let _ = (i as f64 + 1.0).sqrt();
                }
                counter.fetch_add(1000, Ordering::Relaxed);
            }
            std::hint::black_box(acc);
        }));
    }

    thread::sleep(Duration::from_secs(secs));
    stop.store(true, Ordering::Relaxed);
    for h in handles { let _ = h.join(); }

    let total_iters: u64 = counters.iter().map(|c| c.load(Ordering::Relaxed)).sum();
    let ops_per_sec = total_iters as f64 / secs as f64;
    Ok(json!({
        "duration_seconds": secs,
        "threads": threads,
        "total_iterations": total_iters,
        "iterations_per_second": ops_per_sec
    }))
}

#[tauri::command]
pub async fn run_disk_benchmark() -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(run_disk_benchmark_sync).await.map_err(|e| format!("task join error: {}", e))?
}

#[tauri::command]
pub async fn run_stress_test(seconds: u64) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || run_stress_test_sync(seconds)).await.map_err(|e| format!("task join error: {}", e))?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cpu_benchmark_returns_expected_prime_count() {
        let v = run_cpu_benchmark_sync().expect("cpu benchmark");
        assert_eq!(v["prime_count"].as_u64().unwrap(), 348513);
        let _ = v["int_elapsed_ms"].as_u64();
    }
    #[test]
    fn memory_benchmark_positive_bandwidth() {
        let v = run_memory_benchmark_sync().expect("mem benchmark");
        assert!(v["bandwidth_mb_s"].as_f64().unwrap() > 0.0);
    }
    #[test]
    fn disk_benchmark_reads_written_bytes() {
        let v = run_disk_benchmark_sync().expect("disk benchmark");
        assert_eq!(v["size_mb"].as_u64().unwrap(), 64);
        assert!(v["write_mb_s"].as_f64().unwrap() > 0.0);
        assert!(v["read_mb_s"].as_f64().unwrap() > 0.0);
    }
}

