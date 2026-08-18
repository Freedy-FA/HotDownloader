//! 网易云音乐备用音源。
//!
//! QQ 音乐对 `.mp3` / `.m4a` / `.ape` 这类非加密格式要求登录态，即便登录后
//! 部分歌曲仍因版权限制返回 104003。网易游客态对大量歌曲可直接返回 320kmp3
//! 直链，作为 QQ 拿不到 320kmp3 时的兜底音源。
//!
//! 仅使用网易明文 HTTP API（`/api/search/get/web` 与
//! `/api/song/enhance/player/url`），无需加密。游客 Cookie `MUSIC_U=00`。

use serde_json::Value;
use url::Url;

use super::api::CLIENT;
use super::source_match;

/// 游客态 Cookie，足够取到大部分非 VIP 歌曲的 320k mp3 直链。
const GUEST_COOKIE: &str = "os=pc; MUSIC_U=00; __remember_me=true";

/// 网易歌曲信息（取链时只需 id 与名称用于日志）。
#[derive(Debug, Clone)]
pub struct NeteaseSong {
    pub id: i64,
    pub name: String,
    pub artists: String,
}

/// 搜索网易歌曲。`keyword` 直接透传给网易搜索接口。
pub(crate) async fn search_songs(keyword: &str) -> Result<Vec<NeteaseSong>, String> {
    let url = Url::parse_with_params(
        "https://music.163.com/api/search/get/web",
        &[
            ("s", keyword),
            ("type", "1"),
            ("offset", "0"),
            ("limit", "20"),
        ],
    )
    .map_err(|e| format!("网易 URL 构建失败: {}", e))?;

    let resp = CLIENT
        .get(url)
        .header("Referer", "https://music.163.com/")
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| format!("网易搜索网络错误: {}", e))?;

    let text = resp
        .text()
        .await
        .map_err(|e| format!("网易搜索读取失败: {}", e))?;
    let data: Value =
        serde_json::from_str(&text).map_err(|e| format!("网易搜索解析失败: {}", e))?;

    if data["code"].as_i64().unwrap_or(-1) != 200 {
        return Err(format!("网易搜索错误: code={}", data["code"]));
    }

    let arr = data["result"]["songs"]
        .as_array()
        .ok_or("网易搜索无歌曲列表")?;
    let mut list = Vec::new();
    for item in arr {
        let id = item["id"].as_i64().unwrap_or(0);
        if id == 0 {
            continue;
        }
        let name = item["name"].as_str().unwrap_or("").to_string();
        let artists: Vec<String> = item["artists"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|a| a["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        list.push(NeteaseSong {
            id,
            name,
            artists: artists.join(", "),
        });
    }
    Ok(list)
}

/// 一次网易取链尝试的结果。
#[derive(Debug, Clone)]
pub struct NeteaseLink {
    pub url: String,
    pub size: u64,
    pub br: u64,
}

/// 取网易歌曲的 320k mp3 直链。`br=320000` 即 320kbps mp3。
/// 成功返回 Some；返回 None 表示网易也拿不到（无版权/需 VIP）。
pub(crate) async fn fetch_320_mp3_link(song: &NeteaseSong) -> Result<Option<NeteaseLink>, String> {
    let body = format!("ids=%5B{}%5D&br=320000", song.id);
    let resp = CLIENT
        .post("https://music.163.com/api/song/enhance/player/url?br=320000")
        .header("Referer", "https://music.163.com/")
        .header("User-Agent", "Mozilla/5.0")
        .header("Cookie", GUEST_COOKIE)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("网易取链网络错误: {}", e))?;

    let text = resp
        .text()
        .await
        .map_err(|e| format!("网易取链读取失败: {}", e))?;
    let data: Value =
        serde_json::from_str(&text).map_err(|e| format!("网易取链解析失败: {}", e))?;

    if data["code"].as_i64().unwrap_or(-1) != 200 {
        return Err(format!("网易取链错误: code={}", data["code"]));
    }
    let item = data["data"]
        .as_array()
        .and_then(|a| a.first())
        .ok_or("网易取链无数据")?;

    let item_code = item["code"].as_i64().unwrap_or(-1);
    let url = item["url"].as_str().unwrap_or("");
    if url.is_empty() || item_code != 200 {
        return Ok(None);
    }
    let size = item["size"].as_u64().unwrap_or(0);
    let br = item["br"].as_u64().unwrap_or(320000);
    Ok(Some(NeteaseLink {
        url: url.to_string(),
        size,
        br,
    }))
}

/// 在网易上为指定歌曲（标题+歌手）找一条 320k mp3 直链。
/// 走完整流程：搜索 → 选最佳匹配 → 取 320k 链接。
/// 全部失败返回 None（调用方再走酷我或 QQ 加密回退）。
pub(crate) async fn resolve_320_mp3(
    title: &str,
    artist: &str,
) -> Result<Option<NeteaseLink>, String> {
    let keyword = if artist.is_empty() {
        title.to_string()
    } else {
        format!("{} {}", title, artist)
    };
    let songs = search_songs(&keyword).await?;
    if songs.is_empty() {
        return Ok(None);
    }
    let refs: Vec<(&str, &str)> = songs
        .iter()
        .map(|s| (s.name.as_str(), s.artists.as_str()))
        .collect();
    let Some(idx) = source_match::pick_best_index(&refs, title, artist) else {
        return Ok(None);
    };
    let best = &songs[idx];
    log::info!(
        "网易备用：选中「{} - {}」(id={}) 匹配目标「{} - {}」",
        best.name,
        best.artists,
        best.id,
        title,
        artist
    );
    fetch_320_mp3_link(best).await
}
