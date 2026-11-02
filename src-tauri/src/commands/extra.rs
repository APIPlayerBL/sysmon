use serde_json::json;
use serde_json::Value;
use std::os::windows::process::CommandExt;
use std::process::Command;

fn run_ps(script: &str) -> Result<String, String> {
    let ps_script = format!(
        "$ErrorActionPreference='Stop'\n[Console]::OutputEncoding=[System.Text.UTF8Encoding]::new($false)\n{}",
        script
    );
    let out = Command::new(r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_script])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| format!("failed to spawn powershell: {e}"))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("powershell error: {}", stderr.trim()));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let s = stdout.trim().trim_start_matches('\u{feff}');
    Ok(s.to_string())
}

fn parse_json(s: &str) -> Result<Value, String> {
    serde_json::from_str(s).map_err(|e| format!("json parse error: {e}; raw: {}", s.chars().take(300).collect::<String>()))
}

pub fn cpu_perf_sync() -> Result<Value, String> {
    let script = r#"
try {
  $cpu = @(Get-CimInstance Win32_PerfFormattedData_PerfOS_Processor -ErrorAction Stop | Where-Object { $_.Name -ne '_Total' } | Select-Object Name, PercentProcessorTime, PercentPrivilegedTime, PercentUserTime, PercentInterruptTime)
  $freqs = @(Get-CimInstance Win32_Processor -ErrorAction Stop | Select-Object DeviceID, CurrentClockSpeed, MaxClockSpeed)
  $payload = @{ available = $true; cpu = $cpu; frequencies = $freqs }
} catch {
  $payload = @{ available = $false; error = $_.Exception.Message; cpu = @(); frequencies = @() }
}
$payload | ConvertTo-Json -Depth 8 -Compress
"#;
    let raw = run_ps(script)?;
    parse_json(&raw)
}

pub fn memory_perf_sync() -> Result<Value, String> {
    let script = r#"
try {
  $mem = (Get-CimInstance Win32_PerfFormattedData_PerfOS_Memory -ErrorAction Stop | Select-Object -First 1 AvailableMBytes, CommittedBytes, CommitLimit, PercentCommittedBytesInUse, PoolPagedBytes, PoolNonpagedBytes, PagesPerSec, PageReadsPerSec, CacheBytes, CacheBytesPeak)
  $pagefiles = @(Get-CimInstance Win32_PageFileUsage -ErrorAction Stop | Select-Object Name, CurrentUsage, AllocatedBaseSize, PeakUsage)
  $payload = @{ available = $true; memory = $mem; pagefiles = $pagefiles }
} catch {
  $payload = @{ available = $false; error = $_.Exception.Message; memory = $null; pagefiles = @() }
}
$payload | ConvertTo-Json -Depth 8 -Compress
"#;
    let raw = run_ps(script)?;
    parse_json(&raw)
}

pub fn network_perf_sync() -> Result<Value, String> {
    let script = r#"
try {
  $nic = @(Get-CimInstance Win32_PerfFormattedData_Tcpip_NetworkInterface -ErrorAction Stop | Select-Object Name, BytesReceivedPerSec, BytesSentPerSec, BytesTotalPerSec, PacketsReceivedPerSec, PacketsSentPerSec, PacketsReceivedErrors, PacketsOutboundErrors, PacketsReceivedDiscarded, PacketsOutboundDiscarded, CurrentBandwidth)
  $payload = @{ available = $true; interfaces = $nic }
} catch {
  $payload = @{ available = $false; error = $_.Exception.Message; interfaces = @() }
}
$payload | ConvertTo-Json -Depth 8 -Compress
"#;
    let raw = run_ps(script)?;
    parse_json(&raw)
}

pub fn display_details_sync() -> Result<Value, String> {
    let script = r#"
function DecodeBytes($b){ if ($null -eq $b) { "" } else { [System.Text.Encoding]::ASCII.GetString(([byte[]]$b | Where-Object { $_ -ne 0 })) } }
$wmi = @(try { Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorID -ErrorAction Stop | Select-Object InstanceName, ManufacturerName, ProductCodeID, SerialNumberID, UserFriendlyName, WeekOfManufacture, YearOfManufacture } catch { @() })
$decoded = @($wmi | ForEach-Object {
  @{
    InstanceName = $_.InstanceName
    Manufacturer = DecodeBytes $_.ManufacturerName
    ProductCode = DecodeBytes $_.ProductCodeID
    SerialNumber = DecodeBytes $_.SerialNumberID
    FriendlyName = DecodeBytes $_.UserFriendlyName
    Week = $_.WeekOfManufacture
    Year = $_.YearOfManufacture
  }
})
$desktop = @(try { Get-CimInstance Win32_DesktopMonitor -ErrorAction Stop | Select-Object Name, ScreenHeight, ScreenWidth, MonitorManufacturer, MonitorType, PNPDeviceID, Status } catch { @() })
@{ monitors = $decoded; desktop = $desktop } | ConvertTo-Json -Depth 8 -Compress
"#;
    let raw = run_ps(script)?;
    parse_json(&raw)
}

pub fn installed_software_sync() -> Result<Value, String> {
    let script = r#"
$paths = @(
  'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*',
  'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*',
  'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*'
)
$apps = @()
foreach ($path in $paths) {
  if (Test-Path $path) {
    $apps += (Get-ItemProperty $path -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName } | Select-Object DisplayName, DisplayVersion, Publisher, InstallDate, EstimatedSize)
  }
}
@{ count = $apps.Count; applications = @($apps | Sort-Object DisplayName | Select-Object -First 500) } | ConvertTo-Json -Depth 6 -Compress
"#;
    let raw = run_ps(script)?;
    parse_json(&raw)
}

pub fn event_log_sync() -> Result<Value, String> {
    let script = r#"
try {
  $start = (Get-Date).AddHours(-24)
  $events = @(Get-WinEvent -FilterHashtable @{ LogName='System'; Level=1,2,3; StartTime=$start } -MaxEvents 100 -ErrorAction Stop | Select-Object TimeCreated, Id, LevelDisplayName, ProviderName, Message)
  $payload = @{ available = $true; count = $events.Count; events = $events }
} catch {
  $payload = @{ available = $false; error = $_.Exception.Message; count = 0; events = @() }
}
$payload | ConvertTo-Json -Depth 6 -Compress
"#;
    let raw = run_ps(script)?;
    parse_json(&raw)
}

pub fn security_posture_sync() -> Result<Value, String> {
    let script = r#"
$secureBoot = $null
try { $secureBoot = (Get-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\SecureBoot\State' -Name UEFISecureBootEnabled -ErrorAction Stop).UEFISecureBootEnabled } catch { $secureBoot = $null }
$tpm = try { Get-CimInstance -Namespace root/CIMV2/Security/MicrosoftTpm -ClassName Win32_Tpm -ErrorAction Stop | Select-Object IsActivated, IsEnabled, IsOwned, SpecVersion, ManufacturerId, ManufacturerVersion } catch { $null }
$bitlocker = try { Get-CimInstance -Namespace root/CIMV2/Security/MicrosoftVolumeEncryption -ClassName Win32_EncryptableVolume -ErrorAction Stop | Select-Object DriveLetter, ProtectionStatus, ConversionStatus, EncryptionMethod } catch { $null }
@{ secure_boot_enabled = $secureBoot; tpm = $tpm; bitlocker_volumes = @($bitlocker) } | ConvertTo-Json -Depth 8 -Compress
"#;
    let raw = run_ps(script)?;
    parse_json(&raw)
}

pub fn random_disk_benchmark_sync() -> Result<Value, String> {
    use std::fs::{File, remove_file};
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::time::Instant;

    let size_mb: usize = 64;
    let bytes = size_mb * 1024 * 1024;
    let block = 4096usize;
    let path = std::env::temp_dir().join(format!("sysmon_rand_{}.tmp", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    {
        let mut f = std::fs::OpenOptions::new().write(true).create_new(true).open(&path).map_err(|e| format!("create: {e}"))?;
        let chunk = vec![0x37u8; block];
        let mut written = 0usize;
        while written < bytes {
            f.write_all(&chunk).map_err(|e| format!("write fill: {e}"))?;
            written += block;
        }
        f.sync_all().map_err(|e| format!("sync: {e}"))?;
    }
    let ops = 20_000usize;
    let t0 = Instant::now();
    let mut read_acc = 0u64;
    {
        let mut f = File::open(&path).map_err(|e| format!("open read: {e}"))?;
        let mut buf = vec![0u8; block];
        let mut x = 0x9e3779b97f4a7c15u64;
        for _ in 0..ops {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
            let off = ((x as usize) % (bytes / block)) * block;
            f.seek(SeekFrom::Start(off as u64)).map_err(|e| format!("seek: {e}"))?;
            f.read_exact(&mut buf).map_err(|e| format!("read: {e}"))?;
            read_acc = read_acc.wrapping_add(buf[0] as u64);
        }
    }
    let read_ms = t0.elapsed().as_millis().max(1);
    let read_iops = ops as f64 / (read_ms as f64 / 1000.0);
    let t1 = Instant::now();
    let mut x = 0xc2b2ae3d27d4eb4fu64;
    {
        let mut f = std::fs::OpenOptions::new().write(true).open(&path).map_err(|e| format!("open write: {e}"))?;
        let buf = vec![0x5Au8; block];
        for _ in 0..ops {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
            let off = ((x as usize) % (bytes / block)) * block;
            f.seek(SeekFrom::Start(off as u64)).map_err(|e| format!("seek: {e}"))?;
            f.write_all(&buf).map_err(|e| format!("write: {e}"))?;
        }
        f.sync_all().map_err(|e| format!("sync: {e}"))?;
    }
    let write_ms = t1.elapsed().as_millis().max(1);
    let write_iops = ops as f64 / (write_ms as f64 / 1000.0);
    let _ = std::hint::black_box(read_acc);
    let _ = remove_file(&path);
    Ok(json!({
        "size_mb": size_mb,
        "operations": ops,
        "block_bytes": block,
        "read_elapsed_ms": read_ms,
        "read_iops": read_iops,
        "write_elapsed_ms": write_ms,
        "write_iops": write_iops
    }))
}

#[allow(unused_unsafe)]
pub fn cpuid_info_sync() -> Result<Value, String> {
    #[cfg(target_arch = "x86_64")]
    {
        use std::arch::x86_64::__cpuid;
        unsafe {
            let c0 = __cpuid(0);
            let vendor = format!(
                "{}{}{}",
                String::from_utf8_lossy(&c0.ebx.to_le_bytes()),
                String::from_utf8_lossy(&c0.edx.to_le_bytes()),
                String::from_utf8_lossy(&c0.ecx.to_le_bytes())
            );
            let c1 = __cpuid(1);
            let family = (c1.eax >> 8) & 0x0F;
            let model = (c1.eax >> 4) & 0x0F;
            let stepping = c1.eax & 0x0F;
            let extended_family = (c1.eax >> 20) & 0xFF;
            let extended_model = (c1.eax >> 16) & 0x0F;
            let brand = format!("family={} model={} stepping={} extended_family={} extended_model={}", family, model, stepping, extended_family, extended_model);
            let features = serde_json::json!({
                "sse": (c1.edx & (1 << 25)) != 0,
                "sse2": (c1.edx & (1 << 26)) != 0,
                "htt": (c1.edx & (1 << 28)) != 0,
                "avx": (c1.ecx & (1 << 28)) != 0,
                "aes": (c1.ecx & (1 << 25)) != 0,
                "rdrand": (c1.ecx & (1 << 30)) != 0
            });
            let payload = serde_json::json!({
                "available": true,
                "vendor": vendor.trim(),
                "brand_summary": brand,
                "features": features
            });
            Ok(payload)
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        Ok(serde_json::json!({"available": false, "error": "not x86_64"}))
    }
}

#[tauri::command]
pub async fn cpu_perf() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(cpu_perf_sync).await.map_err(|e| format!("task join error: {}", e))?
}
#[tauri::command]
pub async fn memory_perf() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(memory_perf_sync).await.map_err(|e| format!("task join error: {}", e))?
}
#[tauri::command]
pub async fn network_perf() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(network_perf_sync).await.map_err(|e| format!("task join error: {}", e))?
}
#[tauri::command]
pub async fn display_details() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(display_details_sync).await.map_err(|e| format!("task join error: {}", e))?
}
#[tauri::command]
pub async fn installed_software() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(installed_software_sync).await.map_err(|e| format!("task join error: {}", e))?
}
#[tauri::command]
pub async fn event_log() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(event_log_sync).await.map_err(|e| format!("task join error: {}", e))?
}
#[tauri::command]
pub async fn security_posture() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(security_posture_sync).await.map_err(|e| format!("task join error: {}", e))?
}
#[tauri::command]
pub async fn random_disk_benchmark() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(random_disk_benchmark_sync).await.map_err(|e| format!("task join error: {}", e))?
}
#[tauri::command]
pub async fn cpuid_info() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(cpuid_info_sync).await.map_err(|e| format!("task join error: {}", e))?
}

