use std::fs;
use std::path::{Component, Path};

const MAX_EXPORT_BYTES: usize = 10 * 1024 * 1024;

pub fn export_report_sync(path: String, content: String) -> Result<String, String> {
    if content.len() > MAX_EXPORT_BYTES {
        return Err(format!("content too large: {} bytes (max {})", content.len(), MAX_EXPORT_BYTES));
    }

    let p = Path::new(path.trim());
    if p.as_os_str().is_empty() {
        return Err("empty export path".to_string());
    }

    // Reject traversal and unsafe/empty path components without canonicalizing,
    // while still allowing a normal absolute or relative destination.
    for component in p.components() {
        match component {
            Component::ParentDir => return Err("path traversal ('..') is not allowed".to_string()),
            Component::CurDir => return Err("current-directory components are not allowed".to_string()),
            Component::Normal(name) if name.to_string_lossy().is_empty() => {
                return Err("empty path component".to_string());
            }
            _ => {}
        }
    }

    let ext = p
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    if !matches!(ext.as_str(), "json" | "csv" | "html" | "txt" | "log") {
        return Err("unsupported export extension (allowed: json, csv, html, txt, log)".to_string());
    }

    if p.exists() && p.is_dir() {
        return Err("export path is a directory".to_string());
    }

    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| format!("create dir: {e}"))?;
        }
    }

    fs::write(p, content.as_bytes()).map_err(|e| format!("write: {e}"))?;
    Ok(format!("written {} bytes to {}", content.len(), p.display()))
}

#[tauri::command]
pub async fn export_report(path: String, content: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || export_report_sync(path, content))
        .await
        .map_err(|e| format!("task join error: {e}"))?
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn rejects_traversal() {
        let r = export_report_sync("../evil.json".to_string(), "{}".to_string());
        assert!(r.is_err());
    }
    #[test]
    fn rejects_bad_extension() {
        let r = export_report_sync("C:\\tmp\\note.exe".to_string(), "x".to_string());
        assert!(r.is_err());
    }
    #[test]
    fn writes_allowed_file() {
        let path = std::env::temp_dir().join("sysmon_test_export.json");
        let _ = fs::remove_file(&path);
        let r = export_report_sync(path.to_string_lossy().to_string(), "{\"ok\":true}".to_string());
        assert!(r.is_ok());
        let data = fs::read_to_string(&path).unwrap();
        assert_eq!(data, "{\"ok\":true}");
        let _ = fs::remove_file(&path);
    }
}
