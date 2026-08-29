import { ref, computed, reactive } from "vue";
import { useI18n } from "~/composables/useI18n";
import {
  isTauri,
  modrinthSearch,
  modrinthProjectVersions,
  modrinthProject,
  modrinthInstallMod,
  modrinthUpdateMod,
  modrinthCheckUpdates,
  modrinthTags as fetchModrinthTags,
  curseforgeKeyConfigured,
} from "~/lib/bridge";
import type { GameFolderKind, ModrinthInstallFolder, ModrinthSearchKind } from "~/lib/bridge";
import type { ModrinthProject, ModrinthVersion, ModrinthTags, ModUpdate, GameFileEntry, AppStatus, TrackedMod } from "~/lib/types";
import { verCmp, cap } from "~/lib/misc";

export interface UseModSearchDeps {
  packId: { value: string | null };
  status: { value: AppStatus | null };
  playSubTab: { value: string };
  gameFiles: { value: Partial<Record<GameFolderKind, GameFileEntry[]>> };
  notify: (text: string, type?: string) => void;
  loadGameFiles: (folder: string, force?: boolean) => Promise<void>;
  /** Загрузка CurseForge-категорий (из useCurseSearch). */
  loadCurseCategories: () => Promise<void>;
  /** Поиск CurseForge (из useCurseSearch). */
  searchCurse: () => Promise<void>;
}

export interface SearchFilterState {
  versions: string[];
  loaders: string[];
  categories: string[];
  versionType: string;
  sort: string;
  env: string;
}

/** Папка игры для типа проекта Modrinth. */
export const MOD_KIND_FOLDER: Record<ModrinthSearchKind, ModrinthInstallFolder> = {
  mod: "mods",
  modpack: "mods",
  resourcepack: "resourcepacks",
  shaderpack: "shaderpacks",
  datapack: "datapacks",
};

/* Серверные платформы/плагины Modrinth (paper/velocity/spigot/... отдаются как загрузчики
 * и категории для типа "mod"). Это серверная, а не клиентская сторона — убираем из фильтров модов. */
export const SERVER_PLATFORMS = new Set([
  "spigot",
  "paper",
  "purpur",
  "folia",
  "bukkit",
  "velocity",
  "waterfall",
  "bungeecord",
  "sponge",
  "geyser",
]);

export function useModSearch(deps: UseModSearchDeps) {
  const { t } = useI18n();
  const { packId, status, playSubTab, gameFiles, notify, loadGameFiles, loadCurseCategories, searchCurse } = deps;

  // ---- Поиск модов: Modrinth ----
  const modSearchKind = ref<ModrinthSearchKind>("mod");
  const modSearchQuery = ref("");
  const modSearchLoading = ref(false);
  const modSearchResults = ref<ModrinthProject[]>([]);
  const modSearchErr = ref("");
  const modSearchOffset = ref(0);
  const modSearchMore = ref(false);
  const modSearchMoreBusy = ref(false);
  const modSearchBox = ref<HTMLElement | null>(null);
  let modSearchGen = 0;
  const MOD_SEARCH_PAGE = 20;

  const modVersions = ref<ModrinthVersion[] | null>(null);
  const modVersionsRaw = ref<ModrinthVersion[]>([]);

  // ---- Версионные фильтры (окно версий мода) ----
  const verFilterMc = ref<string[]>([]);
  const verFilterLoader = ref<string[]>([]);
  const verFilterType = ref<string[]>([]);

  const verFilterMcOptions = computed(() =>
    Array.from(new Set(modVersionsRaw.value.flatMap((v) => v.gameVersions)))
      .sort((a, b) => verCmp(b, a))
      .map((v) => ({ value: v, label: v }))
  );
  const verFilterLoaderOptions = computed(() =>
    Array.from(new Set(modVersionsRaw.value.flatMap((v) => v.loaders)))
      .map((v) => ({ value: v, label: cap(v) }))
      .sort((a, b) => a.label.localeCompare(b.label))
  );
  const verFilterTypeOptions = computed(() =>
    Array.from(new Set(modVersionsRaw.value.map((v) => v.versionType)))
      .sort((a, b) => a.localeCompare(b))
      .map((v) => ({ value: v, label: t("mods.verType." + v) }))
  );
  const filteredModVersions = computed(() =>
    modVersionsRaw.value.filter((v) => {
      if (verFilterMc.value.length > 0 && !verFilterMc.value.some((mc) => v.gameVersions.includes(mc))) return false;
      if (verFilterLoader.value.length > 0 && !verFilterLoader.value.some((l) => v.loaders.includes(l))) return false;
      if (verFilterType.value.length > 0 && !verFilterType.value.includes(v.versionType)) return false;
      return true;
    })
  );
  const verFilterMcSel = computed({
    get: () => verFilterMc.value,
    set: (v: string[]) => { verFilterMc.value = v; },
  });
  const verFilterLoaderSel = computed({
    get: () => verFilterLoader.value,
    set: (v: string[]) => { verFilterLoader.value = v; },
  });
  const verFilterTypeSel = computed({
    get: () => verFilterType.value,
    set: (v: string[]) => { verFilterType.value = v; },
  });
  const verTypeColor = (ty: string) =>
    ty === "beta" ? "#f59e0b" : ty === "alpha" ? "#ef4444" : "#22c55e";
  const verInstallSize = (v: ModrinthVersion) =>
    v.files.reduce((m, f) => Math.max(m, f.size ?? 0), 0);

  // ---- Статус установки ----
  const modInstallBusy = ref<string | null>(null);
  const quickModBusy = ref<string | null>(null);

  // ---- Обновления модов ----
  const modUpdates = ref<ModUpdate[]>([]);
  const trackedMods = ref<TrackedMod[]>([]);
  const updatingMod = ref<string | null>(null);
  const updateAllBusy = ref(false);
  const updatesCheckedAt = ref(0);
  const UPDATES_TTL_MS = 5 * 60 * 1000;

  /** Обновления только текущей вкладки (моды / ресурспаки / шейдеры). */
  const modUpdatesTab = computed(() =>
    playSubTab.value === "mods" ||
    playSubTab.value === "resourcepacks" ||
    playSubTab.value === "shaderpacks"
      ? modUpdates.value.filter((u) => u.folder === playSubTab.value)
      : []
  );

  /** Индекс обновлений по имени файла (O(1) вместо линейного поиска на строку). */
  const updatesByFile = computed(() => {
    const map = new Map<string, ModUpdate>();
    for (const u of modUpdates.value) map.set(u.fileName, u);
    return map;
  });

  function modUpdateFor(f: GameFileEntry): ModUpdate | undefined {
    return updatesByFile.value.get(f.name);
  }

  /** Проверяет обновления установленных из Modrinth модов (с кешем на 5 минут). */
  async function refreshModUpdates(force = false) {
    if (!isTauri() || !packId.value || !status.value?.installed) {
      modUpdates.value = [];
      trackedMods.value = [];
      return;
    }
    if (!force && updatesCheckedAt.value && Date.now() - updatesCheckedAt.value < UPDATES_TTL_MS) return;
    try {
      modUpdates.value = await modrinthCheckUpdates(packId.value);
      updatesCheckedAt.value = Date.now();
    } catch {
      modUpdates.value = [];
    }
  }

  /** Обновляет один мод (папку берём из записи обновления). */
  async function updateOneMod(u: ModUpdate) {
    if (!packId.value || updatingMod.value) return;
    updatingMod.value = u.fileName;
    try {
      await modrinthUpdateMod(packId.value, u.fileName);
      notify(t("mods.updated", { kind: kindNoun(u.folder as ModrinthInstallFolder), name: u.newVersion.name }), "success");
      await loadGameFiles(u.folder === "datapacks" ? "saves" : (u.folder as GameFolderKind), true);
      await refreshModUpdates(true);
    } catch (e) {
      notify(t("mods.updateErr", { kind: kindNoun(u.folder as ModrinthInstallFolder), e }));
    } finally {
      updatingMod.value = null;
    }
  }

  /** Обновляет все моды текущей вкладки (последовательно). */
  async function updateAllMods() {
    if (!packId.value || updateAllBusy.value || modUpdatesTab.value.length === 0) return;
    updateAllBusy.value = true;
    let ok = 0;
    let fail = 0;
    for (const u of [...modUpdatesTab.value]) {
      try {
        await modrinthUpdateMod(packId.value, u.fileName);
        ok++;
      } catch {
        fail++;
      }
    }
    notify(
      ok > 0
        ? t("mods.updatedCount", { ok, fail })
        : t("mods.updateAllFail", { fail }),
      fail > 0 && ok === 0 ? "error" : "success"
    );
    const tabs: GameFolderKind[] = ["mods", "resourcepacks", "shaderpacks", "saves"];
    if ((tabs as string[]).includes(playSubTab.value)) {
      await loadGameFiles(playSubTab.value as GameFolderKind, true);
    }
    await refreshModUpdates(true);
    updateAllBusy.value = false;
  }

  // ---- Деталь мода ----
  const modDetail = ref<ModrinthProject | null>(null);
  const modDetailTab = ref<"about" | "versions" | "gallery">("about");
  const modDetailTabs: { kind: "about" | "versions" | "gallery" }[] = [
    { kind: "about" },
    { kind: "versions" },
    { kind: "gallery" },
  ];

  // ---- Мир датапаков ----
  const modDatapackWorld = ref<string | null>(null);
  const datapackWorlds = computed(() => (gameFiles.value.saves ?? []).filter((s) => s.kind === "dir").map((s) => s.name));
  const worldOptions = computed(() => datapackWorlds.value.map((w) => ({ value: w, label: w })));
  const modDatapackWorldSel = computed({
    get: () => (modDatapackWorld.value ? [modDatapackWorld.value] : []),
    set: (v: string[]) => {
      modDatapackWorld.value = v[0] ?? null;
    },
  });

  // ---- Фильтры поиска Modrinth ----
  const modrinthTagsMap = ref<Record<string, ModrinthTags | null>>({});
  const modrinthTags = computed(() => modrinthTagsMap.value[modSearchKind.value] ?? null);
  const modFilters = reactive<SearchFilterState>({ versions: [], loaders: [], categories: [], versionType: "", sort: "relevance", env: "" });

  const versionOptions = computed(() =>
    [...(modrinthTags.value?.versions ?? [])].sort((a, b) => verCmp(b, a)).map((v) => ({ value: v, label: v }))
  );
  const loaderOptions = computed(() =>
    (modrinthTags.value?.loaders ?? [])
      .filter((l) => !SERVER_PLATFORMS.has(l))
      .filter((l) => !(modSearchKind.value === "mod" && l === "datapack"))
      .map((l) => ({ value: l, label: cap(l) }))
  );
  const autoFiltersDone = ref(false);
  /** Автофильтры при открытии поиска: версия Minecraft и загрузчик активной сборки.
   *  Загрузчик учитываем только для модов (ресурспаки/шейдеры часто только vanilla). */
  function applyAutoFilters() {
    const mc = status.value?.minecraft_version;
    const loader = status.value?.loader?.replace("-loader", "");
    if (mc && versionOptions.value.some((o) => o.value === mc)) {
      modFilters.versions = [mc];
    } else {
      modFilters.versions = [];
    }
    if (modSearchKind.value === "mod" && loader && loaderOptions.value.some((o) => o.value === loader)) {
      modFilters.loaders = [loader];
    } else {
      modFilters.loaders = [];
    }
  }

  const categoryOptions = computed(() =>
    (modrinthTags.value?.categories ?? [])
      .filter((c) => !(modSearchKind.value === "mod" && SERVER_PLATFORMS.has(c)))
      .map((c) => ({ value: c, label: cap(c) }))
  );
  const envOptions = [
    { value: "client", label: t("mods.fClient") },
    { value: "server", label: t("mods.fServer") },
  ];
  const sortOptions = [
    { id: "relevance", labelKey: "mods.sortRelevance" },
    { id: "downloads", labelKey: "mods.sortDownloads" },
    { id: "follows", labelKey: "mods.sortFollows" },
    { id: "newest", labelKey: "mods.sortNewest" },
    { id: "updated", labelKey: "mods.sortUpdated" },
  ];
  const sortSelectOptions = sortOptions.map((s) => ({ value: s.id, label: t(s.labelKey) }));
  const modEnvSel = computed({
    get: () => (modFilters.env ? [modFilters.env] : []),
    set: (v: string[]) => {
      modFilters.env = v[0] ?? "";
    },
  });
  const modVersionTypeSel = computed({
    get: () => (modFilters.versionType ? [modFilters.versionType] : []),
    set: (v: string[]) => { modFilters.versionType = v[0] ?? ""; },
  });
  const modSortSel = computed({
    get: () => [modFilters.sort],
    set: (v: string[]) => {
      modFilters.sort = v[0] ?? "relevance";
    },
  });

  /** Общие опции поиска из фильтров. */
  function searchOpts(f: SearchFilterState) {
    const opts: { categories?: string[]; loaders?: string[]; versionType?: string; versions?: string[]; environment?: string; index?: string } = {};
    if (f.categories.length) opts.categories = f.categories;
    if (f.loaders.length) opts.loaders = f.loaders;
    if (f.versionType) opts.versionType = f.versionType;
    if (f.versions.length) opts.versions = f.versions;
    if (f.env) opts.environment = f.env;
    if (f.sort && f.sort !== "relevance") opts.index = f.sort;
    return opts;
  }

  /** Загружает теги Modrinth для типа проекта (по одному разу за сессию). */
  async function loadModrinthTags(kind: ModrinthSearchKind = modSearchKind.value) {
    if (!isTauri() || modrinthTagsMap.value[kind]) return;
    try {
      modrinthTagsMap.value = { ...modrinthTagsMap.value, [kind]: await fetchModrinthTags(kind) };
    } catch {
      /* фильтры просто не появятся */
    }
  }

  /** Активны ли фильтры/запрос поиска (для показа кнопки «сбросить» в пустом списке). */
  const modFiltersActive = computed(() => {
    const f = modFilters;
    return (
      !!modSearchQuery.value.trim() ||
      f.categories.length > 0 ||
      f.loaders.length > 0 ||
      f.versions.length > 0 ||
      !!f.versionType ||
      f.sort !== "relevance" ||
      !!f.env
    );
  });

  /** Сбрасывает фильтры и запрос поиска Modrinth и запускает поиск заново. */
  function resetModFiltersAndSearch() {
    modFilters.versions = [];
    modFilters.loaders = [];
    modFilters.categories = [];
    modFilters.env = "";
    modFilters.sort = "relevance";
    modFilters.versionType = "";
    modSearchQuery.value = "";
    modSearchErr.value = "";
    void searchMods();
  }

  /** Поиск модов/ресурспаков/шейдеров/датапаков для добавления в сборку. */
  async function searchMods() {
    if (!isTauri() || !packId.value) return;
    modDetail.value = null;
    const gen = ++modSearchGen;
    modSearchLoading.value = true;
    modSearchErr.value = "";
    modSearchMore.value = false;
    try {
      modSearchResults.value = await modrinthSearch(
        modSearchQuery.value.trim(),
        modSearchKind.value,
        MOD_SEARCH_PAGE,
        searchOpts(modFilters),
        0
      );
      if (gen !== modSearchGen) return;
      modSearchOffset.value = modSearchResults.value.length;
      modSearchMore.value = modSearchResults.value.length >= MOD_SEARCH_PAGE;
      modSearchBox.value?.scrollTo({ top: 0 });
    } catch (e) {
      if (gen === modSearchGen) modSearchErr.value = String(e);
    } finally {
      if (gen === modSearchGen) modSearchLoading.value = false;
    }
  }

  /** Догрузка следующей страницы результатов (бесконечный поиск). */
  async function loadMoreMods() {
    if (modSearchMoreBusy.value || modSearchLoading.value || !modSearchMore.value) return;
    const gen = modSearchGen;
    modSearchMoreBusy.value = true;
    try {
      const page = await modrinthSearch(
        modSearchQuery.value.trim(),
        modSearchKind.value,
        MOD_SEARCH_PAGE,
        searchOpts(modFilters),
        modSearchOffset.value
      );
      if (gen !== modSearchGen) return;
      if (page.length) modSearchResults.value.push(...page);
      modSearchOffset.value += page.length;
      modSearchMore.value = page.length >= MOD_SEARCH_PAGE;
    } catch {
      /* не критично: пользователь увидит кнопку «Ещё» и сможет повторить */
    } finally {
      modSearchMoreBusy.value = false;
    }
  }

  function onModSearchScroll() {
    const el = modSearchBox.value;
    if (!el || modSearchMoreBusy.value || !modSearchMore.value) return;
    if (el.scrollTop + el.clientHeight >= el.scrollHeight - 160) loadMoreMods();
  }

  /** Версии мода: сперва подходящие под версию сборки, остальные ниже.
   *  По загрузчику фильтруем только моды — ресурспаки/шейдеры/датапаки
   *  часто поддержаны только на vanilla, даже в fabric-сборках. */
  async function openModVersions(p: ModrinthProject) {
    modVersions.value = null;
    verFilterMc.value = [];
    verFilterLoader.value = [];
    verFilterType.value = [];
    try {
      const all = await modrinthProjectVersions(p.projectId);
      modVersionsRaw.value = all;
      modVersions.value = all;
      const mc = status.value?.minecraft_version;
      const loader = status.value?.loader?.replace("-loader", "");
      verFilterMc.value = mc ? [mc] : [];
      verFilterLoader.value = loader ? [loader] : [];
    } catch (e) {
      modSearchErr.value = String(e);
      modVersions.value = [];
      modVersionsRaw.value = [];
    }
  }

  /** Открывает «страницу» ресурса: вкладки описание/версии/галерея (как в сборках). */
  async function openModDetail(p: ModrinthProject) {
    modDetail.value = p;
    modDetailTab.value = "about";
    modVersions.value = null;
    openModVersions(p);
    if (!p.body) {
      try {
        modDetail.value = await modrinthProject(p.projectId);
      } catch {
        /* остаётся карточка из поиска */
      }
    }
  }

  /** Устанавливает выбранную версию в папку активной сборки
   *  (датапаки — в saves/<мир>/datapacks). */
  async function installModVersion(v: ModrinthVersion, closeAfter = true) {
    if (modInstallBusy.value) return;
    if (!packId.value) {
      notify(t("mods.noPack"), "error");
      return;
    }
    const folder = MOD_KIND_FOLDER[modSearchKind.value];
    const world = modSearchKind.value === "datapack" ? (modDatapackWorld.value ?? undefined) : undefined;
    if (modSearchKind.value === "datapack" && !world) {
      notify(t("mods.pickWorld"), "info");
      return false;
    }
    modInstallBusy.value = v.id;
    try {
      await modrinthInstallMod(packId.value, v.id, folder, world);
      notify(t("mods.installed", { kind: kindNoun(modSearchKind.value), name: v.name }), "success");
      modVersions.value = null;
      if (folder !== "datapacks") {
        await loadGameFiles(folder, true);
      } else {
        await loadGameFiles("saves", true);
      }
      await refreshModUpdates(true);
      return true;
    } catch (e) {
      notify(t("mods.installErr", { kind: kindNoun(modSearchKind.value), e }));
      return false;
    } finally {
      modInstallBusy.value = null;
    }
  }

  /** Подбирает версию проекта под версию Minecraft и загрузчик сборки
   *  (загрузчик учитываем только для модов). */
  async function pickModVersion(p: ModrinthProject): Promise<ModrinthVersion | null> {
    const isMod = modSearchKind.value === "mod";
    const mc = status.value?.minecraft_version || undefined;
    // Без активной версии Minecraft нельзя выбрать «самую свежую» версию мода —
    // это приведёт к установке файла под более новую (или иную) версию игры.
    if (!mc) return null;
    const loader = status.value?.loader?.replace("-loader", "");
    // Версии под текущую версию Minecraft (и загрузчик для модов) фильтруем на
    // сервере — так не получится скачать файл под другую версию игры.
    const reqLoader = isMod && loader ? loader : undefined;
    let all = await modrinthProjectVersions(p.projectId, mc, reqLoader);
    if (all.length === 0 && mc) {
      // Под текущую MC нет версии с нужным загрузчиком — пробуем любой загрузчик.
      all = await modrinthProjectVersions(p.projectId, mc, undefined);
    }
    if (all.length === 0) return null;
    // project_versions возвращает от новых к старым — берём самую свежую под MC.
    return all[0];
  }

  /** Быстрое скачивание мода: последняя версия под MC и загрузчик сборки. */
  async function quickDownloadMod(p: ModrinthProject, ev: Event) {
    ev.stopPropagation();
    if (quickModBusy.value) return;
    if (!packId.value) {
      notify(t("mods.noPack"), "error");
      return;
    }
    quickModBusy.value = p.projectId;
    try {
      const pick = await pickModVersion(p);
      if (!pick) {
        notify(t("mods.noMatchVersion"), "info");
        return;
      }
      await installModVersion(pick);
    } catch (e) {
      notify(t("mods.installErr", { kind: kindNoun(modSearchKind.value), e }));
    } finally {
      quickModBusy.value = null;
    }
  }

  // ---- Мультивыбор в поиске: скачивание сразу нескольких ресурсов ----
  const selModrinth = ref<Set<string>>(new Set());
  const multiSelBusy = ref(false);

  function toggleModrinthSel(id: string) {
    const s = new Set(selModrinth.value);
    if (s.has(id)) s.delete(id);
    else s.add(id);
    selModrinth.value = s;
  }

  function clearSelAll() {
    selModrinth.value = new Set();
  }

  /** Скачивает все выделенные проекты Modrinth подряд (последние подходящие версии). */
  async function downloadSelectedMods(closeSearch: () => void) {
    if (!isTauri() || !packId.value || multiSelBusy.value) return;
    if (modSearchKind.value === "datapack" && !modDatapackWorld.value) {
      notify(t("mods.pickWorld"), "info");
      return;
    }
    const ids = [...selModrinth.value];
    if (ids.length === 0) return;
    multiSelBusy.value = true;
    let ok = 0;
    for (const id of ids) {
      const p = modSearchResults.value.find((r) => r.projectId === id);
      if (!p) continue;
      try {
        const pick = await pickModVersion(p);
        if (!pick) continue;
        if (await installModVersion(pick, false)) ok++;
      } catch (e) {
        notify(t("mods.installErr", { kind: kindNoun(modSearchKind.value), e }));
      }
    }
    multiSelBusy.value = false;
    clearSelAll();
    closeSearch();
    if (ok > 0) notify(t("mods.installedSel", { n: ok }), "success");
  }

  // ---- CurseForge: ключ API (нужен для runInitialSearch) ----
  const curseKeyOk = ref(true);

  async function loadCurseKeyStatus() {
    if (!isTauri()) return;
    try {
      curseKeyOk.value = await curseforgeKeyConfigured();
    } catch {
      curseKeyOk.value = false;
    }
  }

  /** Запускает первичный поиск при открытии окна: подгружает фильтры/теги,
   *  проставляет автофильтры и наполняет список, чтобы не ждать Enter. */
  async function runInitialSearch(service: string) {
    if (!isTauri() || !packId.value) return;
    if (service === "modrinth") {
      await loadModrinthTags(modSearchKind.value);
      applyAutoFilters();
      await searchMods();
    } else {
      // Теги Modrinth нужны как источник списка версий Minecraft для CF-фильтра.
      await loadModrinthTags(modSearchKind.value);
      await loadCurseKeyStatus();
      await loadCurseCategories();
      if (!curseKeyOk.value) return;
      await searchCurse();
    }
  }

  /** Имя типа проекта для сообщений («мод»/«ресурспак»/«шейдер»/«датапак»); по kind или папке. */
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

  return {
    // Modrinth search state
    modSearchKind,
    modSearchQuery,
    modSearchLoading,
    modSearchResults,
    modSearchErr,
    modSearchOffset,
    modSearchMore,
    modSearchMoreBusy,
    modSearchBox,
    MOD_SEARCH_PAGE,
    modVersions,
    modVersionsRaw,
    // Version filter state
    verFilterMc,
    verFilterLoader,
    verFilterType,
    verFilterMcOptions,
    verFilterLoaderOptions,
    verFilterTypeOptions,
    filteredModVersions,
    verFilterMcSel,
    verFilterLoaderSel,
    verFilterTypeSel,
    verTypeColor,
    verInstallSize,
    // Install status
    modInstallBusy,
    quickModBusy,
    // Mod updates
    modUpdates,
    modUpdatesTab,
    trackedMods,
    updatingMod,
    updateAllBusy,
    refreshModUpdates,
    updateOneMod,
    updateAllMods,
    updatesByFile,
    modUpdateFor,
    // Mod detail
    modDetail,
    modDetailTab,
    modDetailTabs,
    // Datapack world
    modDatapackWorld,
    worldOptions,
    modDatapackWorldSel,
    datapackWorlds,
    // Filter state
    modFilters,
    modrinthTagsMap,
    modrinthTags,
    versionOptions,
    loaderOptions,
    categoryOptions,
    envOptions,
    sortOptions,
    sortSelectOptions,
    modEnvSel,
    modVersionTypeSel,
    modSortSel,
    modFiltersActive,
    autoFiltersDone,
    // Functions
    searchMods,
    loadMoreMods,
    onModSearchScroll,
    openModVersions,
    openModDetail,
    installModVersion,
    pickModVersion,
    quickDownloadMod,
    loadModrinthTags,
    searchOpts,
    applyAutoFilters,
    resetModFiltersAndSearch,
    runInitialSearch,
    kindNoun,
    // Multi-select
    selModrinth,
    toggleModrinthSel,
    clearSelAll,
    downloadSelectedMods,
    multiSelBusy,
    // CurseForge key (needed by runInitialSearch)
    curseKeyOk,
    loadCurseKeyStatus,
  };
}
