import { computed, onMounted, reactive, ref, watch } from "vue";
import { getStatus, isTauri } from "~/lib/bridge";
import type { PackDescriptor, AppStatus } from "~/lib/types";
import { useI18n } from "~/composables/useI18n";

export interface UseLibraryDeps {
  packs: Ref<PackDescriptor[]>;
  notify: (text: string, type?: string) => void;
  selectPack: (id: string) => Promise<void>;
  tab: Ref<string>;
  packId: Ref<string>;
  status: Ref<AppStatus | null>;
  handlePlay: () => Promise<void>;
  handleInstall: () => Promise<void>;
}

export function useLibrary(deps: UseLibraryDeps) {
  const { packs, notify, selectPack, tab, packId, status, handlePlay, handleInstall } = deps;
  const { t } = useI18n();

  // ─── Категории сборок в сайдбаре ──────────────────────────────────────

  /** Категории сборок в сайдбаре: авторские (GitHub), свои, с Modrinth, с CurseForge. */
  type PackCat = "github" | "custom" | "modrinth" | "curseforge";
  const PACK_CATS: PackCat[] = ["github", "custom", "modrinth", "curseforge"];
  /** Ключи переводов для названий вкладок. */
  const PACK_CAT_LABELS: Record<PackCat, string> = {
    github: "side.catGitHub",
    custom: "side.catCustom",
    modrinth: "side.catModrinth",
    curseforge: "side.catCurse",
  };

  // ─── Сворачиваемые категории ───────────────────────────────────────────

  /** Сворачиваемые категории (состояние в localStorage; ключ = PackCat или id своей категории). */
  const SIDEBAR_CATS_KEY = "mono.sidebarCats";
  const sidebarCat = reactive<Record<string, boolean>>({
    github: true,
    custom: true,
    modrinth: true,
    curseforge: true,
  });
  {
    const saved =
      typeof localStorage !== "undefined"
        ? (JSON.parse(localStorage.getItem(SIDEBAR_CATS_KEY) || "{}") as Record<string, boolean>)
        : {};
    for (const k of Object.keys(saved)) if (typeof saved[k] === "boolean") sidebarCat[k] = saved[k];
  }
  function persistSidebarCat() {
    localStorage.setItem(SIDEBAR_CATS_KEY, JSON.stringify(sidebarCat));
  }
  function toggleSidebarCat(k: string) {
    sidebarCat[k] = !sidebarCat[k];
    persistSidebarCat();
  }

  // ─── Порядок вкладок категорий (drag & drop) ──────────────────────────

  /** Порядок вкладок категорий — меняется перетаскиванием (localStorage). */
  const PACK_TABS_KEY = "mono.packTabs";
  const packTabs = ref<PackCat[]>([...PACK_CATS]);
  {
    const saved: unknown =
      typeof localStorage !== "undefined"
        ? JSON.parse(localStorage.getItem(PACK_TABS_KEY) || "null")
        : null;
    if (Array.isArray(saved)) {
      const order = saved.filter((k): k is PackCat => PACK_CATS.includes(k as PackCat));
      for (const k of PACK_CATS) if (!order.includes(k)) order.push(k);
      packTabs.value = order;
    }
  }
  function persistPackTabs() {
    localStorage.setItem(PACK_TABS_KEY, JSON.stringify(packTabs.value));
  }

  /** Перетаскиваемая вкладка (для drag&drop перестановки). */
  const dragPackTab = ref<PackCat | null>(null);
  function packTabDragStart(e: DragEvent, cat: PackCat) {
    dragPackTab.value = cat;
    e.dataTransfer?.setData("text/plain", cat);
    if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
    e.dataTransfer?.setDragImage(e.currentTarget as Element, 12, 12);
  }
  function packTabDragOver(e: DragEvent) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
  }
  function packTabDrop(e: DragEvent, target: PackCat) {
    e.preventDefault();
    const cat = dragPackTab.value;
    dragPackTab.value = null;
    if (!cat || cat === target) return;
    const from = packTabs.value.indexOf(cat);
    const to = packTabs.value.indexOf(target);
    if (from < 0 || to < 0) return;
    packTabs.value.splice(from, 1);
    packTabs.value.splice(to, 0, cat);
    persistPackTabs();
  }
  function packTabDragEnd() {
    dragPackTab.value = null;
  }

  // ─── Разбивка сборок по источнику ──────────────────────────────────────

  /** Разбивка сборок по источнику: id mrn-* → Modrinth, cf-* → CurseForge,
   *  local-* / local:// → свои, остальные (встроенные и GitHub) → авторские. */
  type PacksBySource = Record<PackCat, PackDescriptor[]>;

  /** Платные сборки (с привязанным Boosty-блогом) — для панели Boosty в Настройках. */
  const paidPacks = computed<PackDescriptor[]>(() => packs.value.filter((p) => Boolean(p.boostyBlog)));

  const packsBySource = computed<PacksBySource>(() => {
    const out: PacksBySource = { github: [], custom: [], modrinth: [], curseforge: [] };
    for (const p of filteredPacks.value) {
      const group: PackCat = p.id.startsWith("mrn-")
        ? "modrinth"
        : p.id.startsWith("cf-")
          ? "curseforge"
          : p.id.startsWith("local-") || p.url.startsWith("local://")
            ? "custom"
            : "github";
      out[group].push(p);
    }
    return out;
  });

  // ─── Поиск по библиотеке ───────────────────────────────────────────────

  /** Поиск по библиотеке: фильтр плиток по названию/id. */
  const libQuery = ref("");
  const filteredPacks = computed<PackDescriptor[]>(() => {
    const q = libQuery.value.trim().toLowerCase();
    if (!q) return packs.value;
    return packs.value.filter(
      (p) => p.name.toLowerCase().includes(q) || p.id.toLowerCase().includes(q),
    );
  });

  // ─── Пользовательские категории библиотеки ─────────────────────────────

  /** Пользовательские категории библиотеки (создаются/переименовываются/удаляются).
   *  Привязки сборок хранятся отдельно: packId → [catId, ...]. */
  interface LibCat { id: string; name: string }
  const LIB_CATS_KEY = "mono.libCats";
  const LIB_PACK_CATS_KEY = "mono.libPackCats";
  function loadJson<T>(key: string, fallback: T): T {
    try {
      const raw = localStorage.getItem(key);
      return raw ? (JSON.parse(raw) as T) : fallback;
    } catch {
      return fallback;
    }
  }
  const libCats = ref<LibCat[]>(loadJson<LibCat[]>(LIB_CATS_KEY, []));
  const packLibCats = reactive<Record<string, string[]>>(loadJson<Record<string, string[]>>(LIB_PACK_CATS_KEY, {}));
  function saveLibCats() {
    localStorage.setItem(LIB_CATS_KEY, JSON.stringify(libCats.value));
  }
  function savePackLibCats() {
    localStorage.setItem(LIB_PACK_CATS_KEY, JSON.stringify(packLibCats));
  }
  function makeCatId(name: string): string {
    const slug = name.trim().toLowerCase().replace(/[^a-zа-яё0-9]+/gi, "-").replace(/^-+|-+$/g, "");
    let id = `c-${slug || "cat"}`;
    let n = 2;
    while (libCats.value.some((c) => c.id === id)) id = `c-${slug || "cat"}-${n++}`;
    return id;
  }
  function createLibCat(name: string): boolean {
    const n = name.trim();
    if (!n) return false;
    if (libCats.value.some((c) => c.name.toLowerCase() === n.toLowerCase())) return false;
    libCats.value.push({ id: makeCatId(n), name: n });
    saveLibCats();
    return true;
  }
  function renameLibCat(id: string, name: string): boolean {
    const n = name.trim();
    if (!n) return false;
    const cat = libCats.value.find((c) => c.id === id);
    if (!cat || libCats.value.some((c) => c.id !== id && c.name.toLowerCase() === n.toLowerCase())) return false;
    cat.name = n;
    saveLibCats();
    return true;
  }
  function deleteLibCat(id: string) {
    libCats.value = libCats.value.filter((c) => c.id !== id);
    for (const k of Object.keys(packLibCats)) {
      packLibCats[k] = (packLibCats[k] ?? []).filter((cid) => cid !== id);
      if (packLibCats[k].length === 0) delete packLibCats[k];
    }
    delete sidebarCat[id];
    saveLibCats();
    savePackLibCats();
    persistSidebarCat();
  }
  function packHasCat(packId: string, catId: string): boolean {
    return (packLibCats[packId] ?? []).includes(catId);
  }
  function togglePackCat(packId: string, catId: string) {
    const cur = packLibCats[packId] ?? [];
    packLibCats[packId] = cur.includes(catId) ? cur.filter((c) => c !== catId) : [...cur, catId];
    if (packLibCats[packId].length === 0) delete packLibCats[packId];
    savePackLibCats();
  }
  /** Секции пользовательских категорий: только непустые после фильтра поиска. */
  const customLibSections = computed(() =>
    libCats.value
      .map((c) => ({
        cat: c,
        packs: filteredPacks.value.filter((p) => packHasCat(p.id, c.id)),
      }))
      .filter((s) => s.packs.length > 0),
  );

  // ─── Модалка категории ─────────────────────────────────────────────────

  /** Модалка создания/переименования категории. */
  const libCatModal = ref<{ mode: "create" | "rename"; id?: string } | null>(null);
  const libCatName = ref("");
  function openCatCreate() {
    libCatName.value = "";
    libCatModal.value = { mode: "create" };
  }
  function openCatRename(id: string) {
    const cat = libCats.value.find((c) => c.id === id);
    if (!cat) return;
    libCatName.value = cat.name;
    libCatModal.value = { mode: "rename", id };
  }
  function submitCatModal() {
    const m = libCatModal.value;
    if (!m) return;
    const ok = m.mode === "create" ? createLibCat(libCatName.value) : m.id !== undefined && renameLibCat(m.id, libCatName.value);
    if (!ok) notify(t("library.catDuplicate"), "error");
    libCatModal.value = null;
  }

  // ─── Масштаб и статусы «Библиотеки» ───────────────────────────────────

  /** Масштаб плиток «Библиотеки» (1–4), сохраняется в localStorage. */
  const LIB_SCALE_KEY = "mono.libScale";
  function readLibScale(): number {
    const n = parseInt(localStorage.getItem(LIB_SCALE_KEY) ?? "", 10);
    return Number.isFinite(n) ? Math.min(4, Math.max(1, n)) : 2;
  }
  const libScale = ref(readLibScale());
  function setLibScale(n: number) {
    libScale.value = Math.min(4, Math.max(1, n));
    localStorage.setItem(LIB_SCALE_KEY, String(libScale.value));
  }
  const LIB_TILES: Record<number, { col: string; icon: string }> = {
    1: { col: "grid-cols-[repeat(auto-fill,minmax(92px,1fr))]", icon: "h-10 w-10" },
    2: { col: "grid-cols-[repeat(auto-fill,minmax(124px,1fr))]", icon: "h-14 w-14" },
    3: { col: "grid-cols-[repeat(auto-fill,minmax(164px,1fr))]", icon: "h-20 w-20" },
    4: { col: "grid-cols-[repeat(auto-fill,minmax(220px,1fr))]", icon: "h-28 w-28" },
  };
  const libTile = computed(() => LIB_TILES[libScale.value] ?? LIB_TILES[2]);
  const libPercent = computed(() => 75 + libScale.value * 25);

  /** Кэш статусов сборок для «Библиотеки» (установлена ли, версия). */
  const libStatus = reactive<Record<string, AppStatus | null>>({});
  let libStatusLoading = false;
  async function loadLibraryStatus() {
    if (!isTauri() || libStatusLoading) return;
    libStatusLoading = true;
    try {
      await Promise.all(
        packs.value.map(async (p) => {
          libStatus[p.id] = await getStatus(p.id).catch(() => null);
        }),
      );
    } finally {
      libStatusLoading = false;
    }
  }
  watch(
    () => tab.value,
    (t) => {
      if (t === "library") loadLibraryStatus();
    },
  );
  onMounted(loadLibraryStatus);

  /** Открыть вкладку сборки (выбор + переход на play). */
  async function openPackTab(id: string) {
    if (packId.value !== id) await selectPack(id);
    tab.value = "play";
  }

  /** Запуск (или установка) конкретной сборки из плитки «Библиотеки». */
  async function playLibraryPack(p: PackDescriptor) {
    if (!isTauri()) return;
    await openPackTab(p.id);
    if (status.value?.installed) await handlePlay();
    else await handleInstall();
  }

  // ─── Контекстное меню «Библиотеки» ────────────────────────────────────

  /** Контекстное меню «Библиотеки» (ПКМ по экземпляру). */
  const libMenuPack = ref<PackDescriptor | null>(null);
  const libMenuPos = ref<{ x: number; y: number } | null>(null);

  function openLibMenu(e: MouseEvent, p: PackDescriptor) {
    const menuW = 224;
    const menuH = 120;
    const x = Math.min(e.clientX, window.innerWidth - menuW - 8);
    const y = Math.min(e.clientY, window.innerHeight - menuH - 8);
    libMenuPack.value = p;
    libMenuPos.value = { x: Math.max(8, x), y: Math.max(8, y) };
  }

  function closeLibMenu() {
    libMenuPack.value = null;
    libMenuPos.value = null;
  }

  async function libDoPlay() {
    const p = libMenuPack.value;
    closeLibMenu();
    if (p) await playLibraryPack(p);
  }

  /** Публичный сайт: страница-прослойка /mono открывает лаунчер, а если его нет —
   *  предлагает скачать. https-ссылка работает у любого получателя. */
  const SITE_SHARE_URL = "http://2.27.200.74";

  /** Ссылка шаринга сборки (сайт /mono?url=&name=&blog=). */
  function packDeepLink(p: PackDescriptor | null | undefined): string | null {
    if (!p?.url) return null;
    const params = new URLSearchParams({ url: p.url, name: p.name });
    if (p.boostyBlog) params.set("blog", p.boostyBlog);
    return `${SITE_SHARE_URL}/mono?${params.toString()}`;
  }

  async function copyPackDeepLink(p: PackDescriptor | null | undefined) {
    const link = packDeepLink(p);
    if (!p || !link) {
      notify(t("pack.linkLocal"), "error");
      return;
    }
    try {
      await navigator.clipboard.writeText(link);
      notify(t("pack.linkCopied"), "success");
    } catch {
      notify(t("pack.linkCopyFail"), "error");
    }
  }

  function libCopyLink() {
    const p = libMenuPack.value;
    closeLibMenu();
    if (p) void copyPackDeepLink(p);
  }

  function libOpenSettings() {
    const p = libMenuPack.value;
    closeLibMenu();
    if (p) openPackTab(p.id);
  }

  // ─── Ширина сайдбара ──────────────────────────────────────────────────

  const SIDEBAR_COLLAPSE = 260;
  const SIDEBAR_ICON = 78;
  // Шире ~340px контент строк (иконка + короткая подпись) не заполняет панель —
  // справа остаётся пустота. Дизайн рассчитан на ~272px.
  const SIDEBAR_MAX = 340;
  const sidebarWidth = ref(readSidebarWidth());
  const sidebarDragging = ref(false);
  const sidebarCollapsed = computed(() => sidebarWidth.value < SIDEBAR_COLLAPSE);

  function readSidebarWidth(): number {
    const saved = parseInt(localStorage.getItem("mono.sidebarWidth") ?? "", 10);
    if (!Number.isFinite(saved)) return 272;
    // Мёртвая зона (иконки < ширина < порог сворачивания) даёт «свёрнутый» вид
    // на широкой панели — пустота по бокам. Снапим к ближайшему осмысленному.
    if (saved <= SIDEBAR_ICON) return SIDEBAR_ICON;
    if (saved < SIDEBAR_COLLAPSE) return SIDEBAR_COLLAPSE;
    return Math.min(SIDEBAR_MAX, saved);
  }

  function startSidebarDrag(e: PointerEvent) {
    sidebarDragging.value = true;
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onSidebarDrag(e: PointerEvent) {
    if (!sidebarDragging.value) return;
    let w = Math.min(SIDEBAR_MAX, Math.max(SIDEBAR_ICON, e.clientX));
    if (w < SIDEBAR_COLLAPSE) w = SIDEBAR_ICON;
    sidebarWidth.value = w;
  }

  function endSidebarDrag(e: PointerEvent) {
    if (!sidebarDragging.value) return;
    sidebarDragging.value = false;
    try {
      (e.target as HTMLElement).releasePointerCapture(e.pointerId);
    } catch {
      /* ignore */
    }
    let w = sidebarWidth.value;
    if (w < SIDEBAR_COLLAPSE) w = SIDEBAR_ICON;
    else if (w < SIDEBAR_COLLAPSE + 40) w = SIDEBAR_COLLAPSE;
    sidebarWidth.value = w;
    localStorage.setItem("mono.sidebarWidth", String(sidebarWidth.value));
  }

  // ─── Возврат ───────────────────────────────────────────────────────────

  return {
    // Pack categories
    PACK_CATS,
    PACK_CAT_LABELS,
    packsBySource,
    paidPacks,

    // Sidebar categories
    SIDEBAR_CATS_KEY,
    sidebarCat,
    persistSidebarCat,
    toggleSidebarCat,

    // Pack tabs reorder
    PACK_TABS_KEY,
    packTabs,
    persistPackTabs,
    dragPackTab,
    packTabDragStart,
    packTabDragOver,
    packTabDrop,
    packTabDragEnd,

    // Library search
    libQuery,
    filteredPacks,

    // User lib categories
    LIB_CATS_KEY,
    LIB_PACK_CATS_KEY,
    libCats,
    packLibCats,
    saveLibCats,
    savePackLibCats,
    makeCatId,
    createLibCat,
    renameLibCat,
    deleteLibCat,
    packHasCat,
    togglePackCat,
    customLibSections,

    // Lib cat modal
    libCatModal,
    libCatName,
    openCatCreate,
    openCatRename,
    submitCatModal,

    // Library tab state
    libScale,
    setLibScale,
    libTile,
    libPercent,
    libStatus,
    loadLibraryStatus,
    openPackTab,
    playLibraryPack,

    // Library context menu
    libMenuPack,
    libMenuPos,
    openLibMenu,
    closeLibMenu,
    libDoPlay,
    libCopyLink,
    libOpenSettings,

    // Sidebar width
    SIDEBAR_COLLAPSE,
    SIDEBAR_ICON,
    SIDEBAR_MAX,
    sidebarWidth,
    sidebarDragging,
    sidebarCollapsed,
    readSidebarWidth,
    startSidebarDrag,
    onSidebarDrag,
    endSidebarDrag,
  };
}
