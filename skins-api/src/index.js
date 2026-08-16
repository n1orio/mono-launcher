/**
 * Mono Launcher — публичный скин-сервис (оффлайн-скины).
 * Хранение: Cloudflare KV. Совместим с authlib-injector (yggdrasil-lite).
 *
 * Ключи KV:
 *   skin:<ник>   -> JSON { model, updated }        (метаданные)
 *   png:<ник>    -> ArrayBuffer PNG                (само изображение)
 *   uuid:<uuid>  -> ник (для поиска по uuid)
 */

const PNG_MAGIC = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
const MAX_PNG_BYTES = 3 * 1024 * 1024;
const NICK_RE = /^[a-zA-Z0-9_]{1,16}$/;

// UUID v3 (MD5, пространство имён DNS) — как в лаунчере (src-tauri/src/auth.rs).
const NS_DNS = [0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8];

function offlineUuid(name) {
  const md5 = md5Hex(concatBytes(NS_DNS, utf8(name.toLowerCase())));
  const b = hexToBytes(md5);
  b[6] = (b[6] & 0x0f) | 0x30;
  b[8] = (b[8] & 0x3f) | 0x80;
  return fmtUuid(b);
}

function fmtUuid(b) {
  const h = bytesToHex(b);
  return `${h.slice(0, 8)}-${h.slice(8, 12)}-${h.slice(12, 16)}-${h.slice(16, 20)}-${h.slice(20)}`;
}

function texturesPayload(base, name, model) {
  const uuid = offlineUuid(name);
  const skin = { url: `${base}/skins/${encodeURIComponent(name)}` };
  if (model === "slim") skin.metadata = { model: "slim" };
  return {
    timestamp: Date.now(),
    profileId: uuid.replace(/-/g, ""),
    profileName: name,
    textures: { SKIN: skin },
  };
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const path = url.pathname;
    const kv = env.SKINS;
    const base = `${url.protocol}//${url.host}`;

    if (request.method === "GET" && path === "/health") {
      return json({ ok: true });
    }

    // ---- загрузка / раздача скина ----
    const skinMatch = path.match(/^\/skins\/([^/]+)$/);
    if (skinMatch) {
      const name = skinMatch[1];
      if (!NICK_RE.test(name)) return json({ error: "bad nick" }, 400);
      const key = name.toLowerCase();
      if (request.method === "PUT") {
        const body = new Uint8Array(await request.arrayBuffer());
        if (body.length === 0 || body.length > MAX_PNG_BYTES) {
          return json({ error: "png too big or empty" }, 413);
        }
        if (!isPng(body)) return json({ error: "not a png" }, 400);
        const model = (request.headers.get("X-Skin-Model") || "classic") === "slim" ? "slim" : "classic";
        const meta = { model, updated: Date.now() };
        await Promise.all([
          kv.put(`png:${key}`, body, { metadata: meta }),
          kv.put(`skin:${key}`, JSON.stringify(meta)),
          kv.put(`uuid:${offlineUuid(key)}`, key),
        ]);
        return json({ ok: true, name: key, model });
      }
      if (request.method === "GET") {
        const png = await kv.get(`png:${key}`, "arrayBuffer");
        if (!png) return new Response(null, { status: 204 });
        return new Response(png, {
          headers: { "Content-Type": "image/png", "Cache-Control": "public, max-age=300" },
        });
      }
      if (request.method === "DELETE") {
        await Promise.all([
          kv.delete(`png:${key}`),
          kv.delete(`skin:${key}`),
          kv.delete(`uuid:${offlineUuid(key)}`),
        ]);
        return json({ ok: true });
      }
      return json({ error: "method not allowed" }, 405);
    }

    // ---- textures-пакет по нику (для отладки и плагинов) ----
    const texMatch = path.match(/^\/skins\/([^/]+)\/textures$/);
    if (texMatch && request.method === "POST") {
      const name = texMatch[1];
      if (!NICK_RE.test(name)) return json({ error: "bad nick" }, 400);
      const meta = await getMeta(kv, name);
      return json(texturesPayload(base, name, meta?.model || "classic"));
    }

    // ---- authlib-injector: проверка API ----
    // authlib-injector первым делом дёргает сам «корень» API-URL и читает оттуда
    // метаданные (либо путь из подсказки). Отдаём их и на «/», и на checker — так
    // любой порядок пробинга резолвится без 404/FileNotFoundException.
    if ((path === "/api/authlib-injector/checker" || path === "/") && request.method === "GET") {
      return json(
        { meta: { serverName: "Mono Launcher Skins", implementationName: "nio-skins", implementationVersion: "1.0.0" } },
        200,
        { "Authlib-Injector-API-Location": "/api/yggdrasil" }
      );
    }

    // ---- yggdrasil: вход (оффлайн: любой пароль) ----
    if (path === "/api/yggdrasil/authenticate" && request.method === "POST") {
      let body = {};
      try { body = await request.json(); } catch {}
      const name = String(body.username || "").trim();
      if (!name) return json({ error: "username required" }, 400);
      const uuid = offlineUuid(name);
      const profile = { id: uuid, name };
      return json({
        clientToken: body.clientToken || "",
        accessToken: uuid,
        user: { id: uuid, properties: [] },
        availableProfiles: [profile],
        selectedProfile: profile,
      });
    }
    if (path === "/api/yggdrasil/refresh" && request.method === "POST") {
      // Сессии не храним: просим клиента переавторизоваться.
      return new Response(null, { status: 410 });
    }
    if (path === "/api/yggdrasil/validate" && request.method === "POST") {
      return new Response(null, { status: 204 });
    }
    if (path === "/api/yggdrasil/sessionserver/session/minecraft/join" && request.method === "POST") {
      return new Response(null, { status: 204 });
    }

    // ---- yggdrasil: проверка входа (для сервера, hasJoined) ----
    if (path === "/api/yggdrasil/sessionserver/session/minecraft/hasJoined" && request.method === "GET") {
      const name = url.searchParams.get("username") || "";
      if (!NICK_RE.test(name)) return new Response(null, { status: 204 });
      const meta = await getMeta(kv, name);
      const props = [];
      if (meta) {
        props.push({
          name: "textures",
          value: btoa(JSON.stringify(texturesPayload(base, name, meta.model))),
        });
      }
      const uuid = offlineUuid(name);
      return json({ id: uuid.replace(/-/g, ""), name, properties: props }, 200, { "Cache-Control": "no-store" });
    }

    // ---- yggdrasil: профиль по UUID (клиент и сервер) ----
    const profMatch = path.match(/^\/api\/yggdrasil\/sessionserver\/session\/minecraft\/profile\/([0-9a-f-]+)$/i);
    if (profMatch && request.method === "GET") {
      const uuid = profMatch[1].toLowerCase();
      const name = await kv.get(`uuid:${uuid}`);
      if (!name) return json({ error: "player not found" }, 204);
      const meta = await getMeta(kv, name);
      const props = [];
      if (meta) {
        props.push({
          name: "textures",
          value: btoa(JSON.stringify(texturesPayload(base, name, meta.model))),
        });
      }
      return json({ id: uuid.replace(/-/g, ""), name, properties: props }, 200, { "Cache-Control": "no-store" });
    }
    if (path === "/api/yggdrasil/api/profiles/minecraft" && request.method === "POST") {
      let body = { profiles: [] };
      try { body = await request.json(); } catch {}
      const list = Array.isArray(body.profiles) ? body.profiles : [];
      const out = list
        .map((p) => ({ name: typeof p === "string" ? p : p?.name, id: offlineUuid(typeof p === "string" ? p : p?.name || "") }))
        .filter((p) => NICK_RE.test(p.name));
      return json(out);
    }

    return json({ error: "not found" }, 404);
  },
};

async function getMeta(kv, name) {
  const raw = await kv.get(`skin:${name.toLowerCase()}`);
  if (!raw) return null;
  try { return JSON.parse(raw); } catch { return null; }
}

function json(obj, status = 200, extra = {}) {
  return new Response(JSON.stringify(obj), {
    status,
    headers: { "Content-Type": "application/json; charset=utf-8", ...extra },
  });
}

function isPng(bytes) {
  for (let i = 0; i < PNG_MAGIC.length; i++) {
    if (bytes[i] !== PNG_MAGIC[i]) return false;
  }
  return true;
}

// ---- MD5 (мини) ----
const S = [
  7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
  5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
  4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
  6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];
const K = new Array(64);
for (let i = 0; i < 64; i++) K[i] = Math.floor(Math.abs(Math.sin(i + 1)) * 4294967296);

function md5Hex(input) {
  const bytes = Array.from(input);
  const origLen = bytes.length * 8;
  bytes.push(0x80);
  while (bytes.length % 64 !== 56) bytes.push(0);
  const lenLo = origLen >>> 0;
  const lenHi = Math.floor(origLen / 4294967296);
  for (let i = 0; i < 4; i++) bytes.push((lenLo >>> (i * 8)) & 0xff);
  for (let i = 0; i < 4; i++) bytes.push((lenHi >>> (i * 8)) & 0xff);

  let a0 = 0x67452301, b0 = 0xefcdab89, c0 = 0x98badcfe, d0 = 0x10325476;
  for (let off = 0; off < bytes.length; off += 64) {
    const M = new Array(16);
    for (let i = 0; i < 16; i++) {
      let v = 0;
      for (let j = 3; j >= 0; j--) v = (v << 8) | bytes[off + i * 4 + j];
      M[i] = v >>> 0;
    }
    let A = a0, B = b0, C = c0, D = d0;
    for (let i = 0; i < 64; i++) {
      let F, g;
      if (i < 16) { F = (B & C) | (~B & D); g = i; }
      else if (i < 32) { F = (D & B) | (~D & C); g = (5 * i + 1) % 16; }
      else if (i < 48) { F = B ^ C ^ D; g = (3 * i + 5) % 16; }
      else { F = C ^ (B | ~D); g = (7 * i) % 16; }
      F = (F + A + K[i] + M[g]) >>> 0;
      const t = D; D = C; C = B;
      B = (B + ((F << S[i]) | (F >>> (32 - S[i])))) >>> 0;
      A = t;
    }
    a0 = (a0 + A) >>> 0; b0 = (b0 + B) >>> 0; c0 = (c0 + C) >>> 0; d0 = (d0 + D) >>> 0;
  }
  let out = "";
  for (const v of [a0, b0, c0, d0]) {
    for (let i = 0; i < 4; i++) out += ((v >>> (i * 8)) & 0xff).toString(16).padStart(2, "0");
  }
  return out;
}

function utf8(s) {
  return new TextEncoder().encode(s);
}
function concatBytes(a, b) {
  const out = new Uint8Array(a.length + b.length);
  out.set(a); out.set(b, a.length);
  return out;
}
function hexToBytes(hex) {
  const out = new Uint8Array(hex.length / 2);
  for (let i = 0; i < out.length; i++) out[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  return out;
}
function bytesToHex(bytes) {
  return Array.from(bytes).map((b) => b.toString(16).padStart(2, "0")).join("");
}