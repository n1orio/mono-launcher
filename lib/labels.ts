/** Локализованные подписи для фаз установки, архитектуры Java и языков. */
import type { LocaleMeta } from "~/composables/useI18n";

export type TranslateFn = (key: string, params?: Record<string, string | number | unknown>) => string;

const PHASE_KEYS: Record<string, string> = {
  "Подготовка...": "phase.prepare",
  "Скачивание сборки": "phase.download",
  "Распаковка архива": "phase.extract",
  "Установка модов": "phase.mods",
  "Применение overrides": "phase.overrides",
};

const ARCH_KEYS: Record<string, string> = {
  "64-бит": "java.arch64",
  "32-бит": "java.arch32",
  "недоступна": "java.archUnknown",
};

export function phaseLabel(phase: string, t: TranslateFn): string {
  const key = PHASE_KEYS[phase];
  return key ? t(key) : phase;
}

export function javaArchLabel(arch: string, t: TranslateFn): string {
  const key = ARCH_KEYS[arch];
  return key ? t(key) : arch;
}

/** Подпись языка: «ru — автор · v0.3.0» (для кнопки языка). */
export function localeLabel(
  code: string,
  getMeta: (code: string) => LocaleMeta,
): string {
  const meta = getMeta(code);
  const extra = [meta.author, meta.version ? `v${meta.version}` : ""]
    .filter(Boolean)
    .join(" · ");
  return extra ? `${code} — ${extra}` : code;
}