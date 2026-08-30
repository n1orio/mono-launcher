<script setup lang="ts">
import { useLauncherCtx } from '~/composables/useLauncherContext';
const ctx = useLauncherCtx();
const { t, packs, filteredPacks, packsBySource, sidebarCat, toggleSidebarCat, libQuery, libScale, libTile, setLibScale, libPercent, libStatus, loadLibraryStatus, playLibraryPack, openPackTab, libMenuPack, libMenuPos, openLibMenu, closeLibMenu, libDoPlay, libCopyLink, libOpenSettings, openModPackModal, createPackOpen, customLibSections, libCats, packLibCats, togglePackCat, libCatModal, libCatName, openCatCreate, openCatRename, submitCatModal, packHasCat, makeCatId, createLibCat, renameLibCat, deleteLibCat, packId, convertFileSrc, busy, gameRunning, PACK_CATS, PACK_CAT_LABELS } = ctx;
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
  <div class="mb-5 flex shrink-0 items-center justify-between gap-4 border-b border-[var(--border)]  pb-4">
  <div>
  <h2 class="text-xl font-bold tracking-tight text-[color:var(--tx-strong)]">{{ t("nav.library") }}</h2>
  <p class="mt-1 text-[13px] text-[color:var(--tx-muted)]">{{ t("library.subtitle") }}</p>
  </div>
  <div class="flex shrink-0 items-center gap-1.5">
  <div class="relative">
  <input
  v-model="libQuery"
  type="text"
  class="w-44 rounded-md  bg-[var(--input)] px-2.5 py-1.5 pr-6 text-[13px] text-[color:var(--tx)] placeholder-[color:var(--tx-muted)]  focus:outline-none"
  :placeholder="t('library.search')"
  />
  <button
  v-if="libQuery"
  type="button"
  class="absolute right-1 top-1/2 -translate-y-1/2 rounded p-0.5 text-[color:var(--tx-muted)] hover:text-[color:var(--tx)]"
  @click="libQuery = ''"
  >
  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
  </button>
  </div>
  <button
  type="button"
  class="rounded-md  bg-[var(--input)] px-2 py-1.5 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  :title="t('library.newCat')"
  @click="openCatCreate()"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M8 3.25a.75.75 0 0 1 .75.75v3.25H12a.75.75 0 0 1 0 1.5H8.75V12a.75.75 0 0 1-1.5 0V8.75H4a.75.75 0 0 1 0-1.5h3.25V4a.75.75 0 0 1 .75-.75Z"/></svg>
  </button>
  <button
  type="button"
  class="rounded-md  bg-[var(--input)] p-1.5 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)] disabled:opacity-40"
  :title="t('library.zoomOut')"
  :disabled="libScale <= 1"
  @click="setLibScale(libScale - 1)"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3 8a.75.75 0 0 1 .75-.75h8.5a.75.75 0 0 1 0 1.5h-8.5A.75.75 0 0 1 3 8Z"/></svg>
  </button>
  <span class="w-11 text-center text-[13px] font-semibold tabular-nums text-[color:var(--tx-muted)]">{{ libPercent }}%</span>
  <button
  type="button"
  class="rounded-md  bg-[var(--input)] p-1.5 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)] disabled:opacity-40"
  :title="t('library.zoomIn')"
  :disabled="libScale >= 4"
  @click="setLibScale(libScale + 1)"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M8 3.25a.75.75 0 0 1 .75.75v3.25H12a.75.75 0 0 1 0 1.5H8.75V12a.75.75 0 0 1-1.5 0V8.75H4a.75.75 0 0 1 0-1.5h3.25V4a.75.75 0 0 1 .75-.75Z"/></svg>
  </button>
  </div>
  </div>
  <div class="min-h-0 flex-1 overflow-y-auto pr-1">
  <template v-for="cat in PACK_CATS" :key="cat">
  <section v-if="packsBySource[cat].length > 0" class="mb-6">
  <h3 class="mb-3 flex items-center gap-2 text-[13px] font-semibold uppercase tracking-wider text-[color:var(--tx-muted)]">
  <button
  type="button"
  class="flex h-4 w-4 shrink-0 items-center justify-center rounded transition-colors hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]"
  :title="t('library.toggleCat')"
  @click="toggleSidebarCat(cat)"
  >
   <AppIcon name="chevron-right" class="h-3 w-3 fill-current transition-transform" :class="sidebarCat[cat] ? 'rotate-90' : ''" />
  </button>
  {{ t(PACK_CAT_LABELS[cat]) }}
  <span class="rounded-full bg-[var(--input)] px-1.5 py-0.5 text-[11px] font-bold tabular-nums">{{ packsBySource[cat].length }}</span>
  </h3>
  <div v-if="sidebarCat[cat]" class="grid gap-3" :class="libTile.col">
  <div
  v-for="p in packsBySource[cat]"
  :key="p.id"
  class="flex aspect-square flex-col items-center justify-center gap-2 rounded-md  p-3 text-center transition-colors"
  :class="packId === p.id
  ? ' bg-[color-mix(in_srgb,var(--accent)_8%,transparent)]'
  : ' bg-[var(--panel)]  hover:bg-[var(--input-50)]'"
  @contextmenu.prevent="openLibMenu($event, p)"
  >
  <button
  type="button"
  class="flex w-full flex-col items-center justify-center gap-2"
  :title="p.name"
  @click="openPackTab(p.id)"
  >
  <img
  v-if="p.icon"
  :src="convertFileSrc(p.icon)"
  :alt="p.name"
  class="shrink-0 aspect-square rounded-none  object-cover"
  :class="libTile.icon"
  />
  <svg v-else viewBox="0 0 16 16" class="shrink-0 rounded-none fill-current text-[var(--tx-muted)]" :class="libTile.icon">
  <path d="M1 7.775V2.75C1 1.784 1.784 1 2.75 1h5.025c.464 0 .91.184 1.238.513l6.25 6.25a1.75 1.75 0 0 1 0 2.474l-5.026 5.026a1.75 1.75 0 0 1-2.474 0l-6.25-6.25A1.752 1.752 0 0 1 1 7.775Z"/>
  </svg>
  <span class="w-full min-w-0 truncate text-[13px] font-medium" :class="packId === p.id ? 'text-[var(--accent)]' : 'text-[color:var(--tx)]'">{{ p.name }}</span>
  </button>
  <button
  type="button"
  class="flex w-full items-center justify-center gap-1.5 rounded-md px-2.5 py-1.5 text-[13px] font-semibold text-white shadow-sm transition-colors"
  :class="libStatus[p.id]?.installed ? 'bg-[#238636] hover:bg-[#2ea043]' : 'bg-[var(--accent-deep)] hover:bg-[var(--accent-hover)]'"
  :disabled="busy || gameRunning"
  @click="playLibraryPack(p)"
  >
  <AppIcon name="play" class="h-4 w-4 fill-current" />
  {{ libStatus[p.id]?.installed ? t("side.play") : t("side.downloadPlay") }}
  </button>
  </div>
  </div>
  </section>
  </template>
  <template v-for="s in customLibSections" :key="s.cat.id">
  <section class="mb-6">
  <h3 class="mb-3 flex items-center gap-2 text-[13px] font-semibold uppercase tracking-wider text-[color:var(--tx-muted)]">
  <button
  type="button"
  class="flex h-4 w-4 shrink-0 items-center justify-center rounded transition-colors hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]"
  :title="t('library.toggleCat')"
  @click="toggleSidebarCat(s.cat.id)"
  >
   <AppIcon name="chevron-right" class="h-3 w-3 fill-current transition-transform" :class="sidebarCat[s.cat.id] ? 'rotate-90' : ''" />
  </button>
  {{ s.cat.name }}
  <span class="rounded-full bg-[var(--input)] px-1.5 py-0.5 text-[11px] font-bold tabular-nums">{{ s.packs.length }}</span>
  <span class="flex items-center gap-0.5">
  <button
  type="button"
  class="rounded p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]"
  :title="t('library.renameCat')"
  @click="openCatRename(s.cat.id)"
  >
   <AppIcon name="pencil" class="h-3 w-3 fill-current" />
  </button>
  <button
  type="button"
  class="rounded p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--input-50)] hover:text-red-400"
  :title="t('library.deleteCat')"
  @click="deleteLibCat(s.cat.id)"
  >
   <AppIcon name="trash" class="h-3 w-3 fill-current" />
  </button>
  </span>
  </h3>
  <div v-if="sidebarCat[s.cat.id]" class="grid gap-3" :class="libTile.col">
  <div
  v-for="p in s.packs"
  :key="p.id"
  class="flex aspect-square flex-col items-center justify-center gap-2 rounded-md  p-3 text-center transition-colors"
  :class="packId === p.id
  ? ' bg-[color-mix(in_srgb,var(--accent)_8%,transparent)]'
  : ' bg-[var(--panel)]  hover:bg-[var(--input-50)]'"
  @contextmenu.prevent="openLibMenu($event, p)"
  >
  <button
  type="button"
  class="flex w-full flex-col items-center justify-center gap-2"
  :title="p.name"
  @click="openPackTab(p.id)"
  >
  <img
  v-if="p.icon"
  :src="convertFileSrc(p.icon)"
  :alt="p.name"
  class="shrink-0 aspect-square rounded-none  object-cover"
  :class="libTile.icon"
  />
  <svg v-else viewBox="0 0 16 16" class="shrink-0 rounded-none fill-current text-[var(--tx-muted)]" :class="libTile.icon">
  <path d="M1 7.775V2.75C1 1.784 1.784 1 2.75 1h5.025c.464 0 .91.184 1.238.513l6.25 6.25a1.75 1.75 0 0 1 0 2.474l-5.026 5.026a1.75 1.75 0 0 1-2.474 0l-6.25-6.25A1.752 1.752 0 0 1 1 7.775Z"/>
  </svg>
  <span class="w-full min-w-0 truncate text-[13px] font-medium" :class="packId === p.id ? 'text-[var(--accent)]' : 'text-[color:var(--tx)]'">{{ p.name }}</span>
  </button>
  <button
  type="button"
  class="flex w-full items-center justify-center gap-1.5 rounded-md px-2.5 py-1.5 text-[13px] font-semibold text-white shadow-sm transition-colors"
  :class="libStatus[p.id]?.installed ? 'bg-[#238636] hover:bg-[#2ea043]' : 'bg-[var(--accent-deep)] hover:bg-[var(--accent-hover)]'"
  :disabled="busy || gameRunning"
  @click="playLibraryPack(p)"
  >
  <AppIcon name="play" class="h-4 w-4 fill-current" />
  {{ libStatus[p.id]?.installed ? t("side.play") : t("side.downloadPlay") }}
  </button>
  </div>
  </div>
  </section>
  </template>
  <div
  v-if="packs.length > 0 && filteredPacks.length === 0 && customLibSections.length === 0"
  class="rounded-xl  bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]"
  >
  {{ t("library.noSearch") }}
  </div>
  <div
  v-if="packs.length === 0"
  class="rounded-xl  bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]"
  >
  {{ t("library.empty") }}
  </div>
  </div>

  <!-- Контекстное меню: ПКМ по экземпляру в библиотеке -->
  <div
  v-if="libMenuPack && libMenuPos"
  class="fixed inset-0 z-[70]"
  @mousedown="closeLibMenu"
  @contextmenu.prevent="closeLibMenu"
  >
  <div
  class="fixed z-[71] w-56 overflow-hidden rounded-xl  bg-[var(--panel)] shadow-sm py-1 shadow-2xl"
  :style="{ left: `${libMenuPos.x}px`, top: `${libMenuPos.y}px` }"
  @mousedown.stop
  @contextmenu.stop
  >
  <div class="px-2.5 py-1.5">
  <div class="truncate text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ libMenuPack.name }}</div>
  <div class="truncate font-mono text-xs text-[color:var(--tx-muted)]">{{ libMenuPack.id }}</div>
  </div>
  <div class="mx-3 border-t border-[var(--border)] "></div>
  <button
  type="button"
  class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[13px] transition-colors hover:bg-[var(--hover)] disabled:opacity-50"
  :disabled="busy || gameRunning"
  @click="libDoPlay"
  >
   <AppIcon name="play" class="h-4 w-4 fill-current" />
   {{ libMenuPack && libStatus[libMenuPack.id]?.installed ? t("side.play") : t("side.downloadPlay") }}
  </button>
  <button
  v-if="libMenuPack?.url"
  type="button"
  class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click="libCopyLink"
  >
   <AppIcon name="link" class="h-4 w-4 fill-current" />
  {{ t("pack.copyLink") }}
  </button>
  <button
  type="button"
  class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click="libOpenSettings"
  >
   <AppIcon name="settings" class="h-4 w-4 fill-current" />
  {{ t("nav.settings") }}
  </button>
  <template v-if="libCats.length > 0">
  <div class="mx-3 border-t border-[var(--border)] "></div>
  <div class="px-3 pb-0.5 pt-1.5 text-[11px] font-semibold uppercase tracking-wider text-[color:var(--tx-muted)]">{{ t("library.catsTitle") }}</div>
  <div class="max-h-40 overflow-y-auto">
  <label
  v-for="c in libCats"
  :key="c.id"
  class="flex cursor-pointer items-center gap-2 px-3 py-1 text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  >
  <input
  type="checkbox"
  class="h-3 w-3 accent-[var(--accent)]"
  :checked="libMenuPack ? packHasCat(libMenuPack.id, c.id) : false"
  @change="libMenuPack && togglePackCat(libMenuPack.id, c.id)"
  />
  <span class="min-w-0 truncate">{{ c.name }}</span>
  </label>
  </div>
  </template>
  </div>
  </div>

  <!-- Модалка: создать/переименовать категорию -->
  <div
  v-if="libCatModal"
  class="fixed inset-0 z-[80] flex items-center justify-center bg-black/50 p-6"
  @mousedown.self="libCatModal = null"
  >
  <div class="w-full max-w-sm rounded-xl  bg-[var(--panel)] p-5 shadow-2xl" @keydown.escape="libCatModal = null">
  <h3 class="text-sm font-bold text-[color:var(--tx-strong)]">
  {{ libCatModal.mode === "create" ? t("library.newCat") : t("library.renameCat") }}
  </h3>
  <input
  v-model="libCatName"
  type="text"
  class="mt-3 w-full rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] text-[color:var(--tx)] placeholder-[color:var(--tx-muted)]  focus:outline-none"
  :placeholder="t('library.catName')"
  @keydown.enter.prevent="submitCatModal"
  />
  <div class="mt-4 flex justify-end gap-2">
  <button
  type="button"
  class="rounded-md  bg-[var(--input)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)]"
  @click="libCatModal = null"
  >
  {{ t("files.cancel") }}
  </button>
  <button
  type="button"
  class="rounded-md bg-[var(--accent-deep)] px-2.5 py-1.5 text-[13px] font-semibold text-white hover:bg-[var(--accent-hover)] disabled:opacity-50"
  :disabled="!libCatName.trim()"
  @click="submitCatModal"
  >
  {{ t("settings.save") }}
  </button>
  </div>
  </div>
  </div>
  </div>
</template>
