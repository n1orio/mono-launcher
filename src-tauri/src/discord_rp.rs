use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use discord_rich_presence::{
    activity::{Activity, Timestamps},
    DiscordIpc, DiscordIpcClient,
};

pub const DISCORD_APP_ID: &str = "1536155073170768013";

/// Язык интерфейса (ru/en), выбранный в настройках лаунчера.
static LOCALE: Mutex<String> = Mutex::new(String::new());

pub fn set_locale(locale: String) {
    match LOCALE.lock() {
        Ok(mut l) => *l = locale,
        Err(e) => *e.into_inner() = locale,
    }
}

fn is_russian() -> bool {
    LOCALE.lock().map(|l| l.as_str() != "en").unwrap_or(true)
}

/// Состояние Discord IPC-клиента (защищено Mutex'ом, живёт всё время работы лаунчера).
pub struct RpClient {
    client: Option<DiscordIpcClient>,
    details: String,
    start_ts: i64,
}

impl RpClient {
    pub fn new() -> Self {
        Self {
            client: None,
            details: String::new(),
            start_ts: 0,
        }
    }

    /// Подключается к Discord и показывает активность «Играет в …».
    /// Никогда не падает: Discord не запущен / нет сети → тихо пропускаем.
    pub fn start(&mut self, details: impl Into<String>, state: impl Into<String>) {
        self.stop();
        if !crate::config::discord_rp_enabled() {
            return;
        }
        let mut client = DiscordIpcClient::new(DISCORD_APP_ID);
        if client.connect().is_err() {
            #[cfg(debug_assertions)]
            eprintln!("discord-rp: discord не запущен или IPC недоступен");
            return;
        }
        self.details = details.into();
        self.start_ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let payload = activity_payload(&self.details, &state.into(), self.start_ts);
        if client.set_activity(payload).is_err() {
            #[cfg(debug_assertions)]
            eprintln!("discord-rp: set_activity не удался, отключаюсь");
            self.client = None;
            let _ = client.close();
            return;
        }
        self.client = Some(client);
    }

    /// Обновляет вторую строку статуса (например, наигранное время) без переподключения.
    pub fn update_state(&mut self, state: impl Into<String>) {
        let state = state.into();
        // Discord мог запуститься уже после старта игры — пробуем переподключиться.
        if self.client.is_none() {
            if crate::config::discord_rp_enabled() && !self.details.is_empty() {
                self.start(self.details.clone(), state);
            }
            return;
        }
        let client = match &mut self.client {
            Some(c) => c,
            None => return,
        };
        let payload = activity_payload(&self.details, &state, self.start_ts);
        if client.set_activity(payload).is_err() {
            #[cfg(debug_assertions)]
            eprintln!("discord-rp: set_activity не удался, отключаюсь");
            self.stop();
        }
    }

    /// Скрывает активность и отключается от Discord (безопасно при ошибках).
    pub fn stop(&mut self) {
        if let Some(mut client) = self.client.take() {
            let _ = client.clear_activity();
            let _ = client.close();
        }
    }
}

fn activity_payload(details: &str, state: &str, start_ts: i64) -> Activity<'static> {
    Activity::new()
        .details(details.to_owned())
        .state(state.to_owned())
        .timestamps(Timestamps::new().start(start_ts))
}

/// Глобальный доступ к клиенту для callbacks из game.rs (без AppState).
static RP: Mutex<Option<RpClient>> = Mutex::new(None);

fn with_rp<T>(f: impl FnOnce(&mut RpClient) -> T) -> T {
    let mut option = RP.lock().unwrap_or_else(|e| e.into_inner());
    let guard = option.get_or_insert_with(RpClient::new);
    f(guard)
}

/// Показывает presence «Играет в …» (тумблер проверяется внутри).
pub fn start_presence(details: impl Into<String>, state: impl Into<String>) {
    with_rp(|rp| rp.start(details, state));
}

/// Обновляет вторую строку присутствия (наигранное время и т.п.).
pub fn update_presence(state: impl Into<String>) {
    with_rp(|rp| rp.update_state(state));
}

/// Гасит presence (при выходе из игры или выключении тумблера).
pub fn stop_presence() {
    with_rp(|rp| rp.stop());
}

/// Человекочитаемое наигранное время: «12 ч 34 мин» / «5 мин».
pub fn playtime_string(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    if is_russian() {
        if h > 0 {
            format!("{h} ч {m} мин")
        } else {
            format!("{m} мин")
        }
    } else if h > 0 {
        format!("{h} h {m} min")
    } else {
        format!("{m} min")
    }
}

/// Строка details: «Играет в <сборка>» / «Playing <pack>».
pub fn playing_details(pack_name: &str) -> String {
    if is_russian() {
        format!("Играет в {pack_name}")
    } else {
        format!("Playing {pack_name}")
    }
}

/// Строка state: «Наиграно в сборке: …» / «Played this pack: …».
pub fn played_state(seconds: u64) -> String {
    if is_russian() {
        format!("Наиграно в сборке: {}", playtime_string(seconds))
    } else {
        format!("Played this pack: {}", playtime_string(seconds))
    }
}
