use std::fs;
use std::path::Path;
use std::process::Command;
use tauri::command;

/// 获取系统默认下载目录路径
#[command]
pub fn get_default_download_dir() -> String {
    dirs::download_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .to_string_lossy()
        .to_string()
}

/// 创建目录（若不存在）
#[command]
pub fn create_directory(path: String) -> Result<(), String> {
    fs::create_dir_all(&path).map_err(|e| e.to_string())
}

/// 打开文件所在目录并选中文件
#[command]
pub fn open_file_location(path: String) -> Result<(), String> {
    let file_path = Path::new(&path);
    if !file_path.exists() {
        return Err("文件不存在".to_string());
    }

    let parent = file_path.parent().unwrap_or(Path::new("."));

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        // 使用 xdg-open 打开文件夹，再尝试 dbus
        if Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
        // 备选：使用 nautilus 等直接选中文件（不通用）
        if Command::new("dbus-send")
            .args(["--session", "--print-reply", "--dest=org.freedesktop.FileManager1",
                   "/org/freedesktop/FileManager1",
                   "org.freedesktop.FileManager1.ShowItems",
                   &format!("array:string:{}", path)])
            .spawn()
            .is_err()
        {
            // 最后尝试打开父目录
            open::that(parent).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}