<script setup lang="ts">
import { useLauncherCtx } from '~/composables/useLauncherContext';
const ctx = useLauncherCtx();
const {
  t,
  activePack,
  activeBanner,
  bannerOk,
  loaderLabel,
  status,
  ram,
  maxRam,
  systemRam,
  busy,
  gameRunning,
  handleStop,
  handlePlay,
  handleInstall,
  handleOpenPackDir,
  copyPackDeepLink,
  convertFileSrc,
  formatPlaytime,
  formatPlaytimeShort,
  formatUnixDate,
  formatDate,
  formatBytes,
  cap,
  openExternal,
  openFolder,
  openEditVersion,
  reportPackBug,
  exportMenuRef,
  exportBusy,
  exportMenuOpen,
  openExport,
  openAuthorExport,
  activePackRepo,
  updateInfo,
  handleUpdate,
  licenseInfo,
  licenseBusy,
  removeLicense,
  boostyAuthOpen,
  startBoostyLogin,
  cancelBoostyLogin,
  licenseKeyInput,
  saveLicense,
  warnCustomMods,
  customModsOpen,
  customScanBusy,
  scanActiveCustomMods,
  customBannerClass,
  customBannerNoteClass,
  customBannerState,
  playSubTabsVisible,
  playSubTab,
  subTabCount,
  versions,
  remoteVersions,
  remoteVersionsLoading,
  remoteInstallingId,
  refreshRemoteVersions,
  installRemoteVersion,
  packLocked,
  unbindArmed,
  confirmUnbindPack,
  packId,
  fileVisibleCount,
  enabledCountIn,
  modUpdatesTab,
  updateAllBusy,
  updatingMod,
  updateAllMods,
  openSearch,
  openModScanner,
  selectedFiles,
  setSelectedFilesEnabled,
  openSelected,
  clearFileSelection,
  fileDeleteArmed,
  fileDeleteBusy,
  deleteSelectedFiles,
  fileSortKey,
  fileSortDir,
  toggleFileSort,
  clearFileSort,
  fileStatusFilter,
  setFileStatusFilter,
  fileSearch,
  fileMenuRef,
  fileMenuOpen,
  gameFiles,
  fileListRef,
  fileListScroll,
  fileListTotal,
  fileListStart,
  fileRowStride,
  fileListVisible,
  fileListFiltered,
  isFileSelected,
  toggleFileSelect,
  isFileToggling,
  handleToggleFile,
  modrinthMetaFor,
  curseMetaFor,
  gameFileIcon,
  fileMetaTitle,
  modrinthVersionFor,
  modUpdateFor,
  openFileOnCurseForge,
  openFileOnModrinth,
  openFileDetail,
  updateOneMod,
  duplicatesLoading,
  duplicates,
  keepOne,
  removeDuplicate,
  screenshotsLoading,
  packScreenshotsInstalled,
  packScreenshots,
  shotIdx,
  serverGroups,
  serverKey,
  serverStateOf,
  serverPlayersOf,
  serverStatusText,
  serverStatuses,
  copyServerIp,
  playOnServer,
  jvmArgs,
  jvmArgsSaving,
  saveJvmArgs,
  windowWidth,
  windowHeight,
  javaSelected,
  javaBusy,
  javaList,
  javaArchLabel,
  onJavaChange,
  javaMsg,
  downloadJava,
  discordRp,
  toggleDiscordRp,
  closeToTray,
  toggleCloseToTray,
  autostartOn,
  toggleAutostart,
  toggleWarnCustomMods,
  verifyBusy,
  handleVerify,
  verifyResult,
  logEntries,
  logRef,
  handleCopyLog,
  handleClearLog,
  fileDetail,
  fileDetailMr,
  fileDetailCf,
  fileDetailMrLoading,
  fileDetailCfLoading,
  updatingFileDetail,
  updateFileDetail,
  fileDetailTabs,
  fileDetailTab,
  fileDetailMcSel,
  fileDetailMcOptions,
  fileDetailLoaderSel,
  fileDetailLoaderOptions,
  fileDetailTypeSel,
  versionTypeOptions,
  fileDetailMrVersions,
  fileDetailFilteredVersions,
  fileDetailMrVersionBusy,
  installFileDetailVersion,
  fileDetailInstalledVersion,
  verTypeColor,
  verInstallSize,
  fileDetailExternalUrl,
  licenseError,
  handleSelectVersion,
  selectAllFiles,
} = ctx;
import type { GameFolderKind, ModrinthSearchKind } from "~/lib/bridge";
import type { GameFileEntry } from "~/lib/types";

// ---- Контекстное меню ПКМ по файлу ----
const fileCtx = ref<{ file: GameFileEntry; x: number; y: number } | null>(null);
function openFileCtx(e: MouseEvent, f: GameFileEntry) {
  e.preventDefault();
  fileCtx.value = { file: f, x: e.clientX, y: e.clientY };
}
function closeFileCtx() { fileCtx.value = null; }
const fileCtxStyle = computed(() => {
  if (!fileCtx.value) return {};
  return { left: `${Math.min(fileCtx.value.x, window.innerWidth - 240)}px`, top: `${Math.min(fileCtx.value.y, window.innerHeight - 320)}px` };
});

async function enableAllFiles(enabled: boolean) {
  selectAllFiles(playSubTab.value as GameFolderKind, fileListFiltered.value);
  await nextTick();
  await setSelectedFilesEnabled(enabled);
  clearFileSelection();
}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
  <!-- Header сборки -->
  <div class="mb-6 shrink-0 border-b border-[var(--border)]  pb-5">
  <div v-if="activeBanner && bannerOk" class="relative mb-4 h-44 w-full overflow-hidden rounded-xl ">
  <img
  :src="activeBanner"
  :alt="activePack?.name ?? ''"
  class="h-full w-full object-cover"
  @error="bannerOk = false"
  />
    </div>
  <div class="flex flex-wrap items-end justify-between gap-x-4 gap-y-3" :class="activeBanner && bannerOk ? '-mt-7 px-4' : ''">
  <!-- Левая часть: иконка сборки + название + мета -->
  <div class="flex min-w-0 flex-1 items-end gap-4">
  <img
  v-if="activePack?.icon"
  :src="convertFileSrc(activePack.icon)"
  :alt="activePack.name"
  class="h-[60px] w-[60px] shrink-0 rounded-xl  bg-[var(--panel)] object-cover shadow-lg"
  @error="(e: any) => (e.target.style.display = 'none')"
  />
  <div v-else class="flex h-[60px] w-[60px] shrink-0 items-center justify-center rounded-xl  bg-[var(--panel)] shadow-lg">
  <svg viewBox="0 0 16 16" class="h-6 w-6 fill-[var(--tx-muted)]">
  <path d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-1 1v.878A2.25 2.25 0 1 1 2 13.378V2.5Z"/>
  </svg>
  </div>
  <div class="min-w-0 pb-1">
  <h1 class="truncate text-3xl font-bold leading-tight tracking-tight text-[color:var(--tx-strong)]">
  {{ activePack?.name ?? t("pack.none") }}
  </h1>
  <p v-if="activePack?.author || loaderLabel" class="mt-1 flex flex-wrap items-center gap-x-2.5 gap-y-0.5 text-[13px] text-[color:var(--tx-muted)]">
  <span v-if="activePack?.author" class="font-mono font-medium text-[var(--accent)]">@{{ activePack.author }}</span>
  <span v-if="activePack?.author && loaderLabel" class="opacity-40">·</span>
  <span v-if="loaderLabel">{{ loaderLabel }}</span>
  </p>
  <div class="mt-2.5 flex flex-wrap items-center gap-1.5">
  <span
  class="rounded-full px-2.5 py-0.5 text-xs font-bold uppercase tracking-wider  shadow-sm"
  :class="status?.installed
  ? 'bg-[#238636]/10 text-[#3fb950]'
  : ' bg-[var(--input)] text-[color:var(--tx-muted)]'"
  >
  {{ status?.installed ? t("pack.installed") : t("pack.notInstalled") }}
  </span>
  <span
  v-if="activePack?.minRam"
  class="inline-flex items-center gap-1 rounded-full  px-2 py-0.5 text-[13px] font-semibold"
  :class="(ram * 1024) < activePack.minRam
  ? 'bg-[#f0883e]/10 text-[#f0883e]'
  : ' bg-[var(--input)] text-[color:var(--tx-muted)]'"
  :title="t('pack.minRamTitle', { min: activePack.minRam / 1024 })"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current">
  <path d="M1 3.75C1 2.784 1.784 2 2.75 2h10.5c.966 0 1.75.784 1.75 1.75v5.5A1.75 1.75 0 0 1 13.25 11H10v1.25h.75a.75.75 0 0 1 0 1.5h-5.5a.75.75 0 0 1 0-1.5H6V11H2.75A1.75 1.75 0 0 1 1 9.25v-5.5Zm1.5 0v5.5c0 .138.112.25.25.25h10.5a.25.25 0 0 0 .25-.25v-5.5a.25.25 0 0 0-.25-.25H2.75a.25.25 0 0 0-.25.25ZM4 4.5a.75.75 0 0 1 .75-.75h6.5a.75.75 0 0 1 0 1.5h-6.5A.75.75 0 0 1 4 4.5Zm0 3a.75.75 0 0 1 .75-.75h6.5a.75.75 0 0 1 0 1.5h-6.5A.75.75 0 0 1 4 7.5Z"/>
  </svg>
  ≥ {{ activePack.minRam / 1024 }} {{ t("units.gb") }}
  </span>
  <span
  v-if="status && status.playtime_seconds > 0"
  class="inline-flex items-center gap-1 rounded-full  bg-[var(--input)] px-2 py-0.5 text-[13px] font-semibold text-[color:var(--tx-muted)]"
  :title="t('pack.playtimeTitle', { time: formatPlaytime(status.playtime_seconds) })"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current">
  <path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0Zm0 1.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13ZM7.25 3.5a.75.75 0 0 1 .75.75V7.8l2.58 1.55a.75.75 0 1 1-.77 1.28L7.18 9.1a.75.75 0 0 1-.43-.68V4.25a.75.75 0 0 1 .75-.75Z"/>
  </svg>
  {{ formatPlaytimeShort(status.playtime_seconds) }}
  </span>
  <span
  v-else-if="status && status.installed"
  class="inline-flex items-center gap-1 rounded-full  border-dashed border-[var(--border)] bg-[var(--panel-soft)] px-2 py-0.5 text-[13px] font-medium text-[color:var(--tx-muted)]"
  :title="t('pack.notPlayedTitle')"
  >
  {{ t("pack.notPlayed") }}
  </span>
  </div>
  </div>
  </div>

  <!-- Правая часть: главное действие + вторичные кнопки -->
  <div class="flex shrink-0 flex-col items-end gap-2 pb-1">
  <button
  type="button"
  class="flex items-center justify-center gap-2 rounded-xl px-6 py-2.5 text-sm font-bold tracking-wide text-white shadow-md transition-all active:scale-[0.98] focus-visible:outline focus-visible:outline-offset-2 disabled:cursor-not-allowed disabled:opacity-50 disabled:active:scale-100"
  :class="status?.installed
  ? gameRunning
  ? 'bg-[#b91c1c] hover:bg-[#dc2626]'
  : 'bg-[#238636] hover:bg-[#2ea043]'
  : 'bg-[var(--accent-deep)] hover:bg-[var(--accent-hover)]'"
  :disabled="busy"
  @click="status?.installed ? (gameRunning ? handleStop() : handlePlay()) : handleInstall()"
  >
  <svg v-if="busy" viewBox="0 0 16 16" class="h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
  <svg v-else-if="status?.installed && !gameRunning" viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M4.5 1.94a1 1 0 0 1 1.523-.853l9.6 6.06a1 1 0 0 1 0 1.707l-9.6 6.06A1 1 0 0 1 4.5 14.06V1.94Z"/></svg>
  <svg v-else-if="gameRunning" viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.5 3.5h9v9h-9z"/></svg>
  <svg v-else viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M2.75 14A1.75 1.75 0 0 1 1 12.25v-2.5a.75.75 0 0 1 1.5 0v2.5c0 .138.112.25.25.25h10.5a.25.25 0 0 0 .25-.25v-2.5a.75.75 0 0 1 1.5 0v2.5A1.75 1.75 0 0 1 13.25 14Z"/><path d="M7.97 2.72a.75.75 0 0 1 1.06 0l3 3a.75.75 0 1 1-1.06 1.06l-1.72-1.72v6.69a.75.75 0 0 1-1.5 0v-6.69L6.03 6.78a.75.75 0 0 1-1.06-1.06l3-3Z"/></svg>
  <template v-if="!status?.installed">{{ busy ? t("side.installing") : t("side.downloadPlay") }}</template>
  <template v-else>{{ busy ? t("side.launching") : gameRunning ? t("side.stopGame") : t("side.play") }}</template>
  </button>
  <div class="flex items-center gap-1.5">
  <button
  type="button"
  class="flex items-center gap-1.5 rounded-lg  bg-[var(--input)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)]"
  :title="t('pack.openDir')"
  @click="handleOpenPackDir"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current">
  <path d="M.513 1.513A1.75 1.75 0 0 1 1.75 1h3.5c.55 0 1.07.26 1.4.7l.9 1.2a.25.25 0 0 0 .2.1H13a1 1 0 0 1 1 1v.5H2.75a.75.75 0 0 0 0 1.5h11.978a1 1 0 0 1 .994 1.117L15 13.25A1.75 1.75 0 0 1 13.25 15H1.75A1.75 1.75 0 0 1 0 13.25V2.75c0-.464.184-.91.513-1.237Z"/>
  </svg>
  {{ t("pack.folder") }}
  </button>
  <button
  v-if="activePack?.url"
  type="button"
  class="flex items-center gap-1.5 rounded-lg  bg-[var(--input)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)]"
  :title="t('pack.copyLink')"
  @click="copyPackDeepLink(activePack)"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current">
  <path d="m7.775 3.275a.75.75 0 0 0 1.06 1.06l1.25-1.25a2 2 0 1 1 2.83 2.83l-2.5 2.5a2 2 0 0 1-2.83 0 .75.75 0 0 0-1.06 1.06 3.5 3.5 0 0 0 4.95 0l2.5-2.5a3.5 3.5 0 0 0-4.95-4.95l-1.25 1.25Zm-4.69 9.64a2 2 0 0 1 0-2.83l2.5-2.5a2 2 0 0 1 2.83 0 .75.75 0 0 0 1.06-1.06 3.5 3.5 0 0 0-4.95 0l-2.5 2.5a3.5 3.5 0 0 0 4.95 4.95l1.25-1.25a.75.75 0 0 0-1.06-1.06l-1.25 1.25a2 2 0 0 1-2.83 0Z"/>
  </svg>
  {{ t("pack.copyLink") }}
  </button>
  <template v-if="activePack?.kind === 'local' && status?.installed">
  <div ref="exportMenuRef" class="relative">
  <button
  type="button"
  class="flex items-center gap-1.5 rounded-lg  bg-[var(--input)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)]"
  :title="t('pack.exportTitle')"
  :disabled="exportBusy"
  @click="exportMenuOpen = !exportMenuOpen"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current">
  <path d="M7.97.72a.75.75 0 0 1 1.06 0l3 3a.75.75 0 1 1-1.06 1.06L9 2.81v6.94a.75.75 0 0 1-1.5 0V2.81L5.53 4.78a.75.75 0 0 1-1.06-1.06l3-3Z"/>
  <path d="M2.5 13.25a.75.75 0 0 1 .75.75c0 .138.112.25.25.25h9a.25.25 0 0 0 .25-.25.75.75 0 0 1 1.5 0 1.75 1.75 0 0 1-1.75 1.75h-9A1.75 1.75 0 0 1 1.75 14a.75.75 0 0 1 .75-.75Z"/>
  </svg>
  <span>{{ t("pack.exportBtn") }}</span>
  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current opacity-60"><path d="m4.22 6 3.72 3.72a.75.75 0 0 0 1.06 0L12.72 6l-1.06-1.06L8 8.09 5.28 4.94 4.22 6Z"/></svg>
  </button>
  <div
  v-if="exportMenuOpen"
  class="absolute right-0 top-[calc(100%+4px)] z-50 flex w-44 flex-col overflow-hidden rounded-xl  bg-[var(--panel)] shadow-sm p-1 shadow-xl"
  >
  <button
  type="button"
  class="flex items-center gap-2 rounded px-2 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  :disabled="exportBusy"
  @click="exportMenuOpen = false; openExport('mrpack')"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current opacity-70"><path d="M8 1.5A2.75 2.75 0 0 0 5.5 3.25a.75.75 0 0 1-1.5 0A4.25 4.25 0 0 1 9 1.075 4.25 4.25 0 0 1 13.2 4.5a.75.75 0 0 1-1.47.27A2.751 2.751 0 0 0 8 1.5Zm-4.5 8a2.75 2.75 0 0 1 2.5-1.75h.22a.75.75 0 0 0 .71-.51A3.75 3.75 0 0 1 8 5.25a3.75 3.75 0 0 1 1.07 1.99.75.75 0 0 0 .71.51h.22A2.75 2.75 0 0 1 12.5 9.5 2.75 2.75 0 0 1 9.75 12.25h-3.5A2.75 2.75 0 0 1 3.5 9.5Z"/><path d="M8 7.25a.75.75 0 0 1 .75.75v4.19l.97-.97a.75.75 0 1 1 1.06 1.06l-2.25 2.25a.75.75 0 0 1-1.06 0l-2.25-2.25a.75.75 0 1 1 1.06-1.06l.97.97V8a.75.75 0 0 1 .75-.75Z"/></svg>
  .mrpack
  </button>
  <button
  type="button"
  class="flex items-center gap-2 rounded px-2 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  :disabled="exportBusy"
  @click="exportMenuOpen = false; openAuthorExport()"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current opacity-70"><path d="M7.25 1.75a.75.75 0 0 1 1.5 0v5.5h5.5a.75.75 0 0 1 0 1.5h-5.5v5.5a.75.75 0 0 1-1.5 0v-5.5h-5.5a.75.75 0 0 1 0-1.5h5.5v-5.5Z"/></svg>
  {{ t("pack.exportAuthorShort") }}
  </button>
  <button
  type="button"
  class="flex items-center gap-2 rounded px-2 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  :disabled="exportBusy"
  @click="exportMenuOpen = false; openExport('curseforge')"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current opacity-70"><path d="M8 1.5A2.75 2.75 0 0 0 5.5 3.25a.75.75 0 0 1-1.5 0A4.25 4.25 0 0 1 9 1.075 4.25 4.25 0 0 1 13.2 4.5a.75.75 0 0 1-1.47.27A2.751 2.751 0 0 0 8 1.5Zm-4.5 8a2.75 2.75 0 0 1 2.5-1.75h.22a.75.75 0 0 0 .71-.51A3.75 3.75 0 0 1 8 5.25a3.75 3.75 0 0 1 1.07 1.99.75.75 0 0 0 .71.51h.22A2.75 2.75 0 0 1 12.5 9.5 2.75 2.75 0 0 1 9.75 12.25h-3.5A2.75 2.75 0 0 1 3.5 9.5Z"/><path d="M8 7.25a.75.75 0 0 1 .75.75v4.19l.97-.97a.75.75 0 1 1 1.06 1.06l-2.25 2.25a.75.75 0 0 1-1.06 0l-2.25-2.25a.75.75 0 1 1 1.06-1.06l.97.97V8a.75.75 0 0 1 .75-.75Z"/></svg>
  CurseForge
  </button>
  </div>
  </div>
  </template>
  </div>
  </div>
  </div>

  <p class="mt-2 text-[13px] text-[color:var(--tx-muted)] flex items-center gap-2">
  <span>{{ t("pack.mono") }}</span>
  <span>•</span>
  <span v-if="loaderLabel">{{ t("pack.loader", { name: loaderLabel }) }}</span>
  <button
  v-if="activePack?.kind === 'local'"
  type="button"
  class="inline-flex items-center gap-1 rounded  bg-[var(--input)] px-1.5 py-0.5 text-xs font-medium text-[var(--accent)] transition-colors  hover:bg-[var(--hover)]"
  :title="t('pack.versionChange')"
  @click="openEditVersion"
  >
  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M11.013 1.427a1.75 1.75 0 0 1 2.474 0l1.086 1.086a1.75 1.75 0 0 1 0 2.474l-8.61 8.61c-.21.21-.47.364-.756.445l-3.251 1.302a.75.75 0 0 1-.993-.993l1.302-3.251a1.75 1.75 0 0 1 .445-.756l8.61-8.61Z"/></svg>
  {{ t("pack.versionChange") }}
  </button>
  <span v-if="activePack?.author">•</span>
  <span v-if="activePack?.author" class="font-mono text-[var(--accent)]">@{{ activePack.author }}</span>
  <button
  v-if="activePackRepo"
  type="button"
  class="inline-flex items-center gap-1.5 rounded-md  bg-[var(--input)] px-2.5 py-1 text-[13px] font-medium text-[color:var(--tx)] transition-colors  hover:text-[var(--accent)]"
  :title="activePackRepo"
  @click="openExternal(activePackRepo)"
  >
  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
  <path d="M6.75 2.75h2.5a.75.75 0 0 1 0 1.5h-1.72l4.29 4.29a.75.75 0 0 1-1.06 1.06L6.47 5.31v1.69a.75.75 0 0 1-1.5 0v-3.5a.75.75 0 0 1 .75-.75Z"/>
  <path d="M2.25 5.75A1.75 1.75 0 0 1 4 4h2.75a.75.75 0 0 1 0 1.5H4v6.5h6.5v-2.5a.75.75 0 0 1 1.5 0V11A1.75 1.75 0 0 1 10.25 12.75H4A1.75 1.75 0 0 1 2.25 11V5.75Z"/>
  <path d="M11.75 7.25a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 .75.75v1.5a.75.75 0 0 1-1.5 0V7.75h-1.5a.75.75 0 0 1-.75-.75Z"/>
  </svg>
  {{ t("pack.repo") }}
  </button>
  <button
  v-if="activePackRepo"
  type="button"
  class="inline-flex items-center gap-1.5 rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_10%,transparent)] px-2.5 py-1 text-[13px] font-medium text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)]"
  :title="t('pack.reportBugTitle')"
  @click="reportPackBug()"
  >
  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
  <path d="M8 1c-1.04 0-1.9.81-2 1.84-.1-.06-.21-.11-.32-.16l-.12-.05a1.75 1.75 0 0 0-1.3 3.24c-.5.6-.8 1.36-.8 2.2v.6h.75a.75.75 0 0 1 0 1.5H3a.75.75 0 0 1 0-1.5h.3v-.6a2.6 2.6 0 0 0-.2-1 .75.75 0 0 1 .9-1c.3.2.57.4.8.65V5.8c0-.33.05-.65.16-.95.07.71.63 1.29 1.34 1.38L6.2 6.2c.44.15.86.34 1.24.58.44.28.81.58 1.12.9.47.5.94 1.15 1.44 1.94.2.32.3.69.3 1.07v1.31c.48.1.94.3 1.33.58.31.22.7.32 1.07.28a.9.9 0 1 1 .1 1.8c-.8.08-1.59-.22-2.22-.76-.43-.36-.8-.56-1.14-.61v.69c0 .64-.19 1.24-.52 1.74.8.37 1.3 1.18 1.3 2.16 0 .55-.45 1-1 1H8.25c-.55 0-1-.45-1-1s.45-1 1-1H9v-2.32c-.26.2-.55.35-.87.45-.46.14-.96.14-1.42 0a2.77 2.77 0 0 1-.71-.32V15.5c0 .55-.45 1-1 1H3.75c-.55 0-1-.45-1-1s.45-1 1-1h.61c-.58-.62-1-1.09-1.3-1.42l-.18-.17a2.25 2.25 0 0 1-1.68-2.19v-5.8c0-1.14.84-2.08 1.92-2.26A2 2 0 0 1 3.96.88l.03-.02A2 2 0 0 1 6.09 1H8Z"/>
  </svg>
  {{ t("pack.reportBug") }}
  </button>
  </p>

  <div v-if="updateInfo?.has_update && updateInfo.latest_version" class="mt-4 flex items-center justify-between gap-4 rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_10%,transparent)] px-3.5 py-2.5 text-[13px] text-[var(--accent)]">
  <span class="min-w-0">
  {{ t("update.available") }} <strong class="text-[var(--accent-strong)]">{{ updateInfo.latest_version }}</strong>
  <span v-if="updateInfo.current_version" class="text-[color:var(--tx-muted)]">
  {{ t("update.installed", { v: updateInfo.current_version }) }}
  </span>
  </span>
  <button
  type="button"
  class="shrink-0 rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] disabled:opacity-50"
  :disabled="busy"
  @click="handleUpdate"
  >
  {{ t("update.btn") }}
  </button>
  </div>

  <!-- Подписка Boosty: статус/привязка токена -->
  <div
  v-if="activePack?.boostyBlog"
  class="mt-4 rounded-md  px-3.5 py-2.5 text-[13px]"
  :class="licenseInfo?.subscribed
  ? 'bg-[#238636]/10 text-[#3fb950]'
  : ' bg-[color-mix(in_srgb,var(--accent-deep)_10%,transparent)]'"
  >
  <div v-if="licenseInfo?.subscribed" class="flex items-center justify-between gap-3">
  <span class="flex min-w-0 items-center gap-2">
  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current">
  <path d="M7.75.5A4.5 4.5 0 0 1 11.5 5.5v.85A4.5 4.5 0 0 1 13 10v3A2.5 2.5 0 0 1 10.5 15.5h-6A2.5 2.5 0 0 1 2 13v-3a4.5 4.5 0 0 1 1.5-3.35V5.5A4.25 4.25 0 0 1 7.75.5Zm0 1.5a2.75 2.75 0 0 0-2.75 2.75v.5h5.5v-.5A2.75 2.75 0 0 0 7.75 2Z"/>
  </svg>
  <span class="min-w-0">
  {{
  licenseInfo.expiresAt
  ? t("license.active", {
  blog: licenseInfo.blog,
  until: formatUnixDate(licenseInfo.expiresAt),
  })
  : t("license.activeNoExpiry", { blog: licenseInfo.blog })
  }}
  </span>
  </span>
  <button
  type="button"
  class="shrink-0 rounded-md  bg-[var(--input)] px-2.5 py-1 text-[13px] font-medium text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)] disabled:opacity-50"
  :disabled="licenseBusy"
  @click="removeLicense"
  >
  {{ t("license.remove") }}
  </button>
  </div>
  <div
  v-if="licenseInfo?.requiredTiers.length"
  class="mt-1.5 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-[13px] text-[color:var(--tx-muted)]"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current">
  <path d="M4.5 7.5a3.5 3.5 0 1 1 7 0v1h.75A.75.75 0 0 1 13 9.25v4A.75.75 0 0 1 12.25 14h-8.5a.75.75 0 0 1-.75-.75v-4A.75.75 0 0 1 3.75 8.5h.75v-1Zm1.5 1v-1a2 2 0 0 1 4 0v1h-4Z"/>
  </svg>
  <span>
  {{
  licenseInfo.tier
  ? t("license.tierOk", {
  tier: licenseInfo.tier,
  list: licenseInfo.requiredTiers.join(" / "),
  })
  : t("license.tierList", { list: licenseInfo.requiredTiers.join(" / ") })
  }}
  </span>
  </div>
  <template v-else>
  <div class="flex items-center gap-2">
  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current">
  <path d="M7.75.5A4.5 4.5 0 0 1 11.5 5.5v.85A4.5 4.5 0 0 1 13 10v3A2.5 2.5 0 0 1 10.5 15.5h-6A2.5 2.5 0 0 1 2 13v-3a4.5 4.5 0 0 1 1.5-3.35V5.5A4.25 4.25 0 0 1 7.75.5Zm0 1.5a2.75 2.75 0 0 0-2.75 2.75v.5h5.5v-.5A2.75 2.75 0 0 0 7.75 2Z"/>
  </svg>
  <span class="min-w-0">
  {{
  licenseInfo?.requiredTiers.length
  ? t("license.requiredTier", {
  blog: activePack.boostyBlog,
  list: licenseInfo.requiredTiers.join(" / "),
  })
  : t("license.required", { blog: activePack.boostyBlog })
  }}
  </span>
  </div>
  <div v-if="licenseError" class="mt-1.5 text-[color:var(--tx-muted)]">
  {{ licenseError }}
  </div>
  <div class="mt-2">
  <button
  v-if="!boostyAuthOpen"
  type="button"
  class="w-full rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] disabled:opacity-50"
  :disabled="licenseBusy"
  @click="startBoostyLogin()"
  >
  {{ t("license.oauth") }}
  </button>
  <div
  v-else
  class="flex items-center justify-between gap-3 rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] text-[color:var(--tx-muted)]"
  >
  <span class="flex items-center gap-2">
  <svg class="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 0 1 8-8v3a5 5 0 0 0-5 5H4z"/>
  </svg>
  {{ t("license.waiting") }}
  </span>
  <button type="button" class="text-[var(--accent)] hover:underline" @click="cancelBoostyLogin">
  {{ t("license.cancel") }}
  </button>
  </div>
  <div class="mt-2 flex items-center gap-2 text-[13px] text-[color:var(--tx-muted)]">
  <span class="h-px flex-1 bg-[var(--border)]"></span>
  <span>{{ t("license.orManual") }}</span>
  <span class="h-px flex-1 bg-[var(--border)]"></span>
  </div>
  <form class="mt-2 flex gap-2" @submit.prevent="saveLicense">
  <input
  v-model="licenseKeyInput"
  type="text"
  :placeholder="t('license.placeholder')"
  autocomplete="off"
  spellcheck="false"
  class="min-w-0 flex-1 rounded-md  bg-[var(--input)] px-2.5 py-1.5 font-mono text-[13px] text-[color:var(--tx)] placeholder:text-[color:var(--tx-muted)]  focus:outline-none"
  />
  <button
  type="submit"
  class="shrink-0 rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] disabled:opacity-50"
  :disabled="licenseBusy || !licenseKeyInput.trim()"
  >
  {{ t("license.activate") }}
  </button>
  </form>
  </div>
  <div class="mt-2 flex flex-wrap items-center gap-2 text-[13px] text-[color:var(--tx-muted)]">
  <span>{{ t("license.howTo") }}</span>
  <button
  type="button"
  class="text-[var(--accent)] hover:underline"
  @click="openExternal(`https://boosty.to/${activePack.boostyBlog}`)"
  >
  {{ t("license.openBlog") }} →
  </button>
  </div>
  </template>
  </div>

  <!-- Предупреждение о кастомных файлах (не с Modrinth/CurseForge).
       Цвет плашки: зелёный — всё проверено и безопасно, красный — есть опасные,
       жёлтый — есть непроверенные. -->
  <div
  v-if="warnCustomMods && status?.installed && status.custom_mods.length > 0"
  class="mt-4 rounded-md px-3.5 py-2.5 text-[13px]"
  :class="customBannerClass"
  >
  <button
  type="button"
  class="flex w-full items-center justify-between gap-3 text-left"
  @click="customModsOpen = !customModsOpen"
  >
  <span class="flex min-w-0 items-center gap-2">
  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current">
  <path d="M8 0a8 8 0 1 0 0 16A8 8 0 0 0 8 0ZM8 7.25a.74.74 0 0 1 .74.75v2.5a.74.74 0 0 1-1.48 0V8a.74.74 0 0 1 .74-.75Zm0 5.25a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z"/>
  </svg>
  <span class="min-w-0">
  {{ t("warn.customMods", { n: status.custom_mods.length }) }}
  </span>
  <button
  type="button"
  class="shrink-0 underline decoration-dotted underline-offset-2 disabled:opacity-50"
  :disabled="customScanBusy"
  @click.stop="scanActiveCustomMods()"
  >
  {{ customScanBusy ? t("scanner.scanning") : t("scanner.checkNow") }}
  </button>
  </span>
  <span class="shrink-0 underline decoration-dotted underline-offset-2">
  {{ customModsOpen ? t("warn.hide") : t("warn.show") }}
  </span>
  </button>
  <div v-if="customModsOpen" class="mt-2 space-y-1 border-t border-[var(--border)] pt-2">
  <ul class="space-y-1 font-mono text-[13px]">
  <li v-for="f in status.custom_mods" :key="f.path" class="flex items-start gap-2">
  <span class="truncate" :title="f.url">{{ f.path }}</span>
  <span v-if="f.scan_result" class="ml-auto shrink-0" :class="f.safe ? 'text-[#3fb950]' : 'text-[#f85149]'">{{ f.safe ? t("scanner.safe") : f.scan_result }}</span>
  <span v-else-if="f.sha256" class="ml-auto shrink-0 text-[color:var(--tx-muted)]">{{ t("scanner.unchecked") }}</span>
  </li>
  </ul>
  <p class="pt-1" :class="customBannerNoteClass">{{ customBannerState === "safe" ? t("warn.safeNote") : t("warn.note") }}</p>
  </div>
  </div>
  </div>

  <!-- Сабтабы: релизы / моды / ресурспаки / шейдеры / миры / консоль -->
  <div class="nice-scrollbar mb-4 flex shrink-0 items-center gap-1 overflow-x-auto border-b border-[var(--border)] pb-2">
  <template v-for="st in playSubTabsVisible" :key="st.kind">
  <span v-if="st.kind === 'screenshots'" class="mx-1.5 my-2 h-5 w-px shrink-0 bg-[var(--border)]"></span>
  <button
  type="button"
  class="relative flex shrink-0 items-center gap-1.5 px-3 pb-2.5 pt-1 text-[13px] font-semibold transition-colors"
  :class="playSubTab === st.kind
  ? 'text-[var(--accent)]'
  : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
  @click="playSubTab = st.kind"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current" v-html="st.icon"></svg>
  <span>{{ t("sub." + st.kind) }}</span>
  <span
  v-if="subTabCount(st.kind) > 0"
  class="rounded-full bg-[var(--input)] px-1.5 py-px text-[11px] font-bold tabular-nums"
  :class="playSubTab === st.kind ? 'text-[var(--accent)]' : 'text-[color:var(--tx-muted)]'"
  >{{ subTabCount(st.kind) }}</span>
  <span v-if="playSubTab === st.kind" class="absolute inset-x-2 bottom-0 h-[2.5px] rounded-t-full bg-[var(--accent)]"></span>
  </button>
  </template>
  </div>

  <!-- Список установленных версий -->
  <template v-if="playSubTab === 'releases'">
  <div class="min-h-0 flex-1 space-y-4 overflow-y-auto pr-1">
  <div class="flex items-center justify-end">
  <button type="button" class="rounded-md  bg-[var(--input)] px-2 py-1 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50" :disabled="remoteVersionsLoading" @click="refreshRemoteVersions()">
  <svg v-if="remoteVersionsLoading" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current"><path d="M8 1.5a.75.75 0 0 1 .75.75V8a.75.75 0 0 1-1.5 0V2.25A.75.75 0 0 1 8 1.5Zm3.36 2.14a.75.75 0 0 1 0 1.06 4 4 0 1 1-6.72 0 .75.75 0 0 1 1.06-1.06 2.5 2.5 0 1 0 4.6 0 .75.75 0 0 1 1.06-1.06Z"/></svg>
  <template v-else>{{ t("catalog.refresh") }}</template>
  </button>
  </div>
  <div v-if="versions && versions.installed.length > 0" class="flex items-center justify-between text-[13px] text-[color:var(--tx-muted)]">
  <span class="font-medium">{{ t("releases.count", { n: versions.installed.length }) }}</span>
  </div>

  <article
  v-for="r in versions?.installed ?? []"
  :key="r.version_id"
  class="rounded-xl  bg-[var(--panel)] shadow-sm transition-shadow hover:shadow-md"
  >
  <div class="flex items-center justify-between px-3.5 py-2.5">
  <div class="flex items-center gap-2.5 flex-wrap">
  <span class="font-mono text-sm font-semibold text-[var(--accent)]">
  {{ r.source_tag ?? r.version_id }}
  </span>
  <span v-if="versions && r.version_id === versions.active" class="rounded-full  bg-[#238636]/10 px-2 py-0.5 text-xs font-medium text-[#3fb950]">
  {{ t("releases.active") }}
  </span>
  </div>

  <div class="flex items-center gap-3">
  <span v-if="r.total_seconds > 0" class="font-mono text-[13px] text-[#d29922]" :title="t('releases.playtime')">
  {{ formatPlaytime(r.total_seconds) }}
  </span>
  <button
  v-if="versions && r.version_id !== versions.active"
  type="button"
  class="rounded-md  bg-[var(--input)] px-2.5 py-1 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] hover:text-white disabled:opacity-50"
  :disabled="busy"
  @click="handleSelectVersion(r.source_tag ?? r.version_id)"
  >
  {{ t("releases.switch") }}
  </button>
  </div>
  </div>
  </article>

  <!-- Доступные версии на сервере -->
  <div v-if="remoteVersions && remoteVersions.length > 0" class="pt-2">
  <div class="flex items-center justify-between text-[13px] text-[color:var(--tx-muted)]">
  <span class="font-medium">{{ t("releases.serverTitle", { n: remoteVersions.length }) }}</span>
  </div>
  <article
  v-for="v in remoteVersions"
  :key="v.id"
  class="mt-3 rounded-xl  bg-[var(--panel)] shadow-sm transition-shadow hover:shadow-md"
  >
  <div class="flex items-center justify-between gap-3 px-3.5 py-2.5">
  <div class="min-w-0 flex-1">
  <p class="font-mono text-sm font-semibold text-[var(--accent)]">v{{ v.version }}</p>
  <p class="mt-0.5 text-xs text-[color:var(--tx-muted)]">
  {{ formatDate(v.created_at) }} · {{ formatBytes(v.size) }}
  </p>
  <p v-if="v.changelog" class="mt-1 line-clamp-1 text-xs text-[color:var(--tx-muted)]">{{ v.changelog }}</p>
  </div>
  <button
  type="button"
  class="shrink-0 rounded-md  bg-[var(--input)] px-2.5 py-1 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] hover:text-white disabled:opacity-50"
  :disabled="busy || remoteInstallingId === v.id"
  @click="installRemoteVersion(v)"
  >
  <svg v-if="remoteInstallingId === v.id" viewBox="0 0 16 16" class="h-3.5 w-3.5 animate-spin fill-current"><path d="M8 1.5a.75.75 0 0 1 .75.75V8a.75.75 0 0 1-1.5 0V2.25A.75.75 0 0 1 8 1.5Z"/></svg>
  <template v-else>{{ t("releases.install") }}</template>
  </button>
  </div>
  </article>
  </div>
  </div>

  <div v-if="remoteVersionsLoading" class="shrink-0 rounded-xl  bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]">
  {{ t("files.loading") }}
  </div>
  <div v-else-if="!remoteVersions" class="shrink-0 rounded-xl  bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]">
  {{ t("releases.loadError") }}
  </div>
  </template>

  <!-- Папки файлов игры: моды / ресурспаки / шейдеры / миры -->
  <div
  v-else-if="playSubTab === 'mods' || playSubTab === 'resourcepacks' || playSubTab === 'shaderpacks' || playSubTab === 'saves'"
  class="flex min-h-0 flex-1 flex-col"
  >
  <div
  v-if="packLocked"
  class="mb-3 flex shrink-0 items-center justify-between gap-3 rounded-md  bg-[color-mix(in_srgb,var(--accent)_8%,transparent)] px-3 py-2 text-[13px] text-[color:var(--tx)]"
  >
  <span class="flex items-center gap-2">
  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-[var(--accent)]"><path d="M8 1a4.25 4.25 0 0 0-4.25 4.25V7H3.5A1.5 1.5 0 0 0 2 8.5v5A1.5 1.5 0 0 0 3.5 15h9a1.5 1.5 0 0 0 1.5-1.5v-5A1.5 1.5 0 0 0 12.5 7h-.25V5.25A4.25 4.25 0 0 0 8 1Zm2.5 6h-5V5.25a2.5 2.5 0 0 1 5 0Z"/></svg>
  {{ t("files.locked") }}
  </span>
  <button
  type="button"
  class="shrink-0 rounded px-2 py-1 font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_15%,transparent)]"
  :title="t('files.unbindHint')"
  @click="confirmUnbindPack"
  >{{ unbindArmed ? t("files.unbindConfirm") : t("files.unbind") }}</button>
  </div>
  <!-- Панель действий: строка 1 -->
  <div class="mb-2 flex shrink-0 items-center justify-between gap-3">
  <span class="flex shrink-0 items-center gap-2 text-[13px] text-[color:var(--tx-muted)]">
  {{ playSubTab === "saves" ? t("files.worldsCount", { n: fileVisibleCount }) : t("files.count", { n: fileVisibleCount }) }}
  <span
  v-if="playSubTab !== 'saves' && fileVisibleCount > 0"
  class="rounded-full bg-[color-mix(in_srgb,var(--accent)_8%,transparent)] px-2 py-0.5 text-xs font-medium text-[var(--accent)]"
  >
  {{ t("files.enabledOf", { n: enabledCountIn(playSubTab as GameFolderKind), m: fileVisibleCount }) }}
  </span>
  </span>
  <div class="flex min-w-0 items-center gap-1.5">
  <template v-if="playSubTab !== 'saves' && modUpdatesTab.length > 0">
  <button
  type="button"
  class="flex shrink-0 items-center gap-1.5 rounded-md bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
  :disabled="updateAllBusy || updatingMod !== null || packLocked"
  @click="updateAllMods"
  >
  <svg v-if="updateAllBusy" viewBox="0 0 16 16" class="h-3.5 w-3.5 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
  <svg v-else viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M4.5 3.75a.75.75 0 0 0-1.5 0v2.5A.75.75 0 0 0 3.75 7h2.5a.75.75 0 0 0 0-1.5H5.07a4.5 4.5 0 1 1 .57 6.44.75.75 0 0 0-.98-1.13 6 6 0 1 0-.16-8.5v.49Z"/></svg>
  {{ t("mods.updateAll") }}
  <span class="rounded-full bg-[var(--accent)] px-1.5 text-xs font-bold text-[var(--bg)]">{{ modUpdatesTab.length }}</span>
  </button>
  </template>
  <button
  v-if="playSubTab !== 'saves'"
  type="button"
  class="flex shrink-0 items-center gap-1.5 rounded-md bg-[var(--accent)] px-2.5 py-1.5 text-[13px] font-semibold text-white transition-colors hover:opacity-90 disabled:opacity-50"
  :disabled="packLocked"
  :title="t('mods.addHint')"
  @click="openSearch((playSubTab === 'mods' ? 'mod' : playSubTab === 'resourcepacks' ? 'resourcepack' : 'shaderpack') as ModrinthSearchKind, 'modrinth')"
  >
  <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M8 2.75a.75.75 0 0 1 .75.75v3.75h3.75a.75.75 0 0 1 0 1.5h-3.75v3.75a.75.75 0 0 1-1.5 0V8.75H3.5a.75.75 0 0 1 0-1.5h3.75V3.5A.75.75 0 0 1 8 2.75Z"/></svg>
  {{ playSubTab === 'mods' ? t("mods.add") : playSubTab === 'resourcepacks' ? t("mods.addRP") : t("mods.addShaders") }}
  </button>
  <button
  v-if="playSubTab === 'saves'"
  type="button"
  class="flex shrink-0 items-center gap-1.5 rounded-md bg-[var(--accent)] px-2.5 py-1.5 text-[13px] font-semibold text-white transition-colors hover:opacity-90"
  @click="openSearch('datapack', 'modrinth')"
  >
  <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M8 2.75a.75.75 0 0 1 .75.75v3.75h3.75a.75.75 0 0 1 0 1.5h-3.75v3.75a.75.75 0 0 1-1.5 0V8.75H3.5a.75.75 0 0 1 0-1.5h3.75V3.5A.75.75 0 0 1 8 2.75Z"/></svg>
  {{ t("mods.addDatapack") }}
  </button>
  <button
  v-if="playSubTab === 'mods' && !packLocked"
  type="button"
  class="flex shrink-0 items-center gap-1.5 rounded-md bg-[var(--input)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[var(--tx)]"
  :title="t('scanner.hint')"
  @click="openModScanner"
  >
  <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M8 1.25a.75.75 0 0 1 .75.75v1.5a.75.75 0 0 1-1.5 0V2A.75.75 0 0 1 8 1.25Zm0 9.75a1.75 1.75 0 1 0 0-3.5 1.75 1.75 0 0 0 0 3.5Zm0 1.5a3.25 3.25 0 1 0 0-6.5 3.25 3.25 0 0 0 0 6.5Zm6.75-4.75a.75.75 0 0 0-1.5 0V8a.75.75 0 0 0 1.5 0V7.75ZM8 12.5a.75.75 0 0 1 .75.75V14a.75.75 0 0 1-1.5 0v-.75A.75.75 0 0 1 8 12.5Zm-5.25-4.75a.75.75 0 0 1 .75.75v.25a.75.75 0 0 1-1.5 0V8.5a.75.75 0 0 1 .75-.75Zm8.96-4.46a.75.75 0 0 1 0 1.06l-1.06 1.06a.75.75 0 1 1-1.06-1.06l1.06-1.06a.75.75 0 0 1 1.06 0Zm-8.42 8.42a.75.75 0 0 1 0 1.06L2.23 14.53a.75.75 0 0 1-1.06-1.06l1.06-1.06a.75.75 0 0 1 1.06 0Zm-1.06-8.42a.75.75 0 0 1 1.06 0l1.06 1.06A.75.75 0 1 1 3.29 4.89L2.23 3.83a.75.75 0 0 1 0-1.06Z"/></svg>
  {{ t("scanner.btn") }}
  </button>
  </div>
  </div>

  <!-- Панель действий: строка 2 — поиск + сортировка + фильтры -->
  <div v-if="playSubTab !== 'saves'" class="mb-3 flex shrink-0 items-center gap-2">
  <div class="relative min-w-0 flex-1">
  <svg viewBox="0 0 16 16" class="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 fill-[var(--tx-muted)]">
  <path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"/>
  </svg>
  <input
  v-model="fileSearch"
  type="text"
  :placeholder="t('files.search')"
  class="w-full rounded-lg bg-[var(--bg)] py-1.5 pl-8 pr-3 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)] outline-none transition-colors focus:ring-1 focus:ring-[var(--accent)]/30"
  />
  </div>
  <div class="flex shrink-0 items-center gap-0.5 rounded-lg bg-[var(--bg)] p-0.5">
  <button
  type="button"
  class="flex items-center gap-1 rounded-md px-2.5 py-1 text-[13px] font-medium transition-colors"
  :class="fileSortKey === 'name' ? 'bg-[var(--accent)] text-white' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
  :title="fileSortKey === 'name' ? (fileSortDir === 'asc' ? t('files.sortNameAsc') : t('files.sortNameDesc')) : t('files.sortNameHint')"
  @click="toggleFileSort('name')"
  >
  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M3.75 2h1.5a.75.75 0 0 1 0 1.5h-1.5a.75.75 0 0 1 0-1.5Zm0 4h3.5a.75.75 0 0 1 0 1.5h-3.5a.75.75 0 0 1 0-1.5Zm0 4h5.5a.75.75 0 0 1 0 1.5h-5.5a.75.75 0 0 1 0-1.5Z"/></svg>
  {{ t("files.sortName") }}
  <svg v-if="fileSortKey === 'name'" viewBox="0 0 16 16" class="h-2.5 w-2.5 fill-current" :style="{ transform: fileSortDir === 'asc' ? 'none' : 'rotate(180deg)' }"><path d="M8 11.5 3.5 7h9L8 11.5Z"/></svg>
  </button>
  <button
  type="button"
  class="flex items-center gap-1 rounded-md px-2.5 py-1 text-[13px] font-medium transition-colors"
  :class="fileSortKey === 'date' ? 'bg-[var(--accent)] text-white' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
  :title="fileSortKey === 'date' ? (fileSortDir === 'desc' ? t('files.sortDateNew') : t('files.sortDateOld')) : t('files.sortDateHint')"
  @click="toggleFileSort('date')"
  >
  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M4.75 0a.75.75 0 0 1 .75.75V2h5V.75a.75.75 0 0 1 1.5 0V2h1.25c.966 0 1.75.784 1.75 1.75v10.5A1.75 1.75 0 0 1 13.25 16H2.75A1.75 1.75 0 0 1 1 14.25V3.75C1 2.784 1.784 2 2.75 2H4V.75A.75.75 0 0 1 4.75 0Zm0 3.5h-2a.25.25 0 0 0-.25.25V6h11V3.75a.25.25 0 0 0-.25-.25h-2v.75a.75.75 0 0 1-1.5 0v-.75h-5v.75a.75.75 0 0 1-1.5 0v-.75Zm-.75 4v6.75c0 .138.112.25.25.25h10.5a.25.25 0 0 0 .25-.25V7.5H4Z"/></svg>
  {{ t("files.sortDate") }}
  <svg v-if="fileSortKey === 'date'" viewBox="0 0 16 16" class="h-2.5 w-2.5 fill-current" :style="{ transform: fileSortDir === 'asc' ? 'none' : 'rotate(180deg)' }"><path d="M8 11.5 3.5 7h9L8 11.5Z"/></svg>
  </button>
  <button
  v-if="fileSortKey !== 'none'"
  type="button"
  class="rounded-md px-1.5 py-1 text-[13px] leading-none text-[color:var(--tx-muted)] transition-colors hover:text-[var(--accent)]"
  :title="t('files.sortReset')"
  @click="clearFileSort"
  >×</button>
  </div>
  <div class="flex shrink-0 items-center gap-0.5 rounded-lg bg-[var(--bg)] p-0.5">
  <button
  type="button"
  class="rounded-md px-2.5 py-1 text-[13px] font-medium transition-colors"
  :title="t('files.fAllHint')"
  :class="fileStatusFilter === 'all' ? 'bg-[var(--accent)] text-white' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
  @click="setFileStatusFilter('all')"
  >{{ t("files.fAll") }}</button>
  <button
  type="button"
  class="rounded-md px-2.5 py-1 text-[13px] font-medium transition-colors"
  :title="t('files.fEnabledHint')"
  :class="fileStatusFilter === 'enabled' ? 'bg-[var(--accent)] text-white' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
  @click="setFileStatusFilter('enabled')"
  >{{ t("files.fEnabled") }}</button>
  <button
  type="button"
  class="rounded-md px-2.5 py-1 text-[13px] font-medium transition-colors"
  :title="t('files.fDisabledHint')"
  :class="fileStatusFilter === 'disabled' ? 'bg-[var(--accent)] text-white' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
  @click="setFileStatusFilter('disabled')"
  >{{ t("files.fDisabled") }}</button>
  <button
  type="button"
  class="rounded-md px-2.5 py-1 text-[13px] font-medium transition-colors"
  :title="t('files.fUpdatesHint')"
  :class="fileStatusFilter === 'updates' ? 'bg-[var(--accent)] text-white' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
  @click="setFileStatusFilter('updates')"
  >{{ t("files.fUpdates") }}</button>
  </div>
  <div ref="fileMenuRef" class="relative">
  <button
  type="button"
  class="flex shrink-0 items-center justify-center rounded-lg bg-[var(--bg)] px-2 py-1.5 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[var(--tx)]"
  :title="t('files.more')"
  @click="fileMenuOpen = !fileMenuOpen"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M8 4.25a1.25 1.25 0 1 1 0-2.5 1.25 1.25 0 0 1 0 2.5Zm0 5a1.25 1.25 0 1 1 0-2.5 1.25 1.25 0 0 1 0 2.5Zm0 5a1.25 1.25 0 1 1 0-2.5 1.25 1.25 0 0 1 0 2.5Z"/></svg>
  </button>
  <div
  v-if="fileMenuOpen"
  class="absolute right-0 top-[calc(100%+4px)] z-50 w-56 overflow-hidden rounded-xl bg-[var(--panel)] py-1 shadow-xl"
  >
  <!-- Управление сборкой -->
  <template v-if="!packLocked">
  <button
  type="button"
  class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click="fileMenuOpen = false; openSearch((playSubTab === 'mods' ? 'mod' : playSubTab === 'resourcepacks' ? 'resourcepack' : 'shaderpack') as ModrinthSearchKind, 'modrinth')"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M8 2.75a.75.75 0 0 1 .75.75v3.75h3.75a.75.75 0 0 1 0 1.5h-3.75v3.75a.75.75 0 0 1-1.5 0V8.75H3.5a.75.75 0 0 1 0-1.5h3.75V3.5A.75.75 0 0 1 8 2.75Z"/></svg>
  {{ playSubTab === 'mods' ? t("mods.add") : playSubTab === 'resourcepacks' ? t("mods.addRP") : t("mods.addShaders") }}
  </button>
  <button
  v-if="modUpdatesTab.length > 0 && playSubTab === 'mods'"
  type="button"
  class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] disabled:opacity-50"
  :disabled="updateAllBusy || updatingMod !== null"
  @click="fileMenuOpen = false; updateAllMods()"
  >
  <svg v-if="updateAllBusy" viewBox="0 0 16 16" class="h-4 w-4 shrink-0 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
  <svg v-else viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M4.5 3.75a.75.75 0 0 0-1.5 0v2.5A.75.75 0 0 0 3.75 7h2.5a.75.75 0 0 0 0-1.5H5.07a4.5 4.5 0 1 1 .57 6.44.75.75 0 0 0-.98-1.13 6 6 0 1 0-.16-8.5v.49Z"/></svg>
  {{ t("mods.updateAll") }} <span class="ml-auto rounded-full bg-[var(--accent)] px-1.5 text-xs font-bold text-[var(--bg)]">{{ modUpdatesTab.length }}</span>
  </button>
  <button
  type="button"
  class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click="fileMenuOpen = false; openModScanner"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M8 1.25a.75.75 0 0 1 .75.75v1.5a.75.75 0 0 1-1.5 0V2A.75.75 0 0 1 8 1.25Zm0 9.75a1.75 1.75 0 1 0 0-3.5 1.75 1.75 0 0 0 0 3.5Zm0 1.5a3.25 3.25 0 1 0 0-6.5 3.25 3.25 0 0 0 0 6.5Zm6.75-4.75a.75.75 0 0 0-1.5 0V8a.75.75 0 0 0 1.5 0V7.75ZM8 12.5a.75.75 0 0 1 .75.75V14a.75.75 0 0 1-1.5 0v-.75A.75.75 0 0 1 8 12.5Zm-5.25-4.75a.75.75 0 0 1 .75.75v.25a.75.75 0 0 1-1.5 0V8.5a.75.75 0 0 1 .75-.75Zm8.96-4.46a.75.75 0 0 1 0 1.06l-1.06 1.06a.75.75 0 1 1-1.06-1.06l1.06-1.06a.75.75 0 0 1 1.06 0Zm-8.42 8.42a.75.75 0 0 1 0 1.06L2.23 14.53a.75.75 0 0 1-1.06-1.06l1.06-1.06a.75.75 0 0 1 1.06 0Zm-1.06-8.42a.75.75 0 0 1 1.06 0l1.06 1.06A.75.75 0 1 1 3.29 4.89L2.23 3.83a.75.75 0 0 1 0-1.06Z"/></svg>
  {{ t("scanner.btn") }}
  </button>
  </template>
  <!-- Файлы -->
  <div class="mx-3 my-1 border-t border-[var(--border)]"></div>
  <button
  type="button"
  class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click="fileMenuOpen = false; openFolder(playSubTab as GameFolderKind)"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M.513 1.513A1.75 1.75 0 0 1 1.75 1h3.5c.55 0 1.07.26 1.4.7l.9 1.2a.25.25 0 0 0 .2.1H13a1 1 0 0 1 1 1v.5H2.75a.75.75 0 0 0 0 1.5h11.978a1 1 0 0 1 .994 1.117L15 13.25A1.75 1.75 0 0 1 13.25 15H1.75A1.75 1.75 0 0 1 0 13.25V2.75c0-.464.184-.91.513-1.237Z"/></svg>
  {{ t("files.open") }}
  </button>
  <button
  type="button"
  class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click="fileMenuOpen = false; selectAllFiles(playSubTab as GameFolderKind, fileListFiltered)"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/></svg>
  {{ t("files.selectAll") }}
  </button>
  <button
  type="button"
  class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click="fileMenuOpen = false; enableAllFiles(true)"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/></svg>
  {{ t("files.enable") }} ({{ t("files.fAll") }})
  </button>
  <button
  type="button"
  class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click="fileMenuOpen = false; enableAllFiles(false)"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.749.749 0 0 1 1.275.326.749.749 0 0 1-.215.734L9.06 8l3.22 3.22a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215L8 9.06l-3.22 3.22a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
  {{ t("files.disable") }} ({{ t("files.fAll") }})
  </button>
  <div class="mx-3 my-1 border-t border-[var(--border)]"></div>
  <button
  v-if="Object.keys(selectedFiles).length > 0"
  type="button"
  class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[13px] text-[#f85149] transition-colors hover:bg-[#f85149]/10 disabled:opacity-50"
  :disabled="fileDeleteBusy"
  @click="fileMenuOpen = false; deleteSelectedFiles()"
  >
  <svg v-if="fileDeleteBusy" viewBox="0 0 16 16" class="h-4 w-4 shrink-0 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
  <svg v-else viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M6.5 1.75A1.75 1.75 0 0 1 8.25 0h2.5A1.75 1.75 0 0 1 12 1.75V3h2.25a.75.75 0 0 1 0 1.5h-.65l-.75 9.006A1.75 1.75 0 0 1 10.738 15H5.262a1.75 1.75 0 0 1-1.742-1.494L2.77 4.5H2.12a.75.75 0 0 1 0-1.5H4.5V1.75ZM5.07 4.5l.76 8.91a.25.25 0 0 0 .25.214h5.456a.25.25 0 0 0 .25-.214L12.54 4.5H5.07Z"/></svg>
  {{ t("files.delete") }} ({{ Object.keys(selectedFiles).length }})
  </button>
  </div>
  </div>
  </div>

  <div v-if="!gameFiles[playSubTab]" class="flex flex-1 items-center justify-center text-[13px] text-[color:var(--tx-muted)]">
  <svg class="mr-2 h-4 w-4 animate-spin fill-[var(--accent)]" viewBox="0 0 16 16">
  <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
  </svg>
  {{ t("files.loading") }}
  </div>
  <div v-else-if="(gameFiles[playSubTab] ?? []).length === 0" class="shrink-0 rounded-xl  bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]">
  <p class="mb-3">{{ t("files.empty") }}</p>
  <button
  type="button"
  class="inline-flex items-center gap-1.5 rounded-md  bg-[var(--input)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click="openFolder(playSubTab as GameFolderKind)"
  >
  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M.513 1.513A1.75 1.75 0 0 1 1.75 1h3.5c.55 0 1.07.26 1.4.7l.9 1.2a.25.25 0 0 0 .2.1H13a1 1 0 0 1 1 1v.5H2.75a.75.75 0 0 0 0 1.5h11.978a1 1 0 0 1 .994 1.117L15 13.25A1.75 1.75 0 0 1 13.25 15H1.75A1.75 1.75 0 0 1 0 13.25V2.75c0-.464.184-.91.513-1.237Z"/></svg>
  {{ t("files.open") }}
  </button>
  </div>
  <div
  v-else
  ref="fileListRef"
  class="min-h-0 flex-1 overflow-y-auto overscroll-contain pr-1 pb-8"
  @scroll="fileListScroll"
  >
  <div class="relative" :style="{ height: `${fileListTotal}px` }">
  <div
  class="absolute left-0 right-0 space-y-2"
  :style="{ transform: `translateY(${fileListStart * fileRowStride}px)` }"
  >
<div
  v-for="f in fileListVisible"
  :key="f.name"
  class="file-row flex cursor-pointer items-center gap-3 rounded-lg  px-3 py-2 transition-colors"
  :class="[
  isFileSelected(playSubTab, f)
  ? ' bg-[color-mix(in_srgb,var(--accent-deep)_10%,transparent)]'
  : ' bg-[var(--panel)] ',
  { 'opacity-60': !f.enabled },
  ]"
  @click="toggleFileSelect(playSubTab as GameFolderKind, f)"
  @contextmenu.prevent="openFileCtx($event, f)"
  >
  <svg
  viewBox="0 0 16 16"
  class="h-4 w-4 shrink-0"
  :class="isFileSelected(playSubTab, f) ? 'fill-[var(--accent)]' : 'fill-[var(--tx-muted)]'"
  >
  <path v-if="isFileSelected(playSubTab, f)" d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/>
  <path v-else d="M4 1.75C4 .784 4.784 0 5.75 0h5.586c.464 0 .909.184 1.237.513l2.914 2.914a1.75 1.75 0 0 1 .513 1.237V12.25A1.75 1.75 0 0 1 14.25 14H5.75A1.75 1.75 0 0 1 4 12.25Zm1.75-.25a.25.25 0 0 0-.25.25v10.5c0 .138.112.25.25.25h8.5a.25.25 0 0 0 .25-.25V6h-2.75A1.75 1.75 0 0 1 10 4.25V1.5Z"/>
  </svg>
  <div class="flex h-10 w-10 shrink-0 items-center justify-center overflow-hidden rounded-md  bg-[var(--bg)]">
  <img
  v-if="modrinthMetaFor(f)?.icon || curseMetaFor(f)?.icon || gameFileIcon(playSubTab, f.name)"
  :src="modrinthMetaFor(f)?.icon || curseMetaFor(f)?.icon || gameFileIcon(playSubTab, f.name)"
  alt=""
  loading="lazy"
  class="h-full w-full object-contain"
  />
  <svg v-else viewBox="0 0 16 16" class="h-5 w-5 fill-[var(--tx-muted)]">
  <path d="M.75 6.25a1.75 1.75 0 0 1 1.75-1.75h2.054l1.17-1.17A1.74 1.74 0 0 1 6.902 2.75h1.536l-.055.836a1.44 1.44 0 0 0 .432 1.123l.022.022c.059.059.12.108.183.148.523.34 1.074.405 1.528.429.755.04 1.452.044 1.766.044h3.44v.586c0 .527-.211 1.032-.587 1.404l-3.318 3.318a1.5 1.5 0 0 1-1.06.44H4.78l-.824-.412a1.75 1.75 0 0 1-.736-2.383.5.5 0 0 1-.368-.454A1.75 1.75 0 0 1 .75 6.25Zm13.24 0h-3.14c-.249 0-.679-.004-1.112-.03-.36-.022-.622-.066-.783-.111.05-.066.11-.129.176-.194l.483-.483c.344-.344.416-.861.18-1.283A1.75 1.75 0 0 0 8.5 2.75H4.75A1.75 1.75 0 0 1 4.75 1.5c.692-.06 1.4-.086 2.127-.086.63 0 1.255.022 1.873.064a.75.75 0 0 1 .5.25.75.75 0 0 1 .246.5l.293 2.927.178.04c.646.147 1.548.377 2.615.614.17.038.2.07.22.098a.6.6 0 0 1 .074.233.75.75 0 0 1-.075.4.6.6 0 0 1-.235.23ZM3.75 10.75h5.38l1.5-1.5H3.75a.75.75 0 0 1-.143 1.482l-.14.014a.75.75 0 0 1 .283-.004L3.75 10.75Z"/>
  </svg>
  </div>
  <div class="min-w-0 flex-1">
  <div
  class="truncate text-[13px] font-medium text-[color:var(--tx)]"
  :title="fileMetaTitle(f)"
  >
  {{ fileMetaTitle(f) }}
  </div>
  <div class="truncate text-xs text-[color:var(--tx-muted)]">
  <template v-if="modrinthMetaFor(f)?.title || curseMetaFor(f)?.title">
  <template v-if="modrinthVersionFor(f)">{{ modrinthVersionFor(f) }} · </template>
  <template v-else-if="curseMetaFor(f)?.title">{{ f.displayName }} · </template>
  </template>{{ f.kind === "dir" ? t("files.dir") : `${formatBytes(f.sizeBytes)} · ${formatUnixDate(f.modified)} · ${f.enabled ? t("files.enabled") : t("files.disabled")}` }}
  </div>
  </div>
  <button
  v-if="playSubTab !== 'saves' && f.curseforgeProjectId"
  type="button"
  class="flex shrink-0 items-center gap-1 rounded-md  bg-[var(--input)] px-2 py-1 text-xs font-medium text-[color:var(--tx-muted)] transition-colors  hover:text-[var(--accent)]"
  :title="t('files.curseforge')"
  @click.stop="openFileOnCurseForge(playSubTab as GameFolderKind, f)"
  >
  CurseForge
  </button>
  <button
  v-if="playSubTab !== 'saves' && !f.curseforgeProjectId"
  type="button"
  class="flex shrink-0 items-center gap-1 rounded-md  bg-[var(--input)] px-2 py-1 text-xs font-medium text-[color:var(--tx-muted)] transition-colors  hover:text-[var(--accent)]"
  :title="t('files.modrinth')"
  @click.stop="openFileOnModrinth(playSubTab as GameFolderKind, f)"
  >
  Modrinth
  </button>
  <button
  v-if="playSubTab !== 'saves' && (f.modrinthProjectId || f.modrinthUrl || f.curseforgeProjectId)"
  type="button"
  class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md  bg-[var(--input)] text-[color:var(--tx-muted)] transition-colors  hover:text-[var(--accent)]"
  :title="t('files.view')"
  @click.stop="openFileDetail(playSubTab as GameFolderKind, f)"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M8 3.75a3.25 3.25 0 1 0 0 6.5 3.25 3.25 0 0 0 0-6.5Zm0 8.5A8.75 8.75 0 0 1 0 8a8.75 8.75 0 0 1 8-4.25A8.75 8.75 0 0 1 16 8a8.75 8.75 0 0 1-8 4.25Z"/></svg>
  </button>
  <button
  v-if="playSubTab !== 'saves' && modUpdateFor(f)"
  type="button"
  class="flex shrink-0 items-center gap-1 rounded-md  bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] px-2 py-1 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_22%,transparent)] disabled:opacity-50"
  :disabled="updatingMod !== null || packLocked"
  :title="`${modUpdateFor(f)!.newVersion.name} (${modUpdateFor(f)!.newVersion.versionNumber})`"
  @click.stop="updateOneMod(modUpdateFor(f)!)"
  >
  <svg v-if="updatingMod === f.name" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
  <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
  </svg>
  <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
  <path d="M4.5 3.75a.75.75 0 0 0-1.5 0v2.5A.75.75 0 0 0 3.75 7h2.5a.75.75 0 0 0 0-1.5H5.07a4.5 4.5 0 1 1 .57 6.44.75.75 0 0 0-.98-1.13 6 6 0 1 0-.16-8.5v.49Z"/>
  </svg>
  {{ t("mods.update") }}
  </button>
  <button
  v-if="f.kind === 'file'"
  type="button"
  class="relative h-5 w-9 shrink-0 rounded-full transition-all duration-200"
  :class="[
  f.enabled ? 'bg-[#238636]' : 'bg-[var(--tx-muted)]',
  isFileToggling(playSubTab, f) ? 'opacity-50 cursor-wait' : 'hover:bg-[var(--input-50)]',
  ]"
  role="switch"
  :aria-checked="f.enabled"
  :disabled="isFileToggling(playSubTab, f) || packLocked"
  :title="isFileToggling(playSubTab, f) ? undefined : (packLocked ? t('files.locked') : (f.enabled ? t('files.disable') : t('files.enable')))"
  @click.stop="handleToggleFile(playSubTab as GameFolderKind, f)"
  >
  <span
  class="absolute top-0.5 flex h-4 w-4 items-center justify-center rounded-full bg-white shadow-sm transition-all duration-200"
  :class="f.enabled ? 'left-[18px]' : 'left-0.5'"
  >
  <svg v-if="isFileToggling(playSubTab, f)" viewBox="0 0 16 16" class="h-2.5 w-2.5 animate-spin fill-[var(--accent)]">
  <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
  </svg>
  </span>
  </button>
  </div>
  </div>
  </div>
  </div>
  </div>

  <!-- ======= Дубликаты (mods / resourcepacks / shaderpacks) ======= -->
  <template v-else-if="playSubTab === 'duplicates'">
  <div class="flex min-h-0 flex-1 flex-col">
  <div v-if="duplicatesLoading" class="flex flex-1 items-center justify-center text-[13px] text-[color:var(--tx-muted)]">
  <svg class="mr-2 h-4 w-4 animate-spin fill-[var(--accent)]" viewBox="0 0 16 16">
  <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
  </svg>
  {{ t("duplicates.loading") }}
  </div>
  <div v-else-if="duplicates.groups.length === 0" class="flex flex-1 items-center justify-center">
  <div class="rounded-xl  bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]">
  <p class="font-medium text-[color:var(--tx)]">{{ t("duplicates.empty") }}</p>
  </div>
  </div>
  <div v-else class="min-h-0 flex-1 space-y-3 overflow-y-auto pr-1 pb-8">
  <p class="text-[13px] text-[color:var(--tx-muted)]">
  {{ t("duplicates.found", { n: duplicates.groups.length, size: formatBytes(duplicates.wasted_bytes) }) }}
  </p>
  <div v-for="(g, gi) in duplicates.groups" :key="gi" class="rounded-xl  bg-[var(--panel)] shadow-sm">
  <div class="flex items-center justify-between gap-2 border-b border-[var(--border)]  px-3 py-2">
  <p class="text-[13px] font-medium text-[color:var(--tx-strong)]">
  {{ t("duplicates.group", { n: g.files.length, size: formatBytes(g.size_bytes) }) }}
  </p>
  <button
  type="button"
  class="rounded  bg-[#f85149]/10 px-2 py-0.5 text-xs font-semibold text-[#f85149] transition-colors hover:bg-[#f85149]/20"
  @click="keepOne(g)"
  >{{ t("duplicates.keepOne") }}</button>
  </div>
  <ul class="space-y-1 p-2">
  <li
  v-for="f in g.files"
  :key="f.path"
  class="flex items-center gap-2 rounded px-2 py-1 text-[13px] hover:bg-[var(--input-50)]"
  >
  <span class="min-w-0 flex-1 truncate font-mono text-[color:var(--tx-muted)]" :title="f.path">{{ f.folder }} / {{ f.name }}</span>
  <button
  type="button"
  class="shrink-0 rounded px-1.5 py-0.5 text-xs font-semibold text-[color:var(--tx-muted)] transition-colors hover:bg-[#f85149]/15 hover:text-[#f85149]"
  :title="t('duplicates.delete')"
  @click="removeDuplicate(packId, f)"
  >{{ t("duplicates.delete") }}</button>
  </li>
  </ul>
  </div>
  </div>
  </div>
  </template>

  <!-- ======= Скриншоты сборки (папка screenshots установленной версии) ======= -->
  <template v-else-if="playSubTab === 'screenshots'">
  <div class="flex min-h-0 flex-1 flex-col">
  <div v-if="screenshotsLoading" class="flex flex-1 items-center justify-center text-[13px] text-[color:var(--tx-muted)]">
  <svg class="mr-2 h-4 w-4 animate-spin fill-[var(--accent)]" viewBox="0 0 16 16">
  <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
  </svg>
  {{ t("screenshots.loading") }}
  </div>
  <div v-else-if="!packScreenshotsInstalled" class="flex flex-1 items-center justify-center">
  <div class="rounded-xl  bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]">
  <p class="font-medium text-[color:var(--tx)]">{{ t("screenshots.noInstall") }}</p>
  </div>
  </div>
  <div v-else-if="packScreenshots.length === 0" class="flex flex-1 items-center justify-center">
  <div class="rounded-xl  bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]">
  <p class="font-medium text-[color:var(--tx)]">{{ t("screenshots.empty") }}</p>
  </div>
  </div>
  <div v-else class="min-h-0 flex-1 space-y-3 overflow-y-auto pr-1 pb-8">
  <p class="text-[13px] text-[color:var(--tx-muted)]">
  {{ t("screenshots.count", { n: packScreenshots.length }) }}
  </p>
  <div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
  <button
  v-for="(shot, i) in packScreenshots"
  :key="shot.path"
  type="button"
  class="group relative overflow-hidden rounded-xl  bg-[var(--panel)] shadow-sm transition-colors "
  @click="shotIdx = i"
  >
  <img
  :src="convertFileSrc(shot.path)"
  :alt="`${t('sub.screenshots')} ${i + 1}`"
  loading="lazy"
  class="aspect-video w-full object-cover transition-transform duration-300 group-hover:scale-[1.03]"
  />
  <span
  v-if="shot.modified"
  class="pointer-events-none absolute bottom-1 right-1 rounded bg-black/60 px-1.5 py-0.5 text-xs font-medium text-white"
  >{{ formatUnixDate(shot.modified) }}</span>
  </button>
  </div>
  <p class="text-xs text-[color:var(--tx-muted)]">{{ t("screenshots.note") }}</p>
  </div>
  </div>

  <!-- Лайтбокс -->
  <div
  v-if="shotIdx !== null"
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/85 p-6"
  @click.self="shotIdx = null"
  >
  <button
  type="button"
  class="absolute right-4 top-4 rounded-md bg-[var(--panel)] px-2.5 py-1 text-[13px] font-semibold text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click="shotIdx = null"
  >
  ✕
  </button>
  <button
  v-if="packScreenshots.length > 1"
  type="button"
  class="absolute left-3 top-1/2 -translate-y-1/2 rounded-md bg-[var(--panel)] px-2.5 py-1.5 text-sm font-semibold text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click="shotIdx = ((shotIdx ?? 0) - 1 + packScreenshots.length) % packScreenshots.length"
  >
  ←
  </button>
  <button
  v-if="packScreenshots.length > 1"
  type="button"
  class="absolute right-3 top-1/2 -translate-y-1/2 rounded-md bg-[var(--panel)] px-2.5 py-1.5 text-sm font-semibold text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click="shotIdx = ((shotIdx ?? 0) + 1) % packScreenshots.length"
  >
  →
  </button>
  <img
  :src="convertFileSrc(packScreenshots[shotIdx ?? 0]?.path)"
  class="max-h-[82vh] max-w-full rounded-lg object-contain shadow-2xl"
  alt=""
  />
  <span class="absolute bottom-4 rounded bg-black/60 px-2 py-1 font-mono text-[13px] text-[color:var(--tx-muted)]">
  {{ (shotIdx ?? 0) + 1 }} / {{ packScreenshots.length }}
  <template v-if="packScreenshots[shotIdx ?? 0]?.modified">
  · {{ formatUnixDate(packScreenshots[shotIdx ?? 0]!.modified) }}
  </template>
  </span>
  </div>
  </template>

  <!-- ======= Сервера: сборки (servers.json) сверху + свои (servers.dat) снизу ======= -->
  <template v-else-if="playSubTab === 'servers'">
  <div class="min-h-0 flex-1 overflow-y-auto pr-1 pb-8">
  <div v-for="group in serverGroups" :key="group.key" class="mb-8 last:mb-0">
  <div class="mb-3 flex items-center justify-between">
  <p class="text-[13px] font-medium text-[color:var(--tx-strong)]">
  {{ group.title }}
  <span class="font-normal text-[color:var(--tx-muted)]">· {{ group.servers.length }}</span>
  </p>

  </div>
  <p v-if="group.servers.length === 0" class="rounded-xl  bg-[var(--panel)] shadow-sm px-3.5 py-2.5 text-[13px] text-[color:var(--tx-muted)]">
  {{ group.emptyText }}
  </p>
  <div v-else class="grid gap-3 sm:grid-cols-2">
  <div
  v-for="s in group.servers"
  :key="`${group.key}-${serverKey(s)}`"
  class="flex flex-col gap-3 rounded-xl  bg-[var(--panel)] shadow-sm p-4 transition-colors "
  >
  <div class="flex items-start gap-3">
  <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md  bg-[var(--input)]">
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-[var(--tx-muted)]">
  <path d="M3 1.5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-9a2 2 0 0 0-2-2ZM1.5 4.5H14.5v1.5H1.5ZM1.5 8H14.5v1.25H1.5Zm0 3.25H7v1.5H1.5A.5.5 0 0 1 1 12.25v-1ZM8.5 12.75v-1.5h6v1.5A.5.5 0 0 1 14.5 13h-5a1 1 0 0 1-1-.25ZM2 5.75a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Zm3 0a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0ZM2 9.25a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Zm3 0a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Z"/>
  </svg>
  </div>
  <div class="min-w-0 flex-1">
  <div class="flex items-center gap-2">
  <span class="truncate text-sm font-semibold text-[color:var(--tx-strong)]">{{ s.name }}</span>
  <span
  class="h-1.5 w-1.5 shrink-0 rounded-full"
  :class="serverStateOf(s) === 'online' ? 'bg-[#3fb950]' : serverStateOf(s) === 'offline' ? 'bg-[#f85149]' : 'bg-[var(--tx-muted)]'"
  :title="serverStatusText(s)"
  />
  <span
  v-if="serverPlayersOf(s).length > 0"
  class="shrink-0 rounded-full  bg-[#3fb950]/10 px-1.5 py-0.5 text-xs font-semibold text-[#3fb950]"
  :title="t('servers.players', { n: serverPlayersOf(s).length, names: serverPlayersOf(s).join(', ') })"
  >
  {{ serverStatuses[serverKey(s)]?.playersOnline }}/{{ serverStatuses[serverKey(s)]?.playersMax }}
  </span>
  </div>
  <div v-if="s.desc" class="mt-0.5 line-clamp-2 text-[13px] text-[color:var(--tx-muted)]">{{ s.desc }}</div>
  <div class="mt-1 truncate text-xs text-[color:var(--tx-muted)]" :title="serverStatusText(s)">
  {{ serverStatusText(s) }}
  </div>
  </div>
  <span
  v-if="s.port"
  class="shrink-0 rounded  bg-[var(--input)] px-1.5 py-0.5 font-mono text-xs text-[color:var(--tx-muted)]"
  >
  :{{ s.port }}
  </span>
  </div>
  <div class="mt-auto flex items-center justify-between gap-2 border-t border-[var(--border)]  pt-3">
  <code class="truncate font-mono text-[13px] text-[color:var(--tx)]">{{ s.ip }}{{ s.port ? `:${s.port}` : "" }}</code>
  <div class="flex shrink-0 gap-2">
  <button
  type="button"
  class="flex items-center gap-1.5 rounded-md  bg-[var(--input)] px-2.5 py-1 text-[13px] font-medium text-[color:var(--tx)] transition-colors  hover:text-[var(--accent)]"
  @click="copyServerIp(s)"
  >
  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
  <path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25Z"/>
  <path d="M5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"/>
  </svg>
  {{ t("servers.copy") }}
  </button>
  <button
  type="button"
  class="flex items-center gap-1.5 rounded-md bg-[#238636] px-2.5 py-1.5 text-[13px] font-semibold text-white transition-colors hover:bg-[#2ea043] disabled:cursor-not-allowed disabled:opacity-50"
  :disabled="gameRunning"
  @click="playOnServer(s)"
  >
  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
  <path d="M3.75 2a.75.75 0 0 1 .75.75V7h7V2.75a.75.75 0 0 1 1.5 0v10.5a.75.75 0 0 1-1.5 0V8.5h-7v4.75a.75.75 0 0 1-1.5 0V2.75a.75.75 0 0 1 .75-.75Z"/>
  </svg>
  {{ t("servers.play") }}
  </button>
  </div>
  </div>
  </div>
  </div>
  </div>
  </div>
  </template>

  <!-- Настройки сборки -->
  <template v-else-if="playSubTab === 'settings'">
  <div class="min-h-0 flex-1 overflow-y-auto pr-1">
  <div class="max-w-2xl space-y-6">
  <!-- ОЗУ -->
  <section class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <div class="border-b border-[var(--border)]  px-3.5 py-2.5 flex justify-between items-center">
  <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.ram") }}</h3>
  <span class="font-mono text-[13px] font-semibold text-[var(--accent)]">{{ ram }} {{ t("units.gb") }}</span>
  </div>
  <div class="p-4 space-y-2">
  <input
  type="range"
  min="2"
  :max="maxRam"
  step="1"
  v-model.number="ram"
  class="w-full accent-[var(--accent-deep)] bg-[var(--input)] h-1.5 rounded-lg appearance-none cursor-pointer"
  />
  <div class="flex justify-between text-[13px] text-[color:var(--tx-muted)] font-mono">
  <span>2 {{ t("units.gb") }}</span>
  <span>{{ t("settings.ramMax", { n: maxRam }) }}</span>
  </div>
  <p v-if="systemRam && systemRam.total_ram_gb > 0" class="text-[13px] text-[color:var(--tx-muted)]">
  {{ t("settings.ramTotal", { total: systemRam.total_ram_gb, avail: systemRam.available_ram_gb }) }}
  </p>
  <p
  v-if="activePack?.minRam"
  class="text-[13px]"
  :class="(ram * 1024) < activePack.minRam ? 'font-medium text-[#f0883e]' : 'text-[color:var(--tx-muted)]'"
  >
  {{ t("settings.ramMin", { name: activePack.name, min: activePack.minRam / 1024, gb: ram }) }}
  </p>
  </div>
  </section>

  <!-- JVM-аргументы -->
  <section class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <div class="border-b border-[var(--border)]  px-3.5 py-2.5 flex justify-between items-center">
  <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.jvmArgs") }}</h3>
  <button
  type="button"
  class="text-[13px] underline decoration-dotted underline-offset-2 disabled:opacity-50"
  :disabled="jvmArgsSaving"
  @click="saveJvmArgs"
  >
  {{ jvmArgsSaving ? t("common.saving") : t("common.save") }}
  </button>
  </div>
  <div class="p-4 space-y-2">
  <textarea
  v-model="jvmArgs"
  rows="3"
  spellcheck="false"
  class="w-full rounded-md bg-[var(--input)] border border-[var(--border)] px-3 py-2 font-mono text-[13px] text-[color:var(--tx)] focus:outline-none focus:border-[var(--accent)]"
  :placeholder="t('settings.jvmArgsHint')"
  ></textarea>
  <p class="text-[13px] text-[color:var(--tx-muted)]">{{ t("settings.jvmArgsNote") }}</p>
  </div>
  </section>

  <!-- Размер окна игры -->
  <section class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <div class="border-b border-[var(--border)]  px-3.5 py-2.5 flex justify-between items-center">
  <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.win") }}</h3>
  <span class="font-mono text-[13px] font-semibold text-[var(--accent)]">{{ windowWidth }}×{{ windowHeight }}</span>
  </div>
  <div class="p-4 space-y-2">
  <div class="flex items-center gap-3">
  <label class="w-16 text-[13px] text-[color:var(--tx-muted)]" for="win-width">{{ t("settings.width") }}</label>
  <input
  id="win-width"
  type="number"
  min="320"
  max="7680"
  step="1"
  v-model.number="windowWidth"
  class="flex-1 rounded-md  bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)]  focus:outline-none"
  />
  <label class="w-16 text-[13px] text-[color:var(--tx-muted)]" for="win-height">{{ t("settings.height") }}</label>
  <input
  id="win-height"
  type="number"
  min="240"
  max="4320"
  step="1"
  v-model.number="windowHeight"
  class="flex-1 rounded-md  bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)]  focus:outline-none"
  />
  </div>
  <p class="text-[13px] text-[color:var(--tx-muted)]">
  {{ t("settings.winNote") }}
  </p>
  </div>
  </section>

  <!-- Java -->
  <section class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <div class="border-b border-[var(--border)]  px-3.5 py-2.5">
  <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.java") }}</h3>
  </div>
  <div class="p-4 space-y-3">
  <div class="flex items-center gap-2">
  <select
  :value="javaSelected"
  class="flex-1 appearance-none rounded-md  bg-[var(--input)] px-2.5 py-1.5 pr-8 text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]  focus:outline-none"
  :disabled="javaBusy || busy"
  @change="onJavaChange"
  >
  <option value="">{{ t("settings.javaAuto") }}</option>
  <option v-for="j in javaList" :key="j.path" :value="j.path">
  {{ j.label }} — {{ j.version }} [{{ javaArchLabel(j.arch) }}]
  </option>
  </select>
  <button
  type="button"
  class="shrink-0 rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] disabled:opacity-50"
  :disabled="javaBusy || busy"
  @click="downloadJava"
  >
  {{ javaBusy ? t("settings.javaDownloading") : t("settings.javaDownload") }}
  </button>
  </div>
  <p v-if="javaMsg" class="text-[13px] text-[color:var(--tx-muted)] break-all">{{ javaMsg }}</p>
  <p class="text-[13px] text-[color:var(--tx-muted)]">
  {{ t("settings.javaNote") }}
  </p>
  </div>
  </section>

  <!-- Discord Rich Presence -->
  <section class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <div class="border-b border-[var(--border)]  px-3.5 py-2.5">
  <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.discord") }}</h3>
  </div>
  <div class="p-4">
  <label class="flex cursor-pointer items-center gap-3">
  <input
  type="checkbox"
  class="h-4 w-4 accent-[#5865F2]"
  :checked="discordRp"
  @change="toggleDiscordRp(($event.target as HTMLInputElement).checked)"
  />
  <span class="text-[13px] text-[color:var(--tx)]">{{ t("settings.discordLabel") }}</span>
  </label>
  <p class="mt-2 text-[13px] text-[color:var(--tx-muted)]">
  {{ t("settings.discordNote") }}
  </p>
  </div>
  </section>

  <!-- Система: трей + автозапуск -->
  <section class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <div class="border-b border-[var(--border)]  px-3.5 py-2.5">
  <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.system") }}</h3>
  </div>
  <div class="space-y-3 p-4">
  <label class="flex cursor-pointer items-center gap-3">
  <input
  type="checkbox"
  class="h-4 w-4 accent-[#5865F2]"
  :checked="closeToTray"
  @change="toggleCloseToTray(($event.target as HTMLInputElement).checked)"
  />
  <span class="text-[13px] text-[color:var(--tx)]">{{ t("settings.closeToTray") }}</span>
  </label>
  <label class="flex cursor-pointer items-center gap-3">
  <input
  type="checkbox"
  class="h-4 w-4 accent-[#5865F2]"
  :checked="autostartOn"
  @change="toggleAutostart(($event.target as HTMLInputElement).checked)"
  />
  <span class="text-[13px] text-[color:var(--tx)]">{{ t("settings.autostart") }}</span>
  </label>
  </div>
  </section>

  <!-- Предупреждение о кастомных модах -->
  <section class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <div class="border-b border-[var(--border)]  px-3.5 py-2.5">
  <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.warnCustomMods") }}</h3>
  </div>
  <div class="p-4">
  <label class="flex cursor-pointer items-center gap-3">
  <input
  type="checkbox"
  class="h-4 w-4 accent-[#f0883e]"
  :checked="warnCustomMods"
  @change="toggleWarnCustomMods(($event.target as HTMLInputElement).checked)"
  />
  <span class="text-[13px] text-[color:var(--tx)]">{{ t("settings.warnCustomModsLabel") }}</span>
  </label>
  <p class="mt-2 text-[13px] text-[color:var(--tx-muted)]">
  {{ t("settings.warnCustomModsNote") }}
  </p>
  </div>
  </section>

  <!-- Проверка целостности -->
  <section class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <div class="border-b border-[var(--border)]  px-3.5 py-2.5 flex justify-between items-center">
  <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.verify") }}</h3>
  </div>
  <div class="p-4 space-y-3">
  <p class="text-[13px] text-[color:var(--tx-muted)]">
  {{ t("settings.verifyNote") }}
  </p>
  <button
  type="button"
  class="rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] disabled:opacity-50"
  :disabled="verifyBusy || busy"
  @click="handleVerify"
  >
  {{ verifyBusy ? t("settings.verifying") : t("settings.verifyBtn") }}
  </button>
  <div
  v-if="verifyResult"
  class="rounded-md bg-[var(--bg-60)] p-3 text-[13px]"
>
  <p class="font-medium" :class="verifyResult.broken.length === 0 ? 'text-[#3fb950]' : 'text-[#f85149]'">
  {{ verifyResult.broken.length === 0 ? t("settings.verifyOk") : t("settings.verifyBroken", { n: verifyResult.broken.length }) }}
  </p>
  <p class="mt-0.5 text-[color:var(--tx-muted)]">{{ t("settings.verifyStats", { checked: verifyResult.checked, ok: verifyResult.ok }) }}</p>
  <ul v-if="verifyResult.broken.length > 0" class="mt-2 max-h-32 space-y-1 overflow-y-auto font-mono text-xs text-[#f85149]">
  <li v-for="b in verifyResult.broken" :key="b">{{ b }}</li>
  </ul>
</div>

  </div>
  </section>
  </div>
  </div>
  </template>

  <!-- Консоль / логи -->
  <section v-else class="flex h-full min-h-0 flex-1 flex-col overflow-hidden rounded-xl  bg-[var(--panel)] shadow-sm">
  <div class="flex items-center justify-between border-b border-[var(--border)]  bg-[var(--input-50)] px-4 py-2">
  <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("console.title") }}</h3>
  <div class="flex items-center gap-3">
  <span class="text-xs tabular-nums text-[var(--tx-muted)]">
  {{ t("console.lines", { n: logEntries.length }) }}
  </span>
  <div class="flex gap-2">
  <button
  type="button"
  class="text-[13px] text-[color:var(--tx-muted)] hover:text-[var(--accent)]"
  @click="handleCopyLog"
  >
  {{ t("console.copy") }}
  </button>
  <button
  type="button"
  class="text-[13px] text-[color:var(--tx-muted)] hover:text-[#f85149]"
  @click="handleClearLog"
  >
  {{ t("console.clear") }}
  </button>
  <button
  type="button"
  class="text-[13px] text-[color:var(--tx-muted)] hover:text-[var(--accent)]"
  @click="openFolder('logs')"
  >
  {{ t("console.logs") }}
  </button>
  </div>
  </div>
  </div>
  <div
  ref="logRef"
  class="flex-1 select-text overflow-y-auto bg-[var(--bg)] p-3 font-mono text-[13px] leading-relaxed text-[color:var(--tx-muted)]"
  >
  <p v-if="logEntries.length === 0" class="italic text-[var(--tx-muted)]">
  {{ t("console.empty") }}
  </p>
  <div
  v-for="(e, i) in logEntries"
  :key="i"
  :class="{
  'text-[#f85149]': e.stream === 'err',
  'text-[var(--accent)]': e.stream === 'sys',
  'text-[color:var(--tx)]': e.stream === 'out',
  'font-bold !text-[#f85149]': e.fatal,
  }"
  >
  {{ e.line }}
  </div>
  </div>
  </section>

  <!-- ======= Просмотр ресурса (страница в лаунчере): обновить + перейти на сервис ======= -->
  <div
  v-if="fileDetail && fileDetail.folder === playSubTab"
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-6"
  @click.self="fileDetail = null"
  >
  <div class="flex max-h-[82vh] w-full max-w-xl flex-col overflow-hidden rounded-xl  bg-[var(--panel)] shadow-2xl">
  <div class="flex shrink-0 items-center justify-between border-b border-[var(--border)]  px-3.5 py-2.5">
  <h3 class="truncate text-sm font-semibold text-[color:var(--tx-strong)]">
  {{ fileDetail.entry.displayName }}
  </h3>
  <button
  type="button"
  class="rounded-md p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
  @click="fileDetail = null"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
  </button>
  </div>

  <div class="flex min-h-0 flex-col gap-3 overflow-y-auto p-4">
  <div class="flex items-center gap-3 rounded-md  bg-[var(--bg)] px-3 py-2.5">
  <img
  v-if="fileDetailMr?.iconUrl"
  :src="fileDetailMr.iconUrl"
  :alt="fileDetailMr.title"
  loading="lazy"
  class="h-11 w-11 shrink-0 rounded-md object-cover"
  />
  <img
  v-else-if="fileDetailCf?.iconUrl"
  :src="fileDetailCf.iconUrl"
  :alt="fileDetailCf.name"
  loading="lazy"
  class="h-11 w-11 shrink-0 rounded-md object-cover"
  />
  <div v-else class="flex h-11 w-11 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-[13px] text-[color:var(--tx-muted)]">
  {{ (fileDetailMr?.title || fileDetailCf?.name || fileDetail.entry.displayName).slice(0, 2).toUpperCase() }}
  </div>
  <div class="min-w-0 flex-1">
  <h4 class="truncate text-sm font-semibold text-[color:var(--tx-strong)]">
  {{ fileDetailMr?.title || fileDetailCf?.name || fileDetail.entry.displayName }}
  </h4>
  <p class="truncate text-xs text-[color:var(--tx-muted)]">
  {{ fileDetail.entry.name }}
  <template v-if="fileDetailMr || fileDetailCf"> · {{ fileDetailMr ? t("mods.serviceModrinth") : t("mods.serviceCurseforge") }}</template>
  </p>
  </div>
  </div>

  <button
  type="button"
  class="flex shrink-0 items-center justify-center gap-1.5 rounded-md  bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-3 py-2 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
  :disabled="fileDetailMrLoading || fileDetailCfLoading || updatingFileDetail"
  @click="updateFileDetail()"
  >
  <svg v-if="updatingFileDetail" viewBox="0 0 16 16" class="h-4 w-4 animate-spin fill-current">
  <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
  </svg>
  <svg v-else viewBox="0 0 16 16" class="h-4 w-4 fill-current">
  <path d="M4.5 3.75a.75.75 0 0 0-1.5 0v2.5A.75.75 0 0 0 3.75 7h2.5a.75.75 0 0 0 0-1.5H5.07a4.5 4.5 0 1 1 .57 6.44.75.75 0 0 0-.98-1.13 6 6 0 1 0-.16-8.5v.49Z"/>
  </svg>
  {{ t("files.update") }}
  </button>

  <template v-if="fileDetailMr">
  <div class="rounded-md  bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx-muted)]">
  <p class="line-clamp-3">{{ fileDetailMr.description }}</p>
  <p class="mt-1 flex flex-wrap items-center gap-3">
  <span>{{ t("mods.byAuthor", { author: fileDetailMr.author }) }}</span>
  <span v-if="fileDetailMr.downloads">{{ fileDetailMr.downloads.toLocaleString() }} {{ t("mods.downloads") }}</span>
  <span v-if="fileDetailMr.categories.length">{{ fileDetailMr.categories.slice(0, 4).join(", ") }}</span>
  </p>
  </div>
  <div class="mb-3 flex items-center gap-1 border-b border-[var(--border)]  pb-2">
  <button
  v-for="tb in fileDetailTabs"
  :key="tb.kind"
  type="button"
  class="rounded-md px-2.5 py-1.5 text-[13px] font-medium transition-colors"
  :class="fileDetailTab === tb.kind
  ? 'bg-[var(--input)] text-[color:var(--tx-strong)]'
  : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]'"
  @click="fileDetailTab = tb.kind"
  >
  {{ t("mods.tab" + tb.kind) }}
  </button>
  </div>
  <div v-if="fileDetailTab === 'about'" class="max-h-[40vh] overflow-y-auto rounded-md  bg-[var(--bg)] px-3.5 py-2.5">
  <Markdown v-if="fileDetailMr.body" :source="fileDetailMr.body" />
  <p v-else class="py-6 text-center text-[13px] italic text-[color:var(--tx-muted)]">{{ t("mods.noAbout") }}</p>
  </div>
  <div v-else-if="fileDetailTab === 'versions'">
  <div class="mb-2 flex flex-wrap items-center gap-2">
  <FilterSelect v-model="fileDetailMcSel" :options="fileDetailMcOptions" :placeholder="t('curse.fVersion')" :multiple="true" />
  <FilterSelect v-model="fileDetailLoaderSel" :options="fileDetailLoaderOptions" :placeholder="t('mods.fLoader')" :multiple="true" />
  <FilterSelect v-model="fileDetailTypeSel" :options="versionTypeOptions" :placeholder="t('mods.fType')" :multiple="true" />
  </div>
  <div v-if="fileDetailMrVersions === null" class="flex items-center justify-center py-4 text-[13px] text-[color:var(--tx-muted)]">
  <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
  {{ t("mods.searching") }}
  </div>
  <div v-else-if="fileDetailFilteredVersions.length === 0" class="rounded-md  bg-[var(--input-50)] p-4 text-center text-[13px] text-[color:var(--tx-muted)]">
  {{ t("mods.noVersions") }}
  </div>
  <div v-else class="space-y-1">
  <button
  v-for="v in fileDetailFilteredVersions"
  :key="v.id"
  type="button"
  class="flex w-full items-center gap-2 rounded-md  bg-[var(--bg)] py-1.5 pl-2.5 pr-2 text-left transition-colors  disabled:opacity-50"
  :disabled="fileDetailMrVersionBusy !== null"
  @click="installFileDetailVersion(v)"
  >
  <span
  class="h-2 w-2 shrink-0 rounded-full"
  :style="{ backgroundColor: verTypeColor(v.versionType) }"
  :title="t('mods.verType.' + v.versionType)"
  ></span>
  <span class="min-w-0 flex-1">
<span class="flex items-center gap-1.5 text-[13px] font-medium text-[color:var(--tx-strong)]">
  <span class="truncate">{{ v.name }}</span>
  <span v-if="fileDetailInstalledVersion(v)" class="shrink-0 rounded-full  bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] px-1.5 py-px text-[11px] font-semibold text-[var(--accent)]">{{ t("mods.installedBadge") }}</span>
  </span>
  <span class="block truncate text-xs text-[color:var(--tx-muted)]">
  {{ v.loaders.map(cap).join(" · ") || "vanilla" }} · {{ v.gameVersions.slice(0, 2).join(", ") }} · {{ formatDate(v.datePublished) }}
  <template v-if="verInstallSize(v)"> · {{ formatBytes(verInstallSize(v)) }}</template>
  </span>
  </span>
  <span class="shrink-0 rounded  bg-[var(--input-50)] px-1.5 py-px font-mono text-[11px] text-[color:var(--tx-muted)]">{{ v.versionNumber }}</span>
  <template v-if="fileDetailMrVersionBusy === v.id">
  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 animate-spin fill-[var(--accent)]"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
  </template>
  <svg v-else viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-[var(--accent)]"><path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/></svg>
  </button>
  </div>
  </div>
  <div v-else>
  <div v-if="fileDetailMr.gallery.length" class="grid grid-cols-2 gap-2">
  <img
  v-for="g in fileDetailMr.gallery"
  :key="g.url"
  :src="g.url"
  :alt="g.title ?? ''"
  loading="lazy"
  class="h-32 w-full cursor-zoom-in rounded-md  object-cover transition-transform hover:scale-[1.02]"
  :title="g.title ?? undefined"
  @click="openExternal(g.url)"
  />
  </div>
  <p v-else class="py-10 text-center text-[13px] italic text-[color:var(--tx-muted)]">{{ t("mods.noGallery") }}</p>
  </div>
  </template>

  <template v-else-if="fileDetailCf">
  <div class="rounded-md  bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx-muted)]">
  <p class="max-h-40 overflow-y-auto whitespace-pre-wrap">{{ fileDetailCf.description }}</p>
  </div>
  </template>
  <p v-else-if="fileDetailMrLoading || fileDetailCfLoading" class="flex items-center justify-center py-6 text-[13px] text-[color:var(--tx-muted)]">
  <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
  {{ t("mods.searching") }}
  </p>
  </div>

  <div class="flex shrink-0 items-center justify-end gap-2 border-t border-[var(--border)]  px-3.5 py-2.5">
  <a
  v-if="fileDetailExternalUrl()"
  href="#"
  class="flex items-center gap-1.5 rounded-md  bg-[var(--bg)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click.prevent="openExternal(fileDetailExternalUrl()!)"
  >
  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M3.75 2h3.5a.75.75 0 0 1 0 1.5h-2l6 6V7.5a.75.75 0 0 1 1.5 0v4.5a.75.75 0 0 1-.75.75H5.5a.75.75 0 0 1 0-1.5h2l-6-6v2a.75.75 0 0 1-1.5 0V3.5A1.75 1.75 0 0 1 1.75 1.75h2a.75.75 0 0 1 0 1.5Z"/></svg>
  {{ t("files.openPage") }}
  </a>
  </div>
  </div>
  </div>
  </div>

  <!-- Контекстное меню: ПКМ по файлу/моду -->
  <Teleport to="body">
  <div v-if="fileCtx" class="fixed inset-0 z-[70]" @mousedown="closeFileCtx" @contextmenu.prevent="closeFileCtx">
    <div
      class="fixed z-[71] w-56 overflow-hidden rounded-xl bg-[var(--panel)] py-1 shadow-2xl"
      :style="fileCtxStyle"
      @mousedown.stop @contextmenu.stop
    >
      <div class="px-2.5 py-1.5">
        <div class="truncate text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ fileMetaTitle(fileCtx.file) }}</div>
        <div class="truncate text-xs text-[color:var(--tx-muted)]">{{ formatBytes(fileCtx.file.sizeBytes) }}</div>
      </div>
      <div class="mx-3 border-t border-[var(--border)]"></div>

      <button v-if="fileCtx.file.modrinthProjectId || fileCtx.file.curseforgeProjectId" type="button" class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]" @click="closeFileCtx(); openFileDetail(playSubTab as GameFolderKind, fileCtx!.file)">
        <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M8 3.75a3.25 3.25 0 1 0 0 6.5 3.25 3.25 0 0 0 0-6.5Zm0 8.5A8.75 8.75 0 0 1 0 8a8.75 8.75 0 0 1 8-4.25A8.75 8.75 0 0 1 16 8a8.75 8.75 0 0 1-8 4.25Z"/></svg>
        {{ t("files.view") }}
      </button>

      <button v-if="modUpdateFor(fileCtx.file)" type="button" class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[13px] text-[var(--accent)] transition-colors hover:bg-[var(--hover)] disabled:opacity-50" :disabled="updatingMod !== null || packLocked" @click="closeFileCtx(); updateOneMod(modUpdateFor(fileCtx!.file)!)">
        <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M4.5 3.75a.75.75 0 0 0-1.5 0v2.5A.75.75 0 0 0 3.75 7h2.5a.75.75 0 0 0 0-1.5H5.07a4.5 4.5 0 1 1 .57 6.44.75.75 0 0 0-.98-1.13 6 6 0 1 0-.16-8.5v.49Z"/></svg>
        {{ t("mods.update") }}
      </button>

      <button type="button" class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]" @click="closeFileCtx(); handleToggleFile(playSubTab as GameFolderKind, fileCtx!.file)">
        <svg v-if="fileCtx.file.enabled" viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M2 8a6 6 0 1 1 12 0A6 6 0 0 1 2 8Zm6-4.5a.75.75 0 0 1 .75.75v3.69l2.12 2.12a.75.75 0 1 1-1.06 1.06l-2.25-2.25a.75.75 0 0 1-.22-.53v-4a.75.75 0 0 1 .75-.75Z"/></svg>
        <svg v-else viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0Zm0 2a6 6 0 1 0 0 12A6 6 0 0 0 8 2Z"/></svg>
        {{ fileCtx.file.enabled ? t("files.disable") : t("files.enable") }}
      </button>

      <div v-if="!packLocked && fileCtx.file.kind === 'file'" class="mx-3 border-t border-[var(--border)]"></div>

      <button v-if="!packLocked && fileCtx.file.kind === 'file'" type="button" class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[13px] text-[#f85149] transition-colors hover:bg-[#f85149]/10 disabled:opacity-50" :disabled="fileDeleteBusy" @click="closeFileCtx(); clearFileSelection(); toggleFileSelect(playSubTab as GameFolderKind, fileCtx!.file); fileDeleteArmed = true">
        <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M6 1.75a.75.75 0 0 1 .75-.75h2.5a.75.75 0 0 1 .75.75V2h3.5a.75.75 0 0 1 0 1.5h-.38l-.89 10.055A1.75 1.75 0 0 1 10.495 15H5.505a1.75 1.75 0 0 1-1.735-1.445L2.88 3.5H2.5a.75.75 0 0 1 0-1.5H6v-.25ZM4.416 3.5l.864 9.9A.25.25 0 0 0 5.525 13.5h4.95a.25.25 0 0 0 .245-.22l.864-9.78H4.416Z"/></svg>
        {{ t("dev.remove") }}
      </button>

      <div class="mx-3 border-t border-[var(--border)]"></div>
      <button type="button" class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]" @click="closeFileCtx(); openFolder(playSubTab as GameFolderKind)">
        <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M.513 1.513A1.75 1.75 0 0 1 1.75 1h3.5c.55 0 1.07.26 1.4.7l.9 1.2a.25.25 0 0 0 .2.1H13a1 1 0 0 1 1 1v.5H2.75a.75.75 0 0 0 0 1.5h11.978a1 1 0 0 1 .994 1.117L15 13.25A1.75 1.75 0 0 1 13.25 15H1.75A1.75 1.75 0 0 1 0 13.25V2.75c0-.464.184-.91.513-1.237Z"/></svg>
        {{ t("files.open") }}
      </button>
    </div>
  </div>
  </Teleport>
</template>
