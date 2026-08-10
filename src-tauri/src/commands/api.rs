use serde_json::{json, Value};
use tauri::command;
use url::Url;

/// 搜索歌曲，返回 JSON 数组字符串（扩展 SongInfo，增加 mediaMid 和 qualities）
/// https://github.com/lyswhut/lx-music-desktop/blob/9c364b482e5621a1d38b50e8610d2fb974457e6e/src/renderer/utils/musicSdk/tx/musicSearch.js#L13
#[command]
pub async fn search_songs(keyword: String, page: u32, limit: u32) -> Result<String, String> {
    let client = reqwest::Client::new();

    let searchid = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();

    let request_body = json!({
        "comm": {
            "ct": "11",
            "cv": "14090508",
            "v": "14090508",
            "tmeAppID": "qqmusic",
            "phonetype": "EBG-AN10",
            "deviceScore": "553.47",
            "devicelevel": "50",
            "newdevicelevel": "20",
            "rom": "HuaWei/EMOTION/EmotionUI_14.2.0",
            "os_ver": "12",
            "OpenUDID": "0",
            "OpenUDID2": "0",
            "QIMEI36": "0",
            "udid": "0",
            "chid": "0",
            "aid": "0",
            "oaid": "0",
            "taid": "0",
            "tid": "0",
            "wid": "0",
            "uid": "0",
            "sid": "0",
            "modeSwitch": "6",
            "teenMode": "0",
            "ui_mode": "2",
            "nettype": "1020",
            "v4ip": ""
        },
        "req": {
            "module": "music.search.SearchCgiService",
            "method": "DoSearchForQQMusicMobile",
            "param": {
                "search_type": 0,
                "searchid": searchid,
                "query": keyword,
                "page_num": page,
                "num_per_page": limit,   // 使用参数控制每页数量
                "highlight": 0,
                "nqc_flag": 0,
                "multi_zhida": 0,
                "cat": 2,
                "grp": 1,
                "sin": 0,
                "sem": 0
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

    let text = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;
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

    // 标准品质，按顺序定义 (前端标签, 文件前缀, 后缀, 文件大小字段名)
    let standard_qualities: Vec<(&str, &str, &str, &str)> = vec![
        ("48kacc",   "C200", ".m4a",   "size_48aac"),
        ("96kacc",   "C400", ".m4a",   "size_96aac"),
        ("192kacc",  "C600", ".m4a",   "size_192aac"),
        ("96kogg",   "O4M0", ".mgg",   "size_96ogg"),
        ("192kogg",  "O6M0", ".mgg",   "size_192ogg"),
        ("128kmp3",  "M500", ".mp3",   "size_128mp3"),
        ("320kmp3",  "M800", ".mp3",   "size_320mp3"),
        ("ape",      "A000", ".ape",   "size_ape"),
        ("flac",     "F0M0", ".mflac", "size_flac"),
        ("hires",    "RSM1", ".mflac", "size_hires"),
    ];

    for (label, prefix, suffix, size_key) in &standard_qualities {
        let size = file[*size_key].as_u64().unwrap_or(0);
        if size > 0 {
            list.push(json!({
                "quality": label,
                "filename": format!("{}{}{}", prefix, media_mid, suffix),
                "size": size
            }));
        }
    }

    // 特殊品质：杜比全景声 → 臻品全景声 → 臻品母带（按此顺序）
    let size_new = file["size_new"].as_array();
    let vs_arr = vs.as_array();
    if let (Some(size_new), Some(vs_arr)) = (size_new, vs_arr) {
        let vs3 = vs_arr.get(3).and_then(|v| v.as_str()).unwrap_or("");
        let vs4 = vs_arr.get(4).and_then(|v| v.as_str()).unwrap_or("");

        // 杜比全景声 (size_new[1] + vs[4])
        let size_dolby = size_new.get(1).and_then(|v| v.as_u64()).unwrap_or(0);
        if size_dolby > 0 && !vs4.is_empty() {
            list.push(json!({
                "quality": "杜比全景声",
                "filename": format!("Q0M0{}.mflac", vs4),
                "size": size_dolby
            }));
        }

        // 臻品全景声 (size_new[2] + vs[4])
        let size_panorama = size_new.get(2).and_then(|v| v.as_u64()).unwrap_or(0);
        if size_panorama > 0 && !vs4.is_empty() {
            list.push(json!({
                "quality": "臻品全景声",
                "filename": format!("Q0M1{}.mflac", vs4),
                "size": size_panorama
            }));
        }

        // 臻品母带 (size_new[0] + vs[3])
        let size_master = size_new.get(0).and_then(|v| v.as_u64()).unwrap_or(0);
        if size_master > 0 && !vs3.is_empty() {
            list.push(json!({
                "quality": "臻品母带",
                "filename": format!("AIM0{}.mflac", vs3),
                "size": size_master
            }));
        }
    }

    list
}

/// 获取下载链接与解密密钥
/// 参数：song_id 为歌曲 mid，filename 为品质文件名（如 M800001abc.mp3）
/// 返回 JSON: { "url": "完整下载链接", "key": "ekey" }
/// https://github.com/chrisdong/FileHub/blob/e1d752e1f29f877b7c895ae5aaff32a179fad051/root/importURLs/lxmusic/HeiMusic%E8%81%9A%E5%90%88%E6%BA%90_v1.1.5.js#L287
/// 核心函数：获取下载链接和密钥，供下载模块调用
pub(crate) async fn get_download_link(
    song_id: &str,
    filename: &str,
) -> Result<(String, String), String> {
    let client = reqwest::Client::new();

    let request_body = json!({
        "comm": {
            "ct": "19",
            "cv": "0",
            "guid": "",
            "tmeAppID": "qqmusic",
            "qq": "0"
        },
        "music.vkey.GetEVkey.CgiGetHotVkey": {
            "module": "music.vkey.GetEVkey",
            "method": "CgiGetHotVkey",
            "param": {
                "filename": [filename],
                "songmid": [song_id]
            }
        },
        "music.vkey.GetEVkey.GetEkey": {
            "module": "music.vkey.GetEVkey",
            "method": "GetEkey",
            "param": {
                "finfo": [
                    {
                        "filename": filename,
                        "mid": song_id
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

    let text = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;
    let data: Value = serde_json::from_str(&text).map_err(|e| format!("解析响应失败: {}", e))?;

    // 提取 purl
    let vkey_resp = &data["music.vkey.GetEVkey.CgiGetHotVkey"];
    let urls = vkey_resp["data"]["urls"].as_array().ok_or("缺少 urls")?;
    let purl = urls
        .get(0)
        .and_then(|u| u["purl"].as_str())
        .ok_or("未获取到下载链接")?;

    // 提取 ekey
    let ekey_resp = &data["music.vkey.GetEVkey.GetEkey"];
    let ekeyinfo = ekey_resp["data"]["ekeyinfo"]
        .as_array()
        .ok_or("缺少 ekeyinfo")?;
    let ekey = ekeyinfo
        .get(0)
        .and_then(|e| e["ekey"].as_str())
        .unwrap_or("");

    // 拼接完整下载 URL（使用主 CDN）
    let full_url = format!("https://wx.music.tc.qq.com/{}", purl);
    Ok((full_url, ekey.to_string()))
}

#[command]
pub async fn fetch_download_link(song_id: String, filename: String) -> Result<String, String> {
    let (url, key) = get_download_link(&song_id, &filename).await?;
    let result = json!({ "url": url, "key": key });
    Ok(result.to_string())
}

/// 获取热搜关键词列表，返回 JSON 数组字符串
/// https://github.com/lyswhut/lx-music-desktop/blob/9c364b482e5621a1d38b50e8610d2fb974457e6e/src/renderer/utils/musicSdk/tx/hotSearch.js#L15
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
        .header("Referer", "https://y.qq.com/portal/player.html")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    let text = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;
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

/// 获取搜索建议
/// https://github.com/lyswhut/lx-music-desktop/blob/9c364b482e5621a1d38b50e8610d2fb974457e6e/src/renderer/utils/musicSdk/tx/tipSearch.js#L10
#[command]
pub async fn fetch_suggestions(keyword: String) -> Result<String, String> {
    let client = reqwest::Client::new();

    // 构建 URL，并进行 URL 编码
    let base_url = "http://c.y.qq.com/splcloud/fcgi-bin/smartbox_new.fcg";
    let url = Url::parse_with_params(
        base_url,
        &[
            ("is_xml", "0"),
            ("format", "json"),
            ("key", &keyword),
            ("loginUin", "0"),
            ("hostUin", "0"),
            ("inCharset", "utf8"),
            ("outCharset", "utf-8"),
            ("notice", "0"),
            ("platform", "yqq"),
            ("needNewCode", "0"),
        ],
    )
    .map_err(|e| format!("URL 构建失败: {}", e))?;

    let resp = client
        .get(url)
        .header("Referer", "https://y.qq.com/portal/player.html")
        .header("Accept", "*/*")
        .header("Host", "c.y.qq.com")
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    let text = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    let data: Value = serde_json::from_str(&text).map_err(|e| format!("解析响应失败: {}", e))?;

    // 检查状态码
    let code = data["code"].as_i64().unwrap_or(-1);
    let subcode = data["subcode"].as_i64().unwrap_or(-1);
    if code != 0 || subcode != 0 {
        return Err(format!("接口错误: code={}, subcode={}", code, subcode));
    }

    let root_data = data["data"].as_object().ok_or("缺少 data 字段")?;

    // 定义需要提取的类型列表及其对应的字段名
    let types = vec![
        ("song", "单曲"),
        ("singer", "歌手"),
        ("album", "专辑"),
        ("mv", "MV"),
    ];

    let mut result = serde_json::Map::new();

    for (type_key, _type_name) in types {
        let mut items = Vec::new();

        if let Some(obj) = root_data.get(type_key).and_then(|v| v.as_object()) {
            if let Some(itemlist) = obj.get("itemlist").and_then(|v| v.as_array()) {
                for item in itemlist {
                    let mut map = serde_json::Map::new();
                    // 通用字段
                    if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                        map.insert("id".to_string(), json!(id));
                    }
                    if let Some(mid) = item.get("mid").and_then(|v| v.as_str()) {
                        map.insert("mid".to_string(), json!(mid));
                    }
                    if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                        map.insert("name".to_string(), json!(name));
                    }
                    if let Some(singer) = item.get("singer").and_then(|v| v.as_str()) {
                        map.insert("singer".to_string(), json!(singer));
                    }
                    // 封面图片（歌手、专辑可能有，单曲通常没有）
                    if let Some(pic) = item.get("pic").and_then(|v| v.as_str()) {
                        map.insert("cover".to_string(), json!(pic));
                    } else {
                        map.insert("cover".to_string(), json!(null));
                    }
                    // MV 特有字段 vid
                    if type_key == "mv" {
                        if let Some(vid) = item.get("vid").and_then(|v| v.as_str()) {
                            map.insert("vid".to_string(), json!(vid));
                        }
                    }
                    items.push(Value::Object(map));
                }
            }
        }
        result.insert(type_key.to_string(), json!(items));
    }

    Ok(serde_json::to_string(&Value::Object(result))
        .map_err(|e| format!("序列化结果失败: {}", e))?)
}
