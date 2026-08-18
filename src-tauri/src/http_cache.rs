//! Персистентный HTTP-кеш с условными запросами (ETag/304).
//!
//! Кэширует тело ответа и ETag на диске (ключ = sha1(URL)). При повторном
//! запросе отправляется `If-None-Match`; если сервер отвечает 304 — отдаём
//! кэшированное тело без повторной загрузки. При сетевой ошибке — фолбэк на
//! кэш. Так повторные запуски лаунчера не дёргают сеть повторно.

use crate::config::launcher_root;
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct CachedEntry {
    etag: Option<String>,
    body: String,
}

fn cache_file(url: &str) -> PathBuf {
    let mut h = Sha1::new();
    h.update(url.as_bytes());
    let key = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(h.finalize());
    let root = launcher_root()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".http-cache");
    root.join(format!("{key}.json"))
}

fn read_entry(url: &str) -> Option<CachedEntry> {
    let path = cache_file(url);
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<CachedEntry>(&raw).ok()
}

fn write_entry(url: &str, etag: Option<String>, body: &[u8]) {
    let file = cache_file(url);
    if let Some(parent) = file.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return;
        }
    }
    let entry = CachedEntry {
        etag,
        body: base64::engine::general_purpose::STANDARD.encode(body),
    };
    if let Ok(raw) = serde_json::to_string(&entry) {
        let tmp = file.with_extension("json.tmp");
        if std::fs::write(&tmp, raw).is_ok() {
            let _ = std::fs::rename(&tmp, &file);
        }
    }
}

fn remove_entry(url: &str) {
    let _ = std::fs::remove_file(cache_file(url));
}

/// Выполняет GET с условным запросом. Возвращает тело (из сети или кэша) либо
/// `None` (не нашлось / ошибка сервера / 404 без кэша).
pub async fn cached_get(
    client: &reqwest::Client,
    url: &str,
    extra_headers: &[(&str, &str)],
) -> Option<Vec<u8>> {
    let cached = read_entry(url);
    let mut req = client.get(url);
    for (k, v) in extra_headers {
        req = req.header(*k, *v);
    }
    if let Some(etag) = cached.as_ref().and_then(|c| c.etag.as_deref()) {
        req = req.header("If-None-Match", etag);
    }
    let resp = match req.send().await {
        Ok(r) => r,
        Err(_) => return cached.and_then(|c| base64::engine::general_purpose::STANDARD.decode(c.body).ok()),
    };
    if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
        // 304 — ревалидация прошла, сеть не грузилась.
        return cached.map(|c| {
            let _ = c.etag;
            base64::engine::general_purpose::STANDARD
                .decode(c.body)
                .unwrap_or_default()
        });
    }
    if !resp.status().is_success() {
        // 404/ошибка: сбрасываем кэш, чтобы дальше не возвращать 404 вечно.
        remove_entry(url);
        return None;
    }
    let etag = resp
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let bytes = resp.bytes().await.ok()?;
    write_entry(url, etag, &bytes);
    Some(bytes.to_vec())
}

/// Удобная обёртка для JSON-ответов с заголовком User-Agent.
pub async fn cached_json(
    client: &reqwest::Client,
    url: &str,
) -> Option<serde_json::Value> {
    cached_get(client, url, &[("User-Agent", "mono-launcher")])
        .await
        .and_then(|b| serde_json::from_slice(&b).ok())
}