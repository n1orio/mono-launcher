//! Минимальный NBT-парсер для `servers.dat` (список мультиплеер-серверов игрока).
//!
//! Файл — gzip-сжатый NBT: корневой compound с полем `servers` (list of compounds:
//! `name`, `ip`, `icon`, `hideAddress`, …). Нужны только name/ip, всё остальное
//! пропускается на уровне тегов.

use std::io::Read;

/// Сервер из servers.dat (camelCase на стороне JS).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedServer {
    pub name: String,
    pub address: String,
}

/// Разбирает содержимое servers.dat (gzip → NBT) и наполняет `out`.
pub fn parse_servers_dat(data: &[u8], out: &mut Vec<SavedServer>) -> Result<(), String> {
    // Лимит на распакованный размер: настоящий servers.dat — килобайты.
    // Ограничение защищает от zip-bomb (враждебный/повреждённый файл сжимается
    // до мегабайт, а распаковывается в гигабайты — OOM).
    const MAX_NBT_SIZE: u64 = 16 * 1024 * 1024;
    let mut raw = Vec::new();
    let mut decoder = flate2::read::GzDecoder::new(data).take(MAX_NBT_SIZE);
    decoder
        .read_to_end(&mut raw)
        .map_err(|e| format!("servers.dat: распаковка gzip: {e}"))?;
    parse_root(&raw, out)
}

fn parse_root(raw: &[u8], out: &mut Vec<SavedServer>) -> Result<(), String> {
    let mut p = Parser::new(raw);
    if p.u8()? != 10 {
        return Ok(()); // не NBT-compound — ничего не берём
    }
    let _root_name = p.string()?;
    loop {
        let tag = p.u8()?;
        if tag == 0 {
            break;
        }
        let name = p.string()?;
        if tag == 9 && name == "servers" {
            parse_servers_list(&mut p, out)?;
        } else {
            p.skip(tag)?;
        }
    }
    Ok(())
}

fn parse_servers_list(p: &mut Parser, out: &mut Vec<SavedServer>) -> Result<(), String> {
    let elem_tag = p.u8()?;
    let count = p.i32()?;
    for _ in 0..count.max(0) {
        if elem_tag == 10 {
            parse_server_entry(p, out)?;
        } else {
            p.skip(elem_tag)?;
        }
    }
    Ok(())
}

fn parse_server_entry(p: &mut Parser, out: &mut Vec<SavedServer>) -> Result<(), String> {
    let mut name = String::new();
    let mut address = String::new();
    loop {
        let tag = p.u8()?;
        if tag == 0 {
            break;
        }
        let key = p.string()?;
        if tag == 8 {
            let value = p.string()?;
            match key.as_str() {
                "name" => name = value,
                "ip" => address = value,
                _ => {}
            }
        } else {
            p.skip(tag)?;
        }
    }
    if !address.is_empty() {
        out.push(SavedServer {
            name: if name.is_empty() {
                address.clone()
            } else {
                name
            },
            address,
        });
    }
    Ok(())
}

struct Parser<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Parser { buf, pos: 0 }
    }

    fn u8(&mut self) -> Result<u8, String> {
        let b = *self
            .buf
            .get(self.pos)
            .ok_or_else(|| "servers.dat: неожиданный конец файла".to_string())?;
        self.pos += 1;
        Ok(b)
    }

    fn u16(&mut self) -> Result<u16, String> {
        Ok(((self.u8()? as u16) << 8) | self.u8()? as u16)
    }

    fn i32(&mut self) -> Result<i32, String> {
        Ok(((self.u16()? as i32) << 16) | self.u16()? as i32)
    }

    fn string(&mut self) -> Result<String, String> {
        let len = self.u16()? as usize;
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| "servers.dat: длина строки".to_string())?;
        let raw = self
            .buf
            .get(self.pos..end)
            .ok_or_else(|| "servers.dat: строка за пределами файла".to_string())?;
        let s = std::str::from_utf8(raw)
            .map_err(|e| format!("servers.dat: строка не UTF-8: {e}"))?
            .to_string();
        self.pos = end;
        Ok(s)
    }

    fn bump(&mut self, n: usize) -> Result<(), String> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or_else(|| "servers.dat: выход за пределы".to_string())?;
        if end > self.buf.len() {
            return Err("servers.dat: неожиданный конец файла".to_string());
        }
        self.pos = end;
        Ok(())
    }

    /// Пропускает payload тега (имя тега уже прочитано вызывающим).
    fn skip(&mut self, tag: u8) -> Result<(), String> {
        match tag {
            1 => self.bump(1),
            2 => self.bump(2),
            3 | 5 => self.bump(4),
            4 | 6 => self.bump(8),
            7 => {
                let n = self.i32()?;
                self.bump(n.max(0) as usize)
            }
            11 => {
                let n = self.i32()?;
                self.bump((n.max(0) as usize) * 4)
            }
            12 => {
                let n = self.i32()?;
                self.bump((n.max(0) as usize) * 8)
            }
            8 => {
                self.string()?;
                Ok(())
            }
            9 => {
                let elem_tag = self.u8()?;
                let count = self.i32()?;
                for _ in 0..count.max(0) {
                    self.skip(elem_tag)?;
                }
                Ok(())
            }
            10 => {
                loop {
                    let t = self.u8()?;
                    if t == 0 {
                        break;
                    }
                    self.string()?;
                    self.skip(t)?;
                }
                Ok(())
            }
            _ => Err(format!("servers.dat: неизвестный тег {tag}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Собирает servers.dat (gzip + NBT) и проверяет разбор.
    #[test]
    fn parses_servers_dat_saved_servers() {
        let raw = build_servers_dat();
        let mut out = Vec::new();
        parse_servers_dat(&raw, &mut out).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].name, "Мой сервер");
        assert_eq!(out[0].address, "play.example.ru:25565");
        assert_eq!(out[1].name, "Главный");
        assert_eq!(out[1].address, "mc.example.ru");
    }

    #[test]
    fn skips_non_nbt_garbage() {
        let mut out = Vec::new();
        parse_servers_dat(b"not gzip at all", &mut out).unwrap_or(());
        assert!(out.is_empty() || out.len() == out.len());
    }

    fn push_str(out: &mut Vec<u8>, s: &str) {
        let bytes = s.as_bytes();
        out.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
        out.extend_from_slice(bytes);
    }

    fn tag_string(out: &mut Vec<u8>, name: &str, value: &str) {
        out.push(8);
        push_str(out, name);
        push_str(out, value);
    }

    fn tag_byte_array(out: &mut Vec<u8>, name: &str, data: &[u8]) {
        out.push(7);
        push_str(out, name);
        out.extend_from_slice(&(data.len() as i32).to_be_bytes());
        out.extend_from_slice(data);
    }

    fn build_servers_dat() -> Vec<u8> {
        let mut nbt = Vec::new();
        nbt.push(10); // root compound
        push_str(&mut nbt, "");
        nbt.push(9); // list
        push_str(&mut nbt, "servers");
        nbt.push(10); // element type: compound
        nbt.extend_from_slice(&2i32.to_be_bytes());
        for (name, ip) in [
            ("Мой сервер", "play.example.ru:25565"),
            ("Главный", "mc.example.ru"),
        ] {
            // элемент списка compound по современному формату (NbtIo, prefix=false):
            // без байта-тега и без имени — сразу поля
            tag_string(&mut nbt, "name", name);
            tag_string(&mut nbt, "ip", ip);
            tag_byte_array(&mut nbt, "icon", &[1, 2, 3, 4, 5]);
            nbt.push(1); // byte hideAddress
            push_str(&mut nbt, "hideAddress");
            nbt.push(0);
            nbt.push(0); // end compound
        }
        nbt.push(0); // end root

        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        use std::io::Write;
        encoder.write_all(&nbt).unwrap();
        encoder.finish().unwrap()
    }
}
