<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { packGradient } from "~/lib/misc";
import { useLauncherCtx } from "~/composables/useLauncherContext";
import ModpackCard from "~/components/ModpackCard.vue";
import LibraryFolderCard from "~/components/LibraryFolderCard.vue";

const ctx = useLauncherCtx();
const {
  packs,
  filteredPacks,
  libQuery,
  libScale,
  libTile,
  setLibScale,
  libPercent,
  playLibraryPack,
  openPackTab,
  openLibMenu,
  libCats,
  packLibCats,
  saveCategories,
} = ctx;

const expandedFolderId = ref<string | null>(null);

function toggleFolder(id: string) {
  expandedFolderId.value = expandedFolderId.value === id ? null : id;
}

const draggingPackId = ref<string | null>(null);
const hoverTargetId = ref<string | null>(null);
const mouseX = ref(0);
const mouseY = ref(0);
const isDragging = ref(false);
const startPos = ref({ x: 0, y: 0 });
// rAF-троттлинг pointermove (см. onGlobalPointerMove).
const pendingPointer = { x: 0, y: 0 };
let pointerRaf: number | null = null;

const draggingPack = computed(() => {
  if (!draggingPackId.value) return null;
  return (packs.value || []).find((p: any) => p.id === draggingPackId.value) || null;
});

const draggingIconSrc = computed(() => {
  if (!draggingPack.value) return null;
  const icon = (draggingPack.value as any).icon || (draggingPack.value as any).icon_url;
  return icon && typeof icon === "string" ? convertFileSrc(icon) : null;
});

const gridItems = computed(() => {
  const items: any[] = [];
  const assignedPackIds = new Set<string>();

  for (const cat of libCats.value || []) {
    const folderPacks = (filteredPacks.value || []).filter((p: any) => {
      const cats = packLibCats.value[p.id];
      return Array.isArray(cats) ? cats.includes(cat.id) : cats === cat.id;
    });

    if (folderPacks.length >= 2) {
      if (expandedFolderId.value === cat.id) {
        items.push({
          type: "expanded-category-group",
          folder: cat,
          packs: folderPacks,
        });
        folderPacks.forEach((p: any) => assignedPackIds.add(p.id));
      } else {
        // Collapsed folder tile
        items.push({ type: "folder", cat, packs: folderPacks });
        folderPacks.forEach((p: any) => assignedPackIds.add(p.id));
      }
    }
  }

  // Regular single packs
  for (const pack of filteredPacks.value || []) {
    if (!assignedPackIds.has(pack.id)) {
      items.push({ type: "pack", pack });
    }
  }

  return items;
});

function resetDragState() {
  draggingPackId.value = null;
  hoverTargetId.value = null;
  isDragging.value = false;
  if (pointerRaf !== null) {
    cancelAnimationFrame(pointerRaf);
    pointerRaf = null;
  }
}

function onCardPointerDown(packId: string, e: PointerEvent) {
  if (e.button !== 0) return;
  e.preventDefault();

  draggingPackId.value = packId;
  startPos.value = { x: e.clientX, y: e.clientY };
  mouseX.value = e.clientX;
  mouseY.value = e.clientY;
  isDragging.value = false;
}

function onGlobalPointerMove(e: MouseEvent | PointerEvent) {
  if (!draggingPackId.value) return;
  // Троттлинг через rAF: pointermove стреляет сотнями событий в секунду,
  // а каждая запись в ref перерендеривает сетку. Копим последнюю позицию
  // и применяем её максимум раз за кадр.
  pendingPointer.x = e.clientX;
  pendingPointer.y = e.clientY;
  if (pointerRaf !== null) return;
  pointerRaf = requestAnimationFrame(() => {
    pointerRaf = null;
    const x = pendingPointer.x;
    const y = pendingPointer.y;
    mouseX.value = x;
    mouseY.value = y;

    if (!isDragging.value && Math.hypot(x - startPos.value.x, y - startPos.value.y) > 5) {
      isDragging.value = true;
    }

    if (isDragging.value) {
      const elem = document.elementFromPoint(x, y);
      const dropTarget = elem?.closest("[data-drop-target]");
      const targetId = dropTarget?.getAttribute("data-drop-target");

      if (targetId && targetId !== draggingPackId.value) {
        hoverTargetId.value = targetId;
      } else {
        hoverTargetId.value = null;
      }
    }
  });
}

function onGlobalPointerUp() {
  if (!draggingPackId.value) return;

  const sourceId = draggingPackId.value;
  const targetId = hoverTargetId.value;
  const wasDragging = isDragging.value;

  resetDragState();

  if (wasDragging) {
    if (targetId && targetId !== sourceId) {
      const isTargetFolder = (libCats.value || []).some((c: any) => c.id === targetId);

      if (isTargetFolder) {
        const current = packLibCats.value[sourceId] || [];
        const set = new Set(Array.isArray(current) ? current : [current]);
        set.add(targetId);
        packLibCats.value = { ...packLibCats.value, [sourceId]: [...set] };
      } else {
        const curSrc = (packLibCats.value[sourceId] || []) as string[];
        const curTgt = (packLibCats.value[targetId] || []) as string[];
        const common = curSrc.find(id => curTgt.includes(id) && (libCats.value || []).some((c: any) => c.id === id));

        if (!common) {
          const newCatId = "cat_" + Date.now() + "_" + Math.random().toString(36).slice(2, 6);
          libCats.value = [...(libCats.value || []), { id: newCatId, name: "Категория" }];
          packLibCats.value = {
            ...packLibCats.value,
            [sourceId]: [...curSrc, newCatId],
            [targetId]: [...curTgt, newCatId],
          };
        }
      }
      if (typeof saveCategories === "function") saveCategories();
    } else if (!targetId && expandedFolderId.value) {
      extractPackFromFolder(sourceId, expandedFolderId.value);
    }
  }
}

function extractPackFromFolder(packId: string, folderId: string) {
  const current = packLibCats.value[packId] || [];
  packLibCats.value = {
    ...packLibCats.value,
    [packId]: Array.isArray(current) ? current.filter((id: string) => id !== folderId) : [],
  };

  const remaining = (packs.value || []).filter((p: any) => {
    const cats = packLibCats.value[p.id];
    return Array.isArray(cats) ? cats.includes(folderId) : cats === folderId;
  });

  if (remaining.length <= 1) {
    remaining.forEach((p: any) => {
      const c = packLibCats.value[p.id] || [];
      packLibCats.value[p.id] = Array.isArray(c) ? c.filter((id: string) => id !== folderId) : [];
    });
    libCats.value = (libCats.value || []).filter((c: any) => c.id !== folderId);
    if (expandedFolderId.value === folderId) expandedFolderId.value = null;
  }

  if (typeof saveCategories === "function") saveCategories();
}

function handlePackClick(packId: string) {
  if (isDragging.value) return;
  openPackTab(packId);
}

onMounted(() => {
  window.addEventListener("pointermove", onGlobalPointerMove, { capture: true });
  window.addEventListener("pointerup", onGlobalPointerUp, { capture: true });
  window.addEventListener("pointercancel", resetDragState, { capture: true });
  window.addEventListener("mouseup", onGlobalPointerUp, { capture: true });
  window.addEventListener("blur", resetDragState);
  window.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      expandedFolderId.value = null;
      resetDragState();
    }
  });
});

onBeforeUnmount(() => {
  window.removeEventListener("pointermove", onGlobalPointerMove, { capture: true });
  window.removeEventListener("pointerup", onGlobalPointerUp, { capture: true });
  window.removeEventListener("pointercancel", resetDragState, { capture: true });
  window.removeEventListener("mouseup", onGlobalPointerUp, { capture: true });
  window.removeEventListener("blur", resetDragState);
});
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col select-none relative overflow-hidden">
    <!-- Header -->
    <div class="mb-5 flex shrink-0 items-center justify-between gap-4 border-b border-[var(--border)] pb-4">
      <div class="relative">
        <input
          v-model="libQuery"
          type="text"
          class="w-52 rounded-xl bg-[var(--input)] border border-[var(--border)] px-3.5 py-1.5 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none focus:border-[var(--accent)] transition-all"
          placeholder="Поиск..."
        />
      </div>

      <div class="flex shrink-0 items-center gap-1.5 bg-[var(--input)] p-1 rounded-xl border border-[var(--border)]">
        <button
          type="button"
          class="w-7 h-7 rounded-lg bg-[var(--panel)] hover:brightness-125 flex items-center justify-center text-sm font-bold text-[color:var(--tx-muted)] hover:text-[color:var(--tx)] active:scale-95 disabled:opacity-20 transition-all"
          :disabled="libScale <= 1"
          @click="setLibScale(libScale - 1)"
        >
          -
        </button>
        <span class="w-11 text-center text-xs font-semibold tabular-nums text-[color:var(--tx)]">{{ libPercent }}%</span>
        <button
          type="button"
          class="w-7 h-7 rounded-lg bg-[var(--panel)] hover:brightness-125 flex items-center justify-center text-sm font-bold text-[color:var(--tx-muted)] hover:text-[color:var(--tx)] active:scale-95 disabled:opacity-20 transition-all"
          :disabled="libScale >= 4"
          @click="setLibScale(libScale + 1)"
        >
          +
        </button>
      </div>
    </div>

    <!-- PURE GRID: Every single item is 1 cell. No broken rows, no shelves, no gaps! -->
    <div class="min-h-0 flex-1 overflow-y-auto pr-1 pt-3 pb-6">
      <div class="grid gap-4 items-center px-1 transition-all" :class="libTile.col">
        <template v-for="item in gridItems" :key="item.type + (item.folder?.id || item.cat?.id || item.pack?.id) + (item.pack?.id || '')">

          <!-- UNIFIED EXPANDED CATEGORY (LOCKED GRID ROW HEIGHT) -->
          <div
            v-if="item.type === 'expanded-category-group'"
            :data-drop-target="item.folder.id"
            class="relative grid rounded-3xl bg-white/[0.05] p-2 -m-2 items-center transition-all shadow-inner"
            :style="{
              gridColumn: `span ${1 + item.packs.length}`,
              gridTemplateColumns: `repeat(${1 + item.packs.length}, minmax(0, 1fr))`,
              gap: '1rem'
            }"
          >
            <!-- 1. RED CLOSE BUTTON (EXACT SAME SQUARE SIZE AS CARDS) -->
            <button
              type="button"
              class="aspect-square w-full h-full rounded-2xl bg-rose-500/10 hover:bg-rose-500/20 active:scale-95 transition-all cursor-pointer flex items-center justify-center group select-none shadow-sm"
              @click="toggleFolder(item.folder.id)"
              title="Закрыть категорию"
            >
              <div class="w-12 h-12 rounded-full bg-rose-500/20 group-hover:bg-rose-500 flex items-center justify-center transition-all group-hover:scale-110">
                <svg viewBox="0 0 24 24" class="w-7 h-7 stroke-rose-400 group-hover:stroke-white stroke-[2.5] fill-none stroke-linecap-round stroke-linejoin-round transition-colors">
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
              </div>
            </button>

            <!-- 2. PACK CARDS INSIDE (PERFECT SQUARES) -->
            <div
              v-for="pack in item.packs"
              :key="pack.id"
              class="relative aspect-square w-full"
            >
              <ModpackCard
                :pack="pack"
                class="w-full h-full shadow-md !border-none !ring-0"
                :class="{ 'opacity-30 scale-95': draggingPackId === pack.id }"
                @pointerdown="onCardPointerDown(pack.id, $event)"
                @click="handlePackClick(pack.id)"
                @contextmenu="openLibMenu"
                @play="playLibraryPack"
              />

              <!-- Extract button on hover -->
              <button
                type="button"
                class="absolute bottom-2 right-2 z-30 w-6 h-6 rounded-full bg-black/80 border border-white/20 text-white/80 opacity-0 group-hover:opacity-100 hover:bg-rose-600 hover:text-white hover:border-rose-600 transition-all shadow-md active:scale-90 flex items-center justify-center text-xs"
                @click.stop="extractPackFromFolder(pack.id, item.folder.id)"
                title="Извлечь из категории"
              >
                ✕
              </button>
            </div>
          </div>

          <!-- 2. COLLAPSED FOLDER 2x2 (1 cell) -->
          <div
            v-else-if="item.type === 'folder'"
            :data-drop-target="item.cat.id"
            class="aspect-square w-full rounded-2xl transition-all duration-200"
            :class="{
              'ring-2 ring-[var(--accent)] scale-105 shadow-xl': hoverTargetId === item.cat.id
            }"
          >
            <LibraryFolderCard
              :folder="item.cat"
              :folder-packs="item.packs"
              class="w-full h-full"
              @click="toggleFolder(item.cat.id)"
            />
          </div>

          <!-- 3. ORDINARY PACK (1 cell) -->
          <div
            v-else-if="item.type === 'pack'"
            :data-drop-target="item.pack.id"
            class="aspect-square w-full rounded-2xl transition-all duration-200"
            :class="{
              'opacity-30 scale-95': draggingPackId === item.pack.id,
              'ring-2 ring-[var(--accent)] scale-105 shadow-xl': hoverTargetId === item.pack.id
            }"
          >
            <ModpackCard
              :pack="item.pack"
              class="w-full h-full"
              @pointerdown="onCardPointerDown(item.pack.id, $event)"
              @click="handlePackClick(item.pack.id)"
              @contextmenu="openLibMenu"
              @play="playLibraryPack"
            />
          </div>

        </template>
      </div>
    </div>

    <!-- Drag Ghost -->
    <div
      v-if="isDragging && draggingPack"
      class="fixed pointer-events-none z-[99999] w-20 h-20 rounded-2xl bg-[var(--panel)] border-2 border-[var(--accent)] p-1.5 shadow-2xl flex items-center justify-center -translate-x-1/2 -translate-y-1/2 rotate-3 select-none"
      :style="{ left: `${mouseX}px`, top: `${mouseY}px` }"
    >
      <img
        v-if="draggingIconSrc"
        :src="draggingIconSrc"
        class="w-full h-full object-contain rounded-xl select-none"
        draggable="false"
      />
      <div
        v-else
        class="w-full h-full rounded-xl flex items-center justify-center text-white font-black text-2xl select-none"
        :style="{ background: packGradient((draggingPack as any).color || draggingPack.name || 'M') }"
      >
        {{ (draggingPack.name || 'M')[0].toUpperCase() }}
      </div>
    </div>
  </div>
</template>
