use tauri::command;

/// 搜索歌曲，返回 JSON 数组字符串
#[command]
pub async fn search_songs(keyword: String, page: u32) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.example.com/search?keyword={}&page={}",
        keyword, page
    );
    let resp = client
        .get(&url)
        .header("User-Agent", "HotDownloader/1.0")
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;
    let text = resp.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
    Ok(text)
}

/// 获取下载链接和密钥，返回 JSON 对象字符串
#[command]
pub async fn fetch_download_link(song_id: String, quality: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.example.com/download?song_id={}&quality={}",
        song_id, quality
    );
    let resp = client
        .get(&url)
        .header("User-Agent", "HotDownloader/1.0")
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;
    let text = resp.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
    Ok(text)
}