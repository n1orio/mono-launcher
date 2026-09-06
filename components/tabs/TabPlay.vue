<script setup lang="ts">
import { packGradient } from "~/lib/misc";
import { useLauncherCtx } from '~/composables/useLauncherContext';
const ctx = useLauncherCtx();
const {
  t,
  tp,
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

// ---- Баннер сторонних (кастомных) файлов: safe / unchecked / dangerous ----
const customModsState = computed(() => {
  const files: any[] = status?.value?.custom_mods || [];
  const unchecked = files.filter((f) => f.safe !== true && f.safe !== false);
  const dangerous = files.filter((f) => f.safe === false);
  if (files.length > 0 && unchecked.length === 0 && dangerous.length === 0) return "safe";
  if (dangerous.length > 0) return "dangerous";
  return "unchecked";
});
const customUncheckedCount = computed(() =>
  (status?.value?.custom_mods || []).filter((f: any) => f.safe !== true && f.safe !== false).length
);

// ---- Вкладка «Релизы»: hero активной версии + единый таймлайн без дублей ----
const normTag = (s: string | null | undefined) => (s ?? "").trim().replace(/^v/i, "");
const withV = (s: string | null | undefined) => {
  const t = (s ?? "").trim();
  return t ? (t.toLowerCase().startsWith("v") ? t : `v${t}`) : "";
};
const activeInstalled = computed(() => {
  const v = versions?.value;
  if (!v?.active) return null;
  return (v.installed ?? []).find((r: any) => r.version_id === v.active) ?? null;
});
interface TimelineRow { tag: string; display: string; installed: any | null; remote: any | null }
const versionTimeline = computed<TimelineRow[]>(() => {
  const rows: TimelineRow[] = [];
  const byTag = new Map<string, TimelineRow>();
  for (const rv of (remoteVersions?.value ?? []) as any[]) {
    const tag = normTag(rv.version);
    const row: TimelineRow = { tag, display: withV(rv.version), installed: null, remote: rv };
    rows.push(row);
    if (tag && !byTag.has(tag)) byTag.set(tag, row);
  }
  for (const ins of (versions?.value?.installed ?? []) as any[]) {
    const tag = normTag(ins.source_tag ?? ins.version_id);
    const ex = tag ? byTag.get(tag) : undefined;
    if (ex) {
      ex.installed = ins;
    } else {
      rows.push({ tag, display: withV(ins.source_tag ?? ins.version_id), installed: ins, remote: null });
    }
  }
  return rows;
});
const isRowActive = (row: TimelineRow) =>
  !!versions?.value?.active && !!row.installed && row.installed.version_id === versions.value.active;
const heroMeta = computed(() => {
  const parts: string[] = [];
  const secs = activeInstalled.value?.total_seconds ?? 0;
  if (secs > 0) parts.push(t("releases.inGame", { t: formatPlaytimeShort(secs) }));
  const st = status?.value;
  const loader = [st?.loader, st?.loader_version].filter(Boolean).join(" ");
  if (loader) parts.push(t("releases.loaderIs", { v: loader }));
  else if (st?.minecraft_version) parts.push(st.minecraft_version);
  return parts.join(" • ");
});
function openModsOfActive() {
  (playSubTab as any).value = "mods";
}
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
  <div v-else class="w-[60px] h-[60px] rounded-2xl flex items-center justify-center text-white font-black text-xl select-none shrink-0 shadow-lg" :style="{ background: packGradient(activePack?.name || 'T') }">
  <span>{{ (activePack?.name || 'T')[0].toUpperCase() }}</span>
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
  : 'bg-[#16a34a] hover:bg-[#15803d] text-white shadow-lg'
  : 'bg-[var(--accent-deep)] hover:bg-[var(--accent-hover)]'"
  :disabled="busy"
  @click="status?.installed ? (gameRunning ? handleStop() : handlePlay()) : handleInstall()"
  >
  <AppIcon v-if="busy" name="spinner" class="h-4 w-4 fill-current" />
  <AppIcon v-else-if="status?.installed && !gameRunning" name="play" class="h-4 w-4 fill-current" />
  <AppIcon v-else-if="gameRunning" name="stop" class="h-4 w-4 fill-current" />
  <AppIcon v-else name="arrow-down" class="h-4 w-4 fill-current" />
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
  <AppIcon name="folder" class="h-4 w-4 fill-current" />
  {{ t("pack.folder") }}
  </button>
  <button
  v-if="activePack?.url"
  type="button"
  class="flex items-center gap-1.5 rounded-lg  bg-[var(--input)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)]"
  :title="t('pack.copyLink')"
  @click="copyPackDeepLink(activePack)"
  >
  <AppIcon name="link" class="h-4 w-4 fill-current" />
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
  <AppIcon name="arrow-up" class="h-4 w-4 fill-current" />
  <span>{{ t("pack.exportBtn") }}</span>
  <AppIcon name="chevron-down" class="h-3 w-3 fill-current opacity-60" />
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
  <AppIcon name="cloud-download" class="h-4 w-4 fill-current opacity-70" />
  .mrpack
  </button>
  <button
  type="button"
  class="flex items-center gap-2 rounded px-2 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  :disabled="exportBusy"
  @click="exportMenuOpen = false; openAuthorExport()"
  >
  <AppIcon name="plus" class="h-4 w-4 fill-current opacity-70" />
  {{ t("pack.exportAuthorShort") }}
  </button>
  <button
  type="button"
  class="flex items-center gap-2 rounded px-2 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  :disabled="exportBusy"
  @click="exportMenuOpen = false; openExport('curseforge')"
  >
  <AppIcon name="cloud-download" class="h-4 w-4 fill-current opacity-70" />
  CurseForge
  </button>
  </div>
  </div>
  </template>
  </div>
  </div>
  </div>



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
  ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
  : ' bg-[color-mix(in_srgb,var(--accent-deep)_10%,transparent)]'"
  >
  <div v-if="licenseInfo?.subscribed" class="flex items-center justify-between gap-3">
  <span class="flex min-w-0 items-center gap-2">
  <AppIcon name="lock" class="h-4 w-4 fill-current" />
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
  <AppIcon name="lock" class="h-4 w-4 fill-[var(--accent)]" />
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
  <AppIcon name="lock" class="h-4 w-4 fill-current" />
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
  <AppIcon name="spinner" class="h-4 w-4 fill-current" />
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

  <!-- Managed pack Banner: там же и в том же корпусе, что баннер проверки -->
  <div
    v-if="packLocked"
    class="rounded-xl border px-3.5 py-2.5 my-3 text-xs flex items-center gap-2 font-medium transition-all border-[#16a34a]/30 bg-[#16a34a]/15 text-[#22c55e]"
    :title="t('files.locked')"
  >
    <AppIcon name="shield-check" class="h-4 w-4 fill-current shrink-0" />
    <span>{{ t("pack.managed") }}</span>
  </div>

  <!-- Verification Banner: вид зависит от состояния проверки -->
  <div
    v-if="warnCustomMods && (status?.custom_mods?.length || 0) > 0"
    class="rounded-xl border px-3.5 py-2.5 my-3 text-xs flex items-center justify-between transition-all"
    :class="customModsState === 'safe'
      ? 'border-[#16a34a]/30 bg-[#16a34a]/15 text-[#22c55e]'
      : customModsState === 'dangerous'
        ? 'border-red-500/30 bg-red-500/10 text-red-300'
        : 'border-amber-500/30 bg-amber-500/10 text-amber-300'"
  >
    <div class="flex items-center gap-2 font-medium">
      <AppIcon v-if="customModsState === 'safe'" name="shield-check" class="h-4 w-4 fill-current shrink-0" />
      <AppIcon v-else-if="customModsState === 'dangerous'" name="alert-circle" class="h-4 w-4 fill-current shrink-0" />
      <AppIcon v-else name="shield_alert" class="h-4 w-4 fill-current shrink-0" />
      <span v-if="customModsState === 'safe'">Сторонние файлы проверены сканером (угроз не найдено)</span>
      <span v-else-if="customModsState === 'dangerous'">Сканер обнаружил опасные файлы — запуск небезопасен</span>
      <span v-else>{{ tp("customMods.unchecked", customUncheckedCount) }}</span>
    </div>
    <div class="flex items-center gap-3 shrink-0">
      <button type="button" class="hover:underline font-semibold cursor-pointer disabled:opacity-50" :disabled="customScanBusy" @click="scanActiveCustomMods">
        {{ customModsState === 'safe' ? 'Пересканировать' : 'Сканировать' }}
      </button>
      <button v-if="customModsState === 'safe'" type="button" class="hover:underline font-semibold cursor-pointer" @click="customModsOpen = !customModsOpen">
        {{ customModsOpen ? 'Скрыть список' : 'Список' }}
      </button>
      <button type="button" class="text-current opacity-60 hover:opacity-100 ml-1 flex items-center" @click="warnCustomMods = false" aria-label="Закрыть">
        <AppIcon name="x" class="h-3.5 w-3.5 fill-current" />
      </button>
    </div>
  </div>

  <!-- Expandable Custom Mods List -->
  <div v-if="customModsOpen && customModsState === 'safe' && status?.custom_mods?.length" class="rounded-xl bg-[var(--input)]/30 border border-[var(--border)] p-3 mb-3 flex flex-col gap-1.5 text-xs">
    <div class="font-bold text-[color:var(--tx)] mb-1">Кастомные файлы в сборке:</div>
    <div v-for="f in (status?.custom_mods || [])" :key="f.path" class="flex items-center justify-between gap-2 py-1 px-2 rounded-lg bg-[var(--panel)] border border-[var(--border)] font-mono text-[11px] text-[color:var(--tx-muted)]">
      <span class="truncate">{{ f.path }}</span>
      <span class="shrink-0 flex items-center gap-1 bg-[#16a34a]/15 text-[#22c55e] border border-[#16a34a]/30 px-2 py-0.5 rounded text-[11px] font-semibold font-sans"><AppIcon name="check" class="h-3 w-3 fill-current" />Безопасно</span>
    </div>
    <span class="text-[11px] text-[color:var(--tx-muted)]/70 mt-1 block">Файлы успешно прошли проверку на вредоносный код. Ответственность за совместимость и стабильность лежит на пользователе.</span>
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

  <!-- Управление версиями: hero активной + единый таймлайн -->
  <template v-if="playSubTab === 'releases'">
  <div class="min-h-0 flex-1 space-y-4 overflow-y-auto pr-1">

  <!-- Hero: текущая установленная версия -->
  <div v-if="activeInstalled" class="rounded-2xl bg-[var(--input)]/40 border border-[var(--border)] p-4 mb-5 flex items-center justify-between gap-4 shadow-sm">
    <div class="flex items-center gap-3.5 min-w-0">
      <div class="w-10 h-10 shrink-0 rounded-xl bg-[#16a34a]/15 border border-[#16a34a]/30 flex items-center justify-center text-[#22c55e]">
        <AppIcon name="check" class="h-5 w-5 fill-current" />
      </div>
      <div class="flex flex-col min-w-0">
        <div class="text-[11px] font-semibold uppercase tracking-wider text-[color:var(--tx-muted)]">{{ t("releases.current") }}</div>
        <div class="flex items-center gap-2 flex-wrap">
          <span class="text-base font-black text-[color:var(--tx)]">{{ withV(activeInstalled.source_tag ?? activeInstalled.version_id) }}</span>
          <span class="px-2 py-0.5 rounded-md bg-[#16a34a]/15 text-[#22c55e] text-[11px] font-bold border border-[#16a34a]/25">{{ t("releases.active") }}</span>
        </div>
        <span v-if="heroMeta" class="text-xs text-[color:var(--tx-muted)] mt-0.5 truncate">{{ heroMeta }}</span>
      </div>
    </div>
    <div class="flex items-center gap-2 shrink-0">
      <button type="button" class="px-3.5 py-2 rounded-xl bg-[var(--input)] hover:bg-[var(--panel)] border border-[var(--border)] text-xs font-semibold text-[color:var(--tx)] transition-all active:scale-95 shadow-sm" @click="openModsOfActive">
        {{ t("releases.modsOfVersion") }}
      </button>
    </div>
  </div>

  <!-- Таймлайн: серверные версии + локальные, дубли склеены -->
  <div class="flex items-center justify-between text-[13px] text-[color:var(--tx-muted)]">
    <span class="font-medium">{{ t("releases.timeline", { n: versionTimeline.length }) }}</span>
    <button type="button" class="flex items-center gap-1.5 rounded-md bg-[var(--input)] px-2 py-1 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50" :disabled="remoteVersionsLoading" @click="refreshRemoteVersions()">
      <AppIcon v-if="remoteVersionsLoading" name="spinner" class="h-3 w-3 fill-current" />
      <AppIcon v-else name="refresh" class="h-3 w-3 fill-current" />
      {{ t("catalog.refresh") }}
    </button>
  </div>

  <article
    v-for="row in versionTimeline"
    :key="row.tag || row.display"
    class="rounded-xl bg-[var(--input)]/20 hover:bg-[var(--input)]/40 border border-[var(--border)] p-3 mb-2 flex items-center justify-between gap-4 transition-all"
  >
    <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2 flex-wrap">
          <span class="text-sm font-bold text-[color:var(--tx)]">{{ row.display }}</span>
          <span v-if="isRowActive(row)" class="rounded-full bg-[#238636]/10 px-2 py-0.5 text-xs font-medium text-[#3fb950]">{{ t("releases.active") }}</span>
          <span v-else-if="row.installed" class="rounded-full bg-white/5 border border-[var(--border)] px-2 py-0.5 text-xs font-medium text-[color:var(--tx-muted)]">{{ t("releases.downloaded") }}</span>
        </div>
        <p v-if="row.remote" class="mt-0.5 text-xs text-[color:var(--tx-muted)]">
          {{ formatDate(row.remote.created_at) }} · {{ formatBytes(row.remote.size) }}<span v-if="row.installed && row.installed.total_seconds > 0" :title="t('releases.playtime')"> · {{ formatPlaytime(row.installed.total_seconds) }}</span>
        </p>
        <p v-else-if="row.installed && row.installed.total_seconds > 0" class="mt-0.5 font-mono text-xs text-[#d29922]" :title="t('releases.playtime')">
          {{ formatPlaytime(row.installed.total_seconds) }}
        </p>
        <p v-if="row.remote?.changelog" class="mt-1 line-clamp-1 text-xs text-[color:var(--tx-muted)]">{{ row.remote.changelog }}</p>
      </div>
      <button
        v-if="!row.installed && row.remote"
        type="button"
        class="shrink-0 px-3.5 py-1.5 rounded-xl text-xs font-semibold bg-[var(--accent-deep)] hover:brightness-110 text-white shadow-sm active:scale-95 transition-all disabled:opacity-50"
        :disabled="busy || remoteInstallingId === row.remote.id"
        @click="installRemoteVersion(row.remote)"
      >
        <AppIcon v-if="remoteInstallingId === row.remote.id" name="spinner" class="h-3.5 w-3.5 fill-current" />
        <template v-else>{{ t("releases.install") }}</template>
      </button>
      <button
        v-else-if="row.installed && !isRowActive(row)"
        type="button"
        class="shrink-0 px-3.5 py-1.5 rounded-xl text-xs font-semibold bg-[var(--input)] hover:bg-[var(--panel)] text-[color:var(--tx)] border border-[var(--border)] transition-all disabled:opacity-50"
        :disabled="busy"
        @click="handleSelectVersion(row.installed.source_tag ?? row.installed.version_id)"
      >
        {{ t("releases.switch") }}
      </button>
  </article>

  </div>

  <div v-if="remoteVersionsLoading && versionTimeline.length === 0" class="shrink-0 rounded-xl bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]">
  {{ t("files.loading") }}
  </div>
  <div v-else-if="!remoteVersions && versionTimeline.length === 0" class="shrink-0 rounded-xl bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]">
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
  <AppIcon name="lock" class="h-4 w-4 fill-[var(--accent)]" />
  {{ t("files.locked") }}
  </span>
  <button
  type="button"
  class="shrink-0 rounded px-2 py-1 font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_15%,transparent)]"
  :title="t('files.unbindHint')"
  @click="confirmUnbindPack"
  >{{ unbindArmed ? t("files.unbindConfirm") : t("files.unbind") }}</button>
  </div>
  <!-- Unified toolbar single row -->
  <div class="flex items-center justify-between gap-3 my-3">
  <div class="gap-2.5 flex items-center flex-1 min-w-0 max-w-xl">
  <div class="relative flex-1 min-w-0">
  <svg viewBox="0 0 16 16" class="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 fill-[var(--tx-muted)]"><path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"/></svg>
  <input v-model="fileSearch" type="text" placeholder="Поиск файлов..." class="w-64 min-w-[180px] rounded-xl bg-[var(--input)] border border-[var(--border)] pl-8 pr-3 py-1.5 text-xs text-[color:var(--tx)] placeholder:text-[color:var(--tx-muted)] focus:outline-none focus:border-[var(--accent)] transition-all" />
  </div>
  <div class="flex items-center p-0.5 rounded-xl bg-[var(--input)] border border-[var(--border)] text-xs shrink-0">
  <button
  type="button"
  class="hidden"
  :class="fileSortKey === 'name' ? 'bg-[var(--accent-deep)] text-white' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
  :title="fileSortKey === 'name' ? (fileSortDir === 'asc' ? t('files.sortNameAsc') : t('files.sortNameDesc')) : t('files.sortNameHint')"
  @click="toggleFileSort('name')"
  >
  <AppIcon name="filter" class="h-3 w-3 fill-current" />
  {{ t("files.sortName") }}
  <AppIcon v-if="fileSortKey === 'name'" name="sort-desc" class="h-2.5 w-2.5 fill-current" :style="{ transform: fileSortDir === 'asc' ? 'none' : 'rotate(180deg)' }" />
  </button>
  <button
  type="button"
  class="flex items-center gap-1 rounded-md px-2.5 py-1 text-[13px] font-medium transition-colors"
  :class="fileSortKey === 'date' ? 'bg-[var(--accent-deep)] text-white' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
  :title="fileSortKey === 'date' ? (fileSortDir === 'desc' ? t('files.sortDateNew') : t('files.sortDateOld')) : t('files.sortDateHint')"
  @click="toggleFileSort('date')"
  >
  <AppIcon name="calendar" class="h-3 w-3 fill-current" />
  {{ t("files.sortDate") }}
  <AppIcon v-if="fileSortKey === 'date'" name="sort-desc" class="h-2.5 w-2.5 fill-current" :style="{ transform: fileSortDir === 'asc' ? 'none' : 'rotate(180deg)' }" />
  </button>
  <button
  v-if="fileSortKey !== 'none'"
  type="button"
  class="rounded-md px-1.5 py-1 text-[13px] leading-none text-[color:var(--tx-muted)] transition-colors hover:text-[var(--accent)]"
  :title="t('files.sortReset')"
  @click="clearFileSort"
  >×</button>
  </div>
  <div class="flex items-center p-0.5 rounded-xl bg-[var(--input)] border border-[var(--border)] text-xs shrink-0">
  <button :class="fileStatusFilter === 'all' ? 'bg-[var(--panel)] text-[color:var(--tx)] font-semibold shadow-sm' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx)]'" class="px-2.5 py-1 rounded-lg transition-all" :title="t('files.fAllHint')"
  @click="setFileStatusFilter('all')"
  >{{ t("files.fAll") }}</button>
  <button
  type="button"
  class="rounded-md px-2.5 py-1 text-[13px] font-medium transition-colors"
  :title="t('files.fEnabledHint')"
  :class="fileStatusFilter === 'enabled' ? 'bg-[var(--accent-deep)] text-white' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
  @click="setFileStatusFilter('enabled')"
  >{{ t("files.fEnabled") }}</button>
  <button
  type="button"
  class="rounded-md px-2.5 py-1 text-[13px] font-medium transition-colors"
  :title="t('files.fDisabledHint')"
  :class="fileStatusFilter === 'disabled' ? 'bg-[var(--accent-deep)] text-white' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
  @click="setFileStatusFilter('disabled')"
  >{{ t("files.fDisabled") }}</button>
  <button
  type="button"
  class="rounded-md px-2.5 py-1 text-[13px] font-medium transition-colors"
  :title="t('files.fUpdatesHint')"
  :class="fileStatusFilter === 'updates' ? 'bg-[var(--accent-deep)] text-white' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
  @click="setFileStatusFilter('updates')"
  >{{ t("files.fUpdates") }}</button>
  </div>
  </div>
  <div class="flex items-center gap-2 shrink-0">
  <span class="text-xs text-[color:var(--tx-muted)] tabular-nums mr-2">{{ fileVisibleCount }} мода</span>
  <button v-if="modUpdatesTab.length>0" class="px-3 py-1.5 rounded-xl bg-[var(--accent-deep)] hover:brightness-110 text-white text-xs font-semibold flex items-center gap-1.5 shadow-md active:scale-95 transition-all" @click="updateAllMods"><span>Обновить все</span><span class="px-1.5 py-0.2 rounded-md bg-white/25 text-[10px] font-bold">{{ modUpdatesTab.length }}</span></button>
  <button class="px-3.5 py-1.5 rounded-xl bg-[var(--accent-deep)] hover:brightness-110 text-white text-xs font-semibold flex items-center gap-1.5 shadow-md" @click="openSearch((playSubTab === 'mods' ? 'mod' : playSubTab === 'resourcepacks' ? 'resourcepack' : 'shaderpack') as ModrinthSearchKind, 'modrinth')"><span>+ Добавить мод</span></button>
  <div ref="fileMenuRef" class="relative">
  <button
  type="button"
  class="flex shrink-0 items-center justify-center rounded-lg bg-[var(--bg)] px-2 py-1.5 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[var(--tx)]"
  :title="t('files.more')"
  @click="fileMenuOpen = !fileMenuOpen"
  >
  <AppIcon name="dots" class="h-4 w-4 fill-current" />
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
  <AppIcon name="folder" class="h-4 w-4 fill-current" />
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
  <AppIcon v-if="fileDeleteBusy" name="spinner" class="h-4 w-4 fill-current" />
  <svg v-else viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M6.5 1.75A1.75 1.75 0 0 1 8.25 0h2.5A1.75 1.75 0 0 1 12 1.75V3h2.25a.75.75 0 0 1 0 1.5h-.65l-.75 9.006A1.75 1.75 0 0 1 10.738 15H5.262a1.75 1.75 0 0 1-1.742-1.494L2.77 4.5H2.12a.75.75 0 0 1 0-1.5H4.5V1.75ZM5.07 4.5l.76 8.91a.25.25 0 0 0 .25.214h5.456a.25.25 0 0 0 .25-.214L12.54 4.5H5.07Z"/></svg>
  {{ t("files.delete") }} ({{ Object.keys(selectedFiles).length }})
  </button>
  </div>
  </div>
  </div>
  </div>

  <div v-if="!gameFiles[playSubTab]" class="flex flex-1 items-center justify-center text-[13px] text-[color:var(--tx-muted)]">
  <AppIcon name="spinner" class="mr-2 h-4 w-4 fill-[var(--accent)]" />
  {{ t("files.loading") }}
  </div>
  <div v-else-if="(gameFiles[playSubTab] ?? []).length === 0" class="shrink-0 rounded-xl  bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]">
  <p class="mb-3">{{ t("files.empty") }}</p>
  <button
  type="button"
  class="inline-flex items-center gap-1.5 rounded-md  bg-[var(--input)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click="openFolder(playSubTab as GameFolderKind)"
  >
  <AppIcon name="folder" class="h-3 w-3 fill-current" />
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
  class="file-row flex cursor-pointer items-center justify-between gap-4 rounded-xl bg-[var(--input)]/25 hover:bg-[var(--input)]/50 border border-[var(--border)] p-3 mb-2 transition-all"
  :class="{ 'opacity-60': !f.enabled }"
  @click="toggleFileSelect(playSubTab as GameFolderKind, f)"
  @contextmenu.prevent="openFileCtx($event, f)"
  >

  <!-- LEFT GROUP: Icon + (Title & Subtitle) -->
  <div class="flex items-center gap-3.5 min-w-0 flex-1">
  <div class="w-10 h-10 rounded-xl bg-[var(--input)] border border-[var(--border)] flex items-center justify-center shrink-0 overflow-hidden">
  <img
  v-if="modrinthMetaFor(f)?.icon || curseMetaFor(f)?.icon || gameFileIcon(playSubTab, f.name)"
  :src="modrinthMetaFor(f)?.icon || curseMetaFor(f)?.icon || gameFileIcon(playSubTab, f.name)"
  alt=""
  loading="lazy"
  draggable="false"
  class="h-full w-full object-contain p-1"
  />
  <span v-else class="text-sm font-black text-[color:var(--tx-muted)]">{{ fileMetaTitle(f)[0]?.toUpperCase() }}</span>
  </div>
  <div class="min-w-0 flex-1 text-left flex flex-col gap-0.5">
  <div
  class="truncate text-sm font-bold text-[color:var(--tx)] tracking-tight leading-tight"
  :title="fileMetaTitle(f)"
  >
  {{ fileMetaTitle(f) }}
  </div>
  <div class="flex items-center gap-2 text-xs text-[color:var(--tx-muted)] truncate">
  <span>{{ f.kind === "dir" ? t("files.dir") : formatBytes(f.sizeBytes) }}</span>
  <span class="opacity-40">•</span>
  <span :class="f.enabled ? '' : 'text-rose-400/80 font-medium'">{{ f.enabled ? t("files.enabled") : t("files.disabled") }}</span>
  </div>
  </div>
  </div>
  <!-- RIGHT GROUP: Badge + Eye + Update + Toggle -->
  <div class="flex items-center gap-3 shrink-0">
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
  class="flex h-8 w-8 shrink-0 items-center justify-center rounded-xl hover:bg-[var(--input)] text-[color:var(--tx-muted)] hover:text-[color:var(--tx)] transition-all"
  :title="t('files.view')"
  @click.stop="openFileDetail(playSubTab as GameFolderKind, f)"
  >
  <AppIcon name="eye" class="h-4 w-4 fill-current" />
  </button>
  <button
  v-if="playSubTab !== 'saves' && modUpdateFor(f)"
  type="button"
  class="flex shrink-0 items-center gap-1.5 rounded-xl bg-[var(--accent-deep)] hover:brightness-110 px-3 py-1.5 text-xs font-semibold text-white shadow-md transition-all active:scale-95 disabled:opacity-50"
  :disabled="updatingMod !== null || packLocked"
  :title="`${modUpdateFor(f)!.newVersion.name} (${modUpdateFor(f)!.newVersion.versionNumber})`"
  @click.stop="updateOneMod(modUpdateFor(f)!)"
  >
  <AppIcon v-if="updatingMod === f.name" name="spinner" class="h-3 w-3 fill-current" />
  <AppIcon v-else name="refresh" class="h-3 w-3 fill-current" />
  {{ t("mods.update") }}
  </button>
  <button
  v-if="f.kind === 'file'"
  type="button"
  class="relative h-6 w-11 shrink-0 rounded-full transition-colors duration-200 cursor-pointer shadow-inner"
  :class="[
  f.enabled ? 'bg-[#16a34a]' : 'bg-[var(--input)] border border-[var(--border)]',
  isFileToggling(playSubTab, f) ? 'opacity-50 cursor-wait' : '',
  ]"
  role="switch"
  :aria-checked="f.enabled"
  :disabled="isFileToggling(playSubTab, f) || packLocked"
  :title="isFileToggling(playSubTab, f) ? undefined : (packLocked ? t('files.locked') : (f.enabled ? t('files.disable') : t('files.enable')))"
  @click.stop="handleToggleFile(playSubTab as GameFolderKind, f)"
  >
  <span
  class="absolute top-0.5 flex h-5 w-5 items-center justify-center rounded-full bg-white shadow-md transition-all duration-200"
  :class="f.enabled ? 'left-[22px]' : 'left-0.5'"
  >
  <AppIcon v-if="isFileToggling(playSubTab, f)" name="spinner" class="h-2.5 w-2.5 fill-[var(--accent)]" />
  </span>
  </button>
  </div>
  </div>
  </div>
  </div>
  </div>
  </div>

  <!-- ======= Дубликаты (mods / resourcepacks / shaderpacks) ======= -->
  <template v-else-if="playSubTab === 'duplicates'">
  <div class="flex min-h-0 flex-1 flex-col">
  <div v-if="duplicatesLoading" class="flex flex-1 items-center justify-center text-[13px] text-[color:var(--tx-muted)]">
  <AppIcon name="spinner" class="mr-2 h-4 w-4 fill-[var(--accent)]" />
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
  <AppIcon name="spinner" class="mr-2 h-4 w-4 fill-[var(--accent)]" />
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
  <AppIcon v-if="updatingFileDetail" name="spinner" class="h-4 w-4 fill-current" />
  <AppIcon v-else name="refresh" class="h-4 w-4 fill-current" />
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
  <AppIcon name="spinner" class="mr-2 h-4 w-4 fill-current" />
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
  <AppIcon name="spinner" class="h-4 w-4 fill-[var(--accent)]" />
  </template>
  <AppIcon v-else name="arrow-down" class="h-4 w-4 fill-current" />
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
  <AppIcon name="spinner" class="mr-2 h-4 w-4 fill-current" />
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
  <AppIcon name="link" class="h-3 w-3 fill-current" />
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

      <button v-if="fileCtx.file.modrinthProjectId || fileCtx.file.curseforgeProjectId" type="button" class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]" @click="openFileDetail(playSubTab as GameFolderKind, fileCtx!.file); closeFileCtx()">
        <AppIcon name="eye" class="h-4 w-4 fill-current" />
        {{ t("files.view") }}
      </button>

      <button v-if="modUpdateFor(fileCtx.file)" type="button" class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[13px] text-[var(--accent)] transition-colors hover:bg-[var(--hover)] disabled:opacity-50" :disabled="updatingMod !== null || packLocked" @click="updateOneMod(modUpdateFor(fileCtx!.file)!); closeFileCtx()">
        <AppIcon name="refresh" class="h-4 w-4 fill-current" />
        {{ t("mods.update") }}
      </button>

      <button type="button" class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]" @click="handleToggleFile(playSubTab as GameFolderKind, fileCtx!.file); closeFileCtx()">
        <svg v-if="fileCtx.file.enabled" viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M2 8a6 6 0 1 1 12 0A6 6 0 0 1 2 8Zm6-4.5a.75.75 0 0 1 .75.75v3.69l2.12 2.12a.75.75 0 1 1-1.06 1.06l-2.25-2.25a.75.75 0 0 1-.22-.53v-4a.75.75 0 0 1 .75-.75Z"/></svg>
        <svg v-else viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0Zm0 2a6 6 0 1 0 0 12A6 6 0 0 0 8 2Z"/></svg>
        {{ fileCtx.file.enabled ? t("files.disable") : t("files.enable") }}
      </button>

      <div v-if="!packLocked && fileCtx.file.kind === 'file'" class="mx-3 border-t border-[var(--border)]"></div>

      <button v-if="!packLocked && fileCtx.file.kind === 'file'" type="button" class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[13px] text-[#f85149] transition-colors hover:bg-[#f85149]/10 disabled:opacity-50" :disabled="fileDeleteBusy" @click="clearFileSelection(); toggleFileSelect(playSubTab as GameFolderKind, fileCtx!.file); fileDeleteArmed = true; closeFileCtx()">
        <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M6 1.75a.75.75 0 0 1 .75-.75h2.5a.75.75 0 0 1 .75.75V2h3.5a.75.75 0 0 1 0 1.5h-.38l-.89 10.055A1.75 1.75 0 0 1 10.495 15H5.505a1.75 1.75 0 0 1-1.735-1.445L2.88 3.5H2.5a.75.75 0 0 1 0-1.5H6v-.25ZM4.416 3.5l.864 9.9A.25.25 0 0 0 5.525 13.5h4.95a.25.25 0 0 0 .245-.22l.864-9.78H4.416Z"/></svg>
        {{ t("dev.remove") }}
      </button>

      <div class="mx-3 border-t border-[var(--border)]"></div>
      <button type="button" class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]" @click="closeFileCtx(); openFolder(playSubTab as GameFolderKind)">
        <AppIcon name="folder" class="h-4 w-4 fill-current" />
        {{ t("files.open") }}
      </button>
    </div>
  </div>
  </Teleport>
</template>
