import { computed, onMounted, ref } from "vue";
import { getStatus, isTauri } from "~/lib/bridge";
import type { PackDescriptor, AppStatus } from "~/lib/types";
import { useI18n } from "~/composables/useI18n";

export interface UseLibraryDeps {
  packs: Ref<PackDescriptor[]>;
  notify: (text: string, type?: "info" | "error" | "success") => void;
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

  type PackCat = "github" | "custom" | "modrinth" | "curseforge";
  const PACK_CATS: PackCat[] = ["github", "custom", "modrinth", "curseforge"];
  const PACK_CAT_LABELS: Record<PackCat, string> = {
    github: "side.catGitHub", custom: "side.catCustom", modrinth: "side.catModrinth", curseforge: "side.catCurse",
  };

  const SIDEBAR_CATS_KEY = "mono.sidebarCats";
  const sidebarCat = reactive<Record<string, boolean>>({ github: true, custom: true, modrinth: true, curseforge: true });
  { const saved = typeof localStorage !== "undefined" ? (JSON.parse(localStorage.getItem(SIDEBAR_CATS_KEY) || "{}") as Record<string, boolean>) : {}; for (const k of Object.keys(saved)) if (typeof saved[k] === "boolean") sidebarCat[k] = saved[k]; }
  function persistSidebarCat() { localStorage.setItem(SIDEBAR_CATS_KEY, JSON.stringify(sidebarCat)); }
  function toggleSidebarCat(k: string) { sidebarCat[k] = !sidebarCat[k]; persistSidebarCat(); }

  const PACK_TABS_KEY = "mono.packTabs";
  const packTabs = ref<PackCat[]>([...PACK_CATS]);
  { const saved: unknown = typeof localStorage !== "undefined" ? JSON.parse(localStorage.getItem(PACK_TABS_KEY) || "null") : null; if (Array.isArray(saved)) { const order = saved.filter((k): k is PackCat => PACK_CATS.includes(k as PackCat)); for (const k of PACK_CATS) if (!order.includes(k)) order.push(k); packTabs.value = order; } }
  function persistPackTabs() { localStorage.setItem(PACK_TABS_KEY, JSON.stringify(packTabs.value)); }

  type PacksBySource = Record<PackCat, PackDescriptor[]>;
  const packsBySource = computed<PacksBySource>(() => {
    const out: PacksBySource = { github: [], custom: [], modrinth: [], curseforge: [] };
    for (const p of filteredPacks.value) { const group: PackCat = p.url.includes("modrinth.") ? "modrinth" : p.url.includes("curseforge.com") ? "curseforge" : p.url.startsWith("local://") ? "custom" : "github"; out[group].push(p); }
    return out;
  });

  const libQuery = ref("");
  const filteredPacks = computed<PackDescriptor[]>(() => {
    const q = libQuery.value.trim().toLowerCase();
    if (!q) return packs.value;
    return packs.value.filter((p) => p.name.toLowerCase().includes(q) || p.id.toLowerCase().includes(q));
  });

  interface LibCat { id: string; name: string }
  const LIB_CATS_KEY = "mono.libCats";
  const LIB_PACK_CATS_KEY = "mono.libPackCats";
  function loadJson<T>(key: string, fallback: T): T { try { const raw = localStorage.getItem(key); return raw ? (JSON.parse(raw) as T) : fallback; } catch { return fallback; } }
  const libCats = ref<LibCat[]>(loadJson<LibCat[]>(LIB_CATS_KEY, []));
  const packLibCats = ref<Record<string, string[]>>(loadJson<Record<string, string[]>>(LIB_PACK_CATS_KEY, {}));
  function saveLibCats() {
    const seen = new Set<string>();
    libCats.value = libCats.value.filter(c => c && c.id && !seen.has(c.id) && seen.add(c.id));
    const packSetSeen = new Set<string>();
    const uniqCats: typeof libCats.value = [];
    for (const c of libCats.value) {
      const key = Object.entries(packLibCats.value).filter(([,cats]) => (cats as string[]).includes(c.id)).map(([pid])=>pid).sort().join(",");
      if (!key) { uniqCats.push(c); continue; }
      if (packSetSeen.has(key) && key.split(",").length === 2) {
        for (const pid of key.split(",")) {
          packLibCats.value[pid] = (packLibCats.value[pid]||[]).filter((id:string)=>id!==c.id);
          if (!packLibCats.value[pid]?.length) delete packLibCats.value[pid];
        }
        continue;
      }
      packSetSeen.add(key);
      uniqCats.push(c);
    }
    if (uniqCats.length !== libCats.value.length) libCats.value = uniqCats;
    localStorage.setItem(LIB_CATS_KEY, JSON.stringify(libCats.value));
  }
  function savePackLibCats() {
    for (const k of Object.keys(packLibCats.value)) {
      const uniq = [...new Set((packLibCats.value[k] || []).filter(Boolean))];
      if (uniq.length) packLibCats.value[k] = uniq; else delete packLibCats.value[k];
    }
    localStorage.setItem(LIB_PACK_CATS_KEY, JSON.stringify(packLibCats.value));
  }
  // дедупликация при загрузке (по id и составу)
  {
    const s = new Set<string>();
    const dedup = (libCats.value || []).filter(c => c && c.id && !s.has(c.id) && s.add(c.id));
    if (dedup.length !== libCats.value.length) libCats.value = dedup;
    const before = JSON.stringify(libCats.value);
    saveLibCats();
    if (JSON.stringify(libCats.value) !== before) savePackLibCats();
  }
  function makeCatId(name: string): string { const slug = name.trim().toLowerCase().replace(/[^a-zа-яё0-9]+/gi, "-").replace(/^-+|-+$/g, ""); let id = `c-${slug || "cat"}`; let n = 2; while (libCats.value.some((c) => c.id === id)) id = `c-${slug || "cat"}-${n++}`; return id; }
  function packHasCat(packId: string, catId: string): boolean { return (packLibCats.value[packId] ?? []).includes(catId); }

  function cleanupEmptyFolders() {
    let changed = false;
    for (const cat of [...libCats.value]) {
      const count = filteredPacks.value.filter((p) => packHasCat(p.id, cat.id)).length;
      if (count < 2) {
        libCats.value = libCats.value.filter((c) => c.id !== cat.id);
        for (const k of Object.keys(packLibCats.value)) { packLibCats.value[k] = (packLibCats.value[k] ?? []).filter((cid) => cid !== cat.id); if (packLibCats.value[k].length === 0) delete packLibCats.value[k]; }
        changed = true;
      }
    }
    if (changed) { saveLibCats(); savePackLibCats(); }
  }

  function createFolderWithPacks(packId1: string, packId2: string) {
    const newCatId = 'cat_' + Date.now();
    const newCat = { id: newCatId, name: 'Новая категория' };
    libCats.value = [...libCats.value, newCat];
    packLibCats.value = { ...packLibCats.value, [packId1]: [newCatId], [packId2]: [newCatId] };
    saveLibCats();
    savePackLibCats();
    console.log('КАТЕГОРИЯ СОЗДАНА:', newCatId, 'для сборок:', packId1, packId2);
  }

  function addPackToFolder(packId: string, folderId: string) {
    const cur = packLibCats.value[packId] ?? [];
    if (!cur.includes(folderId)) {
      packLibCats.value[packId] = [...cur, folderId];
      savePackLibCats();
    }
  }

  const folderPacks = computed(() => {
    const map: Record<string, PackDescriptor[]> = {};
    for (const cat of libCats.value) { map[cat.id] = filteredPacks.value.filter((p) => packHasCat(p.id, cat.id)); }
    return map;
  });

  function togglePackCat(packId: string, catId: string) { const cur = packLibCats.value[packId] ?? []; packLibCats.value[packId] = cur.includes(catId) ? cur.filter((c) => c !== catId) : [...cur, catId]; if (packLibCats.value[packId].length === 0) delete packLibCats.value[packId]; savePackLibCats(); }

  const gridItems = computed(() => {
    cleanupEmptyFolders();
    const items: any[] = [];
    for (const cat of libCats.value) {
      const packsInCat = filteredPacks.value.filter((p) => packHasCat(p.id, cat.id));
      if (packsInCat.length >= 2) { items.push({ type: 'folder', cat, packs: packsInCat }); }
    }
    for (const pack of filteredPacks.value) {
      const hasBigFolder = libCats.value.some((cat) => { if (!packHasCat(pack.id, cat.id)) return false; const count = filteredPacks.value.filter((p) => packHasCat(p.id, cat.id)).length; return count >= 2; });
      if (!hasBigFolder) { items.push({ type: 'pack', pack }); }
    }
    return items;
  });

  const selectedPacks = ref<Set<string>>(new Set());
  function togglePackSelect(packId: string, ctrlOrMeta: boolean) { if (ctrlOrMeta) { const s = new Set(selectedPacks.value); s.has(packId) ? s.delete(packId) : s.add(packId); selectedPacks.value = s; } else { selectedPacks.value = new Set([packId]); } }
  function clearSelection() { selectedPacks.value = new Set(); }
  function isPackSelected(id: string): boolean { return selectedPacks.value.has(id); }

  const pointerDragId = ref<string | null>(null);
  const pointerOverId = ref<string | null>(null);
  const pointerDragging = ref(false);

  function onPointerDragStart(packId: string) {
    pointerDragId.value = packId;
    pointerDragging.value = true;
  }

  function onPointerDragOver(packId: string) {
    if (pointerDragId.value && pointerDragId.value !== packId) {
      pointerOverId.value = packId;
    }
  }

  function onPointerDragLeave() {
    pointerOverId.value = null;
  }

  function onPointerDrop(targetId: string) {
    const srcId = pointerDragId.value;
    pointerDragId.value = null;
    pointerOverId.value = null;
    pointerDragging.value = false;
    if (!srcId || srcId === targetId) return;
    createFolderWithPacks(srcId, targetId);
  }

  function onPointerDragEnd() {
    pointerDragId.value = null;
    pointerOverId.value = null;
    pointerDragging.value = false;
  }

  const dragOverFolderId = ref<string | null>(null);
  function onFolderPointerOver(folderId: string) { if (pointerDragId.value) dragOverFolderId.value = folderId; }
  function onFolderPointerLeave() { dragOverFolderId.value = null; }
  function removePackFromFolder(packId: string, folderId: string): boolean {
    const cur = packLibCats.value[packId] ?? [];
    if (cur.includes(folderId)) { packLibCats.value[packId] = cur.filter((c) => c !== folderId); if (packLibCats.value[packId].length === 0) delete packLibCats.value[packId]; }
    const remaining = filteredPacks.value.filter((p) => packHasCat(p.id, folderId));
    if (remaining.length <= 1) { remaining.forEach((p) => { delete packLibCats.value[p.id]; }); libCats.value = libCats.value.filter((c) => c.id !== folderId); saveLibCats(); savePackLibCats(); return true; }
    saveLibCats(); savePackLibCats(); return false;
  }
  function onFolderDrop(folderId: string) {
    const srcId = pointerDragId.value;
    if (!srcId) return;
    addPackToFolder(srcId, folderId);
    pointerDragId.value = null;
    pointerOverId.value = null;
    dragOverFolderId.value = null;
  }

  onMounted(cleanupEmptyFolders);

  const libScale = ref(2);
  const LIB_TILES: Record<number, { col: string; icon: string }> = { 1: { col: "grid-cols-[repeat(auto-fill,minmax(92px,1fr))]", icon: "h-10 w-10" }, 2: { col: "grid-cols-[repeat(auto-fill,minmax(124px,1fr))]", icon: "h-14 w-14" }, 3: { col: "grid-cols-[repeat(auto-fill,minmax(164px,1fr))]", icon: "h-20 w-20" }, 4: { col: "grid-cols-[repeat(auto-fill,minmax(220px,1fr))]", icon: "h-28 w-28" } };
  const libTile = computed(() => LIB_TILES[libScale.value] ?? LIB_TILES[2]);
  const libPercent = computed(() => 75 + libScale.value * 25);
  const libStatus = reactive<Record<string, AppStatus | null>>({});
  let libStatusLoading = false;
  async function loadLibraryStatus() { if (!isTauri() || libStatusLoading) return; libStatusLoading = false; try { await Promise.all(packs.value.map(async (p) => { libStatus[p.id] = await getStatus(p.id).catch(() => null); })); } finally { libStatusLoading = false; } }
  watch(() => tab.value, (t) => { if (t === "library") loadLibraryStatus(); }); onMounted(loadLibraryStatus);

  async function openPackTab(id: string) { if (packId.value !== id) await selectPack(id); tab.value = "play"; }
  async function playLibraryPack(p: PackDescriptor) { if (!isTauri()) return; await openPackTab(p.id); if (status.value?.installed) await handlePlay(); else await handleInstall(); }

  const libMenuPack = ref<PackDescriptor | null>(null);
  const libMenuPos = ref<{ x: number; y: number } | null>(null);
  function openLibMenu(e: MouseEvent, p: PackDescriptor) { const menuW = 224; const menuH = 120; const x = Math.min(e.clientX, window.innerWidth - menuW - 8); const y = Math.min(e.clientY, window.innerHeight - menuH - 8); libMenuPack.value = p; libMenuPos.value = { x: Math.max(8, x), y: Math.max(8, y) }; }
  function closeLibMenu() { libMenuPack.value = null; libMenuPos.value = null; }
  async function libDoPlay() { const p = libMenuPack.value; closeLibMenu(); if (p) await playLibraryPack(p); }
  const SITE_SHARE_URL = "http://2.27.200.74";
  function packDeepLink(p: PackDescriptor | null | undefined): string | null { if (!p?.url) return null; const params = new URLSearchParams({ url: p.url, name: p.name }); if (p.boostyBlog) params.set("blog", p.boostyBlog); return `${SITE_SHARE_URL}/mono?${params.toString()}`; }
  async function copyPackDeepLink(p: PackDescriptor | null | undefined) { const link = packDeepLink(p); if (!p || !link) { notify(t("pack.linkLocal"), "error"); return; } try { await navigator.clipboard.writeText(link); notify(t("pack.linkCopied"), "success"); } catch { notify(t("pack.linkCopyFail"), "error"); } }
  function libCopyLink() { const p = libMenuPack.value; closeLibMenu(); if (p) void copyPackDeepLink(p); }
  function libOpenSettings() { const p = libMenuPack.value; closeLibMenu(); if (p) openPackTab(p.id); }

  const SIDEBAR_COLLAPSE = 260; const SIDEBAR_ICON = 78; const SIDEBAR_MAX = 340;
  const sidebarWidth = ref(272);
  const sidebarDragging = ref(false);
  const sidebarCollapsed = computed(() => sidebarWidth.value < SIDEBAR_COLLAPSE);
  function readSidebarWidth(): number { const saved = parseInt(localStorage.getItem("mono.sidebarWidth") ?? "", 10); if (!Number.isFinite(saved)) return 272; if (saved <= SIDEBAR_ICON) return SIDEBAR_ICON; if (saved < SIDEBAR_COLLAPSE) return SIDEBAR_COLLAPSE; return Math.min(SIDEBAR_MAX, saved); }
  function startSidebarDrag(e: PointerEvent) { sidebarDragging.value = true; (e.target as HTMLElement).setPointerCapture(e.pointerId); }
  function onSidebarDrag(e: PointerEvent) { if (!sidebarDragging.value) return; let w = Math.min(SIDEBAR_MAX, Math.max(SIDEBAR_ICON, e.clientX)); if (w < SIDEBAR_COLLAPSE) w = SIDEBAR_ICON; sidebarWidth.value = w; }
  function endSidebarDrag(e: PointerEvent) { if (!sidebarDragging.value) return; sidebarDragging.value = false; try { (e.target as HTMLElement).releasePointerCapture(e.pointerId); } catch { } let w = sidebarWidth.value; if (w < SIDEBAR_COLLAPSE) w = SIDEBAR_ICON; else if (w < SIDEBAR_COLLAPSE + 40) w = SIDEBAR_COLLAPSE; sidebarWidth.value = w; localStorage.setItem("mono.sidebarWidth", String(sidebarWidth.value)); }

  return {
    PACK_CATS, PACK_CAT_LABELS, packsBySource,
    libQuery, filteredPacks,
    libCats, packLibCats, packHasCat, togglePackCat,
    folderPacks, gridItems,
    selectedPacks, togglePackSelect, clearSelection, isPackSelected,
    pointerDragId, pointerOverId, pointerDragging,
    onPointerDragStart, onPointerDragOver, onPointerDragLeave, onPointerDrop, onPointerDragEnd,
    dragOverFolderId, onFolderPointerOver, onFolderPointerLeave, removePackFromFolder, onFolderDrop, createFolderWithPacks,
    libScale, setLibScale: (n: number) => { libScale.value = n; }, libTile, libPercent, libStatus, loadLibraryStatus, openPackTab, playLibraryPack,
    libMenuPack, libMenuPos, openLibMenu, closeLibMenu, libDoPlay, libCopyLink, libOpenSettings,
    SIDEBAR_COLLAPSE, SIDEBAR_ICON, SIDEBAR_MAX, sidebarWidth, sidebarDragging, sidebarCollapsed, readSidebarWidth, startSidebarDrag, onSidebarDrag, endSidebarDrag,
    saveLibCats, savePackLibCats,
  };
}
