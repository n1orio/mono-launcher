import type { AuthorTheme } from "~/lib/types";

/** Общие чистые утилиты без состояния. */

/** Сравнение версий вида «1.20.1» (числовые сегменты, точки). */
export function verCmp(a: string, b: string): number {
  const pa = a.split(".").map((x) => parseInt(x, 10) || 0);
  const pb = b.split(".").map((x) => parseInt(x, 10) || 0);
  const n = Math.max(pa.length, pb.length);
  for (let i = 0; i < n; i++) {
    const da = pa[i] ?? 0;
    const db = pb[i] ?? 0;
    if (da !== db) return da - db;
  }
  return 0;
}

/** Первая буква — заглавная (для подписей загрузчиков). */
export function cap(s: string): string {
  return s.length ? s[0].toUpperCase() + s.slice(1) : s;
}

/**
 * Обезвреживает SVG из ненадёжного источника перед вставкой через `v-html`.
 * Внешние SVG (например QR из device-code флоу) могут нести скрипты,
 * обработчики событий и `javascript:`-ссылки — вырезаем их.
 */
export function sanitizeSvg(svg: string): string {
  let s = svg
    .replace(/<\?xml[\s\S]*?\?>/gi, "")
    .replace(/<script[\s\S]*?<\/script>/gi, "")
    .replace(/<foreignObject[\s\S]*?<\/foreignObject>/gi, "")
    .replace(/<iframe[\s\S]*?<\/iframe>/gi, "");
  // Убираем обработчики событий (` onclick="..."` и т.п.) во всех тегах.
  s = s.replace(
    /\son[a-z]+\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)/gi,
    ""
  );
  // Блокируем javascript:/data:  в href / xlink:href.
  s = s.replace(
    /\s(xlink:href|href)\s*=\s*("(?:javascript|data):[^"]*"|'(?:javascript|data):[^']*')/gi,
    ""
  );
  return s;
}

function clamp(n: number): number {
  return Math.max(0, Math.min(255, Math.round(n)));
}

function toHex(n: number): string {
  return clamp(n).toString(16).padStart(2, "0");
}

/** Нормализует hex-цвет в `#rrggbb` (или возвращает `null`, если не похоже на hex). */
export function normalizeHex(value: string): string | null {
  let s = value.trim().replace(/^#/, "");
  if (/^[0-9a-f]{3}$/i.test(s)) s = s[0] + s[0] + s[1] + s[1] + s[2] + s[2];
  if (!/^[0-9a-f]{6}$/i.test(s)) return null;
  return `#${s.toLowerCase()}`;
}

/** Парсит `#rrggbb` в [r,g,b] (0..255) или null. */
function rgb(hex: string): [number, number, number] | null {
  const n = normalizeHex(hex);
  if (!n) return null;
  return [parseInt(n.slice(1, 3), 16), parseInt(n.slice(3, 5), 16), parseInt(n.slice(5, 7), 16)];
}

/** Смешивает цвет с белым/чёрным: amt > 0 светлее, amt < 0 темнее. */
function shade(hex: string, amt: number): string {
  const c = rgb(hex);
  if (!c) return hex;
  const t = amt > 0 ? 255 : 0;
  return `#${c.map((v) => toHex(v + (t - v) * Math.abs(amt))).join("")}`;
}

/**
 * Генерирует акцентную тему из одного цвета (семейство accent-переменных).
 * Нейтральные ключи (bg/panel/.../tx*) оставлены пустыми, чтобы лаунчер
 * подставлял текущий светлый/тёмный базовый цвет.
 */
/** Смешивает два цвета: t=0 → a, t=1 → b. */
function mix(a: string, b: string, t: number): string {
  const ca = rgb(a);
  const cb = rgb(b);
  if (!ca || !cb) return b;
  return `#${ca.map((v, i) => toHex(v + (cb[i] - v) * t)).join("")}`;
}

/**
 * Генерирует полную тему из одного акцентного цвета.
 * Нейтральные поверхности (bg/panel/.../tx*) подбираются под яркость акцента:
 * тёмный акцент → тёмные поверхности со светлым текстом, светлый → наоборот.
 * Все ключи заполнены, поэтому в автотеме не остаётся «чёрных» полей.
 */
export function themeFromAccent(hex: string): AuthorTheme | null {
  const n = normalizeHex(hex);
  const c = rgb(hex);
  if (!n || !c) return null;
  const dark = 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2] < 110;
  const base = {
    accent: n,
    accentStrong: shade(n, 0.35),
    accentHover: shade(n, 0.12),
    accentDeep: shade(n, -0.18),
  };
  if (dark) {
    return {
      ...base,
      bg: mix("#14141a", n, 0.12),
      panel: mix("#1e1e26", n, 0.08),
      input: mix("#26262f", n, 0.08),
      border: mix("#33333e", n, 0.07),
      tx: "#e6e6ec",
      txStrong: "#ffffff",
      txMuted: "#9a9aa6",
    };
  }
  return {
    ...base,
    bg: mix("#f5f5f8", n, 0.06),
    panel: "#ffffff",
    input: "#ffffff",
    border: mix("#e0e0e8", n, 0.08),
    tx: "#1c1c24",
    txStrong: "#0b0b10",
    txMuted: "#6c6c78",
  };
}
