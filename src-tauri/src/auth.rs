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

const AZURE_CLIENT_ID: &str = "CHANGE_ME";

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
        uuid: Uuid::parse_str(&user.uuid)
            .map(|u| u.to_string())
            .unwrap_or_else(|_| Uuid::new_v3(&Uuid::NAMESPACE_DNS, user.uuid.as_bytes()).to_string()),
        access_token: mine.access_token,
        user_type: "microsoft".into(),
    })
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
        let resp: MsTokenResp = client
            .post(ELY_TOKEN_URL)
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
        access_token = Some(resp.access_token);
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
}

impl From<&UserSession> for AccountEntry {
    fn from(s: &UserSession) -> Self {
        AccountEntry {
            id: s.uuid.clone(),
            username: s.username.clone(),
            uuid: s.uuid.clone(),
            access_token: s.access_token.clone(),
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
