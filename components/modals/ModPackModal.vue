<script setup lang="ts">
import { useLauncherCtx } from "~/composables/useLauncherContext";
import { useI18n } from "#imports";

const { t } = useI18n();
const {
  modPackOpen,
  modPackService,
  modPackQuery,
  modPackLoading,
  modPackResults,
  modPackDetail,
  modPackVersions,
  modPackTab,
  modPackTabs,
  modPackInstalling,
  quickPackBusy,
  cpLoading,
  cpSearched,
  cpResults,
  cpProject,
  cpDetail,
  cpDetailLoading,
  cpFiles,
  cpTab,
  cpTabs,
  cpBusy,
  cpErr,
  cpCatSel,
  cpCatOptions,
  cpVerSel,
  cpSortSel,
  packFilters,
  packVersionOptions,
  packLoaderOptions,
  packVersionTypeSel,
  packCategoryOptions,
  packEnvSel,
  packSortSel,
  envOptions,
  sortSelectOptions,
  versionTypeOptions,
  status,
  openExternal,
  switchPackService,
  searchPacksOrCurse,
  searchPacks,
  searchCursePacks,
  openPackDetail,
  openCpFiles,
  installPackVersion,
  quickDownloadPack,
  installCpPack,
  curseSortOptions,
  formatDate,
  openModPackModal,
} = useLauncherCtx();
</script>

<template>
  <div
    v-if="modPackOpen"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6"
    @click.self="modPackOpen = false; modPackVersions = null; modPackDetail = null"
  >
    <div class="flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-xl  bg-[var(--panel)] shadow-2xl">
      <div class="flex shrink-0 items-center justify-between border-b border-[var(--border)]  px-3.5 py-2.5">
        <h3 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ t("mods.packsTitle") }}</h3>
        <button
          type="button"
          class="rounded-md p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
          @click="modPackOpen = false; modPackVersions = null; modPackDetail = null"
        >
          <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
        </button>
      </div>
      <div class="flex shrink-0 items-center gap-2 border-b border-[var(--border)]  px-3.5 py-2.5">
        <div class="flex shrink-0 items-center gap-1 rounded-md  bg-[var(--bg)] p-0.5">
          <button
            type="button"
            class="flex items-center gap-1.5 rounded px-2.5 py-1.5 text-[13px] font-semibold transition-colors"
            :class="modPackService === 'modrinth'
              ? 'bg-[var(--accent)] text-white'
              : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
            @click="switchPackService('modrinth')"
          >
            <svg viewBox="0 0 24 24" class="h-4 w-4 shrink-0" :title="t('mods.serviceModrinth')"><path fill="#00AF5C" d="M12.252.004a11.78 11.768 0 0 0-8.92 3.73 11 10.999 0 0 0-2.17 3.11 11.37 11.359 0 0 0-1.16 5.169c0 1.42.17 2.5.6 3.77.24.759.77 1.899 1.17 2.529a12.3 12.298 0 0 0 8.85 5.639c.44.05 2.54.07 2.76.02.2-.04.22.1-.26-1.7l-.36-1.37-1.01-.06a8.5 8.489 0 0 1-5.18-1.8 5.34 5.34 0 0 1-1.3-1.26c0-.05.34-.28.74-.5a37.572 37.545 0 0 1 2.88-1.629c.03 0 .5.45 1.06.98l1 .97 2.07-.43 2.06-.43 1.47-1.47c.8-.8 1.48-1.5 1.48-1.52 0-.09-.42-1.63-.46-1.7-.04-.06-.2-.03-1.02.18-.53.13-1.2.3-1.45.4l-.48.15-.53.53-.53.53-.93.1-.93.07-.52-.5a2.7 2.7 0 0 1-.96-1.7l-.13-.6.43-.57c.68-.9.68-.9 1.46-1.1.4-.1.65-.2.83-.33.13-.099.65-.579 1.14-1.069l.9-.9-.7-.7-.7-.7-1.95.54c-1.07.3-1.96.53-1.97.53-.03 0-2.23 2.48-2.63 2.97l-.29.35.28 1.03c.16.56.3 1.16.31 1.34l.03.3-.34.23c-.37.23-2.22 1.3-2.84 1.63-.36.2-.37.2-.44.1-.08-.1-.23-.6-.32-1.03-.18-.86-.17-2.75.02-3.73a8.84 8.839 0 0 1 7.9-6.93c.43-.03.77-.08.78-.1.06-.17.5-2.999.47-3.039-.01-.02-.1-.02-.2-.03Zm3.68.67c-.2 0-.3.1-.37.38-.06.23-.46 2.42-.46 2.52 0 .04.1.11.22.16a8.51 8.499 0 0 1 2.99 2 8.38 8.379 0 0 1 2.16 3.449 6.9 6.9 0 0 1 .4 2.8c0 1.07 0 1.27-.1 1.73a9.37 9.369 0 0 1-1.76 3.769c-.32.4-.98 1.06-1.37 1.38-.38.32-1.54 1.1-1.7 1.14-.1.03-.1.06-.07.26.03.18.64 2.56.7 2.78l.06.06a12.07 12.058 0 0 0 7.27-9.4c.13-.77.13-2.58 0-3.4a11.96 11.948 0 0 0-5.73-8.578c-.7-.42-2.05-1.06-2.25-1.06Z"/></svg>
            {{ t("mods.serviceModrinth") }}
          </button>
          <button
            type="button"
            class="flex items-center gap-1.5 rounded px-2.5 py-1.5 text-[13px] font-semibold transition-colors"
            :class="modPackService === 'curseforge'
              ? 'bg-[var(--accent)] text-white'
              : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
            @click="switchPackService('curseforge')"
          >
            <svg viewBox="0 0 24 24" class="h-4 w-4 shrink-0" :title="t('mods.serviceCurseforge')"><path fill="#F16436" d="M18.326 9.2145S23.2261 8.4418 24 6.1882h-7.5066V4.4H0l2.0318 2.3576V9.173s5.1267-.2665 7.1098 1.2372c2.7146 2.516-3.053 5.917-3.053 5.917L5.0995 19.6c1.5465-1.4726 4.494-3.3775 9.8983-3.2857-2.0565.65-4.1245 1.6651-5.7344 3.2857h10.9248l-1.0288-3.2726s-7.918-4.6688-.8336-7.1127z"/></svg>
            {{ t("mods.serviceCurseforge") }}
          </button>
        </div>
        <div class="relative min-w-0 flex-1">
          <svg viewBox="0 0 16 16" class="pointer-events-none absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 fill-[var(--tx-muted)]">
            <path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"/>
          </svg>
          <input
            v-model="modPackQuery"
            type="text"
            :placeholder="modPackService === 'modrinth' ? t('mods.packsPlaceholder') : t('curse.packsPlaceholder')"
            class="w-full rounded-md  bg-[var(--bg)] py-1.5 pl-8 pr-3 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)] outline-none transition-colors "
            @keydown.enter="searchPacksOrCurse"
          />
        </div>
        <button
          type="button"
          class="flex shrink-0 items-center gap-1.5 rounded-md  bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
          :disabled="modPackLoading || cpLoading || !modPackQuery.trim()"
          @click="searchPacksOrCurse"
        >
          <svg v-if="modPackLoading || cpLoading" viewBox="0 0 16 16" class="h-4 w-4 animate-spin fill-current">
            <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
          </svg>
          <svg v-else viewBox="0 0 16 16" class="h-4 w-4 fill-current">
            <path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"/>
          </svg>
          {{ t("mods.search") }}
        </button>
      </div>
      <div v-if="modPackService === 'modrinth'" class="flex shrink-0 flex-wrap items-center gap-2 border-b border-[var(--border)]  px-4 py-2">
        <FilterSelect v-model="packFilters.versions" :options="packVersionOptions" :placeholder="t('mods.fVersion')" @change="searchPacks()" />
        <FilterSelect v-model="packFilters.loaders" :options="packLoaderOptions" :placeholder="t('mods.fLoader')" @change="searchPacks()" />
        <FilterSelect v-model="packVersionTypeSel" :options="versionTypeOptions" :placeholder="t('mods.fType')" :multiple="false" @change="searchPacks()" />
        <FilterSelect v-model="packFilters.categories" :options="packCategoryOptions" :placeholder="t('mods.fCategory')" @change="searchPacks()" />
        <FilterSelect v-model="packEnvSel" :options="envOptions" :placeholder="t('mods.fAny')" :multiple="false" @change="searchPacks()" />
        <FilterSelect v-model="packSortSel" :options="sortSelectOptions" :placeholder="t('mods.fSort')" :multiple="false" @change="searchPacks()" />
      </div>
      <div v-if="modPackService === 'modrinth'" class="min-h-0 flex-1 overflow-y-auto p-4">
        <template v-if="modPackDetail">
          <button
            type="button"
            class="mb-3 flex items-center gap-1 text-[13px] text-[color:var(--tx-muted)] transition-colors hover:text-[var(--accent)]"
            @click="modPackDetail = null; modPackVersions = null"
          >
            <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M7.28 3.22a.75.75 0 0 1 0 1.06L3.56 8l3.72 3.72a.75.75 0 1 1-1.06 1.06l-4.25-4.25a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0Zm4 0a.75.75 0 0 1 0 1.06L7.56 8l3.72 3.72a.75.75 0 1 1-1.06 1.06l-4.25-4.25a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0Z"/></svg>
            {{ t("mods.back") }}
          </button>
          <div class="overflow-hidden rounded-md  bg-[var(--bg)]">
            <div class="flex items-start gap-3 px-3.5 py-2.5">
              <img v-if="modPackDetail.iconUrl" :src="modPackDetail.iconUrl" alt="" class="h-14 w-14 shrink-0 rounded-md object-cover" />
              <div v-else class="flex h-14 w-14 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-sm font-bold text-[color:var(--tx-muted)]">
                {{ modPackDetail.title.slice(0, 2).toUpperCase() }}
              </div>
              <div class="min-w-0 flex-1">
                <h4 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ modPackDetail.title }}</h4>
                <div class="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-[color:var(--tx-muted)]">
                  <span>{{ t("mods.byAuthor", { author: modPackDetail.author }) }}</span>
                  <span class="flex items-center gap-1">
                    <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1.75a.75.75 0 0 0-1.5 0v9A2.25 2.25 0 0 0 2.5 13h12.75a.75.75 0 0 0 0-1.5H2.5a.75.75 0 0 1-.75-.75v-9Zm10.75 2.5a.75.75 0 0 0-1.5 0v5a.75.75 0 0 0 1.5 0v-5Zm-3 .75a.75.75 0 0 1 1.5 0v4.25a.75.75 0 0 1-1.5 0V5Zm-3 1.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3Z"/></svg>
                    {{ modPackDetail.downloads.toLocaleString() }}
                  </span>
                  <span v-if="modPackDetail.categories.length">{{ modPackDetail.categories.slice(0, 4).join(", ") }}</span>
                </div>
              </div>
              <button
                type="button"
                class="flex shrink-0 items-center gap-1.5 rounded-md  bg-[var(--input)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                @click="openExternal(`https://modrinth.com/modpack/${modPackDetail!.slug}`)"
              >
                <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M3.75 2h3.5a.75.75 0 0 1 0 1.5h-2l6 6V7.5a.75.75 0 0 1 1.5 0v4.5a.75.75 0 0 1-.75.75H5.5a.75.75 0 0 1 0-1.5h2l-6-6v2a.75.75 0 0 1-1.5 0V3.5A1.75 1.75 0 0 1 1.75 1.75h2a.75.75 0 0 1 0 1.5Z"/></svg>
                {{ t("mods.openPage") }}
              </button>
            </div>
          </div>

          <div class="mt-3 mb-3 flex shrink-0 items-center gap-1 border-b border-[var(--border)]  pb-2">
            <button
              v-for="tb in modPackTabs"
              :key="tb.kind"
              type="button"
              class="rounded-md px-2.5 py-1.5 text-[13px] font-medium transition-colors"
              :class="modPackTab === tb.kind
                ? 'bg-[var(--input)] text-[color:var(--tx-strong)]'
                : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]'"
              @click="modPackTab = tb.kind"
            >
              {{ t("mods.tab" + tb.kind) }}
            </button>
          </div>

          <div v-if="modPackTab === 'about'" class="max-h-[46vh] overflow-y-auto rounded-md  bg-[var(--bg)] px-3.5 py-2.5">
            <Markdown v-if="modPackDetail.body" :source="modPackDetail.body" />
            <p v-else class="py-6 text-center text-[13px] italic text-[color:var(--tx-muted)]">{{ t("mods.noAbout") }}</p>
          </div>

          <div v-else-if="modPackTab === 'versions'">
            <div v-if="modPackVersions && modPackVersions.length === 0" class="py-8 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("mods.noVersions") }}</div>
            <div v-else-if="modPackVersions" class="space-y-2">
              <div v-for="v in modPackVersions" :key="v.id" class="flex items-center gap-3 rounded-md  bg-[var(--bg)] px-3 py-2">
                <div class="min-w-0 flex-1">
                  <div class="flex flex-wrap items-center gap-x-2 gap-y-1">
                    <span class="truncate text-[13px] font-medium text-[color:var(--tx-strong)]">{{ v.name }}</span>
                    <span class="rounded  bg-[var(--input-50)] px-1.5 py-0.5 font-mono text-xs text-[color:var(--tx-muted)]">{{ v.versionNumber }}</span>
                  </div>
                  <div class="mt-0.5 truncate text-xs text-[color:var(--tx-muted)]">
                    {{ v.gameVersions.slice(0, 2).join(", ") }} · {{ v.loaders.join(", ") }} · {{ formatDate(v.datePublished) }}
                  </div>
                </div>
                <button
                  type="button"
                  class="flex shrink-0 items-center gap-1.5 rounded-md  bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                  :disabled="modPackInstalling !== null"
                  @click="installPackVersion(v)"
                >
                  <svg v-if="modPackInstalling === v.id" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                    <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                  </svg>
                  <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                    <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
                  </svg>
                  {{ t("mods.install") }}
                </button>
              </div>
            </div>
            <div v-else class="flex items-center justify-center py-10 text-[13px] text-[color:var(--tx-muted)]">
              <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current">
                <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
              </svg>
              {{ t("mods.searching") }}
            </div>
          </div>

          <div v-else>
            <div v-if="modPackDetail.gallery.length" class="grid grid-cols-2 gap-2">
              <img v-for="g in modPackDetail.gallery" :key="g.url" :src="g.url" :alt="g.title ?? ''" loading="lazy" class="h-32 w-full cursor-zoom-in rounded-md  object-cover transition-transform hover:scale-[1.02]" :title="g.title ?? undefined" @click="openExternal(g.url)" />
            </div>
            <p v-else class="py-10 text-center text-[13px] italic text-[color:var(--tx-muted)]">{{ t("mods.noGallery") }}</p>
          </div>
        </template>
        <template v-else-if="modPackLoading">
          <div class="flex items-center justify-center py-16 text-[13px] text-[color:var(--tx-muted)]">
            <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current">
              <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
            </svg>
            {{ t("mods.searching") }}
          </div>
        </template>
        <template v-else-if="modPackResults.length === 0">
          <div class="py-16 text-center text-[13px] text-[color:var(--tx-muted)]">
            {{ modPackQuery ? t("mods.noResults") : t("mods.packsHelp") }}
          </div>
        </template>
        <template v-else>
          <div class="space-y-2">
            <div v-for="p in modPackResults" :key="p.projectId" class="flex cursor-pointer items-start gap-3 rounded-md  bg-[var(--bg)] px-3 py-2.5 transition-colors " @click="openPackDetail(p)">
              <img v-if="p.iconUrl" :src="p.iconUrl" alt="" loading="lazy" class="h-10 w-10 shrink-0 rounded-md object-cover" />
              <div v-else class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-xs text-[color:var(--tx-muted)]">
                {{ p.title.slice(0, 2).toUpperCase() }}
              </div>
              <div class="min-w-0 flex-1">
                <div class="flex flex-wrap items-center gap-x-2">
                  <svg viewBox="0 0 24 24" class="h-3 w-3 shrink-0 self-center" :title="t('mods.serviceModrinth')"><path fill="#00AF5C" d="M12.252.004a11.78 11.768 0 0 0-8.92 3.73 11 10.999 0 0 0-2.17 3.11 11.37 11.359 0 0 0-1.16 5.169c0 1.42.17 2.5.6 3.77.24.759.77 1.899 1.17 2.529a12.3 12.298 0 0 0 8.85 5.639c.44.05 2.54.07 2.76.02.2-.04.22.1-.26-1.7l-.36-1.37-1.01-.06a8.5 8.489 0 0 1-5.18-1.8 5.34 5.34 0 0 1-1.3-1.26c0-.05.34-.28.74-.5a37.572 37.545 0 0 1 2.88-1.629c.03 0 .5.45 1.06.98l1 .97 2.07-.43 2.06-.43 1.47-1.47c.8-.8 1.48-1.5 1.48-1.52 0-.09-.42-1.63-.46-1.7-.04-.06-.2-.03-1.02.18-.53.13-1.2.3-1.45.4l-.48.15-.53.53-.53.53-.93.1-.93.07-.52-.5a2.7 2.7 0 0 1-.96-1.7l-.13-.6.43-.57c.68-.9.68-.9 1.46-1.1.4-.1.65-.2.83-.33.13-.099.65-.579 1.14-1.069l.9-.9-.7-.7-.7-.7-1.95.54c-1.07.3-1.96.53-1.97.53-.03 0-2.23 2.48-2.63 2.97l-.29.35.28 1.03c.16.56.3 1.16.31 1.34l.03.3-.34.23c-.37.23-2.22 1.3-2.84 1.63-.36.2-.37.2-.44.1-.08-.1-.23-.6-.32-1.03-.18-.86-.17-2.75.02-3.73a8.84 8.839 0 0 1 7.9-6.93c.43-.03.77-.08.78-.1.06-.17.5-2.999.47-3.039-.01-.02-.1-.02-.2-.03Zm3.68.67c-.2 0-.3.1-.37.38-.06.23-.46 2.42-.46 2.52 0 .04.1.11.22.16a8.51 8.499 0 0 1 2.99 2 8.38 8.379 0 0 1 2.16 3.449 6.9 6.9 0 0 1 .4 2.8c0 1.07 0 1.27-.1 1.73a9.37 9.369 0 0 1-1.76 3.769c-.32.4-.98 1.06-1.37 1.38-.38.32-1.54 1.1-1.7 1.14-.1.03-.1.06-.07.26.03.18.64 2.56.7 2.78l.06.06a12.07 12.058 0 0 0 7.27-9.4c.13-.77.13-2.58 0-3.4a11.96 11.948 0 0 0-5.73-8.578c-.7-.42-2.05-1.06-2.25-1.06Z"/></svg>
                  <span class="truncate text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ p.title }}</span>
                  <span class="text-xs text-[color:var(--tx-muted)]">{{ t("mods.byAuthor", { author: p.author }) }}</span>
                </div>
                <p class="mt-0.5 line-clamp-2 text-[13px] leading-snug text-[color:var(--tx-muted)]">{{ p.description }}</p>
                <div class="mt-1 flex items-center gap-3 text-xs text-[color:var(--tx-muted)]">
                  <span class="flex items-center gap-1">
                    <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1.75a.75.75 0 0 0-1.5 0v9A2.25 2.25 0 0 0 2.5 13h12.75a.75.75 0 0 0 0-1.5H2.5a.75.75 0 0 1-.75-.75v-9Zm10.75 2.5a.75.75 0 0 0-1.5 0v5a.75.75 0 0 0 1.5 0v-5Zm-3 .75a.75.75 0 0 1 1.5 0v4.25a.75.75 0 0 1-1.5 0V5Zm-3 1.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3Z"/></svg>
                    {{ p.downloads.toLocaleString() }}
                  </span>
                  <span v-if="status?.minecraft_version">{{ status.minecraft_version }}</span>
                </div>
              </div>
              <button
                type="button"
                class="flex shrink-0 items-center gap-1.5 rounded-md  bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                :disabled="quickPackBusy !== null || modPackInstalling !== null"
                :title="t('mods.downloadHint')"
                @click="quickDownloadPack(p, $event)"
              >
                <svg v-if="quickPackBusy === p.projectId" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                  <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                </svg>
                <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                  <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
                </svg>
                {{ t("mods.download") }}
              </button>
              <svg viewBox="0 0 16 16" class="mt-1 h-4 w-4 shrink-0 fill-[var(--tx-muted)]"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"/></svg>
            </div>
          </div>
        </template>
      </div>
      <div v-if="modPackService === 'curseforge'" class="flex shrink-0 flex-wrap items-center gap-2 border-b border-[var(--border)]  px-4 py-2">
        <FilterSelect v-model="cpCatSel" :options="cpCatOptions" :placeholder="t('curse.fCategory')" :multiple="false" @change="searchCursePacks" />
        <FilterSelect v-model="cpVerSel" :options="packVersionOptions" :placeholder="t('mods.fVersion')" :multiple="false" @change="searchCursePacks" />
        <FilterSelect v-model="cpSortSel" :options="curseSortOptions" :placeholder="t('mods.fSort')" :multiple="false" @change="searchCursePacks" />
      </div>
      <div v-if="modPackService === 'curseforge'" class="min-h-0 flex-1 overflow-y-auto px-3.5 py-2.5">
        <p v-if="!cpSearched" class="py-8 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("curse.packsHelp") }}</p>
        <p v-else-if="cpLoading" class="flex items-center justify-center gap-2 py-8 text-[13px] text-[color:var(--tx-muted)]">
          <svg viewBox="0 0 16 16" class="h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
          {{ t("mods.searchingAll") }}
        </p>
        <div v-else-if="cpErr" class="rounded-md  bg-[var(--input-50)] p-6 text-center text-[13px] text-[color:var(--tx-muted)]">
          <p class="mb-2 whitespace-pre-wrap">{{ cpErr }}</p>
          <button type="button" class="text-[var(--accent)] hover:underline" @click="searchCursePacks">{{ t("catalog.retry") }}</button>
        </div>
        <template v-else-if="cpProject">
          <button type="button" class="mb-3 flex items-center gap-1 text-[13px] text-[color:var(--tx-muted)] transition-colors hover:text-[var(--accent)]" @click="cpProject = null; cpFiles = null; cpDetail = null; cpErr = ''">
            <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M7.28 3.22a.75.75 0 0 1 0 1.06L3.56 8l3.72 3.72a.75.75 0 1 1-1.06 1.06l-4.25-4.25a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0Zm4 0a.75.75 0 0 1 0 1.06L7.56 8l3.72 3.72a.75.75 0 1 1-1.06 1.06l-4.25-4.25a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0Z"/></svg>
            {{ t("mods.back") }}
          </button>
          <div class="mb-3 rounded-md  bg-[var(--bg)]">
            <div class="flex items-start gap-3 px-3.5 py-2.5">
              <img v-if="cpProject.iconUrl" :src="cpProject.iconUrl" alt="" class="h-14 w-14 shrink-0 rounded-md object-cover" />
              <div v-else class="flex h-14 w-14 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-sm font-bold text-[color:var(--tx-muted)]">
                {{ cpProject.name.slice(0, 2).toUpperCase() }}
              </div>
              <div class="min-w-0 flex-1">
                <h4 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ cpProject.name }}</h4>
                <div class="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-[color:var(--tx-muted)]">
                  <span>{{ t("mods.byAuthor", { author: cpProject.author }) }}</span>
                  <span class="flex items-center gap-1">
                    <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1.75a.75.75 0 0 0-1.5 0v9A2.25 2.25 0 0 0 2.5 13h12.75a.75.75 0 0 0 0-1.5H2.5a.75.75 0 0 1-.75-.75v-9Zm10.75 2.5a.75.75 0 0 0-1.5 0v5a.75.75 0 0 0 1.5 0v-5Zm-3 .75a.75.75 0 0 1 1.5 0v4.25a.75.75 0 0 1-1.5 0V5Zm-3 1.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3Z"/></svg>
                    {{ cpProject.downloadCount.toLocaleString() }}
                  </span>
                  <span v-if="cpDetail?.categories.length">{{ cpDetail.categories.slice(0, 4).join(", ") }}</span>
                </div>
              </div>
            </div>
          </div>

          <div class="mt-3 mb-3 flex shrink-0 items-center gap-1 border-b border-[var(--border)]  pb-2">
            <button v-for="tb in cpTabs" :key="tb" type="button" class="rounded-md px-2.5 py-1.5 text-[13px] font-medium transition-colors" :class="cpTab === tb ? 'bg-[var(--input)] text-[color:var(--tx-strong)]' : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]'" @click="cpTab = tb">
              {{ t("curse.tab" + tb) }}
            </button>
          </div>

          <div v-if="cpTab === 'about'">
            <div v-if="cpDetailLoading" class="flex items-center justify-center py-10 text-[13px] text-[color:var(--tx-muted)]">
              <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
              {{ t("mods.searching") }}
            </div>
            <div v-else-if="cpDetail?.description" class="max-h-[46vh] overflow-y-auto rounded-md  bg-[var(--bg)] px-3.5 py-2.5 leading-relaxed">
              <div v-html="cpDetail.description" class="prose prose-invert max-w-none text-[13px] leading-relaxed"></div>
            </div>
            <div v-else class="py-8 text-center text-[13px] italic text-[color:var(--tx-muted)]">{{ t("mods.noAbout") }}</div>
          </div>

          <div v-else-if="cpTab === 'versions'">
            <div v-if="cpFiles === null" class="flex items-center justify-center py-10 text-[13px] text-[color:var(--tx-muted)]">
              <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
              {{ t("mods.searching") }}
            </div>
            <div v-else-if="cpFiles.length === 0" class="py-8 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("curse.noFiles") }}</div>
            <div v-else class="space-y-2">
              <div v-for="f in cpFiles" :key="f.fileId" class="flex items-center gap-3 rounded-md  bg-[var(--bg)] px-3 py-2">
                <div class="min-w-0 flex-1">
                  <div class="truncate text-[13px] font-medium text-[color:var(--tx-strong)]">{{ f.displayName }}</div>
                  <div class="mt-0.5 truncate text-xs text-[color:var(--tx-muted)]">
                    {{ f.gameVersion }} · {{ formatDate(f.fileDate) }}
                  </div>
                </div>
                <button
                  type="button"
                  class="flex shrink-0 items-center gap-1.5 rounded-md  bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                  :disabled="cpBusy !== null"
                  @click="installCpPack(f)"
                >
                  <svg v-if="cpBusy === f.fileId" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
                  <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                    <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
                  </svg>
                  {{ t("mods.install") }}
                </button>
              </div>
            </div>
          </div>

          <div v-else>
            <div v-if="cpDetailLoading" class="flex items-center justify-center py-10 text-[13px] text-[color:var(--tx-muted)]">
              <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
              {{ t("mods.searching") }}
            </div>
            <div v-else-if="cpDetail?.screenshots.length" class="grid grid-cols-2 gap-2">
              <img v-for="(s, i) in cpDetail.screenshots" :key="i" :src="s" alt="" loading="lazy" class="h-32 w-full cursor-zoom-in rounded-md  object-cover transition-transform hover:scale-[1.02]" @click="openExternal(s)" />
            </div>
            <div v-else class="py-8 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("curse.noScreenshots") }}</div>
          </div>
        </template>
        <template v-else-if="cpResults.length === 0">
          <p class="py-8 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("mods.noResults") }}</p>
        </template>
        <template v-else>
          <div class="space-y-2">
            <div v-for="p in cpResults" :key="p.projectId" class="flex cursor-pointer items-start gap-3 rounded-md  bg-[var(--bg)] px-3 py-2.5 transition-colors " @click="openCpFiles(p)">
              <img v-if="p.iconUrl" :src="p.iconUrl" alt="" loading="lazy" class="h-10 w-10 shrink-0 rounded-md object-cover" />
              <div v-else class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-xs text-[color:var(--tx-muted)]">
                {{ p.name.slice(0, 2).toUpperCase() }}
              </div>
              <div class="min-w-0 flex-1">
                <div class="flex flex-wrap items-center gap-x-2">
                  <svg viewBox="0 0 24 24" class="h-3 w-3 shrink-0 self-center" :title="t('mods.serviceCurseforge')"><path fill="#F16436" d="M18.326 9.2145S23.2261 8.4418 24 6.1882h-7.5066V4.4H0l2.0318 2.3576V9.173s5.1267-.2665 7.1098 1.2372c2.7146 2.516-3.053 5.917-3.053 5.917L5.0995 19.6c1.5465-1.4726 4.494-3.3775 9.8983-3.2857-2.0565.65-4.1245 1.6651-5.7344 3.2857h10.9248l-1.0288-3.2726s-7.918-4.6688-.8336-7.1127z"/></svg>
                  <span class="truncate text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ p.name }}</span>
                  <span class="text-xs text-[color:var(--tx-muted)]">{{ t("mods.byAuthor", { author: p.author }) }}</span>
                </div>
                <p class="mt-0.5 line-clamp-2 text-[13px] leading-snug text-[color:var(--tx-muted)]">{{ p.summary }}</p>
                <p class="mt-1 flex items-center gap-1 text-xs text-[color:var(--tx-muted)]">
                  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1.75a.75.75 0 0 0-1.5 0v9A2.25 2.25 0 0 0 2.5 13h12.75a.75.75 0 0 0 0-1.5H2.5a.75.75 0 0 1-.75-.75v-9Zm10.75 2.5a.75.75 0 0 0-1.5 0v5a.75.75 0 0 0 1.5 0v-5Zm-3 .75a.75.75 0 0 1 1.5 0v4.25a.75.75 0 0 1-1.5 0V5Zm-3 1.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3Z"/></svg>
                  {{ p.downloadCount.toLocaleString() }}
                </p>
              </div>
              <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-[var(--tx-muted)]"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"/></svg>
            </div>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>
