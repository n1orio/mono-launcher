import { computed, ref } from "vue";
import {
  curseforgeModpackFiles,
  curseforgeProjectDetail,
  curseforgeInstallFile,
  curseforgeLatestFile,
  installedModSha1,
  isTauri,
  modrinthInstallMod,
  modrinthProject,
  modrinthProjectVersions,
  modrinthUpdateMod,
} from "~/lib/bridge";
import type { GameFolderKind, ModrinthInstallFolder, ModrinthSearchKind } from "~/lib/bridge";
import type {
  GameFileEntry,
  ModrinthProject,
  ModrinthVersion,
  CurseProjectDetail,
  CursePackFile,
  CurseFile,
  AppStatus,
} from "~/lib/types";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18n } from "~/composables/useI18n";
import { cap } from "~/lib/misc";

const { t } = useI18n();

export interface UseFileDetailDeps {
  packId: { value: string | null };
  status: { value: AppStatus | null };
  notify: (text: string, type?: string) => void;
  loadGameFiles: (folder: string, force?: boolean) => Promise<void>;
  refreshModUpdates: (force?: boolean) => Promise<void>;
}

export function useFileDetail(deps: UseFileDetailDeps) {
  const { packId, status, notify, loadGameFiles, refreshModUpdates } = deps;

  // ---- Просмотр ресурса из списка установленных файлов (страница в лаунчере) ----
  const fileDetail = ref<{ folder: GameFolderKind; entry: GameFileEntry } | null>(null);
  const fileDetailMrLoading = ref(false);
  const fileDetailMr = ref<ModrinthProject | null>(null);
  const fileDetailMrVersions = ref<ModrinthVersion[] | null>(null);
  const fileDetailInstalledSha = ref<string | null>(null);
  const fileDetailMcFilter = ref<string[]>([]);
  const fileDetailTab = ref<"about" | "versions" | "gallery">("about");
  const fileDetailTabs = computed<{ kind: "about" | "versions" | "gallery" }[]>(() =>
    fileDetailMr.value
      ? [{ kind: "about" }, { kind: "versions" }, { kind: "gallery" }]
      : [{ kind: "about" }, { kind: "versions" }]
  );
  const fileDetailMrVersionBusy = ref<string | null>(null);
  const fileDetailCfLoading = ref(false);
  const updatingFileDetail = ref(false);
  const fileDetailCf = ref<CurseProjectDetail | null>(null);
  /** Строка — с какого проекта CurseForge открыт просмотр (для кнопки «обновить»). */
  const fileDetailFolder = ref<GameFolderKind>("mods");
  /** Заголовок окна просмотра ресурса (из имени файла), пока проект не подгружен. */
  const fileDetailTitle = ref("");

  // ---- CurseForge: версии ресурса в окне просмотра ----
  const fileDetailCfVersions = ref<CursePackFile[] | null>(null);
  const fileDetailCfVersionBusy = ref<number | null>(null);
  const fileDetailCfMcFilter = ref<string | null>(null);
  const fileDetailCfMcSel = computed({
    get: () => (fileDetailCfMcFilter.value ? [fileDetailCfMcFilter.value] : []),
    set: (v) => { fileDetailCfMcFilter.value = v[0] ?? null; },
  });
  async function loadFileDetailCfVersions(projectId: number) {
    try {
      const files = await curseforgeModpackFiles(projectId);
      files.sort((a, b) => b.fileId - a.fileId);
      fileDetailCfVersions.value = files;
    } catch {
      fileDetailCfVersions.value = [];
    }
  }
  const fileDetailCfMcOptions = computed(() => {
    const set = new Set<string>();
    for (const f of fileDetailCfVersions.value ?? []) if (f.gameVersion) set.add(f.gameVersion);
    return [...set].sort((a, b) => b.localeCompare(a, undefined, { numeric: true })).map((v) => ({ value: v, label: v }));
  });
  const fileDetailCfFilteredVersions = computed(() => {
    const all = fileDetailCfVersions.value ?? [];
    const mc = fileDetailCfMcFilter.value;
    return mc ? all.filter((f) => f.gameVersion === mc) : all;
  });
  /** Прямая ссылка на файл CDN CurseForge (доверенный хост). */
  function forgecdnUrl(fileId: number, fileName: string): string {
    return `https://mediafilez.forgecdn.net/files/${Math.floor(fileId / 1000)}/${fileId % 1000}/${encodeURIComponent(fileName)}`;
  }
  async function installFileDetailCfVersion(f: CursePackFile) {
    const d = fileDetail.value;
    if (!d || !packId.value || !isTauri()) return;
    if (fileDetailCfVersionBusy.value !== null) return;
    fileDetailCfVersionBusy.value = f.fileId;
    try {
      const projectId = d.entry.curseforgeProjectId ?? fileDetailCf.value?.projectId ?? 0;
      const folder = (fileDetailFolder.value === "saves" ? "mods" : fileDetailFolder.value) as ModrinthInstallFolder;
      const title = fileDetailTitle.value || fileDetailCf.value?.name || f.displayName || f.fileName;
      const file: CurseFile = {
        fileId: f.fileId,
        projectId,
        fileName: f.fileName,
        downloadUrl: forgecdnUrl(f.fileId, f.fileName),
        sha1: "",
        gameVersion: f.gameVersion,
      };
      await curseforgeInstallFile(packId.value, file, folder, title);
      await loadGameFiles(fileDetailFolder.value, true);
      await refreshModUpdates(true);
      notify(t("mods.installed", { kind: kindNoun(folder), name: f.displayName || f.fileName }), "success");
    } catch (e) {
      notify(t("files.updateErr", { e }), "error");
    } finally {
      fileDetailCfVersionBusy.value = null;
    }
  }

  /** Закрывает отдельное окно просмотра ресурса. */
  async function closeFileDetailWin() {
    if (isTauri()) {
    try {
    await getCurrentWindow().close();
    } catch {
    /* окно уже закрывается */
    }
    }
  }

  async function openFileDetail(folder: GameFolderKind, entry: GameFileEntry) {
    let slug = entry.modrinthProjectId || "";
    const m = /\/mod\/([^/]+)\/?$/.exec(entry.modrinthUrl ?? "");
    if (!slug && m) slug = m[1];
    // В Tauri — настоящее отдельное окно просмотра ресурса (как окно скачки мода).
    if (isTauri() && (slug || entry.curseforgeProjectId)) {
    if (!packId.value) return;
    const existing = await WebviewWindow.getByLabel("filedetail");
    if (existing) {
    try {
    await existing.close();
    } catch {
    /* окно уже закрывается */
    }
    }
    const devBase = import.meta.env.DEV ? "http://localhost:1420/" : "";
    try {
    new WebviewWindow("filedetail", {
    url: `${devBase}?win=filedetail&slug=${encodeURIComponent(slug)}&cfid=${entry.curseforgeProjectId || ""}&folder=${folder}&packId=${encodeURIComponent(packId.value)}&name=${encodeURIComponent(entry.displayName || entry.name || "")}`,
    title: entry.displayName || entry.name || t("files.view"),
    width: 820,
    height: 660,
    minWidth: 560,
    minHeight: 420,
    resizable: true,
    decorations: false,
    });
    } catch (e) {
    notify(t("mods.windowErr", { e }), "error");
    }
    return;
    }
    fileDetail.value = { folder, entry };
    fileDetailFolder.value = folder;
    fileDetailMr.value = null;
    fileDetailMrVersions.value = null;
    fileDetailCf.value = null;
    fileDetailCfVersions.value = null;
    fileDetailCfMcFilter.value = null;
    fileDetailTab.value = "about";
    if (slug) {
    fileDetailMrLoading.value = true;
    try {
    fileDetailMr.value = await modrinthProject(slug);
    const fl = folder === "saves" && (entry.kind === "dir" ? true : false) ? "mods" : folder;
    await loadFileDetailVersions(slug, fl);
    const ver = status.value?.minecraft_version;
    const ldr = status.value?.loader?.replace("-loader", "");
    fileDetailMcFilter.value = ver ? [ver] : [];
    fileDetailLoaderFilter.value = ldr ? [ldr] : [];
    fileDetailTypeFilter.value = [];
    void loadFileDetailInstalledSha(fileDetailMr.value.projectId);
    } catch {
    /* не удалось — остаётся placeholder проекта */
    } finally {
    fileDetailMrLoading.value = false;
    }
    } else if (entry.curseforgeProjectId) {
    void loadFileDetailCfVersions(entry.curseforgeProjectId);
    fileDetailCfLoading.value = true;
    try {
    fileDetailCf.value = await curseforgeProjectDetail(entry.curseforgeProjectId);
    } catch {
    fileDetailCf.value = null;
    } finally {
    fileDetailCfLoading.value = false;
    }
    }
  }

  async function loadFileDetailVersions(slug: string, folder: GameFolderKind) {
    try {
    fileDetailMrVersions.value = await modrinthProjectVersions(slug, undefined, undefined);
    } catch {
    fileDetailMrVersions.value = [];
    }
  }

  /** Загружает sha1 установленного файла проекта, чтобы отметить текущую версию. */
  async function loadFileDetailInstalledSha(projectId: string) {
    fileDetailInstalledSha.value = null;
    if (!isTauri() || !packId.value || !projectId) return;
    try {
    fileDetailInstalledSha.value = (await installedModSha1(packId.value, projectId)) ?? null;
    } catch {
    fileDetailInstalledSha.value = null;
    }
  }

  /** Установлена ли уже эта версия (по sha1 файла среди файлов версии). */
  const fileDetailInstalledVersion = (v: ModrinthVersion) =>
    !!fileDetailInstalledSha.value && v.files.some((f) => {
    const h = f.hashes?.["sha1"];
    return !!h && h.toLowerCase() === fileDetailInstalledSha.value;
    });

  /** Все версии игры, встречающиеся у файла (для фильтра версии). */
  const fileDetailMcOptions = computed(() =>
    Array.from(new Set((fileDetailMrVersions.value ?? []).flatMap((v) => v.gameVersions)))
    .sort(verCmpDesc)
    .map((mc) => ({ value: mc, label: mc }))
  );
  /** Платформы/загрузчики, встречающиеся у файла. */
  const fileDetailLoaderOptions = computed(() =>
    Array.from(new Set((fileDetailMrVersions.value ?? []).flatMap((v) => v.loaders)))
    .map((l) => ({ value: l, label: cap(l) }))
    .sort((a, b) => a.label.localeCompare(b.label))
  );

  const fileDetailMcSel = computed({
    get: () => fileDetailMcFilter.value,
    set: (v: string[]) => { fileDetailMcFilter.value = v; },
  });
  const fileDetailLoaderFilter = ref<string[]>([]);
  const fileDetailTypeFilter = ref<string[]>([]);
  const fileDetailLoaderSel = computed({
    get: () => fileDetailLoaderFilter.value,
    set: (v: string[]) => { fileDetailLoaderFilter.value = v; },
  });
  const fileDetailTypeSel = computed({
    get: () => fileDetailTypeFilter.value,
    set: (v: string[]) => { fileDetailTypeFilter.value = v; },
  });

  /** Версии с применёнными фильтрами (версия игры, загрузчик, канал). */
  const fileDetailFilteredVersions = computed<ModrinthVersion[]>(() => {
    const all = fileDetailMrVersions.value ?? [];
    return all.filter((v) =>
    (fileDetailMcFilter.value.length === 0 || fileDetailMcFilter.value.some((mc) => v.gameVersions.includes(mc))) &&
    (fileDetailLoaderFilter.value.length === 0 || fileDetailLoaderFilter.value.some((l) => v.loaders.includes(l))) &&
    (fileDetailTypeFilter.value.length === 0 || fileDetailTypeFilter.value.includes(v.versionType))
    );
  });

  /** Сортировка версий «1.21.1» по убыванию (новые сверху). */
  function verCmpDesc(a: string, b: string): number {
    const pa = a.split(".").map((x) => parseInt(x, 10) || 0);
    const pb = b.split(".").map((x) => parseInt(x, 10) || 0);
    const n = Math.max(pa.length, pb.length);
    for (let i = 0; i < n; i++) {
    const da = pa[i] ?? 0;
    const db = pb[i] ?? 0;
    if (da !== db) return db - da;
    }
    return 0;
  }

  /** Кнопка «открыть страницу» внешнего сервиса. */
  function fileDetailExternalUrl(): string | null {
    const d = fileDetail.value;
    if (!d) return null;
    if (d.entry.curseforgeProjectId && fileDetailCf.value?.websiteUrl) {
    return fileDetailCf.value.websiteUrl;
    }
    const slug = fileDetailMr.value?.slug || d.entry.modrinthProjectId;
    const m = /\/mod\/([^/]+)\/?$/.exec(d.entry.modrinthUrl ?? "");
    if (slug) return `https://modrinth.com/mod/${slug}`;
    if (m) return `https://modrinth.com/mod/${m[1]}`;
    return null;
  }

  /** Обновление: Modrinth — текущая версия через update, CurseForge — последняя версия. */
  async function updateFileDetail() {
    const d = fileDetail.value;
    if (!d || !packId.value || updatingFileDetail.value) return;
    const folder = fileDetailFolder.value === "saves" ? "mods" : fileDetailFolder.value;
    updatingFileDetail.value = true;
    try {
    if (d.entry.curseforgeProjectId) {
    const file = await curseforgeLatestFile(packId.value, d.entry.curseforgeProjectId);
    await curseforgeInstallFile(packId.value, file, folder);
    } else if (d.entry.modrinthProjectId && d.entry.name) {
    await modrinthUpdateMod(packId.value, d.entry.name);
    } else if (fileDetailMr.value) {
    // Нет трекера, но есть slug — установим последнюю подходящую версию
    // под версию сборки (через серверную фильтрацию по MC).
    const mc = status.value?.minecraft_version || undefined;
    const versions = fileDetailMr.value
    ? await modrinthProjectVersions(fileDetailMr.value.projectId, mc, undefined)
    : [];
    const target = versions[0] ?? fileDetailMrVersions.value?.[0];
    if (target) {
    await modrinthInstallMod(packId.value, target.id, folder as ModrinthInstallFolder);
    }
    }
    await loadGameFiles(fileDetailFolder.value, true);
    await refreshModUpdates(true);
    notify(t("files.updated"), "success");
    } catch (e) {
    notify(t("files.updateErr", { e }));
    } finally {
    updatingFileDetail.value = false;
    }
  }

  /** Установка конкретной версии из просмотра ресурса (Modrinth). */
  async function installFileDetailVersion(v: ModrinthVersion) {
    const d = fileDetail.value;
    if (!d || !packId.value || fileDetailMrVersionBusy.value) return;
    if (d.entry.curseforgeProjectId) return;
    const folder = (fileDetailFolder.value === "saves" ? "mods" : fileDetailFolder.value) as ModrinthInstallFolder;
    fileDetailMrVersionBusy.value = v.id;
    try {
    await modrinthInstallMod(packId.value, v.id, folder);
    await loadGameFiles(fileDetailFolder.value, true);
    await refreshModUpdates(true);
    notify(t("mods.installed", { kind: kindNoun(folder), name: v.name }), "success");
    } catch (e) {
    notify(t("files.updateErr", { e }));
    } finally {
    fileDetailMrVersionBusy.value = null;
    }
  }

  return {
    fileDetail,
    fileDetailMrLoading,
    fileDetailMr,
    fileDetailMrVersions,
    fileDetailInstalledSha,
    fileDetailMcFilter,
    fileDetailTab,
    fileDetailTabs,
    fileDetailMrVersionBusy,
    fileDetailCfLoading,
    updatingFileDetail,
    fileDetailCf,
    fileDetailFolder,
    fileDetailTitle,
    fileDetailCfVersions,
    fileDetailCfVersionBusy,
    fileDetailCfMcFilter,
    fileDetailCfMcSel,
    fileDetailCfMcOptions,
    fileDetailCfFilteredVersions,
    fileDetailLoaderFilter,
    fileDetailTypeFilter,
    fileDetailMcSel,
    fileDetailLoaderSel,
    fileDetailTypeSel,
    fileDetailMcOptions,
    fileDetailLoaderOptions,
    fileDetailFilteredVersions,
    openFileDetail,
    closeFileDetailWin,
    loadFileDetailVersions,
    loadFileDetailInstalledSha,
    loadFileDetailCfVersions,
    installFileDetailCfVersion,
    installFileDetailVersion,
    updateFileDetail,
    fileDetailExternalUrl,
    forgecdnUrl,
    fileDetailInstalledVersion,
    verCmpDesc,
  };
}

function kindNoun(v: ModrinthSearchKind | ModrinthInstallFolder): string {
  switch (v) {
  case "mod":
  case "mods":
  return t("mods.kindMod");
  case "resourcepack":
  case "resourcepacks":
  return t("mods.kindRP");
  case "shaderpack":
  case "shaderpacks":
  return t("mods.kindShaders");
  default:
  return t("mods.kindDatapack");
  }
}
