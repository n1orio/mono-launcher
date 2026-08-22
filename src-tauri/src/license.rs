//! Лицензии платных сборок через Boosty — без собственного бэкенда.
//!
//! Сборка считается платной, если у неё задан `boostyBlog` (в `packs.json`
//! издателя). Игрок привязывает свой личный Boosty-токен
//! (Настройки Boosty → Приложения → «Создать приложение» — токен выдаётся
//! сразу), лаунчер проверяет по API Boosty подписку на блог издателя:
//!
//!   1. `GET api.boosty.to/v1.0/user/me`          — id владельца токена
//!   2. `GET api.boosty.to/v1.0/user/{id}/subscriptions` — его подписки
//!
//! и ищет блог издателя в списке. Никто из игроков не получает чужих
//! секретов: токен у каждого свой, проверка идёт от лица игрока.
//!
//! Чтобы не дёргать API при каждом запуске, после успешной проверки
//! действует «льгота» (GRACE): до её конца игра разрешена оффлайн.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::config;

/// База Boosty API.
const BOOSTY_API: &str = "https://api.boosty.to/v1.0";
/// Льгота после успешной проверки: столько игра разрешена без сети.
/// При каждом запуске с сетью льгота продлевается.
const GRACE: Duration = Duration::from_secs(3 * 24 * 3600);

/// Данные входа, захваченные из окна входа Boosty (для автопродления токена).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoostyAuth {
    /// Новый access-токен игрока.
    pub access_token: String,
    /// Refresh-токен для автообновления.
    pub refresh_token: String,
    /// Идентификатор устройства Boosty (обязателен для refresh).
    pub device_id: String,
    /// Когда истекает access-токен (unix).
    pub token_expires_at: u64,
}

/// Псевдо-ключ глобальной привязки Boosty (внутренние операции с токенами).
const GLOBAL_KEY: &str = "__global__";

/// Статус лицензии для фронтенда.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseInfo {
    /// Блог издателя, на который проверялась подписка.
    pub blog: String,
    /// Есть активная подписка на блог (по последней проверке).
    pub subscribed: bool,
    /// Дата окончания подписки по Boosty (unix). None — не подписан/неизвестно.
    pub expires_at: Option<u64>,
    /// До какого момента действует локальная льгота без сети (unix).
    pub cached_until: u64,
    /// Название тарифа активной подписки (None — неизвестно/не подписан).
    pub tier: Option<String>,
    /// Тарифы, требуемые сборке (пусто — подходит любой).
    pub required_tiers: Vec<String>,
}

/// Хранилище: `licenses.json` → `{ packs: { "<pack_id>": StoredBoosty }, global: ?StoredBoosty }`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct LicenseStore {
    #[serde(default)]
    packs: HashMap<String, StoredBoosty>,
    /// Глобальная привязка аккаунта Boosty (одна на лаунчер). Любая платная
    /// сборка без собственной записи использует токены отсюда. `blog` не задан —
    /// определяется по сборке при проверке.
    #[serde(default)]
    global: Option<StoredBoosty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredBoosty {
    /// Ник блога издателя (slug из url boosty).
    blog: String,
    /// Личный токен игрока (хранится только локально).
    token: String,
    /// Refresh-токен для автопродления (если привязано через окно входа Boosty).
    #[serde(default)]
    refresh_token: Option<String>,
    /// Идентификатор устройства Boosty (обязателен для обновления токена).
    #[serde(default)]
    device_id: Option<String>,
    /// Когда истекает access-токен (unix). None — срок неизвестен.
    #[serde(default)]
    token_expires_at: Option<u64>,
    /// Срок подписки по Boosty (unix).
    expires_at: Option<u64>,
    /// Льгота действует до (unix).
    cached_until: u64,
    /// Название тарифа (уровня) активной подписки, если известно.
    #[serde(default)]
    tier: Option<String>,
}

impl StoredBoosty {
    /// Есть ли всё необходимое для автопродления токена.
    fn can_refresh(&self) -> bool {
        let rf = self.refresh_token.as_deref();
        let did = self.device_id.as_deref();
        rf.is_some_and(|r| !r.is_empty()) && did.is_some_and(|d| !d.is_empty())
    }
}

fn store_path() -> Result<std::path::PathBuf> {
    Ok(config::launcher_root()?.join("licenses.json"))
}

fn load_store() -> LicenseStore {
    let Ok(path) = store_path() else {
        return LicenseStore::default();
    };
    let Ok(raw) = std::fs::read_to_string(path) else {
        return LicenseStore::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_store(store: &LicenseStore) -> Result<()> {
    let path = store_path()?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&path, serde_json::to_vec_pretty(store)?)?;
    Ok(())
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Ник блога сборки (или None — сборка бесплатная).
pub fn pack_blog(pack_id: &str) -> Result<Option<String>> {
    let pack =
        config::find_pack(pack_id)?.ok_or_else(|| anyhow!("Сборка не найдена: {pack_id}"))?;
    Ok(pack
        .boosty_blog
        .map(|b| b.trim().to_lowercase())
        .filter(|b| !b.is_empty()))
}

/// Один элемент списка подписок Boosty (`subscriptions[]`).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BoostySubscription {
    #[serde(default)]
    blog: serde_json::Value,
    #[serde(default)]
    tokens: Vec<serde_json::Value>,
    #[serde(default)]
    can_view_paid: bool,
    /// Id уровня подписки (тариф), если API его отдал.
    #[serde(default)]
    level_id: Option<u64>,
    /// Объект уровня подписки (`subscription_level`), из него берём название тарифа.
    #[serde(default)]
    subscription_level: Option<serde_json::Value>,
}

/// Название тарифа подписки (уровня Boosty), если API его отдал.
fn sub_tier(sub: &BoostySubscription) -> Option<String> {
    if let Some(lvl) = &sub.subscription_level {
        let name = lvl
            .get("name")
            .or_else(|| lvl.get("title"))
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        if let Some(n) = name {
            return Some(n.to_string());
        }
    }
    // Фолбэк: числовой id уровня.
    sub.level_id.map(|id| id.to_string())
}

/// Итог проверки подписки (с учётом тарифов, если они требуются сборке).
#[derive(Debug, Clone)]
pub struct BoostyCheck {
    /// Есть ли активная подписка на блог (тариф не учитывается).
    pub subscribed: bool,
    /// Название тарифа активной подписки (если известно).
    pub tier: Option<String>,
    /// Дата окончания подписки совместимого тарифа (unix). None — не активна/
    /// не тот тариф/нет срока у отданных токенов.
    pub expires_at: Option<u64>,
    /// Требуемые сборке тарифы (нижний регистр). Пусто — подходит любой.
    pub required_tiers: Vec<String>,
}

impl BoostyCheck {
    /// Подходит ли подписка: активна на блог и (если нужен тариф) покрыта тарифом.
    fn satisfied(&self) -> bool {
        self.expires_at.is_some()
    }
    /// Есть подписка на блог, но её тариф не подходит.
    fn wrong_tier(&self) -> bool {
        self.subscribed && !self.required_tiers.is_empty() && self.expires_at.is_none()
    }
}

/// Чистая логика поиска подписки в ответе (`subscriptions[]`).
fn find_subscription(
    subs: &[BoostySubscription],
    blog: &str,
    now: u64,
    required: Option<&[String]>,
) -> BoostyCheck {
    let required: Vec<String> = required
        .map(|r| {
            r.iter()
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let mut subscribed = false;
    let mut tier: Option<String> = None;
    let mut expires: Option<u64> = None;
    for s in subs {
        let nick = s
            .blog
            .get("nickname")
            .and_then(|v| v.as_str())
            .map(|n| n.to_lowercase())
            .unwrap_or_default();
        // Токены — источник истины о будущем доступе; canViewPaid — фолбэк,
        // когда токенов нет вовсе (у истёкшей подписки токен остаётся, но
        // expireDate в прошлом — такая подписка не активна).
        let has_future_token = s.tokens.iter().any(|t| {
            t.get("expireDate")
                .and_then(|v| v.as_u64())
                .map(|exp| exp > now)
                .unwrap_or(false)
        });
        let active = if s.tokens.is_empty() {
            s.can_view_paid
        } else {
            has_future_token
        };
        if !nick.eq_ignore_ascii_case(blog) || !active {
            continue;
        }
        subscribed = true;
        let this_tier = sub_tier(s);
        if tier.is_none() {
            tier = this_tier.clone();
        }
        let tier_ok = required.is_empty()
            || this_tier
                .as_ref()
                .is_some_and(|t| required.iter().any(|r| r == &t.trim().to_lowercase()));
        if !tier_ok {
            continue;
        }
        for t in &s.tokens {
            if let Some(exp) = t.get("expireDate").and_then(|v| v.as_u64()) {
                expires = Some(expires.map_or(exp, |cur| cur.max(exp)));
            }
        }
    }
    BoostyCheck {
        subscribed,
        tier,
        expires_at: expires,
        required_tiers: required,
    }
}

/// Проверяет подписку игрока (токен) на блог издателя по API Boosty.
/// Возвращает итог с учётом требуемых тарифов (`required`, если сборка их задала).
pub async fn check_subscription(
    client: &reqwest::Client,
    blog: &str,
    token: &str,
    required: Option<&[String]>,
) -> Result<BoostyCheck> {
    let token = token.trim();
    if token.is_empty() {
        return Err(anyhow!("Токен Boosty пустой"));
    }
    let bearer = format!("Bearer {token}");

    // 1. id владельца токена.
    let me: serde_json::Value = client
        .get(format!("{BOOSTY_API}/user/me"))
        .header("Authorization", &bearer)
        .header("User-Agent", "mono-launcher")
        .send()
        .await
        .context("Boosty недоступен")?
        .error_for_status()
        .map_err(|e| boosty_err(&e, "не удалось авторизоваться токеном Boosty"))?
        .json()
        .await
        .context("Неожиданный ответ Boosty")?;
    let user_id = me
        .pointer("/user/id")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| {
            me.pointer("/user/id")
                .and_then(|v| v.as_u64())
                .map(|i| i.to_string())
        })
        .ok_or_else(|| anyhow!("Не удалось прочитать профиль токена"))?;

    // 2. Подписки владельца токена (страницами).
    let mut offset = 0u64;
    let mut subs: Vec<BoostySubscription> = Vec::new();
    loop {
        let url = format!("{BOOSTY_API}/user/{user_id}/subscriptions?offset={offset}&limit=100");
        let resp = client
            .get(&url)
            .header("Authorization", &bearer)
            .header("User-Agent", "mono-launcher")
            .send()
            .await
            .context("Boosty недоступен")?;
        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(anyhow!(
                "Токен Boosty недействителен — проверьте его в Настройках Boosty"
            ));
        }
        let json: serde_json::Value = resp
            .error_for_status()
            .map_err(|e| boosty_err(&e, "не удалось получить подписки"))?
            .json()
            .await
            .context("Неожиданный ответ Boosty")?;
        let list = json.get("subscriptions").and_then(|v| v.as_array());
        let Some(list) = list else { break };
        subs.extend(
            list.iter()
                .filter_map(|v| serde_json::from_value::<BoostySubscription>(v.clone()).ok()),
        );
        match json.get("offset").and_then(|v| v.as_u64()) {
            Some(next) if !list.is_empty() => offset = next,
            _ => break,
        }
    }

    Ok(find_subscription(&subs, blog, now_unix(), required))
}

/// Время, до которого можно доверять кэшированному статусу: не дальше реального
/// окончания подписки и не более GRACE сверх "сейчас". Если реального срока нет —
/// не кэшируем (возвращаем "сейчас", чтобы гейт каждый раз перепроверял).
fn cached_after(expires_at: Option<u64>, now: u64) -> u64 {
    match expires_at {
        Some(e) => e.min(now.saturating_add(GRACE.as_secs())),
        None => now,
    }
}

fn boosty_err(e: &reqwest::Error, hint: &str) -> anyhow::Error {
    let msg = e.to_string();
    if msg.contains("401") || msg.contains("403") {
        anyhow!("Токен Boosty недействителен — проверьте его в Настройках Boosty")
    } else {
        anyhow!("Boosty: {hint} ({msg})")
    }
}

/// Признак ошибки авторизации (проводящий к попытке обновить токен).
fn is_invalid_token(err: &anyhow::Error) -> bool {
    let msg = err.to_string();
    msg.contains("недействителен") || msg.contains("401") || msg.contains("403")
}

/// Обновляет пару токенов Boosty по `refresh_token` (form-urlencoded, как
/// требует API). Возвращает (access_token, refresh_token, срок в секундах).
async fn refresh_tokens(
    client: &reqwest::Client,
    refresh_token: &str,
    device_id: &str,
) -> Result<(String, String, u64)> {
    let resp = client
        .post(format!("{BOOSTY_API}/oauth/token/"))
        .header("User-Agent", "mono-launcher")
        .form(&[
            ("device_id", device_id),
            ("device_os", "web"),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await
        .context("Boosty недоступен")?
        .error_for_status()
        .map_err(|e| boosty_err(&e, "не удалось обновить токен Boosty"))?;
    let json: serde_json::Value = resp
        .json()
        .await
        .context("Неожиданный ответ Boosty")?;
    let access = json
        .get("access_token")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("Boosty не вернул новый access-токен"))?
        .to_string();
    let refresh = json
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("Boosty не вернул новый refresh-токен"))?
        .to_string();
    let expires_in = json
        .get("expires_in")
        .and_then(|v| v.as_u64())
        .unwrap_or(3600);
    Ok((access, refresh, expires_in))
}

/// Сохраняет свежую пару токенов в хранилище (ключ — сборка).
fn store_token_refresh(storage_key: &str, access: &str, refresh: &str, now: u64, expires_in: u64) {
    let mut store = load_store();
    if let Some(st) = store.packs.get_mut(storage_key) {
        st.token = access.to_string();
        st.refresh_token = Some(refresh.to_string());
        st.token_expires_at = Some(now.saturating_add(expires_in));
    } else if storage_key == GLOBAL_KEY {
        if let Some(g) = store.global.as_mut() {
            g.token = access.to_string();
            g.refresh_token = Some(refresh.to_string());
            g.token_expires_at = Some(now.saturating_add(expires_in));
        }
    }
    let _ = save_store(&store);
}

/// Возвращает годный access-токен для сохранённой записи: если есть refresh и
/// срок токена на исходе (или `force`), обновляет его. Ошибки отдаёт «как есть».
///
/// Если обновление падает с ошибкой авторизации (refresh-токен протух/отозван),
/// сохраняем это в записи: пару refresh+device забываем, чтобы не пытаться
/// обновлять одни и те же негодные токены бесконечно — игрок просто войдёт
/// через окно Boosty ещё раз.
async fn stored_effective_token(
    client: &reqwest::Client,
    pack_id: &str,
    stored: &StoredBoosty,
    force: bool,
) -> Result<String> {
    let now = now_unix();
    // Свежий access-токен и без принуждения — берём как есть.
    let fresh = stored
        .token_expires_at
        .is_some_and(|e| e > now.saturating_add(60));
    if !force && fresh {
        return Ok(stored.token.clone());
    }
    if !stored.can_refresh() {
        // Обновлять нечем — используем сохранённый токен.
        return Ok(stored.token.clone());
    }
    let refresh_result = refresh_tokens(
        client,
        stored.refresh_token.as_deref().unwrap_or_default(),
        stored.device_id.as_deref().unwrap_or_default(),
    )
    .await;
    let (access, refresh, expires_in) = match refresh_result {
        Ok(t) => t,
        Err(e) if is_invalid_token(&e) => {
            forget_refresh(pack_id);
            return Err(anyhow!("Boosty-сессия истекла — войдите в Boosty ещё раз"));
        }
        Err(e) => return Err(e),
    };
    store_token_refresh(pack_id, &access, &refresh, now, expires_in);
    Ok(access)
}

/// Забывает пару refresh+device у записи сборки (после неудачного обновления).
fn forget_refresh(storage_key: &str) {
    let mut store = load_store();
    if let Some(st) = store.packs.get_mut(storage_key) {
        st.refresh_token = None;
        st.device_id = None;
        st.token_expires_at = None;
    } else if storage_key == GLOBAL_KEY {
        if let Some(g) = store.global.as_mut() {
            g.refresh_token = None;
            g.device_id = None;
            g.token_expires_at = None;
        }
    }
    let _ = save_store(&store);
}

/// Проверка подписки с автопродлением токена: использует свежий access-токен;
/// при ошибке авторизации пробует один раз принудительно обновить и повторить.
async fn subscription_with_refresh(
    client: &reqwest::Client,
    pack_id: &str,
    blog: &str,
    stored: &StoredBoosty,
    required: Option<&[String]>,
) -> Result<BoostyCheck> {
    let first = stored_effective_token(client, pack_id, stored, false).await?;
    match check_subscription(client, blog, &first, required).await {
        Ok(res) => Ok(res),
        Err(e) if is_invalid_token(&e) && stored.can_refresh() => {
            let token = stored_effective_token(client, pack_id, stored, true).await?;
            check_subscription(client, blog, &token, required).await
        }
        Err(e) => Err(e),
    }
}

/// Требуемые тарифы сборки (или None — любой тариф).
fn required_tiers(pack_id: &str) -> Option<Vec<String>> {
    config::boosty_tiers(pack_id)
}

/// Привязывает токен игрока к сборке и проверяет подписку.
/// `refresh_token`/`device_id`/`token_expires_at` — опциональная пара для
/// автопродления access-токена (захвачена окном входа Boosty).
pub async fn set_license(
    client: &reqwest::Client,
    pack_id: &str,
    token: &str,
    refresh_token: Option<String>,
    device_id: Option<String>,
    token_expires_at: Option<u64>,
) -> Result<LicenseInfo> {
    let blog = pack_blog(pack_id)?
        .ok_or_else(|| anyhow!("Сборка {pack_id} не требует лицензию (нет boostyBlog)"))?;
    let required = required_tiers(pack_id);
    let check = check_subscription(client, &blog, token, required.as_deref()).await?;
    let expires_at = check.expires_at;
    let cached_until = cached_after(expires_at, now_unix());
    let mut store = load_store();
    store.packs.insert(
        pack_id.to_string(),
        StoredBoosty {
            blog: blog.clone(),
            token: token.trim().to_string(),
            refresh_token,
            device_id,
            token_expires_at,
            expires_at,
            cached_until,
            tier: check.tier.clone(),
        },
    );
    save_store(&store)?;
    Ok(LicenseInfo {
        blog,
        subscribed: check.satisfied(),
        expires_at,
        cached_until,
        tier: check.tier,
        required_tiers: check.required_tiers,
    })
}

/// Статус для UI: из кэша, если льгота ещё действует, иначе — свежая проверка.
pub async fn license_status(client: &reqwest::Client, pack_id: &str) -> Result<LicenseInfo> {
    let blog = pack_blog(pack_id)?
        .ok_or_else(|| anyhow!("Сборка {pack_id} не требует лицензию (нет boostyBlog)"))?;
    let required = required_tiers(pack_id);
    let now = now_unix();
    let store = load_store();
    if let Some(s) = store.packs.get(pack_id) {
        let tier = s.tier.clone();
        // Доверяем кэшу, только если льгота ещё действует И субправда
        // (реальный срок окончания ещё впереди).
        if s.cached_until > now && s.expires_at.is_some_and(|e| now < e) {
            return Ok(LicenseInfo {
                blog: s.blog.clone(),
                subscribed: s.expires_at.is_some(),
                expires_at: s.expires_at,
                cached_until: s.cached_until,
                tier,
                required_tiers: required.unwrap_or_default(),
            });
        }
        // Льгота вышла — обновляем по сети.
        let check = subscription_with_refresh(client, pack_id, &blog, s, required.as_deref()).await?;
        let cached_until = cached_after(check.expires_at, now);
        let mut store = load_store();
        if let Some(st) = store.packs.get_mut(pack_id) {
            st.expires_at = check.expires_at;
            st.cached_until = cached_until;
            st.tier = check.tier.clone();
        }
        save_store(&store)?;
        return Ok(LicenseInfo {
            blog,
            subscribed: check.satisfied(),
            expires_at: check.expires_at,
            cached_until,
            tier: check.tier,
            required_tiers: check.required_tiers,
        });
    }
    // Нет собственной записи — пробуем глобальную привязку Boosty.
    if let Some(g) = store.global.clone() {
        if !g.token.trim().is_empty() {
            let check =
                subscription_with_refresh(client, GLOBAL_KEY, &blog, &g, required.as_deref())
                    .await?;
            // Кэш глобальной привязки не храним: статус всегда проверяем вживую.
            return Ok(LicenseInfo {
                blog,
                subscribed: check.satisfied(),
                expires_at: check.expires_at,
                cached_until: 0,
                tier: check.tier,
                required_tiers: check.required_tiers,
            });
        }
    }
    Ok(LicenseInfo {
        blog,
        subscribed: false,
        expires_at: None,
        cached_until: 0,
        tier: None,
        required_tiers: required.unwrap_or_default(),
    })
}

/// Гейт установки/запуска: Ok — лицензия в порядке (или сборка бесплатная),
/// Err — нужна/недействительна подписка (текст ошибки показываем в UI).
pub async fn ensure_license(client: &reqwest::Client, pack_id: &str) -> Result<()> {
    let Some(blog) = pack_blog(pack_id)? else {
        // Сборка бесплатная.
        return Ok(());
    };
    let required = required_tiers(pack_id);
    let now = now_unix();
    let store = load_store();
    let stored = store.packs.get(pack_id).cloned();
    match stored {
        Some(s) if s.cached_until > now && s.expires_at.is_some_and(|e| now < e) => Ok(()),
        Some(s) => {
            let check =
                subscription_with_refresh(client, pack_id, &blog, &s, required.as_deref()).await?;
            let expired = check.expires_at.is_none() || check.expires_at.is_some_and(|e| now >= e);
            if expired {
                // Подписки больше нет (или закончилась, или не тот тариф) —
                // снимаем льготу.
                let mut st = load_store();
                if let Some(x) = st.packs.get_mut(pack_id) {
                    x.expires_at = None;
                    x.cached_until = 0;
                }
                save_store(&st)?;
                if check.wrong_tier() {
                    return Err(anyhow!(
                        "Подписка Boosty на «{blog}» есть, но нужен тариф: {} — см. https://boosty.to/{blog}",
                        check.required_tiers.join(" / ")
                    ));
                }
                return Err(anyhow!(
                    "Подписка Boosty на «{blog}» не активна — оформите её и повторите"
                ));
            }
            let mut st = load_store();
            if let Some(x) = st.packs.get_mut(pack_id) {
                x.expires_at = check.expires_at;
                x.cached_until = cached_after(check.expires_at, now);
                x.tier = check.tier.clone();
            }
            save_store(&st)?;
            Ok(())
        }
        None => {
            // Своей записи нет — используем глобальную привязку Boosty.
            let store = load_store();
            let Some(g) = store.global.filter(|g| !g.token.trim().is_empty()) else {
                return Err(anyhow!(
                    "Сборка «{pack_id}» платная: привяжите Boosty, чтобы скачать и играть"
                ));
            };
            let check =
                subscription_with_refresh(client, GLOBAL_KEY, &blog, &g, required.as_deref())
                    .await?;
            let expired = check.expires_at.is_none() || check.expires_at.is_some_and(|e| now >= e);
            if expired {
                if check.wrong_tier() {
                    return Err(anyhow!(
                        "Подписка Boosty на «{blog}» есть, но нужен тариф: {} — см. https://boosty.to/{blog}",
                        check.required_tiers.join(" / ")
                    ));
                }
                return Err(anyhow!(
                    "Подписка Boosty на «{blog}» не активна — оформите её и повторите"
                ));
            }
            Ok(())
        }
    }
}

/// Удаляет сохранённый токен сборки.
pub fn clear_license(pack_id: &str) -> Result<()> {
    let mut store = load_store();
    store.packs.remove(pack_id);
    save_store(&store)
}

/// Глобальная привязка Boosty: есть ли хоть какие-то токены.
pub fn global_linked() -> bool {
    load_store()
        .global
        .as_ref()
        .is_some_and(|g| !g.token.trim().is_empty())
}

/// Сохраняет глобальную привязку Boosty (единый аккаунт на лаунчер).
pub fn set_global_license(
    token: &str,
    refresh_token: Option<String>,
    device_id: Option<String>,
    token_expires_at: Option<u64>,
) -> Result<()> {
    let mut store = load_store();
    store.global = Some(StoredBoosty {
        blog: String::new(),
        token: token.trim().to_string(),
        refresh_token,
        device_id,
        token_expires_at,
        expires_at: None,
        cached_until: 0,
        tier: None,
    });
    save_store(&store)
}

/// Удаляет глобальную привязку Boosty.
pub fn clear_global_license() -> Result<()> {
    let mut store = load_store();
    store.global = None;
    save_store(&store)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sub_json(nick: &str, exp: Option<u64>, can_view: bool) -> serde_json::Value {
        let tokens = match exp {
            Some(e) => serde_json::json!([{ "expireDate": e }]),
            None => serde_json::json!([]),
        };
        serde_json::json!({
            "blog": { "nickname": nick, "name": nick },
            "tokens": tokens,
            "canViewPaid": can_view,
        })
    }

    /// Чистая функция поиска блога в ответе — тестируется без сети.
    fn find_blog(subs: &[BoostySubscription], blog: &str, now: u64) -> BoostyCheck {
        find_subscription(subs, blog, now, None)
    }

    #[test]
    fn finds_active_subscription_by_nickname() {
        let now = 1_700_000_000u64;
        let subs: Vec<BoostySubscription> = vec![
            serde_json::from_value(sub_json("other-blog", Some(now + 100), false)).unwrap(),
            serde_json::from_value(sub_json("Untold-Legends", Some(now + 9999), true)).unwrap(),
        ];
        let check = find_blog(&subs, "untold-legends", now);
        assert_eq!(check.expires_at, Some(now + 9999));
        assert!(check.satisfied());
    }

    #[test]
    fn ignores_expired_and_foreign_blogs() {
        let now = 1_700_000_000u64;
        let subs: Vec<BoostySubscription> = vec![
            serde_json::from_value(sub_json("untold-legends", Some(now - 5), true)).unwrap(),
            serde_json::from_value(sub_json("other", Some(now + 10), true)).unwrap(),
        ];
        let check = find_blog(&subs, "untold-legends", now);
        assert!(check.expires_at.is_none());
        assert!(!check.subscribed);
    }

    #[test]
    fn no_tokens_but_can_view_paid_counts() {
        let now = 1_700_000_000u64;
        let subs: Vec<BoostySubscription> =
            vec![serde_json::from_value(sub_json("blog", None, true)).unwrap()];
        let check = find_blog(&subs, "blog", now);
        // Подписка есть (canViewPaid), но реального токена/срока нет —
        // льгота по ней не выдаётся.
        assert!(check.subscribed);
        assert!(check.expires_at.is_none());
        assert!(!check.satisfied());
    }

    #[test]
    fn malformed_entries_are_skipped() {
        let now = 1_700_000_000u64;
        let subs: Vec<BoostySubscription> =
            vec![serde_json::from_value(serde_json::json!({ "blog": {} })).unwrap()];
        let check = find_blog(&subs, "blog", now);
        assert!(!check.subscribed);
        assert!(check.expires_at.is_none());
    }

    #[test]
    fn wrong_tier_is_detected() {
        let now = 1_700_000_000u64;
        let subs: Vec<BoostySubscription> = vec![serde_json::from_value(
            serde_json::json!({
                "blog": { "nickname": "untold-legends" },
                "tokens": [{ "expireDate": now + 9999 }],
                "canViewPaid": true,
                "subscriptionLevel": { "name": "База" },
            }),
        )
        .unwrap()];
        let required = vec!["Премиум".to_string(), "vip".to_string()];
        let check = find_subscription(&subs, "untold-legends", now, Some(&required));
        assert!(check.subscribed);
        assert!(check.wrong_tier());
        assert!(check.expires_at.is_none());
        assert_eq!(check.tier.as_deref(), Some("База"));
    }

    #[test]
    fn matching_tier_is_accepted() {
        let now = 1_700_000_000u64;
        let subs: Vec<BoostySubscription> = vec![serde_json::from_value(
            serde_json::json!({
                "blog": { "nickname": "untold-legends" },
                "tokens": [{ "expireDate": now + 9999 }],
                "canViewPaid": true,
                "levelId": 5,
                "subscriptionLevel": { "name": "VIP" },
            }),
        )
        .unwrap()];
        let required = vec!["VIP".to_string()];
        let check = find_subscription(&subs, "untold-legends", now, Some(&required));
        assert!(check.satisfied());
        assert!(!check.wrong_tier());
        assert_eq!(check.tier.as_deref(), Some("VIP"));
    }

    #[test]
    fn no_required_tiers_accepts_any() {
        let now = 1_700_000_000u64;
        let subs: Vec<BoostySubscription> = vec![serde_json::from_value(
            serde_json::json!({
                "blog": { "nickname": "untold-legends" },
                "tokens": [{ "expireDate": now + 9999 }],
                "canViewPaid": true,
            }),
        )
        .unwrap()];
        let check = find_blog(&subs, "untold-legends", now);
        assert!(check.satisfied());
        assert!(check.required_tiers.is_empty());
    }

    #[test]
    fn old_stored_boosty_variant_is_loaded() {
        // Записи из более старого licenses.json (без refresh-полей) должны читаться.
        let old: StoredBoosty = serde_json::from_str(
            r#"{"blog":"untold-legends","token":"abc","expires_at":1700000100,"cached_until":1700000200}"#,
        )
        .unwrap();
        assert_eq!(old.blog, "untold-legends");
        assert!(old.refresh_token.is_none());
        assert!(old.device_id.is_none());
        assert!(old.token_expires_at.is_none());
        assert!(!old.can_refresh());
        assert_eq!(old.token, "abc");
    }

    #[test]
    fn stored_boosty_with_refresh_can_refresh() {
        let s = serde_json::from_value::<StoredBoosty>(serde_json::json!({
            "blog": "blog",
            "token": "a",
            "refresh_token": "r",
            "device_id": "d",
            "token_expires_at": 1700000000,
            "expires_at": null,
            "cached_until": 0,
        }))
        .unwrap();
        assert!(s.can_refresh());
    }

    #[test]
    fn invalid_token_errors_are_flagged() {
        use anyhow::anyhow;
        assert!(is_invalid_token(&anyhow!(
            "Токен Boosty недействителен — проверьте его в Настройках Boosty"
        )));
        assert!(is_invalid_token(&anyhow!("HTTP status client error (401 Unauthorized)")));
        assert!(!is_invalid_token(&anyhow!("Boosty недоступен")));
    }
}
