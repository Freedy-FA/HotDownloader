use super::super::download::task::SongInfo;

/// 过滤文件名中的非法字符
pub fn sanitize_name(raw: &str) -> String {
    raw.replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|'], "_")
}

/// 应用命名模板，替换变量
pub fn apply_template(
    template: &str,
    artist: &str,
    title: &str,
    album: &str,
    quality: &str,
) -> String {
    template
        .replace("{song}", title)
        .replace("{artist}", artist)
        .replace("{album}", album)
        .replace("{quality}", quality)
}

/// 生成最终文件名（不含扩展名）
pub fn build_filename(template: &str, info: &SongInfo) -> String {
    let name = apply_template(
        template,
        &info.artist,
        &info.title,
        &info.album,
        &info.quality,
    );
    let sanitized = sanitize_name(&name);
    // 若过滤后为空，回退到默认模板
    if sanitized.trim().is_empty() {
        let fallback = apply_template(
            "{song} - {artist}",
            &info.artist,
            &info.title,
            &info.album,
            &info.quality,
        );
        let fallback_sanitized = sanitize_name(&fallback);
        if fallback_sanitized.trim().is_empty() {
            "未知歌曲".to_string()
        } else {
            fallback_sanitized
        }
    } else {
        sanitized
    }
}
