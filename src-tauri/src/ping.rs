//! Пинг Minecraft-серверов: современный статус (1.7+ handshake + status request)
//! с фолбэком на legacy-пинг 0xFE для старых серверов.
//!
//! Реализован вручную, без сетевых крейтов поверх tokio: varint-кодирование,
//! два пакета туда, один ответ (packet id 0x00 + JSON).

use std::time::Instant;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

const PING_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(4);
const MAX_RESPONSE: i32 = 32767;

/// Статус сервера, отдаваемый в UI (camelCase на стороне JS).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    pub online: bool,
    pub version: Option<String>,
    pub motd: Option<String>,
    pub players_online: Option<u16>,
    pub players_max: Option<u16>,
    /// Никнеймы игроков из `players.sample` (может быть пустым — сервер не шлёт).
    pub players: Vec<String>,
    pub latency_ms: Option<u64>,
}

#[derive(Debug, Default)]
struct PingData {
    version: Option<String>,
    motd: Option<String>,
    players_online: Option<u16>,
    players_max: Option<u16>,
    players: Vec<String>,
}

/// Пинг сервера: сначала 1.7+, при любой неудаче — legacy 0xFE.
pub async fn ping_server(address: &str, port: u16) -> Result<ServerStatus, String> {
    let start = Instant::now();
    let data = match timeout(PING_TIMEOUT, ping_modern(address, port)).await {
        Ok(Ok(data)) => data,
        Ok(Err(_)) | Err(_) => match timeout(PING_TIMEOUT, ping_legacy(address, port)).await {
            Ok(Ok(data)) => data,
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err("Сервер не ответил (таймаут)".to_string()),
        },
    };
    Ok(ServerStatus {
        online: true,
        latency_ms: Some(start.elapsed().as_millis() as u64),
        version: data.version,
        motd: data.motd,
        players_online: data.players_online,
        players_max: data.players_max,
        players: data.players,
    })
}

/// Протокол 1.7+: handshake (next state = status) + status request.
async fn ping_modern(address: &str, port: u16) -> Result<PingData, String> {
    let mut stream = TcpStream::connect((address, port))
        .await
        .map_err(|e| format!("Нет соединения: {e}"))?;

    let mut payload = Vec::new();
    payload.push(0x00); // packet: handshake
    write_varint(&mut payload, 767); // protocol version (1.21.2): реальный номер,
                                     // не -1 — часть прокси на -1 отвечает id=-1
    write_string(&mut payload, address);
    payload.extend_from_slice(&port.to_be_bytes());
    write_varint(&mut payload, 1); // next state: status

    let mut frame = Vec::new();
    write_varint(&mut frame, payload.len() as i32);
    frame.extend_from_slice(&payload);
    frame.push(0x01); // длина status request (1 байт)
    frame.push(0x00); // packet: status request
    stream
        .write_all(&frame)
        .await
        .map_err(|e| format!("Запрос статуса: {e}"))?;

    let len = read_varint(&mut stream).await?;
    if len <= 0 || len > MAX_RESPONSE {
        return Err("Некорректный ответ сервера".to_string());
    }
    let mut buf = vec![0u8; len as usize];
    stream
        .read_exact(&mut buf)
        .await
        .map_err(|e| format!("Чтение ответа: {e}"))?;

    let mut rest: &[u8] = &buf;
    // packet id (обычно 0x00) пропускаем: старые прокси шлют и другие значения.
    let _ = take_varint(&mut rest)?;
    let json_len = take_varint(&mut rest)?;
    if json_len < 0 || json_len as usize > rest.len() {
        return Err("Некорректный ответ сервера".to_string());
    }
    let json = std::str::from_utf8(&rest[..json_len as usize])
        .map_err(|e| format!("Некорректный ответ сервера: {e}"))?;
    parse_status_json(json)
}

/// Legacy-пинг (beta 1.8 – 1.6): один байт 0xFE, ответ 0xFF + UTF-16BE строка.
async fn ping_legacy(address: &str, port: u16) -> Result<PingData, String> {
    let mut stream = TcpStream::connect((address, port))
        .await
        .map_err(|e| format!("Нет соединения: {e}"))?;
    stream
        .write_all(&[0xFE])
        .await
        .map_err(|e| format!("Запрос статуса: {e}"))?;

    let mut prefix = [0u8; 1];
    stream
        .read_exact(&mut prefix)
        .await
        .map_err(|e| format!("Чтение ответа: {e}"))?;
    let mut len_buf = [0u8; 2];
    stream
        .read_exact(&mut len_buf)
        .await
        .map_err(|e| format!("Чтение ответа: {e}"))?;
    let len = (u16::from_be_bytes(len_buf) as usize) * 2;
    if len == 0 || len > MAX_RESPONSE as usize {
        return Err("Некорректный ответ сервера".to_string());
    }
    let mut raw = vec![0u8; len];
    stream
        .read_exact(&mut raw)
        .await
        .map_err(|e| format!("Чтение ответа: {e}"))?;

    let units: Vec<u16> = raw
        .chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();
    let text = String::from_utf16_lossy(&units);
    let parts: Vec<&str> = text.split('\u{0}').collect();
    let parse = |i: usize| parts.get(i).and_then(|s| s.parse::<u16>().ok());

    // Классический ответ: "§1\0<protocol>\0<version>\0<motd>\0<online>\0<max>".
    if parts.first().is_some_and(|p| p.starts_with('\u{a7}')) && parts.len() >= 4 {
        Ok(PingData {
            version: parts.get(2).map(|s| s.to_string()),
            motd: parts.get(3).and_then(|m| {
                if m.is_empty() {
                    None
                } else {
                    Some(m.to_string())
                }
            }),
            players_online: parse(4),
            players_max: parse(5),
            players: Vec::new(),
        })
    } else {
        // Нетипичный ответ — считаем весь текст описанием.
        Ok(PingData {
            version: None,
            motd: if text.is_empty() { None } else { Some(text) },
            players_online: None,
            players_max: None,
            players: Vec::new(),
        })
    }
}

fn parse_status_json(raw: &str) -> Result<PingData, String> {
    #[derive(serde::Deserialize)]
    struct Status {
        version: Option<Version>,
        players: Option<Players>,
        description: Option<serde_json::Value>,
    }
    #[derive(serde::Deserialize)]
    struct Version {
        name: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct Players {
        online: Option<u16>,
        max: Option<u16>,
        sample: Option<Vec<Sample>>,
    }
    #[derive(serde::Deserialize)]
    struct Sample {
        name: Option<String>,
    }

    let s: Status = serde_json::from_str(raw).map_err(|e| format!("Некорректный JSON: {e}"))?;
    let motd = match s.description {
        Some(serde_json::Value::String(t)) => Some(t),
        Some(serde_json::Value::Object(obj)) => {
            if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                Some(text.to_string())
            } else { obj.get("translate").and_then(|v| v.as_str()).map(|key| format!("{{translate:{key}}}")) }
        }
        _ => None,
    };

    let players = s
        .players
        .as_ref()
        .and_then(|p| p.sample.as_ref())
        .map(|samples| {
            samples
                .iter()
                .filter_map(|s| s.name.clone())
                .take(12)
                .collect()
        })
        .unwrap_or_default();

    Ok(PingData {
        version: s.version.and_then(|v| v.name),
        motd: motd.filter(|m| !m.is_empty()),
        players_online: s.players.as_ref().and_then(|p| p.online),
        players_max: s.players.as_ref().and_then(|p| p.max),
        players,
    })
}

fn write_varint(buf: &mut Vec<u8>, val: i32) {
    // u32-представление: отрицательные значения кодируются как 5-байтовый
    // двухкомпонентный varint (например, protocol version -1).
    let mut v = val as u32;
    loop {
        let byte = (v & 0x7F) as u8;
        v >>= 7;
        if v != 0 {
            buf.push(byte | 0x80);
        } else {
            buf.push(byte);
            break;
        }
    }
}

fn write_string(buf: &mut Vec<u8>, s: &str) {
    write_varint(buf, s.len() as i32);
    buf.extend_from_slice(s.as_bytes());
}

/// Читает varint прямо из потока.
async fn read_varint(stream: &mut TcpStream) -> Result<i32, String> {
    let mut val: i32 = 0;
    for i in 0..5 {
        let mut byte = [0u8; 1];
        stream
            .read_exact(&mut byte)
            .await
            .map_err(|e| format!("Чтение ответа: {e}"))?;
        val |= ((byte[0] & 0x7F) as i32) << (i * 7);
        if byte[0] & 0x80 == 0 {
            return Ok(val);
        }
    }
    Err("Некорректный ответ сервера".to_string())
}

/// Откусывает varint с начала среза, двигая его.
fn take_varint(rest: &mut &[u8]) -> Result<i32, String> {
    let mut val: i32 = 0;
    for i in 0..5 {
        let Some(&byte) = rest.first() else {
            return Err("Некорректный ответ сервера".to_string());
        };
        *rest = &rest[1..];
        val |= ((byte & 0x7F) as i32) << (i * 7);
        if byte & 0x80 == 0 {
            return Ok(val);
        }
    }
    Err("Некорректный ответ сервера".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ручная проверка протокола против публичного сервера:
    /// `cargo test --lib -- --ignored pings_live_server`
    #[test]
    #[ignore = "требует сеть"]
    fn pings_live_server() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        eprintln!("pinging mc.holyworld.ru:25565…");
        let status = rt.block_on(ping_server("mc.holyworld.ru", 25565)).unwrap();
        assert!(status.online, "сервер должен ответить");
        println!("{status:?}");
    }
}
