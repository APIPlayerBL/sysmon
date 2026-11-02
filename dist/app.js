const { invoke } = (window.__TAURI__ && window.__TAURI__.core) || {};
const status = (m) => { const e = document.getElementById('status'); if (e) e.textContent = m; };

const state = {
  summary: null,
  cpuPerf: null,
  memPerf: null,
  netPerf: null,
  display: null,
  software: null,
  eventLog: null,
  security: null,
  cpuid: null,
  loaded: new Set()
};

const VIEW_TITLES = {
  computer: "Компьютер", cpu: "CPU", memory: "Память",
  board: "Материнская плата / BIOS", display: "Дисплей / GPU",
  storage: "Хранилище", network: "Сеть", devices: "Устройства",
  sensors: "Датчики", software: "ПО / безопасность",
  benchmark: "Бенчмарки / стресс", report: "Отчёт / экспорт"
};

const VIEW_NOTES = {
  computer: "Общая информация: операционная система, компьютер, продукт, корпус, процессы и службы.",
  cpu: "Процессор, частоты, кэш, нагрузка, производительность и CPUID.",
  memory: "Модули памяти и статистика использования памяти.",
  board: "Материнская плата, BIOS и сведения о корпусе.",
  display: "Видеокарты, мониторы, разрешение, яркость и драйверы.",
  storage: "Физические и логические диски, SMART, оптические устройства.",
  network: "Сетевые адаптеры, IP-конфигурация и трафик.",
  devices: "Аудио, принтеры, USB, клавиатуры и указатели.",
  sensors: "Батареи, термозоны и датчики температуры.",
  software: "Драйверы, обновления, установленное ПО, события и безопасность.",
  benchmark: "Тесты производительности CPU, памяти, кэша и диска; стресс-тест CPU.",
  report: "Формирование и экспорт отчёта в JSON/CSV/HTML/TXT/LOG."
};

const LABELS = {
  os: "Операционная система", computersystem: "Компьютер", computersystemproduct: "Продукт системы",
  system_enclosure: "Корпус", processes_info: "Процессы (сводка)", services: "Службы (сводка)",
  processors: "Процессоры", memory_modules: "Модули памяти", gpus: "Видеокарты / GPU",
  disks: "Физические диски", logical_disks: "Логические диски", smart_status: "SMART-состояние",
  motherboard: "Материнская плата", bios: "BIOS", network: "Сеть (IP-конфигурация)",
  network_adapters: "Сетевые адаптеры", batteries: "Батареи", portable_batteries: "Портативные батареи",
  monitors: "Мониторы (WMI)", monitor_params: "Параметры мониторов", monitor_brightness: "Яркость мониторов",
  audio: "Аудиоустройства", printers: "Принтеры", cdroms: "Оптические приводы",
  usb_controllers: "USB-контроллеры", usb_devices: "USB-устройства", keyboards: "Клавиатуры",
  pointing_devices: "Указывающие устройства", drivers: "Системные драйверы", hotfix: "Обновления (Hotfix)",
  cpu_perf: "Производительность CPU", cpuid: "CPUID", memory_perf: "Производительность памяти",
  network_perf: "Производительность сети", display_details: "Детали дисплеев",
  installed_software: "Установленное ПО", event_log: "Журнал событий", security_posture: "Безопасность",
  thermal_zones_data: "Термозоны", thermals: "Термозоны", probes: "Датчики температуры",
  available: "Доступно", error: "Ошибка", count: "Количество", applications: "Приложения",
  events: "События", secure_boot_enabled: "Secure Boot включён", tpm: "TPM", bitlocker_volumes: "Тома BitLocker",
  cpu: "CPU", frequencies: "Частоты", memory: "Память", pagefiles: "Файлы подкачки",
  interfaces: "Интерфейсы", monitors_list: "Мониторы", desktop: "Рабочие столы", wmi: "WMI"
};

const KEY_LABELS = {
  Caption: "Название", Version: "Версия", BuildNumber: "Сборка", OSArchitecture: "Архитектура ОС",
  TotalVisibleMemorySize: "Видимая память (КБ)", FreePhysicalMemory: "Свободная память (КБ)",
  LastBootUpTime: "Последняя загрузка", RegisteredUser: "Зарегистрированный пользователь",
  SerialNumber: "Серийный номер", InstallDate: "Дата установки", NumberOfUsers: "Пользователей",
  NumberOfProcesses: "Процессов", SystemDrive: "Системный диск", SystemDevice: "Системное устройство",
  Locale: "Локаль", Manufacturer: "Производитель", Model: "Модель", SystemType: "Тип системы",
  TotalPhysicalMemory: "Общая физ. память (байт)", UserName: "Пользователь", NumberOfProcessors: "Процессоров",
  NumberOfLogicalProcessors: "Логических процессоров", DNSHostName: "Имя хоста", Domain: "Домен",
  Workgroup: "Рабочая группа", HypervisorPresent: "Гипервизор", Vendor: "Поставщик", Name: "Имя",
  IdentifyingNumber: "Идентификатор продукта", UUID: "UUID", SKUNumber: "SKU",
  ManufacturerName: "Производитель (WMI)", ProductCodeID: "Код продукта", SerialNumberID: "Серийный номер (WMI)",
  UserFriendlyName: "Название монитора", WeekOfManufacture: "Неделя выпуска", YearOfManufacture: "Год выпуска",
  MaxHorizontalImageSize: "Макс. ширина (см)", MaxVerticalImageSize: "Макс. высота (см)",
  SupportedDisplayFeatures: "Поддерживаемые функции", VideoInputType: "Тип видеовхода",
  CurrentBrightness: "Текущая яркость", Level: "Уровни яркости", InstanceName: "Экземпляр",
  PredictFailure: "Прогноз отказа", Reason: "Причина", Active: "Активен", State: "Состояние",
  StartMode: "Режим запуска", ProcessId: "PID", PathName: "Путь", DisplayName: "Отображаемое имя",
  WorkingSetSize: "ОЗУ процесса (байт)", VirtualSize: "Вирт. память (байт)", ParentProcessId: "Родительский PID",
  ExecutionState: "Состояние выполнения", ProviderName: "Источник", TimeCreated: "Время", Id: "ID",
  LevelDisplayName: "Уровень", Message: "Сообщение", DisplayNameVersion: "Версия",
  DisplayVersion: "Версия", Publisher: "Издатель", InstallDateRaw: "Дата установки", EstimatedSize: "Размер (КБ)",
  Description: "Описание", HotFixID: "ID обновления", InstalledOn: "Установлено", InstalledBy: "Кем установлено",
  FixComments: "Комментарии", Path: "Путь", ServiceType: "Тип службы", Status: "Статус",
  PNPDeviceID: "PNP Device ID", PNPClass: "PNP-класс", DeviceID: "ID устройства", Service: "Служба",
  NumberOfFunctionKeys: "Клавиш функций", HardwareType: "Тип оборудования", NumberOfButtons: "Кнопок",
  DeviceInterface: "Интерфейс", AdapterRAM: "Память адаптера", DriverVersion: "Версия драйвера",
  CurrentHorizontalResolution: "Ширина", CurrentVerticalResolution: "Высота", CurrentRefreshRate: "Частота (Гц)",
  VideoModeDescription: "Режим видео", VideoProcessor: "Видеопроцессор", AdapterCompatibility: "Совместимость",
  ScreenWidth: "Ширина экрана", ScreenHeight: "Высота экрана", MonitorManufacturer: "Производитель монитора",
  MonitorType: "Тип монитора", Size: "Размер (байт)", MediaType: "Тип носителя", InterfaceType: "Интерфейс",
  Partitions: "Разделов", BytesPerSector: "Байт/сектор", FirmwareRevision: "Прошивка",
  VolumeName: "Метка тома", FileSystem: "ФС", FreeSpace: "Свободно (байт)", DriveType: "Тип диска",
  Compressed: "Сжат", SupportsDiskQuotas: "Квоты", VolumeSerialNumber: "Серийный тома",
  Chemistry: "Химия", DesignCapacity: "Проектная ёмкость", FullChargeCapacity: "Полный заряд",
  EstimatedChargeRemaining: "Заряд (%)", EstimatedRunTime: "Расчётное время (мин)", Voltage: "Напряжение (мВ)",
  CapacityMultiplier: "Множитель ёмкости", Location: "Расположение", Date: "Дата", Tag: "Тег",
  Weight: "Вес", Height: "Высота", Width: "Ширина", ChassisTypes: "Типы корпуса",
  SMBIOSAssetTag: "Инв. номер", HotSwappable: "Горячая замена", LockPresent: "Замок", PoweredOn: "Питание вкл.",
  SecurityBreach: "Нарушение безопасности", BIOSVersion: "Версии BIOS", PrimaryBIOS: "Основной BIOS",
  SMBIOSBIOSVersion: "Версия SMBIOS", ReleaseDate: "Дата релиза", EmbeddedControllerMajorVersion: "EC major",
  EmbeddedControllerMinorVersion: "EC minor", MaxClockSpeed: "Макс. частота (МГц)", CurrentClockSpeed: "Текущая частота (МГц)",
  NumberOfCores: "Ядра", NumberOfLogicalProcessors: "Логические процессоры", Architecture: "Архитектура",
  Family: "Семейство", LoadPercentage: "Загрузка (%)", SocketDesignation: "Сокет", L2CacheSize: "Кэш L2 (КБ)",
  L3CacheSize: "Кэш L3 (КБ)", VirtualizationFirmwareEnabled: "Виртуализация firmware", VMMonitorModeExtensions: "VM monitor ext",
  PartNumber: "Номер детали", Speed: "Скорость", MemoryType: "Тип памяти", FormFactor: "Форм-фактор",
  BankLabel: "Банк", DeviceLocator: "Разъём", ConfiguredClockSpeed: "Настроенная частота (МГц)",
  TotalWidth: "Общая ширина (бит)", DataWidth: "Ширина данных (бит)", ConfiguredVoltage: "Напряжение (мВ)",
  TotalPhysicalMemory: "ОЗУ (байт)", PercentProcessorTime: "Загрузка CPU (%)",
  PercentPrivilegedTime: "Привилегированное (%)", PercentUserTime: "Пользовательское (%)",
  PercentInterruptTime: "Прерывания (%)", Name: "Имя", MACAddress: "MAC-адрес", IPAddress: "IP-адрес",
  DHCPEnabled: "DHCP", DNSHostName: "DNS-имя", DNSServerSearchOrder: "DNS-серверы",
  DefaultIPGateway: "Шлюз", IPSubnet: "Подсеть", DHCPServer: "DHCP-сервер",
  LeaseObtained: "Аренда получена", LeaseExpires: "Аренда истекает", AdapterType: "Тип адаптера",
  NetConnectionID: "Подключение", PhysicalAdapter: "Физический", Speed: "Скорость",
  AvailableMBytes: "Доступно (МБ)", CommittedBytes: "Зафиксировано (байт)", CommitLimit: "Лимит (байт)",
  PercentCommittedBytesInUse: "Зафиксировано (%)", PoolPagedBytes: "Paged pool (байт)",
  PoolNonpagedBytes: "Nonpaged pool (байт)", PagesPerSec: "Страниц/сек", PageReadsPerSec: "Чтение страниц/сек",
  CacheBytes: "Кэш (байт)", CacheBytesPeak: "Пик кэша (байт)", CurrentUsage: "Использование",
  AllocatedBaseSize: "Базовый размер", PeakUsage: "Пик", BytesReceivedPerSec: "Получено (байт/сек)",
  BytesSentPerSec: "Отправлено (байт/сек)", BytesTotalPerSec: "Всего (байт/сек)",
  PacketsReceivedPerSec: "Пакеты получены/сек", PacketsSentPerSec: "Пакеты отправлены/сек",
  PacketsReceivedErrors: "Ошибки получения", PacketsOutboundErrors: "Ошибки отправки",
  PacketsReceivedDiscarded: "Отброшено получено", PacketsOutboundDiscarded: "Отброшено отправлено",
  CurrentBandwidth: "Текущая пропускная (бит/с)", InstalledDisplayDrivers: "Драйверы дисплея",
  SerialNumber: "Серийный номер", PrimaryBIOS: "Основной BIOS",
  IsActivated: "Активирован", IsEnabled: "Включён", IsOwned: "Владелец", SpecVersion: "Версия спецификации",
  ManufacturerId: "ID производителя", ManufacturerVersion: "Версия производителя", DriveLetter: "Буква диска",
  ProtectionStatus: "Статус защиты", ConversionStatus: "Статус шифрования", EncryptionMethod: "Метод",
  result: "Результат", prime_count: "Простых чисел", int_elapsed_ms: "Целочисленный тест (мс)",
  fpu_iterations: "FPU итераций", fpu_elapsed_ms: "FPU (мс)", fpu_accumulator: "FPU аккумулятор",
  elements: "Элементы", bytes_moved_approx: "Перемещено байт", elapsed_ms: "Время (мс)",
  bandwidth_mb_s: "Полоса (МБ/с)", iterations: "Итерации", operations_per_second: "Операций/сек",
  accumulator: "Аккумулятор", levels: "Уровни кэша", size_bytes: "Размер (байт)",
  passes: "Проходы", read_elapsed_ms: "Чтение (мс)", write_elapsed_ms: "Запись (мс)",
  read_mb_s: "Чтение (МБ/с)", write_mb_s: "Запись (МБ/с)", total_iterations: "Всего итераций",
  iterations_per_second: "Итераций/сек", duration_seconds: "Длительность (сек)", threads: "Потоков",
  size_mb: "Размер (МБ)", operations: "Операции", block_bytes: "Блок (байт)",
  read_iops: "Чтение IOPS", write_iops: "Запись IOPS", synchronizations: "Синхронизации",
  vendor_summary: "Производитель", brand_summary: "Сводка", features: "Возможности",
  vendor: "Производитель (CPUID)", brand: "CPU", family: "Семейство", model: "Модель",
  stepping: "Stepping", sse: "SSE", sse2: "SSE2", htt: "HTT", avx: "AVX", aes: "AES", rdrand: "RDRAND",
  UEFISecureBootEnabled: "Secure Boot", MaxReadable: "Макс. читаемое", MinReadable: "Мин. читаемое",
  CurrentReading: "Текущее значение", CriticalTripPoint: "Критическая точка", PassiveTripPoint: "Пассивная точка",
  CurrentTemperature: "Температура (0.1 K)", Descriptions: "Описания"
};

function prettyKey(k) {
  return String(k)
    .replace(/_/g, " ")
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace(/\s+/g, " ")
    .trim()
    .replace(/^./, c => c.toUpperCase());
}

function labelFor(k) { return KEY_LABELS[k] || LABELS[k] || prettyKey(k); }

function formatScalar(v, key) {
  if (v === null || v === undefined) return "—";
  if (typeof v === "boolean") return v ? "Да" : "Нет";
  if (typeof v === "number") {
    if (!Number.isFinite(v)) return String(v);
    return v.toLocaleString("ru-RU");
  }
  const s = String(v);
  if (s.length > 240) return s.slice(0, 240) + "…";
  return s;
}

function safeStringify(v) {
  try { return JSON.stringify(v, null, 2); } catch (e) { return String(v); }
}

function renderBlock(container, value, depth) {
  depth = depth || 0;
  if (value === null || value === undefined) {
    const s = document.createElement("span"); s.className = "muted"; s.textContent = "—"; container.appendChild(s); return;
  }
  if (typeof value !== "object") {
    const s = document.createElement("span"); s.textContent = formatScalar(value, ""); container.appendChild(s); return;
  }
  if (Array.isArray(value)) {
    if (value.length === 0) {
      const s = document.createElement("span"); s.className = "muted"; s.textContent = "нет данных"; container.appendChild(s); return;
    }
    const info = document.createElement("div"); info.className = "row-count"; info.textContent = "Записей: " + value.length; container.appendChild(info);
    const allPrimitive = value.every(x => x === null || typeof x !== "object");
    if (allPrimitive) {
      const p = document.createElement("div"); p.textContent = value.map(x => formatScalar(x, "")).join(", "); container.appendChild(p); return;
    }
    const cols = Object.keys(value[0] || {}).slice(0, 7);
    const table = document.createElement("table");
    table.className = "kv-table compact";
    const thead = document.createElement("thead");
    const hr = document.createElement("tr");
    cols.forEach(c => {
      const th = document.createElement("th"); th.textContent = labelFor(c); th.title = c; hr.appendChild(th);
    });
    thead.appendChild(hr); table.appendChild(thead);
    const tbody = document.createElement("tbody");
    value.slice(0, 12).forEach(item => {
      const tr = document.createElement("tr");
      cols.forEach(c => {
        const td = document.createElement("td"); td.textContent = formatScalar(item ? item[c] : "", c); tr.appendChild(td);
      });
      tbody.appendChild(tr);
    });
    table.appendChild(tbody); container.appendChild(table);
    if (value.length > 12) {
      const more = document.createElement("div"); more.className = "muted"; more.textContent = "… и ещё " + (value.length - 12); container.appendChild(more);
    }
    const details = document.createElement("details");
    details.className = "raw-details";
    const summary = document.createElement("summary"); summary.textContent = "Исходные данные (raw JSON)"; details.appendChild(summary);
    const pre = document.createElement("pre"); pre.className = "mono small"; pre.textContent = safeStringify(value); details.appendChild(pre);
    container.appendChild(details);
    return;
  }

  const table = document.createElement("table");
  table.className = "kv-table";
  const tbody = document.createElement("tbody");
  Object.keys(value).forEach(k => {
    const tr = document.createElement("tr");
    const th = document.createElement("th");
    th.textContent = labelFor(k);
    th.title = (KEY_LABELS[k] || LABELS[k]) ? "" : k;
    tr.appendChild(th);
    const td = document.createElement("td");
    const v = value[k];
    if (v !== null && typeof v === "object") {
      if (Array.isArray(v)) {
        const badge = document.createElement("span"); badge.className = "badge"; badge.textContent = "массив: " + v.length; td.appendChild(badge);
        if (v.length) {
          const details = document.createElement("details"); details.className = "nested";
          const summary = document.createElement("summary"); summary.textContent = "подробнее"; details.appendChild(summary);
          const wrap = document.createElement("div"); renderBlock(wrap, v, depth + 1); details.appendChild(wrap);
          td.appendChild(details);
        }
      } else {
        const details = document.createElement("details"); details.className = "nested";
        const summary = document.createElement("summary"); summary.textContent = "показать"; details.appendChild(summary);
        const wrap = document.createElement("div"); renderBlock(wrap, v, depth + 1); details.appendChild(wrap);
        td.appendChild(details);
      }
    } else {
      const span = document.createElement("span"); span.textContent = formatScalar(v, k); td.appendChild(span);
    }
    tr.appendChild(td);
    tbody.appendChild(tr);
  });
  table.appendChild(tbody);
  container.appendChild(table);
}

function card(parentId, rawTitle, obj, hint) {
  const parent = document.getElementById(parentId);
  if (!parent) return;
  const c = document.createElement("div");
  c.className = "card";
  const h = document.createElement("h3");
  h.textContent = LABELS[rawTitle] || labelFor(rawTitle);
  c.appendChild(h);
  if (hint) {
    const p = document.createElement("p");
    p.className = "card-desc";
    p.textContent = hint;
    c.appendChild(p);
  }
  const body = document.createElement("div");
  body.className = "card-body";
  renderBlock(body, obj, 0);
  c.appendChild(body);
  parent.appendChild(c);
}

function clearView(id) { const e = document.getElementById(id); if (e) e.innerHTML = ""; }

function addViewNote(id) {
  const view = document.getElementById(id);
  if (!view) return;
  const n = document.createElement("p");
  n.className = "view-note";
  n.textContent = VIEW_NOTES[id.replace("view-", "")] || "";
  view.appendChild(n);
}

function analyzeAlerts(data) {
  try {
    if (data && data.os && data.os.FreePhysicalMemory && data.os.TotalVisibleMemorySize) {
      const used = 1 - (data.os.FreePhysicalMemory / data.os.TotalVisibleMemorySize);
      if (used > 0.9) pushAlert("Память заполнена более чем на 90% (" + (used * 100).toFixed(1) + "%).");
    }
    (data.logical_disks || []).forEach(d => {
      if (d.Size && d.FreeSpace != null) {
        const freePct = d.FreeSpace / d.Size;
        if (freePct < 0.1) pushAlert("Мало места на " + d.DeviceID + ": " + (freePct * 100).toFixed(1) + "% свободно.");
      }
    });
    (data.processors || []).forEach(p => {
      if (p.LoadPercentage != null && p.LoadPercentage > 90) {
        pushAlert("CPU " + (p.Name || "") + " загрузка " + p.LoadPercentage + "%.");
      }
    });
  } catch (_) { /* ignore analysis errors */ }
}

function pushAlert(msg) {
  const box = document.getElementById("alerts");
  if (!box) return;
  const d = document.createElement("div");
  d.textContent = new Date().toLocaleTimeString() + "  " + msg;
  box.prepend(d);
  while (box.children.length > 50) box.removeChild(box.lastChild);
}

async function loadSummary() {
  status("Загрузка данных системы… (может занять 10–20 сек)");
  try {
    const data = await invoke("system_summary");
    state.summary = data;
    analyzeAlerts(data);
    status("Готово");
    renderView(currentView());
  } catch (e) {
    status("Ошибка: " + e);
  }
}

let current = "computer";
function currentView() { return current; }

async function ensureSummary() { if (!state.summary) await loadSummary(); }

async function ensureExtra(stateKey, invoker, domKey) {
  if (state.loaded.has(stateKey)) return state[stateKey];
  try {
    const data = await invoke(invoker);
    state[stateKey] = data;
    state.loaded.add(stateKey);
    return data;
  } catch (e) {
    const val = { error: String(e) };
    state[stateKey] = val;
    state.loaded.add(stateKey);
    return val;
  }
}

async function renderView(name) {
  current = name;
  document.querySelectorAll(".view").forEach(v => v.classList.remove("active"));
  document.querySelectorAll(".tree-item").forEach(b => b.classList.remove("active"));
  const viewEl = document.getElementById("view-" + name);
  if (viewEl) viewEl.classList.add("active");
  const nav = document.querySelector(`.tree-item[data-view="${name}"]`);
  if (nav) nav.classList.add("active");
  const titleEl = document.getElementById("view-title");
  if (titleEl) titleEl.textContent = VIEW_TITLES[name] || name;

  switch (name) {
    case "computer": {
      clearView("view-computer");
      addViewNote("view-computer");
      if (!state.summary) { await loadSummary(); return; }
      ["os", "computersystem", "computersystemproduct", "system_enclosure", "processes_info", "services"].forEach(k => {
        if (state.summary[k] != null) card("view-computer", k, state.summary[k], k === "processes_info" ? "Сводка по запущенным процессам." : (k === "services" ? "Службы Windows и их состояние." : ""));
      });
      break;
    }
    case "cpu": {
      clearView("view-cpu");
      addViewNote("view-cpu");
      if (!state.summary) { await loadSummary(); return; }
      if (state.summary.processors != null) card("view-cpu", "processors", state.summary.processors, "Основные характеристики процессора.");
      const cp = await ensureExtra("cpuPerf", "cpu_perf", "cpuPerf");
      card("view-cpu", "cpu_perf", cp, "Текущая загрузка и частоты.");
      const ci = await ensureExtra("cpuid", "cpuid_info", "cpuid");
      card("view-cpu", "cpuid", ci, "Идентификация и поддерживаемые наборы инструкций.");
      break;
    }
    case "memory": {
      clearView("view-memory");
      addViewNote("view-memory");
      if (!state.summary) { await loadSummary(); return; }
      if (state.summary.memory_modules != null) card("view-memory", "memory_modules", state.summary.memory_modules, "Установленные модули ОЗУ.");
      const mp = await ensureExtra("memPerf", "memory_perf", "memPerf");
      card("view-memory", "memory_perf", mp, "Использование, файл подкачки, кэш.");
      break;
    }
    case "board": {
      clearView("view-board");
      addViewNote("view-board");
      if (!state.summary) { await loadSummary(); return; }
      const boardHints = {
        motherboard: "Материнская плата и её firmware.",
        bios: "Версия BIOS/UEFI и дата релиза.",
        system_enclosure: "Корпус, серийный номер, инв. номер.",
        computersystemproduct: "Продукт/OEM-система и UUID."
      };
      ["motherboard", "bios", "system_enclosure", "computersystemproduct"].forEach(k => {
        if (state.summary[k] != null) card("view-board", k, state.summary[k], boardHints[k] || "");
      });
      break;
    }
    case "display": {
      clearView("view-display");
      addViewNote("view-display");
      if (!state.summary) { await loadSummary(); return; }
      if (state.summary.gpus != null) card("view-display", "gpus", state.summary.gpus, "Графические адаптеры и драйверы.");
      if (state.summary.monitors != null) card("view-display", "monitors", state.summary.monitors, "Мониторы (WMI).");
      if (state.summary.monitor_params != null) card("view-display", "monitor_params", state.summary.monitor_params, "Размер и входы.");
      if (state.summary.monitor_brightness != null) card("view-display", "monitor_brightness", state.summary.monitor_brightness, "Уровни яркости.");
      const dd = await ensureExtra("display", "display_details", "display");
      card("view-display", "display_details", dd, "Расшифровка названия/серийного номера.");
      break;
    }
    case "storage": {
      clearView("view-storage");
      addViewNote("view-storage");
      if (!state.summary) { await loadSummary(); return; }
      const storageHints = {
        disks: "Физические накопители и интерфейсы.",
        logical_disks: "Разделы, тома и свободное место.",
        smart_status: "Прогноз SMART (где поддерживается).",
        cdroms: "Оптические приводы."
      };
      ["disks", "logical_disks", "smart_status", "cdroms"].forEach(k => {
        if (state.summary[k] != null) card("view-storage", k, state.summary[k], storageHints[k] || "");
      });
      const wrap = document.createElement("div");
      wrap.className = "action-row";
      const btn = document.createElement("button");
      btn.textContent = "Случайные IOPS (64 МБ, 20 000 операций)";
      btn.onclick = async () => {
        btn.disabled = true;
        btn.textContent = "Идёт измерение…";
        const out = document.createElement("pre");
        out.className = "mono small";
        out.textContent = "запуск...";
        document.getElementById("view-storage").appendChild(out);
        try {
          const d = await invoke("random_disk_benchmark");
          out.textContent = safeStringify(d);
        } catch (e) { out.textContent = "ошибка: " + e; }
        finally {
          btn.disabled = false;
          btn.textContent = "Случайные IOPS (64 МБ, 20 000 операций)";
        }
      };
      wrap.appendChild(btn);
      document.getElementById("view-storage").appendChild(wrap);
      break;
    }
    case "network": {
      clearView("view-network");
      addViewNote("view-network");
      if (!state.summary) { await loadSummary(); return; }
      if (state.summary.network != null) card("view-network", "network", state.summary.network, "IP-конфигурация активных адаптеров.");
      if (state.summary.network_adapters != null) card("view-network", "network_adapters", state.summary.network_adapters, "Сетевые адаптеры.");
      const np = await ensureExtra("netPerf", "network_perf", "netPerf");
      card("view-network", "network_perf", np, "Трафик и ошибки.");
      break;
    }
    case "devices": {
      clearView("view-devices");
      addViewNote("view-devices");
      if (!state.summary) { await loadSummary(); return; }
      const deviceHints = {
        audio: "Аудиоустройства и драйверы.", printers: "Принтеры и порты.", usb_controllers: "USB-контроллеры.",
        usb_devices: "Подключённые USB-устройства.", keyboards: "Клавиатуры.", pointing_devices: "Мыши и указатели."
      };
      ["audio", "printers", "usb_controllers", "usb_devices", "keyboards", "pointing_devices"].forEach(k => {
        if (state.summary[k] != null) card("view-devices", k, state.summary[k], deviceHints[k] || "");
      });
      break;
    }
    case "sensors": {
      clearView("view-sensors");
      addViewNote("view-sensors");
      if (!state.summary) { await loadSummary(); return; }
      if (state.summary.batteries != null) card("view-sensors", "batteries", state.summary.batteries, "Батареи питания.");
      if (state.summary.portable_batteries != null) card("view-sensors", "portable_batteries", state.summary.portable_batteries, "Портативные батареи.");
      if (state.summary.monitors != null) card("view-sensors", "monitors", state.summary.monitors, "Мониторы.");
      if (state.summary.monitor_brightness != null) card("view-sensors", "monitor_brightness", state.summary.monitor_brightness, "Яркость.");
      try {
        const t = await invoke("thermal_zones");
        card("view-sensors", "thermal_zones_data", t, "Температурные зоны ACPI и датчики.");
      } catch (e) {
        pushAlert("thermal_zones: " + e);
      }
      break;
    }
    case "software": {
      clearView("view-software");
      addViewNote("view-software");
      if (!state.summary) { await loadSummary(); return; }
      const softwareHints = {
        drivers: "Системные драйверы Windows.", hotfix: "Установленные обновления.", processes_info: "Сводка по процессам.",
        services: "Службы и их состояние."
      };
      ["drivers", "hotfix", "processes_info", "services"].forEach(k => {
        if (state.summary[k] != null) card("view-software", k, state.summary[k], softwareHints[k] || "");
      });
      const sw = await ensureExtra("software", "installed_software", "software");
      card("view-software", "installed_software", sw, "Установленные приложения из реестра Uninstall.");
      const ev = await ensureExtra("eventLog", "event_log", "eventLog");
      card("view-software", "event_log", ev, "Системные события уровня предупреждение/ошибка за 24 ч.");
      const sp = await ensureExtra("security", "security_posture", "security");
      card("view-software", "security_posture", sp, "Secure Boot, TPM, BitLocker.");
      break;
    }
    case "benchmark": {
      const host = document.getElementById("view-benchmark");
      if (host) host.innerHTML = "";
      addViewNote("view-benchmark");
      const panel = document.createElement("div");
      panel.className = "panel";
      panel.innerHTML = `
        <div class="toolbar">
          <button id="btn-cpu-b">CPU: решето</button>
          <button id="btn-fpu-b">FPU</button>
          <button id="btn-cache-b">Кэш</button>
          <button id="btn-mem-b">Память</button>
          <button id="btn-disk-b">Диск 64 МБ</button>
          <button id="btn-randdisk-b">Случ. IOPS</button>
          <label>стресс, сек <input id="stress-seconds" type="number" min="2" max="30" value="5" style="width:60px"></label>
          <button id="btn-stress-b">CPU стресс</button>
        </div>
        <pre id="bench-out" class="mono small"></pre>`;
      if (host) host.appendChild(panel);
      wireBenchButtons();
      break;
    }
    case "report": {
      break;
    }
  }
}

document.querySelectorAll(".tree-item").forEach(b => b.addEventListener("click", () => showView(b.dataset.view)));
function showView(name) { renderView(name); }

document.getElementById("btn-refresh").addEventListener("click", async () => {
  status("Обновление…");
  state.summary = null;
  state.cpuPerf = state.memPerf = state.netPerf = state.display = state.software = state.eventLog = state.security = state.cpuid = null;
  state.loaded.clear();
  await loadSummary();
  renderView(currentView());
});

const autoChk = document.getElementById("auto-refresh");
let autoTimer = null;
if (autoChk) autoChk.addEventListener("change", () => {
  if (autoChk.checked) {
    autoTimer = setInterval(async () => {
      state.summary = null;
      state.loaded.delete("summary");
      await loadSummary();
    }, 30000);
  } else if (autoTimer) { clearInterval(autoTimer); autoTimer = null; }
});

function wireBenchButtons() {
  const out = document.getElementById("bench-out");
  async function run(kind) {
    if (!out) return;
    out.textContent = "запуск " + kind + "…";
    const map = {
      cpu: "run_cpu_benchmark", fpu: "run_fpu_benchmark", cache: "run_cache_benchmark",
      mem: "run_memory_benchmark", disk: "run_disk_benchmark", randdisk: "random_disk_benchmark"
    };
    try {
      const d = kind === "stress"
        ? await invoke("run_stress_test", { seconds: Number((document.getElementById("stress-seconds") || {}).value || 5) })
        : await invoke(map[kind]);
      out.textContent = safeStringify(d);
    } catch (e) { out.textContent = "ошибка: " + e; }
  }
  const binds = {
    "btn-cpu-b": "cpu", "btn-fpu-b": "fpu", "btn-cache-b": "cache", "btn-mem-b": "mem",
    "btn-disk-b": "disk", "btn-randdisk-b": "randdisk"
  };
  Object.keys(binds).forEach(id => {
    const el = document.getElementById(id);
    if (el) el.onclick = () => run(binds[id]);
  });
  const sb = document.getElementById("btn-stress-b");
  if (sb) sb.onclick = () => run("stress");
}

document.getElementById("btn-genreport").addEventListener("click", async () => {
  try {
    const data = await invoke("system_summary");
    let th = {};
    try { th = await invoke("thermal_zones"); } catch (_) { th = {}; }
    const report = { generated_at: new Date().toISOString(), system: data, thermals: th };
    const el = document.getElementById("report-text");
    if (el) el.value = safeStringify(report);
    window.__lastReport = report;
    status("Отчёт сформирован");
  } catch (e) {
    status("ошибка: " + e);
  }
});

function toCsv(data) {
  const rows = [["section", "key", "value"]];
  function walk(section, obj, prefix) {
    prefix = prefix || "";
    if (obj === null || obj === undefined) return;
    if (Array.isArray(obj)) obj.forEach((it, idx) => walk(section, it, prefix ? prefix + "[" + idx + "]" : "[" + idx + "]"));
    else if (typeof obj === "object") Object.keys(obj).forEach(k => walk(section, obj[k], prefix ? prefix + "." + k : k));
    else rows.push([section, prefix, String(obj)]);
  }
  Object.keys(data).forEach(k => walk(k, data[k]));
  return rows.map(r => r.map(v => "\"" + String(v).replace(/"/g, "\"\"") + "\"").join(",")).join("\n");
}

function toHtml(data) {
  return "<!DOCTYPE html><html><head><meta charset=\"UTF-8\"><title>AIDA67 report</title></head><body><h1>AIDA67 report</h1><pre>" +
    safeStringify(data).replace(/</g, "&lt;") + "</pre></body></html>";
}

document.getElementById("btn-export").addEventListener("click", async () => {
  const fmt = document.getElementById("export-format").value;
  const pathBase = document.getElementById("export-path").value.trim();
  if (!pathBase) { status("Укажите путь/базовое имя"); return; }
  const content = document.getElementById("report-text").value;
  if (!content) { status("Сначала сформируйте отчёт"); return; }
  let data = null;
  try { data = JSON.parse(content); } catch (_) { data = null; }
  let body = content, ext = ".json";
  if (fmt === "csv") { body = data ? toCsv(data.system || data) : content; ext = ".csv"; }
  else if (fmt === "html") { body = data ? toHtml(data) : content; ext = ".html"; }
  else { ext = ".json"; }
  const path = pathBase.endsWith(ext) ? pathBase : pathBase + ext;
  try {
    const r = await invoke("export_report", { path: path, content: body });
    const st = document.getElementById("export-status");
    if (st) st.textContent = r;
    status("Экспорт выполнен");
  } catch (e) {
    const st = document.getElementById("export-status");
    if (st) st.textContent = "ошибка: " + e;
    status("Ошибка экспорта");
  }
});

(async function init() {
  wireBenchButtons();
  await loadSummary();
  renderView("computer");
})();
