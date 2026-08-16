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