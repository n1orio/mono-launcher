/** Персистентный кеш иконок (base64/JSON-строки) в localStorage.
 *  Иконки модов/ресурспаков отдаются мгновенно из кеша, а устаревшие записи
 *  обновляются фоном — при следующем показе списка. */

type CachedIcon = { d: string; t: number };

const KEY = "mono.iconCache";
const MAX_ENTRIES = 600;
const TTL_MS = 7 * 24 * 3600 * 1000;

let mem: Record<string, CachedIcon> | null = null;

function read(): Record<string, CachedIcon> {
  if (mem) return mem;
  mem = {};
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return mem;
    const obj = JSON.parse(raw) as Record<string, unknown>;
    if (typeof obj !== "object" || obj === null || Array.isArray(obj)) return mem;
    for (const [k, v] of Object.entries(obj)) {
      const e = v as CachedIcon;
      if (typeof e?.d === "string" && typeof e?.t === "number") mem[k] = e;
    }
  } catch {
    /* битый кеш — просто начнём заново */
  }
  return mem;
}

function write() {
  if (!mem) return;
  try {
    const entries = Object.entries(mem)
      .sort((a, b) => b[1].t - a[1].t)
      .slice(0, MAX_ENTRIES);
    mem = Object.fromEntries(entries);
    localStorage.setItem(KEY, JSON.stringify(mem));
  } catch {
    /* квота localStorage — кеш просто перестанет сохраняться */
  }
}

/** Возвращает закешированное значение и признак «устарело» (нужно фоновое обновление). */
export function getCachedIcon(key: string): { data: string; stale: boolean } | null {
  const e = read()[key];
  if (!e) return null;
  return { data: e.d, stale: Date.now() - e.t > TTL_MS };
}

export function setCachedIcon(key: string, data: string) {
  read()[key] = { d: data, t: Date.now() };
  write();
}

export function clearCachedIcons() {
  mem = {};
  try {
    localStorage.removeItem(KEY);
  } catch {
    /* ignore */
  }
}