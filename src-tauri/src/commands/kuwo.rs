//! 酷我音乐备用音源。
//!
//! 当 QQ 与网易都拿不到 320kmp3 时，酷我作为最后兜底。酷我的
//! `antiserver` 取链接口（`type=convert_url3`）对 320kmp3 / flac 均免登录、
//! 免 token，覆盖率在三个音源里最高——但原唱常因版权下架，结果里翻唱/伴奏/
//! 片段较多，`source_match` 会尽量挑原唱并排除伴奏/片段版。
//!
//! 参考实现：lx-music-desktop `src/renderer/utils/musicSdk/kw/`。
//!
//! 接口：
//! - 搜索：`http://search.kuwo.cn/r.s`（返回单引号 JSON，需规整为双引号）
//! - 取链：`https://antiserver.kuwo.cn/anti.s?type=convert_url3&format=mp3&br=320kmp3&rid=<rid>`

use serde_json::Value;
use url::Url;

use super::api::CLIENT;
use super::source_match;

/// 酷我歌曲信息。
#[derive(Debug, Clone)]
pub struct KuwoSong {
    pub rid: String,
    pub name: String,
    pub artist: String,
}

/// 把酷我 `r.s` 返回的「单引号 JSON」规整为合法 JSON，并清理 `&nbsp;`。
fn sanitize_kuwo_json(raw: &str) -> String {
    // 酷我返回用单引号包裹键值，且字符串内不会出现转义双引号，
    // 直接做字符级替换即可解析。`&nbsp;` 替换为空格便于歌名匹配。
    raw.replace('&', "&amp;") // 先转义 & 避免后续误伤
        .replace("&amp;nbsp;", " ")
        .replace('\'', "\"")
}

/// 搜索酷我歌曲。
pub(crate) async fn search_songs(keyword: &str) -> Result<Vec<KuwoSong>, String> {
    let url = Url::parse_with_params(
        "http://search.kuwo.cn/r.s",
        &[
            ("all", keyword),
            ("ft", "music"),
            ("itemset", "web_2013"),
            ("client", "kt"),
            ("pn", "0"),
            ("rn", "20"),
            ("rformat", "json"),
            ("encoding", "utf8"),
        ],
    )
    .map_err(|e| format!("酷我 URL 构建失败: {}", e))?;

    let resp = CLIENT
        .get(url)
        .header("Referer", "https://www.kuwo.cn/")
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| format!("酷我搜索网络错误: {}", e))?;

    let text = resp
        .text()
        .await
        .map_err(|e| format!("酷我搜索读取失败: {}", e))?;
    let cleaned = sanitize_kuwo_json(&text);
    let data: Value =
        serde_json::from_str(&cleaned).map_err(|e| format!("酷我搜索解析失败: {}", e))?;

    let arr = data["abslist"].as_array().ok_or("酷我搜索无歌曲列表")?;
    let mut list = Vec::new();
    for item in arr {
        // rid 优先 DC_TARGETID，否则 MUSICRID 去掉 "MUSIC_" 前缀
        let rid = item["DC_TARGETID"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(String::from)
            .or_else(|| {
                item["MUSICRID"]
                    .as_str()
                    .and_then(|s| s.strip_prefix("MUSIC_"))
                    .map(String::from)
            });
        let Some(rid) = rid else { continue };
        if rid.is_empty() {
            continue;
        }
        let name = item["SONGNAME"].as_str().unwrap_or("").to_string();
        let artist = item["ARTIST"].as_str().unwrap_or("").to_string();
        list.push(KuwoSong { rid, name, artist });
    }
    Ok(list)
}

/// 一次酷我取链尝试的结果。
#[derive(Debug, Clone)]
pub struct KuwoLink {
    pub url: String,
}

/// 取酷我歌曲的指定码率直链。`br` 取 `320kmp3` / `flac` / `128kmp3`。
/// 成功返回 Some(url)；返回 None 表示该码率无资源。
pub(crate) async fn fetch_link(song: &KuwoSong, br: &str) -> Result<Option<KuwoLink>, String> {
    let url = Url::parse_with_params(
        "https://antiserver.kuwo.cn/anti.s",
        &[
            ("type", "convert_url3"),
            ("format", "mp3"),
            ("br", br),
            ("rid", song.rid.as_str()),
        ],
    )
    .map_err(|e| format!("酷我取链 URL 构建失败: {}", e))?;

    let resp = CLIENT
        .get(url)
        .header("Referer", "https://www.kuwo.cn/")
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| format!("酷我取链网络错误: {}", e))?;

    let text = resp
        .text()
        .await
        .map_err(|e| format!("酷我取链读取失败: {}", e))?;
    let data: Value =
        serde_json::from_str(&text).map_err(|e| format!("酷我取链解析失败: {}", e))?;

    if data["code"].as_i64().unwrap_or(-1) != 200 {
        return Ok(None);
    }
    let url = data["url"].as_str().unwrap_or("").to_string();
    if url.is_empty() {
        return Ok(None);
    }
    Ok(Some(KuwoLink { url }))
}

/// 在酷我上为指定歌曲（标题+歌手）找一条 320k mp3 直链。
/// 走完整流程：搜索 → 选最佳匹配 → 取 320k 链接。
/// 320k 失败时尝试 flac（flac 在酷我覆盖也很广，作为更高质量兜底，但本函数
/// 语义为「320kmp3」，因此 flac 命中也返回，由调用方按文件名区分）。
/// 全部失败返回 None。
pub(crate) async fn resolve_320_mp3(
    title: &str,
    artist: &str,
) -> Result<Option<KuwoLink>, String> {
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
        .map(|s| (s.name.as_str(), s.artist.as_str()))
        .collect();
    let Some(idx) = source_match::pick_best_index(&refs, title, artist) else {
        return Ok(None);
    };
    let best = &songs[idx];
    log::info!(
        "酷我备用：选中「{} - {}」(rid={}) 匹配目标「{} - {}」",
        best.name,
        best.artist,
        best.rid,
        title,
        artist
    );
    if let Some(link) = fetch_link(best, "320kmp3").await? {
        return Ok(Some(link));
    }
    // 320k 没有，再试 flac（酷我 flac 免登录覆盖广）
    fetch_link(best, "flac").await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_fixes_single_quotes_and_nbsp() {
        let raw = "{'abslist':[{'SONGNAME':'晴天&nbsp;(伴奏)','ARTIST':'周杰伦'}]}";
        let cleaned = sanitize_kuwo_json(raw);
        let v: Value = serde_json::from_str(&cleaned).expect("valid json");
        assert_eq!(v["abslist"][0]["SONGNAME"], "晴天 (伴奏)");
        assert_eq!(v["abslist"][0]["ARTIST"], "周杰伦");
    }

    #[test]
    fn rid_from_dc_targetid() {
        let raw = "{'abslist':[{'DC_TARGETID':'123','MUSICRID':'MUSIC_999','SONGNAME':'x','ARTIST':'y'}]}";
        let v: Value = serde_json::from_str(&sanitize_kuwo_json(raw)).unwrap();
        let item = &v["abslist"][0];
        let rid = item["DC_TARGETID"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(String::from)
            .or_else(|| {
                item["MUSICRID"]
                    .as_str()
                    .and_then(|s| s.strip_prefix("MUSIC_"))
                    .map(String::from)
            })
            .unwrap();
        assert_eq!(rid, "123");
    }

    #[test]
    fn rid_falls_back_to_musicrid() {
        let raw = "{'abslist':[{'MUSICRID':'MUSIC_999','SONGNAME':'x','ARTIST':'y'}]}";
        let v: Value = serde_json::from_str(&sanitize_kuwo_json(raw)).unwrap();
        let item = &v["abslist"][0];
        let rid = item["DC_TARGETID"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(String::from)
            .or_else(|| {
                item["MUSICRID"]
                    .as_str()
                    .and_then(|s| s.strip_prefix("MUSIC_"))
                    .map(String::from)
            })
            .unwrap();
        assert_eq!(rid, "999");
    }
}
