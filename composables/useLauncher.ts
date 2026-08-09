import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import {
  checkForUpdates,
  clearLaunchLog,
  getLaunchLog,
  getStatus,
  getSystemInfo,
  installMrpack,
  isTauri,
launchGame,
  launcherVersion,
  listPacks,
  listVersions,
  loginOffline,
  msDeviceCode,
  msPoll,
  onDownloadProgress,
  onGameExited,
  onLaunchLog,
  onPlaytimeUpdated,
  openExternal,
  openPackDir,
  switchVersion,
} from "~/lib/bridge";
import type {
  AppStatus,
  DownloadProgress,
  LaunchLogEntry,
  MsDeviceCodeInfo,
  PackDescriptor,
  SystemInfo,
  UpdateInfo,
  UserSession,
  VersionsInfo,
} from "~/lib/types";

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
const CONSOLE_LIMIT = 2000;

function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 Б";
  const units = ["Б", "КБ", "МБ", "ГБ"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  return `${(bytes / Math.pow(1024, i)).toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

function formatPlaytime(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (h > 0) return `${h} ч ${m} мин`;
  return `${m} мин`;
}

function formatDate(iso: string | null): string {
  if (!iso) return "";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "";
  return d.toLocaleDateString("ru-RU", {
    day: "2-digit",
    month: "short",
    year: "numeric",
  });
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
  const tab = ref<"play" | "settings">("play");
  const notifications = ref<Notice[]>([]);
  let noticeSeq = 0;
  const launcherVer = ref("");
  const msFlow = ref<MsDeviceCodeInfo | null>(null);
  const msPolling = ref(false);
  const ISSUES_URL = "https://github.com/n1orio/nio-launcher/issues/new";

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
      "**Описание ошибки:**",
      "```",
      (errorText || "(пусто)").slice(0, 3000),
      "```",
      "",
      "**Окружение:**",
      `- Лаунчер: v${launcherVer.value || "?"}`,
      `- ОС: ${detectOS()}`,
      `- Сборка: ${activePack.value?.name ?? (packId.value || "—")}`,
      `- Установлена: ${status.value?.installed ? "да" : "нет"}`,
      status.value?.active_source_tag
        ? `- Активная версия: ${status.value.active_source_tag}`
        : null,
      status.value?.minecraft_version
        ? `- Minecraft: ${status.value.minecraft_version}${status.value.loader ? ` / ${status.value.loader}` : ""}`
        : null,
      "",
      "**Лог запуска:**",
      "```",
      (log || "(лог пуст)").slice(-50000),
      "```",
    ]
      .filter((l): l is string => l !== null)
      .join("\n");
    const title = `[Автоотчёт] ${(errorText || "Ошибка лаунчера").slice(0, 80)}`;
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

  const packs = ref<PackDescriptor[]>([]);
  const packId = ref("");

  let lastBytes = { value: 0, at: 0 };
  let speed = 0;
  let unlistenSync: (() => void) | undefined;
  let unlistenLogSync: (() => void) | undefined;
  let unlistenPlaytimeSync: (() => void) | undefined;
  let unlistenGameExitedSync: (() => void) | undefined;

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

  async function load() {
    if (!isTauri() || !packId.value) return;
    const s = await getStatus(packId.value);
    status.value = s;
    session.value = s.session;
    if (s.session) username.value = s.session.username;
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

  async function refreshVersions() {
    if (!packId.value) return;
    listVersions(packId.value)
      .then((v) => (versions.value = v))
      .catch(() => {});
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
      const list = await listPacks();
      packs.value = list;
      const saved =
        typeof localStorage !== "undefined" ? localStorage.getItem(PACK_KEY) : null;
      packId.value =
        (saved && list.some((p) => p.id === saved) ? saved : undefined) ??
        list[0]?.id ??
        "";
    } catch {
      packId.value = "untold-legends";
    }
    await load();
    refreshVersions();
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
          e.code > 0 ? ` (код ${e.code})` : e.code === 0 ? "" : " (аварийно)";
        notify(
          `Игра завершилась с ошибкой${code}. Подробности — в консоли внизу.`,
          "error"
        );
      }
    }).then((fn) => (unlistenGameExitedSync = fn));
    launcherVersion()
      .then((v) => (launcherVer.value = v))
      .catch(() => {});
  });

  onUnmounted(() => {
    unlistenSync?.();
    unlistenLogSync?.();
    unlistenPlaytimeSync?.();
    unlistenGameExitedSync?.();
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
      notify(`Ошибка установки: ${e}`);
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
        notify(`Ошибка переключения: ${e}`);
      }
    } else {
      await handleInstall(tag);
    }
  }

  async function handleOffline() {
    if (!isTauri()) return;
    if (!username.value.trim()) {
      notify("Введите никнейм", "info");
      return;
    }
    try {
      const s = await loginOffline(username.value.trim());
      session.value = s;
      await load();
    } catch (e) {
      notify(`Ошибка входа: ${e}`);
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
    } catch (e) {
      notify(`Ошибка Microsoft: ${e}`);
    } finally {
      msPolling.value = false;
      msFlow.value = null;
    }
  }

  async function openMsAuthPage() {
    if (!isTauri() || !msFlow.value) return;
    try {
      await openExternal(msFlow.value.verification_uri);
    } catch {
      notify(`Не удалось открыть страницу: ${msFlow.value.verification_uri}`, "info");
    }
  }

  async function handlePlay() {
    if (!isTauri() || !packId.value) return;
    if (!session.value) {
      notify("Войдите в аккаунт перед запуском", "info");
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
      windowHeight.value
    );
    } catch (e) {
      notify(`Ошибка запуска: ${e}`);
    } finally {
      busy.value = false;
    }
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
      notify(`Не удалось открыть папку сборки: ${e}`);
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
    packs,
    packId,
    activePack,
    percent,
    loaderLabel,
    formatBytes,
    formatDate,
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
    handlePlay,
    handleClearLog,
    handleCopyLog,
    handleOpenPackDir,
    selectPack,
    notifications,
    notify,
    dismissNotification,
    reportError,
  };
}