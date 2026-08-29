<script setup lang="ts">
import { useLauncherCtx } from "~/composables/useLauncherContext";
import { useI18n } from "#imports";

const { t } = useI18n();
const {
  isFileDetailWin,
  fileDetailMr,
  fileDetailCf,
  fileDetailTitle,
  fileDetailTab,
  fileDetailTabs,
  fileDetailMrLoading,
  fileDetailMrVersions,
  fileDetailMcSel,
  fileDetailMcOptions,
  fileDetailLoaderSel,
  fileDetailLoaderOptions,
  fileDetailTypeSel,
  versionTypeOptions,
  fileDetailFilteredVersions,
  fileDetailMrVersionBusy,
  fileDetailCfLoading,
  fileDetailCfMcSel,
  fileDetailCfMcOptions,
  fileDetailCfFilteredVersions,
  fileDetailCfVersionBusy,
  verTypeColor,
  formatDate,
  cap,
  verInstallSize,
  formatBytes,
  searchIconUrl,
  fileDetailExternalUrl,
  closeFileDetailWin,
  openExternal,
  installFileDetailVersion,
  installFileDetailCfVersion,
  fileDetailInstalledVersion,
} = useLauncherCtx();
</script>

<template>
  <div v-if="isFileDetailWin" class="fixed inset-0 z-50 flex flex-col overflow-hidden bg-[var(--bg)] text-[color:var(--tx)] font-sans">
    <SubTitleBar :title="fileDetailMr?.title || fileDetailCf?.name || fileDetailTitle" />
    <div class="flex shrink-0 items-center justify-between gap-3 border-b border-[var(--border)]  bg-[var(--panel)] px-4 py-2.5">
      <div class="flex min-w-0 flex-1 items-center gap-3">
        <img v-if="fileDetailMr?.iconUrl || fileDetailCf?.iconUrl" :src="searchIconUrl((fileDetailMr?.iconUrl ?? fileDetailCf?.iconUrl)!)" :alt="fileDetailMr?.title ?? fileDetailCf?.name ?? ''" loading="lazy" class="h-10 w-10 shrink-0 rounded-md object-cover" />
        <div v-else-if="fileDetailMr || fileDetailCf" class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-[13px] text-[color:var(--tx-muted)]">
          {{ (fileDetailMr?.title ?? fileDetailCf?.name ?? "?").slice(0, 2).toUpperCase() }}
        </div>
        <div class="min-w-0">
          <h2 class="truncate text-sm font-semibold text-[color:var(--tx-strong)]">{{ fileDetailMr?.title ?? fileDetailTitle }}</h2>
          <div class="flex flex-wrap items-center gap-x-3 gap-y-0.5 text-xs text-[color:var(--tx-muted)]">
            <template v-if="fileDetailMr">
              <span>{{ t("mods.byAuthor", { author: fileDetailMr.author }) }}</span>
              <span class="flex items-center gap-1">
                <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1.75a.75.75 0 0 0-1.5 0v9A2.25 2.25 0 0 0 2.5 13h12.75a.75.75 0 0 0 0-1.5H2.5a.75.75 0 0 1-.75-.75v-9Zm10.75 2.5a.75.75 0 0 0-1.5 0v5a.75.75 0 0 0 1.5 0v-5Zm-3 .75a.75.75 0 0 1 1.5 0v4.25a.75.75 0 0 1-1.5 0V5Zm-3 1.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3Z"/></svg>
                {{ fileDetailMr.downloads.toLocaleString() }}
              </span>
              <span v-if="fileDetailMr.categories.length">{{ fileDetailMr.categories.slice(0, 4).join(", ") }}</span>
            </template>
          </div>
        </div>
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <button
          v-if="fileDetailExternalUrl()"
          type="button"
          class="rounded-md  bg-[var(--input)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx-muted)] transition-colors  hover:text-[var(--accent)]"
          @click="openExternal(fileDetailExternalUrl()!)"
        >
          {{ t("mods.openPage") }}
        </button>
        <button
          type="button"
          class="rounded-md p-1.5 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
          @click="closeFileDetailWin"
        >
          <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
        </button>
      </div>
    </div>
    <div class="flex shrink-0 items-center gap-1 border-b border-[var(--border)]  px-4 pb-2 pt-3">
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
    <div v-if="fileDetailMrLoading" class="flex min-h-0 flex-1 items-center justify-center text-[13px] text-[color:var(--tx-muted)]">
      <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
      {{ t("mods.searching") }}
    </div>
    <div v-else-if="fileDetailMr" class="min-h-0 flex-1 overflow-y-auto px-3.5 py-2.5">
      <div v-if="fileDetailTab === 'about'" class="rounded-md  bg-[var(--bg)] px-3.5 py-2.5">
        <Markdown v-if="fileDetailMr.body" :source="fileDetailMr.body" />
        <p v-else class="py-6 text-center text-[13px] italic text-[color:var(--tx-muted)]">{{ t("mods.noAbout") }}</p>
      </div>
      <div v-else-if="fileDetailTab === 'versions'">
        <div v-if="fileDetailMrVersions !== null" class="mb-2 flex flex-wrap items-center gap-2">
          <FilterSelect v-model="fileDetailMcSel" :options="fileDetailMcOptions" :placeholder="t('curse.fVersion')" :multiple="true" />
          <FilterSelect v-model="fileDetailLoaderSel" :options="fileDetailLoaderOptions" :placeholder="t('mods.fLoader')" :multiple="true" />
          <FilterSelect v-model="fileDetailTypeSel" :options="versionTypeOptions" :placeholder="t('mods.fType')" :multiple="true" />
        </div>
        <div v-if="fileDetailMrVersions === null" class="flex items-center justify-center py-10 text-[13px] text-[color:var(--tx-muted)]">
          <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
          {{ t("mods.searching") }}
        </div>
        <div v-else-if="fileDetailFilteredVersions.length === 0" class="rounded-md  bg-[var(--input-50)] p-6 text-center text-[13px] text-[color:var(--tx-muted)]">
          {{ t("mods.noVersions") }}
        </div>
        <div v-else class="space-y-1.5">
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
            class="h-40 w-full cursor-zoom-in rounded-md  object-cover transition-transform hover:scale-[1.02]"
            :title="g.title ?? undefined"
            @click="openExternal(g.url)"
          />
        </div>
        <p v-else class="py-10 text-center text-[13px] italic text-[color:var(--tx-muted)]">{{ t("mods.noGallery") }}</p>
      </div>
    </div>
    <div v-else-if="fileDetailCf" class="min-h-0 flex-1 overflow-y-auto px-3.5 py-2.5 nice-scrollbar">
      <div v-if="fileDetailTab === 'about'" class="space-y-4">
        <p class="whitespace-pre-wrap text-sm leading-relaxed text-[color:var(--tx)]">{{ fileDetailCf.description || t("mods.noAbout") }}</p>
        <dl class="grid grid-cols-2 gap-2 text-xs">
          <dt class="text-[color:var(--tx-muted)]">{{ t("mods.downloads") }}</dt><dd class="tabular-nums">{{ fileDetailCf.downloadCount.toLocaleString() }}</dd>
        </dl>
        <a :href="'https://www.curseforge.com/projects/' + fileDetailCf.slug" target="_blank" rel="noopener" @click.prevent="openExternal('https://www.curseforge.com/projects/' + fileDetailCf.slug)" class="inline-block text-xs font-medium text-[var(--accent)] hover:underline">CurseForge ↗</a>
      </div>
      <div v-else-if="fileDetailTab === 'versions'" class="space-y-2">
        <div class="flex flex-wrap items-center gap-2 pb-1">
          <FilterSelect :model-value="fileDetailCfMcSel" :options="fileDetailCfMcOptions" :label="t('mods.gameVersion')" placeholder="" />
        </div>
        <p v-if="fileDetailCfLoading" class="py-8 text-center text-xs text-[color:var(--tx-muted)]">{{ t("files.loading") }}…</p>
        <p v-else-if="fileDetailCfFilteredVersions.length === 0" class="py-8 text-center text-xs text-[color:var(--tx-muted)]">{{ t("files.empty") }}</p>
        <button v-for="f in fileDetailCfFilteredVersions" :key="f.fileId" type="button"
          class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left transition-colors hover:bg-[var(--input-50)] disabled:opacity-60"
          :disabled="fileDetailCfVersionBusy !== null"
          @click="installFileDetailCfVersion(f)">
          <svg v-if="fileDetailCfVersionBusy === f.fileId" viewBox="0 0 16 16" class="h-4 w-4 shrink-0 animate-spin fill-current text-[var(--accent)]"><path d="M8 1.5a.75.75 0 0 1 .75.75V8a.75.75 0 0 1-1.5 0V2.25A.75.75 0 0 1 8 1.5Zm3.36 2.14a.75.75 0 0 1 0 1.06 4 4 0 1 1-6.72 0 .75.75 0 0 1 1.06-1.06 2.5 2.5 0 1 0 4.6 0 .75.75 0 0 1 1.06-1.06Z"/></svg>
          <svg v-else viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current text-[var(--accent)]"><path d="M7.47 10.78a.75.75 0 0 0 1.06 0l3.75-3.75a.75.75 0 0 0-1.06-1.06L8.75 8.44V1.75a.75.75 0 0 0-1.5 0v6.69L4.78 5.97a.75.75 0 0 0-1.06 1.06l3.75 3.75ZM3.75 13a.75.75 0 0 0 0 1.5h8.5a.75.75 0 0 0 0-1.5h-8.5Z"/></svg>
          <span class="min-w-0 flex-1">
            <span class="block truncate text-sm font-medium text-[color:var(--tx-strong)]">{{ f.displayName || f.fileName }}</span>
            <span class="block truncate text-xs text-[color:var(--tx-muted)]">{{ [f.gameVersion, formatDate(f.fileDate)].filter(Boolean).join(" · ") }}</span>
          </span>
          <span class="shrink-0 rounded bg-[var(--input)] px-1.5 py-0.5 font-mono text-[11px] tabular-nums text-[color:var(--tx-muted)]">#{{ f.fileId }}</span>
        </button>
      </div>
    </div>
    <div v-else class="flex min-h-0 flex-1 items-center justify-center px-4 py-10">
      <p class="text-[13px] text-[color:var(--tx-muted)]">{{ t("mods.noAbout") }}</p>
    </div>
  </div>
</template>
