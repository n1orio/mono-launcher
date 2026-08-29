import { ref, computed } from "vue";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { isTauri } from "~/lib/bridge";
import type { ModrinthSearchKind } from "~/lib/bridge";
import { useI18n } from "~/composables/useI18n";

export type SearchService = "modrinth" | "curseforge";

export interface UseSearchWindowDeps {
  packId: { value: string | null };
  modSearchKind: { value: ModrinthSearchKind };
  isSearchWin: { value: boolean };
  modSearchQuery: { value: string };
  modSearchResults: { value: any[] };
  modVersions: { value: any };
  modDetail: { value: any };
  modSearchErr: { value: string };
  modDatapackWorld: { value: string | null };
  autoFiltersDone: { value: boolean };
  modFilters: {
    versions: string[];
    loaders: string[];
    categories: string[];
    env: string;
    sort: string;
    versionType: string;
  };
  curseQuery: { value: string };
  curseResults: { value: any[] };
  curseSearched: { value: boolean };
  curseLoading: { value: boolean };
  curseErr: { value: string };
  modSearchLoading: { value: boolean };
  notify: (text: string, type?: string) => void;
  loadGameFiles: (folder: string, force?: boolean) => Promise<void>;
  runInitialSearch: () => Promise<void>;
  clearSelAll: () => void;
  loadModrinthTags: (kind?: ModrinthSearchKind) => Promise<void>;
  loadCurseKeyStatus: () => Promise<void>;
  loadCurseCategories: () => Promise<void>;
  curseKeyOk: { value: boolean };
  gameFiles: { value: Record<string, any[]> };
  searchMods: () => Promise<void>;
  searchCurse: () => Promise<void>;
}

export function useSearchWindow(deps: UseSearchWindowDeps) {
  const {
    packId,
    modSearchKind,
    isSearchWin,
    modSearchQuery,
    modSearchResults,
    modVersions,
    modDetail,
    modSearchErr,
    modDatapackWorld,
    autoFiltersDone,
    modFilters,
    curseQuery,
    curseResults,
    curseSearched,
    curseLoading,
    curseErr,
    modSearchLoading,
    notify,
    loadGameFiles,
    runInitialSearch,
    clearSelAll,
    loadModrinthTags,
    loadCurseKeyStatus,
    loadCurseCategories,
    curseKeyOk,
    gameFiles,
    searchMods,
    searchCurse,
  } = deps;

  const { t } = useI18n();

  // ---- Поиск файлов: Modrinth / CurseForge ----
  const searchService = ref<SearchService>("modrinth");
  const searchOpen = ref(false);
  const searchPos = ref<{ x: number | null; y: number | null }>({ x: null, y: null });
  const searchWinStyle = computed(() => ({
    right: searchPos.value.x === null ? "2rem" : undefined,
    bottom: searchPos.value.y === null ? "2rem" : undefined,
    left: searchPos.value.x === null ? undefined : `${searchPos.value.x}px`,
    top: searchPos.value.y === null ? undefined : `${searchPos.value.y}px`,
  }));
  let searchDrag: { dx: number; dy: number } | null = null;
  function dragSearchWin(e: PointerEvent) {
    if (isSearchWin.value) return;
    if ((e.target as HTMLElement).closest("button")) return;
    const x = searchPos.value.x ?? window.innerWidth - 768;
    const y = searchPos.value.y ?? window.innerHeight - 480;
    searchDrag = { dx: e.clientX - x, dy: e.clientY - y };
    window.addEventListener("pointermove", moveSearchWin);
    window.addEventListener("pointerup", endSearchDrag, { once: true });
  }
  function moveSearchWin(e: PointerEvent) {
    if (!searchDrag) return;
    searchPos.value = {
      x: Math.max(0, Math.min(window.innerWidth - 120, e.clientX - searchDrag.dx)),
      y: Math.max(0, Math.min(window.innerHeight - 64, e.clientY - searchDrag.dy)),
    };
  }
  function endSearchDrag() {
    searchDrag = null;
    window.removeEventListener("pointermove", moveSearchWin);
  }
  function closeSearch() {
    if (isSearchWin.value && isTauri()) {
      getCurrentWindow().close();
      return;
    }
    searchOpen.value = false;
    modVersions.value = null;
    modDetail.value = null;
    clearSelAll();
    window.removeEventListener("pointermove", moveSearchWin);
  }
  /** Открывает поиск файлов. В Tauri — настоящее отдельное окно, в браузере — плавающая панель. */
  async function openSearch(kind: ModrinthSearchKind, service: SearchService = "modrinth") {
    modSearchKind.value = kind;
    if (isTauri()) {
      if (!packId.value) return;
      const existing = await WebviewWindow.getByLabel("search");
      if (existing) {
        try {
          await existing.close();
        } catch {
          /* окно уже закрывается */
        }
      }
      try {
        const devBase = import.meta.env.DEV ? "http://localhost:1420/" : "";
        new WebviewWindow("search", {
          url: `${devBase}?win=search&kind=${kind}&service=${service}&packId=${encodeURIComponent(packId.value)}`,
          title: searchTitle.value,
          width: 760,
          height: 640,
          minWidth: 520,
          minHeight: 400,
          resizable: true,
          decorations: false,
        });
      } catch (e) {
        notify(t("mods.windowErr", { e }), "error");
      }
      return;
    }
    searchService.value = service;
    modSearchQuery.value = "";
    modSearchResults.value = [];
    modSearchErr.value = "";
    modVersions.value = null;
    modDetail.value = null;
    modFilters.versions = [];
    modFilters.loaders = [];
    modFilters.categories = [];
    modFilters.env = "";
    modFilters.sort = "relevance";
    modDatapackWorld.value = null;
    curseQuery.value = "";
    curseResults.value = [];
    curseSearched.value = false;
    curseErr.value = "";
    searchOpen.value = true;
    autoFiltersDone.value = false;
    if (kind === "datapack" && !gameFiles.value.saves) {
      void loadGameFiles("saves");
    }
    // Сразу подгружаем фильтры/теги и запускаем поиск, чтобы не ждать Enter.
    await runInitialSearch();
  }
  const searchInput = computed({
    get: () => (searchService.value === "modrinth" ? modSearchQuery.value : curseQuery.value),
    set: (v: string) => {
      if (searchService.value === "modrinth") modSearchQuery.value = v;
      else curseQuery.value = v;
    },
  });
  const searchLoading = computed(() =>
    searchService.value === "modrinth" ? modSearchLoading.value : curseLoading.value,
  );
  const searchTitle = computed(() => {
    const kind = modSearchKind.value;
    switch (kind) {
      case "mod":
        return t("mods.title");
      case "resourcepack":
        return t("mods.titleRP");
      case "shaderpack":
        return t("mods.titleShaders");
      case "datapack":
        return t("mods.titleDatapack");
      default:
        return t("mods.title");
    }
  });
  async function switchSearchService(s: SearchService) {
    if (s === searchService.value) return;
    if (s === "curseforge" && modSearchKind.value === "datapack") return;
    searchService.value = s;
    // Сразу грузим фильтры/ключ и запускаем первичный поиск, чтобы не ждать Enter.
    await runInitialSearch();
  }
  function doSearch() {
    if (searchService.value === "modrinth") void searchMods();
    else void searchCurse();
  }

  return {
    searchService,
    searchOpen,
    searchPos,
    searchWinStyle,
    dragSearchWin,
    moveSearchWin,
    endSearchDrag,
    closeSearch,
    openSearch,
    searchInput,
    searchLoading,
    searchTitle,
    switchSearchService,
    doSearch,
  };
}
