use crate::utils::qrc;
use once_cell::sync::Lazy;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::command;

const LYRIC_ENDPOINT: &str = "https://u.y.qq.com/cgi-bin/musicu.fcg";

static LYRIC_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("Failed to build lyric HTTP client")
});

#[derive(Debug, Serialize)]
pub struct LyricResponse {
    /// Plain LRC lyrics (merged words).
    pub lrc: Option<String>,
    /// Enhanced LRC lyrics (word-level timing), present only when QRC is available.
    pub elrc: Option<String>,
    /// Raw decrypted LyricContent (QRC or plain LRC string).
    pub raw: Option<String>,
    /// Whether the song appears to be instrumental (basic detection).
    pub instrumental: bool,
}

#[derive(Serialize)]
struct Comm {
    ct: &'static str,
    cv: &'static str,
    uin: &'static str,
}

#[derive(Serialize)]
struct LyricReq {
    method: &'static str,
    module: &'static str,
    param: LyricParam,
}

#[derive(Serialize)]
struct LyricParam {
    crypt: u32,
    ct: u32,
    cv: u32,
    interval: u32,
    lrc_t: u32,
    qrc: u32,
    qrc_t: u32,
    roma: u32,
    roma_t: u32,
    #[serde(rename = "songID")]
    song_id: u64,
    trans: u32,
    trans_t: u32,
    #[serde(rename = "type")]
    type_: i32,
}

impl LyricParam {
    fn new(song_id: u64) -> Self {
        Self {
            crypt: 1,
            ct: 19,
            cv: 1873,
            interval: 0,
            lrc_t: 0,
            qrc: 1,
            qrc_t: 0,
            roma: 0,
            roma_t: 0,
            song_id,
            trans: 0,
            trans_t: 0,
            type_: -1,
        }
    }
}

/// Fetch lyrics by QQ Music song ID.
///
/// Returns `LyricResponse` with both plain LRC and enhanced LRC if QRC is available.
/// https://github.com/lyswhut/lx-music-desktop/blob/9c364b482e5621a1d38b50e8610d2fb974457e6e/src/renderer/utils/musicSdk/tx/lyric.js#L230
#[command]
pub async fn get_lyric_by_id(song_id: u64) -> Result<LyricResponse, String> {
    let body = json!({
        "comm": Comm {
            ct: "19",
            cv: "1859",
            uin: "0",
        },
        "req": LyricReq {
            method: "GetPlayLyricInfo",
            module: "music.musichallSong.PlayLyricInfo",
            param: LyricParam::new(song_id),
        },
    });

    let response = LYRIC_CLIENT
        .post(LYRIC_ENDPOINT)
        .header("Content-Type", "application/json")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.198 Safari/537.36")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("歌词请求失败: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("歌词接口返回 HTTP {}", response.status()));
    }

    let json: Value = response
        .json()
        .await
        .map_err(|e| format!("解析歌词响应失败: {e}"))?;

    // Check gateway and module return codes
    if json["code"] != 0 {
        let code = json["code"].as_i64().unwrap_or(-1);
        return Err(format!("歌词网关错误码 {code}"));
    }
    if json["req"]["code"] != 0 {
        let code = json["req"]["code"].as_i64().unwrap_or(-1);
        return Err(format!("歌词模块错误码 {code}"));
    }

    let encrypted = json["req"]["data"]["lyric"].as_str().unwrap_or("").trim();

    if encrypted.is_empty() {
        return Ok(LyricResponse {
            lrc: None,
            elrc: None,
            raw: None,
            instrumental: false,
        });
    }

    // Decrypt QRC (hex -> custom 3DES -> zlib -> XML)
    let xml = qrc::decrypt(encrypted).map_err(|e| format!("歌词解密失败: {e}"))?;
    let raw_content =
        qrc::extract_lyric_content(&xml).ok_or_else(|| "解密后未找到 LyricContent".to_string())?;

    let is_qrc = qrc::is_qrc(&raw_content);

    let lrc = if is_qrc {
        Some(qrc::to_lrc(&raw_content))
    } else {
        Some(raw_content.clone()) // plain LRC
    };

    let elrc = if is_qrc {
        Some(qrc::to_enhanced_lrc(&raw_content))
    } else {
        None
    };

    // Simple instrumental detection (optional)
    let instrumental = lrc
        .as_ref()
        .map(|s| s.contains("纯音乐") || s.contains("Instrumental"))
        .unwrap_or(false);

    Ok(LyricResponse {
        lrc,
        elrc,
        raw: Some(raw_content),
        instrumental,
    })
}
