use std::fs;
use std::path::Path;
use std::process::Command;
use tauri::command;

#[cfg(target_os = "linux")]
use open;


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
        // 获取父目录
        let parent = file_path.parent().ok_or("无法获取父目录")?;

        // 尝试用 xdg-open 打开父目录（直接传 Path）
        if Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }

        // 备选：使用 dbus-send 选中文件（部分文件管理器支持）
        if Command::new("dbus-send")
            .args([
                "--session",
                "--print-reply",
                "--dest=org.freedesktop.FileManager1",
                "/org/freedesktop/FileManager1",
                "org.freedesktop.FileManager1.ShowItems",
                &format!("array:string:{}", path), // path 是 String 类型
            ])
            .spawn()
            .is_ok()
        {
            return Ok(());
        }

        // 最后回退：用 open 打开父目录（不选中文件）
        open::that(parent).map_err(|e| e.to_string())?;
    }

    Ok(())
}
