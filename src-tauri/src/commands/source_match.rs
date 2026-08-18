//! 音源歌曲匹配工具：跨音源（QQ / 网易 / 酷我）选歌时统一归一化歌名、
//! 模糊匹配歌手，尽量挑出原唱而非翻唱/伴奏/片段版。
//!
//! 各音源搜索返回结构不同，但都需要同样的「歌名归一化 + 歌手包含」判断，
//! 因此把这部分逻辑集中在此，供 `netease` / `kuwo` 等模块复用。

/// 归一化歌名：去除括号及其后内容、去除首尾空白与多余符号。
/// 例：`晴天(深情版)` / `晴天 (女声版)` → `晴天`
pub fn normalize_name(raw: &str) -> String {
    let mut out = String::new();
    let mut depth = 0i32;
    for c in raw.chars() {
        match c {
            '(' | '（' | '[' | '【' => depth += 1,
            ')' | '）' | ']' | '】' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out.trim().trim_end_matches('-').trim().to_string()
}

/// 简易包含匹配：判断 `target` 是否出现在 `haystack` 中（忽略大小写、去空格）。
pub fn loose_contains(haystack: &str, target: &str) -> bool {
    let norm = |s: &str| s.to_lowercase().replace(' ', "");
    norm(haystack).contains(&norm(target))
}

/// 歌名是否疑似「非原唱」版本：伴奏 / 片段 / DJ / 翻唱 标记。
/// 用于把搜索结果里的低质量版本排到后面。
pub fn looks_like_non_original(name: &str) -> bool {
    let lower = name.to_lowercase();
    ["伴奏", "片段", "dj版", "dj ray", "remix", "cover", "翻唱", "ktv"]
        .iter()
        .any(|k| lower.contains(k))
}

/// 通用「按 标题+歌手 在搜索结果里挑最佳」策略。
/// `songs` 为 `(name, artists)` 元组列表，返回选中的下标。
///
/// 优先级：
/// 1. 歌名归一化相等 + 歌手匹配 + 非伴奏/片段
/// 2. 歌名归一化相等 + 非伴奏/片段
/// 3. 歌名归一化相等（含伴奏/片段也接受）
/// 4. 歌名包含目标 + 歌手匹配 + 非伴奏/片段
/// 5. 任意首条
pub fn pick_best_index(songs: &[(&str, &str)], title: &str, artist: &str) -> Option<usize> {
    if songs.is_empty() {
        return None;
    }
    let norm_title = normalize_name(title);
    if norm_title.is_empty() {
        return Some(0);
    }

    let mut tiers: [Vec<usize>; 4] = Default::default();
    for (i, (name, artists)) in songs.iter().enumerate() {
        let norm_name = normalize_name(name);
        let name_eq = norm_name == norm_title;
        let name_contains = loose_contains(&norm_name, &norm_title);
        let artist_match = !artist.is_empty() && loose_contains(artists, artist);
        let original = !looks_like_non_original(name);

        if name_eq && artist_match && original {
            return Some(i);
        }
        if name_eq && original {
            tiers[0].push(i);
        } else if name_eq {
            tiers[1].push(i);
        } else if name_contains && artist_match && original {
            tiers[2].push(i);
        } else if name_contains {
            tiers[3].push(i);
        }
    }
    for tier in &tiers {
        if let Some(&i) = tier.first() {
            return Some(i);
        }
    }
    Some(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_brackets() {
        assert_eq!(normalize_name("晴天(深情版)"), "晴天");
        assert_eq!(normalize_name("晴天 (女声版)"), "晴天");
        assert_eq!(normalize_name("晴天"), "晴天");
    }

    #[test]
    fn loose_contains_ignores_case_and_space() {
        assert!(loose_contains("Jay Chou", "jaychou"));
        assert!(loose_contains("周杰伦", "周杰"));
        assert!(!loose_contains("刘德华", "周杰伦"));
    }

    #[test]
    fn pick_best_prefers_original_artist_match() {
        let songs = vec![
            ("晴天(伴奏)", "周杰伦"),
            ("晴天", "周杰伦"),
            ("晴天(女声版)", "GYBeat"),
        ];
        assert_eq!(pick_best_index(&songs, "晴天", "周杰伦"), Some(1));
    }

    #[test]
    fn pick_best_skips_accompaniment_when_possible() {
        let songs = vec![
            ("晴天(伴奏)", "周杰伦"),
            ("晴天(片段)", "周杰伦"),
            ("晴天", "Zyboy忠宇"),
        ];
        // 没有原唱+歌手匹配的非伴奏项，退到归一化相等的非伴奏（第3首）
        assert_eq!(pick_best_index(&songs, "晴天", "周杰伦"), Some(2));
    }

    #[test]
    fn pick_best_returns_none_on_empty() {
        let songs: Vec<(&str, &str)> = vec![];
        assert!(pick_best_index(&songs, "晴天", "周杰伦").is_none());
    }
}
