import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { check as checkAppUpdate, type Update as AppUpdate } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import {
  addPack,
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
  onDownloadProgress,
  onGameExited,
  onLaunchLog,
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
  listAccounts,
  switchAccount,
  removeAccount,
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
} from "~/lib/types";
import type { GameFolderKind, PackAddedPayload } from "~/lib/bridge";
import { useI18n } from "~/composables/useI18n";

const { t, locale } = useI18n();

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

const PACK_KEY = "nio.pack";
const RAM_KEY = "nio.ram";
const WIN_W_KEY = "nio.win.w";
const WIN_H_KEY = "nio.win.h";
const THEME_KEY = "nio.theme";
const CONSOLE_LIMIT = 2000;

function formatBytes(bytes: number): string {
  if (bytes <= 0) return `0 ${t("units.b")}`;
  const units = [t("units.b"), t("units.kb"), t("units.mb"), t("units.gb")];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  return `${(bytes / Math.pow(1024, i)).toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

function formatPlaytime(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (h > 0) return t("time.hm", { h, m });
  return t("time.min", { m });
}

function formatDate(iso: string | null): string {
  if (!iso) return "";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "";
  return d.toLocaleDateString(locale.value === "en" ? "en-US" : "ru-RU", {
    day: "2-digit",
    month: "short",
    year: "numeric",
  });
}

/** Дата из unix-секунд (для лицензий) в локальном формате. */
function formatUnixDate(epoch: number | null): string {
  if (!epoch) return "";
  return formatDate(new Date(epoch * 1000).toISOString());
}

function capLog(entries: LaunchLogEntry[]): LaunchLogEntry[] {
  return entries.slice(-CONSOLE_LIMIT);
}

export function useLauncher() {
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
  const progress = ref<ProgressState | null>(null);
  const updateInfo = ref<UpdateInfo | null>(null);
  const versions = ref<VersionsInfo | null>(null);
  const logEntries = ref<LaunchLogEntry[]>([]);
  const logRef = ref<HTMLElement | null>(null);
  const tab = ref<"play" | "settings" | "news" | "catalog" | "dev">("play");
  const theme = ref<"dark" | "light">("dark");

  /** Применяет тему лаунчера (тёмная/светлая) и сохраняет выбор. */
  function applyTheme(th: "dark" | "light") {
    theme.value = th;
    if (typeof document !== "undefined") {
      document.documentElement.classList.toggle("theme-light", th === "light");
    }
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(THEME_KEY, th);
    }
  }

  function toggleTheme() {
    applyTheme(theme.value === "dark" ? "light" : "dark");
  }

  applyTheme(
    typeof localStorage !== "undefined" && localStorage.getItem(THEME_KEY) === "light"
      ? "light"
      : "dark"
  );
  const notifications = ref<Notice[]>([]);
  let noticeSeq = 0;
  const launcherVer = ref("");
  const msFlow = ref<MsDeviceCodeInfo | null>(null);
  const msPolling = ref(false);
  const accounts = ref<Accounts>({ active: null, list: [] });
  const accountBusy = ref(false);
  const ISSUES_URL = "https://github.com/n1orio/nio-launcher/issues/new";
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
  const playSubTab = ref<"releases" | "mods" | "resourcepacks" | "shaderpacks" | "saves" | "screenshots" | "servers" | "console">("releases");
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

  /** Выделить все ПКМ-выбранные... нет: выделяет все файлы текущей папки. */
  function selectAllFiles(folder: GameFolderKind) {
    const list = gameFiles.value[folder] ?? [];
    const next = { ...selectedFiles.value };
    for (const entry of list) {
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

  function openFileOnModrinth(folder: GameFolderKind, entry: GameFileEntry) {
    // Точная страница мода (из downloads индекса сборки), иначе — поиск по имени.
    if (entry.modrinthUrl) {
      openExternal(entry.modrinthUrl);
      return;
    }
    const q = encodeURIComponent(cleanFileQuery(entry.displayName));
    openExternal(`https://modrinth.com/mods?q=${q}`);
  }

  function openFileOnCurseForge(folder: GameFolderKind, entry: GameFileEntry) {
    const q = encodeURIComponent(cleanFileQuery(entry.displayName));
    openExternal(`https://www.curseforge.com/search/mods?q=${q}`);
  }

  async function loadGameFiles(folder: GameFolderKind) {
    if (!isTauri() || !packId.value) return;
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
    // Уже загруженные пропускаем; лимит — не грузить сотни архивов разом.
    const missing = list
      .filter((f) => fileIcons.value[key + f.name] === undefined)
      .map((f) => f.name)
      .slice(0, 200);
    if (missing.length === 0) return;
    try {
      const icons = await getGameFileIcons(packId.value, folder, missing);
      const patch: Record<string, string> = {};
      for (const ic of icons) {
        patch[key + ic.name] = ic.data ?? "";
      }
      fileIcons.value = { ...fileIcons.value, ...patch };
    } catch (e) {
      // Иконки некритичны — покажем заглушку. Причина логируется для диагностики.
      console.error("icon batch failed", folder, e);
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
    try {
      news.value = await getNews();
    } catch (e) {
      notify(t("err.newsLoad", { e }));
      news.value = [];
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
      (errorText || t("report.empty")).slice(0, 3000),
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
      (log || t("report.logEmpty")).slice(-50000),
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
            ...logLines,
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

  const packs = ref<PackDescriptor[]>([]);
  const packId = ref("");

  let lastBytes = { value: 0, at: 0 };
  let speed = 0;
  let unlistenSync: (() => void) | undefined;
  let unlistenLogSync: (() => void) | undefined;
  let unlistenPlaytimeSync: (() => void) | undefined;
  let unlistenGameExitedSync: (() => void) | undefined;
  let unlistenDeepLinkSync: (() => void) | undefined;

  // Буфер логов: Java может выдавать тысячи строк в секунду —
  // рендерим консоль порциями, чтобы не ронять UI.
  const pendingLog: LaunchLogEntry[] = [];
  let logFlushTimer: ReturnType<typeof setTimeout> | undefined;

  function pushLog(entries: LaunchLogEntry[]) {
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
      typeof localStorage !== "undefined" ? localStorage.getItem(PACK_KEY) : null;
    packId.value =
      (saved && list.some((p) => p.id === saved) ? saved : undefined) ??
      (list.some((p) => p.id === packId.value) ? packId.value : undefined) ??
      list[0]?.id ??
      "";
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

  /** Обрабатывает результат добавления сборки по deep link (nio://add-pack...). */
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
    if (!isTauri() || !packId.value) return;
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
        speed = (deltaBytes / deltaMs) * 1000;
      }
      lastBytes = { value: p.current, at: now };
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
    onGameExited((e) => {
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
    () => {
      gameFiles.value = {};
      fileIcons.value = {};
      selectedFiles.value = {};
      fileSearch.value = "";
    },
    { flush: "post" }
  );

  onUnmounted(() => {
    unlistenSync?.();
    unlistenLogSync?.();
    unlistenPlaytimeSync?.();
    unlistenGameExitedSync?.();
    unlistenDeepLinkSync?.();
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
    if (!isTauri() || !msFlow.value) return;
    try {
      await openExternal(msFlow.value.verification_uri);
    } catch {
      notify(t("err.openPage", { url: msFlow.value.verification_uri }), "info");
    }
  }

  async function handlePlay() {
    await runGame(null);
  }

  /** Запуск игры, опционально с авто-коннектом на сервер ("host" или "host:port"). */
  async function runGame(server: string | null) {
    if (!isTauri() || !packId.value) return;
    if (!session.value) {
      notify(t("err.loginFirst"), "info");
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
    if (!isTauri() || !packId.value) return;
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
    if (progress.value.total > 0) {
      return Math.min(100, Math.round((progress.value.current / progress.value.total) * 100));
    }
    return 100;
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
    progress,
    updateInfo,
    versions,
    logEntries,
    logRef,
    tab,
    theme,
    toggleTheme,
    packs,
    packId,
    activePack,
    percent,
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
    openMsAuthPage,
    msFlow,
    msPolling,
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
  };
}