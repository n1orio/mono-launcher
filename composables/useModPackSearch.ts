import { computed, nextTick, reactive, ref } from "vue";
import { isTauri, modrinthSearch, modrinthProject, modrinthProjectVersions, modrinthInstallPack, modrinthTags as fetchModrinthTags, curseforgeSearch, curseforgeCategories, curseforgeModpackFiles, curseforgeInstallPack, curseforgeProjectDetail } from "~/lib/bridge";
import type { ModrinthSearchKind } from "~/lib/bridge";
import type { ModrinthProject, ModrinthTags, ModrinthVersion, CurseSearchHit, CursePackFile, CurseProjectDetail, AppStatus } from "~/lib/types";
import type { Ref } from "vue";
import { verCmp, cap } from "~/lib/misc";

export interface UseModPackSearchDeps {
  packId: Ref<string | null>;
  status: Ref<AppStatus | null>;
  modSearchKind: Ref<ModrinthSearchKind>;
  t: (key: string, params?: Record<string, unknown>) => string;
  notify: (text: string, type?: string) => void;
  loadPacks: () => Promise<void>;
  openPackTab: (id: string) => Promise<void>;
}

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

interface SearchFilterState {
  versions: string[];
  loaders: string[];
  categories: string[];
  versionType: string;
  sort: string;
  env: string;
}

export function useModPackSearch(deps: UseModPackSearchDeps) {
  const { status, modSearchKind, t, notify, loadPacks, openPackTab } = deps;

  // ---- Состояние модалки поиска сборок ----
  const modPackOpen = ref(false);
  const modPackService = ref<"modrinth" | "curseforge">("modrinth");
  const modPackQuery = ref("");
  const modPackLoading = ref(false);
  const modPackResults = ref<ModrinthProject[]>([]);
  const modPackVersions = ref<ModrinthVersion[] | null>(null);
  const modPackInstalling = ref<string | null>(null);
  const modPackDetail = ref<ModrinthProject | null>(null);
  const modPackTab = ref<"about" | "versions" | "gallery">("about");

  // ---- CurseForge сборки (отдельное состояние от поиска файлов) ----
  const cpSearched = ref(false);
  const cpLoading = ref(false);
  const cpErr = ref("");
  const cpResults = ref<CurseSearchHit[]>([]);
  const cpProject = ref<CurseSearchHit | null>(null);
  const cpFiles = ref<CursePackFile[] | null>(null);
  const cpBusy = ref<number | null>(null);
  const cpDetail = ref<CurseProjectDetail | null>(null);
  const cpDetailLoading = ref(false);
  const cpTab = ref<"about" | "versions" | "screenshots">("about");
  const cpTabs: ("about" | "versions" | "screenshots")[] = ["about", "versions", "screenshots"];
  const cpTabBusy = ref(false);
  /** Ссылка на сайт сборки для кнопки «Открыть страницу». */
  const cpWebsiteUrl = ref("");

  // ---- Фильтры поиска сборок на CurseForge (категория/версия/сортировка) ----
  const cpCatOptions = ref<{ value: string; label: string }[]>([]);
  const cpCatIds = ref<number[]>([]);
  const cpCatSel = computed({
    get: () => cpCatIds.value.map(String),
    set: (v: string[]) => {
      cpCatIds.value = v.map(Number);
    },
  });
  const cpVersion = ref("");
  const cpVerSel = computed({
    get: () => (cpVersion.value ? [cpVersion.value] : []),
    set: (v: string[]) => {
      cpVersion.value = v[0] ?? "";
    },
  });
  const cpSortField = ref("2");
  const cpSortSel = computed({
    get: () => [cpSortField.value],
    set: (v: string[]) => {
      cpSortField.value = v[0] ?? "2";
    },
  });

  async function loadCpCategories() {
    if (!isTauri()) return;
    cpCatOptions.value = [];
    cpCatIds.value = [];
    try {
      const cats = await curseforgeCategories(4471);
      cpCatOptions.value = cats.map((c) => ({ value: String(c.id), label: c.name }));
    } catch {
      /* фильтр просто не появится */
    }
  }

  // ---- Фильтры поиска Modrinth (теги грузятся по типам проектов) ----
  const modrinthTagsMap = ref<Record<string, ModrinthTags | null>>({});
  const packFilters = reactive<SearchFilterState>({ versions: [], loaders: [], categories: [], versionType: "", sort: "relevance", env: "" });

  const packVersionTypeSel = computed({
    get: () => (packFilters.versionType ? [packFilters.versionType] : []),
    set: (v: string[]) => { packFilters.versionType = v[0] ?? ""; },
  });

  /** Теги модпаков (для фильтров окна поиска сборок). */
  const packTags = computed(() => modrinthTagsMap.value["modpack"] ?? null);
  const packVersionOptions = computed(() =>
    [...(packTags.value?.versions ?? [])].sort((a, b) => verCmp(b, a)).map((v) => ({ value: v, label: v }))
  );
  const packLoaderOptions = computed(() =>
    (packTags.value?.loaders ?? []).map((l) => ({ value: l, label: cap(l) }))
  );
  const packCategoryOptions = computed(() =>
    (packTags.value?.categories ?? []).map((c) => ({ value: c, label: cap(c) }))
  );
  const packEnvSel = computed({
    get: () => (packFilters.env ? [packFilters.env] : []),
    set: (v: string[]) => {
      packFilters.env = v[0] ?? "";
    },
  });
  const packSortSel = computed({
    get: () => [packFilters.sort],
    set: (v: string[]) => {
      packFilters.sort = v[0] ?? "relevance";
    },
  });

  function applyPackAutoFilters() {
    const mc = status.value?.minecraft_version;
    if (mc && packVersionOptions.value.some((o) => o.value === mc)) {
      packFilters.versions = [mc];
      cpVersion.value = mc;
    } else {
      packFilters.versions = [];
      cpVersion.value = "";
    }
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

  /** Открывает модалку скачивания сборки (Modrinth по умолчанию, либо CurseForge).
   *  Сразу грузит теги, проставляет автофильтры и запускает поиск — чтобы не ждать Enter. */
  async function openModPackModal(service: "modrinth" | "curseforge" = "modrinth") {
    modPackOpen.value = true;
    modPackQuery.value = "";
    modPackService.value = service;
    modPackVersions.value = null;
    modPackDetail.value = null;
    modPackTab.value = "about";
    cpProject.value = null;
    cpFiles.value = null;
    cpDetail.value = null;
    cpErr.value = "";
    cpSearched.value = false;
    await loadModrinthTags("modpack");
    applyPackAutoFilters();
    if (service === "modrinth") {
      await searchPacks();
    } else {
      await loadCpCategories();
      await searchCursePacks();
    }
  }

  function switchPackService(s: "modrinth" | "curseforge") {
    if (s === modPackService.value) return;
    modPackService.value = s;
    modPackDetail.value = null;
    modPackVersions.value = null;
    cpProject.value = null;
    cpFiles.value = null;
    cpDetail.value = null;
    cpErr.value = "";
    cpVersion.value = "";
    if (s === "curseforge") {
      void loadCpCategories();
      void searchCursePacks();
    } else {
      void searchPacks();
    }
  }

  function searchPacksOrCurse() {
    if (modPackService.value === "modrinth") void searchPacks();
    else void searchCursePacks();
  }

  /** Поиск сборок на CurseForge (класс modpacks). */
  async function searchCursePacks() {
    if (!isTauri() || cpLoading.value) return;
    cpLoading.value = true;
    cpSearched.value = true;
    cpErr.value = "";
    cpProject.value = null;
    cpFiles.value = null;
    cpDetail.value = null;
    try {
      cpResults.value = await curseforgeSearch(
        modPackQuery.value.trim(),
        4471,
        cpCatIds.value,
        cpVersion.value || undefined,
        cpSortField.value
      );
    } catch (e) {
      cpResults.value = [];
      cpErr.value = String(e);
    } finally {
      cpLoading.value = false;
    }
  }

  /** Файлы сборки CurseForge (выбор версии). */
  async function openCpFiles(p: CurseSearchHit) {
    cpProject.value = p;
    cpFiles.value = null;
    cpErr.value = "";
    cpTab.value = "about";
    // Деталка (описание/скриншоты/категории) подгружается независимо и не блокирует список файлов.
    void loadCpDetail(p.projectId);
    try {
      cpFiles.value = await curseforgeModpackFiles(p.projectId);
    } catch (e) {
      cpErr.value = String(e);
      cpFiles.value = [];
    }
  }

  /** Загружает полное описание проекта CurseForge (описание/скриншоты/категории). */
  async function loadCpDetail(projectId: number) {
    cpDetailLoading.value = true;
    cpDetail.value = null;
    cpWebsiteUrl.value = "";
    try {
      const d = await curseforgeProjectDetail(projectId);
      cpDetail.value = d;
      cpWebsiteUrl.value = d.websiteUrl;
    } catch (e) {
      cpDetail.value = null;
      notify(t("err.curseDetail", { e }));
    } finally {
      cpDetailLoading.value = false;
    }
  }

  /** Скачивает и устанавливает сборку CurseForge как отдельную сборку. */
  async function installCpPack(f: CursePackFile) {
    if (!cpProject.value || cpBusy.value !== null) return;
    cpBusy.value = f.fileId;
    try {
      const pack = await curseforgeInstallPack(cpProject.value.projectId, f.fileId);
      notify(t("mods.packInstalled", { name: pack.name }), "success");
      modPackOpen.value = false;
      cpProject.value = null;
      cpFiles.value = null;
      cpDetail.value = null;
      await loadPacks();
      await nextTick();
      openPackTab(pack.id);
    } catch (e) {
      notify(t("mods.packInstallErr", { e }), "error");
    } finally {
      cpBusy.value = null;
    }
  }

  // ---- Modrinth: поиск сборок ----

  /** Поиск модпаков на Modrinth для установки как сборки. */
  async function searchPacks() {
    if (!isTauri()) return;
    modPackLoading.value = true;
    modPackDetail.value = null;
    modPackVersions.value = null;
    try {
      modPackResults.value = await modrinthSearch(
        modPackQuery.value.trim(),
        "modpack",
        20,
        searchOpts(packFilters)
      );
    } catch (e) {
      notify(t("mods.packsSearchErr", { e }));
    } finally {
      modPackLoading.value = false;
    }
  }

  /** Версии модпака (по убыванию даты). */
  async function openPackVersions(p: ModrinthProject) {
    modPackVersions.value = null;
    try {
      const all = await modrinthProjectVersions(p.projectId);
      modPackVersions.value = all.sort(
        (a, b) => Date.parse(b.datePublished) - Date.parse(a.datePublished)
      );
    } catch (e) {
      notify(t("mods.packsSearchErr", { e }));
      modPackVersions.value = [];
    }
  }

  /** Открывает «страницу» сборки: вкладки описание/версии/галерея. */
  async function openPackDetail(p: ModrinthProject) {
    modPackDetail.value = p;
    modPackTab.value = "about";
    modPackVersions.value = null;
    openPackVersions(p);
    if (!p.body) {
      try {
        modPackDetail.value = await modrinthProject(p.projectId);
      } catch {
        /* оставляем карточку из поиска */
      }
    }
  }

  /** Скачивает и устанавливает модпак с Modrinth. */
  async function installPackVersion(v: ModrinthVersion) {
    if (modPackInstalling.value) return;
    modPackInstalling.value = v.id;
    try {
      const pack = await modrinthInstallPack(v.id);
      notify(t("mods.packInstalled", { name: pack.name }), "success");
      modPackOpen.value = false;
      modPackVersions.value = null;
      modPackDetail.value = null;
      await loadPacks();
      await nextTick();
      openPackTab(pack.id);
    } catch (e) {
      notify(t("mods.packInstallErr", { e }));
    } finally {
      modPackInstalling.value = null;
    }
  }

  // ---- Константы ----

  const modPackTabs: { kind: "about" | "versions" | "gallery"; icon: string }[] = [
    { kind: "about", icon: '<path d="M3.5 2.75A1.75 1.75 0 0 1 5.25 1h5.5c.966 0 1.75.784 1.75 1.75v10.5A1.75 1.75 0 0 1 10.75 15h-5.5a1.75 1.75 0 0 1-1.75-1.75V2.75ZM5.25 2.5a.25.25 0 0 0-.25.25v10.5c0 .138.112.25.25.25h5.5a.25.25 0 0 0 .25-.25V2.75a.25.25 0 0 0-.25-.25h-5.5ZM6.5 5.75a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 0 1.5h-1.5a.75.75 0 0 1-.75-.75Zm0 3a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 0 1.5h-1.5a.75.75 0 0 1-.75-.75Zm0 3a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 0 1.5h-1.5a.75.75 0 0 1-.75-.75Z"/>' },
    { kind: "versions", icon: '<path d="M2.22 3.305l5.25-2.625a1.75 1.75 0 0 1 1.56 0l5.25 2.625c.511.255.722.862.61 1.41L12.53 10.65c-.121.6-.416 1.154-.836 1.57l-3.117 3.09a.75.75 0 0 1-1.056 0l-3.117-3.09a3.25 3.25 0 0 1-.836-1.57L1.61 4.715a1.75 1.75 0 0 1 .61-1.41Zm7.78 2.195a1.75 1.75 0 0 1 .75-1.415l4.925-2.462L11.15 1.5h-6.3L1.075 1.623l4.925 2.462a1.75 1.75 0 0 1 .75 1.415v4.837c0 .034.001.068.004.102l3.647-1.462L10 5.5Z"/>' },
    { kind: "gallery", icon: '<path d="M1.75 1.75A1.75 1.75 0 0 0 0 3.5v9A1.75 1.75 0 0 0 1.75 14.25h12.5A1.75 1.75 0 0 0 16 12.5v-9a1.75 1.75 0 0 0-1.75-1.75H1.75ZM1.5 3.5a.25.25 0 0 1 .25-.25h12.5a.25.25 0 0 1 .25.25v9a.25.25 0 0 1-.25.25H1.75a.25.25 0 0 1-.25-.25v-9ZM2.5 12.25v-2.5h11v2.5h-11Zm.83-3.5h9.34a2.75 2.75 0 0 0-2.24-1.25h-4.86a2.75 2.75 0 0 0-2.24 1.25Zm.8-2a1.5 1.5 0 1 0-1.5-1.5 1.5 1.5 0 0 0 1.5 1.5Z"/>' },
  ];

  return {
    // Состояние модалки
    modPackOpen,
    modPackService,
    modPackQuery,
    modPackLoading,
    modPackResults,
    modPackVersions,
    modPackInstalling,
    modPackDetail,
    modPackTab,

    // CurseForge сборки
    cpSearched,
    cpLoading,
    cpErr,
    cpResults,
    cpProject,
    cpFiles,
    cpBusy,
    cpDetail,
    cpDetailLoading,
    cpTab,
    cpTabs,
    cpTabBusy,
    cpWebsiteUrl,
    cpCatOptions,
    cpCatIds,
    cpCatSel,
    cpVersion,
    cpVerSel,
    cpSortField,
    cpSortSel,

    // Modrinth pack фильтры
    modrinthTagsMap,
    packFilters,
    packVersionTypeSel,
    packTags,
    packVersionOptions,
    packLoaderOptions,
    packCategoryOptions,
    packEnvSel,
    packSortSel,

    // Функции
    openModPackModal,
    switchPackService,
    searchPacksOrCurse,
    searchCursePacks,
    openCpFiles,
    loadCpDetail,
    installCpPack,
    loadCpCategories,
    searchPacks,
    openPackVersions,
    openPackDetail,
    installPackVersion,
    applyPackAutoFilters,
    loadModrinthTags,

    // Константы
    modPackTabs,
  };
}
