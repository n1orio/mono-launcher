<script setup lang="ts">
import { useLauncherCtx } from '~/composables/useLauncherContext';
const ctx = useLauncherCtx();
const {
  t,
  activePack,
  news,
  openExternal,
  formatUnixDate,
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
  busy,
  packNameFor,
} = ctx;
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <div class="mb-6 shrink-0 border-b border-[var(--border)]  pb-5">
      <h1 class="text-xl font-bold tracking-tight text-[color:var(--tx-strong)]">{{ t("news.title") }}</h1>
      <p class="mt-2 text-[13px] text-[color:var(--tx-muted)]">
        {{ t("news.subtitle") }}
      </p>
      <div class="mt-4 flex flex-wrap items-center gap-2">
        <button
          v-for="src in newsSources"
          :key="src"
          type="button"
          class="rounded-full  px-3.5 py-1.5 text-[13px] font-medium transition-colors"
          :class="newsFilter === src
            ? ' bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] text-white'
            : ' bg-[var(--input)] text-[color:var(--tx-muted)] hover:bg-[var(--hover)] hover:text-[color:var(--tx)]'"
          @click="newsFilter = src"
        >
          {{ src === "launcher" ? "Mono Launcher" : packNameFor(src) }}
        </button>
        <button
          type="button"
          class="rounded-full  px-3.5 py-1.5 text-[13px] font-medium transition-colors"
          :class="newsFilter === 'all'
            ? ' bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] text-white'
            : ' bg-[var(--input)] text-[color:var(--tx-muted)] hover:bg-[var(--hover)] hover:text-[color:var(--tx)]'"
          @click="newsFilter = 'all'"
        >
          {{ t("news.all") }}
        </button>
      </div>
    </div>

    <div v-if="news === null" class="flex flex-1 items-center justify-center text-[13px] text-[color:var(--tx-muted)]">
      <svg class="mr-2 h-4 w-4 animate-spin fill-[var(--accent)]" viewBox="0 0 16 16">
        <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
      </svg>
      {{ t("news.loading") }}
    </div>

    <div v-else-if="news.length === 0" class="shrink-0 rounded-xl  bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]">
      {{ t("news.none") }}
    </div>

    <div v-else-if="filteredNews.length === 0" class="shrink-0 rounded-xl  bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]">
      {{ t("news.emptyCat") }}
    </div>

    <div v-else class="min-h-0 flex-1 space-y-4 overflow-y-auto pr-1 pb-8">
      <article
        v-for="n in filteredNews"
        :key="`${n.kind}-${n.url || n.tag}`"
        class="rounded-xl  bg-[var(--panel)] shadow-sm transition-shadow hover:shadow-md"
      >
        <div class="flex items-start justify-between gap-3 border-b border-[var(--border)]  px-3.5 py-2.5">
          <div class="min-w-0">
            <div class="flex items-center gap-2 flex-wrap">
              <span
                class="rounded-full px-2 py-0.5 text-xs font-medium"
                :class="n.kind === 'update'
                  ? ' bg-[color-mix(in_srgb,var(--accent-deep)_10%,transparent)] text-[var(--accent)]'
                  : 'bg-[#9e6a03]/10 text-[#d29922]'"
              >
                {{ n.kind === "update" ? t("news.update") : t("news.post") }}
              </span>
              <span v-if="n.category" class="rounded-full  bg-[var(--bg)] px-2 py-0.5 text-xs font-medium text-[color:var(--tx-muted)]">
                {{ n.category }}
              </span>
              <span class="rounded-full  bg-[var(--bg)] px-2 py-0.5 text-xs font-medium text-[color:var(--tx-muted)]">
                {{ n.pack_name }}
              </span>
              <span v-if="n.kind === 'update' && n.tag" class="font-mono text-[13px] font-semibold text-[var(--accent)]">
                {{ n.tag }}
              </span>
            </div>
            <h2 class="mt-1.5 text-sm font-semibold text-[color:var(--tx-strong)] break-words">
              {{ n.title }}
            </h2>
          </div>
          <div class="flex shrink-0 flex-col items-end gap-1.5">
            <span class="text-[13px] text-[color:var(--tx-muted)]">
              {{ formatDate(n.date) }}
            </span>
            <div class="flex gap-1.5">
              <button
                v-if="n.kind === 'post' && n.url"
                type="button"
                class="rounded-md  bg-[var(--input)] px-2.5 py-1 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] hover:text-white"
                @click="openNewsLink(n.url)"
              >
                {{ t("news.open") }}
              </button>
              <button
                v-else-if="n.kind === 'update' && n.pack_id === 'launcher' && n.url"
                type="button"
                class="rounded-md  bg-[var(--input)] px-2.5 py-1 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] hover:text-white"
                @click="openNewsLink(n.url)"
              >
                {{ t("news.open") }}
              </button>
              <button
                v-if="n.kind === 'update' && n.pack_id !== 'launcher' && n.tag"
                type="button"
                class="rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] disabled:opacity-50"
                :disabled="busy"
                @click="installNews(n)"
              >
                {{ isInstalledVersion(n.tag) ? (isActiveNewsTag(n.tag) ? t("releases.selected") : t("releases.switch")) : t("releases.install") }}
              </button>
            </div>
          </div>
        </div>

        <!-- Тело: ченджлог/пост -->
        <div v-if="changelogLines(n.body).length > 0" class="p-4 text-[13px] text-[color:var(--tx)] space-y-1.5">
          <div class="changelog space-y-1 font-sans" @click="onChangelogLinkClick">
            <template v-for="(line, idx) in visibleNewsLines(n)" :key="idx">
              <div v-if="line.type === 'bullet'" class="flex items-start gap-2 text-[color:var(--tx)]">
                <span class="text-[color:var(--tx-muted)] select-none">•</span>
                <span v-html="renderInline(line.text)"></span>
              </div>
              <div v-else-if="line.type === 'body'" class="font-semibold text-[color:var(--tx-strong)] pt-1.5" v-html="renderInline(line.text)"></div>
              <div v-else class="text-[color:var(--tx-muted)]" v-html="renderInline(line.text)"></div>
            </template>
          </div>
          <button
            v-if="isNewsExpandable(n)"
            type="button"
            class="mt-2 inline-block text-[13px] font-medium text-[var(--accent)] hover:underline"
            @click="toggleNewsExpanded(n)"
          >
            {{ isNewsExpanded(n) ? t("news.collapse") : t("news.showAll") }}
          </button>
        </div>
        <div v-else class="p-4 text-[13px] text-[color:var(--tx-muted)] italic">
          {{ t("news.noText") }}
        </div>
      </article>
    </div>
  </div>
</template>
