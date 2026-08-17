/// QQ 音乐网页 Cookie 解析。
/// 登录后浏览器里需要至少有 `uin`/`wxuin`，以及 `qm_keyst` 或 `qqmusic_key`。

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

static CURRENT_SESSION: Lazy<Mutex<Option<QqSession>>> = Lazy::new(|| Mutex::new(None));

/// 用设置里的 Cookie 更新当前会话。空字符串或无效 Cookie 视为未登录。
pub fn set_session_from_cookie(cookie: &str) {
    let session = parse_qq_session(cookie);
    if let Ok(mut guard) = CURRENT_SESSION.lock() {
        *guard = session;
    }
}

/// 当前已解析的登录会话。
pub fn current_session() -> Option<QqSession> {
    CURRENT_SESSION.lock().ok().and_then(|g| g.clone())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QqSession {
    pub cookie: String,
    pub uin: String,
}

fn parse_cookie_map(cookie: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for part in cookie.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (key, value) = match part.split_once('=') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue,
        };
        if key.is_empty() {
            continue;
        }
        map.insert(key.to_ascii_lowercase(), value.to_string());
    }
    map
}

fn normalize_uin(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_start_matches(['o', 'O']);
    if trimmed.is_empty() || trimmed == "0" {
        return None;
    }
    if !trimmed.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(trimmed.to_string())
}

/// 从 Cookie 字符串提取登录会话。缺少 uin 或登录票据时返回 None。
pub fn parse_qq_session(cookie: &str) -> Option<QqSession> {
    let cookie = cookie.trim();
    if cookie.is_empty() {
        return None;
    }
    let map = parse_cookie_map(cookie);
    let uin = map
        .get("uin")
        .or_else(|| map.get("wxuin"))
        .and_then(|v| normalize_uin(v))?;
    let has_ticket = ["qm_keyst", "qqmusic_key", "psrf_qqaccess_token"]
        .iter()
        .any(|k| map.get(*k).map(|v| !v.is_empty()).unwrap_or(false));
    if !has_ticket {
        return None;
    }
    Some(QqSession {
        cookie: cookie.to_string(),
        uin,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_uin_and_qm_keyst() {
        let session = parse_qq_session("uin=o123456789; qm_keyst=abc; other=1").expect("session");
        assert_eq!(session.uin, "123456789");
        assert!(session.cookie.contains("qm_keyst=abc"));
    }

    #[test]
    fn accepts_wxuin_and_qqmusic_key() {
        let session = parse_qq_session("wxuin=987654; qqmusic_key=xyz").expect("session");
        assert_eq!(session.uin, "987654");
    }

    #[test]
    fn rejects_missing_ticket() {
        assert!(parse_qq_session("uin=o123456789").is_none());
    }

    #[test]
    fn rejects_guest_uin() {
        assert!(parse_qq_session("uin=o0; qm_keyst=abc").is_none());
        assert!(parse_qq_session("uin=0; qm_keyst=abc").is_none());
    }

    #[test]
    fn rejects_empty() {
        assert!(parse_qq_session("").is_none());
        assert!(parse_qq_session("   ").is_none());
    }
}
