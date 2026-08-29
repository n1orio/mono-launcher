import { ref } from "vue";
import { emitThemeChanged } from "~/lib/bridge";

const THEME_KEY = "mono.theme";

/** Палитра светлой темы. */
const THEME_LIGHT: Record<string, string> = {
  "--bg": "#f6f8fa",
  "--app-bg": "#eef1f4",
  "--panel": "#ffffff",
  "--panel-soft": "rgba(255, 255, 255, 0.6)",
  "--input": "#eef1f4",
  "--input-50": "rgba(238, 241, 244, 0.5)",
  "--hover": "#dbe1e8",
  "--border": "#d0d7de",
  "--tx": "#1f2328",
  "--tx-strong": "#111417",
  "--tx-muted": "#656d76",
  "--bg-60": "rgba(246, 248, 250, 0.6)",
  "--bg-30": "rgba(246, 248, 250, 0.8)",
  "--scrollbar": "#b6c2cf",
  "--scrollbar-hover": "#8c959f",
  "--nav-hover": "rgba(9, 30, 66, 0.06)",
  "--nav-active": "rgba(9, 30, 66, 0.09)",
  "--toast-shadow": "rgba(31, 35, 40, 0.2)",
  "--accent": "#58a6ff",
  "--accent-deep": "#1f6beb",
  "--accent-strong": "#79c0ff",
  "--accent-hover": "#388bfd",
};

/** Палитра тёмной темы. */
const THEME_DARK: Record<string, string> = {
  "--bg": "#05070c",
  "--app-bg": "#010308",
  "--panel": "#090c12",
  "--panel-soft": "rgba(5, 7, 12, 0.5)",
  "--input": "#0f131c",
  "--input-50": "rgba(15, 19, 28, 0.5)",
  "--hover": "#171c26",
  "--border": "#191e2a",
  "--tx": "#b3bdc9",
  "--tx-strong": "#e3ebf5",
  "--tx-muted": "#717b87",
  "--bg-60": "rgba(5, 7, 12, 0.6)",
  "--bg-30": "rgba(5, 7, 12, 0.3)",
  "--scrollbar": "#162e54",
  "--scrollbar-hover": "#234b8f",
  "--nav-hover": "rgba(255, 255, 255, 0.05)",
  "--nav-active": "rgba(255, 255, 255, 0.08)",
  "--toast-shadow": "rgba(0, 0, 0, 0.55)",
  "--accent": "#58a6ff",
  "--accent-deep": "#1f6beb",
  "--accent-strong": "#79c0ff",
  "--accent-hover": "#388bfd",
};

const TEXT_VARS = new Set(["--tx", "--tx-strong", "--tx-muted"]);

function parseColor(c: string): [number, number, number, number] {
  if (c.startsWith("#")) {
    const n = parseInt(c.slice(1), 16);
    return [(n >> 16) & 255, (n >> 8) & 255, n & 255, 1];
  }
  const m = c.match(/rgba?\(([^)]+)\)/);
  if (m) {
    const p = m[1].split(",").map((s) => parseFloat(s.trim()));
    return [p[0] ?? 0, p[1] ?? 0, p[2] ?? 0, p[3] ?? 1];
  }
  return [0, 0, 0, 1];
}

function rgbaStr([r, g, b, a]: [number, number, number, number]): string {
  const round = (x: number) => Math.max(0, Math.min(255, Math.round(x)));
  if (a >= 1) return `rgb(${round(r)}, ${round(g)}, ${round(b)})`;
  return `rgba(${round(r)}, ${round(g)}, ${round(b)}, ${Math.max(0, Math.min(1, a))})`;
}

const srgbToLinear = (c: number) =>
  c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);

const linearToSrgb = (c: number) => {
  const v = c <= 0.0031308 ? 12.92 * c : 1.055 * Math.pow(c, 1 / 2.4) - 0.055;
  return Math.max(0, Math.min(1, v));
};

function rgbToOklab([r, g, b]: number[]): [number, number, number] {
  const lr = srgbToLinear(r / 255);
  const lg = srgbToLinear(g / 255);
  const lb = srgbToLinear(b / 255);
  let l = 0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb;
  let m = 0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb;
  let s = 0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb;
  l = Math.cbrt(l);
  m = Math.cbrt(m);
  s = Math.cbrt(s);
  return [
    0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
    1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
    0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s,
  ];
}

function oklabToRgb([L, A, B]: number[]): [number, number, number] {
  const ll = L + 0.3963377774 * A + 0.2158037573 * B;
  const mm = L - 0.1055613458 * A - 0.0638541728 * B;
  const ss = L - 0.0894841775 * A - 1.291485548 * B;
  const l1 = ll * ll * ll;
  const m1 = mm * mm * mm;
  const s1 = ss * ss * ss;
  const r = 4.0767416621 * l1 - 3.3077115913 * m1 + 0.2309699292 * s1;
  const g = -1.2684380046 * l1 + 2.6097574011 * m1 - 0.3413193965 * s1;
  const b2 = -0.0041960863 * l1 - 0.7034186147 * m1 + 1.707614701 * s1;
  return [linearToSrgb(r) * 255, linearToSrgb(g) * 255, linearToSrgb(b2) * 255];
}

function mixColors(light: string, dark: string, t: number): string {
  const l = parseColor(light);
  const d = parseColor(dark);
  const lc = rgbToOklab([l[0], l[1], l[2]]);
  const dc = rgbToOklab([d[0], d[1], d[2]]);
  const lerp = (a: number, b: number) => a + (b - a) * t;
  const out = oklabToRgb([lerp(lc[0], dc[0]), lerp(lc[1], dc[1]), lerp(lc[2], dc[2])]);
  return rgbaStr([out[0], out[1], out[2], lerp(l[3], d[3])]);
}

function panelGrad(t: number): string {
  return `linear-gradient(180deg, ${mixColors("#ffffff", "rgba(11, 22, 44, 0.55)", t)} 0%, ${mixColors(
    "#f6f8fa",
    "rgba(3, 7, 20, 0.8)",
    t
  )} 100%)`;
}

function fieldShadow(t: number): string {
  return `inset 0 1px 3px ${mixColors("rgba(31, 35, 40, 0.08)", "rgba(0, 0, 0, 0.6)", t)}`;
}

function readableMix(light: string, dark: string, t: number): string {
  const W = 0.06;
  const lo = 0.5 - W / 2;
  const hi = 0.5 + W / 2;
  if (t <= lo) return light;
  if (t >= hi) return dark;
  return mixColors(light, dark, (t - lo) / W);
}

function midEase(t: number): number {
  const u = t * 2 - 1;
  const s = Math.sign(u) * Math.pow(Math.abs(u), 0.42);
  return (s + 1) / 2;
}

export function useTheme() {
  const themeLevel = ref<number>(1);
  const packThemeActive = ref(false);
  let packThemeVars = new Set<string>();

  function setPackThemeVars(keys: Set<string>) {
    packThemeVars = keys;
    packThemeActive.value = keys.size > 0;
    if (typeof document !== "undefined") {
      applyThemeLevel(themeLevel.value, false);
    }
  }

  function applyThemeLevel(level: number, persist = true) {
    const clamped = Math.min(1, Math.max(0, level));
    themeLevel.value = clamped;
    if (typeof document !== "undefined") {
      const root = document.documentElement;
      const surf = midEase(clamped);
      for (const [cssVar, lightVal] of Object.entries(THEME_LIGHT)) {
        if (packThemeVars.has(cssVar)) continue;
        const darkVal = THEME_DARK[cssVar]!;
        const t = TEXT_VARS.has(cssVar)
          ? readableMix(lightVal, darkVal, clamped)
          : mixColors(lightVal, darkVal, surf);
        root.style.setProperty(cssVar, t);
      }
      if (!packThemeVars.has("--panel-grad")) root.style.setProperty("--panel-grad", panelGrad(surf));
      if (!packThemeVars.has("--field-shadow")) root.style.setProperty("--field-shadow", fieldShadow(surf));
    }
    if (persist && typeof localStorage !== "undefined") {
      localStorage.setItem(THEME_KEY, String(clamped));
      emitThemeChanged(clamped);
    }
  }

  function killThemeFade() {
    if (typeof document !== "undefined") {
      document.documentElement.classList.remove("pack-theme-fade");
    }
  }

  let themeDragTimer: ReturnType<typeof setTimeout> | null = null;
  function suppressTransitions() {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    root.classList.add("no-theme-transition");
    if (themeDragTimer) clearTimeout(themeDragTimer);
    themeDragTimer = setTimeout(() => root.classList.remove("no-theme-transition"), 150);
  }

  function toggleTheme() {
    if (packThemeActive.value) return;
    killThemeFade();
    suppressTransitions();
    applyThemeLevel(themeLevel.value < 0.5 ? 1 : 0);
  }

  let themeRaf: number | null = null;
  let themePersistTimer: ReturnType<typeof setTimeout> | null = null;
  let themePendingLevel = 1;

  function applyThemeLevelThrottled(level: number) {
    themePendingLevel = level;
    if (themeRaf !== null) return;
    themeRaf = requestAnimationFrame(() => {
      themeRaf = null;
      applyThemeLevel(themePendingLevel, false);
    });
  }

  function setThemeLevel(level: number) {
    if (packThemeActive.value) return;
    killThemeFade();
    suppressTransitions();
    applyThemeLevelThrottled(level);
    if (themePersistTimer) clearTimeout(themePersistTimer);
    themePersistTimer = setTimeout(() => {
      themePersistTimer = null;
      applyThemeLevel(themePendingLevel, true);
    }, 400);
  }

  {
    let init = 1;
    if (typeof localStorage !== "undefined") {
      const raw = localStorage.getItem(THEME_KEY);
      if (raw === "light") init = 0;
      else if (raw === "dark") init = 1;
      else if (raw !== null) {
        const n = Number(raw);
        init = Number.isFinite(n) ? Math.min(1, Math.max(0, n)) : 1;
      }
    }
    applyThemeLevel(init, false);
  }

  return {
    themeLevel,
    applyThemeLevel,
    setThemeLevel,
    packThemeActive,
    setPackThemeVars,
    toggleTheme,
  };
}
