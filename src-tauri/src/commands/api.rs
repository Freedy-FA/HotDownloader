use tauri::command;
use serde_json::{json, Value};

/// 搜索歌曲，返回 JSON 数组字符串（扩展 SongInfo，增加 mediaMid 和 qualities）
#[command]
pub async fn search_songs(keyword: String, page: u32) -> Result<String, String> {
    let client = reqwest::Client::new();

    let request_body = json!({
        "comm": {
            "ct": 11,
            "cv": "1003006",
            "v": "1003006",
            "os_ver": "12",
            "phonetype": "0",
            "devicelevel": "31",
            "tmeAppID": "qqmusiclight",
            "nettype": "NETWORK_WIFI"
        },
        "req": {
            "module": "music.search.SearchCgiService",
            "method": "DoSearchForQQMusicLite",
            "param": {
                "query": keyword,
                "search_type": 0,
                "num_per_page": 20,   // 默认每页20条，可根据需要调整
                "page_num": page,
                "nqc_flag": 0,
                "grp": 1
            }
        }
    });

    // 发送 POST 请求
    let resp = client
        .post("https://u.y.qq.com/cgi-bin/musicu.fcg")
        .header("User-Agent", "HotDownloader/1.0")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    let text = resp.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
    let data: Value = serde_json::from_str(&text).map_err(|e| format!("解析响应失败: {}", e))?;

    // 检查整体状态
    if data["code"] != 0 {
        return Err(format!("接口错误: code={}", data["code"]));
    }
    let req = &data["req"];
    if req["code"] != 0 {
        return Err(format!("搜索错误: req.code={}", req["code"]));
    }

    // 提取歌曲列表
    let item_song = req["data"]["body"]["item_song"]
        .as_array()
        .ok_or("未找到歌曲列表")?;

    let mut songs = Vec::new();
    for item in item_song {
        let file = &item["file"];
        // 必须有 media_mid 才能下载，否则跳过
        let media_mid = file["media_mid"].as_str().unwrap_or("");
        if media_mid.is_empty() {
            continue;
        }

        // 歌曲唯一标识（使用 mid）
        let mid = item["mid"].as_str().unwrap_or("").to_string();

        // 歌名拼接附加信息
        let name = item["name"].as_str().unwrap_or("");
        let title_extra = item["title_extra"].as_str().unwrap_or("");
        // 避免 if 表达式类型推断问题，改用可变变量赋值
        let mut title = name.to_string();
        if !title_extra.is_empty() {
            title = format!("{}{}", name, title_extra);
        }

        // 歌手列表，用逗号连接
        let singers: Vec<String> = item["singer"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| s["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let artist = singers.join(", ");

        // 专辑名
        let album_name = item["album"]["name"].as_str().unwrap_or("").to_string();

        // 专辑 mid 与第一个歌手 mid，用于构建封面
        let album_mid = item["album"]["mid"].as_str().unwrap_or("");
        let first_singer_mid = item["singer"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|s| s["mid"].as_str())
            .unwrap_or("");

        // 封面URL，同样用可变变量避免 if 表达式问题
        let mut cover_url = String::new();
        if !album_mid.is_empty() && album_mid != "空" {
            cover_url = format!(
                "https://y.gtimg.cn/music/photo_new/T002R500x500M000{}.jpg",
                album_mid
            );
        } else if !first_singer_mid.is_empty() {
            cover_url = format!(
                "https://y.gtimg.cn/music/photo_new/T001R500x500M000{}.jpg",
                first_singer_mid
            );
        }

        // 构建品质列表（与前端 Quality 类型对应）
        let qualities = build_qualities(file, &item["vs"]);

        songs.push(json!({
            "id": mid,
            "title": title,
            "artist": artist,
            "album": album_name,
            "coverUrl": cover_url,
            "mediaMid": media_mid,
            "qualities": qualities
        }));
    }

    Ok(serde_json::to_string(&songs).map_err(|e| format!("序列化结果失败: {}", e))?)
}

/// 根据 file 和 vs 生成可用品质列表
fn build_qualities(file: &Value, vs: &Value) -> Vec<Value> {
    let media_mid = file["media_mid"].as_str().unwrap_or("");
    let mut list = Vec::new();

    // 标准品质映射 (品质标签, 前缀, 后缀)
    let standard = [
        ("试听", "RS02", ".mp3"),
        ("48k", "C200", ".m4a"),
        ("96k", "C400", ".m4a"),
        ("192k", "C600", ".m4a"),
        ("96k_ogg", "O4M0", ".mgg"),
        ("192k_ogg", "O6M0", ".mgg"),
        ("128k", "M500", ".mp3"),
        ("320k", "M800", ".mp3"),
        ("APE", "A000", ".ape"),
        ("FLAC", "F0M0", ".mflac"),
        ("Hi-Res", "RSM1", ".mflac"),
    ];

    for (label, prefix, suffix) in &standard {
        // 用 size_ 字段判断是否存在
        let size_key = format!("size_{}", label.to_lowercase().replace("-", "").replace(' ', ""));
        let size_key = if size_key == "size_320k" { "size_320mp3".to_string() }
                      else if size_key == "size_128k" { "size_128mp3".to_string() }
                      else if size_key == "size_ape" { "size_ape".to_string() }
                      else if size_key == "size_flac" { "size_flac".to_string() }
                      else if size_key == "size_hi-res" { "size_hires".to_string() }
                      else if size_key == "size_48k" { "size_48aac".to_string() }
                      else if size_key == "size_96k" { "size_96aac".to_string() }
                      else if size_key == "size_192k" { "size_192aac".to_string() }
                      else { size_key };

        let size = file[&size_key].as_u64().unwrap_or(0);
        if size > 0 {
            list.push(json!({
                "quality": label,
                "filename": format!("{}{}{}", prefix, media_mid, suffix)
            }));
        }
    }

    // 特殊品质：杜比全景声 / 臻品全景声 / 臻品母带
    let size_new = file["size_new"].as_array();
    let vs_arr = vs.as_array();
    if let (Some(size_new), Some(vs_arr)) = (size_new, vs_arr) {
        let vs3 = vs_arr.get(3).and_then(|v| v.as_str()).unwrap_or("");
        let vs4 = vs_arr.get(4).and_then(|v| v.as_str()).unwrap_or("");

        // 臻品母带 (size_new[0] + vs[3])
        if size_new.get(0).and_then(|v| v.as_u64()).unwrap_or(0) > 0 && !vs3.is_empty() {
            list.push(json!({
                "quality": "臻品母带",
                "filename": format!("AIM0{}.mflac", vs3)
            }));
        }
        // 杜比全景声 (size_new[1] + vs[4])
        if size_new.get(1).and_then(|v| v.as_u64()).unwrap_or(0) > 0 && !vs4.is_empty() {
            list.push(json!({
                "quality": "杜比全景声",
                "filename": format!("Q0M0{}.mflac", vs4)
            }));
        }
        // 臻品全景声 (size_new[2] + vs[4])
        if size_new.get(2).and_then(|v| v.as_u64()).unwrap_or(0) > 0 && !vs4.is_empty() {
            list.push(json!({
                "quality": "臻品全景声",
                "filename": format!("Q0M1{}.mflac", vs4)
            }));
        }
    }

    list
}

/// 获取下载链接与解密密钥
/// 参数：song_id 为歌曲 mid，filename 为品质文件名（如 M800001abc.mp3）
/// 返回 JSON: { "url": "完整下载链接", "key": "ekey" }
#[command]
pub async fn fetch_download_link(song_id: String, filename: String) -> Result<String, String> {
    let client = reqwest::Client::new();

    let request_body = json!({
        "comm": {
            "ct": "19",
            "cv": "2111"
        },
        "music.vkey.GetEVkey.CgiGetHotVkey": {
            "module": "music.vkey.GetEVkey",
            "method": "CgiGetHotVkey",
            "param": {
                "filename": [&filename],
                "songmid": [&song_id]
            }
        },
        "music.vkey.GetEVkey.GetEkey": {
            "module": "music.vkey.GetEVkey",
            "method": "GetEkey",
            "param": {
                "finfo": [
                    {
                        "filename": &filename,
                        "mid": &song_id
                    }
                ]
            }
        }
    });

    let resp = client
        .post("https://ut.y.qq.com/cgi-bin/musicu.fcg")
        .header("Content-Type", "application/json")
        .header("User-Agent", "HotDownloader/1.0")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    let text = resp.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
    let data: Value = serde_json::from_str(&text).map_err(|e| format!("解析响应失败: {}", e))?;

    // 提取 purl
    let vkey_resp = &data["music.vkey.GetEVkey.CgiGetHotVkey"];
    let urls = vkey_resp["data"]["urls"].as_array().ok_or("缺少 urls")?;
    let purl = urls.get(0)
        .and_then(|u| u["purl"].as_str())
        .ok_or("未获取到下载链接")?;

    // 提取 ekey
    let ekey_resp = &data["music.vkey.GetEVkey.GetEkey"];
    let ekeyinfo = ekey_resp["data"]["ekeyinfo"].as_array().ok_or("缺少 ekeyinfo")?;
    let ekey = ekeyinfo.get(0)
        .and_then(|e| e["ekey"].as_str())
        .unwrap_or("");

    // 拼接完整下载 URL（使用主 CDN）
    let full_url = format!("https://wx.music.tc.qq.com/{}", purl);

    let result = json!({
        "url": full_url,
        "key": ekey
    });

    Ok(result.to_string())
}

/// 获取热搜关键词列表
#[command]
pub async fn fetch_hot_keywords() -> Result<String, String> {
    let client = reqwest::Client::new();

    let request_body = json!({
        "comm": {
            "ct": "19",
            "cv": "1803",
            "guid": "0",
            "patch": "118",
            "psrf_access_token_expiresAt": 0,
            "psrf_qqaccess_token": "",
            "psrf_qqopenid": "",
            "psrf_qqunionid": "",
            "tmeAppID": "qqmusic",
            "tmeLoginType": 0,
            "uin": "0",
            "wid": "0"
        },
        "hotkey": {
            "module": "tencent_musicsoso_hotkey.HotkeyService",
            "method": "GetHotkeyForQQMusicPC",
            "param": {
                "search_id": "",
                "uin": 0
            }
        }
    });

    let resp = client
        .post("https://u.y.qq.com/cgi-bin/musicu.fcg")
        .header("User-Agent", "HotDownloader/1.0")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    let text = resp.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
    let data: Value = serde_json::from_str(&text).map_err(|e| format!("解析响应失败: {}", e))?;

    // 热搜数据在独立的 "hotkey" 字段中
    let hotkey = &data["hotkey"];
    if hotkey.is_null() {
        return Err("热搜数据缺失".into());
    }
    let code = hotkey["code"].as_i64().unwrap_or(-1);
    if code != 0 {
        return Err(format!("热搜接口错误: code={}", code));
    }

    let vec_hotkey = hotkey["data"]["vec_hotkey"]
        .as_array()
        .ok_or("未找到热搜列表")?;

    let mut keywords = Vec::new();
    for item in vec_hotkey.iter().take(30) {
        if let Some(q) = item["query"].as_str() {
            if !q.is_empty() {
                keywords.push(q.to_string());
            }
        }
    }

    Ok(serde_json::to_string(&keywords).map_err(|e| format!("序列化结果失败: {}", e))?)
}