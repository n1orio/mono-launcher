<script setup lang="ts">
import { useLauncherCtx } from "~/composables/useLauncherContext";
import { curseforgeFileById } from "~/lib/bridge";
import type { CursePackFile } from "~/lib/types";
import { useI18n } from "#imports";

const { t } = useI18n();
const {
  searchOpen,
  isSearchWin,
  searchWinStyle,
  searchTitle,
  searchService,
  searchInput,
  searchLoading,
  modSearchKind,
  modDatapackWorldSel,
  worldOptions,
  modFilters,
  versionOptions,
  loaderOptions,
  modVersionTypeSel,
  versionTypeOptions,
  categoryOptions,
  modEnvSel,
  envOptions,
  modSortSel,
  sortSelectOptions,
  modSearchBox,
  selModrinth,
  multiSelBusy,
  quickModBusy,
  modInstallBusy,
  modSearchErr,
  modDetail,
  modDetailTab,
  modDetailTabs,
  modVersions,
  modVersionsRaw,
  modSearchLoading,
  modSearchResults,
  modSearchQuery,
  modFiltersActive,
  modSearchMore,
  modSearchMoreBusy,
  installedModrinthSlugs,
  status,
  curseKeyOk,
  curseLoaderSel,
  curseLoaderOptions,
  curseCatSel,
  curseCatOptions,
  curseVerSel,
  curseSortSel,
  curseSortOptions,
  curseLoading,
  curseSearched,
  curseResults,
  curseErr,
  selCurse,
  curseInstallBusy,
  installedCurseIds,
  curseDetail,
  curseDetailTab,
  curseVersions,
  openCurseDetail,
  closeCurseDetail,
  verFilterMcSel,
  verFilterMcOptions,
  verFilterLoaderSel,
  verFilterLoaderOptions,
  verFilterTypeSel,
  verFilterTypeOptions,
  filteredModVersions,
  cap,
  openExternal,
  formatDate,
  verTypeColor,
  searchIconUrl,
  dragSearchWin,
  closeSearch,
  switchSearchService,
  doSearch,
  searchMods,
  searchCurse,
  openModDetail,
  toggleModrinthSel,
  clearSelAll,
  downloadSelectedMods,
  quickDownloadMod,
  installModVersion,
  installCurse,
  toggleCurseSel,
  downloadSelectedCurse,
  loadMoreMods,
  resetModFiltersAndSearch,
  packId,
} = useLauncherCtx();

const CF_LOADERS = ["forge", "fabric", "quilt", "neoforge", "liteloader", "rift", "risugami's modloader", "rift"];

function cfGameVersions(f: { gameVersions: string[]; gameVersion: string }) {
  const loaders: string[] = [];
  const mcVersions: string[] = [];
  for (const v of f.gameVersions) {
    const low = v.toLowerCase();
    if (CF_LOADERS.some((l) => low.includes(l))) loaders.push(v);
    else if (/^\d+\.\d+/.test(v)) mcVersions.push(v);
  }
  return { loaders, mcVersions: mcVersions.length ? mcVersions : [f.gameVersion].filter(Boolean) };
}

const cfVerFilterMc = ref<string[]>([]);
const cfVerFilterLoader = ref<string[]>([]);
const cfVerFilterType = ref<string[]>([]);

const cfVerMcOptions = computed(() => {
  if (!curseVersions.value) return [];
  const set = new Set<string>();
  for (const f of curseVersions.value) for (const v of cfGameVersions(f).mcVersions) set.add(v);
  return [...set].sort().reverse().map((v) => ({ value: v, label: v }));
});

const cfVerLoaderOptions = computed(() => {
  if (!curseVersions.value) return [];
  const set = new Set<string>();
  for (const f of curseVersions.value) for (const l of cfGameVersions(f).loaders) set.add(l.toLowerCase());
  return [...set].sort().map((l) => ({ value: l, label: l }));
});

const cfVerTypeOptions = [
  { value: "1", label: "Release" },
  { value: "2", label: "Beta" },
  { value: "3", label: "Alpha" },
];

const filteredCfVersions = computed(() => {
  if (!curseVersions.value) return [];
  return curseVersions.value.filter((f: CursePackFile) => {
    const { loaders, mcVersions } = cfGameVersions(f);
    if (cfVerFilterMc.value.length && !mcVersions.some((v) => cfVerFilterMc.value.includes(v))) return false;
    if (cfVerFilterLoader.value.length && !loaders.some((l) => cfVerFilterLoader.value.some((fl) => l.toLowerCase() === fl.toLowerCase()))) return false;
    if (cfVerFilterType.value.length && !cfVerFilterType.value.includes(String(f.releaseType))) return false;
    return true;
  });
});

watch(curseDetail, () => {
  if (curseDetail.value) {
    const mc = status.value?.minecraft_version;
    const loader = status.value?.loader?.replace("-loader", "");
    cfVerFilterMc.value = mc ? [mc] : [];
    cfVerFilterLoader.value = loader ? [loader] : [];
  } else {
    cfVerFilterMc.value = [];
    cfVerFilterLoader.value = [];
  }
  cfVerFilterType.value = [];
});

async function installCurseFile(f: { fileId: number; fileName: string; displayName?: string }) {
  if (!curseDetail.value || !packId.value) return;
  curseInstallBusy.value = f.fileId;
  try {
    const file = await curseforgeFileById(curseDetail.value.projectId, f.fileId);
    const { curseforgeInstallFile } = await import("~/lib/bridge");
    const folder = modSearchKind.value === "resourcepack" ? "resourcepacks" : modSearchKind.value === "shaderpack" ? "shaderpacks" : "mods";
    await curseforgeInstallFile(packId.value, file, folder, curseDetail.value.name, curseDetail.value.iconUrl);
  } catch {
  } finally {
    curseInstallBusy.value = null;
  }
}
</script>

<template>
  <div
    v-if="searchOpen"
    class="fixed z-50"
    :class="isSearchWin ? 'inset-0' : ''"
    :style="isSearchWin ? undefined : searchWinStyle"
  >
    <div
      class="flex flex-col overflow-hidden bg-[var(--panel)]"
      :class="isSearchWin
        ? 'h-full w-full'
        : 'max-h-[85vh] w-[720px] max-w-[92vw] rounded-xl  shadow-2xl'"
    >
      <SubTitleBar v-if="isSearchWin" />
      <div
        class="flex shrink-0 items-center justify-between gap-3 border-b border-[var(--border)]  px-3.5 py-2.5"
        :class="isSearchWin ? '' : 'cursor-move'"
        @pointerdown="dragSearchWin"
      >
        <div class="flex min-w-0 items-center gap-2.5">
          <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg  bg-[var(--input)]">
            <svg viewBox="0 0 16 16" class="h-4 w-4 fill-[var(--accent)]"><path d="M10.68 1.997a5.5 5.5 0 0 1 4.553 8.573l.783 2.802a.5.5 0 0 1-.62.619l-2.775-.783A5.5 5.5 0 1 1 10.68 1.997ZM6.5 7A.75.75 0 0 0 6.5 8.5h4A.75.75 0 0 0 10.5 7h-4Zm0 3a.75.75 0 0 0 0 1.5h2.75a.75.75 0 0 0 0-1.5H6.5Z"/></svg>
          </div>
          <div class="min-w-0">
            <h3 class="truncate text-[15px] font-bold tracking-tight text-[color:var(--tx-strong)]">
              {{ searchTitle }}
            </h3>
            <p class="truncate text-xs leading-tight text-[color:var(--tx-muted)]">
              {{ searchService === "modrinth" ? t("mods.serviceModrinth") : t("mods.serviceCurseforge") }}
            </p>
          </div>
        </div>
        <div class="flex shrink-0 items-center gap-2">
          <div class="flex shrink-0 items-center gap-1 rounded-lg  bg-[var(--bg)] p-0.5">
            <button
              type="button"
              class="flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-[13px] font-semibold transition-colors"
              :class="searchService === 'modrinth'
                ? 'bg-[var(--input)] text-[color:var(--tx-strong)] shadow-sm'
                : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
              @click="switchSearchService('modrinth')"
            >
              <svg viewBox="0 0 24 24" class="h-4 w-4 shrink-0" :title="t('mods.serviceModrinth')"><path fill="#00AF5C" d="M12.252.004a11.78 11.768 0 0 0-8.92 3.73 11 10.999 0 0 0-2.17 3.11 11.37 11.359 0 0 0-1.16 5.169c0 1.42.17 2.5.6 3.77.24.759.77 1.899 1.17 2.529a12.3 12.298 0 0 0 8.85 5.639c.44.05 2.54.07 2.76.02.2-.04.22.1-.26-1.7l-.36-1.37-1.01-.06a8.5 8.489 0 0 1-5.18-1.8 5.34 5.34 0 0 1-1.3-1.26c0-.05.34-.28.74-.5a37.572 37.545 0 0 1 2.88-1.629c.03 0 .5.45 1.06.98l1 .97 2.07-.43 2.06-.43 1.47-1.47c.8-.8 1.48-1.5 1.48-1.52 0-.09-.42-1.63-.46-1.7-.04-.06-.2-.03-1.02.18-.53.13-1.2.3-1.45.4l-.48.15-.53.53-.53.53-.93.1-.93.07-.52-.5a2.7 2.7 0 0 1-.96-1.7l-.13-.6.43-.57c.68-.9.68-.9 1.46-1.1.4-.1.65-.2.83-.33.13-.099.65-.579 1.14-1.069l.9-.9-.7-.7-.7-.7-1.95.54c-1.07.3-1.96.53-1.97.53-.03 0-2.23 2.48-2.63 2.97l-.29.35.28 1.03c.16.56.3 1.16.31 1.34l.03.3-.34.23c-.37.23-2.22 1.3-2.84 1.63-.36.2-.37.2-.44.1-.08-.1-.23-.6-.32-1.03-.18-.86-.17-2.75.02-3.73a8.84 8.839 0 0 1 7.9-6.93c.43-.03.77-.08.78-.1.06-.17.5-2.999.47-3.039-.01-.02-.1-.02-.2-.03Zm3.68.67c-.2 0-.3.1-.37.38-.06.23-.46 2.42-.46 2.52 0 .04.1.11.22.16a8.51 8.499 0 0 1 2.99 2 8.38 8.379 0 0 1 2.16 3.449 6.9 6.9 0 0 1 .4 2.8c0 1.07 0 1.27-.1 1.73a9.37 9.369 0 0 1-1.76 3.769c-.32.4-.98 1.06-1.37 1.38-.38.32-1.54 1.1-1.7 1.14-.1.03-.1.06-.07.26.03.18.64 2.56.7 2.78l.06.06a12.07 12.058 0 0 0 7.27-9.4c.13-.77.13-2.58 0-3.4a11.96 11.948 0 0 0-5.73-8.578c-.7-.42-2.05-1.06-2.25-1.06Z"/></svg>
              {{ t("mods.serviceModrinth") }}
            </button>
            <button
              v-if="modSearchKind !== 'datapack'"
              type="button"
              class="flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-[13px] font-semibold transition-colors"
              :class="searchService === 'curseforge'
                ? 'bg-[var(--input)] text-[color:var(--tx-strong)] shadow-sm'
                : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
              @click="switchSearchService('curseforge')"
            >
              <svg viewBox="0 0 24 24" class="h-4 w-4 shrink-0" :title="t('mods.serviceCurseforge')"><path fill="#F16436" d="M18.326 9.2145S23.2261 8.4418 24 6.1882h-7.5066V4.4H0l2.0318 2.3576V9.173s5.1267-.2665 7.1098 1.2372c2.7146 2.516-3.053 5.917-3.053 5.917L5.0995 19.6c1.5465-1.4726 4.494-3.3775 9.8983-3.2857-2.0565.65-4.1245 1.6651-5.7344 3.2857h10.9248l-1.0288-3.2726s-7.918-4.6688-.8336-7.1127z"/></svg>
              {{ t("mods.serviceCurseforge") }}
            </button>
          </div>
          <button
            type="button"
            class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
            :title="t('common.close')"
            @click="closeSearch"
          >
            <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
          </button>
        </div>
      </div>
      <div v-if="searchService === 'curseforge' && !curseKeyOk" class="border-b border-[var(--border)]  px-4 py-2.5">
        <p class="text-[13px] text-[color:var(--tx-muted)]">{{ t("curse.noKey") }}</p>
      </div>
      <div class="flex shrink-0 items-center gap-2 border-b border-[var(--border)]  px-3.5 py-2.5">
        <div class="relative min-w-0 flex-1">
          <svg viewBox="0 0 16 16" class="pointer-events-none absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 fill-[var(--tx-muted)]">
            <path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"/>
          </svg>
          <input
            v-model="searchInput"
            type="text"
            :placeholder="searchService === 'modrinth' ? t('mods.searchPlaceholder') : t('curse.searchPlaceholder')"
            class="w-full rounded-md  bg-[var(--bg)] py-1.5 pl-8 pr-3 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)] outline-none transition-colors "
            @keydown.enter="doSearch"
          />
        </div>
        <button
          type="button"
          class="flex shrink-0 items-center gap-1.5 rounded-md  bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
          :disabled="searchLoading || !searchInput.trim()"
          @click="doSearch"
        >
          <svg v-if="searchLoading" viewBox="0 0 16 16" class="h-4 w-4 animate-spin fill-current">
            <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
          </svg>
          <svg v-else viewBox="0 0 16 16" class="h-4 w-4 fill-current">
            <path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"/>
          </svg>
          {{ searchService === 'modrinth' ? t("mods.search") : t("curse.search") }}
        </button>
      </div>
      <div v-if="searchService === 'curseforge'" class="flex shrink-0 flex-wrap items-center gap-2 border-b border-[var(--border)]  px-4 py-2">
        <FilterSelect
          v-model="curseLoaderSel"
          :options="curseLoaderOptions"
          :placeholder="t('curse.fLoader')"
          :multiple="false"
          @change="searchCurse()"
        />
        <FilterSelect
          v-model="curseCatSel"
          :options="curseCatOptions"
          :placeholder="t('curse.fCategory')"
          :multiple="true"
          @change="searchCurse()"
        />
        <FilterSelect
          v-model="curseVerSel"
          :options="versionOptions"
          :placeholder="t('curse.fVersion')"
          :multiple="false"
          @change="searchCurse()"
        />
        <FilterSelect
          v-model="curseSortSel"
          :options="curseSortOptions"
          :placeholder="t('curse.fSort')"
          :multiple="false"
          @change="searchCurse()"
        />
      </div>
      <div v-if="searchService === 'modrinth'" class="flex shrink-0 flex-wrap items-center gap-2 border-b border-[var(--border)]  px-4 py-2">
        <FilterSelect
          v-if="modSearchKind === 'datapack'"
          v-model="modDatapackWorldSel"
          :options="worldOptions"
          :placeholder="t('mods.fWorld')"
          :multiple="false"
        />
        <FilterSelect
          v-model="modFilters.versions"
          :options="versionOptions"
          :placeholder="t('mods.fVersion')"
          @change="searchMods()"
        />
        <FilterSelect
          v-model="modFilters.loaders"
          :options="loaderOptions"
          :placeholder="t('mods.fLoader')"
          @change="searchMods()"
        />
        <FilterSelect
          v-model="modVersionTypeSel"
          :options="versionTypeOptions"
          :placeholder="t('mods.fType')"
          :multiple="false"
          @change="searchMods()"
        />
        <FilterSelect
          v-model="modFilters.categories"
          :options="categoryOptions"
          :placeholder="t('mods.fCategory')"
          @change="searchMods()"
        />
        <FilterSelect
          v-model="modEnvSel"
          :options="envOptions"
          :placeholder="t('mods.fAny')"
          :multiple="false"
          @change="searchMods()"
        />
        <FilterSelect
          v-model="modSortSel"
          :options="sortSelectOptions"
          :placeholder="t('mods.fSort')"
          :multiple="false"
          @change="searchMods()"
        />
      </div>
      <div v-if="searchService === 'modrinth'" ref="modSearchBox" class="min-h-0 flex-1 overflow-y-auto p-4">
        <div
          v-if="selModrinth.size > 0 && !modSearchLoading && !modDetail"
          class="mb-3 flex flex-wrap items-center gap-2 rounded-md  bg-[color-mix(in_srgb,var(--accent)_8%,transparent)] px-3 py-2"
        >
          <span class="text-[13px] font-medium text-[var(--accent)]">
            {{ t("mods.selected", { n: selModrinth.size }) }}
          </span>
          <button
            type="button"
            class="flex shrink-0 items-center gap-1.5 rounded-md  bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_25%,transparent)] disabled:opacity-50"
            :disabled="multiSelBusy || quickModBusy !== null || modInstallBusy !== null"
            @click="downloadSelectedMods"
          >
            <svg v-if="multiSelBusy" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
              <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
            </svg>
            <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
              <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
            </svg>
            {{ multiSelBusy ? t("mods.installingSel") : t("mods.downloadSel") }}
          </button>
          <button
            type="button"
            class="rounded-md px-1.5 py-0.5 text-[13px] text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
            :title="t('files.clear')"
            @click="clearSelAll"
          >
            ×
          </button>
        </div>
        <div v-if="modSearchErr" class="rounded-md  bg-[var(--input-50)] p-6 text-center text-[13px] text-[color:var(--tx-muted)]">
          <p class="mb-2">{{ modSearchErr }}</p>
          <button type="button" class="text-[var(--accent)] hover:underline" @click="searchMods">{{ t("catalog.retry") }}</button>
        </div>
        <template v-else-if="modDetail">
          <button
            type="button"
            class="mb-3 flex items-center gap-1 text-[13px] text-[color:var(--tx-muted)] transition-colors hover:text-[var(--accent)]"
            @click="modDetail = null; modVersions = null"
          >
            <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M7.28 3.22a.75.75 0 0 1 0 1.06L3.56 8l3.72 3.72a.75.75 0 1 1-1.06 1.06l-4.25-4.25a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0Zm4 0a.75.75 0 0 1 0 1.06L7.56 8l3.72 3.72a.75.75 0 1 1-1.06 1.06l-4.25-4.25a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0Z"/></svg>
            {{ t("mods.back") }}
          </button>
          <div class="mb-3 flex items-center gap-3 rounded-md  bg-[var(--bg)] px-3 py-2.5">
            <img v-if="modDetail.iconUrl" :src="searchIconUrl(modDetail.iconUrl)" :alt="modDetail.title" loading="lazy" class="h-11 w-11 shrink-0 rounded-md object-cover" />
            <div v-else class="flex h-11 w-11 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-[13px] text-[color:var(--tx-muted)]">
              {{ modDetail.title.slice(0, 2).toUpperCase() }}
            </div>
            <div class="min-w-0 flex-1">
              <h4 class="truncate text-sm font-semibold text-[color:var(--tx-strong)]">{{ modDetail.title }}</h4>
              <div class="mt-0.5 flex flex-wrap items-center gap-3 text-xs text-[color:var(--tx-muted)]">
                <span>{{ t("mods.byAuthor", { author: modDetail.author }) }}</span>
                <span class="flex items-center gap-1">
                  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1.75a.75.75 0 0 0-1.5 0v9A2.25 2.25 0 0 0 2.5 13h12.75a.75.75 0 0 0 0-1.5H2.5a.75.75 0 0 1-.75-.75v-9Zm10.75 2.5a.75.75 0 0 0-1.5 0v5a.75.75 0 0 0 1.5 0v-5Zm-3 .75a.75.75 0 0 1 1.5 0v4.25a.75.75 0 0 1-1.5 0V5Zm-3 1.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3Z"/></svg>
                  {{ modDetail.downloads.toLocaleString() }}
                </span>
                <span v-if="modDetail.categories.length">{{ modDetail.categories.slice(0, 4).join(", ") }}</span>
                <button
                  v-if="modDetail.slug"
                  type="button"
                  class="text-[var(--accent)] hover:underline"
                  @click="openExternal(`https://modrinth.com/mod/${modDetail!.slug}`)"
                >
                  {{ t("mods.openPage") }}
                </button>
              </div>
            </div>
            <button
              type="button"
              class="flex shrink-0 items-center gap-1.5 rounded-md  bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
              :disabled="quickModBusy !== null || modInstallBusy !== null"
              :title="t('mods.downloadHint')"
              @click="quickDownloadMod(modDetail, $event)"
            >
              <svg v-if="quickModBusy === modDetail.projectId" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
              </svg>
              <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
              </svg>
              {{ t("mods.download") }}
            </button>
          </div>
          <div class="mb-3 flex shrink-0 items-center gap-1 border-b border-[var(--border)]  pb-2">
            <button
              v-for="tb in modDetailTabs"
              :key="tb.kind"
              type="button"
              class="rounded-md px-2.5 py-1.5 text-[13px] font-medium transition-colors"
              :class="modDetailTab === tb.kind
                ? 'bg-[var(--input)] text-[color:var(--tx-strong)]'
                : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]'"
              @click="modDetailTab = tb.kind"
            >
              {{ t("mods.tab" + tb.kind) }}
            </button>
          </div>
          <div v-if="modDetailTab === 'about'" class="max-h-[46vh] overflow-y-auto rounded-md  bg-[var(--bg)] px-3.5 py-2.5">
            <Markdown v-if="modDetail.body" :source="modDetail.body" />
            <p v-else class="py-6 text-center text-[13px] italic text-[color:var(--tx-muted)]">{{ t("mods.noAbout") }}</p>
          </div>
          <div v-else-if="modDetailTab === 'versions'">
            <div v-if="modVersions === null" class="flex items-center justify-center py-10 text-[13px] text-[color:var(--tx-muted)]">
              <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current">
                <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
              </svg>
              {{ t("mods.searching") }}
            </div>
            <div v-else-if="modVersions.length === 0" class="py-8 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("mods.noVersions") }}</div>
            <div v-else class="space-y-2">
              <div v-if="modVersionsRaw.length > 1" class="flex flex-wrap items-center gap-2">
                <FilterSelect v-model="verFilterMcSel" :options="verFilterMcOptions" :placeholder="t('curse.fVersion')" :multiple="true" />
                <FilterSelect v-model="verFilterLoaderSel" :options="verFilterLoaderOptions" :placeholder="t('curse.fLoader')" :multiple="true" />
                <FilterSelect v-model="verFilterTypeSel" :options="verFilterTypeOptions" :placeholder="t('mods.fType')" :multiple="true" />
              </div>
              <p v-if="filteredModVersions.length === 0" class="py-6 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("mods.noVersions") }}</p>
              <div
                v-for="v in filteredModVersions"
                :key="v.id"
                class="flex items-center gap-2 rounded-md  bg-[var(--bg)] py-1.5 pl-2.5 pr-1.5"
              >
                <span
                  class="h-2 w-2 shrink-0 rounded-full"
                  :style="{ backgroundColor: verTypeColor(v.versionType) }"
                  :title="t('mods.verType.' + v.versionType)"
                ></span>
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-1.5">
                    <span class="truncate text-[13px] font-medium text-[color:var(--tx-strong)]">{{ v.name }}</span>
                  </div>
                  <div class="truncate text-xs text-[color:var(--tx-muted)]">
                    {{ v.loaders.map(cap).join(" · ") || "vanilla" }} · {{ v.gameVersions.slice(0, 2).join(", ") }} · {{ formatDate(v.datePublished) }}
                  </div>
                </div>
                <span class="shrink-0 rounded  bg-[var(--input-50)] px-1.5 py-px font-mono text-[11px] text-[color:var(--tx-muted)]">{{ v.versionNumber }}</span>
                <button
                  type="button"
                  class="flex shrink-0 items-center gap-1 rounded-md  bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-2 py-1 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                  :disabled="modInstallBusy !== null"
                  :title="v.versionNumber"
                  @click="installModVersion(v)"
                >
                  <svg v-if="modInstallBusy === v.id" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                    <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                  </svg>
                  <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                    <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
                  </svg>
                </button>
              </div>
            </div>
          </div>
          <div v-else>
            <div v-if="modDetail.gallery.length" class="grid grid-cols-2 gap-2">
              <img v-for="g in modDetail.gallery" :key="g.url" :src="g.url" :alt="g.title ?? ''" loading="lazy" class="h-32 w-full cursor-zoom-in rounded-md  object-cover transition-transform hover:scale-[1.02]" :title="g.title ?? undefined" @click="openExternal(g.url)" />
            </div>
            <p v-else class="py-10 text-center text-[13px] italic text-[color:var(--tx-muted)]">{{ t("mods.noGallery") }}</p>
          </div>
        </template>
        <template v-else-if="modSearchLoading">
          <div class="flex items-center justify-center py-16 text-[13px] text-[color:var(--tx-muted)]">
            <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current">
              <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
            </svg>
            {{ t("mods.searching") }}
          </div>
        </template>
        <template v-else-if="modSearchResults.length === 0">
          <div class="py-16 text-center text-[13px] text-[color:var(--tx-muted)]">
            <p class="mb-3">{{ modSearchQuery ? t("mods.noResults") : t("mods.help") }}</p>
            <button
              v-if="modFiltersActive"
              type="button"
              class="inline-flex items-center gap-1.5 rounded-md  bg-[var(--input)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
              @click="resetModFiltersAndSearch"
            >
              <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1A1.75 1.75 0 0 0 0 2.75v3.5C0 7.216.784 8 1.75 8h3.5A1.75 1.75 0 0 0 7 6.25v-3.5A1.75 1.75 0 0 0 5.25 1h-3.5ZM1 2.75a.75.75 0 0 1 .75-.75h3.5a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-.75.75h-3.5a.75.75 0 0 1-.75-.75v-3.5Zm9-1.75A1.75 1.75 0 0 0 8.25 2.75v3.5A1.75 1.75 0 0 0 10 8h3.5A1.75 1.75 0 0 0 15.25 6.25v-3.5A1.75 1.75 0 0 0 13.5 1H10Zm-.75 1.75a.75.75 0 0 1 .75-.75h3.5a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-.75.75H10a.75.75 0 0 1-.75-.75v-3.5ZM1.75 9A1.75 1.75 0 0 0 0 10.75v3.5A1.75 1.75 0 0 0 1.75 16h3.5A1.75 1.75 0 0 0 7 14.25v-3.5A1.75 1.75 0 0 0 5.25 9h-3.5Zm-.75 1.75a.75.75 0 0 1 .75-.75h3.5a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-.75.75h-3.5a.75.75 0 0 1-.75-.75v-3.5ZM10 9A1.75 1.75 0 0 0 8.25 10.75v3.5A1.75 1.75 0 0 0 10 16h3.5A1.75 1.75 0 0 0 15.25 14.25v-3.5A1.75 1.75 0 0 0 13.5 9H10Zm-.75 1.75a.75.75 0 0 1 .75-.75h3.5a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-.75.75H10a.75.75 0 0 1-.75-.75v-3.5Z"/></svg>
              {{ t("mods.resetFilters") }}
            </button>
          </div>
        </template>
        <template v-else>
          <div class="space-y-2">
            <div
              v-for="p in modSearchResults"
              :key="p.projectId"
              class="flex cursor-pointer items-start gap-3 rounded-md  bg-[var(--bg)] px-3 py-2.5 transition-colors "
              @click="openModDetail(p)"
            >
              <button
                type="button"
                class="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-md  transition-colors"
                :class="selModrinth.has(p.projectId)
                  ? ' bg-[color-mix(in_srgb,var(--accent)_20%,transparent)]'
                  : ' '"
                :title="selModrinth.has(p.projectId) ? t('files.clear') : t('mods.selForDownload')"
                @click.stop="toggleModrinthSel(p.projectId)"
              >
                <svg v-if="selModrinth.has(p.projectId)" viewBox="0 0 16 16" class="h-3 w-3 fill-[var(--accent)]">
                  <path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/>
                </svg>
              </button>
              <img v-if="p.iconUrl" :src="searchIconUrl(p.iconUrl)" alt="" loading="lazy" class="h-10 w-10 shrink-0 rounded-md object-cover" />
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
                :disabled="quickModBusy !== null || modInstallBusy !== null || installedModrinthSlugs.has(p.slug)"
                :title="t('mods.downloadHint')"
                @click="quickDownloadMod(p, $event)"
              >
                <svg v-if="quickModBusy === p.projectId" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                  <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                </svg>
                <svg v-else-if="installedModrinthSlugs.has(p.slug)" viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                  <path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/>
                </svg>
                <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                  <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
                </svg>
                {{ installedModrinthSlugs.has(p.slug) ? t("mods.installedBadge") : t("mods.download") }}
              </button>
              <svg viewBox="0 0 16 16" class="mt-1 h-4 w-4 shrink-0 fill-[var(--tx-muted)]"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"/></svg>
            </div>
          </div>
        </template>
        <div v-if="modSearchMore || modSearchMoreBusy" class="flex justify-center py-4">
          <svg v-if="modSearchMoreBusy" viewBox="0 0 16 16" class="h-4 w-4 animate-spin fill-[var(--tx-muted)]">
            <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
          </svg>
          <button
            v-else
            type="button"
            class="text-[13px] font-medium text-[var(--accent)] hover:underline"
            @click="loadMoreMods"
          >{{ t("mods.loadMore") }}</button>
        </div>
      </div>
      <div v-else class="min-h-0 flex-1 overflow-y-auto px-3.5 py-2.5">
        <div
          v-if="selCurse.size > 0 && !curseLoading"
          class="mb-3 flex flex-wrap items-center gap-2 rounded-md  bg-[color-mix(in_srgb,var(--accent)_8%,transparent)] px-3 py-2"
        >
          <span class="text-[13px] font-medium text-[var(--accent)]">
            {{ t("mods.selected", { n: selCurse.size }) }}
          </span>
          <button
            type="button"
            class="flex shrink-0 items-center gap-1.5 rounded-md  bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_25%,transparent)] disabled:opacity-50"
            :disabled="multiSelBusy || curseInstallBusy !== null"
            @click="downloadSelectedCurse"
          >
            <svg v-if="multiSelBusy" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
              <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
            </svg>
            <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
              <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
            </svg>
            {{ multiSelBusy ? t("mods.installingSel") : t("mods.downloadSel") }}
          </button>
          <button
            type="button"
            class="rounded-md px-1.5 py-0.5 text-[13px] text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
            :title="t('files.clear')"
            @click="clearSelAll"
          >
            ×
          </button>
        </div>
        <p v-if="!curseSearched" class="py-8 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("curse.help") }}</p>
        <p v-else-if="curseLoading" class="flex items-center justify-center gap-2 py-8 text-[13px] text-[color:var(--tx-muted)]">
          <svg viewBox="0 0 16 16" class="h-4 w-4 animate-spin fill-current">
            <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
          </svg>
          {{ t("mods.searchingAll") }}
        </p>
        <template v-else-if="curseDetail">
          <button
            type="button"
            class="mb-3 flex items-center gap-1 text-[13px] text-[color:var(--tx-muted)] transition-colors hover:text-[var(--accent)]"
            @click="closeCurseDetail()"
          >
            <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M7.28 3.22a.75.75 0 0 1 0 1.06L3.56 8l3.72 3.72a.75.75 0 1 1-1.06 1.06l-4.25-4.25a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0Zm4 0a.75.75 0 0 1 0 1.06L7.56 8l3.72 3.72a.75.75 0 1 1-1.06 1.06l-4.25-4.25a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0Z"/></svg>
            {{ t("mods.back") }}
          </button>
          <div class="mb-3 flex items-center gap-3 rounded-md bg-[var(--bg)] px-3 py-2.5">
            <img v-if="curseDetail.iconUrl" :src="searchIconUrl(curseDetail.iconUrl)" :alt="curseDetail.name" loading="lazy" class="h-11 w-11 shrink-0 rounded-md object-cover" />
            <div v-else class="flex h-11 w-11 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-[13px] text-[color:var(--tx-muted)]">
              {{ curseDetail.name.slice(0, 2).toUpperCase() }}
            </div>
            <div class="min-w-0 flex-1">
              <h4 class="truncate text-sm font-semibold text-[color:var(--tx-strong)]">{{ curseDetail.name }}</h4>
              <div class="mt-0.5 flex flex-wrap items-center gap-3 text-xs text-[color:var(--tx-muted)]">
                <span>{{ t("mods.byAuthor", { author: curseDetail.author }) }}</span>
                <span class="flex items-center gap-1">
                  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1.75a.75.75 0 0 0-1.5 0v9A2.25 2.25 0 0 0 2.5 13h12.75a.75.75 0 0 0 0-1.5H2.5a.75.75 0 0 1-.75-.75v-9Zm10.75 2.5a.75.75 0 0 0-1.5 0v5a.75.75 0 0 0 1.5 0v-5Zm-3 .75a.75.75 0 0 1 1.5 0v4.25a.75.75 0 0 1-1.5 0V5Zm-3 1.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3Z"/></svg>
                  {{ curseDetail.downloadCount.toLocaleString() }}
                </span>
                <span v-if="curseDetail.categories.length">{{ curseDetail.categories.slice(0, 4).join(", ") }}</span>
                <button
                  v-if="curseDetail.websiteUrl"
                  type="button"
                  class="text-[var(--accent)] hover:underline"
                  @click="openExternal(curseDetail!.websiteUrl)"
                >
                  {{ t("mods.openPage") }}
                </button>
              </div>
            </div>
            <button
              type="button"
              class="shrink-0 rounded-md bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
              :disabled="curseInstallBusy !== null || installedCurseIds.has(curseDetail!.projectId)"
              @click="installCurse(curseDetail!)"
            >
              <template v-if="installedCurseIds.has(curseDetail!.projectId)">
                <svg viewBox="0 0 16 16" class="mr-1 inline h-3 w-3 fill-current"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/></svg>
                {{ t("mods.installedBadge") }}
              </template>
              <template v-else>{{ t("mods.download") }}</template>
            </button>
          </div>
          <div class="mb-3 flex shrink-0 items-center gap-1 border-b border-[var(--border)] pb-2">
            <button
              type="button"
              class="rounded-md px-2.5 py-1.5 text-[13px] font-medium transition-colors"
              :class="curseDetailTab === 'about' ? 'bg-[var(--input)] text-[color:var(--tx-strong)]' : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]'"
              @click="curseDetailTab = 'about'"
            >
              {{ t("mods.tababout") }}
            </button>
            <button
              type="button"
              class="rounded-md px-2.5 py-1.5 text-[13px] font-medium transition-colors"
              :class="curseDetailTab === 'versions' ? 'bg-[var(--input)] text-[color:var(--tx-strong)]' : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]'"
              @click="curseDetailTab = 'versions'"
            >
              {{ t("mods.tabversions") }}
            </button>
            <button
              type="button"
              class="rounded-md px-2.5 py-1.5 text-[13px] font-medium transition-colors"
              :class="curseDetailTab === 'screenshots' ? 'bg-[var(--input)] text-[color:var(--tx-strong)]' : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]'"
              @click="curseDetailTab = 'screenshots'"
            >
              {{ t("mods.tabscreenshots") }}
            </button>
          </div>
          <div v-if="curseDetailTab === 'about'" class="max-h-[46vh] overflow-y-auto rounded-md bg-[var(--bg)] px-3.5 py-2.5">
            <div v-if="curseDetail.description" v-html="curseDetail.description" class="prose prose-invert max-w-none text-[13px] leading-relaxed"></div>
            <p v-else-if="curseDetail.summary" class="text-[13px] leading-relaxed text-[color:var(--tx)]">{{ curseDetail.summary }}</p>
            <p v-else class="py-6 text-center text-[13px] italic text-[color:var(--tx-muted)]">{{ t("mods.noAbout") }}</p>
          </div>
          <div v-else-if="curseDetailTab === 'versions'" class="max-h-[46vh] overflow-y-auto">
            <div v-if="curseVersions === null" class="flex items-center justify-center py-10 text-[13px] text-[color:var(--tx-muted)]">
              <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current">
                <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
              </svg>
              {{ t("mods.searching") }}
            </div>
            <div v-else-if="curseVersions.length === 0" class="py-8 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("mods.noVersions") }}</div>
            <div v-else class="space-y-2">
              <div v-if="curseVersions.length > 1" class="flex flex-wrap items-center gap-2">
                <FilterSelect v-model="cfVerFilterMc" :options="cfVerMcOptions" :placeholder="t('curse.fVersion')" :multiple="true" />
                <FilterSelect v-model="cfVerFilterLoader" :options="cfVerLoaderOptions" :placeholder="t('curse.fLoader')" :multiple="true" />
                <FilterSelect v-model="cfVerFilterType" :options="cfVerTypeOptions" :placeholder="t('mods.fType')" :multiple="true" />
              </div>
              <p v-if="filteredCfVersions.length === 0" class="py-6 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("mods.noVersions") }}</p>
              <div
                v-for="f in filteredCfVersions"
                :key="f.fileId"
                class="flex items-center gap-2 rounded-md bg-[var(--bg)] py-1.5 pl-2.5 pr-1.5"
              >
                <span
                  class="h-2 w-2 shrink-0 rounded-full"
                  :style="{ backgroundColor: verTypeColor(f.releaseType === 1 ? 'release' : f.releaseType === 2 ? 'beta' : 'alpha') }"
                  :title="f.releaseType === 1 ? 'Release' : f.releaseType === 2 ? 'Beta' : 'Alpha'"
                ></span>
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-1.5">
                    <span class="truncate text-[13px] font-medium text-[color:var(--tx-strong)]">{{ f.displayName || f.fileName }}</span>
                  </div>
                  <div class="truncate text-xs text-[color:var(--tx-muted)]">
                    {{ cfGameVersions(f).loaders.map(cap).join(" · ") || "—" }} · {{ cfGameVersions(f).mcVersions.join(", ") }} · {{ f.fileDate ? formatDate(f.fileDate) : "" }}
                  </div>
                </div>
                <span class="shrink-0 rounded bg-[var(--input-50)] px-1.5 py-px font-mono text-[11px] text-[color:var(--tx-muted)]">{{ f.displayName?.match(/v?([\d.]+)/)?.[1] || '' }}</span>
                <button
                  type="button"
                  class="flex shrink-0 items-center gap-1 rounded-md bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-2 py-1 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                  :disabled="curseInstallBusy !== null"
                  :title="f.fileName"
                  @click="installCurseFile(f)"
                >
                  <svg v-if="curseInstallBusy === f.fileId" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                    <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                  </svg>
                  <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                    <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
                  </svg>
                </button>
              </div>
            </div>
          </div>
          <div v-else-if="curseDetailTab === 'screenshots'" class="max-h-[46vh] overflow-y-auto">
            <div v-if="curseDetail.screenshots.length" class="grid grid-cols-2 gap-2">
              <div v-for="(s, i) in curseDetail.screenshots" :key="i" class="group relative cursor-pointer" @click="openExternal(s.url)">
                <img :src="s.url" :alt="s.title || ''" loading="lazy" class="h-32 w-full rounded-md object-cover transition-transform hover:scale-[1.02]" />
                <div v-if="s.title" class="absolute inset-x-0 bottom-0 rounded-b-md bg-black/60 px-2 py-1 text-[11px] text-white opacity-0 transition-opacity group-hover:opacity-100">{{ s.title }}</div>
              </div>
            </div>
            <p v-else class="py-10 text-center text-[13px] italic text-[color:var(--tx-muted)]">{{ t("mods.noGallery") }}</p>
          </div>
        </template>
        <div v-else-if="curseErr" class="rounded-md  bg-[var(--input-50)] p-6 text-center text-[13px] text-[color:var(--tx-muted)]">
          <p class="mb-2 whitespace-pre-wrap">{{ curseErr }}</p>
          <button type="button" class="text-[var(--accent)] hover:underline" @click="searchCurse">{{ t("catalog.retry") }}</button>
        </div>
        <p v-else-if="curseResults.length === 0" class="py-8 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("mods.noResults") }}</p>
        <div v-else class="space-y-2">
          <div
            v-for="p in curseResults"
            :key="p.projectId"
            class="flex cursor-pointer items-center gap-3 rounded-md bg-[var(--bg)] px-3 py-2 transition-colors hover:bg-[var(--hover)]"
            @click="openCurseDetail(p)"
          >
            <button
              type="button"
              class="flex h-6 w-6 shrink-0 items-center justify-center rounded-md  transition-colors"
              :class="selCurse.has(p.projectId)
                ? ' bg-[color-mix(in_srgb,var(--accent)_20%,transparent)]'
                : ' '"
              :title="selCurse.has(p.projectId) ? t('files.clear') : t('mods.selForDownload')"
              @click="toggleCurseSel(p.projectId)"
            >
              <svg v-if="selCurse.has(p.projectId)" viewBox="0 0 16 16" class="h-3 w-3 fill-[var(--accent)]">
                <path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/>
              </svg>
            </button>
            <img
              v-if="p.iconUrl"
              :src="searchIconUrl(p.iconUrl)"
              :alt="p.name"
              loading="lazy"
              class="h-10 w-10 shrink-0 rounded-md object-cover"
            />
            <div v-else class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-xs text-[color:var(--tx-muted)]">
              {{ p.name.slice(0, 2).toUpperCase() }}
            </div>
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <svg viewBox="0 0 24 24" class="h-3 w-3 shrink-0" :title="t('mods.serviceCurseforge')"><path fill="#F16436" d="M18.326 9.2145S23.2261 8.4418 24 6.1882h-7.5066V4.4H0l2.0318 2.3576V9.173s5.1267-.2665 7.1098 1.2372c2.7146 2.516-3.053 5.917-3.053 5.917L5.0995 19.6c1.5465-1.4726 4.494-3.3775 9.8983-3.2857-2.0565.65-4.1245 1.6651-5.7344 3.2857h10.9248l-1.0288-3.2726s-7.918-4.6688-.8336-7.1127z"/></svg>
                <span class="truncate text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ p.name }}</span>
                <span v-if="curseInstallBusy === p.projectId" class="text-xs text-[var(--accent)]">{{ t("curse.installing") }}</span>
              </div>
              <p class="line-clamp-1 text-xs text-[color:var(--tx-muted)]">{{ p.summary }}</p>
              <p class="mt-0.5 flex items-center gap-2 text-xs text-[color:var(--tx-muted)]">
                <span>{{ t("mods.byAuthor", { author: p.author }) }}</span>
                <span>{{ p.downloadCount.toLocaleString() }}</span>
              </p>
            </div>
            <button
              type="button"
              class="shrink-0 rounded-md  bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
              :disabled="curseInstallBusy !== null && curseInstallBusy !== p.projectId || installedCurseIds.has(p.projectId)"
              @click="installCurse(p)"
            >
              <template v-if="installedCurseIds.has(p.projectId)">
                <svg viewBox="0 0 16 16" class="mr-1 inline h-3 w-3 fill-current"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/></svg>
                {{ t("mods.installedBadge") }}
              </template>
              <template v-else>{{ t("mods.download") }}</template>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
