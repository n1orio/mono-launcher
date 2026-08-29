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
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current transition-transform" :class="sidebarCat[cat] ? 'rotate-90' : ''"><path d="M6 4l4 4-4 4V4Z"/></svg>
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
  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M4.5 1.94a1 1 0 0 1 1.523-.853l9.6 6.06a1 1 0 0 1 0 1.707l-9.6 6.06A1 1 0 0 1 4.5 14.06V1.94Z"/></svg>
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
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current transition-transform" :class="sidebarCat[s.cat.id] ? 'rotate-90' : ''"><path d="M6 4l4 4-4 4V4Z"/></svg>
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
  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M11.013 1.427a1.75 1.75 0 0 1 2.474 0l1.086 1.086a1.75 1.75 0 0 1 0 2.474l-8.61 8.61c-.21.21-.47.364-.756.445l-3.251.93a.75.75 0 0 1-.927-.928l.929-3.25a1.75 1.75 0 0 1 .445-.758l8.61-8.61Zm.176 4.82 1.658-1.659-1.085-1.085-1.66 1.657 1.087 1.087Z"/></svg>
  </button>
  <button
  type="button"
  class="rounded p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--input-50)] hover:text-red-400"
  :title="t('library.deleteCat')"
  @click="deleteLibCat(s.cat.id)"
  >
  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M11 1.75V3h2.25a.75.75 0 0 1 0 1.5H2.75a.75.75 0 0 1 0-1.5H5V1.75C5 .784 5.784 0 6.75 0h2.5C10.216 0 11 .784 11 1.75ZM4.496 6.675l.66 6.6a.25.25 0 0 0 .249.225h5.19a.25.25 0 0 0 .249-.225l.66-6.6a.75.75 0 0 1 1.492.15l-.66 6.6A1.748 1.748 0 0 1 10.595 15h-5.19a1.75 1.75 0 0 1-1.741-1.575l-.66-6.6a.75.75 0 1 1 1.492-.15ZM6.5 1.75V3h3V1.75a.25.25 0 0 0-.25-.25h-2.5a.25.25 0 0 0-.25.25Z"/></svg>
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
  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M4.5 1.94a1 1 0 0 1 1.523-.853l9.6 6.06a1 1 0 0 1 0 1.707l-9.6 6.06A1 1 0 0 1 4.5 14.06V1.94Z"/></svg>
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
  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M4.5 1.94a1 1 0 0 1 1.523-.853l9.6 6.06a1 1 0 0 1 0 1.707l-9.6 6.06A1 1 0 0 1 4.5 14.06V1.94Z"/></svg>
  {{ libMenuPack && libStatus[libMenuPack.id]?.installed ? t("side.play") : t("side.downloadPlay") }}
  </button>
  <button
  v-if="libMenuPack?.url"
  type="button"
  class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click="libCopyLink"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="m7.775 3.275a.75.75 0 0 0 1.06 1.06l1.25-1.25a2 2 0 1 1 2.83 2.83l-2.5 2.5a2 2 0 0 1-2.83 0 .75.75 0 0 0-1.06 1.06 3.5 3.5 0 0 0 4.95 0l2.5-2.5a3.5 3.5 0 0 0-4.95-4.95l-1.25 1.25Zm-4.69 9.64a2 2 0 0 1 0-2.83l2.5-2.5a2 2 0 0 1 2.83 0 .75.75 0 0 0 1.06-1.06 3.5 3.5 0 0 0-4.95 0l-2.5 2.5a3.5 3.5 0 0 0 4.95 4.95l1.25-1.25a.75.75 0 0 0-1.06-1.06l-1.25 1.25a2 2 0 0 1-2.83 0Z"/></svg>
  {{ t("pack.copyLink") }}
  </button>
  <button
  type="button"
  class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click="libOpenSettings"
  >
  <svg viewBox="0 0 24 24" class="h-4 w-4 shrink-0 fill-none stroke-current" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1Z"></path></svg>
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
