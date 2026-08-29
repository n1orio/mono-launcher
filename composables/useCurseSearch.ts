import { computed, ref, watch } from "vue";
import type { Ref } from "vue";
import type { CurseSearchHit, CurseProjectDetail, CursePackFile, CurseInstallResult, GameFileEntry, AppStatus, ModrinthProject } from "~/lib/types";
import type { GameFolderKind, ModrinthInstallFolder, ModrinthSearchKind } from "~/lib/bridge";
import {
  curseforgeSearch,
  curseforgeCategories,
  curseforgeLatestFile,
  curseforgeInstallFile,
  curseforgeKeyConfigured,
  curseforgeProjectDetail,
  curseforgeModpackFiles,
  isTauri,
} from "~/lib/bridge";
import { setCachedIcon } from "~/lib/iconCache";
import { useI18n } from "~/composables/useI18n";

export interface UseCurseSearchDeps {
  packId: Ref<string | null>;
  modSearchKind: Ref<ModrinthSearchKind>;
  status: Ref<AppStatus | null>;
  gameFiles: Ref<Record<string, GameFileEntry[]>>;
  modSearchResults: Ref<ModrinthProject[]>;
  modDetail: Ref<unknown>;
  curseMeta: Ref<Record<number, { title: string; icon: string }>>;
  notify: (text: string, type?: string) => void;
  loadGameFiles: (folder: string, force?: boolean) => Promise<void>;
  closeSearch: () => void;
  refreshModUpdates: (force?: boolean) => Promise<void>;
  warmSearchIcon: (url: string) => void;
}

/** Класс проектов CurseForge для типа проекта (моды/ресурспаки/шейдеры). */
const CURSE_CLASS: Partial<Record<ModrinthSearchKind, number>> = {
  mod: 6,
  resourcepack: 12,
  shaderpack: 6552,
};

/** Папка игры для типа проекта CurseForge. */
const CURSE_FOLDER: Partial<Record<ModrinthSearchKind, ModrinthInstallFolder>> = {
  mod: "mods",
  resourcepack: "resourcepacks",
  shaderpack: "shaderpacks",
};

/* CurseForge ModsSearchSortField: 1 Featured, 2 Popularity, 3 LastUpdated, 4 Name, 6 TotalDownloads. */
const CURSE_SORT = [
  { value: "2", labelKey: "mods.sortRelevance" },
  { value: "6", labelKey: "mods.sortDownloads" },
  { value: "3", labelKey: "mods.sortNewest" },
  { value: "4", labelKey: "mods.sortName" },
];

export function useCurseSearch(deps: UseCurseSearchDeps) {
  const {
    packId,
    modSearchKind,
    gameFiles,
    modSearchResults,
    modDetail,
    curseMeta,
    notify,
    loadGameFiles,
    closeSearch,
    refreshModUpdates,
    warmSearchIcon,
  } = deps;

  const { t } = useI18n();

  // ---- Состояние поиска CurseForge ----
  const curseQuery = ref("");
  const curseLoading = ref(false);
  const curseSearched = ref(false);
  const curseResults = ref<CurseSearchHit[]>([]);
  const curseErr = ref("");
  const curseInstallBusy = ref<number | null>(null);
  const curseKeyOk = ref(true);

  // Тёплый кеш иконок для видимых результатов поиска (оба сервиса). Объявлен после
  // результатов: watch исполняет геттер сразу при setup и не должен ссылаться
  // на ещё не инициализированные ref'ы (TDZ).
  watch(
    () => [modSearchResults.value.map((r) => r.iconUrl), curseResults.value.map((r) => r.iconUrl)],
    ([m, c]) => {
      for (const u of [...m, ...c]) {
        if (u) warmSearchIcon(u);
      }
    },
    { deep: true },
  );

  // Установленные в активной сборке проекты — для плашки «Установлено» в поиске.
  // Modrinth: slug из трекера (.mono-modrinth.json) или из modrinth_url (`/mod/{slug}`);
  // CurseForge: project_id из трекера (.mono-curseforge.json).
  const installedModrinthSlugs = computed(() => {
    const set = new Set<string>();
    for (const list of Object.values(gameFiles.value)) {
      for (const f of list) {
        if (f.modrinthProjectId) set.add(f.modrinthProjectId);
        const m = /\/mod\/([^/]+)\/?$/.exec(f.modrinthUrl ?? "");
        if (m) set.add(m[1]);
      }
    }
    return set;
  });

  const installedCurseIds = computed(() => {
    const set = new Set<number>();
    for (const list of Object.values(gameFiles.value)) {
      for (const f of list) {
        if (f.curseforgeProjectId) set.add(f.curseforgeProjectId);
      }
    }
    return set;
  });

  // ---- Ключ API ----
  async function loadCurseKeyStatus() {
    if (!isTauri()) return;
    try {
      curseKeyOk.value = await curseforgeKeyConfigured();
    } catch {
      curseKeyOk.value = false;
    }
  }

  // ---- Категории CurseForge для фильтра (грузим по классу проекта) ----
  const curseCatOptions = ref<{ value: string; label: string }[]>([]);
  const curseCatIds = ref<number[]>([]);
  const curseCatSel = computed({
    get: () => curseCatIds.value.map(String),
    set: (v: string[]) => {
      curseCatIds.value = v.map(Number);
    },
  });

  /** Загрузчики CurseForge (modLoaderType: 0=Any, 1=Forge, 4=Fabric, 5=Quilt, 6=NeoForge). */
  const CF_MOD_LOADER_TYPES = [
    { value: "1", label: "Forge" },
    { value: "4", label: "Fabric" },
    { value: "5", label: "Quilt" },
    { value: "6", label: "NeoForge" },
  ];
  const curseLoaderOptions = ref<{ value: string; label: string }[]>(CF_MOD_LOADER_TYPES);
  const curseLoaderType = ref<number | null>(null);
  const curseLoaderSel = computed({
    get: () => (curseLoaderType.value !== null ? [String(curseLoaderType.value)] : []),
    set: (v: string[]) => {
      curseLoaderType.value = v[0] ? Number(v[0]) : null;
    },
  });
  const CF_LOADER_MAP: Record<string, number> = { forge: 1, fabric: 4, quilt: 5, neoforge: 6 };

  async function loadCurseCategories() {
    if (!isTauri()) return;
    const cls = CURSE_CLASS[modSearchKind.value] ?? 6;
    curseCatOptions.value = [];
    curseCatIds.value = [];
    curseLoaderType.value = null;
    try {
      const cats = await curseforgeCategories(cls);
      curseCatOptions.value = cats.map((c) => ({ value: String(c.id), label: c.name }));
    } catch {
      /* фильтр просто не появится */
    }
  }

  // ---- Фильтры CurseForge: версия Minecraft и сортировка ----
  const curseVersion = ref("");
  const curseVerSel = computed({
    get: () => (curseVersion.value ? [curseVersion.value] : []),
    set: (v: string[]) => {
      curseVersion.value = v[0] ?? "";
    },
  });

  const curseSortField = ref<string>("2");
  const curseSortSel = computed({
    get: () => [curseSortField.value],
    set: (v: string[]) => {
      curseSortField.value = v[0] ?? "2";
    },
  });

  const curseSortOptions = CURSE_SORT.map((s) => ({ value: s.value, label: t(s.labelKey) }));

  // ---- Поиск на CurseForge ----
  async function searchCurse() {
    if (!isTauri() || !packId.value) return;
    modDetail.value = null;
    curseLoading.value = true;
    curseSearched.value = true;
    curseErr.value = "";
    try {
      const categoryIds = [...curseCatIds.value];
      curseResults.value = await curseforgeSearch(
        curseQuery.value.trim(),
        CURSE_CLASS[modSearchKind.value] ?? 6,
        categoryIds,
        curseVersion.value || undefined,
        curseSortField.value,
        curseLoaderType.value ?? undefined,
      );
    } catch (e) {
      curseResults.value = [];
      curseErr.value = String(e);
    } finally {
      curseLoading.value = false;
    }
  }

  /** Скачивает последний подходящий файл проекта CurseForge в папку вкладки.
   *  Возвращает результат установки (false — ошибка/нет данных). */
  async function installCurseCore(p: CurseSearchHit): Promise<CurseInstallResult | null> {
    if (!isTauri() || !packId.value) return null;
    curseInstallBusy.value = p.projectId;
    try {
      const file = await curseforgeLatestFile(packId.value, p.projectId);
      const folder = (CURSE_FOLDER[modSearchKind.value] ?? "mods") as GameFolderKind;
      return await curseforgeInstallFile(packId.value, file, folder, p.name, p.iconUrl);
    } finally {
      curseInstallBusy.value = null;
    }
  }

  /** Скачивает один проект CurseForge с уведомлением и обновлением списка файлов. */
  async function installCurse(p: CurseSearchHit) {
    if (!isTauri() || !packId.value || curseInstallBusy.value !== null) return;
    try {
      const res = await installCurseCore(p);
      if (!res) {
        notify(t("curse.installErr", { e: "unknown" }));
        return;
      }
      notify(
        res.depsInstalled > 0
          ? t("curse.installedDeps", { name: p.name, deps: res.depsInstalled })
          : t("curse.installed", { name: p.name }),
        "success",
      );
      // Сразу сохраняем мету из поискового хита (название + иконка) в кеш, чтобы
      // main-окно показало их без отдельного API-запроса project_detail.
      curseMeta.value = { ...curseMeta.value, [p.projectId]: { title: p.name, icon: p.iconUrl ?? "" } };
      setCachedIcon(`cf:${p.projectId}`, JSON.stringify({ title: p.name, icon: p.iconUrl ?? "" }));
      closeSearch();
      const folder = (CURSE_FOLDER[modSearchKind.value] ?? "mods") as GameFolderKind;
      await loadGameFiles(folder, true);
      await refreshModUpdates(true);
    } catch (e) {
      notify(t("curse.installErr", { e }));
    }
  }

  // ---- Детальный просмотр проекта CurseForge ----
  const curseDetail = ref<CurseProjectDetail | null>(null);
  const curseDetailTab = ref<"about" | "versions" | "screenshots">("about");
  const curseVersions = ref<CursePackFile[] | null>(null);

  async function openCurseDetail(p: CurseSearchHit) {
    modDetail.value = null;
    curseDetail.value = null;
    curseDetailTab.value = "about";
    curseVersions.value = null;
    curseDetail.value = {
      projectId: p.projectId,
      name: p.name,
      slug: "",
      summary: p.summary,
      description: p.description ?? undefined,
      author: p.author,
      downloadCount: p.downloadCount,
      iconUrl: p.iconUrl,
      screenshots: [],
      categories: [],
      websiteUrl: `https://www.curseforge.com/projects/${p.projectId}`,
    };
    try {
      const detail = await curseforgeProjectDetail(p.projectId);
      curseDetail.value = detail;
    } catch {
      /* остаётся карточка из поиска */
    }
    // Загружаем файлы проекта
    try {
      curseVersions.value = await curseforgeModpackFiles(p.projectId);
    } catch {
      curseVersions.value = [];
    }
  }

  function closeCurseDetail() {
    curseDetail.value = null;
    curseVersions.value = null;
  }

  return {
    // Состояние
    curseQuery,
    curseLoading,
    curseSearched,
    curseResults,
    curseErr,
    curseInstallBusy,
    curseKeyOk,
    // Фильтры
    curseCatOptions,
    curseCatIds,
    curseCatSel,
    curseLoaderOptions,
    curseLoaderType,
    curseLoaderSel,
    curseVersion,
    curseVerSel,
    curseSortField,
    curseSortSel,
    curseSortOptions,
    // Установленные проекты
    installedModrinthSlugs,
    installedCurseIds,
    // Функции
    loadCurseKeyStatus,
    loadCurseCategories,
    searchCurse,
    installCurseCore,
    installCurse,
    // Детальный просмотр
    curseDetail,
    curseDetailTab,
    curseVersions,
    openCurseDetail,
    closeCurseDetail,
    // Константы
    CURSE_CLASS,
    CURSE_FOLDER,
    CURSE_SORT,
  };
}
