//! Wasm C-ABI 导出层
//!
//! 供 Go 等宿主（wazero / wasmtime-go 等 Wasm runtime）直接调用的 `extern "C"` 导出函数。
//! 宿主侧通过模块导出名（如 `nothings_md5`）调用，数据以「指针 + 长度」方式传递。
//!
//! # ABI 约定
//!
//! - **输入参数**：`ptr: *const u8` + `len: usize`，空指针或零长度视为空输入
//! - **定长返回值**（如 MD5 的 32 字节 hex、SHA256 的 64 字节 hex）：
//!   宿主预分配缓冲区，通过 `out` 指针写入
//! - **变长返回值**（如 bcrypt 哈希、JSON）：返回 `*mut u8` 指针，
//!   前 4 字节为小端长度前缀，其后是数据；宿主读取完毕后必须调用
//!   `nothings_free(ptr, 4 + 长度)` 释放；失败时返回空指针
//! - **返回码**：`0` 表示成功，负数表示失败（具体含义见各函数文档）
//!
//! # Go (wazero) 调用示例
//!
//! ```text
//! mod, _ := wazero.Runtime.NewModule(ctx, wasmBin)
//! md5Fn := mod.ExportedFunction("nothings_md5")
//! ```
//!
//! 注意：本模块在原生平台同样参与编译，便于用单元测试验证导出逻辑。

use crate::any_maps::app::AnyMap;
use crate::digests::bcrypt::Bcrypt;
use crate::digests::md5::Md5Hash;
use crate::digests::sha::Sha256Hash;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

/// 从宿主指针构造字节切片
///
/// 空指针或零长度返回空切片，避免宿主侧传 null 崩溃
unsafe fn slice_from_raw<'a>(ptr: *const u8, len: usize) -> &'a [u8] {
    if ptr.is_null() || len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

/// 将变长数据封包为「4 字节小端长度前缀 + 数据」的堆指针，转移所有权给宿主
///
/// 宿主读取完毕后需调用 [`nothings_free`] 释放
fn to_leaked_ptr(data: Vec<u8>) -> *mut u8 {
    let mut buf: Vec<u8> = Vec::with_capacity(4 + data.len());
    buf.extend_from_slice(&(data.len() as u32).to_le_bytes());
    buf.extend_from_slice(&data);
    let mut boxed = buf.into_boxed_slice();
    let ptr = boxed.as_mut_ptr();
    std::mem::forget(boxed);
    ptr
}

/// 从「4 字节小端长度前缀 + 数据」的指针读取数据（仅供测试使用）
#[cfg(test)]
unsafe fn from_leaked_ptr(ptr: *const u8) -> Option<Vec<u8>> {
    unsafe {
        if ptr.is_null() {
            return None;
        }
        let len = u32::from_le_bytes([
            *ptr,
            *(ptr.add(1)),
            *(ptr.add(2)),
            *(ptr.add(3)),
        ]) as usize;
        Some(std::slice::from_raw_parts(ptr.add(4), len).to_vec())
    }
}

// ─────────────────────────────────────────────
// 内存管理
// ─────────────────────────────────────────────

/// 在 wasm 线性内存中分配 `len` 字节，供宿主写入输入数据
///
/// 宿主写入完毕后可将指针传给各导出函数；释放使用 [`nothings_free`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nothings_alloc(len: usize) -> *mut u8 {
    if len == 0 {
        return std::ptr::null_mut();
    }
    let mut buf: Vec<u8> = Vec::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// 释放由 [`nothings_alloc`] 或变长返回值分配的内存
///
/// `len` 必须与分配时一致（变长返回值为 `4 + 数据长度`）
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nothings_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    unsafe { drop(Vec::from_raw_parts(ptr, len, len)) };
}

// ─────────────────────────────────────────────
// 摘要（MD5 / SHA256）
// ─────────────────────────────────────────────

/// 计算 MD5 摘要，输出 32 字节小写 hex 到 `out`（宿主预分配）
///
/// # 返回码
/// - `0`：成功
/// - `-1`：输入不是合法 UTF-8（当前摘要 API 基于字符串）
/// - `-2`：`out` 为空指针
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nothings_md5(data: *const u8, len: usize, out: *mut u8) -> i32 {
    let bytes = unsafe { slice_from_raw(data, len) };
    let Ok(text) = std::str::from_utf8(bytes) else {
        return -1;
    };
    if out.is_null() {
        return -2;
    }
    let hex = Md5Hash::new(text).hash().into_bytes();
    unsafe { std::ptr::copy_nonoverlapping(hex.as_ptr(), out, hex.len()) };
    0
}

/// 计算 SHA256 摘要，输出 64 字节小写 hex 到 `out`（宿主预分配）
///
/// # 返回码
/// - `0`：成功
/// - `-1`：输入不是合法 UTF-8
/// - `-2`：`out` 为空指针
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nothings_sha256(data: *const u8, len: usize, out: *mut u8) -> i32 {
    let bytes = unsafe { slice_from_raw(data, len) };
    let Ok(text) = std::str::from_utf8(bytes) else {
        return -1;
    };
    if out.is_null() {
        return -2;
    }
    let hex = Sha256Hash::new(text).hash().into_bytes();
    unsafe { std::ptr::copy_nonoverlapping(hex.as_ptr(), out, hex.len()) };
    0
}

// ─────────────────────────────────────────────
// Bcrypt 密码哈希
// ─────────────────────────────────────────────

/// 计算 bcrypt 密码哈希
///
/// # 参数
/// - `password` / `len`：明文密码
/// - `cost`：计算开销（4-31，常用 10-12）
///
/// # 返回
/// 长度前缀指针（bcrypt 哈希字符串，约 60 字节），失败返回空指针。
/// 读取后调用 [`nothings_free`] 释放
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nothings_bcrypt_hash(
    password: *const u8,
    len: usize,
    cost: u32,
) -> *mut u8 {
    let bytes = unsafe { slice_from_raw(password, len) };
    let Ok(text) = std::str::from_utf8(bytes) else {
        return std::ptr::null_mut();
    };
    let mut bcrypt = Bcrypt::new(text);
    bcrypt.set_cost(cost);
    match bcrypt.hash() {
        Ok(hashed) => to_leaked_ptr(hashed.into_bytes()),
        Err(_) => std::ptr::null_mut(),
    }
}

/// 校验 bcrypt 密码哈希
///
/// # 返回码
/// - `1`：密码匹配
/// - `0`：密码不匹配
/// - `-1`：输入非法（非 UTF-8）
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nothings_bcrypt_verify(
    password: *const u8,
    password_len: usize,
    hash: *const u8,
    hash_len: usize,
) -> i32 {
    let pw_bytes = unsafe { slice_from_raw(password, password_len) };
    let hash_bytes = unsafe { slice_from_raw(hash, hash_len) };
    let Ok(pw) = std::str::from_utf8(pw_bytes) else {
        return -1;
    };
    let Ok(hashed) = std::str::from_utf8(hash_bytes) else {
        return -1;
    };
    let bcrypt = Bcrypt::new(pw);
    match bcrypt.check(&hashed.to_string()) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(_) => -1,
    }
}

// ─────────────────────────────────────────────
// AnyMap（保序键值映射）
// ─────────────────────────────────────────────

/// 全局 AnyMap 注册表：宿主持有 u32 句柄，Rust 侧维护实例
///
/// 值统一为 JSON（`serde_json::Value`），宿主侧无需关心 Rust 类型系统
static ANYMAP_REGISTRY: Mutex<Option<HashMap<u32, AnyMap<String, serde_json::Value>>>> =
    Mutex::new(None);

/// 句柄计数器，从 1 开始（0 保留为无效值）
static ANYMAP_NEXT_HANDLE: AtomicU32 = AtomicU32::new(1);

fn with_map<R>(handle: u32, f: impl FnOnce(&mut AnyMap<String, serde_json::Value>) -> R) -> Option<R> {
    let mut guard = ANYMAP_REGISTRY.lock().unwrap();
    guard
        .as_mut()?
        .get_mut(&handle)
        .map(f)
}

/// 创建保序键值映射，返回句柄（失败返回 0）
///
/// Go 侧 map 本身无序，AnyMap 保持插入顺序，
/// 通过 [`nothings_anymap_to_json`] 可导出顺序稳定的 JSON 对象
#[unsafe(no_mangle)]
pub extern "C" fn nothings_anymap_new() -> u32 {
    let handle = ANYMAP_NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
    let mut guard = ANYMAP_REGISTRY.lock().unwrap();
    guard
        .get_or_insert_with(HashMap::new)
        .insert(handle, AnyMap::new());
    handle
}

/// 插入或更新键值对（值为 JSON 文本）
///
/// # 返回码
/// - `0`：成功（已存在则原地更新，保持原位置）
/// - `-1`：句柄无效
/// - `-2`：值不是合法 JSON
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nothings_anymap_insert(
    handle: u32,
    key: *const u8,
    key_len: usize,
    value_json: *const u8,
    value_len: usize,
) -> i32 {
    let key_bytes = unsafe { slice_from_raw(key, key_len) };
    let Ok(key) = std::str::from_utf8(key_bytes).map(|s| s.to_string()) else {
        return -2;
    };
    let value_bytes = unsafe { slice_from_raw(value_json, value_len) };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(value_bytes) else {
        return -2;
    };

    match with_map(handle, |map| {
        if map.has(&key) {
            map.set_value_by_key(key.clone(), value)
                .map(|_| ())
                .is_ok()
        } else {
            map.push_datum(key.clone(), value);
            true
        }
    }) {
        Some(true) => 0,
        Some(false) => -3,
        None => -1,
    }
}

/// 读取指定 key 的值（JSON 文本）
///
/// # 返回
/// 长度前缀指针，key 不存在或句柄无效返回空指针。
/// 读取后调用 [`nothings_free`] 释放
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nothings_anymap_get(
    handle: u32,
    key: *const u8,
    key_len: usize,
) -> *mut u8 {
    let key_bytes = unsafe { slice_from_raw(key, key_len) };
    let Ok(key) = std::str::from_utf8(key_bytes) else {
        return std::ptr::null_mut();
    };
    with_map(handle, |map| {
        map.get_value_by_key(&key.to_string())
            .ok()
            .and_then(|v| serde_json::to_string(&v).ok())
    })
    .flatten()
    .map(|json| to_leaked_ptr(json.into_bytes()))
    .unwrap_or(std::ptr::null_mut())
}

/// 删除指定 key
///
/// # 返回码
/// - `1`：删除成功
/// - `0`：key 不存在
/// - `-1`：句柄无效或 key 非法
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nothings_anymap_remove(
    handle: u32,
    key: *const u8,
    key_len: usize,
) -> i32 {
    let key_bytes = unsafe { slice_from_raw(key, key_len) };
    let Ok(key) = std::str::from_utf8(key_bytes) else {
        return -1;
    };
    match with_map(handle, |map| {
        let existed = map.has(&key.to_string());
        map.remove_by_key(&key.to_string());
        existed
    }) {
        Some(true) => 1,
        Some(false) => 0,
        None => -1,
    }
}

/// 导出整个映射为 JSON 对象文本（严格保持插入顺序）
///
/// # 返回
/// 长度前缀指针，句柄无效返回空指针。读取后调用 [`nothings_free`] 释放
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nothings_anymap_to_json(handle: u32) -> *mut u8 {
    let json = with_map(handle, |map| {
        let entries = map.to_entries();
        let mut out = String::from("{");
        for (i, (key, value)) in entries.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            // 借助 serde_json 序列化保证 key 转义正确
            let key_json = serde_json::to_string(key).unwrap_or_else(|_| "\"\"".into());
            let value_json = serde_json::to_string(value).unwrap_or_else(|_| "null".into());
            out.push_str(&key_json);
            out.push(':');
            out.push_str(&value_json);
        }
        out.push('}');
        out
    });
    match json {
        Some(text) => to_leaked_ptr(text.into_bytes()),
        None => std::ptr::null_mut(),
    }
}

/// 获取键值对数量（句柄无效返回 -1）
#[unsafe(no_mangle)]
pub extern "C" fn nothings_anymap_len(handle: u32) -> i32 {
    with_map(handle, |map| map.len() as i32).unwrap_or(-1)
}

/// 销毁句柄对应的映射，释放 Rust 侧内存
///
/// # 返回码
/// - `0`：成功
/// - `-1`：句柄无效
#[unsafe(no_mangle)]
pub extern "C" fn nothings_anymap_drop(handle: u32) -> i32 {
    let mut guard = ANYMAP_REGISTRY.lock().unwrap();
    match guard.as_mut().map(|maps| maps.remove(&handle)) {
        Some(Some(_)) => 0,
        _ => -1,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    unsafe fn leak_bytes(data: &[u8]) -> (*const u8, usize) {
        unsafe {
            let ptr = nothings_alloc(data.len());
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
            (ptr, data.len())
        }
    }

    #[test]
    fn test_md5_export() {
        let input = b"hello";
        let mut out = [0u8; 32];
        let code = unsafe { nothings_md5(input.as_ptr(), input.len(), out.as_mut_ptr()) };
        assert_eq!(code, 0);
        assert_eq!(String::from_utf8_lossy(&out), "5d41402abc4b2a76b9719d911017c592");
    }

    #[test]
    fn test_md5_invalid_utf8() {
        let input = &[0xff, 0xfe];
        let mut out = [0u8; 32];
        let code = unsafe { nothings_md5(input.as_ptr(), input.len(), out.as_mut_ptr()) };
        assert_eq!(code, -1);
    }

    #[test]
    fn test_sha256_export() {
        let input = b"hello";
        let mut out = [0u8; 64];
        let code = unsafe { nothings_sha256(input.as_ptr(), input.len(), out.as_mut_ptr()) };
        assert_eq!(code, 0);
        assert_eq!(
            String::from_utf8_lossy(&out),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_bcrypt_roundtrip() {
        let password = b"secret-password";
        let hash_ptr = unsafe { nothings_bcrypt_hash(password.as_ptr(), password.len(), 4) };
        assert!(!hash_ptr.is_null());

        let hash = unsafe { from_leaked_ptr(hash_ptr).unwrap() };
        unsafe { nothings_free(hash_ptr, 4 + hash.len()) };
        assert!(String::from_utf8_lossy(&hash).starts_with("$2b$"));

        let code = unsafe {
            nothings_bcrypt_verify(
                password.as_ptr(),
                password.len(),
                hash.as_ptr(),
                hash.len(),
            )
        };
        assert_eq!(code, 1);

        let wrong = b"wrong-password";
        let code = unsafe {
            nothings_bcrypt_verify(wrong.as_ptr(), wrong.len(), hash.as_ptr(), hash.len())
        };
        assert_eq!(code, 0);
    }

    #[test]
    fn test_anymap_lifecycle() {
        let handle = nothings_anymap_new();
        assert!(handle != 0);

        // 插入三个键值对
        let (k1, k1_len) = unsafe { leak_bytes(b"alpha") };
        let (v1, v1_len) = unsafe { leak_bytes(br#""first""#) };
        let code = unsafe { nothings_anymap_insert(handle, k1, k1_len, v1, v1_len) };
        assert_eq!(code, 0);

        let (k2, k2_len) = unsafe { leak_bytes(b"beta") };
        let (v2, v2_len) = unsafe { leak_bytes(b"42") };
        let code = unsafe { nothings_anymap_insert(handle, k2, k2_len, v2, v2_len) };
        assert_eq!(code, 0);

        let (k3, k3_len) = unsafe { leak_bytes(b"gamma") };
        let (v3, v3_len) = unsafe { leak_bytes(br#"{"nested":true}"#) };
        let code = unsafe { nothings_anymap_insert(handle, k3, k3_len, v3, v3_len) };
        assert_eq!(code, 0);

        assert_eq!(nothings_anymap_len(handle), 3);

        // 读取单值
        let get_ptr = unsafe { nothings_anymap_get(handle, k2, k2_len) };
        assert!(!get_ptr.is_null());
        let value = unsafe { from_leaked_ptr(get_ptr).unwrap() };
        unsafe { nothings_free(get_ptr, 4 + value.len()) };
        assert_eq!(String::from_utf8_lossy(&value), "42");

        // 保序 JSON 导出
        let json_ptr = unsafe { nothings_anymap_to_json(handle) };
        assert!(!json_ptr.is_null());
        let json = unsafe { from_leaked_ptr(json_ptr).unwrap() };
        unsafe { nothings_free(json_ptr, 4 + json.len()) };
        let json_text = String::from_utf8_lossy(&json).to_string();
        // 顺序必须是 alpha, beta, gamma
        let alpha = json_text.find("alpha").unwrap();
        let beta = json_text.find("beta").unwrap();
        let gamma = json_text.find("gamma").unwrap();
        assert!(alpha < beta && beta < gamma);

        // 更新已有 key 保持位置
        let (v1b, v1b_len) = unsafe { leak_bytes(br#""updated""#) };
        let code = unsafe { nothings_anymap_insert(handle, k1, k1_len, v1b, v1b_len) };
        assert_eq!(code, 0);
        let json_ptr = unsafe { nothings_anymap_to_json(handle) };
        let json = unsafe { from_leaked_ptr(json_ptr).unwrap() };
        unsafe { nothings_free(json_ptr, 4 + json.len()) };
        assert!(json_text.contains("alpha"));
        assert_eq!(String::from_utf8_lossy(&json).find("alpha").unwrap(), alpha);

        // 删除
        let code = unsafe { nothings_anymap_remove(handle, k2, k2_len) };
        assert_eq!(code, 1);
        assert_eq!(nothings_anymap_len(handle), 2);
        let code = unsafe { nothings_anymap_remove(handle, k2, k2_len) };
        assert_eq!(code, 0);

        // 非法 JSON
        let (bad, bad_len) = unsafe { leak_bytes(b"not-json{{") };
        let code = unsafe { nothings_anymap_insert(handle, k1, k1_len, bad, bad_len) };
        assert_eq!(code, -2);

        // 无效句柄
        assert_eq!(nothings_anymap_len(9999), -1);
        assert_eq!(nothings_anymap_drop(9999), -1);

        // 销毁
        assert_eq!(nothings_anymap_drop(handle), 0);
        assert_eq!(nothings_anymap_len(handle), -1);
    }
}
