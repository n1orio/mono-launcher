import { ref } from "vue";

/**
 * Метаданные перевода. Лежат в корне файла локали как "__meta__":
 *   {
 *     "__meta__": { "author": "n1orio", "version": "1.0.0" },
 *     "nav.news": "Новости",
 *     ...
 *   }
 * `author` — кто перевёл, `version` — версия лаунчера, для которой актуален перевод.
 */
export interface LocaleMeta {
  /** Автор перевода (ник/имя). */
  author?: string;
  /** Версия лаунчера, для которой актуален перевод. */
  version?: string;
}

type LocaleFile = Record<string, string> & { __meta__?: LocaleMeta };

interface LocaleInfo {
  code: string;
  dict: Record<string, string>;
  meta: LocaleMeta;
}

/**
 * Все локали из папки locales/*.json подхватываются автоматически:
 * положил новый файл перевода — язык появился в UI, ничего больше править не нужно.
 */
const modules = import.meta.glob("../locales/*.json", { eager: true }) as Record<
  string,
  unknown
>;

const dicts: Record<string, LocaleInfo> = {};
for (const [path, raw] of Object.entries(modules)) {
  const code = path.split("/").pop()?.replace(/\.json$/, "") ?? "";
  if (!code) continue;
  const file = (
    raw && typeof raw === "object" && "default" in (raw as object)
      ? (raw as { default: LocaleFile }).default
      : raw
  ) as LocaleFile;
  const { __meta__, ...strings } = file;
  dicts[code] = { code, dict: strings, meta: __meta__ ?? {} };
}

/** Коды доступных языков (в порядке файлов в папке locales). */
export const locales: string[] = Object.keys(dicts);

export type Locale = string;

const STORAGE_KEY = "mono.language";
const DEFAULT_LOCALE: Locale = dicts.ru ? "ru" : (locales[0] ?? "");

/** Метаданные языка (автор перевода, версия) — для показа в UI. */
export function getLocaleMeta(code: string): LocaleMeta {
  return dicts[code]?.meta ?? {};
}

const locale = ref<Locale>(DEFAULT_LOCALE);

function loadLocale(): Locale {
  if (typeof window === "undefined") return DEFAULT_LOCALE;
  try {
    const saved = window.localStorage.getItem(STORAGE_KEY);
    if (saved && saved in dicts) return saved;
  } catch {
    // приватный режим браузера
  }
  return DEFAULT_LOCALE;
}

locale.value = loadLocale();

export function useI18n() {
  function setLocale(l: Locale) {
    locale.value = l;
    if (typeof window === "undefined") return;
    try {
      window.localStorage.setItem(STORAGE_KEY, l);
    } catch {
      // приватный режим браузера
    }
  }

  function t(key: string, params?: Record<string, string | number | unknown>): string {
    const info = dicts[locale.value] ?? dicts[DEFAULT_LOCALE];
    let s = info?.dict[key] ?? (dicts.ru?.dict[key] as string | undefined) ?? key;
    if (params) {
      for (const [k, v] of Object.entries(params)) {
        s = s.replaceAll(`{${k}}`, String(v));
      }
    }
    return s;
  }

  return { locale, locales, setLocale, t };
}
