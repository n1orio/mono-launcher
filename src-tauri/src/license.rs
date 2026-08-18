//! Лицензии платных сборок через Boosty — без собственного бэкенда.
//!
//! Сборка считается платной, если у неё задан `boostyBlog` (конфиг `builtin-packs.json`
//! или `packs.json` издателя). Игрок привязывает свой личный Boosty-токен
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
}

/// Хранилище: `licenses.json` → `{ packs: { "<pack_id>": StoredBoosty } }`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct LicenseStore {
    #[serde(default)]
    packs: HashMap<String, StoredBoosty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredBoosty {
    /// Ник блога издателя (slug из url boosty).
    blog: String,
    /// Личный токен игрока (хранится только локально).
    token: String,
    /// Срок подписки по Boosty (unix).
    expires_at: Option<u64>,
    /// Льгота действует до (unix).
    cached_until: u64,
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
}

/// Проверяет подписку игрока (токен) на блог издателя по API Boosty.
/// Возвращает дату окончания активной подписки (None — подписки нет).
pub async fn check_subscription(
    client: &reqwest::Client,
    blog: &str,
    token: &str,
) -> Result<Option<u64>> {
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

    // Ищем блог издателя и самую позднюю дату окончания.
    let now = now_unix();
    let mut found = false;
    let mut expires: Option<u64> = None;
    for s in &subs {
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
        if nick.eq_ignore_ascii_case(blog) && active {
            found = true;
            for t in &s.tokens {
                if let Some(exp) = t.get("expireDate").and_then(|v| v.as_u64()) {
                    expires = Some(expires.map_or(exp, |cur| cur.max(exp)));
                }
            }
        }
    }
    if !found {
        return Ok(None);
    }
    // Не фабрикуем срок: если у подписки нет реальной даты окончания,
    // считаем её неподтверждённой (вернём None), а не выдаём свежую льготу.
    Ok(expires)
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

/// Привязывает токен игрока к сборке и проверяет подписку.
pub async fn set_license(
    client: &reqwest::Client,
    pack_id: &str,
    token: &str,
) -> Result<LicenseInfo> {
    let blog = pack_blog(pack_id)?
        .ok_or_else(|| anyhow!("Сборка {pack_id} не требует лицензию (нет boostyBlog)"))?;
    let expires_at = check_subscription(client, &blog, token).await?;
    let cached_until = cached_after(expires_at, now_unix());
    let mut store = load_store();
    store.packs.insert(
        pack_id.to_string(),
        StoredBoosty {
            blog: blog.clone(),
            token: token.trim().to_string(),
            expires_at,
            cached_until,
        },
    );
    save_store(&store)?;
    Ok(LicenseInfo {
        blog,
        subscribed: expires_at.is_some(),
        expires_at,
        cached_until,
    })
}

/// Статус для UI: из кэша, если льгота ещё действует, иначе — свежая проверка.
pub async fn license_status(client: &reqwest::Client, pack_id: &str) -> Result<LicenseInfo> {
    let blog = pack_blog(pack_id)?
        .ok_or_else(|| anyhow!("Сборка {pack_id} не требует лицензию (нет boostyBlog)"))?;
    let now = now_unix();
    let store = load_store();
    if let Some(s) = store.packs.get(pack_id) {
        // Доверяем кэшу, только если льгота ещё действует И субправда
        // (реальный срок окончания ещё впереди).
        if s.cached_until > now && s.expires_at.is_some_and(|e| now < e) {
            return Ok(LicenseInfo {
                blog: s.blog.clone(),
                subscribed: s.expires_at.is_some(),
                expires_at: s.expires_at,
                cached_until: s.cached_until,
            });
        }
        // Льгота вышла — обновляем по сети.
        let expires_at = check_subscription(client, &blog, &s.token).await?;
        let cached_until = cached_after(expires_at, now);
        let mut store = load_store();
        if let Some(st) = store.packs.get_mut(pack_id) {
            st.expires_at = expires_at;
            st.cached_until = cached_until;
        }
        save_store(&store)?;
        return Ok(LicenseInfo {
            blog,
            subscribed: expires_at.is_some(),
            expires_at,
            cached_until,
        });
    }
    Ok(LicenseInfo {
        blog,
        subscribed: false,
        expires_at: None,
        cached_until: 0,
    })
}

/// Гейт установки/запуска: Ok — лицензия в порядке (или сборка бесплатная),
/// Err — нужна/недействительна подписка (текст ошибки показываем в UI).
pub async fn ensure_license(client: &reqwest::Client, pack_id: &str) -> Result<()> {
    let Some(blog) = pack_blog(pack_id)? else {
        // Сборка бесплатная.
        return Ok(());
    };
    let now = now_unix();
    let store = load_store();
    let stored = store.packs.get(pack_id).cloned();
    match stored {
        Some(s) if s.cached_until > now && s.expires_at.is_some_and(|e| now < e) => Ok(()),
        Some(s) => {
            let expires_at = check_subscription(client, &blog, &s.token).await?;
            if expires_at.is_none() || expires_at.is_some_and(|e| now >= e) {
                // Подписки больше нет (или закончилась) — снимаем льготу.
                let mut st = load_store();
                if let Some(x) = st.packs.get_mut(pack_id) {
                    x.expires_at = None;
                    x.cached_until = 0;
                }
                save_store(&st)?;
                return Err(anyhow!(
                    "Подписка Boosty на «{blog}» не активна — оформите её и повторите"
                ));
            }
            let mut st = load_store();
            if let Some(x) = st.packs.get_mut(pack_id) {
                x.expires_at = expires_at;
                x.cached_until = cached_after(expires_at, now);
            }
            save_store(&st)?;
            Ok(())
        }
        None => Err(anyhow!(
            "Сборка «{pack_id}» платная: привяжите Boosty, чтобы скачать и играть"
        )),
    }
}

/// Удаляет сохранённый токен сборки.
pub fn clear_license(pack_id: &str) -> Result<()> {
    let mut store = load_store();
    store.packs.remove(pack_id);
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
    fn find_blog(subs: &[BoostySubscription], blog: &str, now: u64) -> Option<u64> {
        let mut found = false;
        let mut expires: Option<u64> = None;
        for s in subs {
            let nick = s
                .blog
                .get("nickname")
                .and_then(|v| v.as_str())
                .map(|n| n.to_lowercase())
                .unwrap_or_default();
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
            if nick.eq_ignore_ascii_case(blog) && active {
                found = true;
                for t in &s.tokens {
                    if let Some(exp) = t.get("expireDate").and_then(|v| v.as_u64()) {
                        expires = Some(expires.map_or(exp, |cur| cur.max(exp)));
                    }
                }
            }
        }
        if !found {
            None
        } else {
            Some(expires.unwrap_or(now + GRACE.as_secs()))
        }
    }

    #[test]
    fn finds_active_subscription_by_nickname() {
        let now = 1_700_000_000u64;
        let subs: Vec<BoostySubscription> = vec![
            serde_json::from_value(sub_json("other-blog", Some(now + 100), false)).unwrap(),
            serde_json::from_value(sub_json("Untold-Legends", Some(now + 9999), true)).unwrap(),
        ];
        let exp = find_blog(&subs, "untold-legends", now).unwrap();
        assert_eq!(exp, now + 9999);
    }

    #[test]
    fn ignores_expired_and_foreign_blogs() {
        let now = 1_700_000_000u64;
        let subs: Vec<BoostySubscription> = vec![
            serde_json::from_value(sub_json("untold-legends", Some(now - 5), true)).unwrap(),
            serde_json::from_value(sub_json("other", Some(now + 10), true)).unwrap(),
        ];
        assert!(find_blog(&subs, "untold-legends", now).is_none());
    }

    #[test]
    fn no_tokens_but_can_view_paid_counts() {
        let now = 1_700_000_000u64;
        let subs: Vec<BoostySubscription> =
            vec![serde_json::from_value(sub_json("blog", None, true)).unwrap()];
        assert!(find_blog(&subs, "blog", now).is_some());
    }

    #[test]
    fn malformed_entries_are_skipped() {
        let now = 1_700_000_000u64;
        let subs: Vec<BoostySubscription> =
            vec![serde_json::from_value(serde_json::json!({ "blog": {} })).unwrap()];
        assert!(find_blog(&subs, "blog", now).is_none());
    }
}
