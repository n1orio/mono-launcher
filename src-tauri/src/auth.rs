use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config;

/// Сессия пользователя, передаваемая в команду запуска игры.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub user_type: String,
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
    })
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
    access_token: String,
    refresh_token: Option<String>,
    token_type: String,
    expires_in: u64,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MsUserResp {
    username: String,
    id: String,
}

const AZURE_CLIENT_ID: &str = "CHANGE_ME";

/// Microsoft OAuth2 «устройственный» поток (device code flow).
///
/// Для полной поддержки нужно зарегистрировать приложение в Azure AD и
/// указать настоящий `AZURE_CLIENT_ID`, а также настроить Xbox Live → XSTS →
/// Minecraft цепочку. Здесь реализована основа: получение кода, токена и профиля.
pub async fn login_microsoft(client: &reqwest::Client) -> Result<UserSession> {
    if AZURE_CLIENT_ID == "CHANGE_ME" {
        return Err(anyhow!(
            "Microsoft OAuth2 ещё не настроен: укажите AZURE_CLIENT_ID в src/auth.rs"
        ));
    }

    // 1. Запрашиваем device code.
    let code_resp: MsCodeResp = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
        .form(&[
            ("client_id", AZURE_CLIENT_ID),
            ("scope", "XboxLive.signin offline_access"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    // Здесь фронтенд должен показать code.user_code и code.verification_uri.
    // В реальной реализации открываем браузер и ждём авторизации.
    // Для простоты делаем паузу, пока пользователь авторизуется.
    println!(
        "Откройте {} и введите код {}",
        code_resp.verification_uri, code_resp.user_code
    );

    // 2. Поллинг токена.
    let mut token = None;
    let mut elapsed = 0u64;
    while elapsed < code_resp.expires_in {
        let resp: MsTokenResp = client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("client_id", AZURE_CLIENT_ID),
                ("device_code", &code_resp.device_code),
            ])
            .send()
            .await?
            .json()
            .await?;

        if let Some(err) = resp.error {
            if err == "authorization_pending" {
                tokio::time::sleep(std::time::Duration::from_secs(code_resp.interval)).await;
                elapsed += code_resp.interval;
                continue;
            }
            if err == "authorization_declined" {
                return Err(anyhow!("Пользователь отклонил авторизацию"));
            }
            return Err(anyhow!("Ошибка OAuth2: {err}"));
        }

        token = Some(resp);
        break;
    }

    let token = token.ok_or_else(|| anyhow!("Таймаут авторизации Microsoft"))?;

    // 3. Получаем профиль игрока (упрощённо — минуя XSTS, это заглушка).
    let user: MsUserResp = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(&token.access_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(UserSession {
        username: user.username,
        uuid: user.id,
        access_token: token.access_token,
        user_type: "microsoft".into(),
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
