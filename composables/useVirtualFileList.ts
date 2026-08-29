import { ref, computed, watch, nextTick } from "vue";
import type { Ref, ComputedRef } from "vue";
import type { GameFileEntry, ModUpdate, ModrinthProject, ModrinthVersion } from "~/lib/types";
import type { GameFolderKind } from "~/lib/bridge";
import { getCachedIcon, setCachedIcon } from "~/lib/iconCache";
import { curseforgeProjectDetail } from "~/lib/bridge";

export interface UseVirtualFileListDeps {
  gameFiles: Ref<Record<string, GameFileEntry[]>>;
  playSubTab: Ref<string>;
  fileSearch: Ref<string>;
  fileToggling: Ref<Set<string>>;
  /** Функция поиска обновления для файла (из useModSearch). */
  modUpdateFor: (f: GameFileEntry) => ModUpdate | undefined;
  /** Состояния модалок для обработчика Escape. */
  exportOpen: Ref<boolean>;
  editVerOpen: Ref<boolean>;
  crashAnalysis: Ref<unknown>;
  modPackOpen: Ref<boolean>;
  modPackVersions: Ref<ModrinthVersion[] | null>;
  modPackDetail: Ref<ModrinthProject | null>;
  createPackOpen: Ref<boolean>;
  bugReportOpen: Ref<boolean>;
  searchOpen: Ref<boolean>;
  fileDetail: Ref<{ folder: GameFolderKind; entry: GameFileEntry } | null>;
  shotIdx: Ref<number | null>;
  closeCrashAnalysis: () => void;
  closeSearch: () => void;
  closeBugReport: () => void;
}

export function useVirtualFileList(deps: UseVirtualFileListDeps) {
  const {
    gameFiles,
    playSubTab,
    fileSearch,
    fileToggling,
    modUpdateFor,
    exportOpen,
    editVerOpen,
    crashAnalysis,
    modPackOpen,
    modPackVersions,
    modPackDetail,
    createPackOpen,
    bugReportOpen,
    searchOpen,
    fileDetail,
    shotIdx,
    closeCrashAnalysis,
    closeSearch,
    closeBugReport,
  } = deps;

  // ---- Виртуализация: измерение и скролл списка файлов ----
  const FILE_OVERSCAN = 10;
  const fileListRef = ref<HTMLElement | null>(null);
  const fileListScrollTop = ref(0);
  const fileListViewportH = ref(480);
  const fileRowStride = ref(64);

  function measureFileRow() {
    const row = fileListRef.value?.querySelector(".file-row") as HTMLElement | null;
    if (row && row.offsetHeight > 0) {
      fileRowStride.value = row.offsetHeight + 8;
    }
  }

  function fileListScroll(e: Event) {
    const el = e.target as HTMLElement;
    fileListScrollTop.value = el.scrollTop;
    fileListViewportH.value = el.clientHeight;
  }

  function resetFileListScroll() {
    const el = fileListRef.value;
    if (el) el.scrollTop = 0;
  }

  // ---- Сортировка ----
  const fileSortKey = ref<"none" | "name" | "date">("none");
  const fileSortDir = ref<"asc" | "desc">("asc");

  function toggleFileSort(k: "name" | "date") {
    if (fileSortKey.value === k) {
      // Повторный клик — инвертируем направление.
      fileSortDir.value = fileSortDir.value === "asc" ? "desc" : "asc";
    } else {
      fileSortKey.value = k;
      // Дата — новые сверху, имя — A→Z; повторный клик перевернёт.
      fileSortDir.value = k === "date" ? "desc" : "asc";
    }
  }

  function clearFileSort() {
    fileSortKey.value = "none";
    fileSortDir.value = "asc";
  }

  // ---- Фильтр по статусу ----
  const fileStatusFilter = ref<"all" | "enabled" | "disabled" | "updates">("all");

  function setFileStatusFilter(k: "all" | "enabled" | "disabled" | "updates") {
    fileStatusFilter.value = fileStatusFilter.value === k ? "all" : k;
  }

  // ---- Выпадающие меню ----
  const fileMenuOpen = ref(false);
  const fileMenuRef = ref<HTMLElement | null>(null);
  const exportMenuOpen = ref(false);
  const exportMenuRef = ref<HTMLElement | null>(null);

  function onFileMenuDoc(e: MouseEvent) {
    if (!fileMenuOpen.value) return;
    if (fileMenuRef.value && fileMenuRef.value.contains(e.target as Node)) return;
    fileMenuOpen.value = false;
  }

  function onFileMenuKey(e: KeyboardEvent) {
    if (e.key === "Escape") fileMenuOpen.value = false;
  }

  function onExportMenuDoc(e: MouseEvent) {
    if (!exportMenuOpen.value) return;
    if (exportMenuRef.value && exportMenuRef.value.contains(e.target as Node)) return;
    exportMenuOpen.value = false;
  }

  function onExportMenuKey(e: KeyboardEvent) {
    if (e.key === "Escape") exportMenuOpen.value = false;
  }

  /** ESC закрывает верхнюю открытую модалку (единый порядок для всех окон). */
  function onGlobalEscapeKey(e: KeyboardEvent) {
    if (e.key !== "Escape") return;
    if (exportOpen.value) {
      exportOpen.value = false;
    } else if (editVerOpen.value) {
      editVerOpen.value = false;
    } else if (crashAnalysis.value) {
      closeCrashAnalysis();
    } else if (modPackOpen.value) {
      modPackOpen.value = false;
      modPackVersions.value = null;
      modPackDetail.value = null;
    } else if (createPackOpen.value) {
      createPackOpen.value = false;
    } else if (bugReportOpen.value) {
      closeBugReport();
    } else if (searchOpen.value) {
      closeSearch();
    } else if (fileDetail.value !== null) {
      fileDetail.value = null;
    } else if (shotIdx.value !== null) {
      shotIdx.value = null;
    }
  }

  // ---- Пайплайн фильтрации/сортировки ----
  const fileListFiltered = computed(() => {
    let list = gameFiles.value[playSubTab.value as GameFolderKind] ?? [];
    if (fileSortKey.value === "name") {
      const c = fileSortDir.value === "asc" ? 1 : -1;
      list = [...list].sort((a, b) =>
        c * a.displayName.toLowerCase().localeCompare(b.displayName.toLowerCase())
      );
    } else if (fileSortKey.value === "date") {
      const c = fileSortDir.value === "asc" ? 1 : -1;
      list = [...list].sort(
        (a, b) =>
          c *
          (a.modified - b.modified ||
            a.displayName.toLowerCase().localeCompare(b.displayName.toLowerCase()))
      );
    }
    // "none" — порядок с бэка (включённые сверху, затем алфавит).
    const q = fileSearch.value.trim().toLowerCase();
    if (q) {
      list = list.filter(
        (f) => f.displayName.toLowerCase().includes(q) || f.name.toLowerCase().includes(q)
      );
    }
    const st = fileStatusFilter.value;
    if (st === "enabled") return list.filter((f) => f.enabled);
    if (st === "disabled") return list.filter((f) => !f.enabled);
    if (st === "updates" && playSubTab.value !== "saves") {
      return list.filter((f) => !!modUpdateFor(f));
    }
    return list;
  });

  // ---- Индексы виртуализации ----
  const fileListTotal = computed(
    () => fileListFiltered.value.length * fileRowStride.value - 8
  );

  const fileListStart = computed(() =>
    Math.max(0, Math.floor(fileListScrollTop.value / fileRowStride.value) - FILE_OVERSCAN)
  );

  const fileListVisible = computed(() => {
    const list = fileListFiltered.value;
    if (!list.length) return list;
    const start = fileListStart.value;
    const end = Math.min(
      list.length,
      Math.ceil((fileListScrollTop.value + fileListViewportH.value) / fileRowStride.value) +
        FILE_OVERSCAN
    );
    return list.slice(start, Math.max(end, start + 1));
  });

  // Сброс скролла при изменении отфильтрованного списка или поиска.
  watch(
    () => [fileListFiltered.value.length, fileSearch.value],
    () => {
      fileListScrollTop.value = 0;
      nextTick(() => {
        resetFileListScroll();
        measureFileRow();
      });
    }
  );

  // ---- Метаданные Modrinth (название + аватар проекта), подгружаются лениво -----
  const modrinthMeta = ref<Record<string, { title: string; icon: string; version?: string }>>({});

  // ---- Метаданные CurseForge (название + иконка проекта), подгружаются лениво ----
  const curseMeta = ref<Record<number, { title: string; icon: string }>>({});

  function curseMetaFor(f: GameFileEntry) {
    // Мета напрямую из трекера (уже в GameFileEntry) — без API-запроса.
    if (f.curseforgeTitle || f.curseforgeIcon) {
      return { title: f.curseforgeTitle ?? "", icon: f.curseforgeIcon ?? "" };
    }
    if (!f.curseforgeProjectId) return undefined;
    return curseMeta.value[f.curseforgeProjectId];
  }

  /** Человекочитаемое название файла в списке: мета Modrinth или CurseForge, иначе имя файла. */
  function fileMetaTitle(f: GameFileEntry): string {
    const mr = modrinthMetaFor(f)?.title;
    if (mr) return mr;
    const cf = curseMetaFor(f)?.title;
    if (cf) return cf;
    return f.name;
  }

  async function fetchCurseMeta(projectId: number) {
    if (!projectId || curseMeta.value[projectId]) return;
    const cacheKey = `cf:${projectId}`;
    const cached = getCachedIcon(cacheKey);
    if (cached) {
      try {
        const j = JSON.parse(cached.data) as { title?: string; icon?: string };
        if (j && typeof j.title === "string") {
          curseMeta.value = {
            ...curseMeta.value,
            [projectId]: { title: j.title, icon: typeof j.icon === "string" ? j.icon : "" },
          };
          if (!cached.stale) return;
        }
      } catch {
        /* повреждённая запись — перезагрузим с API */
      }
    }
    try {
      const d = await curseforgeProjectDetail(projectId);
      const meta = { title: d.name, icon: d.iconUrl ?? "" };
      curseMeta.value = { ...curseMeta.value, [projectId]: meta };
      setCachedIcon(cacheKey, JSON.stringify(meta));
    } catch {
      curseMeta.value = { ...curseMeta.value, [projectId]: { title: "", icon: "" } };
    }
  }

  // ---- Вспомогатели Modrinth ----
  function modrinthProjectId(url: string): string | null {
    return url.match(/\/mod\/([^/]+)/)?.[1] ?? null;
  }

  /** ID проекта Modrinth файла: из трекинга (скачан вручную) либо из URL индекса сборки. */
  function modrinthProjectIdFor(f: GameFileEntry): string | null {
    if (f.modrinthProjectId) return f.modrinthProjectId;
    if (!f.modrinthUrl) return null;
    return modrinthProjectId(f.modrinthUrl) ?? null;
  }

  function modrinthMetaFor(f: GameFileEntry) {
    const id = modrinthProjectIdFor(f);
    return id ? modrinthMeta.value[id] : undefined;
  }

  /** Версия Modrinth файла (числовое имя версии), если известна. */
  function modrinthVersionFor(f: GameFileEntry): string | undefined {
    return modrinthMetaFor(f)?.version;
  }

  async function fetchModrinthMeta(f: GameFileEntry) {
    const id = modrinthProjectIdFor(f);
    if (!id || modrinthMeta.value[id]) return;
    const versionId = f.modrinthVersionId;
    const cacheKey = `mr:${id}`;
    const cached = getCachedIcon(cacheKey);
    if (cached) {
      try {
        const j = JSON.parse(cached.data) as { title?: string; icon?: string; version?: string };
        if (j && typeof j.title === "string") {
          modrinthMeta.value = {
            ...modrinthMeta.value,
            [id]: {
              title: j.title,
              icon: typeof j.icon === "string" ? j.icon : "",
              version: typeof j.version === "string" ? j.version : undefined,
            },
          };
          if (!cached.stale) return;
        }
      } catch {
        /* повреждённая запись — перезагрузим с API */
      }
    }
    try {
      const [proj, ver] = await Promise.all([
        fetch(`https://api.modrinth.com/v2/project/${id}?fields=title,icon_url`),
        versionId
          ? fetch(`https://api.modrinth.com/v2/version/${versionId}?fields=version_number`)
          : Promise.resolve(null),
      ]);
      const meta: { title: string; icon: string; version?: string } = { title: "", icon: "" };
      if (proj.ok) {
        const j = await proj.json();
        if (typeof j?.title !== "string") return;
        meta.title = j.title;
        meta.icon = typeof j.icon_url === "string" ? j.icon_url : "";
      }
      if (ver?.ok) {
        const j = await ver.json();
        if (typeof j?.version_number === "string") meta.version = j.version_number;
      }
      if (!meta.title) return;
      modrinthMeta.value = { ...modrinthMeta.value, [id]: meta };
      setCachedIcon(cacheKey, JSON.stringify(meta));
    } catch {
      /* метаданные некритичны */
    }
  }

  // ---- Иконки из поиска: кеш data-URL, чтобы повторные поиски не дёргали сеть ----
  const searchIconData = ref<Record<string, string>>({});

  function searchIconUrl(url: string): string {
    return searchIconData.value[url] ?? url;
  }

  async function warmSearchIcon(url: string) {
    if (!url || searchIconData.value[url]) return;
    const cached = getCachedIcon(`pic:${url}`);
    if (cached) {
      searchIconData.value = { ...searchIconData.value, [url]: cached.data };
      if (!cached.stale) return;
    }
    try {
      const res = await fetch(url);
      if (!res.ok) return;
      const blob = await res.blob();
      const dataUrl = await new Promise<string>((resolve, reject) => {
        const r = new FileReader();
        r.onload = () => resolve(String(r.result));
        r.onerror = () => reject(new Error("file read"));
        r.readAsDataURL(blob);
      });
      searchIconData.value = { ...searchIconData.value, [url]: dataUrl };
      setCachedIcon(`pic:${url}`, dataUrl);
    } catch {
      /* иконки некритичны */
    }
  }

  // Тянем мету только для видимых строк (виртуализированный список).
  watch(fileListVisible, (rows) => {
    for (const f of rows) {
      if (f.modrinthProjectId || f.modrinthUrl) fetchModrinthMeta(f);
      if (f.curseforgeProjectId) fetchCurseMeta(f.curseforgeProjectId);
    }
  });

  return {
    // Виртуализация
    FILE_OVERSCAN,
    fileListRef,
    fileListScrollTop,
    fileListViewportH,
    fileRowStride,
    measureFileRow,
    fileListScroll,
    fileListTotal,
    resetFileListScroll,
    fileListStart,
    fileListVisible,
    fileListFiltered,

    // Сортировка
    fileSortKey,
    fileSortDir,
    toggleFileSort,
    clearFileSort,

    // Фильтр по статусу
    fileStatusFilter,
    setFileStatusFilter,

    // Выпадающие меню
    fileMenuOpen,
    fileMenuRef,
    exportMenuOpen,
    exportMenuRef,
    onFileMenuDoc,
    onFileMenuKey,
    onExportMenuDoc,
    onExportMenuKey,
    onGlobalEscapeKey,

    // Метаданные
    modrinthMeta,
    curseMeta,
    curseMetaFor,
    fileMetaTitle,
    fetchCurseMeta,
    fetchModrinthMeta,
    modrinthProjectIdFor,
    modrinthMetaFor,
    modrinthVersionFor,
    searchIconData,
    searchIconUrl,
    warmSearchIcon,
  };
}
