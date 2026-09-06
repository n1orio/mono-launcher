<script setup lang="ts">
import { computed } from "vue";
import { useLauncherCtx } from '~/composables/useLauncherContext';
const ctx = useLauncherCtx();
const {
  t,
  news,
  openExternal,
  newsSources,
  newsFilter,
  filteredNews,
  formatDate,
  openNewsLink,
  changelogLines,
  visibleNewsLines,
  renderInline,
  onChangelogLinkClick,
  isNewsExpandable,
  toggleNewsExpanded,
  isNewsExpanded,
  isInstalledVersion,
  isActiveNewsTag,
  installNews,
  installAppUpdate,
  appUpdating,
  launcherVer,
  packNameFor,
  busy,
} = ctx;

/** Тело-заглушка релиза (CI-дефолт) — не рендерим. */
const PLACEHOLDER_BODIES = new Set(["Сборка...", "Mono Launcher.", "Mono Launcher", "Сборка"]);
function hasRealBody(n: any): boolean {
  const b = (n?.body ?? "").trim();
  return b.length > 0 && !PLACEHOLDER_BODIES.has(b);
}

const isLauncherPost = (n: any) =>
  n?.pack_id === "launcher" || String(n?.tag ?? "").startsWith("launcher-v");
/** Версия из тега launcher-vX.Y.Z → X.Y.Z. */
function launcherVersionOf(n: any): string {
  const tag = String(n?.tag ?? "");
  return tag.startsWith("launcher-v") ? tag.slice("launcher-v".length) : tag.replace(/^v/i, "");
}
const currentVer = computed(() => String(launcherVer?.value ?? "").replace(/^v/i, ""));

function cmpVersions(a: string, b: string): number {
  const pa = a.split(/[^0-9a-zA-Z]+/).filter(Boolean);
  const pb = b.split(/[^0-9a-zA-Z]+/).filter(Boolean);
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const x = pa[i] ?? "";
    const y = pb[i] ?? "";
    if (x === y) continue;
    const nx = /^\d+$/.test(x) ? parseInt(x, 10) : null;
    const ny = /^\d+$/.test(y) ? parseInt(y, 10) : null;
    if (nx !== null && ny !== null) return nx - ny;
    return x < y ? -1 : 1;
  }
  return 0;
}
const isCurrentLauncher = (n: any) =>
  isLauncherPost(n) && !!launcherVersionOf(n) && launcherVersionOf(n) === currentVer.value;
const isNewerLauncher = (n: any) =>
  isLauncherPost(n) && !!launcherVersionOf(n) && !!currentVer.value && cmpVersions(launcherVersionOf(n), currentVer.value) > 0;
/** Чистый заголовок: у лаунчера — только версия, без дублей имени релиза. */
function cardTitle(n: any): string {
  if (isLauncherPost(n) && launcherVersionOf(n)) {
    const v = launcherVersionOf(n);
    return v.toLowerCase().startsWith("v") ? v : `v${v}`;
  }
  return n.title;
}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <div class="mb-6 shrink-0 border-b border-[var(--border)] pb-5">
      <h1 class="text-xl font-bold tracking-tight text-[color:var(--tx-strong)]">{{ t("news.title") }}</h1>
      <p class="mt-2 text-[13px] text-[color:var(--tx-muted)]">
        {{ t("news.subtitle") }}
      </p>
      <!-- Фильтры ленты: Все первым, дальше источники -->
      <div class="no-scrollbar mt-4 flex items-center gap-1.5 overflow-x-auto pb-1 mb-4">
        <button
          type="button"
          class="shrink-0 px-3 py-1.5 rounded-xl text-xs transition-all"
          :class="newsFilter === 'all'
            ? 'bg-[var(--panel)] text-[color:var(--tx)] font-semibold shadow-sm border border-[var(--border)]'
            : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx)] hover:bg-[var(--hover)] border border-transparent'"
          @click="newsFilter = 'all'"
        >
          {{ t("news.all") }}
        </button>
        <button
          v-for="src in newsSources"
          :key="src"
          type="button"
          class="shrink-0 px-3 py-1.5 rounded-xl text-xs transition-all"
          :class="newsFilter === src
            ? 'bg-[var(--panel)] text-[color:var(--tx)] font-semibold shadow-sm border border-[var(--border)]'
            : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx)] hover:bg-[var(--hover)] border border-transparent'"
          @click="newsFilter = src"
        >
          {{ src === "launcher" ? "Mono Launcher" : packNameFor(src) }}
        </button>
      </div>
    </div>

    <div v-if="news === null" class="flex flex-1 items-center justify-center text-[13px] text-[color:var(--tx-muted)]">
      <svg class="mr-2 h-4 w-4 animate-spin fill-[var(--accent)]" viewBox="0 0 16 16">
        <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
      </svg>
      {{ t("news.loading") }}
    </div>

    <div v-else-if="news.length === 0" class="shrink-0 rounded-xl bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]">
      {{ t("news.none") }}
    </div>

    <div v-else-if="filteredNews.length === 0" class="shrink-0 rounded-xl bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]">
      {{ t("news.emptyCat") }}
    </div>

    <div v-else class="min-h-0 flex-1 space-y-4 overflow-y-auto pr-1 pb-8">
      <article
        v-for="n in filteredNews"
        :key="`${n.kind}-${n.url || n.tag}`"
        class="rounded-2xl bg-[var(--input)]/35 hover:bg-[var(--input)]/50 border border-[var(--border)] p-4 mb-3 transition-all"
      >
        <div class="flex items-center justify-between gap-4 mb-2">
          <div class="min-w-0">
            <span
              v-if="n.kind === 'update'"
              class="text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-md"
              :class="isLauncherPost(n)
                ? 'bg-[var(--accent)] text-white'
                : 'bg-[#16a34a]/15 text-[#22c55e] border border-[#16a34a]/30'"
            >
              {{ isLauncherPost(n) ? t("news.launcherBadge") : t("news.packBadge") }}
            </span>
            <span
              v-else
              class="text-[10px] font-bold uppercase px-2 py-0.5 rounded-md border border-[var(--border)] text-[color:var(--tx-muted)]"
            >
              {{ t("news.post") }}
            </span>
            <h2 class="mt-1.5 text-base font-black text-[color:var(--tx)] tracking-tight break-words">
              {{ cardTitle(n) }}
            </h2>
            <p class="mt-0.5 text-xs text-[color:var(--tx-muted)]">
              {{ formatDate(n.date) }}<span v-if="!isLauncherPost(n) && n.pack_name"> · {{ n.pack_name }}</span>
            </p>
          </div>
          <div class="flex shrink-0 items-center gap-2">
            <!-- Лаунчер: текущая версия → тихий тег, новее → обновить -->
            <span
              v-if="isCurrentLauncher(n)"
              class="flex items-center gap-1.5 text-xs font-bold text-[#22c55e] bg-[#16a34a]/10 px-3 py-1.5 rounded-xl border border-[#16a34a]/20"
            >
              <AppIcon name="check" class="h-3.5 w-3.5 fill-current" />
              {{ t("news.currentVersion") }}
            </span>
            <button
              v-else-if="isNewerLauncher(n)"
              type="button"
              class="flex items-center gap-1.5 bg-[var(--accent-deep)] hover:brightness-110 text-white text-xs font-semibold px-3.5 py-1.5 rounded-xl shadow-sm active:scale-95 transition-all disabled:opacity-50"
              :disabled="appUpdating"
              @click="installAppUpdate()"
            >
              <AppIcon v-if="appUpdating" name="spinner" class="h-3.5 w-3.5 fill-current" />
              <AppIcon v-else name="refresh" class="h-3.5 w-3.5 fill-current" />
              {{ t("news.updateLauncher") }}
            </button>
            <!-- Сборки: установка/переключение версии -->
            <button
              v-else-if="n.kind === 'update' && !isLauncherPost(n) && n.tag"
              type="button"
              class="bg-[var(--accent-deep)] hover:brightness-110 text-white text-xs font-semibold px-3.5 py-1.5 rounded-xl shadow-sm active:scale-95 transition-all disabled:opacity-50"
              :disabled="busy"
              @click="installNews(n)"
            >
              {{ isInstalledVersion(n.tag) ? (isActiveNewsTag(n.tag) ? t("releases.selected") : t("releases.switch")) : t("news.installUpdate") }}
            </button>
            <button
              v-else-if="(n.kind === 'post' || isLauncherPost(n)) && n.url"
              type="button"
              class="bg-[var(--input)] hover:bg-white/10 text-[color:var(--tx-muted)] hover:text-[color:var(--tx)] text-xs font-semibold px-3.5 py-1.5 rounded-xl transition-all"
              @click="openNewsLink(n.url)"
            >
              {{ t("news.open") }}
            </button>
          </div>
        </div>

        <!-- Тело: ченджлог, заглушки скрыты -->
        <div v-if="hasRealBody(n) && changelogLines(n.body).length > 0" class="text-[13px] text-[color:var(--tx)] space-y-1.5">
          <div class="changelog space-y-1 font-sans" @click="onChangelogLinkClick">
            <template v-for="(line, idx) in visibleNewsLines(n)" :key="idx">
              <div v-if="line.type === 'bullet'" class="flex items-start gap-2 text-[color:var(--tx)]">
                <span class="text-[var(--accent)] select-none">•</span>
                <span class="space-y-1.5 text-xs text-[color:var(--tx-muted)]" v-html="renderInline(line.text)"></span>
              </div>
              <div v-else-if="line.type === 'body'" class="font-semibold text-[color:var(--tx-strong)] pt-1.5" v-html="renderInline(line.text)"></div>
              <div v-else class="text-xs text-[color:var(--tx-muted)]" v-html="renderInline(line.text)"></div>
            </template>
          </div>
          <button
            v-if="isNewsExpandable(n)"
            type="button"
            class="mt-2 inline-block text-[13px] font-medium text-[var(--accent)] hover:text-[var(--accent-strong)] transition-colors"
            @click="toggleNewsExpanded(n)"
          >
            {{ isNewsExpanded(n) ? t("news.collapse") : t("news.showAll") }}
          </button>
        </div>
      </article>
    </div>
  </div>
</template>
