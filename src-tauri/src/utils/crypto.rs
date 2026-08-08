pub struct DecryptContext {
    pub key: String,
    pub enabled: bool,
}

/// 初始化解密上下文
pub fn init_decryption(key: &str, enabled: bool) -> DecryptContext {
    DecryptContext {
        key: key.to_string(),
        enabled,
    }
}

/// 数据块流式解密（预留接口，当前不做处理）
pub fn decrypt_chunk(_ctx: &DecryptContext, _data: &mut [u8], _size: u64, _offset: u64) {
    // 预留解密逻辑，后续实现
}