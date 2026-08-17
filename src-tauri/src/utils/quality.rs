/// QQ 音乐品质文件名辅助：从品质文件名提取 media_mid，并在当前品质被拦截时给出**更低**音质回退。
///
/// 未登录时 `CgiGetVkey` 对 `.mp3` / `.m4a` / `.ape` 一律返回 104003。
/// 加密容器 `.mflac` / `.mgg` 仍可通过 `GetEVkey` 下载。
/// 回退只允许比请求音质更低的加密格式，禁止升到 flac。

use std::path::Path;

/// 标准品质：前缀、扩展名、从低到高的等级、是否加密容器。
/// 等级与前端 `ALL_QUALITY_ORDER` 对齐。
const STANDARD_QUALITIES: &[(&str, &str, u8, bool)] = &[
    ("C200", ".m4a", 0, false),   // 48kacc
    ("C400", ".m4a", 1, false),   // 96kacc
    ("C600", ".m4a", 2, false),   // 192kacc
    ("O4M0", ".mgg", 3, true),    // 96kogg
    ("O6M0", ".mgg", 4, true),    // 192kogg
    ("M500", ".mp3", 5, false),   // 128kmp3
    ("M800", ".mp3", 6, false),   // 320kmp3
    ("A000", ".ape", 7, false),   // ape
    ("F0M0", ".mflac", 8, true),  // flac
    ("RSM1", ".mflac", 9, true),  // hires
];

fn stem_of(filename: &str) -> Option<&str> {
    Path::new(filename).file_stem()?.to_str()
}

fn prefix_of(filename: &str) -> Option<&str> {
    let stem = stem_of(filename)?;
    if stem.len() < 4 {
        return None;
    }
    Some(&stem[..4])
}

fn spec_by_prefix(prefix: &str) -> Option<(&'static str, &'static str, u8, bool)> {
    STANDARD_QUALITIES
        .iter()
        .copied()
        .find(|(p, _, _, _)| *p == prefix)
}

/// 从品质文件名提取 media_mid。
/// 文件名格式为 4 字符前缀 + media_mid + 扩展名，例如 `M800002dXBY24GGon8.mp3`。
pub fn extract_media_mid(filename: &str) -> Option<&str> {
    let stem = stem_of(filename)?;
    if stem.len() <= 4 {
        return None;
    }
    Some(&stem[4..])
}

fn extension_lower(filename: &str) -> String {
    Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default()
}

/// 已是加密容器，走 GetEVkey。
pub fn is_encrypted_filename(filename: &str) -> bool {
    matches!(extension_lower(filename).as_str(), "mgg" | "mflac")
}

const QUALITY_LABELS: &[(&str, &str)] = &[
    ("C200", "48kacc"),
    ("C400", "96kacc"),
    ("C600", "192kacc"),
    ("O4M0", "96kogg"),
    ("O6M0", "192kogg"),
    ("M500", "128kmp3"),
    ("M800", "320kmp3"),
    ("A000", "ape"),
    ("F0M0", "flac"),
    ("RSM1", "hires"),
    ("Q0M0", "杜比全景声"),
    ("Q0M1", "臻品全景声"),
    ("AIM0", "臻品母带"),
];

/// 从品质文件名还原前端音质标签。无法识别时返回文件名本身。
pub fn quality_label_from_filename(filename: &str) -> String {
    prefix_of(filename)
        .and_then(|prefix| {
            QUALITY_LABELS
                .iter()
                .find(|(p, _)| *p == prefix)
                .map(|(_, label)| (*label).to_string())
        })
        .unwrap_or_else(|| filename.to_string())
}

/// 当前品质被平台拦截时，按从高到低返回**更低**的加密文件名。
/// 不会返回比请求更高的音质（例如 320kmp3 不会回退到 flac）。
/// 特殊音质（杜比 / 臻品）的 mid 不是 media_mid，无法安全推导，返回空。
pub fn lower_encrypted_fallbacks(filename: &str) -> Vec<String> {
    let Some(prefix) = prefix_of(filename) else {
        return Vec::new();
    };
    let Some((_, _, rank, _)) = spec_by_prefix(prefix) else {
        return Vec::new();
    };
    let Some(media) = extract_media_mid(filename) else {
        return Vec::new();
    };

    STANDARD_QUALITIES
        .iter()
        .rev()
        .filter(|(_, _, cand_rank, encrypted)| *encrypted && *cand_rank < rank)
        .map(|(cand_prefix, cand_ext, _, _)| format!("{}{}{}", cand_prefix, media, cand_ext))
        .collect()
}

/// 是否属于「该音质当前拿不到链接」——应尝试更低音质，而不是当网络抖动重试。
pub fn is_unavailable_link_error(err: &str) -> bool {
    err.contains("无法获取下载链接")
        || err.contains("104003")
        || err.contains("已下架")
        || err.contains("禁止下载")
}

/// 是否属于可重试的瞬时网络错误。
pub fn is_retryable_link_error(err: &str) -> bool {
    err.starts_with("网络错误") || err.starts_with("读取响应失败")
}

/// 一次取链尝试的结果。`filename` 是这次实际请求的品质文件名。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkAttempt {
    Ok {
        url: String,
        key: String,
        filename: String,
    },
    Err {
        filename: String,
        error: String,
    },
}

/// 根据首选品质的结果，决定最终用哪条链接。
/// 首选成功则原样返回；平台拒绝时按 `lower_encrypted_fallbacks` 顺序取第一条成功的回退。
pub fn resolve_link_attempt(
    requested: &str,
    primary: LinkAttempt,
    fallbacks: &[LinkAttempt],
) -> Result<(String, String, String), String> {
    match primary {
        LinkAttempt::Ok { url, key, filename } => Ok((url, key, filename)),
        LinkAttempt::Err { error, .. } if is_unavailable_link_error(&error) => {
            if lower_encrypted_fallbacks(requested).is_empty() {
                return Err(error);
            }
            let mut last_err = error;
            for attempt in fallbacks {
                match attempt {
                    LinkAttempt::Ok { url, key, filename } => {
                        return Ok((url.clone(), key.clone(), filename.clone()));
                    }
                    LinkAttempt::Err { error, .. } => last_err = error.clone(),
                }
            }
            Err(last_err)
        }
        LinkAttempt::Err { error, .. } => Err(error),
    }
}

/// 从歌词接口 JSON 中取出 LRC 文本。
pub fn parse_lyric_json(text: &str) -> Result<String, String> {
    let data: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("解析歌词失败: {}", e))?;
    let code = data["code"].as_i64().unwrap_or(-1);
    let retcode = data["retcode"].as_i64().unwrap_or(0);
    if code != 0 && retcode != 0 {
        return Err(format!("歌词接口错误: code={}, retcode={}", code, retcode));
    }
    let lyric = data["lyric"].as_str().unwrap_or("").trim().to_string();
    if lyric.is_empty() {
        return Err("暂无歌词".into());
    }
    Ok(lyric)
}

/// 音频文件对应的同名 `.lrc` 路径。
pub fn sidecar_lrc_path(audio_path: &str) -> std::path::PathBuf {
    Path::new(audio_path).with_extension("lrc")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_media_mid_from_standard_names() {
        assert_eq!(
            extract_media_mid("M800002dXBY24GGon8.mp3"),
            Some("002dXBY24GGon8")
        );
        assert_eq!(
            extract_media_mid("C200002dXBY24GGon8.m4a"),
            Some("002dXBY24GGon8")
        );
        assert_eq!(
            extract_media_mid("F0M0002dXBY24GGon8.mflac"),
            Some("002dXBY24GGon8")
        );
    }

    #[test]
    fn extract_media_mid_rejects_short_names() {
        assert_eq!(extract_media_mid("abc.mp3"), None);
        assert_eq!(extract_media_mid(""), None);
    }

    #[test]
    fn mp3_downgrades_to_lower_ogg_not_flac() {
        assert_eq!(
            lower_encrypted_fallbacks("M800002dXBY24GGon8.mp3"),
            vec![
                "O6M0002dXBY24GGon8.mgg".to_string(),
                "O4M0002dXBY24GGon8.mgg".to_string(),
            ]
        );
        assert_eq!(
            lower_encrypted_fallbacks("M500002dXBY24GGon8.mp3"),
            vec![
                "O6M0002dXBY24GGon8.mgg".to_string(),
                "O4M0002dXBY24GGon8.mgg".to_string(),
            ]
        );
    }

    #[test]
    fn flac_downgrades_to_ogg_only() {
        assert_eq!(
            lower_encrypted_fallbacks("F0M0002dXBY24GGon8.mflac"),
            vec![
                "O6M0002dXBY24GGon8.mgg".to_string(),
                "O4M0002dXBY24GGon8.mgg".to_string(),
            ]
        );
    }

    #[test]
    fn lowest_encrypted_has_no_fallback() {
        assert!(lower_encrypted_fallbacks("O4M0002dXBY24GGon8.mgg").is_empty());
    }

    #[test]
    fn special_quality_cannot_derive_media_mid_fallbacks() {
        // 臻品母带的 stem 不是 media_mid，不能拿去拼 F0M0/O6M0
        assert!(lower_encrypted_fallbacks("AIM0notMediaMid.mflac").is_empty());
    }

    #[test]
    fn unavailable_error_detects_platform_blocks() {
        assert!(is_unavailable_link_error("无法获取下载链接"));
        assert!(is_unavailable_link_error("获取下载链接失败，错误码: 104003"));
        assert!(is_unavailable_link_error("该歌曲已下架或禁止下载"));
        assert!(!is_unavailable_link_error("网络错误: timeout"));
        assert!(!is_unavailable_link_error("解析响应失败"));
    }

    #[test]
    fn retryable_error_only_matches_transport() {
        assert!(is_retryable_link_error("网络错误: connection reset"));
        assert!(is_retryable_link_error("读取响应失败: eof"));
        assert!(!is_retryable_link_error("无法获取下载链接"));
    }

    #[test]
    fn resolve_falls_back_from_blocked_mp3_to_lower_ogg() {
        let requested = "M800002dXBY24GGon8.mp3";
        let primary = LinkAttempt::Err {
            filename: requested.to_string(),
            error: "无法获取下载链接".to_string(),
        };
        let fallbacks = vec![LinkAttempt::Ok {
            url: "https://wx.music.tc.qq.com/O6M0.mgg".to_string(),
            key: "ekey".to_string(),
            filename: "O6M0002dXBY24GGon8.mgg".to_string(),
        }];
        let (url, key, filename) =
            resolve_link_attempt(requested, primary, &fallbacks).expect("should fallback");
        assert!(url.ends_with(".mgg"));
        assert_eq!(key, "ekey");
        assert_eq!(filename, "O6M0002dXBY24GGon8.mgg");
    }

    #[test]
    fn resolve_keeps_successful_primary() {
        let requested = "M800002dXBY24GGon8.mp3";
        let primary = LinkAttempt::Ok {
            url: "https://cdn/M800.mp3".to_string(),
            key: String::new(),
            filename: requested.to_string(),
        };
        let (url, key, filename) =
            resolve_link_attempt(requested, primary, &[]).expect("primary should win");
        assert!(url.ends_with(".mp3"));
        assert!(key.is_empty());
        assert_eq!(filename, requested);
    }

    #[test]
    fn resolve_does_not_fallback_on_network_error() {
        let requested = "M800002dXBY24GGon8.mp3";
        let primary = LinkAttempt::Err {
            filename: requested.to_string(),
            error: "网络错误: timeout".to_string(),
        };
        let fallbacks = vec![LinkAttempt::Ok {
            url: "https://cdn/O6M0.mgg".to_string(),
            key: "ekey".to_string(),
            filename: "O6M0002dXBY24GGon8.mgg".to_string(),
        }];
        let err = resolve_link_attempt(requested, primary, &fallbacks).unwrap_err();
        assert!(err.starts_with("网络错误"));
    }

    #[test]
    fn parse_lyric_json_reads_lrc() {
        let raw = r#"{"retcode":0,"code":0,"subcode":0,"lyric":"[ti:冻结]\n[ar:林俊杰]\n[00:00.00]冻结"}"#;
        let lyric = parse_lyric_json(raw).expect("lyric");
        assert!(lyric.contains("[ti:冻结]"));
        assert!(lyric.contains("林俊杰"));
    }

    #[test]
    fn parse_lyric_json_rejects_empty() {
        let raw = r#"{"retcode":0,"code":0,"lyric":""}"#;
        assert!(parse_lyric_json(raw).is_err());
    }

    #[test]
    fn quality_label_from_common_filenames() {
        assert_eq!(quality_label_from_filename("M800002dXBY24GGon8.mp3"), "320kmp3");
        assert_eq!(quality_label_from_filename("O6M0002dXBY24GGon8.mgg"), "192kogg");
        assert_eq!(quality_label_from_filename("O4M0002dXBY24GGon8.mgg"), "96kogg");
        assert_eq!(quality_label_from_filename("F0M0002dXBY24GGon8.mflac"), "flac");
    }

    #[test]
    fn sidecar_lrc_replaces_extension() {
        assert_eq!(
            sidecar_lrc_path(r"D:\Music\冻结 - 林俊杰.mp3"),
            Path::new(r"D:\Music\冻结 - 林俊杰.lrc")
        );
        assert_eq!(
            sidecar_lrc_path(r"D:\Music\冻结 - 林俊杰.ogg"),
            Path::new(r"D:\Music\冻结 - 林俊杰.lrc")
        );
    }
}
