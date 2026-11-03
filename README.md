# AIDA67

Независимое десктоп-приложение для просмотра системной информации, мониторинга, бенчмарков и отчётов. Стек: **Rust + Tauri 2**. Проект **не** использует реверс-инжиниринг и не содержит кода/ресурсов стороннего проприетарного ПО; данные получаются только через документированные Windows API (WMI/CIM), Registry и собственные алгоритмы.

## Реализованные категории данных (значительная часть функционала системных информаторов этого класса)
- **ОС/система:** ОС, компьютер, продукт, корпус, BIOS, материнская плата, хотфиксы, системные драйверы.
- **CPU:** имя, ядра/логические, частоты, кэши, сокет, архитектура/семейство, виртуализация, нагрузка, per-core performance counters, CPUID/vendor/features.
- **Память:** модули, vendor/part/capacity/speed/form factor/bank/serial/widths; perf (available/commit/pools/paging), pagefiles.
- **Графика/дисплеи:** адаптеры, драйверы, разрешения/частота, `WmiMonitorID`, desktop monitors, brightness, декодирование мониторов.
- **Хранилище:** физические диски, SMART failure prediction, логические тома/free space, CD/DVD, SMART raw classes (через PowerShell при доступности).
- **Сеть:** IP-конфигурация, активные адаптеры/speed, perf counters throughput/errors.
- **Питание:** батареи/портативные батареи.
- **Устройства/периферия:** USB controllers/devices, аудио, принтеры, клавиатуры, указатели.
- **ПО/безопасность:** установленное ПО (Uninstall registry fast path), службы/процессы, event log (24h), security posture (Secure Boot/TPM/BitLocker).

## Датчики/мониторинг
- Термозоны `MSAcpi_ThermalZoneTemperature`, `Win32_TemperatureProbe`.
- Автообновление каждые 5 с, журнал оповещений по памяти/месту/загрузке CPU.

## Бенчмарки/стресс
- CPU sieve, FPU, cache L1/L2/L3, память, диск (sequential 64MB + random IOPS 4K), CPU stress 2–30 сек многопоточно.

## Отчёты/экспорт
- Формирование JSON-отчёта, экспорт JSON/CSV/HTML/TXT/LOG с валидацией расширения/пути/размера.
- CSP включён, capability minimal (`core:default`), мощные unsafe plugin пути исключены.

## Запуск из исходников
```powershell
cd src-tauri
$env:CARGO_NET_OFFLINE="true"
cargo run
```

## Сборка
```powershell
cd src-tauri
$env:CARGO_NET_OFFLINE="true"
cargo build --release
```
Готовый бинарник: `src-tauri/target/release/aida67.exe`.

## Установщик
Самораспаковывающийся `AIDA67-Setup.exe` собирается из `installer/installer.cs` и встроенного `aida67.exe`: он распаковывает приложение в `%LOCALAPPDATA%\Programs\AIDA67`, создаёт ярлык на рабочем столе и запускает его.

## Проверки
```powershell
cd src-tauri
$env:CARGO_NET_OFFLINE="true"
cargo test
cargo clippy --offline --all-targets -- -D warnings
cargo build --release
```

## Важные ограничения
- Низкоуровневые датчики (вольтаж, обороты/PWM, SuperIO/EC, GPU SDK telemetry) требуют отдельного подписчика/драйвера и не включены.
- WMI-классы доступны не на всех системах; endpoint-ы возвращают частичные данные и ошибки вместо падения всего отчёта.
- Таймсерии/история, remote monitoring, HTML/PDF-отчёты с графиками — следующий этап.
- Данные бенчмарков и отчёты предназначены для диагностики, не для сертификации.
