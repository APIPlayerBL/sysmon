use serde_json::Value;
use std::os::windows::process::CommandExt;
use std::process::Command;

fn run_ps(script: &str) -> Result<String, String> {
    let ps_script = format!(
        "$ErrorActionPreference='Stop'\n$OutputEncoding=[Console]::OutputEncoding=[System.Text.UTF8Encoding]::new($false)\n[Console]::OutputEncoding=[System.Text.UTF8Encoding]::new($false)\n{}",
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

pub fn system_summary_sync() -> Result<Value, String> {
    let script = r#"
function Wrap($x){ if ($null -eq $x) { @() } elseif ($x -is [array]) { $x } else { @($x) } }
$servicesAll = @(Get-CimInstance Win32_Service)
$processesAll = @(Get-CimInstance Win32_Process)

$payload = @{
  os = (Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version,BuildNumber,OSArchitecture,TotalVisibleMemorySize,FreePhysicalMemory,LastBootUpTime,RegisteredUser,SerialNumber,InstallDate,NumberOfUsers,NumberOfProcesses,SystemDrive,SystemDevice,Locale)
  computersystem = (Get-CimInstance Win32_ComputerSystem | Select-Object Manufacturer,Model,SystemType,TotalPhysicalMemory,UserName,NumberOfProcessors,NumberOfLogicalProcessors,DNSHostName,Domain,Workgroup,HypervisorPresent)
  computersystemproduct = (Get-CimInstance Win32_ComputerSystemProduct | Select-Object Vendor,Name,IdentifyingNumber,Version,UUID,SKUNumber)
  processors = @(Get-CimInstance Win32_Processor | Select-Object Name,Manufacturer,NumberOfCores,NumberOfLogicalProcessors,MaxClockSpeed,L2CacheSize,L3CacheSize,SocketDesignation,Architecture,LoadPercentage,DeviceID,Family,VirtualizationFirmwareEnabled,VMMonitorModeExtensions)
  memory_modules = @(Get-CimInstance Win32_PhysicalMemory | Select-Object Manufacturer,PartNumber,Capacity,Speed,MemoryType,FormFactor,BankLabel,DeviceLocator,SerialNumber,ConfiguredClockSpeed,TotalWidth,DataWidth)
  gpus = @(Get-CimInstance Win32_VideoController | Select-Object Name,AdapterRAM,DriverVersion,Status,VideoProcessor,CurrentHorizontalResolution,CurrentVerticalResolution,CurrentRefreshRate,VideoModeDescription,InstalledDisplayDrivers,PNPDeviceID,AdapterCompatibility)
  disks = @(Get-CimInstance Win32_DiskDrive | Select-Object Model,Size,MediaType,InterfaceType,Partitions,SerialNumber,Status,Index,BytesPerSector,TotalCylinders,TotalHeads,TotalSectors,TotalTracks,TracksPerCylinder,PNPDeviceID,FirmwareRevision)
  logical_disks = @(Get-CimInstance Win32_LogicalDisk | Select-Object DeviceID,VolumeName,FileSystem,Size,FreeSpace,DriveType,Description,Compressed,SupportsDiskQuotas,VolumeSerialNumber)
  smart_status = @(try { Get-CimInstance -Namespace root/WMI -ClassName MSStorageDriver_FailurePredictStatus -ErrorAction Stop | Select-Object InstanceName,PredictFailure,Reason,Active } catch { @() })
  motherboard = (Get-CimInstance Win32_BaseBoard | Select-Object Manufacturer,Product,Version,SerialNumber,Tag,Weight,Height,Width)
  bios = (Get-CimInstance Win32_BIOS | Select-Object Manufacturer,SMBIOSBIOSVersion,ReleaseDate,SerialNumber,Version,PrimaryBIOS,EmbeddedControllerMajorVersion,EmbeddedControllerMinorVersion,BIOSVersion)
  system_enclosure = (Get-CimInstance Win32_SystemEnclosure | Select-Object Manufacturer,Model,SerialNumber,ChassisTypes,SMBIOSAssetTag,HotSwappable,LockPresent,PoweredOn,SecurityBreach)
  network = @(Get-CimInstance Win32_NetworkAdapterConfiguration -Filter "IPEnabled=True" | Select-Object Description,MACAddress,IPAddress,DHCPEnabled,DNSHostName,DNSServerSearchOrder,DefaultIPGateway,IPSubnet,DHCPServer,LeaseObtained,LeaseExpires)
  network_adapters = @(Get-CimInstance Win32_NetworkAdapter | Where-Object { $_.NetEnabled -eq $true } | Select-Object Name,AdapterType,MACAddress,NetConnectionID,Speed,PhysicalAdapter,PNPDeviceID)
  batteries = @(Get-CimInstance Win32_Battery | Select-Object Name,Status,Availability,BatteryStatus,Chemistry,DesignCapacity,FullChargeCapacity,EstimatedChargeRemaining,EstimatedRunTime,ExpectedLife,MaxRechargeTime,TimeOnBattery,Voltage)
  portable_batteries = @(Get-CimInstance Win32_PortableBattery | Select-Object Name,Description,Location,CapacityMultiplier,Date,Chemistry,Tag,Voltage,Weight)
  monitors = @(try { Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorID -ErrorAction Stop | Select-Object InstanceName, ManufacturerName, ProductCodeID, SerialNumberID, UserFriendlyName, WeekOfManufacture, YearOfManufacture } catch { @() })
  monitor_params = @(try { Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBasicDisplayParams -ErrorAction Stop | Select-Object InstanceName, MaxHorizontalImageSize, MaxVerticalImageSize, SupportedDisplayFeatures, VideoInputType } catch { @() })
  monitor_brightness = @(try { Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightness -ErrorAction Stop | Select-Object InstanceName, CurrentBrightness, Level } catch { @() })
  audio = @(Get-CimInstance Win32_SoundDevice | Select-Object Name,Manufacturer,ProductName,Status,PNPDeviceID,DeviceID)
  printers = @(Get-CimInstance Win32_Printer | Select-Object Name,DriverName,PortName,Status,Default,Network,Local,Shared,PrinterStatus,WorkOffline,PNPDeviceID)
  cdroms = @(Get-CimInstance Win32_CDROMDrive | Select-Object Name,Drive,MediaLoaded,MediaType,PNPDeviceID,Status)
  usb_controllers = @(Get-CimInstance Win32_USBController | Select-Object Name,Manufacturer,PNPDeviceID,Status,ProtocolSupported)
  usb_devices = @(Get-CimInstance Win32_PnPEntity | Where-Object { $_.PNPClass -eq 'USB' } | Select-Object -First 80 Name,Manufacturer,DeviceID,Status,Service,PNPClass)
  keyboards = @(Get-CimInstance Win32_Keyboard | Select-Object Name,Description,DeviceID,NumberOfFunctionKeys,Status)
  pointing_devices = @(Get-CimInstance Win32_PointingDevice | Select-Object Name,HardwareType,NumberOfButtons,DeviceInterface,Status)
  services = @{
      total = $servicesAll.Count
      running = ($servicesAll | Where-Object { $_.State -eq 'Running' } | Measure-Object).Count
      stopped = ($servicesAll | Where-Object { $_.State -ne 'Running' } | Measure-Object).Count
      top_running = @($servicesAll | Where-Object { $_.State -eq 'Running' } | Select-Object -First 40 Name,DisplayName,State,StartMode,ProcessId,PathName)
  }
  processes_info = @{
      total = $processesAll.Count
      top = @($processesAll | Sort-Object WorkingSetSize -Descending | Select-Object -First 15 Name,ProcessId,WorkingSetSize,VirtualSize,ParentProcessId,ExecutionState)
  }
  hotfix = @(Get-CimInstance Win32_QuickFixEngineering | Select-Object -First 40 HotFixID,InstalledOn,Description,FixComments,InstalledBy)
  drivers = @(Get-CimInstance Win32_SystemDriver | Select-Object -First 60 Name,DisplayName,State,Status,PathName,ServiceType)
}
$payload | ConvertTo-Json -Depth 12 -Compress
"#;
    let raw = run_ps(script)?;
    parse_json(&raw)
}

pub fn thermal_zones_sync() -> Result<Value, String> {
    let script = r#"
try {
  $zones = @(Get-CimInstance -Namespace root/WMI -ClassName MSAcpi_ThermalZoneTemperature -ErrorAction Stop | Select-Object InstanceName, CurrentTemperature, CriticalTripPoint, PassiveTripPoint)
} catch {
  $zones = @()
}
try {
  $probes = @(Get-CimInstance Win32_TemperatureProbe -ErrorAction Stop | Select-Object Name,Description,CurrentReading,Status,MaxReadable,MinReadable)
} catch {
  $probes = @()
}
@{ thermals = $zones; probes = $probes } | ConvertTo-Json -Depth 8 -Compress
"#;
    let raw = run_ps(script)?;
    parse_json(&raw)
}

#[tauri::command]
pub async fn system_summary() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(system_summary_sync)
        .await
        .map_err(|e| format!("task join error: {e}"))?
}

#[tauri::command]
pub async fn thermal_zones() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(thermal_zones_sync)
        .await
        .map_err(|e| format!("task join error: {e}"))?
}
