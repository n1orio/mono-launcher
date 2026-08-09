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

/// Данные для входа: код и страница подтверждения (фаза 1 device code flow).
#[derive(Debug, Clone, Serialize)]
pub struct MsDeviceCodeInfo {
    pub user_code: String,
    pub verification_uri: String,
    pub device_code: String,
    pub interval: u64,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
struct XboxResp {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: XboxClaims,
}

#[derive(Debug, Deserialize)]
struct XboxClaims {
    xui: Vec<XboxUser>,
}

#[derive(Debug, Deserialize)]
struct XboxUser {
    uhs: Option<String>,
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

const AZURE_CLIENT_ID: &str = "CHANGE_ME";

/// Client id можно задать без пересборки:
/// 1) файл `<данные лаунчера>/azure-client-id` (одной строкой),
/// 2) либо переменная окружения NIO_AZURE_CLIENT_ID,
/// 3) либо константа AZURE_CLIENT_ID в этом файле.
fn azure_client_id() -> Option<String> {
    let file = read_client_id_file();
    let env = std::env::var("NIO_AZURE_CLIENT_ID").ok();
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
pub async fn ms_device_code(client: &reqwest::Client) -> Result<MsDeviceCodeInfo> {
    let client_id = require_client_id()?;

    let code_resp: MsCodeResp = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
        .form(&[
            ("client_id", client_id.as_str()),
            ("scope", "XboxLive.signin offline_access"),
        ])
        .send()
        .await
        .context("Не удалось связаться с Microsoft")?
        .error_for_status()
        .context("Microsoft не выдал device code")?
        .json()
        .await?;

    Ok(MsDeviceCodeInfo {
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
    let mut poll_interval = interval.max(5);
    let mut elapsed: u64 = 0;
    while elapsed < expires_in {
        let resp: MsTokenResp = client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("client_id", client_id.as_str()),
                ("device_code", device_code),
            ])
            .send()
            .await?
            .json()
            .await?;

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

        ms_token = Some(resp.access_token);
        break;
    }
    let ms_token = ms_token.ok_or_else(|| anyhow!("Таймаут авторизации Microsoft"))?;

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
    let xsts_body: Value = xsts_resp
        .json()
        .await
        .context("Некорректный ответ XSTS")?;
    if !xsts_status.is_success() {
        let xerr = xsts_body["XErr"].as_u64();
        let msg = match xerr {
            Some(2148916233) | Some(2148916235) | Some(2148916236) | Some(2148916237) =>
                "К этому Microsoft-аккаунту не привязан аккаунт Xbox Live. \
                 Зарегистрируйте бесплатный аккаунт Xbox на xbox.com",
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
    let mine: MineTokenResp = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&json!({
            "identityToken": format!("XBL3.0 x={uhs};{xsts_token}"),
        }))
        .send()
        .await
        .context("Не удалось связаться с Minecraft Services")?
        .error_for_status()
        .context("Minecraft Services отклонил XSTS-токен")?
        .json()
        .await?;

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
        uuid: user.uuid,
        access_token: mine.access_token,
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
