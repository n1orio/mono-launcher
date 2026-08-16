import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { check as checkAppUpdate, type Update as AppUpdate } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import {
  addPack,
  analyzeCrash,
  checkForUpdates,
  clearLaunchLog,
  clearLocalSkin,
  ensureJava,
  fetchCatalog,
  getGameFileIcons,
  getLaunchLog,
  getLocalSkin,
  getNews,
  getSkin,
  getStatus,
  getSystemInfo,
  installMrpack,
  isTauri,
  launchGame,
  launcherVersion,
  listGameFiles,
  listJava,
  listPacks,
  listSavedServers,
  listScreenshots,
  listVersions,
  loginOffline,
  msDeviceCode,
  msPoll,
  onCrashAnalyzed,
  onDownloadProgress,
  onGameExited,
  onLaunchLog,
  onModsChanged,
  onNewsChunk,
  onPackAdded,
  onPlaytimeUpdated,
  openExternal,
  openGameFolder,
  openPackDir,
  packRepoContent,
  removePack,
  setJavaPath,
  setDiscordRp,
  setWarnCustomMods,
  setLocale,
  setLocalSkin,
  skinApiUrl,
  setBoosty,
  licenseStatus,
  clearLicense,
  switchVersion,
  takePendingPackAdd,
  toggleGameFile,
  verifyGame,
  fetchPackIcon,
  listAccounts,
  switchAccount,
  removeAccount,
  elyDeviceCode,
  elyPoll,
  packLocked as packLockedCmd,
  setPackLocked as setPackLockedCmd,
} from "~/lib/bridge";
import type {
  AppStatus,
  Accounts,
  DownloadProgress,
  GameFileEntry,
  JavaInfo,
  LaunchLogEntry,
  LicenseInfo,
  MsDeviceCodeInfo,
  NewsItem,
  CatalogEntry,
  PackDescriptor,
  PackRepoContent,
  PackServer,
  SavedServer,
  SkinInfo,
  SystemInfo,
  UpdateInfo,
  UserSession,
  VerifyResult,
  VersionsInfo,
  CrashAnalysis,
} from "~/lib/types";
import type { GameFolderKind, PackAddedPayload } from "~/lib/bridge";
import { useI18n } from "~/composables/useI18n";
import { getCachedIcon, setCachedIcon } from "~/lib/iconCache";
import {
  formatBytes as _formatBytes,
  formatDate as _formatDate,
  formatUnixDate as _formatUnixDate,
  formatPlaytime as _formatPlaytime,
} from "~/lib/format";

const { t, locale } = useI18n();

/** Локализованные форматтеры (привязаны к активной локали). */
const formatBytes = (bytes: number) => _formatBytes(bytes, t);
const formatDate = (iso: string | null) => _formatDate(iso, locale.value);
const formatUnixDate = (epoch: number | null) => _formatUnixDate(epoch, locale.value);
const formatPlaytime = (seconds: number) => _formatPlaytime(seconds, t);

export interface ProgressState {
  phase: string;
  current: number;
  total: number;
  speed: number;
  fileIndex: number;
  fileTotal: number;
  currentFile: string;
}

export interface Notice {
  id: number;
  type: "error" | "info" | "success";
  text: string;
  reportable?: boolean;
}

const PACK_KEY = "mono.pack";
const RAM_KEY = "mono.ram";
const WIN_W_KEY = "mono.win.w";
const WIN_H_KEY = "mono.win.h";
const THEME_KEY = "mono.theme";
const CONSOLE_LIMIT = 2000;
/** Размер партии иконок файлов за один IPC-вызов (чтобы большие сборки не блокировали UI). */
const ICON_BATCH = 40;

function capLog(entries: LaunchLogEntry[]): LaunchLogEntry[] {
  return entries.slice(-CONSOLE_LIMIT);
}

export function useLauncher(options: { keepPackId?: boolean } = {}) {
  const status = ref<AppStatus | null>(null);
  const username = ref("");
  const savedRam =
    typeof localStorage !== "undefined" ? Number(localStorage.getItem(RAM_KEY)) : NaN;
  const ram = ref(Number.isFinite(savedRam) ? savedRam : 16);
  const maxRam = ref(16);
  const systemRam = ref<SystemInfo | null>(null);
  const windowWidth = ref(
    Number(localStorage.getItem(WIN_W_KEY)) || 854
  );
  const windowHeight = ref(
    Number(localStorage.getItem(WIN_H_KEY)) || 480
  );
  const session = ref<UserSession | null>(null);
  const busy = ref(false);
  const gameRunning = ref(false);
  const progress = ref<ProgressState | null>(null);
  /** Сколько файлов сборки уже обработано (монотонно — только растёт). */
  const filesDone = ref(0);
  const updateInfo = ref<UpdateInfo | null>(null);
  const versions = ref<VersionsInfo | null>(null);
  const logEntries = ref<LaunchLogEntry[]>([]);
  const logRef = ref<HTMLElement | null>(null);
  const tab = ref<"play" | "settings" | "news" | "catalog" | "dev">("play");
  /** Уровень темы: 0 = самая светлая, 1 = самая тёмная. */
  const themeLevel = ref<number>(1);

  /** Палитра светлой темы (куда стремимся при level = 0). */
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

  /** Палитра тёмной темы (куда стремимся при level = 1). */
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

  /** CSS-переменные, которые в данный момент задаёт тема сборки (их не перезаписываем). */
  let packThemeVars = new Set<string>();
  const packThemeActive = ref(false);

  function setPackThemeVars(keys: Set<string>) {
    packThemeVars = keys;
    packThemeActive.value = keys.size > 0;
    if (typeof document !== "undefined") {
      applyThemeLevel(themeLevel.value, false);
    }
  }

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

  /** Преобразует [r,g,b] (0..255) в [L,a,b] пространства OKLab. */
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

  /** Преобразует [L,a,b] OKLab в [r,g,b] (0..255). */
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
    return [
      linearToSrgb(r) * 255,
      linearToSrgb(g) * 255,
      linearToSrgb(b2) * 255,
    ];
  }

  /** Интерполирует цвет между светлым (t=0) и тёмным (t=1) в OKLab. */
  function mix(light: string, dark: string, t: number): string {
    const l = parseColor(light);
    const d = parseColor(dark);
    const lc = rgbToOklab([l[0], l[1], l[2]]);
    const dc = rgbToOklab([d[0], d[1], d[2]]);
    const lerp = (a: number, b: number) => a + (b - a) * t;
    const out = oklabToRgb([lerp(lc[0], dc[0]), lerp(lc[1], dc[1]), lerp(lc[2], dc[2])]);
    return rgbaStr([out[0], out[1], out[2], lerp(l[3], d[3])]);
  }

  /** Градиент панели из двух интерполированных концов. */
  function panelGrad(t: number): string {
    return `linear-gradient(180deg, ${mix("#ffffff", "rgba(11, 22, 44, 0.55)", t)} 0%, ${mix(
      "#f6f8fa",
      "rgba(3, 7, 20, 0.8)",
      t
    )} 100%)`;
  }

  /** Тень полей из интерполированного цвета. */
  function fieldShadow(t: number): string {
    return `inset 0 1px 3px ${mix("rgba(31, 35, 40, 0.08)", "rgba(0, 0, 0, 0.6)", t)}`;
  }

  /** Текстовые переменные: им нужно всегда высокое — не сливающееся с фоном.
      Поэтому текст держим тёмным на светлой половине и светлым на тёмной,
      плавно переключая полярность лишь в узкой зоне вокруг середины. */
  const TEXT_VARS = new Set(["--tx", "--tx-strong", "--tx-muted"]);

  /** Цвет текста с гарантированным контрастом: на светлой половине светлый
      фон → тёмный текст, на тёмной — наоборот. */
  function readableMix(light: string, dark: string, t: number): string {
    const W = 0.06;
    const lo = 0.5 - W / 2;
    const hi = 0.5 + W / 2;
    if (t <= lo) return light;
    if (t >= hi) return dark;
    return mix(light, dark, (t - lo) / W);
  }

  /** Плавный изгиб для фоновых переменных: задевая середину намного быстрее,
      чтобы не задерживаться на глухой серой зоне и почти сразу выходить
      к чистым светлому/тёмному краям. */
  function midEase(t: number): number {
    const u = t * 2 - 1;
    const s = Math.sign(u) * Math.pow(Math.abs(u), 0.42);
    return (s + 1) / 2;
  }

  /** Применяет уровень темы (0..1) к CSS-переменным и сохраняет выбор. */
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
          : mix(lightVal, darkVal, surf);
        root.style.setProperty(cssVar, t);
      }
      if (!packThemeVars.has("--panel-grad")) root.style.setProperty("--panel-grad", panelGrad(surf));
      if (!packThemeVars.has("--field-shadow")) root.style.setProperty("--field-shadow", fieldShadow(surf));
    }
    if (persist && typeof localStorage !== "undefined") {
      localStorage.setItem(THEME_KEY, String(clamped));
    }
  }

  /** Снимает кроссфейд темы (`pack-theme-fade`), чтобы смена ползунком была
      мгновенной и равномерной: иначе цвета анимируются 0.6s, а градиенты/тени
      (не участвуют в transition) переключаются сразу — элементы «догоняют» раздельно. */
  function killThemeFade() {
    if (typeof document !== "undefined") {
      document.documentElement.classList.remove("pack-theme-fade");
    }
  }

  let themeDragTimer: ReturnType<typeof setTimeout> | null = null;
  /** Пока ползунок тянут — глушим ВСЕ transition (`no-theme-transition` на html):
      иначе элементы с собственными `transition-colors/all` анимируют цвет за свою
      длительность и «запаздывают» относительно остальных. Через ~150мс после
      остановки транзишены возвращаются. */
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

  function setThemeLevel(level: number) {
    if (packThemeActive.value) return;
    killThemeFade();
    suppressTransitions();
    applyThemeLevel(level);
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
  const notifications = ref<Notice[]>([]);
  let noticeSeq = 0;
  const launcherVer = ref("");
  const msFlow = ref<MsDeviceCodeInfo | null>(null);
  const msPolling = ref(false);
  const elyFlow = ref<MsDeviceCodeInfo | null>(null);
  const elyPolling = ref(false);

  /** Текущий device code flow (Microsoft или Ely.by) для панели в настройках. */
  const deviceFlow = computed<MsDeviceCodeInfo | null>(() => msFlow.value ?? elyFlow.value);
  const accounts = ref<Accounts>({ active: null, list: [] });
  const accountBusy = ref(false);
  const ISSUES_URL = "https://github.com/n1orio/mono-launcher/issues/new";
  const appUpdate = ref<{ version: string; notes: string } | null>(null);
  const appUpdating = ref(false);
  const appUpdateProgress = ref<number | null>(null);
  let pendingAppUpdate: AppUpdate | null = null;
  const javaList = ref<JavaInfo[]>([]);
  const javaSelected = ref<string>("");
  const javaBusy = ref(false);
  const javaMsg = ref("");
  const verifyBusy = ref(false);
  const verifyResult = ref<VerifyResult | null>(null);
  const skinUrl = ref("");
  const localSkin = ref<SkinInfo | null>(null);
  const skinModel = ref<"classic" | "slim">("classic");
  const skinBusy = ref(false);
  const skinApi = ref("");
  /** Заблокирована ли правка файлов активной сборки (managed-сборка). */
  const packLocked = ref(false);

  async function loadPackLocked(id: string) {
    packLocked.value = !!id && (await packLockedCmd(id).catch(() => false));
  }

  /** Отвязывает/возвращает блокировку сборки; возвращает новое состояние. */
  async function setActivePackLocked(locked: boolean): Promise<boolean> {
    if (!packId.value) return packLocked.value;
    try {
      packLocked.value = await setPackLockedCmd(packId.value, locked);
    } catch (e) {
      notify(t("err.packLock", { e }));
    }
    return packLocked.value;
  }
  const licenseInfo = ref<LicenseInfo | null>(null);
  const licenseKeyInput = ref("");
  const licenseBusy = ref(false);
  const licenseError = ref("");
  const discordRp = ref(true);
  const warnCustomMods = ref(true);
  const news = ref<NewsItem[] | null>(null);
  const newsFilter = ref<string>("all");
  const packUrl = ref("");
  const packName = ref("");
  const addingPack = ref(false);
  const removingPack = ref("");
  const removeArmed = ref<string | null>(null);

  // ==== Вкладки файлов игры (моды/ресурспаки/шейдеры/миры/консоль) ====
  const playSubTab = ref<"releases" | "mods" | "resourcepacks" | "shaderpacks" | "saves" | "screenshots" | "servers" | "console" | "settings">("releases");
  const gameFiles = ref<Partial<Record<GameFolderKind, GameFileEntry[]>>>({});
  const fileIcons = ref<Record<string, string>>({});
  const fileSearch = ref("");
  const fileToggling = ref<Set<string>>(new Set());
  const selectedFiles = ref<Record<string, { folder: GameFolderKind; entry: GameFileEntry }>>({});
  // Контент репозитория сборки: звёзды, скриншоты, сервера.
  const repoContent = ref<Record<string, PackRepoContent>>({});
  const repoContentLoading = ref<Record<string, boolean>>({});

  function fileEntryKey(folder: GameFolderKind, entry: GameFileEntry): string {
    return `${folder}/${entry.name}`;
  }

  function toggleFileSelect(folder: GameFolderKind, entry: GameFileEntry) {
    const key = fileEntryKey(folder, entry);
    const next = { ...selectedFiles.value };
    if (next[key]) delete next[key];
    else next[key] = { folder, entry };
    selectedFiles.value = next;
  }

  function clearFileSelection() {
    selectedFiles.value = {};
  }

  /** Выделяет все файлы текущей папки (или переданный отфильтрованный список). */
  function selectAllFiles(folder: GameFolderKind, list?: GameFileEntry[]) {
    const src = list ?? gameFiles.value[folder] ?? [];
    const next = { ...selectedFiles.value };
    for (const entry of src) {
      next[fileEntryKey(folder, entry)] = { folder, entry };
    }
    selectedFiles.value = next;
  }

  /** Кол-во включённых файлов в папке (для счётчика в заголовке). */
  function enabledCountIn(folder: GameFolderKind): number {
    return (gameFiles.value[folder] ?? []).filter((f) => f.kind === "file" && f.enabled).length;
  }

  async function setSelectedFilesEnabled(enabled: boolean) {
    const targets = Object.values(selectedFiles.value).filter(
      (s) => s.entry.kind === "file" && s.entry.enabled !== enabled
    );
    for (const s of targets) {
      await handleToggleFile(s.folder, s.entry);
    }
  }

  /// Пытается выделить название мода/ресурспака из имени файла
  /// (отбрасывает версии, мусорные токены), чтобы поиск на Modrinth попадал точно.
  function cleanFileQuery(displayName: string): string {
    const STOP = new Set([
      "neoforge", "forge", "fabric", "quilt", "neo", "fapi", "architectury",
      "api", "fabric-api", "spigot", "bukkit", "paper", "backport", "mc",
      "minecraft", "edition", "editions", "mod", "mods", "v",
    ]);
    const tokens = displayName.split(/[-_+\s]+/).filter(Boolean);
    const clean = tokens.filter((t) => {
      const lower = t.toLowerCase();
      return !/\d/.test(t) && !STOP.has(lower);
    });
    if (clean.length === 0) {
      return tokens[0] ?? displayName;
    }
    return clean.slice(0, 4).join(" ");
  }

  const FOLDER_SEARCH_PATH: Record<GameFolderKind, string> = {
    mods: "mods",
    resourcepacks: "resourcepacks",
    shaderpacks: "shaders",
    saves: "mods",
  };

  function openFileOnModrinth(folder: GameFolderKind, entry: GameFileEntry) {
    // Точная страница мода (из downloads индекса сборки), иначе — поиск по имени.
    if (entry.modrinthUrl) {
      openExternal(entry.modrinthUrl);
      return;
    }
    const q = encodeURIComponent(cleanFileQuery(entry.displayName));
    openExternal(`https://modrinth.com/${FOLDER_SEARCH_PATH[folder]}?q=${q}`);
  }

  function openFileOnCurseForge(folder: GameFolderKind, entry: GameFileEntry) {
    const q = encodeURIComponent(cleanFileQuery(entry.displayName));
    openExternal(`https://www.curseforge.com/search/mods?q=${q}`);
  }

  async function loadGameFiles(folder: GameFolderKind, force = false) {
    if (!isTauri() || !packId.value) return;
    if (!force && gameFiles.value[folder]) return;
    try {
      const list = await listGameFiles(packId.value, folder);
      gameFiles.value = { ...gameFiles.value, [folder]: list };
      preloadIcons(folder, list);
    } catch (e) {
      notify(t("err.folderLoad", { e }));
    }
  }

  async function preloadIcons(folder: GameFolderKind, list: GameFileEntry[]) {
    if (!isTauri() || !packId.value || list.length === 0) return;
    const key = `${folder}/`;
    const prefix = `file:${packId.value}/${key}`;
    // 1) Мгновенно показываем закешированные иконки; устаревшие и отсутствующие
    //    догружаем в фоне (лимит — не грузить сотни архивов разом).
    const patch: Record<string, string> = {};
    const refetch: string[] = [];
    for (const f of list) {
      const k = key + f.name;
      if (fileIcons.value[k] !== undefined) continue;
      const cached = getCachedIcon(prefix + f.name);
      if (cached) {
        patch[k] = cached.data;
        if (cached.stale) refetch.push(f.name);
      } else {
        refetch.push(f.name);
      }
    }
    if (Object.keys(patch).length) {
      fileIcons.value = { ...fileIcons.value, ...patch };
    }
    if (refetch.length === 0) return;
    const missing = refetch.slice(0, 200);
    // Небольшими партиями, с уступкой между ними: большие сборки не блокируют UI,
    // иконки появляются постепенно, а не одним гигантским (медленным) вызовом.
    for (let i = 0; i < missing.length; i += ICON_BATCH) {
      const batch = missing.slice(i, i + ICON_BATCH);
      try {
        const icons = await getGameFileIcons(packId.value, folder, batch);
        const newPatch: Record<string, string> = {};
        for (const ic of icons) {
          if (ic.data) {
            newPatch[key + ic.name] = ic.data;
            setCachedIcon(prefix + ic.name, ic.data);
          }
        }
        if (Object.keys(newPatch).length) {
          fileIcons.value = { ...fileIcons.value, ...newPatch };
        }
      } catch (e) {
        // Иконки некритичны — покажем заглушку. Причина логируется для диагностики.
        console.error("icon batch failed", folder, e);
        break;
      }
      if (i + ICON_BATCH < missing.length) {
        // Даём браузеру отрисовать уже полученные иконки и не «зависаем».
        await new Promise((r) => setTimeout(r, 0));
      }
    }
  }

  async function handleToggleFile(folder: GameFolderKind, entry: GameFileEntry) {
    if (!isTauri() || !packId.value) return;
    const key = `${folder}/${entry.name}`;
    if (fileToggling.value.has(key)) return;
    const prev = entry.enabled;
    entry.enabled = !prev;
    fileToggling.value.add(key);
    try {
      await toggleGameFile(packId.value, folder, entry.name, entry.enabled);
    } catch (e) {
      entry.enabled = prev;
      notify(t("err.toggleFile", { e }));
    } finally {
      fileToggling.value.delete(key);
    }
  }

  // Уникальные источники новостей: «launcher» + id сборок (для фильтра).
  const newsSources = computed(() => {
    const ids = ["launcher", ...packs.value.map((p) => p.id)];
    return Array.from(new Set(ids));
  });

  const filteredNews = computed(() => {
    if (!news.value) return [];
    if (newsFilter.value === "all") return news.value;
    return news.value.filter((n) => n.pack_id === newsFilter.value);
  });

  async function loadNews() {
    if (!isTauri()) return;
    news.value = null;
    // Стримим: финальный результат придёт из getNews(), но UI обновляется
    // по `news-chunk` по мере подгрузки источников (свежие сверху).
    if (!unlistenNewsChunk) {
      onNewsChunk((chunk) => {
        news.value = chunk;
      }).then((fn) => (unlistenNewsChunk = fn));
    }
    try {
      news.value = await getNews();
    } catch (e) {
      if (news.value === null) notify(t("err.newsLoad", { e }));
      news.value ??= [];
    }
  }

  async function toggleDiscordRp(on: boolean) {
    discordRp.value = on;
    if (!isTauri()) return;
    try {
      await setDiscordRp(on);
    } catch (e) {
      notify(t("err.discordSave", { e }));
    }
  }

  async function toggleWarnCustomMods(on: boolean) {
    warnCustomMods.value = on;
    if (!isTauri()) return;
    try {
      await setWarnCustomMods(on);
    } catch (e) {
      notify(t("err.warnSave", { e }));
    }
  }

  /** Проверяет обновление лаунчера (шаблон обновления из GitHub Releases). */
  async function checkAppUpdates() {
    if (!isTauri()) return;
    try {
      const u = await checkAppUpdate();
      if (u) {
        pendingAppUpdate = u;
        appUpdate.value = { version: u.version, notes: u.body ?? "" };
      }
    } catch {
      // Нет сети/нет latest.json — молча пропускаем, не мешаем работе.
    }
  }

  /** Скачивает и ставит обновление лаунчера, затем перезапускается. */
  async function installAppUpdate() {
    if (!pendingAppUpdate || appUpdating.value) return;
    appUpdating.value = true;
    appUpdateProgress.value = 0;
    try {
      let contentLength = 0;
      let downloaded = 0;
      await pendingAppUpdate.downloadAndInstall((event) => {
        if (event.event === "Started") {
          contentLength = event.data.contentLength ?? 0;
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          if (contentLength > 0) {
            appUpdateProgress.value = Math.min(
              100,
              Math.round((downloaded / contentLength) * 100)
            );
          }
        }
      });
      appUpdate.value = null;
      await relaunch();
    } catch (e) {
      notify(t("err.appUpdate", { e }));
    } finally {
      appUpdating.value = false;
      appUpdateProgress.value = null;
    }
  }

  function notify(text: string, type: Notice["type"] = "error") {
    const id = ++noticeSeq;
    notifications.value.push({ id, type, text, reportable: type === "error" });
    if (notifications.value.length > 5) notifications.value.shift();
    if (type === "error") console.error(text);
    setTimeout(() => dismissNotification(id), type === "error" ? 20000 : 7000);
  }

  function dismissNotification(id: number) {
    notifications.value = notifications.value.filter((n) => n.id !== id);
  }

  function detectOS(): string {
    const ua = navigator.userAgent;
    if (/Windows/i.test(ua)) return "Windows";
    if (/Mac/i.test(ua)) return "macOS";
    if (/Linux/i.test(ua) || /X11/i.test(ua)) return "Linux";
    return ua.slice(0, 80);
  }

  /** Собирает тикет с ошибкой и логом запуска и открывает форму GitHub Issues. */
  async function reportError(errorText: string) {
    let log = "";
    try {
      log = await getLaunchLog();
    } catch {
      log = "";
    }
    const body = [
      t("report.desc"),
      "```",
      (errorText || t("report.empty")).slice(0, 600),
      "```",
      "",
      t("report.env"),
      t("report.launcher", { ver: launcherVer.value || "?" }),
      t("report.os", { os: detectOS() }),
      t("report.pack", { name: activePack.value?.name ?? (packId.value || "—") }),
      t("report.installed", {
        v: status.value?.installed ? t("report.installedYes") : t("report.installedNo"),
      }),
      status.value?.active_source_tag
        ? t("report.activeVer", { v: status.value.active_source_tag })
        : null,
      status.value?.minecraft_version
        ? `- Minecraft: ${status.value.minecraft_version}${status.value.loader ? ` / ${status.value.loader}` : ""}`
        : null,
      "",
      t("report.log"),
      "```",
      // Хвост лога — самое ценное для диагностики (исключение всегда в конце).
      // GitHub/браузеры обрезают длинные query-string, так что лимит жёсткий.
      (log || t("report.logEmpty")).slice(-3500),
      "```",
    ]
      .filter((l): l is string => l !== null)
      .join("\n");
    const title = t("report.title", {
      text: (errorText || t("report.launcherError")).slice(0, 80),
    });
    const url = `${ISSUES_URL}?title=${encodeURIComponent(title)}&body=${encodeURIComponent(body)}`;
    try {
      if (isTauri()) {
        await openExternal(url);
      } else {
        window.open(url, "_blank");
      }
    } catch {
      window.open(url, "_blank");
    }
  }

  /** Ссылка на GitHub-репозиторий сборки по её URL, "" если это не github-сборка. */
  function packRepoUrl(url: string): string {
    const rest = (url || "").replace(/^https?:\/\/github\.com\//, "");
    const [owner, repo] = rest.split("/");
    if (!owner || !repo || owner === "USER" || repo === "REPO") return "";
    return `https://github.com/${owner}/${repo}`;
  }

  /** Открывает форму GitHub Issues репозитория сборки с предзаполненным окружением. */
  const bugReportOpen = ref(false);
  const bugBody = ref("");
  const bugLog = ref("");
  const bugRepo = ref("");
  const bugCopied = ref(false);

  /** Хвост лога, попадающий в отчёт (чтобы URL Issues не раздувался). */
  const LOG_IN_ISSUE_LINES = 60;

  /** Собирает отчёт: окружение + хвост launch.log (в спойлере). */
  function buildBugReport(): string {
    const pack = activePack.value;
    const ver =
      status.value?.active_source_tag ?? status.value?.active_version ?? t("reportPack.unknown");
    const logLines = bugLog.value.split("\n").slice(-LOG_IN_ISSUE_LINES);
    // Ограничиваем размер лога в отчёте (GitHub/браузер режут длинные URL).
    const logBlock = logLines.join("\n").slice(-3500);
    const body = [
      t("reportPack.desc"),
      "",
      t("reportPack.steps"),
      "",
      t("reportPack.env"),
      t("reportPack.pack", { name: pack?.name ?? "?" }),
      t("reportPack.ver", { v: ver }),
      status.value?.minecraft_version
        ? `- Minecraft: ${status.value.minecraft_version}${status.value.loader ? ` / ${status.value.loader}` : ""}`
        : null,
      t("reportPack.launcher", { ver: launcherVer.value || "?" }),
      t("reportPack.os", { os: detectOS() }),
      `- RAM: ${ram.value} GB / окно: ${windowWidth.value}×${windowHeight.value}`,
      "",
      logLines.length
        ? [
            `**${t("reportPack.log")}**`,
            "",
            "<details>",
            "<summary>launch.log</summary>",
            "",
            "```",
            logBlock,
            "```",
            "</details>",
          ].join("\n")
        : null,
      "",
      `_${t("reportPack.note")}_`,
    ]
      .filter((l): l is string => l !== null)
      .join("\n");
    return body;
  }

  /** «Сообщить о баге»: собираем отчёт и показываем превью (скопировать / открыть Issues). */
  async function reportPackBug() {
    const pack = activePack.value;
    const repo = packRepoUrl(pack?.url ?? "");
    if (!pack || !repo) {
      notify(t("reportPack.noRepo"));
      return;
    }
    bugRepo.value = repo;
    bugLog.value = "";
    bugCopied.value = false;
    if (isTauri()) {
      try {
        bugLog.value = await getLaunchLog();
      } catch {
        /* без лога — отчёт соберётся без секции лога */
      }
    }
    bugBody.value = buildBugReport();
    bugReportOpen.value = true;
  }

  function closeBugReport() {
    bugReportOpen.value = false;
  }

  async function copyBugReport() {
    if (!bugBody.value) return;
    try {
      await navigator.clipboard.writeText(bugBody.value);
    } catch {
      const ta = document.createElement("textarea");
      ta.value = bugBody.value;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      ta.remove();
    }
    bugCopied.value = true;
    setTimeout(() => (bugCopied.value = false), 2000);
  }

  function openBugReportIssue() {
    const title = t("reportPack.title", { name: activePack.value?.name ?? "?" });
    const url = `${bugRepo.value}/issues/new?title=${encodeURIComponent(title)}&body=${encodeURIComponent(bugBody.value)}`;
    if (isTauri()) {
      openExternal(url).catch(() => window.open(url, "_blank"));
    } else {
      window.open(url, "_blank");
    }
  }

  /** Текущий анализ краш-репорта (null = не было сбоя / ещё не анализировали). */
  const crashAnalysis = ref<CrashAnalysis | null>(null);

  /** Запускает анализ свежих краш-артефактов текущей сборки. */
  async function runCrashAnalysis() {
    const packId = activePack.value?.id;
    if (!packId || !isTauri()) return;
    try {
      const a = await analyzeCrash(packId);
      crashAnalysis.value = a.hasCause ? a : null;
    } catch {
      crashAnalysis.value = null;
    }
  }

  function closeCrashAnalysis() {
    crashAnalysis.value = null;
  }

  const packs = ref<PackDescriptor[]>([]);
  const packId = ref("");

  let lastBytes = { value: 0, at: 0 };
  let speed = 0;
  let unlistenSync: (() => void) | undefined;
  let unlistenLogSync: (() => void) | undefined;
  let unlistenPlaytimeSync: (() => void) | undefined;
  let unlistenModsChangedSync: (() => void) | undefined;
  let unlistenGameExitedSync: (() => void) | undefined;
  let unlistenCrashSync: (() => void) | undefined;
  let unlistenDeepLinkSync: (() => void) | undefined;
  let unlistenNewsChunk: (() => void) | undefined;

  // Буфер логов: Java может выдавать тысячи строк в секунду —
  // рендерим консоль порциями, чтобы не ронять UI.
  const pendingLog: LaunchLogEntry[] = [];
  let logFlushTimer: ReturnType<typeof setTimeout> | undefined;

  /** Маркеры фатальных сбоев в живом логе игры. */
  const FATAL_LOG_RE =
    // eslint-disable-next-line no-misleading-character-class
    /(Failed to locate library:|Could not initialize class |ExceptionInInitializerError|NoClassDefFoundError|ThreadGroup:uncaughtException|A fatal error has been detected|Uncaught exception|Minecraft Crashed!|Fatal error)/;
  /** Флаг: уведомили о фатальном маркере в тек. сессии (без спама). */
  let crashMarkerNotified = false;

  function pushLog(entries: LaunchLogEntry[]) {
    for (const e of entries) {
      if (FATAL_LOG_RE.test(e.line)) e.fatal = true;
    }
    const hit = entries.some((e) => e.fatal);
    if (hit && !crashMarkerNotified) {
      crashMarkerNotified = true;
      notify(t("crash.logDetected"), "error");
    }
    pendingLog.push(...entries);
    if (pendingLog.length > CONSOLE_LIMIT) {
      pendingLog.splice(0, pendingLog.length - CONSOLE_LIMIT);
    }
    if (logFlushTimer != null) return;
    logFlushTimer = setTimeout(() => {
      logFlushTimer = undefined;
      logEntries.value = pendingLog.slice();
    }, 120);
  }

  const isInstalledVersion = (tag: string) =>
    versions.value?.installed.some(
      (iv) => iv.source_tag === tag || iv.version_id === tag
    ) ?? false;

  const activePack = computed(() => packs.value.find((p) => p.id === packId.value) ?? null);

  /** Обновляет список сборок, сохраняя выбранную (если она ещё существует). */
  async function loadPacks() {
    const list = await listPacks();
    packs.value = list;
    const saved =
      options.keepPackId || typeof localStorage === "undefined"
        ? null
        : localStorage.getItem(PACK_KEY);
    packId.value =
      (saved && list.some((p) => p.id === saved) ? saved : undefined) ??
      (list.some((p) => p.id === packId.value) ? packId.value : undefined) ??
      list[0]?.id ??
      "";
    await syncPackIcons();
  }

  // Иконки сборок: у авторских — icon.png из репозитория, у сборок с
  // Modrinth/CurseForge — иконка проекта с сайта. Скачиваем один раз за
  // сессию, дальше иконка лежит локально.
  const iconFetchTried = new Set<string>();
  async function syncPackIcons() {
    if (!isTauri()) return;
    let changed = false;
    for (const p of packs.value) {
      if (p.icon || iconFetchTried.has(p.id)) continue;
      iconFetchTried.add(p.id);
      try {
        if (await fetchPackIcon(p.id)) changed = true;
      } catch {
        // Нет сети или репозиторий без иконки — пропускаем.
      }
    }
    if (changed) await loadPacks();
  }

  /** Добавляет сборку по URL GitHub-репозитория или прямой ссылке на .mrpack. */
  async function handleAddPack() {
    if (!isTauri()) return;
    if (!packUrl.value.trim()) {
      notify(t("dev.errUrl"), "info");
      return;
    }
    addingPack.value = true;
    try {
      const added = await addPack(
        packUrl.value.trim(),
        packName.value.trim() ? packName.value.trim() : undefined
      );
      packUrl.value = "";
      packName.value = "";
      await loadPacks();
      await load();
      refreshVersions();
      if (packId.value !== added.id) {
        await selectPack(added.id);
      }
      notify(t("dev.added", { name: added.name }), "success");
    } catch (e) {
      notify(t("dev.errAdd", { e }), "error");
    } finally {
      addingPack.value = false;
    }
  }

  /** Каталог сборок (catalog.json репозитория лаунчера). */
  const catalog = ref<CatalogEntry[]>([]);
  const catalogLoading = ref(false);
  const catalogError = ref("");

  async function loadCatalog() {
    if (!isTauri()) return;
    catalogLoading.value = true;
    catalogError.value = "";
    try {
      catalog.value = await fetchCatalog();
    } catch (e) {
      catalogError.value = String(e);
    } finally {
      catalogLoading.value = false;
    }
  }

  /** «Добавить» из каталога: как по deep link, блог из записи каталога. */
  async function addFromCatalog(entry: CatalogEntry) {
    if (!isTauri() || addingPack.value || busy.value) return;
    addingPack.value = true;
    try {
      const added = await addPack(entry.url, entry.name, entry.boostyBlog ?? undefined);
      await loadPacks();
      await load();
      refreshVersions();
      if (packId.value !== added.id) {
        await selectPack(added.id);
      }
      notify(t("catalog.added", { name: added.name }), "success");
    } catch (e) {
      notify(t("dev.errAdd", { e }), "error");
    } finally {
      addingPack.value = false;
    }
  }

  /** Удаляет пользовательскую сборку (двухшаговое подтверждение кнопкой). */
  async function handleRemovePack(id: string) {
    if (!isTauri() || busy.value) return;
    if (removeArmed.value !== id) {
      removeArmed.value = id;
      return;
    }
    removeArmed.value = null;
    removingPack.value = id;
    try {
      await removePack(id);
      await loadPacks();
      await load();
      refreshVersions();
      notify(t("dev.removed"), "success");
    } catch (e) {
      notify(t("dev.errRemove", { e }), "error");
    } finally {
      removingPack.value = "";
    }
  }

  function resetRemoveArm() {
    removeArmed.value = null;
  }

  /** Обрабатывает результат добавления сборки по deep link (mono://add-pack...). */
  function handlePackAdded(p: PackAddedPayload) {
    if (!p.ok) {
      notify(t("dev.errAdd", { e: p.error ?? "?" }), "error");
      return;
    }
    loadPacks()
      .then(async () => {
        await load();
        refreshVersions();
        if (p.id && packId.value !== p.id) {
          await selectPack(p.id);
        }
        notify(
          p.already
            ? t("dev.already", { name: p.name || "?" })
            : t("dev.added", { name: p.name || "?" }),
          p.already ? "info" : "success"
        );
      })
      .catch(() => {});
  }

  async function load() {
    if (!isTauri() || !packId.value) return;
    const s = await getStatus(packId.value);
    status.value = s;
    session.value = s.session;
    discordRp.value = s.discord_rp_enabled;
    warnCustomMods.value = s.warn_custom_mods;
    if (s.session) username.value = s.session.username;
    refreshSkin();
    loadLicenseStatus();
    loadAccounts();
    const u = await checkForUpdates(packId.value).catch(() => null);
    if (u && u.has_update && u.latest_version) {
      updateInfo.value = {
        current_version: u.current_version,
        latest_version: u.latest_version,
        has_update: true,
      };
    } else {
      updateInfo.value = null;
    }
  }

  /** Аватар скина для Microsoft-профиля (offline-ники — без скина). */
  async function refreshSkin() {
    if (!isTauri() || !session.value) {
      skinUrl.value = "";
      return;
    }
    const s = session.value;
    if (s.user_type !== "microsoft") {
      skinUrl.value = "";
      return;
    }
    skinUrl.value = "";
    try {
      const url = await getSkin(s.uuid);
      if (url) skinUrl.value = url;
    } catch {
      skinUrl.value = "";
    }
  }

  /** Загружает локальный скин + URL скин-API (для инструкции). */
  async function loadLocalSkin() {
    if (!isTauri()) return;
    try {
      const info = await getLocalSkin();
      localSkin.value = info;
      if (info.has_skin) skinModel.value = info.model === "slim" ? "slim" : "classic";
    } catch {
      localSkin.value = null;
    }
    try {
      skinApi.value = await skinApiUrl();
    } catch {
      skinApi.value = "";
    }
  }

  /** Устанавливает скин из выбранного файла, затем грузит в скин-API. */
  async function applyLocalSkin(path: string) {
    if (!isTauri()) return;
    const nick = (session.value?.username ?? username.value.trim()) || "";
    if (!nick) {
      notify(t("err.nickname"), "info");
      return;
    }
    skinBusy.value = true;
    try {
      const info = await setLocalSkin(path, skinModel.value, nick);
      localSkin.value = info;
      if (info.has_skin) skinModel.value = info.model === "slim" ? "slim" : "classic";
      notify(t("skin.done"), "success");
    } catch (e) {
      notify(String(e), "error");
    } finally {
      skinBusy.value = false;
    }
  }

  /** Удаляет скин (локально + из API). */
  async function removeLocalSkin() {
    if (!isTauri()) return;
    const nick = (session.value?.username ?? username.value.trim()) || "";
    skinBusy.value = true;
    try {
      await clearLocalSkin(nick);
      localSkin.value = null;
      notify(t("skin.removed"), "success");
    } catch (e) {
      notify(String(e), "error");
    } finally {
      skinBusy.value = false;
    }
  }

  /** Статус лицензии активной сборки (null — сборка бесплатная/не привязан Boosty). */
  async function loadLicenseStatus() {
    if (!isTauri() || !packId.value) {
      licenseInfo.value = null;
      return;
    }
    if (!activePack.value?.boostyBlog) {
      licenseInfo.value = null;
      licenseError.value = "";
      return;
    }
    try {
      licenseInfo.value = await licenseStatus(packId.value);
      licenseError.value = "";
    } catch (e) {
      licenseInfo.value = null;
      licenseError.value = String(e);
    }
  }

  /** Принимает токен Boosty от игрока: сохраняет и проверяет подписку. */
  async function saveLicense() {
    if (!isTauri() || !packId.value) return;
    const token = licenseKeyInput.value.trim();
    if (!token) {
      notify(t("license.errEmpty"), "info");
      return;
    }
    licenseBusy.value = true;
    try {
      licenseInfo.value = await setBoosty(packId.value, token);
      licenseKeyInput.value = "";
      licenseError.value = "";
      notify(t("license.ok"), "success");
    } catch (e) {
      licenseError.value = String(e);
      notify(t("license.errSave", { e }), "error");
    } finally {
      licenseBusy.value = false;
    }
  }

  /** Удаляет сохранённый токен Boosty сборки. */
  async function removeLicense() {
    if (!isTauri() || !packId.value) return;
    licenseBusy.value = true;
    try {
      await clearLicense(packId.value);
      licenseInfo.value = null;
      licenseKeyInput.value = "";
      licenseError.value = "";
      notify(t("license.removed"), "success");
    } catch (e) {
      notify(String(e), "error");
    } finally {
      licenseBusy.value = false;
    }
  }

  async function loadJava() {
    if (!isTauri()) return;
    try {
      javaList.value = await listJava();
      const sel = javaList.value.find((j) => j.selected);
      javaSelected.value = sel ? sel.path : "";
    } catch {
      javaList.value = [];
    }
  }

  async function selectJava(path: string) {
    if (!isTauri()) return;
    javaSelected.value = path;
    javaMsg.value = "";
    try {
      await setJavaPath(path || null);
      await loadJava();
    } catch (e) {
      notify(t("err.javaSave", { e }));
    }
  }

  async function downloadJava() {
    if (!isTauri() || javaBusy.value) return;
    javaBusy.value = true;
    javaMsg.value = t("java.downloading");
    try {
      const path = await ensureJava();
      javaMsg.value = t("java.installed", { path });
      await loadJava();
      if (packId.value) {
        await handleInstall();
      }
    } catch (e) {
      javaMsg.value = "";
      notify(t("err.javaDownload", { e }));
    } finally {
      javaBusy.value = false;
    }
  }

  async function handleVerify() {
    if (!isTauri() || !packId.value || verifyBusy.value) return;
    verifyBusy.value = true;
    verifyResult.value = null;
    try {
      verifyResult.value = await verifyGame(packId.value);
    } catch (e) {
      notify(t("err.verify", { e }));
    } finally {
      verifyBusy.value = false;
    }
  }

  async function openFolder(folder: GameFolderKind | "screenshots" | "logs") {
    if (!packId.value) return;
    if (!isTauri()) {
      notify(t("err.desktopOnly"), "info");
      return;
    }
    try {
      await openGameFolder(packId.value, folder);
    } catch (e) {
      notify(t("err.openFolder", { e }));
    }
  }

  async function refreshVersions() {
    if (!packId.value) return;
    listVersions(packId.value)
      .then((v) => (versions.value = v))
      .catch(() => {});
  }

  /** Загружает контент репозитория сборки (звёзды/скриншоты/сервера), один раз. */
  async function loadPackRepoContent(id: string) {
    if (!isTauri() || !id || repoContentLoading.value[id]) return;
    if (repoContent.value[id]) return;
    repoContentLoading.value = { ...repoContentLoading.value, [id]: true };
    try {
      const c = await packRepoContent(id);
      repoContent.value = { ...repoContent.value, [id]: c };
    } catch {
      // Не критично: без звёзд/скриншотов лаунчер работает.
    } finally {
      repoContentLoading.value = { ...repoContentLoading.value, [id]: false };
    }
  }

  /** Скриншоты установленной версии (папка screenshots) и сервера игрока (servers.dat). */
  const packScreenshots = ref<string[]>([]);
  const packScreenshotsInstalled = ref(false);
  const screenshotsLoading = ref(false);
  const myServers = ref<SavedServer[]>([]);
  const myServersInstalled = ref(false);

  /** Загружает скриншоты из папки screenshots активной версии сборки. */
  async function loadPackScreenshots(id: string) {
    if (!isTauri() || !id) return;
    screenshotsLoading.value = true;
    try {
      const list = await listScreenshots(id);
      packScreenshots.value = list.screenshots;
      packScreenshotsInstalled.value = list.installed;
    } catch {
      packScreenshots.value = [];
      packScreenshotsInstalled.value = false;
    } finally {
      screenshotsLoading.value = false;
    }
  }

  /** Загружает сервера игрока из servers.dat активной версии. */
  async function loadMyServers(id: string) {
    if (!isTauri() || !id) return;
    try {
      const list = await listSavedServers(id);
      myServers.value = list.servers;
      myServersInstalled.value = list.installed;
    } catch {
      myServers.value = [];
      myServersInstalled.value = false;
    }
  }

  async function selectPack(id: string) {
    if (id === packId.value) return;
    packId.value = id;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(PACK_KEY, id);
    }
    await load();
    refreshVersions();
  }

  onMounted(async () => {
    if (!isTauri()) {
      packId.value = "untold-legends";
      return;
    }
    try {
      await loadPacks();
    } catch {
      packId.value = "untold-legends";
    }
    await load();
    refreshVersions();
    loadLocalSkin();
    getSystemInfo()
      .then((sys) => {
        const total = Math.max(2, sys.total_ram_gb);
        const max = Math.min(Math.max(total, 2), 64);
        systemRam.value = sys;
        maxRam.value = max;
        ram.value = Math.min(Math.max(ram.value, 2), max);
      })
      .catch(() => {});
    getLaunchLog()
      .then((text) => {
        if (text.trim()) {
          logEntries.value = capLog(
            text.split("\n").map((line) => ({ stream: "out" as const, line }))
          );
        }
      })
      .catch(() => {});
    onDownloadProgress((p: DownloadProgress) => {
      const now = Date.now();
      const deltaBytes = p.current - lastBytes.value;
      const deltaMs = now - lastBytes.at;
      if (deltaMs > 0 && deltaBytes > 0) {
        const inst = (deltaBytes / deltaMs) * 1000;
        speed = speed > 0 ? speed * 0.7 + inst * 0.3 : inst;
      }
      lastBytes = { value: p.current, at: now };
      if (p.file_total > 1 && p.file_index >= 0) {
        filesDone.value = Math.max(filesDone.value, p.file_index);
      }
      progress.value = {
        phase: p.phase,
        current: p.current,
        total: p.total,
        speed,
        fileIndex: p.file_index,
        fileTotal: p.file_total,
        currentFile: p.current_file,
      };
    }).then((fn) => (unlistenSync = fn));
    onLaunchLog((entry: LaunchLogEntry) => {
      pushLog([entry]);
    }).then((fn) => (unlistenLogSync = fn));
    onPlaytimeUpdated((p) => {
      if (!versions.value) return;
      const idx = versions.value.installed.findIndex(
        (v) => v.version_id === p.version_id
      );
      if (idx >= 0) {
        versions.value.installed[idx].total_seconds = p.total_seconds;
        versions.value.installed = versions.value.installed.slice();
      }
    }).then((fn) => (unlistenPlaytimeSync = fn));
    onModsChanged(() => {
      if (
        playSubTab.value === "mods" ||
        playSubTab.value === "resourcepacks" ||
        playSubTab.value === "shaderpacks" ||
        playSubTab.value === "saves"
      ) {
        loadGameFiles(playSubTab.value, true);
      }
    }).then((fn) => (unlistenModsChangedSync = fn));
    onGameExited((e) => {
      gameRunning.value = false;
      if (!e.success) {
        const code =
          e.code > 0
            ? t("game.code", { code: e.code })
            : e.code === 0
              ? ""
              : t("game.crash");
        notify(t("game.exitError", { code }), "error");
      }
    }).then((fn) => (unlistenGameExitedSync = fn));
    onCrashAnalyzed((a) => {
      crashAnalysis.value = a.hasCause ? a : null;
    }).then((fn) => (unlistenCrashSync = fn));
    onPackAdded((p) => {
      handlePackAdded(p);
    }).then((fn) => (unlistenDeepLinkSync = fn));
    takePendingPackAdd()
      .then((p) => {
        if (p) handlePackAdded(p);
      })
      .catch(() => {});
    launcherVersion()
      .then((v) => (launcherVer.value = v))
      .catch(() => {});
    loadJava();
    setTimeout(() => checkAppUpdates(), 4000);
  });

  onMounted(() => {
    if (isTauri()) setLocale(locale.value);
  });

  watch(
    locale,
    (l) => {
      if (isTauri()) setLocale(l);
    },
    { flush: "post" }
  );

  watch(
    tab,
    (t) => {
      if (t === "news") loadNews();
      if (t === "catalog") loadCatalog();
    },
    { flush: "post" }
  );

  watch(
    playSubTab,
    (t) => {
      fileSearch.value = "";
      selectedFiles.value = {};
      if (t === "mods" || t === "resourcepacks" || t === "shaderpacks" || t === "saves") {
        loadGameFiles(t);
      }
    },
    { flush: "post" }
  );

  watch(
    packId,
    (id) => {
      gameFiles.value = {};
      fileIcons.value = {};
      selectedFiles.value = {};
      fileSearch.value = "";
      loadPackLocked(id);
      // Кэш сброшен — сразу перечитаем файлы активной папки новой сборки,
      // иначе экран «Моды/Миры/...» останется пустым до переключения сабтаба.
      if (
        playSubTab.value === "mods" ||
        playSubTab.value === "resourcepacks" ||
        playSubTab.value === "shaderpacks" ||
        playSubTab.value === "saves"
      ) {
        loadGameFiles(playSubTab.value);
      }
    },
    { flush: "post" }
  );

  onUnmounted(() => {
    unlistenSync?.();
    unlistenLogSync?.();
    unlistenPlaytimeSync?.();
    unlistenModsChangedSync?.();
    unlistenGameExitedSync?.();
    unlistenCrashSync?.();
    unlistenDeepLinkSync?.();
    unlistenNewsChunk?.();
  });

  watch(
    ram,
    (v) => {
      if (typeof localStorage !== "undefined") {
        localStorage.setItem(RAM_KEY, String(v));
      }
    },
    { flush: "post" }
  );

  watch(
    [windowWidth, windowHeight],
    ([w, h]) => {
      if (typeof localStorage === "undefined") return;
      localStorage.setItem(WIN_W_KEY, String(w));
      localStorage.setItem(WIN_H_KEY, String(h));
    },
    { flush: "post" }
  );

  watch(
    logEntries,
    () => {
      if (logRef.value) {
        logRef.value.scrollTop = logRef.value.scrollHeight;
      }
    },
    { flush: "post" }
  );

  async function handleInstall(tag?: string) {
    if (!isTauri() || !packId.value) return;
    busy.value = true;
    filesDone.value = 0;
    progress.value = { phase: "Подготовка...", current: 0, total: 0, speed: 0, fileIndex: 0, fileTotal: 0, currentFile: "" };
    try {
      await installMrpack(packId.value, tag);
      await load();
      refreshVersions();
    } catch (e) {
      notify(t("err.install", { e }));
    } finally {
      busy.value = false;
      lastBytes = { value: 0, at: 0 };
      speed = 0;
    }
  }

  async function handleUpdate() {
    const tag = updateInfo.value?.latest_version;
    if (!tag) return;
    await handleInstall(tag);
  }

  async function handleSelectVersion(tag: string) {
    if (!isTauri() || !tag || !packId.value) return;
    const found = versions.value?.installed.find(
      (v) => v.source_tag === tag || v.version_id === tag
    );
    if (found) {
      try {
        await switchVersion(packId.value, tag);
        await load();
        refreshVersions();
      } catch (e) {
notify(t("err.switch", { e }));
      }
    } else {
      await handleInstall(tag);
    }
  }

  async function handleOffline() {
    if (!isTauri()) return;
    if (!username.value.trim()) {
      notify(t("err.nickname"), "info");
      return;
    }
    try {
      const s = await loginOffline(username.value.trim());
      session.value = s;
      await load();
      await loadAccounts();
    } catch (e) {
      notify(t("err.login", { e }));
    }
  }

  async function handleMicrosoft() {
    if (!isTauri()) return;
    try {
      const info = await msDeviceCode();
      msFlow.value = info;
      msPolling.value = true;
      const s = await msPoll(info.device_code, info.interval, info.expires_in);
      session.value = s;
      msFlow.value = null;
      await load();
      await loadAccounts();
    } catch (e) {
      notify(t("err.microsoft", { e }));
    } finally {
      msPolling.value = false;
      msFlow.value = null;
    }
  }

  /** Вход через Ely.by (device code flow, как у Microsoft). */
  async function handleEly() {
    if (!isTauri()) return;
    try {
      const info = await elyDeviceCode();
      elyFlow.value = info;
      elyPolling.value = true;
      const s = await elyPoll(info.device_code, info.interval, info.expires_in);
      session.value = s;
      elyFlow.value = null;
      await load();
      await loadAccounts();
    } catch (e) {
      notify(t("err.ely", { e }));
    } finally {
      elyPolling.value = false;
      elyFlow.value = null;
    }
  }

  async function loadAccounts() {
    if (!isTauri()) return;
    try {
      accounts.value = await listAccounts();
    } catch {
      /* список аккаунтов — не критично */
    }
  }

  async function handleSwitchAccount(id: string) {
    if (!isTauri() || accountBusy.value || id === accounts.value.active) return;
    accountBusy.value = true;
    try {
      session.value = await switchAccount(id);
      await load();
      await loadAccounts();
    } catch (e) {
      notify(t("err.accountSwitch", { e }));
    } finally {
      accountBusy.value = false;
    }
  }

  async function handleRemoveAccount(id: string) {
    if (!isTauri() || accountBusy.value) return;
    if (!confirm(t("accounts.confirmRemove"))) return;
    accountBusy.value = true;
    try {
      const next = await removeAccount(id);
      session.value = next;
      await load();
      await loadAccounts();
    } catch (e) {
      notify(t("err.accountRemove", { e }));
    } finally {
      accountBusy.value = false;
    }
  }

  async function openMsAuthPage() {
    if (!isTauri() || !deviceFlow.value) return;
    try {
      await openExternal(deviceFlow.value.verification_uri);
    } catch {
      notify(t("err.openPage", { url: deviceFlow.value.verification_uri }), "info");
    }
  }

  async function handlePlay() {
    await runGame(null);
  }

  /** Запуск игры, опционально с авто-коннектом на сервер ("host" или "host:port"). */
  async function runGame(server: string | null) {
    if (!isTauri() || !packId.value) return;
    if (busy.value) return;
    if (!session.value) {
      notify(t("err.loginFirst"), "info");
      return;
    }
    if (gameRunning.value) {
      notify(t("err.gameRunning"), "info");
      return;
    }
    const minMb = activePack.value?.minRam;
    if (minMb && ram.value * 1024 < minMb) {
      notify(t("err.lowRam", { min: minMb / 1024, gb: ram.value }), "error");
      return;
    }
    busy.value = true;
    logEntries.value = [];
    pendingLog.length = 0;
    try {
      await launchGame(
        packId.value,
        ram.value,
        session.value,
        windowWidth.value,
        windowHeight.value,
        server
      );
      crashMarkerNotified = false;
      gameRunning.value = true;
    } catch (e) {
      notify(t("err.launch", { e }));
    } finally {
      busy.value = false;
    }
  }

  /** «Играть на сервере»: запуск с --server/--port из карточки сервера. */
  function playOnServer(srv: PackServer) {
    return runGame(srv.port ? `${srv.ip}:${srv.port}` : srv.ip);
  }

  async function handleClearLog() {
    logEntries.value = [];
    pendingLog.length = 0;
    if (isTauri()) clearLaunchLog().catch(() => {});
  }

  async function handleOpenPackDir() {
    if (!packId.value) return;
    if (!isTauri()) {
      notify(t("err.desktopOnly"), "info");
      return;
    }
    try {
      await openPackDir(packId.value);
    } catch (e) {
      notify(t("err.openPackDir", { e }));
    }
  }

  async function handleCopyLog() {
    const text = logEntries.value.map((e) => e.line).join("\n");
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      const ta = document.createElement("textarea");
      ta.value = text;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      ta.remove();
    }
  }

  const percent = computed(() => {
    if (!progress.value) return 0;
    // Фаза «Установка модов»: прогресс по числу обработанных файлов (монотонно).
    if (progress.value.fileTotal > 1) {
      const fs = progress.value.fileTotal;
      return Math.min(100, Math.round((filesDone.value / fs) * 100));
    }
    if (progress.value.total > 0) {
      return Math.min(100, Math.round((progress.value.current / progress.value.total) * 100));
    }
    return 0;
  });

  /** Прогресс текущего файла в байтах (для тонкой полоски). */
  const filePercent = computed(() => {
    const pr = progress.value;
    if (!pr || pr.total <= 0 || pr.current <= 0) return 0;
    return Math.min(100, Math.round((pr.current / pr.total) * 100));
  });

  const loaderLabel = computed(() => {
    if (status.value?.loader) {
      return `${status.value.loader}${status.value.minecraft_version ? ` · ${status.value.minecraft_version}` : ""}`;
    }
    return "";
  });

  return {
    status,
    username,
    ram,
    maxRam,
    systemRam,
    windowWidth,
    windowHeight,
    session,
    busy,
    gameRunning,
    progress,
    updateInfo,
    launcherVer,
    versions,
    logEntries,
    logRef,
    tab,
    themeLevel,
    setThemeLevel,
    packLocked,
    setActivePackLocked,
    setPackThemeVars,
    packThemeActive,
    toggleTheme,
    packs,
    packId,
    activePack,
    percent,
    filePercent,
    filesDone,
    loaderLabel,
    formatBytes,
    formatDate,
    formatUnixDate,
    formatPlaytime,
    isInstalledVersion,
    handleInstall,
    handleUpdate,
    handleSelectVersion,
    handleOffline,
    handleMicrosoft,
    handleEly,
    openMsAuthPage,
    msFlow,
    msPolling,
    elyFlow,
    elyPolling,
    deviceFlow,
    accounts,
    accountBusy,
    loadAccounts,
    handleSwitchAccount,
    handleRemoveAccount,
    handlePlay,
    playOnServer,
    handleClearLog,
    handleCopyLog,
    handleOpenPackDir,
    selectPack,
    skinUrl,
    localSkin,
    skinModel,
    skinBusy,
    skinApi,
    loadLocalSkin,
    applyLocalSkin,
    removeLocalSkin,
    licenseInfo,
    licenseKeyInput,
    licenseBusy,
    licenseError,
    loadLicenseStatus,
    saveLicense,
    removeLicense,
    notifications,
    notify,
    dismissNotification,
    reportError,
    reportPackBug,
    bugReportOpen,
    bugBody,
    bugLog,
    crashAnalysis,
    runCrashAnalysis,
    closeCrashAnalysis,
    bugCopied,
    closeBugReport,
    copyBugReport,
    openBugReportIssue,
    catalog,
    catalogLoading,
    catalogError,
    loadCatalog,
    addFromCatalog,
    appUpdate,
    appUpdating,
    appUpdateProgress,
    installAppUpdate,
    javaList,
    javaSelected,
    javaBusy,
    javaMsg,
    loadJava,
    selectJava,
    downloadJava,
    verifyBusy,
    verifyResult,
    handleVerify,
    openFolder,
    refreshSkin,
    discordRp,
    toggleDiscordRp,
    warnCustomMods,
    toggleWarnCustomMods,
    news,
    loadNews,
    newsFilter,
    newsSources,
    filteredNews,
    playSubTab,
    repoContent,
    repoContentLoading,
    loadPackRepoContent,
  packScreenshots,
  packScreenshotsInstalled,
  screenshotsLoading,
  loadPackScreenshots,
  myServers,
  myServersInstalled,
  loadMyServers,
    gameFiles,
    fileIcons,
    loadGameFiles,
    handleToggleFile,
    fileSearch,
    fileToggling,
    selectedFiles,
    toggleFileSelect,
    clearFileSelection,
    selectAllFiles,
    enabledCountIn,
    setSelectedFilesEnabled,
    openFileOnModrinth,
    openFileOnCurseForge,
    packUrl,
    packName,
    addingPack,
    removingPack,
    removeArmed,
    handleAddPack,
    handleRemovePack,
    resetRemoveArm,
    loadPacks,
    load,
  };
}