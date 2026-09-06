<script setup lang="ts">
import { ref } from "vue";
import { useLauncherCtx } from '~/composables/useLauncherContext';
const ctx = useLauncherCtx();
const {
  t,
  catalogSource,
  switchCatalogSource,
  loadMonoCatalog,
  monoCatalogLoading,
  catalogDetail,
  openCatalogDetail,
  closeCatalogDetail,
  addMonoPack,
  openMonoPack,
  openProfileView,
  packs,
  addingPack,
  ratePack,
  catalogCommentsBusy,
  catalogDetailBusy,
  catalogDetailTab,
  catalogCommentCount,
  commentDraft,
  monoProfile,
  sendCatalogComment,
  catalogComments,
  commentEditId,
  commentEditDraft,
  editCatalogComment,
  commentReplyTo,
  commentReplyDraft,
  removeCatalogComment,
  rateCatalogComment,
  isAdmin,
  monoCatalog,
  monoCatalogError,
  isMonoPackAdded,
  formatBytes,
  formatDate,
  openExternal,
  packFilters,
  packVersionOptions,
  packLoaderOptions,
  packVersionTypeSel,
  versionTypeOptions,
  packCategoryOptions,
  packEnvSel,
  envOptions,
  packSortSel,
  sortSelectOptions,
  modPackQuery,
  modPackLoading,
  modPackResults,
  openCatalogModrinthDetail,
  quickPackBusy,
  quickDownloadPack,
  searchPacks,
  cpCatSel,
  cpCatOptions,
  cpVerSel,
  cpSortSel,
  curseSortOptions,
  searchCursePacks,
  curseKeyOk,
  cpSearched,
  cpLoading,
  cpErr,
  cpResults,
  openCatalogCurseDetail,
  quickCpBusy,
  quickDownloadCpPack,
  loadMorePacks,
  loadMoreCpPacks,
  modPackMoreBusy,
  cpMoreBusy,
  modPackMore,
cpMore,
  createPackOpen,
  modPackFullPage,
  modPackDetail,
  cpProject,
  closeModPackFullPage,
} = ctx;

// Grid system for pack display
const isCategoryExpanded = ref({'mono': false, 'modrinth': false, 'curse': false});

const GRID_COLS = 2;
const PACKS_PER_SQUARE = 4;

function calculateGridLayout(totalPacks: number) {
  const fullSquares = Math.floor(totalPacks / PACKS_PER_SQUARE);
  const remainingPacks = totalPacks % PACKS_PER_SQUARE;
  return { fullSquares, remainingPacks };
}

function getRemainingColumns(remainingPacks: number) {
  if (remainingPacks <= 2) return remainingPacks;
  return 3;
}

function getPackGridInfo(packIndex: number, totalPacks: number) {
  const { fullSquares, remainingPacks } = calculateGridLayout(totalPacks);
  const remainingColumns = getRemainingColumns(remainingPacks);
  
  if (packIndex < fullSquares * PACKS_PER_SQUARE) {
    const squareIndex = Math.floor(packIndex / PACKS_PER_SQUARE);
    const posInSquare = packIndex % PACKS_PER_SQUARE;
    const col = posInSquare % GRID_COLS;
    const row = Math.floor(posInSquare / GRID_COLS);
    return { col, row, squareIndex, isFullSquare: true };
  } else {
    const posInRemaining = packIndex - fullSquares * PACKS_PER_SQUARE;
    const col = posInRemaining % remainingColumns;
    const row = Math.floor(posInRemaining / remainingColumns);
    return { col, row, isRemaining: true };
  }
}

const catalogScrollRef = ref<HTMLElement | null>(null);

function onCatalogScroll() {
  const el = catalogScrollRef.value;
  if (!el) return;
  if (catalogSource.value === "modrinth") loadMorePacks();
  else if (catalogSource.value === "curse") loadMoreCpPacks();
}

// Дебаунс поиска: кнопку «Найти» убрали, ищем по Enter или через 600мс после ввода.
let packSearchTimer: ReturnType<typeof setTimeout> | null = null;
function debouncedSearchPacks() {
  if (packSearchTimer) clearTimeout(packSearchTimer);
  packSearchTimer = setTimeout(() => searchPacks(), 600);
}
let curseSearchTimer: ReturnType<typeof setTimeout> | null = null;
function debouncedSearchCursePacks() {
  if (curseSearchTimer) clearTimeout(curseSearchTimer);
  curseSearchTimer = setTimeout(() => searchCursePacks(), 600);
}

/** Компактные счётчики: 2.6M вместо 2 600 000. */
function compactDownloads(n: number): string {
  if (!Number.isFinite(n)) return "0";
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1).replace(/\.0$/, "")}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1).replace(/\.0$/, "")}K`;
  return String(n);
}

/** Full-page профиль сборки (вместо модалки). Скролл сетки сохраняем/возвращаем. */
const profileOpen = computed(() => !!(modPackFullPage.value && (modPackDetail.value || cpProject.value)));
let savedCatalogScroll = 0;
watch(profileOpen, (open) => {
  const el = catalogScrollRef.value;
  if (open) {
    if (el) savedCatalogScroll = el.scrollTop;
  } else if (el) {
    nextTick(() => { el.scrollTop = savedCatalogScroll; });
  }
});
function backToCatalogList() {
  closeModPackFullPage();
}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
  <div class="mb-5 flex shrink-0 items-center justify-between gap-4 border-b border-[var(--border)]  pb-4">
  <div>
  <h2 class="text-xl font-bold tracking-tight text-[color:var(--tx-strong)]">{{ t("catalog.title") }}</h2>
  <p class="mt-1 text-[13px] text-[color:var(--tx-muted)]">{{ t("catalog.subtitle") }}</p>
  </div>
  <div class="flex shrink-0 items-center gap-2">
  <button
  type="button"
  class="rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] disabled:opacity-50"
  :disabled="monoCatalogLoading"
  @click="loadMonoCatalog"
  >
  {{ t("catalog.refresh") }}
  </button>
  </div>
  </div>

  <div v-if="!catalogDetail && !profileOpen" class="mb-4 inline-flex w-fit shrink-0 items-center gap-1 rounded-xl bg-[var(--panel)] p-1 shadow-sm">
  <button
  v-for="src in (['mono', 'modrinth', 'curse'] as const)"
  :key="src"
  type="button"
  class="flex items-center justify-center gap-2 rounded-lg px-4 py-2 text-[13px] font-semibold transition-colors"
  :class="catalogSource === src
  ? 'bg-[var(--input)] text-[var(--accent)]'
  : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]'"
  @click="switchCatalogSource(src)"
  >
  <svg v-if="src === 'mono'" viewBox="0 0 24 24" class="h-4 w-4 rounded-[4px] bg-[var(--accent)]"><path d="M6.5 17V7l5.5 5 5.5-5v10" fill="none" style="stroke: var(--panel)" stroke-width="2.6" stroke-linecap="square" stroke-linejoin="miter"/></svg>
  <svg v-else-if="src === 'modrinth'" viewBox="0 0 24 24" class="h-4 w-4 fill-current"><path fill="#00AF5C" d="M12.252.004a11.78 11.768 0 0 0-8.92 3.73 11 10.999 0 0 0-2.17 3.11 11.37 11.359 0 0 0-1.16 5.169c0 1.42.17 2.5.6 3.77.24.759.77 1.899 1.17 2.529a12.3 12.298 0 0 0 8.85 5.639c.44.05 2.54.07 2.76.02.2-.04.22.1-.26-1.7l-.36-1.37-1.01-.06a8.5 8.489 0 0 1-5.18-1.8 5.34 5.34 0 0 1-1.3-1.26c0-.05.34-.28.74-.5a37.572 37.545 0 0 1 2.88-1.629c.03 0 .5.45 1.06.98l1 .97 2.07-.43 2.06-.43 1.47-1.47c.8-.8 1.48-1.5 1.48-1.52 0-.09-.42-1.63-.46-1.7-.04-.06-.2-.03-1.02.18-.53.13-1.2.3-1.45.4l-.48.15-.53.53-.53.53-.93.1-.93.07-.52-.5a2.7 2.7 0 0 1-.96-1.7l-.13-.6.43-.57c.68-.9.68-.9 1.46-1.1.4-.1.65-.2.83-.33.13-.099.65-.579 1.14-1.069l.9-.9-.7-.7-.7-.7-1.95.54c-1.07.3-1.96.53-1.97.53-.03 0-2.23 2.48-2.63 2.97l-.29.35.28 1.03c.16.56.3 1.16.31 1.34l.03.3-.34.23c-.37.23-2.22 1.3-2.84 1.63-.36.2-.37.2-.44.1-.08-.1-.23-.6-.32-1.03-.18-.86-.17-2.75.02-3.73a8.84 8.839 0 0 1 7.9-6.93c.43-.03.77-.08.78-.1.06-.17.5-2.999.47-3.039-.01-.02-.1-.02-.2-.03Zm3.68.67c-.2 0-.3.1-.37.38-.06.23-.46 2.42-.46 2.52 0 .04.1.11.22.16a8.51 8.499 0 0 1 2.99 2 8.38 8.379 0 0 1 2.16 3.449 6.9 6.9 0 0 1 .4 2.8c0 1.07 0 1.27-.1 1.73a9.37 9.369 0 0 1-1.76 3.769c-.32.4-.98 1.06-1.37 1.38-.38.32-1.54 1.1-1.7 1.14-.1.03-.1.06-.07.26.03.18.64 2.56.7 2.78l.06.06a12.07 12.058 0 0 0 7.27-9.4c.13-.77.13-2.58 0-3.4a11.96 11.948 0 0 0-5.73-8.578c-.7-.42-2.05-1.06-2.25-1.06Z"/></svg>
  <svg v-else viewBox="0 0 24 24" class="h-4 w-4 fill-current"><path fill="#F16436" d="M18.326 9.2145S23.2261 8.4418 24 6.1882h-7.5066V4.4H0l2.0318 2.3576V9.173s5.1267-.2665 7.1098 1.2372c2.7146 2.516-3.053 5.917-3.053 5.917L5.0995 19.6c1.5465-1.4726 4.494-3.3775 9.8983-3.2857-2.0565.65-4.1245 1.6651-5.7344 3.2857h10.9248l-1.0288-3.2726s-7.918-4.6688-.8336-7.1127z"/></svg>
  {{ src === "mono" ? t("catalog.sourceMono") : src === "modrinth" ? t("mods.serviceModrinth") : t("mods.serviceCurseforge") }}
  </button>
  </div>

  <div ref="catalogScrollRef" class="min-h-0 flex-1 overflow-y-auto pb-6" @scroll="onCatalogScroll">
  <template v-if="catalogSource === 'mono'">
  <!-- Catalog Detail View -->
  <template v-if="catalogDetail">
  <div class="flex flex-col h-full">
  <div class="flex items-center gap-3 mb-4">
  <button type="button" @click="closeCatalogDetail()" class="shrink-0 rounded-md  bg-[var(--input)] px-2 py-1 text-[13px] text-[color:var(--tx-muted)] hover:text-[var(--accent)] transition-colors">
  &larr; {{ t("catalog.title") }}
  </button>
  <div class="flex items-center gap-2 min-w-0">
  <img v-if="catalogDetail.icon_url" :src="catalogDetail.icon_url" class="h-8 w-8 shrink-0 rounded object-cover" />
  <h3 class="truncate text-base font-bold text-[color:var(--tx-strong)]">{{ catalogDetail.name }}</h3>
  <span v-if="catalogDetail.versions?.length" class="shrink-0 rounded  bg-[var(--input-50)] px-1.5 py-0.5 text-xs font-mono text-[color:var(--accent)]">v{{ catalogDetail.versions[0].version }}</span>
  </div>
  <div class="ml-auto flex items-center gap-2 shrink-0">
  <template v-if="catalogDetail.author_user_id">
  <button
  type="button"
  class="max-w-[140px] truncate rounded-md  bg-[var(--input)] px-2 py-1 font-mono text-[13px] text-[color:var(--tx-muted)] transition-colors hover:text-[var(--accent)]"
  :title="t('profile.open')"
  @click="openProfileView(catalogDetail.author_user_id!)"
  >
  @{{ catalogDetail.author_name ?? "?" }}
  </button>
  </template>
  <button
  type="button"
  class="flex shrink-0 items-center gap-1 rounded-md  px-2 py-1 text-[13px] font-semibold transition-colors disabled:opacity-50"
  :class="catalogDetail.my_rating === 1
  ? 'bg-[#3fb950]/15 text-[#3fb950]'
  : ' bg-[var(--input)] text-[color:var(--tx-muted)] hover:text-[#3fb950]'"
  :title="t('comments.like')"
  :disabled="catalogCommentsBusy"
  @click="ratePack(catalogDetail.id, 1)"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M14.25 6.5c.69 0 1.25.56 1.25 1.25v.5c0 .29-.1.57-.28.79l-3.2 3.88a1.75 1.75 0 0 1-1.35.63H6.1a1.75 1.75 0 0 1-1.35-.63L1.53 9.04a1 1 0 0 1-.28-.68V3.5c0-.675.525-1.225 1.193-1.225h2.934c.51 0 .976.285 1.2.74L8.4 5.5c.09.188.28.31.49.31h5.36ZM2.75 3.775a.225.225 0 0 0-.225.225v4.19l2.946 3.573a.25.25 0 0 0 .193.087h1.06L5.05 6.36a1.75 1.75 0 0 1-.3-.985V3.775Z"/></svg>
  {{ catalogDetail.likes }}
  </button>
  <button
  type="button"
  class="flex shrink-0 items-center gap-1 rounded-md  px-2 py-1 text-[13px] font-semibold transition-colors disabled:opacity-50"
  :class="catalogDetail.my_rating === -1
  ? 'bg-[#f85149]/15 text-[#f85149]'
  : ' bg-[var(--input)] text-[color:var(--tx-muted)] hover:text-[#f85149]'"
  :title="t('comments.dislike')"
  :disabled="catalogCommentsBusy"
  @click="ratePack(catalogDetail.id, -1)"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 rotate-180 fill-current"><path d="M14.25 6.5c.69 0 1.25.56 1.25 1.25v.5c0 .29-.1.57-.28.79l-3.2 3.88a1.75 1.75 0 0 1-1.35.63H6.1a1.75 1.75 0 0 1-1.35-.63L1.53 9.04a1 1 0 0 1-.28-.68V3.5c0-.675.525-1.225 1.193-1.225h2.934c.51 0 .976.285 1.2.74L8.4 5.5c.09.188.28.31.49.31h5.36ZM2.75 3.775a.225.225 0 0 0-.225.225v4.19l2.946 3.573a.25.25 0 0 0 .193.087h1.06L5.05 6.36a1.75 1.75 0 0 1-.3-.985V3.775Z"/></svg>
  {{ catalogDetail.dislikes }}
  </button>
  <button v-if="!packs.some((p: any) => p.url === catalogDetail!.url)" type="button" @click="addMonoPack({ url: catalogDetail!.url, name: catalogDetail!.name, boosty_blog: catalogDetail!.boosty_blog } as any)" :disabled="addingPack"
  class="rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] disabled:opacity-50">
  {{ addingPack ? t("dev.adding") : t("catalog.add") }}
  </button>
  <button v-else type="button" @click="openMonoPack({ url: catalogDetail!.url, name: catalogDetail!.name } as any)"
  class="rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]">
  {{ t("catalog.open") }}
  </button>
  </div>
  </div>

  <div v-if="catalogDetailBusy" class="flex items-center justify-center py-16 text-[13px] text-[color:var(--tx-muted)]">
  <AppIcon name="spinner" class="mr-2 h-4 w-4 fill-current" />
  {{ t("catalog.loading") }}
  </div>

  <template v-else>
  <!-- Detail tabs -->
  <div class="mb-4 flex gap-1 overflow-x-auto border-b border-[var(--border)] ">
  <button type="button" @click="catalogDetailTab = 'description'"
  class="relative shrink-0 px-3.5 pb-2.5 pt-1 text-[13px] font-semibold transition-colors"
  :class="catalogDetailTab === 'description' ? 'text-[var(--accent)]' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'">
  {{ t('pack.description') }}
  <span v-if="catalogDetailTab === 'description'" class="absolute inset-x-2 bottom-0 h-[2.5px] rounded-t-full bg-[var(--accent)]"></span>
  </button>
  <button v-if="(catalogDetail.meta as any)?.screenshots?.length" type="button" @click="catalogDetailTab = 'screenshots'"
  class="relative shrink-0 px-3.5 pb-2.5 pt-1 text-[13px] font-semibold transition-colors"
  :class="catalogDetailTab === 'screenshots' ? 'text-[var(--accent)]' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'">
  {{ t('pack.screenshots') }} ({{ ((catalogDetail.meta as any)?.screenshots ?? []).length }})
  <span v-if="catalogDetailTab === 'screenshots'" class="absolute inset-x-2 bottom-0 h-[2.5px] rounded-t-full bg-[var(--accent)]"></span>
  </button>
  <button v-if="catalogDetail.versions?.length" type="button" @click="catalogDetailTab = 'versions'"
  class="relative shrink-0 px-3.5 pb-2.5 pt-1 text-[13px] font-semibold transition-colors"
  :class="catalogDetailTab === 'versions' ? 'text-[var(--accent)]' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'">
  {{ t('pack.versions') }} ({{ catalogDetail.versions.length }})
  <span v-if="catalogDetailTab === 'versions'" class="absolute inset-x-2 bottom-0 h-[2.5px] rounded-t-full bg-[var(--accent)]"></span>
  </button>
  <button v-if="catalogDetail.news?.length" type="button" @click="catalogDetailTab = 'news'"
  class="relative shrink-0 px-3.5 pb-2.5 pt-1 text-[13px] font-semibold transition-colors"
  :class="catalogDetailTab === 'news' ? 'text-[var(--accent)]' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'">
  {{ t('pack.news') }} ({{ catalogDetail.news.length }})
  <span v-if="catalogDetailTab === 'news'" class="absolute inset-x-2 bottom-0 h-[2.5px] rounded-t-full bg-[var(--accent)]"></span>
  </button>
  <button type="button" @click="catalogDetailTab = 'comments'"
  class="relative shrink-0 px-3.5 pb-2.5 pt-1 text-[13px] font-semibold transition-colors"
  :class="catalogDetailTab === 'comments' ? 'text-[var(--accent)]' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'">
  {{ t('comments.tab') }} ({{ catalogCommentCount }})
  <span v-if="catalogDetailTab === 'comments'" class="absolute inset-x-2 bottom-0 h-[2.5px] rounded-t-full bg-[var(--accent)]"></span>
  </button>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto pr-1">
  <!-- Description -->
  <div v-if="catalogDetailTab === 'description'" class="space-y-3">
  <div class="rounded-xl  bg-[var(--panel)] shadow-sm p-4">
  <div v-if="catalogDetail.description" class="text-sm leading-relaxed text-[color:var(--tx)] whitespace-pre-wrap">{{ catalogDetail.description }}</div>
  <p v-else class="text-[13px] text-[color:var(--tx-muted)]">{{ t("common.notFound") }}</p>
  </div>
  <div v-if="catalogDetail.boosty_blog" class="rounded-xl  bg-[var(--panel)] shadow-sm p-3 text-[13px]">
  <span class="text-[color:var(--tx-muted)]">Boosty: </span>
  <a :href="catalogDetail.boosty_blog" class="text-[var(--accent)] hover:underline" @click.prevent="openExternal(catalogDetail.boosty_blog!)">{{ catalogDetail.boosty_blog }}</a>
  </div>
  <div v-if="catalogDetail.min_ram_mb" class="rounded-xl  bg-[var(--panel)] shadow-sm p-3 text-[13px]">
  <span class="text-[color:var(--tx-muted)]">{{ t("pack.minRam") || "Мин. RAM" }}: </span>
  <span>{{ catalogDetail.min_ram_mb }} MB</span>
  </div>
  </div>

  <!-- Screenshots -->
  <div v-if="catalogDetailTab === 'screenshots'" class="space-y-3">
  <div v-if="!(catalogDetail.meta as any)?.screenshots?.length" class="text-center py-8 text-[13px] text-[color:var(--tx-muted)]">{{ t("common.notFound") }}</div>
  <div v-for="(s, i) in ((catalogDetail.meta as any)?.screenshots || [])" :key="i" class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <img :src="typeof s === 'string' ? s : s.url" :alt="`Screenshot ${i + 1}`" class="w-full object-cover max-h-64" loading="lazy" />
  <p v-if="typeof s !== 'string' && s.caption" class="px-3 py-2 text-[13px] text-[color:var(--tx-muted)]">{{ s.caption }}</p>
  </div>
  </div>

  <!-- Versions -->
  <div v-if="catalogDetailTab === 'versions'" class="space-y-2">
  <div v-if="!catalogDetail.versions?.length" class="text-center py-8 text-[13px] text-[color:var(--tx-muted)]">{{ t("common.notFound") }}</div>
  <div v-for="v in catalogDetail.versions" :key="v.id" class="rounded-xl  bg-[var(--panel)] shadow-sm p-3 flex items-center justify-between gap-3">
  <div class="min-w-0">
  <div class="flex items-center gap-2">
  <span class="font-mono text-[13px] font-bold text-[var(--accent)]">v{{ v.version }}</span>
  <span class="text-xs text-[color:var(--tx-muted)]">{{ formatDate(v.created_at) }}</span>
  <span class="text-xs text-[color:var(--tx-muted)]">{{ formatBytes(v.size) }}</span>
  </div>
  <p v-if="v.changelog" class="mt-1 text-[13px] text-[color:var(--tx-muted)] whitespace-pre-wrap line-clamp-2">{{ v.changelog }}</p>
  </div>
  <a :href="v.url" class="shrink-0 rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)]">
  {{ t("pack.download") }}
  </a>
  </div>
  </div>

  <!-- News -->
  <div v-if="catalogDetailTab === 'news'" class="space-y-2">
  <div v-if="!catalogDetail.news?.length" class="text-center py-8 text-[13px] text-[color:var(--tx-muted)]">{{ t("common.notFound") }}</div>
  <div v-for="n in catalogDetail.news" :key="n.id" class="rounded-xl  bg-[var(--panel)] shadow-sm p-3">
  <div class="flex items-center gap-2 mb-1">
  <span class="px-1.5 py-0.5 rounded text-xs font-medium"
  :class="n.kind === 'update' ? 'bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] text-[var(--accent)]' : 'bg-green-500/10 text-green-400'">
  {{ n.kind === 'update' ? 'Update' : 'Post' }}
  </span>
  <span class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ n.title }}</span>
  <span class="text-xs text-[color:var(--tx-muted)] ml-auto">{{ formatDate(n.created_at) }}</span>
  </div>
  <p class="text-[13px] text-[color:var(--tx-muted)] whitespace-pre-wrap">{{ n.body }}</p>
  </div>
  </div>

  <!-- Comments -->
  <div v-if="catalogDetailTab === 'comments'" class="space-y-3">
  <div v-if="catalogCommentsBusy && catalogComments.length === 0" class="flex items-center justify-center py-8 text-[13px] text-[color:var(--tx-muted)]">
  <AppIcon name="spinner" class="mr-2 h-4 w-4 fill-current" />
  {{ t("catalog.loading") }}
  </div>

  <!-- Композер нового комментария -->
  <div v-if="monoProfile" class="rounded-xl  bg-[var(--panel)] shadow-sm p-3">
  <textarea v-model="commentDraft" rows="2" :placeholder="t('comments.placeholder')" class="w-full resize-y rounded-md  bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)]  focus:outline-none"></textarea>
  <div class="mt-2 flex justify-end">
  <button type="button" class="rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-3 py-1 text-[13px] font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] disabled:opacity-50"
  :disabled="catalogCommentsBusy || !commentDraft.trim()"
  @click="sendCatalogComment(catalogDetail.id, commentDraft); commentDraft = ''">
  {{ t("comments.send") }}
  </button>
  </div>
  </div>
  <div v-else class="rounded-lg  border-dashed border-[var(--border)] bg-[var(--panel)] p-3 text-center text-[13px] text-[color:var(--tx-muted)]">
  {{ t("comments.needLogin") }}
  </div>

  <div v-if="!catalogCommentsBusy && catalogComments.length === 0" class="text-center py-8 text-[13px] text-[color:var(--tx-muted)]">{{ t("comments.empty") }}</div>

  <!-- Дерево: корень + 1 уровень ответов -->
  <div v-for="c in catalogComments" :key="c.id" class="space-y-2">
  <div class="rounded-xl  bg-[var(--panel)] shadow-sm p-3">
  <div class="flex items-center gap-2">
  <button type="button" class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full  bg-[var(--input)] font-mono text-xs font-bold text-[color:var(--tx-strong)] transition-colors hover:text-[var(--accent)]"
  @click="openProfileView(c.userId)">
  {{ c.user.username?.[0]?.toUpperCase() ?? "?" }}
  </button>
  <button type="button" class="font-mono text-[13px] font-semibold text-[color:var(--tx-strong)] hover:text-[var(--accent)] transition-colors" @click="openProfileView(c.userId)">
  {{ c.user.displayName || c.user.username }}
  </button>
  <span class="text-xs text-[color:var(--tx-muted)]">{{ formatDate(c.createdAt) }}</span>
  <div class="ml-auto flex shrink-0 items-center gap-1">
  <button type="button" class="flex items-center gap-1 rounded  px-1.5 py-0.5 text-xs font-semibold transition-colors"
  :class="c.myRating === 1 ? 'bg-[#3fb950]/15 text-[#3fb950]' : ' bg-[var(--input)] text-[color:var(--tx-muted)] hover:text-[#3fb950]'"
  :title="t('comments.like')"
  @click="rateCatalogComment(catalogDetail.id, c.id, 1)">
  👍 {{ c.likes }}
  </button>
  <button type="button" class="flex items-center gap-1 rounded  px-1.5 py-0.5 text-xs font-semibold transition-colors"
  :class="c.myRating === -1 ? 'bg-[#f85149]/15 text-[#f85149]' : ' bg-[var(--input)] text-[color:var(--tx-muted)] hover:text-[#f85149]'"
  :title="t('comments.dislike')"
  @click="rateCatalogComment(catalogDetail.id, c.id, -1)">
  👎 {{ c.dislikes }}
  </button>
  </div>
  </div>

  <!-- Редактирование своего комментария -->
  <template v-if="commentEditId === c.id">
  <textarea v-model="commentEditDraft" rows="2" class="mt-2 w-full resize-y rounded-md  bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)]  focus:outline-none"></textarea>
  <div class="mt-1.5 flex gap-2">
  <button type="button" class="rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-white disabled:opacity-50"
  :disabled="catalogCommentsBusy || !commentEditDraft.trim()"
  @click="editCatalogComment(catalogDetail.id, c.id, commentEditDraft); commentEditId = null">
  {{ t("author.save") }}
  </button>
  <button type="button" class="rounded-md  bg-[var(--input)] px-2.5 py-1 text-[13px] text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="commentEditId = null">
  {{ t("author.cancel") }}
  </button>
  </div>
  </template>
  <p v-else class="mt-1.5 text-[13px] leading-relaxed text-[color:var(--tx)] whitespace-pre-wrap">{{ c.body }}</p>

  <div class="mt-2 flex items-center gap-2">
  <button v-if="monoProfile && c.parentId === null" type="button" class="text-xs font-medium text-[var(--accent)] hover:underline" @click="commentReplyTo = commentReplyTo === c.id ? null : c.id; commentReplyDraft = ''">
  {{ commentReplyTo === c.id ? t("author.cancel") : t("comments.reply") }}
  </button>
  <button v-if="monoProfile?.uuid === c.userId" type="button" class="text-xs font-medium text-[color:var(--tx-muted)] hover:text-[var(--accent)]"
  @click="commentEditId = c.id; commentEditDraft = c.body">
  {{ t("comments.edit") }}
  </button>
  <button v-if="monoProfile?.uuid === c.userId || isAdmin" type="button" class="text-xs font-medium text-[#f87171] hover:underline" @click="removeCatalogComment(catalogDetail.id, c.id)">
  {{ t("author.delete") }}
  </button>
  </div>

  <!-- Форма ответа -->
  <div v-if="commentReplyTo === c.id" class="mt-2 rounded-md  bg-[var(--bg)] p-2">
  <textarea v-model="commentReplyDraft" rows="2" :placeholder="t('comments.placeholder')" class="w-full resize-y rounded-md  bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)]  focus:outline-none"></textarea>
  <div class="mt-1.5 flex justify-end gap-2">
  <button type="button" class="rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-white disabled:opacity-50"
  :disabled="catalogCommentsBusy || !commentReplyDraft.trim()"
  @click="sendCatalogComment(catalogDetail.id, commentReplyDraft, c.id); commentReplyTo = null; commentReplyDraft = ''">
  {{ t("comments.send") }}
  </button>
  </div>
  </div>
  </div>

  <!-- Ответы (1 уровень) -->
  <div v-for="r in c.replies" :key="r.id" class="ml-6 rounded-lg  bg-[var(--input-50)] p-3">
  <div class="flex items-center gap-2">
  <button type="button" class="flex h-5 w-5 shrink-0 items-center justify-center rounded-full  bg-[var(--input)] font-mono text-[11px] font-bold text-[color:var(--tx-strong)] transition-colors hover:text-[var(--accent)]"
  @click="openProfileView(r.userId)">
  {{ r.user.username?.[0]?.toUpperCase() ?? "?" }}
  </button>
  <button type="button" class="font-mono text-[13px] font-semibold text-[color:var(--tx-strong)] hover:text-[var(--accent)] transition-colors" @click="openProfileView(r.userId)">
  {{ r.user.displayName || r.user.username }}
  </button>
  <span class="text-xs text-[color:var(--tx-muted)]">{{ formatDate(r.createdAt) }}</span>
  <div class="ml-auto flex shrink-0 items-center gap-1">
  <button type="button" class="flex items-center gap-1 rounded  px-1.5 py-0.5 text-xs font-semibold transition-colors"
  :class="r.myRating === 1 ? 'bg-[#3fb950]/15 text-[#3fb950]' : ' bg-[var(--input)] text-[color:var(--tx-muted)] hover:text-[#3fb950]'"
  @click="rateCatalogComment(catalogDetail.id, r.id, 1)">
  👍 {{ r.likes }}
  </button>
  <button type="button" class="flex items-center gap-1 rounded  px-1.5 py-0.5 text-xs font-semibold transition-colors"
  :class="r.myRating === -1 ? 'bg-[#f85149]/15 text-[#f85149]' : ' bg-[var(--input)] text-[color:var(--tx-muted)] hover:text-[#f85149]'"
  @click="rateCatalogComment(catalogDetail.id, r.id, -1)">
  👎 {{ r.dislikes }}
  </button>
  </div>
  </div>
  <template v-if="commentEditId === r.id">
  <textarea v-model="commentEditDraft" rows="2" class="mt-2 w-full resize-y rounded-md  bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)]  focus:outline-none"></textarea>
  <div class="mt-1.5 flex gap-2">
  <button type="button" class="rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-white disabled:opacity-50"
  :disabled="catalogCommentsBusy || !commentEditDraft.trim()"
  @click="editCatalogComment(catalogDetail.id, r.id, commentEditDraft); commentEditId = null">
  {{ t("author.save") }}
  </button>
  <button type="button" class="rounded-md  bg-[var(--input)] px-2.5 py-1 text-[13px] text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="commentEditId = null">
  {{ t("author.cancel") }}
  </button>
  </div>
  </template>
  <p v-else class="mt-1.5 text-[13px] leading-relaxed text-[color:var(--tx)] whitespace-pre-wrap">{{ r.body }}</p>
  <div class="mt-2 flex items-center gap-2">
  <button v-if="monoProfile?.uuid === r.userId" type="button" class="text-xs font-medium text-[color:var(--tx-muted)] hover:text-[var(--accent)]"
  @click="commentEditId = r.id; commentEditDraft = r.body">
  {{ t("comments.edit") }}
  </button>
  <button v-if="monoProfile?.uuid === r.userId || isAdmin" type="button" class="text-xs font-medium text-[#f87171] hover:underline" @click="removeCatalogComment(catalogDetail.id, r.id)">
  {{ t("author.delete") }}
  </button>
  </div>
  </div>
   </div>
   </div>
   </div>
   </template>
   </div>
   </template>

  <!-- Catalog List View -->
  <template v-else>
  <div v-if="monoCatalogLoading && monoCatalog.length === 0" class="flex items-center justify-center py-16 text-[13px] text-[color:var(--tx-muted)]">
  <AppIcon name="spinner" class="mr-2 h-4 w-4 fill-current" />
  {{ t("catalog.loading") }}
  </div>
  <div v-else-if="monoCatalogError && monoCatalog.length === 0" class="rounded-xl  bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]">
  <p class="mb-3">{{ t("catalog.err", { e: monoCatalogError }) }}</p>
  <button type="button" class="text-[var(--accent)] hover:underline" @click="loadMonoCatalog">
  {{ t("catalog.retry") }}
  </button>
  </div>
  <div v-else-if="monoCatalog.length === 0" class="rounded-2xl border-2 border-dashed border-[var(--border)] p-10 text-center">
  <AppIcon name="plus" class="mx-auto h-8 w-8 fill-[var(--tx-muted)]" />
  <p class="mt-3 text-[13px] text-[color:var(--tx-muted)]">{{ t("catalog.emptyMono") }}</p>
  <button type="button" class="mt-4 px-4 py-2 rounded-xl bg-[var(--accent)] hover:brightness-110 text-white text-xs font-bold shadow-md active:scale-95 transition-all" @click="createPackOpen = true">
  {{ t("catalog.create") }}
  </button>
  </div>
  <div v-else class="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
  <article
  v-for="entry in monoCatalog"
  :key="entry.id"
  class="flex cursor-pointer flex-col overflow-hidden rounded-xl  bg-[var(--panel)] shadow-sm transition-all  hover:shadow-md"
  @click="openCatalogDetail(entry)"
  >
  <div class="flex flex-1 flex-col p-5">
  <div class="flex items-start justify-between gap-3">
  <div class="min-w-0">
  <div class="flex items-center gap-2.5">
  <img v-if="entry.icon_url" :src="entry.icon_url" :alt="entry.name" loading="lazy" @error="(e: any) => (e.target.style.display = 'none')" class="h-9 w-9 shrink-0 rounded-lg  object-cover" />
  <div v-else class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg  bg-[var(--input)] text-[13px] font-bold text-[var(--accent)]">{{ entry.name?.[0]?.toUpperCase() }}</div>
  <h3 class="truncate text-[15px] font-semibold text-[color:var(--tx-strong)]">{{ entry.name }}</h3>
  </div>
  <div v-if="entry.author_name" class="mt-1 font-mono text-[13px] text-[color:var(--tx-muted)]">
  @{{ entry.author_name }}
  </div>
  </div>
  <div class="flex shrink-0 flex-wrap items-center gap-1.5">
  <span
  v-if="entry.boosty_blog"
  class="inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-semibold"
  :class="isMonoPackAdded(entry) ? 'opacity-60' : ''"
  >
  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
  <path d="M7.75.5A4.5 4.5 0 0 1 11.5 5.5v.85A4.5 4.5 0 0 1 13 10v3A2.5 2.5 0 0 1 10.5 15.5h-6A2.5 2.5 0 0 1 2 13v-3a4.5 4.5 0 0 1 1.5-3.35V5.5A4.25 4.25 0 0 1 7.75.5Zm0 1.5a2.75 2.75 0 0 0-2.75 2.75v.5h5.5v-.5A2.75 2.75 0 0 0 7.75 2Z"/>
  </svg>
  {{ t("catalog.paid") }}
  </span>
  <span
  v-if="entry.min_ram_mb"
  class="rounded-full  px-2 py-0.5 text-xs font-medium text-[color:var(--tx-muted)]"
  >
  ≥ {{ entry.min_ram_mb / 1024 }} {{ t("units.gb") }}
  </span>
  </div>
  </div>
  <p v-if="entry.description" class="mt-3 min-h-0 flex-1 text-sm leading-relaxed text-[color:var(--tx-muted)] line-clamp-2">
  {{ entry.description }}
  </p>
  <div class="mt-3 flex flex-wrap items-center gap-1.5">
  <span v-if="entry.version" class="rounded-full  bg-[var(--input-50)] px-2 py-0.5 text-[13px] text-[color:var(--tx-muted)]">
  v{{ entry.version }}
  </span>
  <span v-if="entry.size" class="rounded-full  bg-[var(--input-50)] px-2 py-0.5 text-[13px] text-[color:var(--tx-muted)]">
  {{ formatBytes(entry.size) }}
  </span>
  <span v-if="entry.rating" class="inline-flex items-center gap-1 rounded-full  bg-[var(--input-50)] px-2 py-0.5 text-[13px] text-[color:var(--tx-muted)]">
  <AppIcon name="star" class="h-3 w-3 fill-[var(--accent)]" />
  {{ entry.rating }}
  <template v-if="entry.likes + entry.dislikes">({{ entry.likes }}👍/{{ entry.dislikes }}👎)</template>
  </span>
  </div>
  <div class="mt-4 flex items-center gap-2 border-t border-[var(--border)]  pt-3.5">
  <button
  type="button"
  v-if="!isMonoPackAdded(entry)"
  class="flex-1 rounded-xl bg-[var(--accent)] hover:brightness-110 px-3 py-2 text-xs font-semibold text-white shadow-sm active:scale-95 transition-all disabled:opacity-50"
  :disabled="addingPack"
  @click.stop="addMonoPack(entry)"
  >
  {{ addingPack ? t("dev.adding") : t("catalog.add") }}
  </button>
  <button
  type="button"
  v-else
  class="flex-1 rounded-lg  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click.stop="openMonoPack(entry)"
  >
  {{ t("catalog.open") }}
  </button>
  <button
  type="button"
  class="shrink-0 rounded-lg  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx-muted)] transition-colors hover:text-[var(--accent)]"
  :title="t('catalog.detailsHint')"
  @click.stop="openCatalogDetail(entry)"
  >
  <span class="inline-flex items-center gap-1.5">
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current">
  <path d="M8 1.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13ZM7.25 5.25a.75.75 0 1 1 1.5 0 .75.75 0 0 1-1.5 0Zm.5 2.25h.5a.75.75 0 0 1 .75.75v3a.75.75 0 0 1-1.5 0V8a.75.75 0 0 1 .75-.75Z" transform="translate(0 .5)"/>
  </svg>
  {{ t("catalog.details") }}
  </span>
  </button>
  <button
  type="button"
  class="shrink-0 rounded-lg  bg-[var(--input)] px-2.5 py-2 text-[13px] text-[color:var(--tx-muted)] transition-colors hover:text-[var(--accent)]"
  :title="entry.url"
  @click.stop="openExternal(entry.url)"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current">
  <path d="M3.75 2h3.5a.75.75 0 0 1 0 1.5h-2l6 6V7.5a.75.75 0 0 1 1.5 0v4.5a.75.75 0 0 1-.75.75H5.5a.75.75 0 0 1 0-1.5h2l-6-6v2a.75.75 0 0 1-1.5 0V3.5A1.75 1.75 0 0 1 1.75 1.75h2a.75.75 0 0 1 0 1.5Z"/>
  </svg>
  </button>
  </div>
  </div>
  </article>
  </div>
  </template>
  </template>

  <template v-else-if="catalogSource === 'modrinth'">
  <CatalogPackProfile v-if="profileOpen" @back="backToCatalogList" />
  <template v-else>
  <div class="mb-3 flex shrink-0 flex-wrap items-center gap-2">
  <FilterSelect
  v-model="packFilters.versions"
  :options="packVersionOptions"
  :placeholder="t('mods.fVersion')"
  @change="searchPacks()"
  />
  <FilterSelect
  v-model="packFilters.loaders"
  :options="packLoaderOptions"
  :placeholder="t('mods.fLoader')"
  @change="searchPacks()"
  />
  <FilterSelect
  v-model="packVersionTypeSel"
  :options="versionTypeOptions"
  :placeholder="t('mods.fType')"
  :multiple="false"
  @change="searchPacks()"
  />
  <FilterSelect
  v-model="packFilters.categories"
  :options="packCategoryOptions"
  :placeholder="t('mods.fCategory')"
  @change="searchPacks()"
  />
  <FilterSelect
  v-model="packEnvSel"
  :options="envOptions"
  :placeholder="t('mods.fAny')"
  :multiple="false"
  @change="searchPacks()"
  />
  <FilterSelect
  v-model="packSortSel"
  :options="sortSelectOptions"
  :placeholder="t('mods.fSort')"
  :multiple="false"
  @change="searchPacks()"
  />
  </div>
  <div class="mb-3 flex items-center gap-2">
  <div class="relative min-w-0 flex-1">
  <AppIcon name="search" class="pointer-events-none absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 fill-[var(--tx-muted)]" />
  <input
  v-model="modPackQuery"
  type="text"
  :placeholder="t('mods.packsPlaceholder')"
  class="h-9 w-full rounded-xl bg-[var(--input)] border border-[var(--border)] pl-8 pr-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none focus:border-[var(--accent)] transition-all"
  @keydown.enter="searchPacks"
  @input="debouncedSearchPacks"
  />
  </div>
  </div>
  <div v-if="modPackLoading" class="flex items-center justify-center py-16 text-[13px] text-[color:var(--tx-muted)]">
  <AppIcon name="spinner" class="mr-2 h-4 w-4 fill-current" />
  {{ t("mods.searchingAll") }}
  </div>
  <div v-else-if="modPackResults.length === 0" class="py-16 text-center text-[13px] text-[color:var(--tx-muted)]">
  {{ modPackQuery ? t("mods.noResults") : t("mods.packsHelp") }}
  </div>
  <div v-else class="grid grid-cols-1 gap-3 xl:grid-cols-2">
  <div
  v-for="p in modPackResults"
  :key="p.projectId"
  class="flex cursor-pointer items-center gap-3 rounded-xl bg-[var(--input)]/25 hover:bg-[var(--input)]/45 border border-[var(--border)] p-3 transition-all"
  @click="openCatalogModrinthDetail(p)"
  >
  <img v-if="p.iconUrl" :src="p.iconUrl" alt="" loading="lazy" class="h-12 w-12 shrink-0 rounded-xl object-cover" />
  <div v-else class="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-[var(--input)] text-sm font-bold text-[color:var(--tx-muted)]">
  {{ p.title.slice(0, 2).toUpperCase() }}
  </div>
  <div class="min-w-0 flex-1">
  <div class="flex flex-wrap items-center gap-x-2">
  <span class="truncate text-sm font-bold text-[color:var(--tx)]">{{ p.title }}</span>
  </div>
  <p class="truncate text-xs text-[color:var(--tx-muted)]">{{ t("mods.byAuthor", { author: p.author }) }}</p>
  <p class="mt-0.5 line-clamp-2 text-xs leading-snug text-[color:var(--tx-muted)]">{{ p.description }}</p>
  <p class="mt-1 flex items-center gap-2 text-xs text-[color:var(--tx-muted)]">
  <span class="flex items-center gap-1">
  <AppIcon name="download-bars" class="h-3 w-3 fill-current" />
  {{ compactDownloads(p.downloads) }}
  </span>
  </p>
  </div>
  <button
  type="button"
  class="flex shrink-0 items-center gap-1.5 self-center px-3.5 py-1.5 rounded-xl text-xs font-semibold bg-[var(--accent)] hover:brightness-110 text-white shadow-sm active:scale-95 transition-all disabled:opacity-50"
  :disabled="quickPackBusy !== null"
  :title="t('mods.downloadHint')"
  @click.stop="quickDownloadPack(p, $event)"
  >
   <AppIcon v-if="quickPackBusy === p.projectId" name="spinner" class="h-3 w-3 fill-current" />
   <AppIcon v-else name="arrow-down" class="h-3 w-3 fill-current" />
   {{ t("mods.download") }}
   </button>
   </div>
   </div>
   <div v-if="modPackMoreBusy" class="flex items-center justify-center py-4 text-[13px] text-[color:var(--tx-muted)]">
   <AppIcon name="spinner" class="mr-2 h-4 w-4 fill-current" />
   {{ t("mods.searchingAll") }}
   </div>
   </template>
   </template>

   <template v-else-if="catalogSource === 'curse'">
  <CatalogPackProfile v-if="profileOpen" @back="backToCatalogList" />
  <template v-else>
  <div class="mb-3 flex shrink-0 flex-wrap items-center gap-2">
  <FilterSelect
  v-model="cpCatSel"
  :options="cpCatOptions"
  :placeholder="t('curse.fCategory')"
  :multiple="false"
  @change="searchCursePacks"
  />
  <FilterSelect
  v-model="cpVerSel"
  :options="packVersionOptions"
  :placeholder="t('mods.fVersion')"
  :multiple="false"
  @change="searchCursePacks"
  />
  <FilterSelect
  v-model="cpSortSel"
  :options="curseSortOptions"
  :placeholder="t('mods.fSort')"
  :multiple="false"
  @change="searchCursePacks"
  />
  </div>
  <div v-if="!curseKeyOk" class="mb-3 rounded-xl  bg-[var(--panel)] shadow-sm p-4 text-center text-[13px] text-[color:var(--tx-muted)]">
  {{ t("curse.noKey") }}
  </div>
  <div class="mb-3 flex items-center gap-2">
  <div class="relative min-w-0 flex-1">
  <AppIcon name="search" class="pointer-events-none absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 fill-[var(--tx-muted)]" />
  <input
  v-model="modPackQuery"
  type="text"
  :placeholder="t('curse.packsPlaceholder')"
  class="h-9 w-full rounded-xl bg-[var(--input)] border border-[var(--border)] pl-8 pr-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none focus:border-[var(--accent)] transition-all"
  @keydown.enter="searchCursePacks"
  @input="debouncedSearchCursePacks"
  />
  </div>
  </div>
  <p v-if="!cpSearched" class="py-8 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("curse.packsHelp") }}</p>
  <p v-else-if="cpLoading" class="flex items-center justify-center gap-2 py-8 text-[13px] text-[color:var(--tx-muted)]">
  <AppIcon name="spinner" class="h-4 w-4 fill-current" />
  {{ t("mods.searchingAll") }}
  </p>
  <div v-else-if="cpErr" class="rounded-md  bg-[var(--input-50)] p-6 text-center text-[13px] text-[color:var(--tx-muted)]">
  <p class="mb-2 whitespace-pre-wrap">{{ cpErr }}</p>
  <button type="button" class="text-[var(--accent)] hover:underline" @click="searchCursePacks">{{ t("catalog.retry") }}</button>
  </div>
  <div v-else-if="cpResults.length === 0" class="py-16 text-center text-[13px] text-[color:var(--tx-muted)]">
  {{ modPackQuery ? t("mods.noResults") : t("curse.packsHelp") }}
  </div>
  <div v-else class="grid grid-cols-1 gap-3 xl:grid-cols-2">
  <div
  v-for="p in cpResults"
  :key="p.projectId"
  class="flex cursor-pointer items-center gap-3 rounded-xl bg-[var(--input)]/25 hover:bg-[var(--input)]/45 border border-[var(--border)] p-3 transition-all"
  @click="openCatalogCurseDetail(p)"
  >
  <img v-if="p.iconUrl" :src="p.iconUrl" alt="" loading="lazy" class="h-12 w-12 shrink-0 rounded-xl object-cover" />
  <div v-else class="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-[var(--input)] text-sm font-bold text-[color:var(--tx-muted)]">
  {{ p.name.slice(0, 2).toUpperCase() }}
  </div>
  <div class="min-w-0 flex-1">
  <span class="truncate block text-sm font-bold text-[color:var(--tx)]">{{ p.name }}</span>
  <p class="truncate text-xs text-[color:var(--tx-muted)]">{{ t("mods.byAuthor", { author: p.author }) }}</p>
  <p class="mt-0.5 line-clamp-2 text-xs leading-snug text-[color:var(--tx-muted)]">{{ p.summary }}</p>
  <p class="mt-1 flex items-center gap-1 text-xs text-[color:var(--tx-muted)]">
  <AppIcon name="download-bars" class="h-3 w-3 fill-current" />
  {{ compactDownloads(p.downloadCount) }}
  </p>
  </div>
  <button
  type="button"
  class="flex shrink-0 items-center gap-1.5 self-center px-3.5 py-1.5 rounded-xl text-xs font-semibold bg-[var(--accent)] hover:brightness-110 text-white shadow-sm active:scale-95 transition-all disabled:opacity-50"
  :disabled="quickCpBusy !== null"
  :title="t('mods.downloadHint')"
  @click.stop="quickDownloadCpPack(p, $event)"
  >
   <AppIcon v-if="quickCpBusy === p.projectId" name="spinner" class="h-3 w-3 fill-current" />
   <AppIcon v-else name="arrow-down" class="h-3 w-3 fill-current" />
   {{ t("mods.download") }}
   </button>
   </div>
   </div>
    <div v-if="cpMoreBusy" class="flex items-center justify-center py-4 text-[13px] text-[color:var(--tx-muted)]">
    <AppIcon name="spinner" class="mr-2 h-4 w-4 fill-current" />
    {{ t("mods.searchingAll") }}
    </div>
    </template>
    </template>
  </div>
  </div>
</template>
