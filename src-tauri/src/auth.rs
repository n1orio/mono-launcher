use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::config;

/// Сессия пользователя, передаваемая в команду запуска игры.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub user_type: String,
    /// Microsoft OAuth2 refresh_token (только для user_type = "microsoft").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

/// Оффлайн-сессия (для «пираток»): генерируется детерминированный UUID из ника.
pub fn login_offline(username: &str) -> Result<UserSession> {
    let username = username.trim();
    if username.is_empty() {
        return Err(anyhow!("Никнейм не может быть пустым"));
    }
    let normalized = username.to_lowercase();
    let uuid = Uuid::new_v3(&Uuid::NAMESPACE_DNS, normalized.as_bytes()).to_string();
    Ok(UserSession {
        username: username.to_string(),
        uuid,
        access_token: String::new(),
        user_type: "offline".into(),
        refresh_token: None,
    })
}

/// Профиль аккаунта Mono: отдельный слой поверх игровых аккаунтов.
/// Не попадает ни в `session.json`, ни в `accounts.json` — живёт в `mono.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonoProfile {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
}

fn mono_profile_file() -> Result<PathBuf> {
    Ok(config::launcher_root()?.join("mono.json"))
}

/// Загружает сохранённый профиль Mono (None — не залогинен).
pub fn load_mono_profile() -> Result<Option<MonoProfile>> {
    let path = mono_profile_file()?;
    if !path.is_file() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path).context("Не удалось прочитать профиль Mono")?;
    Ok(serde_json::from_str(&text).ok())
}

fn save_mono_profile(profile: &MonoProfile) -> Result<()> {
    let path = mono_profile_file()?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(profile)?)
        .context("Не удалось сохранить профиль Mono")
}

/// Удаляет локальный профиль Mono.
pub fn clear_mono_profile() -> Result<()> {
    let path = mono_profile_file()?;
    if path.is_file() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

/// Ответ бэкенда Mono при регистрации/входе.
#[derive(Debug, Deserialize)]
struct MonoAuthResp {
    token: String,
    user: MonoUser,
}

#[derive(Debug, Deserialize)]
struct MonoUser {
    id: String,
    username: String,
    #[serde(default, rename = "displayName")]
    display_name: Option<String>,
}

/// Вход через аккаунт Mono (собственный бэкенд лаунчера):
/// POST /auth/register | /auth/login, ожидаем `{ token, user: { id, username, displayName } }`.
/// Возвращаем профиль Mono — отдельно от игровых аккаунтов.
async fn mono_auth(
    client: &reqwest::Client,
    path: &str,
    username: &str,
    password: &str,
) -> Result<MonoProfile> {
    let username = username.trim();
    if username.is_empty() {
        return Err(anyhow!("Логин не может быть пустым"));
    }
    if password.is_empty() {
        return Err(anyhow!("Пароль не может быть пустым"));
    }
    let base = crate::config::backend_url();
    let url = format!("{base}/auth/{path}");
    let resp = client
        .post(&url)
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await
        .context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        // Бэкенд отдаёт `{ "error": "..." }`.
        let msg = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|v| v["error"].as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| text.clone());
        let hint = match status.as_u16() {
            401 | 403 => " (проверьте логин и пароль)",
            404 => " (бэкенд Mono недоступен на этом адресе)",
            _ => "",
        };
        return Err(anyhow!("Mono: {}{}", msg, hint));
    }
    let resp: MonoAuthResp = serde_json::from_str(&text).context("Некорректный ответ сервера Mono")?;
    Ok(MonoProfile {
        username: resp.user.display_name.unwrap_or(resp.user.username.clone()),
        uuid: resp.user.id,
        access_token: resp.token,
    })
}

/// Регистрация аккаунта Mono. Сохраняет профиль отдельно от игровых аккаунтов.
pub async fn mono_register(client: &reqwest::Client, username: &str, password: &str) -> Result<MonoProfile> {
    let profile = mono_auth(client, "register", username, password).await?;
    save_mono_profile(&profile)?;
    Ok(profile)
}

/// Вход в аккаунт Mono. Сохраняет профиль отдельно от игровых аккаунтов.
pub async fn mono_login(client: &reqwest::Client, username: &str, password: &str) -> Result<MonoProfile> {
    let profile = mono_auth(client, "login", username, password).await?;
    save_mono_profile(&profile)?;
    Ok(profile)
}

/// Разлогин на сервере Mono (отзывает все токены пользователя). Ошибка не критична.
pub async fn mono_logout(client: &reqwest::Client, access_token: &str) {
    let base = crate::config::backend_url();
    let url = format!("{base}/auth/logout");
    let _ = client
        .post(&url)
        .bearer_auth(access_token)
        .send()
        .await;
}

/// Сборка, вернувшаяся с бэкенда после загрузки на storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonoPackPublic {
    pub id: String,
    pub file: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub size: i64,
    pub sha1: String,
    pub sha512: String,
}

/// Достаёт сообщение об ошибке из JSON-ответа бэкенда `{ "error": "..." }`.
fn api_error(text: &str) -> String {
    serde_json::from_str::<Value>(text)
        .ok()
        .and_then(|v| v["error"].as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| text.to_string())
}

/// Читает .mrpack в multipart-часть "file" (общий паттерн загрузки на бэкенд).
async fn mrpack_part(file_path: &str) -> Result<reqwest::multipart::Part> {
    let bytes = tokio::fs::read(file_path)
        .await
        .context("Не удалось прочитать файл сборки")?;
    if bytes.is_empty() {
        return Err(anyhow!("Файл сборки пуст"));
    }
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("pack.mrpack")
        .to_string();
    reqwest::multipart::Part::bytes(bytes)
        .file_name(file_name)
        .mime_str("application/octet-stream")
        .context("Ошибка multipart")
}

/// Загружает .mrpack на бэкенд (POST /packs/upload, multipart),
/// который проверит мат/длину и перешлёт файл на storage-сервер.
pub async fn mono_upload_pack(
    client: &reqwest::Client,
    access_token: &str,
    file_path: &str,
    name: &str,
    description: &str,
    version: &str,
    changelog: &str,
    min_ram_mb: Option<i64>,
    boosty_blog: Option<String>,
    meta: Option<serde_json::Value>,
    icon_url: Option<String>,
) -> Result<MonoPackPublic> {
    let mut form = reqwest::multipart::Form::new()
        .text("name", name.to_string())
        .text("description", description.to_string())
        .part("file", mrpack_part(file_path).await?);
    if !version.trim().is_empty() {
        form = form.text("version", version.trim().to_string());
    }
    if !changelog.trim().is_empty() {
        form = form.text("changelog", changelog.trim().to_string());
    }
    if let Some(mb) = min_ram_mb {
        form = form.text("minRamMb", mb.to_string());
    }
    if let Some(blog) = boosty_blog.filter(|b| !b.trim().is_empty()) {
        form = form.text("boostyBlog", blog.trim().to_string());
    }
    if let Some(m) = meta {
        if !m.is_null() {
            form = form.text("meta", serde_json::to_string(&m).unwrap_or_default());
        }
    }
    if let Some(url) = icon_url.filter(|u| !u.trim().is_empty()) {
        form = form.text("iconUrl", url.trim().to_string());
    }
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/upload");
    let resp = client
        .post(&url)
        .bearer_auth(access_token)
        .multipart(form)
        .send()
        .await
        .context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("Mono: {}", api_error(&text)));
    }
    serde_json::from_str::<MonoPackPublic>(&text).context("Некорректный ответ сервера Mono")
}

/// Сборка в каталоге Mono (GET /packs, GET /packs/mine).
/// Сериализуется в TS в camelCase; alias принимает snake_case бэкенда.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackCatalog {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, alias = "author_user_id")]
    pub author_user_id: Option<String>,
    #[serde(default, alias = "author_name")]
    pub author_name: Option<String>,
    #[serde(default, alias = "icon_url")]
    pub icon_url: Option<String>,
    #[serde(default, alias = "min_ram_mb")]
    pub min_ram_mb: Option<i64>,
    #[serde(default, alias = "boosty_blog")]
    pub boosty_blog: Option<String>,
    #[serde(default)]
    pub meta: Option<Value>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub size: i64,
    #[serde(default, alias = "versions_count")]
    pub versions_count: i64,
    #[serde(default)]
    pub likes: i64,
    #[serde(default)]
    pub dislikes: i64,
    #[serde(default)]
    pub rating: f64,
    #[serde(default, alias = "created_at")]
    pub created_at: String,
}

/// Публичная версия сборки Mono (GET /packs/{id}/versions и ответы POST).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackVersionPublic {
    pub id: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub changelog: String,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub sha1: String,
    #[serde(default)]
    pub sha512: String,
    #[serde(default, alias = "created_at")]
    pub created_at: String,
}

/// Запись новостей над сборкой (или глобальных новостей) на бэкенде Mono.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackNewsPublic {
    pub id: String,
    #[serde(default, alias = "pack_id")]
    pub pack_id: Option<String>,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default, alias = "created_at")]
    pub created_at: String,
}

/// Деталь сборки Mono (GET /packs/{id}, PUT /packs/{id}).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackDetail {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, alias = "author_user_id")]
    pub author_user_id: Option<String>,
    #[serde(default, alias = "author_name")]
    pub author_name: Option<String>,
    #[serde(default, alias = "icon_url")]
    pub icon_url: Option<String>,
    #[serde(default, alias = "min_ram_mb")]
    pub min_ram_mb: Option<i64>,
    #[serde(default, alias = "boosty_blog")]
    pub boosty_blog: Option<String>,
    #[serde(default)]
    pub meta: Option<Value>,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub likes: i64,
    #[serde(default)]
    pub dislikes: i64,
    #[serde(default, alias = "created_at")]
    pub created_at: String,
    #[serde(default)]
    pub versions: Vec<PackVersionPublic>,
    #[serde(default)]
    pub news: Vec<PackNewsPublic>,
    #[serde(default, alias = "my_rating")]
    pub my_rating: Option<i64>,
}

/// Каталог сборок Mono (публичный).
pub async fn mono_pack_catalog(client: &reqwest::Client) -> Result<Vec<PackCatalog>> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs");
    let resp = client
        .get(&url)
        .send()
        .await
        .context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("Mono: {}", api_error(&text)));
    }
    serde_json::from_str::<Vec<PackCatalog>>(&text).context("Некорректный ответ сервера Mono")
}

/// Сборки, автором которых является текущий пользователь.
pub async fn mono_pack_mine(
    client: &reqwest::Client,
    access_token: &str,
) -> Result<Vec<PackCatalog>> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/mine");
    let resp = client
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("Mono: {}", api_error(&text)));
    }
    serde_json::from_str::<Vec<PackCatalog>>(&text).context("Некорректный ответ сервера Mono")
}

/// Новости бэкенда Mono (глобальные и по сборкам, свежие сверху).
pub async fn mono_pack_news(client: &reqwest::Client) -> Result<Vec<PackNewsPublic>> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/news");
    let resp = client
        .get(&url)
        .send()
        .await
        .context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("Mono: {}", api_error(&text)));
    }
    serde_json::from_str::<Vec<PackNewsPublic>>(&text).context("Некорректный ответ сервера Mono")
}

/// Деталь сборки Mono; пустой access_token — без авторизации.
/// Находит id сборки на бэкенде по URL (packs.url или URL любой версии).
pub async fn mono_pack_id_by_url(
    client: &reqwest::Client,
    url: &str,
) -> Result<Option<String>> {
    let base = crate::config::backend_url();
    let base = base.trim_end_matches('/');
    let resp = client
        .get(format!("{base}/packs/by-url"))
        .query(&[("url", url)])
        .send()
        .await
        .context("Не удалось связаться с сервером Mono")?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("Mono: {}", api_error(&text)));
    }
    let v: Value = serde_json::from_str(&text).context("Некорректный ответ сервера Mono")?;
    Ok(v["id"].as_str().map(|s| s.to_string()))
}

pub async fn mono_pack_detail(
    client: &reqwest::Client,
    access_token: &str,
    id: &str,
) -> Result<PackDetail> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{id}");
    let mut req = client.get(&url);
    if !access_token.is_empty() {
        req = req.bearer_auth(access_token);
    }
    let resp = req
        .send()
        .await
        .context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("Mono: {}", api_error(&text)));
    }
    serde_json::from_str::<PackDetail>(&text).context("Некорректный ответ сервера Mono")
}

/// Частичное обновление описания сборки (PUT /packs/{id}, COALESCE).
pub async fn mono_pack_update(
    client: &reqwest::Client,
    access_token: &str,
    id: &str,
    body: Value,
) -> Result<PackDetail> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{id}");
    let resp = client
        .put(&url)
        .bearer_auth(access_token)
        .json(&body)
        .send()
        .await
        .context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("Mono: {}", api_error(&text)));
    }
    serde_json::from_str::<PackDetail>(&text).context("Некорректный ответ сервера Mono")
}

/// Удаляет сборку с бэкенда и storage (DELETE /packs/{id}, 204).
pub async fn mono_pack_delete(
    client: &reqwest::Client,
    access_token: &str,
    id: &str,
) -> Result<()> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{id}");
    let resp = client
        .delete(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    if status.is_success() {
        return Ok(());
    }
    let text = resp.text().await.unwrap_or_default();
    Err(anyhow!("Mono: {}", api_error(&text)))
}

/// Загружает новую версию .mrpack для сборки (multipart POST /packs/{id}/versions).
pub async fn mono_pack_add_version(
    client: &reqwest::Client,
    access_token: &str,
    id: &str,
    file_path: &str,
    version: &str,
    changelog: &str,
) -> Result<PackVersionPublic> {
    let form = reqwest::multipart::Form::new()
        .text("version", version.to_string())
        .text("changelog", changelog.to_string())
        .part("file", mrpack_part(file_path).await?);
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{id}/versions");
    let resp = client
        .post(&url)
        .bearer_auth(access_token)
        .multipart(form)
        .send()
        .await
        .context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("Mono: {}", api_error(&text)));
    }
    serde_json::from_str::<PackVersionPublic>(&text).context("Некорректный ответ сервера Mono")
}

/// Загружает скриншот сборки (multipart POST /packs/{id}/screenshots).
/// Бэкенд кладёт файл на storage и дописывает URL в meta.screenshots[].
pub async fn mono_pack_upload_screenshot(
    client: &reqwest::Client,
    access_token: &str,
    id: &str,
    file_path: &str,
    caption: &str,
) -> Result<Value> {
    let bytes = tokio::fs::read(file_path)
        .await
        .context("Не удалось прочитать файл скриншота")?;
    if bytes.is_empty() {
        return Err(anyhow!("Файл скриншота пуст"));
    }
    let ext = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "application/octet-stream",
    };
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("screenshot.png")
        .to_string();
    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name(file_name)
        .mime_str(mime)
        .context("Ошибка multipart")?;
    let mut form = reqwest::multipart::Form::new().part("file", part);
    if !caption.trim().is_empty() {
        form = form.text("caption", caption.trim().to_string());
    }
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{id}/screenshots");
    let resp = client
        .post(&url)
        .bearer_auth(access_token)
        .multipart(form)
        .send()
        .await
        .context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("Mono: {}", api_error(&text)));
    }
    serde_json::from_str::<Value>(&text).context("Некорректный ответ сервера Mono")
}

/// Удаляет скриншот по индексу (DELETE /packs/{id}/screenshots/{index}).
pub async fn mono_pack_delete_screenshot(
    client: &reqwest::Client,
    access_token: &str,
    id: &str,
    index: usize,
) -> Result<Value> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{id}/screenshots/{index}");
    let resp = client
        .delete(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("Mono: {}", api_error(&text)));
    }
    serde_json::from_str::<Value>(&text).context("Некорректный ответ сервера Mono")
}

/// Удаляет версию сборки (DELETE /packs/{id}/versions/{version_id}, 204).
pub async fn mono_pack_delete_version(
    client: &reqwest::Client,
    access_token: &str,
    id: &str,
    version_id: &str,
) -> Result<()> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{id}/versions/{version_id}");
    let resp = client
        .delete(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    if status.is_success() {
        return Ok(());
    }
    let text = resp.text().await.unwrap_or_default();
    Err(anyhow!("Mono: {}", api_error(&text)))
}

/// Добавляет новость к сборке (POST /packs/{id}/news).
pub async fn mono_pack_add_news(
    client: &reqwest::Client,
    access_token: &str,
    id: &str,
    kind: &str,
    title: &str,
    body: &str,
) -> Result<PackNewsPublic> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{id}/news");
    let resp = client
        .post(&url)
        .bearer_auth(access_token)
        .json(&json!({ "kind": kind, "title": title, "body": body }))
        .send()
        .await
        .context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("Mono: {}", api_error(&text)));
    }
    serde_json::from_str::<PackNewsPublic>(&text).context("Некорректный ответ сервера Mono")
}

/// Удаляет новость сборки (DELETE /packs/{id}/news/{news_id}, 204).
pub async fn mono_pack_delete_news(
    client: &reqwest::Client,
    access_token: &str,
    id: &str,
    news_id: &str,
) -> Result<()> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{id}/news/{news_id}");
    let resp = client
        .delete(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    if status.is_success() {
        return Ok(());
    }
    let text = resp.text().await.unwrap_or_default();
    Err(anyhow!("Mono: {}", api_error(&text)))
}

/// Оценивает сборку (POST /packs/{id}/rate), возвращает {likes, dislikes, rating, myRating}.
pub async fn mono_pack_rate(
    client: &reqwest::Client,
    access_token: &str,
    id: &str,
    value: i64,
) -> Result<Value> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{id}/rate");
    let resp = client
        .post(&url)
        .bearer_auth(access_token)
        .json(&json!({ "value": value }))
        .send()
        .await
        .context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("Mono: {}", api_error(&text)));
    }
    serde_json::from_str::<Value>(&text).context("Некорректный ответ сервера Mono")
}

// ==== Комментарии ====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonoUserPublic {
    pub id: String,
    pub username: String,
    #[serde(default, alias = "display_name")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentPublic {
    pub id: String,
    #[serde(alias = "pack_id")]
    pub pack_id: String,
    #[serde(alias = "user_id")]
    pub user_id: String,
    pub user: MonoUserPublic,
    #[serde(alias = "parent_id")]
    pub parent_id: Option<String>,
    pub body: String,
    pub likes: i64,
    pub dislikes: i64,
    #[serde(alias = "my_rating")]
    pub my_rating: Option<i64>,
    #[serde(alias = "created_at")]
    pub created_at: String,
    #[serde(alias = "updated_at")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentWithReplies {
    #[serde(flatten)]
    pub comment: CommentPublic,
    pub replies: Vec<CommentWithReplies>,
}

pub async fn mono_list_comments(client: &reqwest::Client, pack_id: &str) -> Result<Vec<CommentWithReplies>> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{pack_id}/comments");
    let resp = client.get(&url).send().await.context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() { return Err(anyhow!("Mono: {}", api_error(&text))); }
    serde_json::from_str(&text).context("Некорректный ответ сервера Mono")
}

pub async fn mono_create_comment(
    client: &reqwest::Client, access_token: &str, pack_id: &str, body: &str, parent_id: Option<&str>,
) -> Result<CommentPublic> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{pack_id}/comments");
    let mut payload = json!({ "body": body });
    if let Some(pid) = parent_id { payload["parent_id"] = json!(pid); }
    let resp = client.post(&url).bearer_auth(access_token).json(&payload).send().await.context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() { return Err(anyhow!("Mono: {}", api_error(&text))); }
    serde_json::from_str(&text).context("Некорректный ответ сервера Mono")
}

pub async fn mono_update_comment(
    client: &reqwest::Client, access_token: &str, pack_id: &str, comment_id: &str, body: &str,
) -> Result<CommentPublic> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{pack_id}/comments/{comment_id}");
    let resp = client.put(&url).bearer_auth(access_token).json(&json!({ "body": body })).send().await.context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() { return Err(anyhow!("Mono: {}", api_error(&text))); }
    serde_json::from_str(&text).context("Некорректный ответ сервера Mono")
}

pub async fn mono_delete_comment(
    client: &reqwest::Client, access_token: &str, pack_id: &str, comment_id: &str,
) -> Result<()> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{pack_id}/comments/{comment_id}");
    let resp = client.delete(&url).bearer_auth(access_token).send().await.context("Не удалось связаться с сервером Mono")?;
    if resp.status().is_success() { return Ok(()); }
    let text = resp.text().await.unwrap_or_default();
    Err(anyhow!("Mono: {}", api_error(&text)))
}

pub async fn mono_rate_comment(
    client: &reqwest::Client, access_token: &str, pack_id: &str, comment_id: &str, value: i64,
) -> Result<Value> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{pack_id}/comments/{comment_id}/rate");
    let resp = client.post(&url).bearer_auth(access_token).json(&json!({ "value": value })).send().await.context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() { return Err(anyhow!("Mono: {}", api_error(&text))); }
    serde_json::from_str(&text).context("Некорректный ответ сервера Mono")
}

// ==== Профили ====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePublic {
    pub user: MonoUserPublic,
    pub bio: String,
    #[serde(default, alias = "avatar_url")]
    pub avatar_url: Option<String>,
    #[serde(default, alias = "packs_count")]
    pub packs_count: i64,
    #[serde(default, alias = "comments_count")]
    pub comments_count: i64,
    #[serde(default, alias = "joined_at")]
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPackSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default, alias = "icon_url")]
    pub icon_url: Option<String>,
    pub version: Option<String>,
    pub likes: i64,
    pub dislikes: i64,
    #[serde(default, alias = "versions_count")]
    pub versions_count: i64,
    #[serde(alias = "created_at")]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserCommentSummary {
    pub id: String,
    #[serde(alias = "pack_id")]
    pub pack_id: String,
    #[serde(alias = "pack_name")]
    pub pack_name: String,
    pub body: String,
    #[serde(alias = "created_at")]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDetail {
    pub profile: ProfilePublic,
    pub packs: Vec<UserPackSummary>,
    pub comments: Vec<UserCommentSummary>,
}

pub async fn mono_get_profile(client: &reqwest::Client, user_id: &str) -> Result<ProfilePublic> {
    let base = crate::config::backend_url();
    let url = format!("{base}/profiles/{user_id}");
    let resp = client.get(&url).send().await.context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() { return Err(anyhow!("Mono: {}", api_error(&text))); }
    serde_json::from_str(&text).context("Некорректный ответ сервера Mono")
}

pub async fn mono_get_profile_full(client: &reqwest::Client, user_id: &str) -> Result<ProfileDetail> {
    let base = crate::config::backend_url();
    let url = format!("{base}/profiles/{user_id}/full");
    let resp = client.get(&url).send().await.context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() { return Err(anyhow!("Mono: {}", api_error(&text))); }
    serde_json::from_str(&text).context("Некорректный ответ сервера Mono")
}

pub async fn mono_update_profile(
    client: &reqwest::Client, access_token: &str, bio: Option<&str>, avatar_url: Option<&str>,
) -> Result<ProfilePublic> {
    let base = crate::config::backend_url();
    let url = format!("{base}/profiles/me");
    let mut payload = json!({});
    if let Some(b) = bio { payload["bio"] = json!(b); }
    if let Some(a) = avatar_url { payload["avatar_url"] = json!(a); }
    let resp = client.put(&url).bearer_auth(access_token).json(&payload).send().await.context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() { return Err(anyhow!("Mono: {}", api_error(&text))); }
    serde_json::from_str(&text).context("Некорректный ответ сервера Mono")
}

// ==== Сканер модов ====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub id: String,
    #[serde(alias = "file_name")]
    pub file_name: String,
    pub sha256: String,
    pub safe: bool,
    #[serde(alias = "scan_result")]
    pub scan_result: String,
    #[serde(alias = "dangerous_classes")]
    pub dangerous_classes: Option<String>,
    pub cached: bool,
}

pub async fn mono_scan_mod(
    client: &reqwest::Client, access_token: &str, file_path: &str,
) -> Result<ScanResult> {
    let form = reqwest::multipart::Form::new()
        .part("file", mrpack_part(file_path).await?);
    let base = crate::config::backend_url();
    let url = format!("{base}/scanner/scan");
    let resp = client.post(&url).bearer_auth(access_token).multipart(form).send().await.context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() { return Err(anyhow!("Mono: {}", api_error(&text))); }
    serde_json::from_str(&text).context("Некорректный ответ сервера Mono")
}

pub async fn mono_check_hash(client: &reqwest::Client, sha256: &str) -> Result<ScanResult> {
    let base = crate::config::backend_url();
    let url = format!("{base}/scanner/check");
    let resp = client.post(&url).json(&json!({ "sha256": sha256 })).send().await.context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() { return Err(anyhow!("Mono: {}", api_error(&text))); }
    serde_json::from_str(&text).context("Некорректный ответ сервера Mono")
}

/// Сообщает бэкенду событие по версии сборки (установка/запуск) — best-effort:
/// ошибки игнорируются, ответ не важен. kind: "install" | "launch".
pub async fn mono_report_event(
    client: &reqwest::Client,
    pack_id: &str,
    version: &str,
    kind: &str,
) {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{pack_id}/versions/{version}/event");
    let _ = client
        .post(&url)
        .json(&json!({ "kind": kind }))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;
}

// ==== Playtime + Library sync (fire-and-forget) ====

/// Отправить суммарное время игры по сборке на бэкенд (POST /auth/playtime).
/// Идемпотентно: бэкенд хранит MAX(old, new). Тихо проглатывает ошибки.
pub async fn mono_report_playtime(client: &reqwest::Client, pack_id: &str, seconds: u64) {
    let Some(profile) = load_mono_profile().ok().flatten() else {
        return;
    };
    let base = crate::config::backend_url();
    let url = format!("{base}/auth/playtime");
    let _ = client
        .post(&url)
        .bearer_auth(&profile.access_token)
        .json(&json!({ "packId": pack_id, "seconds": seconds as i64 }))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;
}

/// Синхронизировать сборку в библиотеку на бэкенде (POST /auth/library).
/// Вызывается при добавлении сборки. Тихо проглатывает ошибки.
pub async fn mono_sync_library(
    client: &reqwest::Client,
    pack_id: &str,
    pack_name: &str,
    pack_url: &str,
    kind: &str,
    boosty_blog: Option<&str>,
    min_ram_mb: Option<i32>,
) {
    let Some(profile) = load_mono_profile().ok().flatten() else {
        return;
    };
    let base = crate::config::backend_url();
    let url = format!("{base}/auth/library");
    let mut body = json!({
        "packId": pack_id,
        "packName": pack_name,
        "packUrl": pack_url,
        "kind": kind,
    });
    if let Some(blog) = boosty_blog {
        body["boostyBlog"] = json!(blog);
    }
    if let Some(ram) = min_ram_mb {
        body["minRamMb"] = json!(ram);
    }
    let _ = client
        .post(&url)
        .bearer_auth(&profile.access_token)
        .json(&body)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;
}

/// Удалить сборку из библиотеки на бэкенде (DELETE /auth/library/{pack_id}).
/// Тихо проглатывает ошибки.
pub async fn mono_remove_library(client: &reqwest::Client, pack_id: &str) {
    let Some(profile) = load_mono_profile().ok().flatten() else {
        return;
    };
    let base = crate::config::backend_url();
    let url = format!("{base}/auth/library/{pack_id}");
    let _ = client
        .delete(&url)
        .bearer_auth(&profile.access_token)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;
}

// ==== Соавторы ====
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaboratorPublic {
    pub id: String,
    pub user: MonoUserPublic,
    #[serde(alias = "perm_edit_meta")]
    pub perm_edit_meta: bool,
    #[serde(alias = "perm_manage_versions")]
    pub perm_manage_versions: bool,
    #[serde(alias = "perm_manage_news")]
    pub perm_manage_news: bool,
}

pub async fn mono_list_collaborators(
    client: &reqwest::Client, access_token: &str, pack_id: &str,
) -> Result<Vec<CollaboratorPublic>> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{pack_id}/collaborators");
    let mut req = client.get(&url);
    if !access_token.is_empty() { req = req.bearer_auth(access_token); }
    let resp = req.send().await.context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() { return Err(anyhow!("Mono: {}", api_error(&text))); }
    serde_json::from_str(&text).context("Некорректный ответ сервера Mono")
}

pub async fn mono_add_collaborator(
    client: &reqwest::Client, access_token: &str, pack_id: &str,
    username: &str, perm_edit_meta: bool, perm_manage_versions: bool, perm_manage_news: bool,
) -> Result<CollaboratorPublic> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{pack_id}/collaborators");
    let resp = client.post(&url).bearer_auth(access_token).json(&json!({
        "username": username,
        "perm_edit_meta": perm_edit_meta,
        "perm_manage_versions": perm_manage_versions,
        "perm_manage_news": perm_manage_news,
    })).send().await.context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() { return Err(anyhow!("Mono: {}", api_error(&text))); }
    serde_json::from_str(&text).context("Некорректный ответ сервера Mono")
}

pub async fn mono_update_collaborator(
    client: &reqwest::Client, access_token: &str, pack_id: &str, collab_id: &str,
    perm_edit_meta: Option<bool>, perm_manage_versions: Option<bool>, perm_manage_news: Option<bool>,
) -> Result<CollaboratorPublic> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{pack_id}/collaborators/{collab_id}");
    let mut payload = json!({});
    if let Some(v) = perm_edit_meta { payload["perm_edit_meta"] = json!(v); }
    if let Some(v) = perm_manage_versions { payload["perm_manage_versions"] = json!(v); }
    if let Some(v) = perm_manage_news { payload["perm_manage_news"] = json!(v); }
    let resp = client.put(&url).bearer_auth(access_token).json(&payload).send().await.context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() { return Err(anyhow!("Mono: {}", api_error(&text))); }
    serde_json::from_str(&text).context("Некорректный ответ сервера Mono")
}

pub async fn mono_remove_collaborator(
    client: &reqwest::Client, access_token: &str, pack_id: &str, collab_id: &str,
) -> Result<()> {
    let base = crate::config::backend_url();
    let url = format!("{base}/packs/{pack_id}/collaborators/{collab_id}");
    let resp = client.delete(&url).bearer_auth(access_token).send().await.context("Не удалось связаться с сервером Mono")?;
    if resp.status().is_success() { return Ok(()); }
    let text = resp.text().await.unwrap_or_default();
    Err(anyhow!("Mono: {}", api_error(&text)))
}

// ==== Админ ====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUser {
    pub id: String,
    pub username: String,
    #[serde(default, alias = "display_name")]
    pub display_name: Option<String>,
    pub email: Option<String>,
    #[serde(default, alias = "email_confirmed")]
    pub email_confirmed: bool,
    pub role: String,
    pub banned: bool,
    #[serde(default, alias = "ban_reason")]
    pub ban_reason: Option<String>,
    #[serde(default, alias = "created_at")]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminPack {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default, alias = "author_user_id")]
    pub author_user_id: Option<String>,
    #[serde(default, alias = "author_name")]
    pub author_name: Option<String>,
    pub likes: i64,
    pub dislikes: i64,
    #[serde(default, alias = "versions_count")]
    pub versions_count: i64,
    #[serde(default, alias = "created_at")]
    pub created_at: String,
}

pub async fn mono_admin_list_users(client: &reqwest::Client, access_token: &str) -> Result<Vec<AdminUser>> {
    let base = crate::config::backend_url();
    let url = format!("{base}/admin/users");
    let resp = client.get(&url).bearer_auth(access_token).send().await.context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() { return Err(anyhow!("Mono: {}", api_error(&text))); }
    serde_json::from_str(&text).context("Некорректный ответ сервера Mono")
}

pub async fn mono_admin_list_packs(client: &reqwest::Client, access_token: &str) -> Result<Vec<AdminPack>> {
    let base = crate::config::backend_url();
    let url = format!("{base}/admin/packs");
    let resp = client.get(&url).bearer_auth(access_token).send().await.context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() { return Err(anyhow!("Mono: {}", api_error(&text))); }
    serde_json::from_str(&text).context("Некорректный ответ сервера Mono")
}

pub async fn mono_admin_ban_user(client: &reqwest::Client, access_token: &str, user_id: &str, reason: Option<&str>) -> Result<()> {
    let base = crate::config::backend_url();
    let url = format!("{base}/admin/users/{user_id}/ban");
    let resp = client.put(&url).bearer_auth(access_token).json(&json!({ "reason": reason })).send().await.context("Не удалось связаться с сервером Mono")?;
    if resp.status().is_success() { return Ok(()); }
    let text = resp.text().await.unwrap_or_default();
    Err(anyhow!("Mono: {}", api_error(&text)))
}

pub async fn mono_admin_unban_user(client: &reqwest::Client, access_token: &str, user_id: &str) -> Result<()> {
    let base = crate::config::backend_url();
    let url = format!("{base}/admin/users/{user_id}/unban");
    let resp = client.put(&url).bearer_auth(access_token).send().await.context("Не удалось связаться с сервером Mono")?;
    if resp.status().is_success() { return Ok(()); }
    let text = resp.text().await.unwrap_or_default();
    Err(anyhow!("Mono: {}", api_error(&text)))
}

pub async fn mono_admin_delete_user(client: &reqwest::Client, access_token: &str, user_id: &str) -> Result<()> {
    let base = crate::config::backend_url();
    let url = format!("{base}/admin/users/{user_id}");
    let resp = client.delete(&url).bearer_auth(access_token).send().await.context("Не удалось связаться с сервером Mono")?;
    if resp.status().is_success() { return Ok(()); }
    let text = resp.text().await.unwrap_or_default();
    Err(anyhow!("Mono: {}", api_error(&text)))
}

pub async fn mono_admin_delete_pack(client: &reqwest::Client, access_token: &str, pack_id: &str) -> Result<()> {
    let base = crate::config::backend_url();
    let url = format!("{base}/admin/packs/{pack_id}");
    let resp = client.delete(&url).bearer_auth(access_token).send().await.context("Не удалось связаться с сервером Mono")?;
    if resp.status().is_success() { return Ok(()); }
    let text = resp.text().await.unwrap_or_default();
    Err(anyhow!("Mono: {}", api_error(&text)))
}

pub async fn mono_admin_delete_comment(client: &reqwest::Client, access_token: &str, comment_id: &str) -> Result<()> {
    let base = crate::config::backend_url();
    let url = format!("{base}/admin/comments/{comment_id}");
    let resp = client.delete(&url).bearer_auth(access_token).send().await.context("Не удалось связаться с сервером Mono")?;
    if resp.status().is_success() { return Ok(()); }
    let text = resp.text().await.unwrap_or_default();
    Err(anyhow!("Mono: {}", api_error(&text)))
}

pub async fn mono_admin_set_role(client: &reqwest::Client, access_token: &str, user_id: &str, role: &str) -> Result<()> {
    let base = crate::config::backend_url();
    let url = format!("{base}/admin/users/{user_id}/role");
    let resp = client.put(&url).bearer_auth(access_token).json(&json!({ "role": role })).send().await.context("Не удалось связаться с сервером Mono")?;
    if resp.status().is_success() { return Ok(()); }
    let text = resp.text().await.unwrap_or_default();
    Err(anyhow!("Mono: {}", api_error(&text)))
}

// ==== Auth v2 ====

pub async fn mono_forgot_password(client: &reqwest::Client, email: &str) -> Result<()> {
    let base = crate::config::backend_url();
    let url = format!("{base}/auth/forgot-password");
    let resp = client.post(&url).json(&json!({ "email": email })).send().await.context("Не удалось связаться с сервером Mono")?;
    if resp.status().is_success() { return Ok(()); }
    let text = resp.text().await.unwrap_or_default();
    Err(anyhow!("Mono: {}", api_error(&text)))
}

pub async fn mono_reset_password(client: &reqwest::Client, token: &str, password: &str) -> Result<()> {
    let base = crate::config::backend_url();
    let url = format!("{base}/auth/reset-password");
    let resp = client.post(&url).json(&json!({ "token": token, "password": password })).send().await.context("Не удалось связаться с сервером Mono")?;
    if resp.status().is_success() { return Ok(()); }
    let text = resp.text().await.unwrap_or_default();
    Err(anyhow!("Mono: {}", api_error(&text)))
}

pub async fn mono_confirm_email(client: &reqwest::Client, access_token: &str) -> Result<()> {
    let base = crate::config::backend_url();
    let url = format!("{base}/auth/confirm");
    let resp = client.post(&url).bearer_auth(access_token).send().await.context("Не удалось связаться с сервером Mono")?;
    if resp.status().is_success() { return Ok(()); }
    let text = resp.text().await.unwrap_or_default();
    Err(anyhow!("Mono: {}", api_error(&text)))
}

#[derive(Debug, Deserialize)]
struct MsCodeResp {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MsTokenResp {
    access_token: Option<String>,
    refresh_token: Option<String>,
    token_type: Option<String>,
    expires_in: Option<u64>,
    error: Option<String>,
    error_description: Option<String>,
}

/// Данные для входа: код и страница подтверждения (фаза 1 device code flow).
#[derive(Debug, Clone, Serialize)]
pub struct DeviceCodeInfo {
    pub user_code: String,
    pub verification_uri: String,
    pub device_code: String,
    pub interval: u64,
    pub expires_in: u64,
    /// QR-код страницы подтверждения (SVG) — сканировать с телефона.
    pub qr_svg: String,
}

/// Рендерит QR-код как SVG-строку (без внешних зависимостей на image).
fn qr_svg(text: &str) -> String {
    use qrcode::QrCode;
    let Ok(code) = QrCode::new(text.as_bytes()) else {
        return String::new();
    };
    let cells = code.to_colors();
    let size = code.width();
    let cell = 4usize;
    let border = 2usize;
    let dim = (size + border * 2) * cell;
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {dim} {dim}\" shape-rendering=\"crispEdges\">"
    );
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/>");
    for (i, color) in cells.iter().enumerate() {
        if *color != qrcode::types::Color::Light {
            let x = i % size;
            let y = i / size;
            svg.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{cell}\" height=\"{cell}\" fill=\"#0d1117\"/>",
                (x + border) * cell,
                (y + border) * cell
            ));
        }
    }
    svg.push_str("</svg>");
    svg
}

#[derive(Debug, Deserialize)]
struct XboxResp {
    #[serde(rename = "Token")]
    token: String,
}

#[derive(Debug, Deserialize)]
struct MineTokenResp {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct MsUserResp {
    name: String,
    #[serde(rename = "id")]
    uuid: String,
}

const AZURE_CLIENT_ID: &str = "39e3cb28-4aa8-4f9a-a2ac-a5bed7724be5";

/// Client id можно задать без пересборки:
/// 1) файл `<данные лаунчера>/azure-client-id` (одной строкой),
/// 2) либо переменная окружения MONO_AZURE_CLIENT_ID,
/// 3) либо константа AZURE_CLIENT_ID в этом файле.
fn azure_client_id() -> Option<String> {
    let file = read_client_id_file();
    let env = std::env::var("MONO_AZURE_CLIENT_ID").ok();
    for candidate in [file, env].into_iter().flatten() {
        let t = candidate.trim().to_string();
        if !t.is_empty() && t != "CHANGE_ME" {
            return Some(t);
        }
    }
    if AZURE_CLIENT_ID != "CHANGE_ME" {
        return Some(AZURE_CLIENT_ID.to_string());
    }
    None
}

fn read_client_id_file() -> Option<String> {
    let path = config::launcher_root().ok()?.join("azure-client-id");
    std::fs::read_to_string(path).ok()
}

fn require_client_id() -> Result<String> {
    azure_client_id().ok_or_else(|| {
        anyhow!(
            "Microsoft OAuth2 не настроен.\n\
             Зарегистрируйте приложение в Azure (Entra ID): portal.azure.com → App registrations →\n\
             New registration, тип аккаунтов «Personal Microsoft accounts only», включите\n\
             «Allow public client flows» и запишите Application (client) ID одной строкой в файл:\n\
             {}",
            config::launcher_root()
                .map(|p| p.join("azure-client-id").display().to_string())
                .unwrap_or_else(|_| "<данные лаунчера>/azure-client-id".into())
        )
    })
}

/// Фаза 1: запрашиваем device code у Microsoft и возвращаем код для показа.
pub async fn ms_device_code(client: &reqwest::Client) -> Result<DeviceCodeInfo> {
    let client_id = require_client_id()?;

    let code_resp = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
        .form(&[
            ("client_id", client_id.as_str()),
            ("scope", "XboxLive.signin offline_access"),
        ])
        .send()
        .await
        .context("Не удалось связаться с Microsoft")?;
    if !code_resp.status().is_success() {
        let status = code_resp.status();
        let body = code_resp.text().await.unwrap_or_default();
        return Err(anyhow!("Microsoft не выдал device code ({status}): {body}"));
    }
    let code_resp: MsCodeResp = code_resp.json().await.context("Некорректный ответ Microsoft")?;

    Ok(DeviceCodeInfo {
        qr_svg: qr_svg(&code_resp.verification_uri),
        user_code: code_resp.user_code,
        verification_uri: code_resp.verification_uri,
        device_code: code_resp.device_code,
        interval: code_resp.interval,
        expires_in: code_resp.expires_in,
    })
}

/// Фаза 2: поллим токен, а затем проходим цепочку
/// Microsoft → Xbox Live → XSTS → Minecraft → профиль.
pub async fn ms_poll(
    client: &reqwest::Client,
    device_code: &str,
    interval: u64,
    expires_in: u64,
) -> Result<UserSession> {
    // Поллинг Microsoft-токена.
    let client_id = require_client_id()?;
    let mut ms_token: Option<String> = None;
    let mut ms_refresh_tok: Option<String> = None;
    let mut poll_interval = interval.max(5);
    let mut elapsed: u64 = 0;
    while elapsed < expires_in {
        let resp_body = client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("client_id", client_id.as_str()),
                ("device_code", device_code),
            ])
            .send()
            .await?;
        let status = resp_body.status();
        let text = resp_body.text().await.unwrap_or_default();
        let resp: MsTokenResp = serde_json::from_str(&text)
            .with_context(|| format!("Microsoft вернул не-JSON токен ({status}): {text}"))?;

        if let Some(err) = resp.error {
            match err.as_str() {
                "authorization_pending" => (),
                "authorization_declined" => {
                    return Err(anyhow!("Авторизация отклонена"));
                }
                "expired_token" => return Err(anyhow!("Код авторизации истёк")),
                "slow_down" => poll_interval += 5,
                other => return Err(anyhow!("Ошибка OAuth2: {other}")),
            }
            tokio::time::sleep(std::time::Duration::from_secs(poll_interval)).await;
            elapsed += poll_interval;
            continue;
        }

        ms_refresh_tok = resp.refresh_token.clone();
        ms_token = resp.access_token;
        break;
    }
    let ms_token = ms_token.ok_or_else(|| anyhow!("Таймаут авторизации Microsoft"))?;

    // Цепочка XBL → XSTS → Minecraft вынесена в session_from_ms_token.
    let mut session = session_from_ms_token(client, &ms_token).await?;
    if let Some(rt) = ms_refresh_tok {
        session.refresh_token = Some(rt);
    }
    Ok(session)
}

/// Цепочка Microsoft-токен → XBL → XSTS → Minecraft Services → профиль.
async fn session_from_ms_token(client: &reqwest::Client, ms_token: &str) -> Result<UserSession> {
    // Xbox Live: размениваем Microsoft-токен на XBL-токен (RPS-тикет).
    let xbl: XboxResp = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .header("x-xbl-contract-version", "1")
        .header("Accept", "application/json")
        .json(&json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={ms_token}"),
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT",
        }))
        .send()
        .await
        .context("Не удалось связаться с Xbox Live")?
        .error_for_status()
        .context("Xbox Live отклонил токен Microsoft")?
        .json()
        .await?;

    // XSTS: получаем токен для Minecraft.
    let xsts_resp = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .header("x-xbl-contract-version", "1")
        .header("Accept", "application/json")
        .json(&json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbl.token],
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT",
        }))
        .send()
        .await
        .context("Не удалось связаться с XSTS")?;

    let xsts_status = xsts_resp.status();
    let xsts_body: Value = xsts_resp.json().await.context("Некорректный ответ XSTS")?;
    if !xsts_status.is_success() {
        let xerr = xsts_body["XErr"].as_u64();
        let msg = match xerr {
            Some(2148916233) | Some(2148916235) | Some(2148916236) | Some(2148916237) => {
                "К этому Microsoft-аккаунту не привязан аккаунт Xbox Live. \
                 Зарегистрируйте бесплатный аккаунт Xbox на xbox.com"
            }
            Some(2148916238) => "Xbox-аккаунт этого пользователя не достиг 18 лет",
            _ => "Xbox/XSTS не приняли токен (безлицензионный или заблокированный аккаунт)",
        };
        return Err(anyhow!(msg));
    }
    let uhs = xsts_body["DisplayClaims"]["xui"][0]["uhs"]
        .as_str()
        .ok_or_else(|| anyhow!("XSTS не вернул uhs"))?
        .to_string();
    let xsts_token = xsts_body["Token"]
        .as_str()
        .ok_or_else(|| anyhow!("XSTS не вернул токен"))?
        .to_string();

    // Minecraft: обмениваем XSTS-токен на игровой токен.
    let mine_resp = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&json!({
            "identityToken": format!("XBL3.0 x={uhs};{xsts_token}"),
        }))
        .send()
        .await
        .context("Не удалось связаться с Minecraft Services")?;
    if !mine_resp.status().is_success() {
        let status = mine_resp.status();
        let body = mine_resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "Minecraft Services отклонил XSTS-токен ({status}): {body}"
        ));
    }
    let mine: MineTokenResp = mine_resp
        .json()
        .await
        .context("Minecraft Services вернул некорректный ответ")?;

    // Профиль: ник и UUID.
    let profile = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(&mine.access_token)
        .send()
        .await
        .context("Не удалось получить профиль Minecraft")?;
    if profile.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(anyhow!(
            "У этого аккаунта нет лицензии Minecraft Java Edition — купите игру на minecraft.net"
        ));
    }
    let user: MsUserResp = profile
        .error_for_status()
        .context("Minecraft Services отклонил запрос профиля")?
        .json()
        .await?;

    Ok(UserSession {
        username: user.name,
        uuid: Uuid::parse_str(&user.uuid)
            .map(|u| u.to_string())
            .unwrap_or_else(|_| Uuid::new_v3(&Uuid::NAMESPACE_DNS, user.uuid.as_bytes()).to_string()),
        access_token: mine.access_token,
        refresh_token: None,
        user_type: "microsoft".into(),
    })
}

/// Обновляет Microsoft-сессию по refresh_token.
pub async fn ms_refresh(client: &reqwest::Client, refresh_token: &str) -> Result<UserSession> {
    let client_id = require_client_id()?;
    let resp_body = client.post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id.as_str()),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await?;
    let text = resp_body.text().await.unwrap_or_default();
    let resp: MsTokenResp = serde_json::from_str(&text).context("Некорректный ответ Microsoft при обновлении")?;
    if let Some(err) = resp.error {
        return Err(anyhow!("Ошибка OAuth2: {err}"));
    }
    let mut session = session_from_ms_token(
        client,
        resp.access_token.as_deref().ok_or_else(|| anyhow!("Microsoft не вернул access_token"))?,
    )
    .await?;
    if resp.refresh_token.is_some() {
        session.refresh_token = resp.refresh_token.clone();
    }
    Ok(session)
}
/// Вход через Ely.by: device code flow (как у Microsoft), но токен приходит
/// сразу с правами `minecraft_server_session` — его передаём игре напрямую.
///
/// Client id задаётся так же, как для Azure:
/// 1) файл `<данные лаунчера>/ely-client-id` (одной строкой),
/// 2) переменная окружения MONO_ELY_CLIENT_ID,
/// 3) константа ELY_CLIENT_ID ниже.
///
/// Зарегистрировать приложение: account.ely.by → «Создать приложение» (любой тип,
/// device flow обходится без redirect URI), в настройках приложения скопировать clientId.
const ELY_CLIENT_ID: &str = "CHANGE_ME";
const ELY_DEVICE_URL: &str = "https://account.ely.by/api/oauth2/v1/devicecode";
const ELY_TOKEN_URL: &str = "https://account.ely.by/api/oauth2/v1/token";
const ELY_PROFILE_URL: &str = "https://account.ely.by/api/mojang/services/minecraft/profile";
const ELY_SCOPES: &str = "account_info offline_access minecraft_server_session";

fn ely_client_id_from_cfg() -> Option<String> {
    let file = std::fs::read_to_string(config::launcher_root().ok()?.join("ely-client-id")).ok();
    let env = std::env::var("MONO_ELY_CLIENT_ID").ok();
    for candidate in [file, env].into_iter().flatten() {
        let t = candidate.trim().to_string();
        if !t.is_empty() && t != "CHANGE_ME" {
            return Some(t);
        }
    }
    if ELY_CLIENT_ID != "CHANGE_ME" {
        return Some(ELY_CLIENT_ID.to_string());
    }
    None
}

fn require_ely_client_id() -> Result<String> {
    ely_client_id_from_cfg().ok_or_else(|| {
        anyhow!(
            "Ely.by вход не настроен.\n\
             Зарегистрируйте приложение на account.ely.by (профиль → «Приложения» → создать),\n\
             затем запишите его clientId одной строкой в файл:\n\
             {}",
            config::launcher_root()
                .map(|p| p.join("ely-client-id").display().to_string())
                .unwrap_or_else(|_| "<данные лаунчера>/ely-client-id".into())
        )
    })
}

/// Фаза 1 Ely.by: запрашиваем device code (формат ответа как у Microsoft).
pub async fn ely_device_code(client: &reqwest::Client) -> Result<DeviceCodeInfo> {
    let client_id = require_ely_client_id()?;
    let code_resp: MsCodeResp = client
        .post(ELY_DEVICE_URL)
        .form(&[("client_id", client_id.as_str()), ("scope", ELY_SCOPES)])
        .send()
        .await
        .context("Не удалось связаться с Ely.by")?
        .error_for_status()
        .context("Ely.by не выдал device code")?
        .json()
        .await?;
    Ok(DeviceCodeInfo {
        qr_svg: qr_svg(&code_resp.verification_uri),
        user_code: code_resp.user_code,
        verification_uri: code_resp.verification_uri,
        device_code: code_resp.device_code,
        interval: code_resp.interval,
        expires_in: code_resp.expires_in,
    })
}

/// Фаза 2 Ely.by: поллим токен и получаем профиль Minecraft.
pub async fn ely_poll(
    client: &reqwest::Client,
    device_code: &str,
    interval: u64,
    expires_in: u64,
) -> Result<UserSession> {
    let client_id = require_ely_client_id()?;
    let mut access_token: Option<String> = None;
    let mut poll_interval = interval.max(5);
    let mut elapsed: u64 = 0;
    while elapsed < expires_in {
        let resp_body = client
            .post(ELY_TOKEN_URL)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("client_id", client_id.as_str()),
                ("device_code", device_code),
            ])
            .send()
            .await?;
        let status = resp_body.status();
        let text = resp_body.text().await.unwrap_or_default();
        let resp: MsTokenResp = serde_json::from_str(&text)
            .with_context(|| format!("Ely.by вернул не-JSON токен ({status}): {text}"))?;
        if let Some(err) = resp.error {
            match err.as_str() {
                "authorization_pending" => (),
                "authorization_declined" => {
                    return Err(anyhow!("Авторизация отклонена"));
                }
                "expired_token" => return Err(anyhow!("Код авторизации истёк")),
                "slow_down" => poll_interval += 5,
                other => return Err(anyhow!("Ошибка OAuth2: {other}")),
            }
            tokio::time::sleep(std::time::Duration::from_secs(poll_interval)).await;
            elapsed += poll_interval;
            continue;
        }
        access_token = resp.access_token;
        break;
    }
    let access_token = access_token.ok_or_else(|| anyhow!("Таймаут авторизации Ely.by"))?;

    // Профиль Minecraft (Mojang-совместимый формат: id без дефисов, name).
    let profile = client
        .get(ELY_PROFILE_URL)
        .bearer_auth(&access_token)
        .send()
        .await
        .context("Не удалось получить профиль Ely.by")?;
    if profile.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(anyhow!("У Ely.by-аккаунта нет профиля Minecraft"));
    }
    let user: MsUserResp = profile
        .error_for_status()
        .context("Ely.by отклонил запрос профиля")?
        .json()
        .await?;
    let uuid = Uuid::parse_str(&user.uuid)
        .unwrap_or_else(|_| Uuid::new_v3(&Uuid::NAMESPACE_DNS, user.uuid.as_bytes()))
        .to_string();
    Ok(UserSession {
        username: user.name,
        uuid,
        access_token,
        refresh_token: None,
        user_type: "ely".into(),
    })
}

fn session_file() -> Result<PathBuf> {
    Ok(config::launcher_root()?.join("session.json"))
}

/// Сохраняет последнюю сессию, чтобы не логиниться каждый раз.
pub fn save_session(session: &UserSession) -> Result<()> {
    let path = session_file()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_vec_pretty(session)?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn load_session() -> Result<Option<UserSession>> {
    let path = session_file()?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&raw)?))
}

/// Аккаунт в списке «несколько аккаунтов» (accounts.json). id = uuid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountEntry {
    pub id: String,
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub user_type: String,
    /// Microsoft OAuth2 refresh_token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

impl From<&UserSession> for AccountEntry {
    fn from(s: &UserSession) -> Self {
        AccountEntry {
            id: s.uuid.clone(),
            username: s.username.clone(),
            uuid: s.uuid.clone(),
            access_token: s.access_token.clone(),
            refresh_token: s.refresh_token.clone(),
            user_type: s.user_type.clone(),
        }
    }
}

impl AccountEntry {
    pub fn to_session(&self) -> UserSession {
        UserSession {
            username: self.username.clone(),
            uuid: self.uuid.clone(),
            access_token: self.access_token.clone(),
            refresh_token: self.refresh_token.clone(),
            user_type: self.user_type.clone(),
        }
    }
}

/// Список сохранённых аккаунтов + какой из них активный.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Accounts {
    #[serde(default)]
    pub active: Option<String>,
    #[serde(default)]
    pub list: Vec<AccountEntry>,
}

fn accounts_file() -> Result<PathBuf> {
    Ok(config::launcher_root()?.join("accounts.json"))
}

pub fn load_accounts() -> Accounts {
    accounts_file()
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub fn save_accounts(accounts: &Accounts) -> Result<()> {
    let path = accounts_file()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_vec_pretty(accounts)?)?;
    Ok(())
}

/// Добавляет/обновляет аккаунт по сессии и делает его активным.
/// Возвращает None, если аккаунт уже есть и ничего не изменилось.
pub fn upsert_account(session: &UserSession) -> Result<()> {
    let mut accounts = load_accounts();
    let entry = AccountEntry::from(session);
    if let Some(existing) = accounts.list.iter_mut().find(|a| a.id == entry.id) {
        *existing = entry.clone();
    } else {
        accounts.list.push(entry.clone());
    }
    accounts.active = Some(entry.id);
    save_accounts(&accounts)
}

/// Переключает активный аккаунт; возвращает его сессию (для session.json).
pub fn switch_account(id: &str) -> Result<Option<UserSession>> {
    let mut accounts = load_accounts();
    let Some(entry) = accounts.list.iter().find(|a| a.id == id) else {
        return Err(anyhow!("Аккаунт не найден"));
    };
    accounts.active = Some(id.to_string());
    save_accounts(&accounts)?;
    let session = entry.to_session();
    save_session(&session)?;
    Ok(Some(session))
}

/// Удаляет аккаунт. Если удалили активный — активным становится первый
/// оставшийся (или выход из аккаунта). Возвращает сессию нового активного.
pub fn remove_account(id: &str) -> Result<Option<UserSession>> {
    let mut accounts = load_accounts();
    accounts.list.retain(|a| a.id != id);
    if accounts.active.as_deref() == Some(id) {
        accounts.active = accounts.list.first().map(|a| a.id.clone());
        match &accounts.active {
            Some(next_id) => {
                save_accounts(&accounts)?;
                let session = switch_account(next_id)?;
                return Ok(session);
            }
            None => {
                // Аккаунтов не осталось — выходим полностью.
                save_accounts(&accounts)?;
                let path = session_file()?;
                let _ = std::fs::remove_file(path);
                return Ok(None);
            }
        }
    }
    save_accounts(&accounts)?;
    Ok(None)
}

#[cfg(test)]
mod mono_api_tests {
    use super::*;

    /// Бэкенд отдаёт snake_case; лаунчер должен парсить это и наружу в TS отдавать camelCase.
    #[test]
    fn comments_parse_backend_snake_case() {
        let json = r#"[{"id":"a","pack_id":"p","user_id":"u","user":{"id":"u","username":"niorio","display_name":null},"parent_id":null,"body":"hi","likes":1,"dislikes":0,"my_rating":null,"created_at":"2026-08-22T12:24:22.969131Z","updated_at":"2026-08-22T12:24:22.969131Z","replies":[]}]"#;
        let list: Vec<CommentWithReplies> = serde_json::from_str(json).unwrap();
        assert_eq!(list[0].comment.pack_id, "p");
        assert_eq!(list[0].comment.user.username, "niorio");
        let out = serde_json::to_value(&list).unwrap();
        assert!(out[0].get("packId").is_some(), "TS ждёт camelCase");
        assert!(out[0].get("pack_id").is_none());
        assert!(out[0].get("replies").is_some());
    }

    #[test]
    fn catalog_parses_backend_snake_case() {
        let json = r#"{"id":"1","name":"n","author_user_id":"u1","author_name":"niorio","icon_url":null,"min_ram_mb":2048,"boosty_blog":null,"versions_count":2,"created_at":"x","url":"u","rating":1.5}"#;
        let p: PackCatalog = serde_json::from_str(json).unwrap();
        assert_eq!(p.author_name.as_deref(), Some("niorio"));
        assert_eq!(p.min_ram_mb, Some(2048));
        assert_eq!(p.versions_count, 2);
    }

    #[test]
    fn detail_and_admin_parse_backend_snake_case() {
        let d: PackDetail = serde_json::from_str(
            r#"{"id":"1","name":"n","my_rating":1,"boosty_blog":"b","icon_url":"i","created_at":"c"}"#,
        ).unwrap();
        assert_eq!(d.my_rating, Some(1));
        assert_eq!(d.boosty_blog.as_deref(), Some("b"));
        let u: AdminUser = serde_json::from_str(
            r#"{"id":"1","username":"n","display_name":null,"email":null,"email_confirmed":false,"role":"admin","banned":false,"ban_reason":null,"created_at":"c"}"#,
        ).unwrap();
        assert_eq!(u.role, "admin");
        let out = serde_json::to_value(&u).unwrap();
        assert!(out.get("emailConfirmed").is_some(), "TS ждёт camelCase");
    }
}

/// Комментарий в админ-ленте модерации.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminComment {
    pub id: String,
    #[serde(default, alias = "pack_id")]
    pub pack_id: String,
    #[serde(default, alias = "pack_name")]
    pub pack_name: String,
    #[serde(default, alias = "author_name")]
    pub author_name: String,
    pub body: String,
    #[serde(default, alias = "created_at")]
    pub created_at: String,
}

/// Payload создания пользователя админом.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminCreateUser {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
}

pub async fn mono_admin_list_comments(client: &reqwest::Client, access_token: &str) -> Result<Vec<AdminComment>> {
    let base = crate::config::backend_url();
    let url = format!("{base}/admin/comments");
    let resp = client.get(&url).bearer_auth(access_token).send().await.context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() { return Err(anyhow!("Mono: {}", api_error(&text))); }
    serde_json::from_str(&text).context("Некорректный ответ сервера Mono")
}

pub async fn mono_admin_create_user(client: &reqwest::Client, access_token: &str, payload: &AdminCreateUser) -> Result<AdminUser> {
    let base = crate::config::backend_url();
    let url = format!("{base}/admin/users");
    let resp = client.post(&url).bearer_auth(access_token).json(payload).send().await.context("Не удалось связаться с сервером Mono")?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() { return Err(anyhow!("Mono: {}", api_error(&text))); }
    serde_json::from_str(&text).context("Некорректный ответ сервера Mono")
}
