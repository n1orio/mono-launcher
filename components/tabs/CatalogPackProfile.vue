<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount } from "vue";
import { useLauncherCtx } from "~/composables/useLauncherContext";

const emit = defineEmits<{ (e: "back"): void }>();
const ctx = useLauncherCtx();
const {
  t,
  modPackDetail,
  modPackVersions,
  modPackTab,
  modPackInstalling,
  quickDownloadPack,
  quickPackBusy,
  openExternal,
  installPackVersion,
  cpProject,
  cpFiles,
  cpDetail,
  cpTab,
  cpBusy,
  installCpPack,
  modInstallBusy,
  modPackTabs,
  cpTabs,
  formatDate,
} = ctx;

const isModrinth = computed(() => !!modPackDetail.value);
const title = computed(() => modPackDetail.value?.title ?? cpProject.value?.name ?? "");
const author = computed(() => modPackDetail.value?.author ?? cpProject.value?.author ?? "");
const downloads = computed(() => modPackDetail.value?.downloads ?? cpProject.value?.downloadCount ?? 0);
const iconUrl = computed(() => modPackDetail.value?.iconUrl ?? cpProject.value?.iconUrl ?? "");
const externalUrl = computed(() =>
  modPackDetail.value?.slug
    ? `https://modrinth.com/modpack/${modPackDetail.value.slug}`
    : (cpDetail.value?.websiteUrl || "")
);

/** Мета сборки для hero: лоадеры и версии игры. */
const metaLine = computed(() => {
  if (modPackDetail.value) {
    const vers = (modPackVersions.value ?? []) as any[];
    const loaders = [...new Set(vers.flatMap((v) => v.loaders ?? []))].slice(0, 3);
    const mcs = [...new Set(vers.flatMap((v) => v.gameVersions ?? []))].slice(0, 2);
    if (loaders.length || mcs.length) return [...loaders, ...mcs].join(" / ");
    return (modPackDetail.value.categories ?? []).slice(0, 4).join(" / ");
  }
  const cats = cpDetail.value?.categories ?? [];
  return cats.slice(0, 4).join(" / ");
});

const gallery = computed(() => {
  if (modPackDetail.value) return (modPackDetail.value.gallery ?? []) as { url: string; title?: string }[];
  return ((cpDetail.value?.screenshots ?? []) as { url: string; title?: string }[]).map((s) => ({ url: s.url, title: s.title }));
});

function cfLoaders(f: any): string[] {
  const out: string[] = [];
  for (const v of f.gameVersions ?? []) {
    const low = String(v).toLowerCase();
    if (["forge", "fabric", "quilt", "neoforge"].some((l) => low.includes(l))) out.push(v);
  }
  return out;
}

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") emit("back");
}
onMounted(() => window.addEventListener("keydown", onKey));
onBeforeUnmount(() => window.removeEventListener("keydown", onKey));
</script>

<template>
  <div class="min-h-0 flex-1 overflow-y-auto pr-1 pb-6">
    <button
      type="button"
      class="text-xs font-semibold px-3.5 py-2 rounded-xl bg-[var(--input)] hover:bg-[var(--panel)] border border-[var(--border)] text-[color:var(--tx)] flex items-center gap-2 mb-4 w-fit active:scale-95 transition-all"
      @click="emit('back')"
    >
      <AppIcon name="chevron-right" class="h-3.5 w-3.5 fill-current -rotate-180" />
      {{ t("catalog.backToCatalog") }}
    </button>

    <!-- Hero -->
    <div class="rounded-3xl bg-[var(--input)]/30 border border-[var(--border)] p-6 mb-6 flex items-center justify-between gap-6 shadow-sm">
      <div class="flex items-center gap-5 min-w-0">
        <img
          v-if="iconUrl"
          :src="iconUrl"
          :alt="title"
          loading="lazy"
          class="w-20 h-20 rounded-2xl object-contain bg-black/20 p-1.5 shadow-md shrink-0"
        />
        <div v-else class="w-20 h-20 rounded-2xl bg-[var(--input)] flex items-center justify-center font-black text-2xl text-[color:var(--tx-muted)] shrink-0">
          {{ title.slice(0, 2).toUpperCase() }}
        </div>
        <div class="flex flex-col gap-1 min-w-0">
          <h1 class="text-2xl font-black text-[color:var(--tx)] tracking-tight truncate">{{ title }}</h1>
          <div class="flex items-center gap-3 text-xs text-[color:var(--tx-muted)] flex-wrap">
            <span>{{ t("mods.byAuthor", { author }) }}</span>
            <span>•</span>
            <span class="flex items-center gap-1">
              <AppIcon name="download-bars" class="h-3 w-3 fill-current" />
              {{ downloads.toLocaleString() }}
            </span>
            <template v-if="metaLine">
              <span>•</span>
              <span>{{ metaLine }}</span>
            </template>
          </div>
        </div>
      </div>
      <div class="flex items-center gap-3 shrink-0">
        <button
          v-if="isModrinth"
          type="button"
          class="px-6 py-3 rounded-2xl bg-[var(--accent)] hover:brightness-110 text-white font-bold text-sm shadow-xl active:scale-95 transition-all flex items-center gap-2 disabled:opacity-50"
          :disabled="quickPackBusy !== null || modPackInstalling !== null"
          @click="quickDownloadPack(modPackDetail, $event)"
        >
          <AppIcon v-if="quickPackBusy === modPackDetail?.projectId" name="spinner" class="h-4 w-4 fill-current" />
          <AppIcon v-else name="arrow-down" class="h-4 w-4 fill-current" />
          <span>{{ t("catalog.installPack") }}</span>
        </button>
        <button
          v-else
          type="button"
          class="px-6 py-3 rounded-2xl bg-[var(--accent)] hover:brightness-110 text-white font-bold text-sm shadow-xl active:scale-95 transition-all flex items-center gap-2 disabled:opacity-50"
          :disabled="cpBusy !== null || !(cpFiles?.length)"
          @click="installCpPack(cpFiles![0])"
        >
          <AppIcon v-if="cpBusy !== null" name="spinner" class="h-4 w-4 fill-current" />
          <AppIcon v-else name="arrow-down" class="h-4 w-4 fill-current" />
          <span>{{ t("catalog.installPack") }}</span>
        </button>
        <button
          v-if="externalUrl"
          type="button"
          class="px-4 py-3 rounded-2xl bg-[var(--input)] hover:bg-[var(--panel)] border border-[var(--border)] text-xs font-semibold text-[color:var(--tx)] transition-all flex items-center gap-1.5"
          @click="openExternal(externalUrl)"
        >
          <span>{{ isModrinth ? t("mods.openPage") : t("curse.openPage") }}</span>
          <AppIcon name="external-link" class="h-3 w-3 fill-current" />
        </button>
      </div>
    </div>

    <!-- Tabs -->
    <div class="flex items-center gap-2 border-b border-[var(--border)] pb-3 mb-5">
      <template v-if="isModrinth">
        <button
          v-for="tb in modPackTabs"
          :key="tb.kind"
          type="button"
          class="px-3 py-1.5 rounded-xl text-xs font-semibold transition-all border"
          :class="modPackTab === tb.kind
            ? 'bg-[var(--panel)] text-[color:var(--tx)] border-[var(--border)] shadow-sm'
            : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx)] hover:bg-[var(--hover)] border-transparent'"
          @click="modPackTab = tb.kind"
        >
          {{ t("mods.tab" + tb.kind) }}<span v-if="tb.kind === 'versions' && modPackVersions?.length"> ({{ modPackVersions.length }})</span>
        </button>
      </template>
      <template v-else>
        <button
          v-for="tb in cpTabs"
          :key="tb"
          type="button"
          class="px-3 py-1.5 rounded-xl text-xs font-semibold transition-all border"
          :class="cpTab === tb
            ? 'bg-[var(--panel)] text-[color:var(--tx)] border-[var(--border)] shadow-sm'
            : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx)] hover:bg-[var(--hover)] border-transparent'"
          @click="cpTab = tb"
        >
          {{ t("curse.tab" + tb) }}<span v-if="tb === 'versions' && cpFiles?.length"> ({{ cpFiles.length }})</span>
        </button>
      </template>
    </div>

    <!-- About -->
    <div v-if="(isModrinth ? modPackTab : cpTab) === 'about'" class="max-w-4xl text-sm leading-relaxed text-[color:var(--tx-muted)]">
      <Markdown v-if="isModrinth && modPackDetail?.body" :source="modPackDetail.body" />
      <div v-else-if="!isModrinth && cpDetail?.description" v-html="cpDetail.description" class="prose prose-invert max-w-none text-sm leading-relaxed"></div>
      <p v-else-if="!isModrinth && cpDetail?.summary" class="text-sm leading-relaxed">{{ cpDetail.summary }}</p>
      <p v-else class="py-6 text-center text-[13px] italic">{{ t("mods.noAbout") }}</p>
    </div>

    <!-- Versions -->
    <div v-else-if="(isModrinth ? modPackTab : cpTab) === 'versions'" class="space-y-2 max-w-4xl">
      <template v-if="isModrinth">
        <div v-if="modPackVersions && modPackVersions.length === 0" class="py-8 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("mods.noVersions") }}</div>
        <div
          v-else-if="modPackVersions"
          v-for="v in modPackVersions"
          :key="v.id"
          class="flex items-center gap-4 rounded-xl bg-[var(--input)]/25 hover:bg-[var(--input)]/45 border border-[var(--border)] px-4 py-3 transition-all"
        >
          <div class="min-w-0 flex-1">
            <div class="text-sm font-bold text-[color:var(--tx)] truncate">{{ v.name }}</div>
            <div class="mt-0.5 truncate text-xs text-[color:var(--tx-muted)]">
              {{ (v.gameVersions ?? []).slice(0, 2).join(", ") }} • {{ (v.loaders ?? []).join(", ") }} • {{ formatDate(v.datePublished) }}
            </div>
          </div>
          <span class="shrink-0 rounded bg-[var(--input)] px-1.5 py-0.5 font-mono text-[11px] text-[color:var(--tx-muted)]">{{ v.versionNumber }}</span>
          <button
            type="button"
            class="flex shrink-0 items-center gap-1.5 px-3.5 py-1.5 rounded-xl text-xs font-semibold bg-[var(--accent)] hover:brightness-110 text-white shadow-sm active:scale-95 transition-all disabled:opacity-50"
            :disabled="modPackInstalling !== null || modInstallBusy !== null"
            @click="installPackVersion(v)"
          >
            <AppIcon v-if="modPackInstalling === v.id" name="spinner" class="h-3 w-3 fill-current" />
            <AppIcon v-else name="arrow-down" class="h-3 w-3 fill-current" />
            {{ t("mods.install") }}
          </button>
        </div>
        <div v-else class="flex items-center justify-center py-10 text-[13px] text-[color:var(--tx-muted)]">
          <AppIcon name="spinner" class="mr-2 h-4 w-4 fill-current" />
          {{ t("mods.searching") }}
        </div>
      </template>
      <template v-else>
        <div v-if="cpFiles === null" class="flex items-center justify-center py-10 text-[13px] text-[color:var(--tx-muted)]">
          <AppIcon name="spinner" class="mr-2 h-4 w-4 fill-current" />
          {{ t("mods.searching") }}
        </div>
        <div v-else-if="cpFiles.length === 0" class="py-8 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("curse.noFiles") }}</div>
        <div
          v-else
          v-for="f in cpFiles"
          :key="f.fileId"
          class="flex items-center gap-4 rounded-xl bg-[var(--input)]/25 hover:bg-[var(--input)]/45 border border-[var(--border)] px-4 py-3 transition-all"
        >
          <div class="min-w-0 flex-1">
            <div class="text-sm font-bold text-[color:var(--tx)] truncate">{{ f.displayName || f.fileName }}</div>
            <div class="mt-0.5 truncate text-xs text-[color:var(--tx-muted)]">
              {{ cfLoaders(f).join(", ") || "—" }} • {{ (f.gameVersions ?? []).filter((v: string) => /^\d/.test(v)).slice(0, 2).join(", ") }} • {{ f.fileDate ? formatDate(f.fileDate) : "" }}
            </div>
          </div>
          <button
            type="button"
            class="flex shrink-0 items-center gap-1.5 px-3.5 py-1.5 rounded-xl text-xs font-semibold bg-[var(--accent)] hover:brightness-110 text-white shadow-sm active:scale-95 transition-all disabled:opacity-50"
            :disabled="cpBusy !== null"
            :title="f.fileName"
            @click="installCpPack(f)"
          >
            <AppIcon v-if="cpBusy === f.fileId" name="spinner" class="h-3 w-3 fill-current" />
            <AppIcon v-else name="arrow-down" class="h-3 w-3 fill-current" />
            {{ t("mods.install") }}
          </button>
        </div>
      </template>
    </div>

    <!-- Gallery -->
    <div v-else class="max-w-4xl">
      <div v-if="gallery.length" class="grid grid-cols-2 gap-3">
        <div v-for="g in gallery" :key="g.url" class="group relative cursor-pointer overflow-hidden rounded-xl" @click="openExternal(g.url)">
          <img :src="g.url" :alt="g.title ?? ''" loading="lazy" class="w-full object-cover transition-transform group-hover:scale-[1.02]" />
          <div v-if="g.title" class="absolute inset-x-0 bottom-0 bg-black/60 px-3 py-1.5 text-xs text-white opacity-0 transition-opacity group-hover:opacity-100">{{ g.title }}</div>
        </div>
      </div>
      <p v-else class="py-10 text-center text-[13px] italic text-[color:var(--tx-muted)]">{{ t("mods.noGallery") }}</p>
    </div>
  </div>
</template>
