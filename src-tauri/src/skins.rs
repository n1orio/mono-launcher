//! Оффлайн-скины: выбор PNG + загрузка в публичный скин-API (Cloudflare Worker).
//! При запуске игры с оффлайн-сессией и установленным скином в аргументы Java
//! добавляется `-javaagent:authlib-injector.jar=<SKINS_API_URL>`, благодаря чему
//! свой скин виден в одиночке и на серверах, подключённых к этому же API
//! (см. skins-api/README.md — инструкция для разработчиков серверов).

use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::launcher_root;

/// Базовый URL публичного скин-API (Cloudflare Worker).
/// деплой: skins-api/README.md
pub const SKINS_API_URL: &str = "https://nio-skins.skins-api.workers.dev";

const SKIN_FILE: &str = "skin.png";
const SKIN_META_FILE: &str = "skin.json";

/// Текущий скин игрока (локальное хранилище).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinInfo {
    pub has_skin: bool,
    pub model: String,
    pub path: Option<String>,
}

pub(crate) fn skin_path() -> Result<PathBuf> {
    Ok(launcher_root()?.join(SKIN_FILE))
}

fn skin_meta_path() -> Result<PathBuf> {
    Ok(launcher_root()?.join(SKIN_META_FILE))
}

/// Проверяет, что файл — валидный PNG 64×32 или 64×64.
fn validate_skin_png(bytes: &[u8]) -> Result<()> {
    const MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    if bytes.len() < 8 || !bytes.starts_with(&MAGIC) {
        return Err(anyhow!("Файл не является PNG"));
    }
    if bytes.len() < 29 {
        return Err(anyhow!("PNG повреждён (нет IHDR)"));
    }
    let w = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let h = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    if w != 64 || (h != 32 && h != 64) {
        return Err(anyhow!(
            "Некорректный размер скина: {w}x{h} (нужно 64x32 или 64x64)"
        ));
    }
    Ok(())
}

/// Текущий скин.
pub fn get_skin() -> Result<SkinInfo> {
    let path = skin_path()?;
    let has = path.exists();
    let model = skin_meta_path()
        .and_then(|p| match fs::read_to_string(&p) {
            Ok(raw) => {
                let v: serde_json::Value = serde_json::from_str(&raw)?;
                Ok(v.get("model")
                    .and_then(|m| m.as_str())
                    .unwrap_or("classic")
                    .to_string())
            }
            Err(_) => Ok("classic".to_string()),
        })
        .unwrap_or_else(|_| "classic".to_string());
    Ok(SkinInfo {
        has_skin: has,
        model,
        path: has.then(|| path.to_string_lossy().to_string()),
    })
}

/// Копирует выбранный PNG в хранилище лаунчера.
pub fn set_skin_local(source: &str, model: &str) -> Result<SkinInfo> {
    let model = if model == "slim" { "slim" } else { "classic" };
    let src = PathBuf::from(source);
    let bytes = fs::read(&src).with_context(|| format!("Не удалось прочитать файл: {source}"))?;
    validate_skin_png(&bytes)?;
    let target = skin_path()?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&target, &bytes)?;
    fs::write(
        skin_meta_path()?,
        serde_json::json!({ "model": model }).to_string(),
    )?;
    get_skin()
}

/// Удаляет локальный скин.
pub fn clear_skin_local() -> Result<()> {
    for p in [skin_path()?, skin_meta_path()?] {
        if p.exists() {
            fs::remove_file(p)?;
        }
    }
    Ok(())
}

/// Загружает скин в публичный API (`PUT /skins/<ник>`, тело — PNG).
pub async fn upload_skin(
    client: &reqwest::Client,
    nick: &str,
    bytes: &[u8],
    model: &str,
) -> Result<()> {
    let url = format!("{SKINS_API_URL}/skins/{}", nick.to_lowercase());
    let resp = client
        .put(&url)
        .header("X-Skin-Model", model)
        .body(bytes.to_vec())
        .send()
        .await
        .with_context(|| format!("Не удалось связаться со скин-API: {SKINS_API_URL}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        return Err(anyhow!("Скин-API вернул ошибку: {status}"));
    }
    Ok(())
}

/// Удаляет скин из API.
pub async fn delete_remote_skin(client: &reqwest::Client, nick: &str) -> Result<()> {
    let url = format!("{SKINS_API_URL}/skins/{}", nick.to_lowercase());
    let resp = client.delete(&url).send().await?;
    if !resp.status().is_success() && resp.status().as_u16() != 404 {
        return Err(anyhow!("Скин-API вернул ошибку: {}", resp.status()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_of(w: u32, h: u32) -> Vec<u8> {
        let mut b = vec![0u8; 29];
        b[..8].copy_from_slice(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        b[16..20].copy_from_slice(&w.to_be_bytes());
        b[20..24].copy_from_slice(&h.to_be_bytes());
        b
    }

    #[test]
    fn accepts_64x64_and_64x32() {
        assert!(validate_skin_png(&png_of(64, 64)).is_ok());
        assert!(validate_skin_png(&png_of(64, 32)).is_ok());
    }

    #[test]
    fn rejects_bad_sizes_and_non_png() {
        assert!(validate_skin_png(&png_of(32, 32)).is_err());
        assert!(validate_skin_png(&png_of(64, 128)).is_err());
        assert!(validate_skin_png(b"not-a-png").is_err());
        assert!(validate_skin_png(&[]).is_err());
    }
}
