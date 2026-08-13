/** Чистые функции форматирования чисел/дат/размеров для UI. */

export type TranslateFn = (key: string, params?: Record<string, string | number | unknown>) => string;

export function formatBytes(bytes: number, t: TranslateFn): string {
  if (bytes <= 0) return `0 ${t("units.b")}`;
  const units = [t("units.b"), t("units.kb"), t("units.mb"), t("units.gb")];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  return `${(bytes / Math.pow(1024, i)).toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export function formatPlaytime(seconds: number, t: TranslateFn): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (h > 0) return t("time.hm", { h, m });
  return t("time.min", { m });
}

export function formatDate(iso: string | null, locale: string): string {
  if (!iso) return "";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "";
  return d.toLocaleDateString(locale === "en" ? "en-US" : "ru-RU", {
    day: "2-digit",
    month: "short",
    year: "numeric",
  });
}

/** Дата из unix-секунд (для лицензий) в локальном формате. */
export function formatUnixDate(epoch: number | null, locale: string): string {
  if (!epoch) return "";
  return formatDate(new Date(epoch * 1000).toISOString(), locale);
}

export function formatPlaytimeShort(seconds: number, t: TranslateFn): string {
  const h = Math.floor(seconds / 3600);
  if (h >= 1) return `${h} ${t("units.h")}`;
  return `${Math.max(1, Math.round(seconds / 60))} ${t("units.min")}`;
}