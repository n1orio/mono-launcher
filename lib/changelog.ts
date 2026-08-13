import { isTauri, openExternal } from "~/lib/bridge";

/** Строка ченджлога: маркер типа + текст без приставки (`- `, `#`, ...). */
export interface ChangelogLine {
  type: "bullet" | "body" | "text";
  text: string;
}

/** Сколько строк показывать до «Показать всё». */
export const CHANGELOG_PREVIEW_LINES = 8;

/** Разбирает markdown-ченджлог на строки с типом (буллеты/заголовки/текст). */
export function changelogLines(body: string): ChangelogLine[] {
  if (!body) return [];
  return body
    .split("\n")
    .map((raw) => {
      const line = raw.replace(/\r$/, "").trim();
      if (!line) return null;
      if (/^[-*]\s+/.test(line)) {
        return { type: "bullet" as const, text: line.replace(/^[-*]\s+/, "") };
      }
      if (/^#+\s+/.test(line)) {
        return { type: "body" as const, text: line.replace(/^#+\s+/, "") };
      }
      return { type: "text" as const, text: line };
    })
    .filter((l): l is ChangelogLine => l !== null);
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function inlineStyle(s: string): string {
  return s
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
    .replace(/`([^`]+)`/g, "<code>$1</code>")
    .replace(/\*([^*]+)\*/g, "<em>$1</em>")
    .replace(/~~([^~]+)~~/g, "<del>$1</del>");
}

/** Рендерит inline-markdown строки (ссылки, жирный, код, зачёркнутый) в HTML. */
export function renderInline(raw: string): string {
  let t = escapeHtml(raw.trim());
  t = inlineStyle(t);
  // Ссылки [текст](http...)
  t = t.replace(
    /\[([^\]]+)\]\((https?:\/\/[^\s)"<>]+)\)/g,
    (_, text: string, url: string) => `<a href="${url}">${text}</a>`
  );
  // Голые ссылки (минуя href уже вставленных <a>)
  t = t.replace(/(https?:\/\/[^\s)"<>]+)/g, (m: string, _g: string, offset: number) => {
    const before = t.slice(0, offset);
    const opens = before.match(/<a /g)?.length ?? 0;
    const closes = before.match(/<\/a>/g)?.length ?? 0;
    return opens > closes ? m : `<a href="${m}">${m}</a>`;
  });
  return t;
}

/** Открывает внешнюю ссылку из в-html разметки (клик по <a> внутри changelog). */
export function onChangelogLinkClick(e: MouseEvent) {
  const target = e.target as HTMLElement;
  const anchor = target.closest("a");
  if (!anchor) return;
  const href = anchor.getAttribute("href");
  if (!href || !/^https?:\/\//i.test(href)) return;
  e.preventDefault();
  if (isTauri()) {
    openExternal(href).catch(() => window.open(href, "_blank"));
  } else {
    window.open(href, "_blank");
  }
}