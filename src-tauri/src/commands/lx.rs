/* 洛雪风格 HTTP 音源。

用户提供的 4 个音源可用性（2026-08-18 实测）：
- 野草 grass.tempmusics.tk：urlinfo 存活，仅声明 kw|128k，/url/ 一律 404
- 野花 flower.tempmusics.tk：urlinfo 存活，声明 tx/wy/kw/kg/mg 128k，/url/ 一律 404
- Huibq lxmusicapi.onrender.com：脚本可下载，Render 服务已 Suspended
- 六音 sixyin：脚本高度加密，官网 sixyin.com DNS 已失效

同合集 pdone/lx-music-source 里的聚合 API api.music.lerd.dpdns.org 可用：
用 QQ songmid 直接取链，无需按歌名跨站匹配。code=200 返回直链；
code=303 时按返回的 request 再请求第三方，从指定 JSON 路径取 url。
*/

use serde_json::{json, Value};

use super::api::CLIENT;

const JUHE_API: &str = "https://api.music.lerd.dpdns.org";
const FLOWER_API: &str = "http://flower.tempmusics.tk/v1";

/// LX 音源命中的直链。
#[derive(Debug, Clone)]
pub struct LxLink {
    pub url: String,
    /// `128k` / `320k` / `flac` / `flac24bit`
    pub quality: String,
}

/// 按 QQ 品质文件名前缀映射到洛雪 quality。
pub fn quality_from_filename(filename: &str) -> &'static str {
    let prefix = filename.get(..4).unwrap_or("");
    match prefix {
        "M800" => "320k",
        "A000" | "F0M0" => "flac",
        "RSM1" | "Q0M0" | "Q0M1" | "AIM0" => "flac24bit",
        _ => "128k",
    }
}

/// 命中后复用的 QQ 品质文件名（明文 mp3，无密钥）。
pub fn backup_filename(media_mid: &str, quality: &str) -> String {
    match quality {
        "320k" | "flac" | "flac24bit" => format!("M800{}.mp3", media_mid),
        _ => format!("M500{}.mp3", media_mid),
    }
}

/// 为 QQ 歌曲（songmid）按优先级尝试 LX 音源。
/// 先走聚合（可到 320k），再走野花 128k（协议保留，当前服务端 /url 会 404）。
pub(crate) async fn resolve_tx_url(
    songmid: &str,
    title: &str,
    artist: &str,
    filename: &str,
) -> Result<Option<LxLink>, String> {
    if songmid.is_empty() {
        return Ok(None);
    }
    let requested = quality_from_filename(filename);
    let mut qualities: Vec<&str> = Vec::new();
    match requested {
        "flac24bit" => qualities.extend(["flac24bit", "flac", "320k", "128k"]),
        "flac" => qualities.extend(["flac", "320k", "128k"]),
        "320k" => qualities.extend(["320k", "128k"]),
        _ => qualities.push("128k"),
    }

    for quality in qualities {
        match fetch_juhe_tx(songmid, title, artist, quality).await {
            Ok(Some(link)) => return Ok(Some(link)),
            Ok(None) => log::info!("聚合音源 {} {} 无结果", songmid, quality),
            Err(e) => log::warn!("聚合音源 {} {} 失败: {}", songmid, quality, e),
        }
    }

    match fetch_flower_tx(songmid, "128k").await {
        Ok(Some(link)) => return Ok(Some(link)),
        Ok(None) => log::info!("野花音源 {} 无结果", songmid),
        Err(e) => log::warn!("野花音源失败: {}", e),
    }

    Ok(None)
}

async fn fetch_juhe_tx(
    songmid: &str,
    title: &str,
    artist: &str,
    quality: &str,
) -> Result<Option<LxLink>, String> {
    let body = json!({
        "type": quality,
        "musicInfo": {
            "songmid": songmid,
            "strMediaMid": songmid,
            "source": "tx",
            "name": title,
            "singer": artist,
        }
    });
    let resp = CLIENT
        .post(format!("{}/tx", JUHE_API))
        .header("Content-Type", "application/json")
        .header("User-Agent", "lx-music-desktop/2.5.0")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("聚合音源网络错误: {}", e))?;
    let text = resp
        .text()
        .await
        .map_err(|e| format!("聚合音源读取失败: {}", e))?;
    let data: Value =
        serde_json::from_str(&text).map_err(|e| format!("聚合音源解析失败: {}", e))?;
    let code = data["code"].as_i64().unwrap_or(-1);
    if code == 200 {
        let url = data["data"]["url"]
            .as_str()
            .unwrap_or("")
            .trim()
            .to_string();
        if url.starts_with("http") {
            return Ok(Some(LxLink {
                url,
                quality: quality.to_string(),
            }));
        }
        return Ok(None);
    }
    if code == 303 {
        return follow_juhe_303(&data["data"], quality).await;
    }
    log::info!(
        "聚合音源返回 code={} msg={}",
        code,
        data["msg"].as_str().unwrap_or("")
    );
    Ok(None)
}

async fn follow_juhe_303(spec: &Value, quality: &str) -> Result<Option<LxLink>, String> {
    let url = spec["request"]["url"].as_str().unwrap_or("").trim();
    if url.is_empty() {
        return Ok(None);
    }
    let method = spec["request"]["options"]["method"]
        .as_str()
        .unwrap_or("GET")
        .to_uppercase();
    let mut req = match method.as_str() {
        "POST" => CLIENT.post(url),
        _ => CLIENT.get(url),
    };
    if let Some(headers) = spec["request"]["options"]["headers"].as_object() {
        for (k, v) in headers {
            if let Some(val) = v.as_str() {
                req = req.header(k.as_str(), val);
            }
        }
    }
    let resp = req
        .send()
        .await
        .map_err(|e| format!("聚合音源 303 网络错误: {}", e))?;
    let text = resp
        .text()
        .await
        .map_err(|e| format!("聚合音源 303 读取失败: {}", e))?;
    let parsed: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => {
            log::info!("聚合音源 303 响应不是 JSON");
            return Ok(None);
        }
    };
    let wrapped = json!({ "body": parsed });
    let check_keys = json_path_keys(&spec["response"]["check"]["key"]);
    let expected = &spec["response"]["check"]["value"];
    match json_path(&wrapped, &check_keys) {
        Some(actual) if json_values_match(actual, expected) => {}
        Some(actual) => {
            log::info!("聚合音源 303 校验失败: {} != {}", actual, expected);
            return Ok(None);
        }
        None => {
            log::info!("聚合音源 303 校验路径缺失");
            return Ok(None);
        }
    }
    let url_keys = json_path_keys(&spec["response"]["url"]);
    let Some(url_val) = json_path(&wrapped, &url_keys) else {
        return Ok(None);
    };
    let url = url_val.as_str().unwrap_or("").trim().to_string();
    if url.starts_with("http") {
        Ok(Some(LxLink {
            url,
            quality: quality.to_string(),
        }))
    } else {
        Ok(None)
    }
}

fn json_values_match(actual: &Value, expected: &Value) -> bool {
    if actual == expected {
        return true;
    }
    actual.as_i64() == expected.as_i64() && actual.as_i64().is_some()
}

/// 野花：GET {base}/url/tx/{songmid}/{quality}，tag 为 path 数字片段 JSON 的 hex。
async fn fetch_flower_tx(songmid: &str, quality: &str) -> Result<Option<LxLink>, String> {
    let path = format!("/url/tx/{}/{}", songmid, quality);
    let tag = flower_tag(&path);
    let url = format!("{}{}", FLOWER_API, path);
    let resp = CLIENT
        .get(&url)
        .header("User-Agent", "lx-music/desktop")
        .header("ver", "2.5.0")
        .header("source-ver", "1.0.0")
        .header("tag", tag)
        .timeout(std::time::Duration::from_secs(4))
        .send()
        .await
        .map_err(|e| format!("野花网络错误: {}", e))?;
    if !resp.status().is_success() {
        return Ok(None);
    }
    let text = resp
        .text()
        .await
        .map_err(|e| format!("野花读取失败: {}", e))?;
    let data: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    if data["code"].as_i64().unwrap_or(-1) != 0 {
        return Ok(None);
    }
    let url = data["data"]
        .as_str()
        .or_else(|| data["data"]["url"].as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if url.starts_with("http") {
        Ok(Some(LxLink {
            url,
            quality: quality.to_string(),
        }))
    } else {
        Ok(None)
    }
}

fn json_path_keys(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn json_path<'a>(root: &'a Value, keys: &[String]) -> Option<&'a Value> {
    let mut cur = root;
    for key in keys {
        cur = if let Ok(i) = key.parse::<usize>() {
            cur.get(i)?
        } else {
            cur.get(key)?
        };
    }
    Some(cur)
}

/// 等价 JS /(?:\d\w)+/g：连续「数字+单词字符」对。
pub fn digit_word_groups(input: &str) -> Vec<String> {
    let bytes = input.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i].is_ascii_digit() && is_word_char(bytes[i + 1]) {
            let start = i;
            i += 2;
            while i + 1 < bytes.len() && bytes[i].is_ascii_digit() && is_word_char(bytes[i + 1]) {
                i += 2;
            }
            out.push(input[start..i].to_string());
        } else {
            i += 1;
        }
    }
    out
}

fn is_word_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// 与野花脚本 JSON.stringify(match, null, 1) 再 hex 一致。
pub fn flower_tag(path: &str) -> String {
    let groups = digit_word_groups(path);
    let json = js_stringify_indent1(&groups);
    to_hex(json.as_bytes())
}

fn js_stringify_indent1(items: &[String]) -> String {
    let mut s = String::from("[\n");
    for (i, item) in items.iter().enumerate() {
        s.push(' ');
        s.push_str(&serde_json::to_string(item).unwrap_or_else(|_| format!("\"{}\"", item)));
        if i + 1 != items.len() {
            s.push(',');
        }
        s.push('\n');
    }
    s.push(']');
    s
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_maps_common_prefixes() {
        assert_eq!(quality_from_filename("M800002dXBY24GGon8.mp3"), "320k");
        assert_eq!(quality_from_filename("M500002dXBY24GGon8.mp3"), "128k");
        assert_eq!(quality_from_filename("F0M0002dXBY24GGon8.mflac"), "flac");
        assert_eq!(
            quality_from_filename("RSM1002dXBY24GGon8.mflac"),
            "flac24bit"
        );
    }

    #[test]
    fn backup_filename_uses_mp3_tags() {
        assert_eq!(
            backup_filename("002dXBY24GGon8", "320k"),
            "M800002dXBY24GGon8.mp3"
        );
        assert_eq!(
            backup_filename("002dXBY24GGon8", "128k"),
            "M500002dXBY24GGon8.mp3"
        );
    }

    #[test]
    fn flower_tag_matches_js_implementation() {
        let path = "/url/tx/003oL2pE3RZat5/128k";
        assert_eq!(digit_word_groups(path), vec!["003o", "2p", "3R", "128k"]);
        assert_eq!(
            flower_tag(path),
            "5b0a20223030336f222c0a20223270222c0a20223352222c0a20223132386b220a5d"
        );
    }

    #[test]
    fn json_path_walks_nested_keys() {
        let v = json!({"body":{"code":200,"data":{"url":"http://x"}}});
        let keys = vec!["body".into(), "data".into(), "url".into()];
        assert_eq!(
            json_path(&v, &keys).and_then(|x| x.as_str()),
            Some("http://x")
        );
    }
}
