import { ref } from "vue";
import ruJson from "../locales/ru.json";
import enJson from "../locales/en.json";
import ukJson from "../locales/uk.json";

export type Locale = "ru" | "en" | "uk";

const STORAGE_KEY = "mono.language";
const DEFAULT_LOCALE: Locale = "ru";

const ru: Record<string, string> = ruJson;
const en: Record<string, string> = enJson;
const uk: Record<string, string> = ukJson;

const dicts: Record<Locale, Record<string, string>> = { ru, en, uk };


const locale = ref<Locale>(DEFAULT_LOCALE);

function loadLocale(): Locale {
  if (typeof window === "undefined") return DEFAULT_LOCALE;
  try {
    const saved = window.localStorage.getItem(STORAGE_KEY);
    if (saved === "ru" || saved === "en" || saved === "uk") return saved;
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
    const dict = dicts[locale.value];
    let s = dict[key] ?? ru[key] ?? key;
    if (params) {
      for (const [k, v] of Object.entries(params)) {
        s = s.replaceAll(`{${k}}`, String(v));
      }
    }
    return s;
  }

  return { locale, setLocale, t };
}
