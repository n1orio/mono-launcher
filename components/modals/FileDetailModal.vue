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

/** Лоадеры/версии игры для сайдбара совместимости (из загруженных файлов). */
const detailCompatLoaders = computed(() => {
  const vers = (fileDetailMrVersions.value ?? []) as any[];
  const out = new Set<string>();
  for (const v of vers.slice(0, 20)) {
    for (const l of v.loaders ?? []) out.add(String(l));
    for (const g of (v.gameVersions ?? []).slice(0, 1)) out.add(String(g));
  }
  return [...out].slice(0, 8);
});
</script>

<template>
  <div v-if="isFileDetailWin" class="fixed inset-0 z-50 flex flex-col overflow-hidden bg-[var(--bg)] text-[color:var(--tx)] font-sans">
    <SubTitleBar :title="fileDetailMr?.title || fileDetailCf?.name || fileDetailTitle" />
    <div class="flex shrink-0 items-center justify-between gap-4 border-b border-[var(--border)] bg-[var(--panel)] px-5 py-4">
      <div class="flex min-w-0 flex-1 items-center gap-4">
        <img v-if="fileDetailMr?.iconUrl || fileDetailCf?.iconUrl" :src="searchIconUrl((fileDetailMr?.iconUrl ?? fileDetailCf?.iconUrl)!)" :alt="fileDetailMr?.title ?? fileDetailCf?.name ?? ''" loading="lazy" class="h-16 w-16 shrink-0 rounded-2xl object-cover" />
        <div v-else-if="fileDetailMr || fileDetailCf" class="flex h-16 w-16 shrink-0 items-center justify-center rounded-2xl bg-[var(--input)] text-lg font-bold text-[color:var(--tx-muted)]">
          {{ (fileDetailMr?.title ?? fileDetailCf?.name ?? "?").slice(0, 2).toUpperCase() }}
        </div>
        <div class="min-w-0 flex-1">
          <h2 class="truncate text-xl font-bold text-[color:var(--tx)] tracking-tight">{{ fileDetailMr?.title ?? fileDetailCf?.name ?? fileDetailTitle }}</h2>
          <div class="mt-1 flex flex-wrap items-center gap-x-3 gap-y-0.5 text-xs text-[color:var(--tx-muted)]">
            <template v-if="fileDetailMr">
              <span>{{ t("mods.byAuthor", { author: fileDetailMr.author }) }}</span>
              <span>•</span>
              <span class="flex items-center gap-1">
                <AppIcon name="download-bars" class="h-3 w-3 fill-current" />
                {{ fileDetailMr.downloads.toLocaleString() }} {{ t("mods.downloads") }}
              </span>
              <template v-if="fileDetailMr.categories.length">
                <span>•</span>
                <span>{{ fileDetailMr.categories.slice(0, 4).join(" / ") }}</span>
              </template>
            </template>
            <template v-else-if="fileDetailCf">
              <span>{{ t("mods.byAuthor", { author: fileDetailCf.author }) }}</span>
              <span>•</span>
              <span class="flex items-center gap-1">
                <AppIcon name="download-bars" class="h-3 w-3 fill-current" />
                {{ fileDetailCf.downloadCount.toLocaleString() }} {{ t("mods.downloads") }}
              </span>
            </template>
          </div>
        </div>
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <button
          v-if="fileDetailMr && fileDetailFilteredVersions.length"
          type="button"
          class="flex items-center gap-1.5 rounded-xl bg-[var(--accent)] hover:brightness-110 px-4 py-2 text-[13px] font-bold text-white shadow-md transition-all active:scale-95 disabled:opacity-50"
          :disabled="fileDetailMrVersionBusy !== null"
          @click="installFileDetailVersion(fileDetailFilteredVersions[0])"
        >
          <AppIcon v-if="fileDetailMrVersionBusy !== null" name="spinner" class="h-4 w-4 fill-current" />
          <AppIcon v-else name="arrow-down" class="h-4 w-4 fill-current" />
          {{ t("mods.install") }}
        </button>
        <button
          v-else-if="fileDetailCf && fileDetailCfFilteredVersions.length"
          type="button"
          class="flex items-center gap-1.5 rounded-xl bg-[var(--accent)] hover:brightness-110 px-4 py-2 text-[13px] font-bold text-white shadow-md transition-all active:scale-95 disabled:opacity-50"
          :disabled="fileDetailCfVersionBusy !== null"
          @click="installFileDetailCfVersion(fileDetailCfFilteredVersions[0])"
        >
          <AppIcon v-if="fileDetailCfVersionBusy !== null" name="spinner" class="h-4 w-4 fill-current" />
          <AppIcon v-else name="arrow-down" class="h-4 w-4 fill-current" />
          {{ t("mods.install") }}
        </button>
        <button
          v-if="fileDetailExternalUrl()"
          type="button"
          class="rounded-xl bg-[var(--input)] hover:bg-[var(--panel)] border border-[var(--border)] px-3.5 py-2 text-xs font-semibold text-[color:var(--tx)] transition-all"
          @click="openExternal(fileDetailExternalUrl()!)"
        >
          {{ t("mods.openPage") }}
        </button>
        <button
          type="button"
          class="rounded-md p-1.5 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
          @click="closeFileDetailWin"
        >
          <AppIcon name="x" class="h-4 w-4 fill-current" />
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
      <AppIcon name="spinner" class="mr-2 h-4 w-4 fill-current" />
      {{ t("mods.searching") }}
    </div>
    <div v-else-if="fileDetailMr" class="min-h-0 flex-1 overflow-y-auto">
      <div class="grid grid-cols-1 lg:grid-cols-[minmax(0,1fr)_320px] gap-6 w-full max-w-7xl mx-auto mt-6 px-4 pb-6">
      <div class="min-w-0">
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
          <AppIcon name="spinner" class="mr-2 h-4 w-4 fill-current" />
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
              <AppIcon name="spinner" class="h-4 w-4 shrink-0 fill-[var(--accent)]" />
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
            class="h-40 w-full max-w-full cursor-zoom-in rounded-xl object-cover transition-transform hover:scale-[1.02]"
            :title="g.title ?? undefined"
            @click="openExternal(g.url)"
          />
        </div>
        <p v-else class="py-10 text-center text-[13px] italic text-[color:var(--tx-muted)]">{{ t("mods.noGallery") }}</p>
      </div>
      </div>
      <aside class="space-y-4 min-w-0">
        <div class="rounded-2xl border border-[var(--border)] bg-[var(--panel)] p-4">
          <h4 class="mb-3 text-xs font-bold uppercase tracking-wider text-[color:var(--tx-muted)]">{{ t("mods.aboutProject") }}</h4>
          <dl class="space-y-2 text-xs">
            <div class="flex items-center justify-between gap-2">
              <dt class="text-[color:var(--tx-muted)]">{{ t("mods.projectId") }}</dt>
              <dd class="font-mono text-[color:var(--tx)] truncate">{{ fileDetailMr.projectId }}</dd>
            </div>
            <div class="flex items-center justify-between gap-2">
              <dt class="text-[color:var(--tx-muted)]">{{ t("mods.projectType") }}</dt>
              <dd class="text-[color:var(--tx)]">{{ fileDetailMr.projectType }}</dd>
            </div>
            <div v-if="fileDetailMr.latestVersion" class="flex items-center justify-between gap-2">
              <dt class="text-[color:var(--tx-muted)]">{{ t("mods.latestVersion") }}</dt>
              <dd class="font-mono text-[color:var(--tx)]">{{ fileDetailMr.latestVersion }}</dd>
            </div>
          </dl>
        </div>
        <div v-if="detailCompatLoaders.length || fileDetailMr.categories.length" class="rounded-2xl border border-[var(--border)] bg-[var(--panel)] p-4">
          <h4 class="mb-3 text-xs font-bold uppercase tracking-wider text-[color:var(--tx-muted)]">{{ t("mods.compat") }}</h4>
          <div class="flex flex-wrap gap-1.5">
            <span v-for="l in detailCompatLoaders" :key="l" class="text-[11px] px-2 py-0.5 rounded-md bg-white/5 text-[color:var(--tx-muted)]">{{ l }}</span>
            <span v-for="c in fileDetailMr.categories.slice(0, 6)" :key="c" class="text-[11px] px-2 py-0.5 rounded-md bg-white/5 text-[color:var(--tx-muted)]">{{ c }}</span>
          </div>
        </div>
        <div v-if="fileDetailExternalUrl()" class="rounded-2xl border border-[var(--border)] bg-[var(--panel)] p-4">
          <h4 class="mb-3 text-xs font-bold uppercase tracking-wider text-[color:var(--tx-muted)]">{{ t("mods.links") }}</h4>
          <button type="button" class="flex w-full items-center gap-2 rounded-xl bg-[var(--input)] hover:bg-[var(--hover)] px-3 py-2 text-xs font-semibold text-[color:var(--tx)] transition-all" @click="openExternal(fileDetailExternalUrl()!)">
            <AppIcon name="external-link" class="h-3.5 w-3.5 fill-current" />
            {{ t("mods.openPage") }}
          </button>
        </div>
      </aside>
      </div>
    </div>
    <div v-else-if="fileDetailCf" class="min-h-0 flex-1 overflow-y-auto nice-scrollbar">
      <div class="grid grid-cols-1 lg:grid-cols-[minmax(0,1fr)_320px] gap-6 w-full max-w-7xl mx-auto mt-6 px-4 pb-6">
      <div class="min-w-0">
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
          <AppIcon v-if="fileDetailCfVersionBusy === f.fileId" name="spinner" class="h-4 w-4 shrink-0 fill-current text-[var(--accent)]" />
          <svg v-else viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current text-[var(--accent)]"><path d="M7.47 10.78a.75.75 0 0 0 1.06 0l3.75-3.75a.75.75 0 0 0-1.06-1.06L8.75 8.44V1.75a.75.75 0 0 0-1.5 0v6.69L4.78 5.97a.75.75 0 0 0-1.06 1.06l3.75 3.75ZM3.75 13a.75.75 0 0 0 0 1.5h8.5a.75.75 0 0 0 0-1.5h-8.5Z"/></svg>
          <span class="min-w-0 flex-1">
            <span class="block truncate text-sm font-medium text-[color:var(--tx-strong)]">{{ f.displayName || f.fileName }}</span>
            <span class="block truncate text-xs text-[color:var(--tx-muted)]">{{ [f.gameVersion, formatDate(f.fileDate)].filter(Boolean).join(" · ") }}</span>
          </span>
          <span class="shrink-0 rounded bg-[var(--input)] px-1.5 py-0.5 font-mono text-[11px] tabular-nums text-[color:var(--tx-muted)]">#{{ f.fileId }}</span>
        </button>
      </div>
      </div>
      <aside class="space-y-4 min-w-0">
        <div class="rounded-2xl border border-[var(--border)] bg-[var(--panel)] p-4">
          <h4 class="mb-3 text-xs font-bold uppercase tracking-wider text-[color:var(--tx-muted)]">{{ t("mods.aboutProject") }}</h4>
          <dl class="space-y-2 text-xs">
            <div class="flex items-center justify-between gap-2">
              <dt class="text-[color:var(--tx-muted)]">{{ t("mods.projectId") }}</dt>
              <dd class="font-mono text-[color:var(--tx)]">{{ fileDetailCf.projectId }}</dd>
            </div>
            <div class="flex items-center justify-between gap-2">
              <dt class="text-[color:var(--tx-muted)]">{{ t("mods.downloads") }}</dt>
              <dd class="tabular-nums text-[color:var(--tx)]">{{ fileDetailCf.downloadCount.toLocaleString() }}</dd>
            </div>
          </dl>
        </div>
        <div v-if="fileDetailCf.categories.length" class="rounded-2xl border border-[var(--border)] bg-[var(--panel)] p-4">
          <h4 class="mb-3 text-xs font-bold uppercase tracking-wider text-[color:var(--tx-muted)]">{{ t("mods.compat") }}</h4>
          <div class="flex flex-wrap gap-1.5">
            <span v-for="c in fileDetailCf.categories.slice(0, 8)" :key="c" class="text-[11px] px-2 py-0.5 rounded-md bg-white/5 text-[color:var(--tx-muted)]">{{ c }}</span>
          </div>
        </div>
        <div class="rounded-2xl border border-[var(--border)] bg-[var(--panel)] p-4">
          <h4 class="mb-3 text-xs font-bold uppercase tracking-wider text-[color:var(--tx-muted)]">{{ t("mods.links") }}</h4>
          <button type="button" class="flex w-full items-center gap-2 rounded-xl bg-[var(--input)] hover:bg-[var(--hover)] px-3 py-2 text-xs font-semibold text-[color:var(--tx)] transition-all" @click="openExternal('https://www.curseforge.com/projects/' + fileDetailCf.slug)">
            <AppIcon name="external-link" class="h-3.5 w-3.5 fill-current" />
            CurseForge
          </button>
        </div>
      </aside>
      </div>
    </div>
    <div v-else class="flex min-h-0 flex-1 items-center justify-center px-4 py-10">
      <p class="text-[13px] text-[color:var(--tx-muted)]">{{ t("mods.noAbout") }}</p>
    </div>
  </div>
</template>
