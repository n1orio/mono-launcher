<script setup lang="ts">
import { ref } from "vue";
import { packGradient } from "~/lib/misc";
import { useLauncherCtx } from '~/composables/useLauncherContext';
const ctx = useLauncherCtx();
const {
  t, monoProfile, tab, authorBusy, pickAuthorImportFile,
  authorDetail, closeAuthorDetail, authorVersions, openCatalogPackById, copyAuthorLink,
  authorTab, authorNews, authorCollaborators, catalogCommentCount,
  authorDelArm, armAuthorDelete, cancelAuthorDelete,
  saveAuthorOverview, resetAuthorForm, authorDirty,
  authorShots, setShotCaption, moveAuthorShot, removeAuthorShot,
  authorShotUrl, addAuthorShot, pickAuthorShotFile, authorShotCaption,
  formatDate, deleteAuthorVersion, pickAuthorVersionFile, authorVersionFile,
  authorNewVersion, authorNewChangelog, submitAuthorVersion,
  deleteAuthorNews, authorNewsKind, authorNewTitle, authorNewBody, addAuthorNews,
  collabBusy, updateCollaborator, removeCollaborator, authorSelected,
  collabName, collabPerms, addCollaborator, openProfileView,
  catalogCommentsBusy, catalogComments, removeCatalogComment,
  authorPacks, openAuthorDetail, authorOverviewBanner,
} = ctx;

// Свернутый ввод URL скриншота (основной путь — дропзона с выбором файла).
 const showShotUrl = ref(false);
 function toggleAuthlib() {
   const m = authorDetail.value?.meta as Record<string, unknown> | null;
   if (!m) { authorDetail.value!.meta = { use_authlib: true }; return; }
   m.use_authlib = m.use_authlib !== true;
 }
 </script>

<template>
  <div class="min-h-0 flex-1 overflow-y-auto pr-1">
  <div class="space-y-6">
  <div class="border-b border-[var(--border)]  pb-3 flex items-start justify-between gap-4">
  <div>
  <h1 class="text-xl font-bold tracking-tight text-[color:var(--tx-strong)]">{{ t("author.title") }}</h1>
  <p class="text-[13px] text-[color:var(--tx-muted)]">{{ t("author.subtitle") }}</p>
  </div>
  <button
  v-if="monoProfile"
  type="button"
  class="shrink-0 rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
  :disabled="authorBusy"
  @click="pickAuthorImportFile"
  >
  {{ t("author.create") }}
  </button>
  </div>

  <div v-if="!monoProfile" class="rounded-xl  bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]">
  <p>{{ t("author.needLogin") }}</p>
  <button type="button" class="mt-4 rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="tab = 'settings'">
  {{ t("nav.settings") }}
  </button>
  </div>

  <template v-else-if="authorDetail">
  <button type="button" class="flex items-center gap-1 rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="closeAuthorDetail()">
  <AppIcon name="chevron-right" class="h-3 w-3 fill-current -rotate-180" />
  {{ t("author.back") }}
  </button>

  <section class="rounded-2xl bg-[var(--input)]/30 border border-[var(--border)] p-4 mb-4 flex items-center justify-between gap-4 shadow-sm">
  <div class="flex items-center gap-3.5 min-w-0">
  <img v-if="authorDetail.icon_url" :src="authorDetail.icon_url" class="w-12 h-12 shrink-0 rounded-xl object-cover" />
  <div v-else class="w-12 h-12 shrink-0 rounded-xl flex items-center justify-center font-black text-xl text-white select-none" :style="{ background: packGradient(authorDetail.name || 'T') }">
  {{ authorDetail.name?.[0]?.toUpperCase() || 'T' }}
  </div>
  <div class="flex flex-col text-left min-w-0">
  <div class="flex items-center gap-2 flex-wrap">
  <span class="text-base font-bold text-[color:var(--tx)] truncate">{{ authorDetail.name }}</span>
  <span v-if="authorVersions.length > 0" class="text-xs px-2 py-0.5 rounded-md bg-white/5 border border-[var(--border)] text-[color:var(--tx-muted)]">v{{ authorVersions[0].version }}</span>
  </div>
  <div class="flex items-center gap-2 text-xs text-[color:var(--tx-muted)] mt-1 flex-wrap">
  <span>Автор: <strong class="text-[color:var(--tx)]">@{{ authorDetail.author_name ?? t("author.unknown") }}</strong></span>
  <span>•</span>
  <span class="flex items-center gap-1 text-[#22c55e] font-semibold"><span class="text-[10px]">▲</span>{{ authorDetail.likes }}</span>
  <span class="flex items-center gap-1 text-rose-400 font-semibold"><span class="text-[10px]">▼</span>{{ authorDetail.dislikes }}</span>
  </div>
  </div>
  </div>
  <div class="flex items-center gap-2 shrink-0">
  <button type="button" class="px-3.5 py-1.5 rounded-xl bg-[var(--input)] hover:bg-[var(--panel)] border border-[var(--border)] text-xs font-semibold text-[color:var(--tx)] transition-all" @click="openCatalogPackById(authorSelected!)">
  {{ t("author.openCatalog") }}
  </button>
  <button type="button" class="px-3.5 py-1.5 rounded-xl bg-[var(--input)] hover:bg-[var(--panel)] border border-[var(--border)] text-xs font-semibold text-[color:var(--tx)] transition-all" @click="copyAuthorLink(authorDetail.url)">
  {{ t("author.copyLink") }}
  </button>
  </div>
  </section>

  <!-- Сабтабы панели автора -->
  <div class="flex gap-1 overflow-x-auto border-b border-[var(--border)] ">
  <button v-for="st in (['overview', 'versions', 'news', 'collabs', 'comments'] as const)" :key="st" type="button"
  class="relative shrink-0 px-3.5 pb-2.5 pt-1 text-[13px] font-semibold transition-colors"
  :class="authorTab === st ? 'text-[var(--accent)]' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
  @click="authorTab = st">
  {{ st === 'overview' ? t('author.tabOverview') : st === 'versions' ? t('author.versions') : st === 'news' ? t('author.news') : st === 'collabs' ? t('collabs.tab') : t('comments.tab') }}
  <template v-if="st === 'versions' && authorVersions.length"> ({{ authorVersions.length }})</template>
  <template v-else-if="st === 'news' && authorNews.length"> ({{ authorNews.length }})</template>
  <template v-else-if="st === 'collabs' && authorCollaborators.length"> ({{ authorCollaborators.length }})</template>
  <template v-else-if="st === 'comments' && catalogCommentCount"> ({{ catalogCommentCount }})</template>
  <span v-if="authorTab === st" class="absolute inset-x-2 bottom-0 h-[2.5px] rounded-t-full bg-[var(--accent)]"></span>
  </button>
  </div>

  <div v-if="authorTab === 'overview'" class="space-y-6">
  <section class="rounded-2xl bg-[var(--input)]/20 border border-[var(--border)] p-4 shadow-sm">
  <div class="space-y-3">
  <label class="block text-xs font-semibold text-[color:var(--tx-muted)]">
  {{ t("author.name") }}
  <input v-model="authorDetail.name" type="text" class="mt-1.5 w-full rounded-xl bg-[var(--input)] border border-[var(--border)] px-3.5 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none focus:border-[var(--accent)] transition-all" />
  </label>
  <label class="block text-xs font-semibold text-[color:var(--tx-muted)]">
  {{ t("author.desc") }}
  <MdEditor v-model="authorDetail.description" :rows="12" :placeholder="t('author.descPh')" class="mt-1.5" />
  </label>
  <div class="grid grid-cols-2 gap-3">
  <label class="block text-xs font-semibold text-[color:var(--tx-muted)]">
  {{ t("author.minRamMb") }}
  <input v-model.number="authorDetail.min_ram_mb" type="number" min="0" class="mt-1.5 w-full rounded-xl bg-[var(--input)] border border-[var(--border)] px-3.5 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none focus:border-[var(--accent)] transition-all" />
  </label>
  <label class="block text-xs font-semibold text-[color:var(--tx-muted)]">
  {{ t("author.iconUrl") }}
  <input v-model="authorDetail.icon_url" type="text" class="mt-1.5 w-full rounded-xl bg-[var(--input)] border border-[var(--border)] px-3.5 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none focus:border-[var(--accent)] transition-all" />
  <img v-if="authorDetail.icon_url" :src="authorDetail.icon_url" class="mt-1.5 h-10 w-10 rounded-md object-cover" loading="lazy" @error="($event.target as HTMLImageElement).style.display = 'none'" />
  </label>
  <label class="col-span-2 block text-xs font-semibold text-[color:var(--tx-muted)]">
  {{ t("author.banner") }}
  <input v-model="authorOverviewBanner" type="text" class="mt-1.5 w-full rounded-xl bg-[var(--input)] border border-[var(--border)] px-3.5 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none focus:border-[var(--accent)] transition-all" />
  <img v-if="authorOverviewBanner.trim()" :src="authorOverviewBanner" class="mt-1.5 h-16 w-full rounded-md object-cover" loading="lazy" @error="($event.target as HTMLImageElement).style.display = 'none'" />
  </label>
   <label class="col-span-2 block text-xs font-semibold text-[color:var(--tx-muted)]">
   {{ t("author.boosty") }}
   <input v-model="authorDetail.boosty_blog" type="text" class="mt-1.5 w-full rounded-xl bg-[var(--input)] border border-[var(--border)] px-3.5 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none focus:border-[var(--accent)] transition-all" />
   </label>
   </div>
   <div class="flex items-center gap-3 pt-1">
   <label class="flex cursor-pointer items-center gap-2 text-xs text-[color:var(--tx-muted)]">
   <input type="checkbox" class="accent-[var(--accent)]" :checked="(authorDetail.meta as Record<string, unknown> | null)?.use_authlib === true" @change="toggleAuthlib" />
   {{ t("author.useAuthlib") }}
   </label>
   <button type="button" class="px-6 py-2.5 rounded-xl bg-[var(--accent-deep)] hover:brightness-110 text-white font-bold text-xs shadow-lg active:scale-95 transition-all disabled:opacity-50" :disabled="authorBusy || !authorDirty" @click="saveAuthorOverview">
   {{ t("author.save") }}
   </button>
  <button v-if="authorDirty" type="button" class="rounded-xl px-3 py-2 text-xs font-medium text-[color:var(--tx-muted)] hover:text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="resetAuthorForm">
  {{ t("author.reset") }}
  </button>
  <span v-if="authorDirty" class="text-xs text-[color:var(--tx-muted)]">{{ t("author.unsaved") }}</span>
  </div>
  </div>
  </section>

  <!-- Скриншоты: дропзона + список, URL — вторичная опция -->
  <section class="rounded-2xl bg-[var(--input)]/20 border border-[var(--border)] p-4 shadow-sm">
  <div class="mb-3 text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("pack.screenshots") }}</div>
  <div class="space-y-2">
  <div v-if="authorShots.length === 0" class="text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("author.noShots") }}</div>
  <div v-for="(s, i) in authorShots" :key="i" class="flex items-center gap-2 rounded-xl bg-[var(--panel)] border border-[var(--border)] p-2">
  <img :src="s.url" :alt="`Screenshot ${i + 1}`" class="h-10 w-16 shrink-0 rounded object-cover" loading="lazy" />
  <input
  :value="s.caption"
  type="text"
  :placeholder="t('author.shotCaption')"
  class="min-w-0 flex-1 rounded bg-transparent px-2 py-1 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none"
  @change="setShotCaption(i, ($event.target as HTMLInputElement).value)"
  />
  <button type="button" class="shrink-0 rounded-md bg-[var(--input)] px-1.5 py-1 text-[13px] text-[color:var(--tx-muted)] hover:bg-[var(--hover)] disabled:opacity-50"
  :disabled="authorBusy || i === 0"
  @click="moveAuthorShot(i, -1)">↑</button>
  <button type="button" class="shrink-0 rounded-md bg-[var(--input)] px-1.5 py-1 text-[13px] text-[color:var(--tx-muted)] hover:bg-[var(--hover)] disabled:opacity-50"
  :disabled="authorBusy || i === authorShots.length - 1"
  @click="moveAuthorShot(i, 1)">↓</button>
  <button type="button" class="shrink-0 rounded-md bg-[var(--input)] px-2 py-1 text-[13px] text-[#f87171] hover:bg-[#b91c1c]/20 disabled:opacity-50"
  :disabled="authorBusy"
  @click="removeAuthorShot(i)">
  {{ t("author.delete") }}
  </button>
  </div>
  <div
  role="button"
  tabindex="0"
  class="rounded-2xl border-2 border-dashed border-[var(--border)] hover:border-[var(--accent)]/50 p-6 flex flex-col items-center justify-center gap-2 text-center cursor-pointer transition-all"
  @click="pickAuthorShotFile"
  @keydown.enter="pickAuthorShotFile"
  >
  <AppIcon name="import" class="h-6 w-6 fill-current text-[color:var(--tx-muted)]" />
  <span class="text-xs text-[color:var(--tx-muted)]">{{ t("author.shotsDrop") }}</span>
  </div>
  <button type="button" class="text-xs text-[color:var(--tx-muted)] hover:text-[color:var(--tx)] hover:underline transition-all" @click="showShotUrl = !showShotUrl">
  {{ t("author.shotUrlToggle") }}
  </button>
  <div v-if="showShotUrl" class="space-y-2 border-t border-[var(--border)] pt-3">
  <div class="flex items-center gap-2">
  <input v-model="authorShotUrl" type="text" :placeholder="t('author.shotUrlPh')" class="min-w-0 flex-1 rounded-xl bg-[var(--input)] border border-[var(--border)] px-3.5 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none focus:border-[var(--accent)] transition-all" />
  <button type="button" class="shrink-0 rounded-xl bg-[var(--input)] hover:bg-[var(--panel)] border border-[var(--border)] px-3 py-2 text-xs font-semibold text-[color:var(--tx)] transition-all disabled:opacity-50"
  :disabled="authorBusy || !authorShotUrl.trim()"
  @click="addAuthorShot">
  {{ t("author.addShot") }}
  </button>
  </div>
  <input v-model="authorShotCaption" type="text" :placeholder="t('author.shotCaptionNew')" class="w-full rounded-xl bg-[var(--input)] border border-[var(--border)] px-3.5 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none focus:border-[var(--accent)] transition-all" />
  </div>
  </div>
  </section>

  <!-- Опасная зона: удаление сборки с сервера -->
  <section class="rounded-2xl bg-rose-500/[0.04] border border-rose-500/20 p-4 mt-6 flex items-center justify-between gap-4">
  <div class="min-w-0">
  <p class="text-sm font-bold text-rose-400">{{ t("author.deleteTitle") }}</p>
  <p class="mt-0.5 text-xs text-rose-300/70">{{ t("author.deleteWarn") }}</p>
  </div>
  <div class="flex items-center gap-2 shrink-0">
  <button type="button" class="px-4 py-2 rounded-xl bg-rose-500/15 hover:bg-rose-500 text-rose-300 hover:text-white border border-rose-500/30 text-xs font-bold transition-all active:scale-95 disabled:opacity-50" :disabled="authorBusy" @click="armAuthorDelete">
  {{ authorDelArm ? t("author.deleteSure") : t("author.deleteArm") }}
  </button>
  <button v-if="authorDelArm" type="button" class="px-4 py-2 rounded-xl bg-[var(--input)] hover:bg-[var(--panel)] border border-[var(--border)] text-xs font-semibold text-[color:var(--tx)] transition-all" @click="cancelAuthorDelete">
  {{ t("author.cancel") }}
  </button>
  </div>
  </section>
  </div>

  <div v-else-if="authorTab === 'versions'">
  <section class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <div class="border-b border-[var(--border)]  px-3.5 py-2.5 text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("author.versions") }}</div>
  <div class="divide-y divide-[var(--border)]">
  <div v-for="(v, vi) in authorVersions" :key="v.id" class="flex items-center gap-3 px-4 py-2.5">
  <div class="min-w-0 flex-1">
  <p class="flex items-center gap-1.5 truncate text-[13px] font-medium text-[color:var(--tx)]">
  {{ v.version }}
  <span v-if="vi === 0" class="rounded bg-[var(--accent)]/15 px-1.5 py-0.5 text-[11px] font-semibold text-[var(--accent)]">{{ t("author.latest") }}</span>
  </p>
  <p v-if="v.changelog" class="truncate text-[13px] text-[color:var(--tx-muted)]">{{ v.changelog }}</p>
  </div>
  <span class="shrink-0 text-[13px] text-[color:var(--tx-muted)]">{{ formatDate(v.created_at) }}</span>
  <button type="button" class="shrink-0 rounded-md  bg-[var(--input)] px-2 py-1 text-[13px] text-[#f87171] hover:bg-[#b91c1c]/20 disabled:opacity-50" :disabled="authorBusy" @click="deleteAuthorVersion(v.id)">
  {{ t("author.delete") }}
  </button>
  </div>
  </div>
  <div class="space-y-2 border-t border-[var(--border)]  p-4">
  <p class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("author.addVersion") }}</p>
  <div class="flex items-center gap-2">
  <button type="button" class="shrink-0 rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="pickAuthorVersionFile">
  {{ t("author.pickFile") }}
  </button>
  <span class="truncate text-[13px] text-[color:var(--tx-muted)]">{{ authorVersionFile || t("author.noFile") }}</span>
  </div>
  <input v-model="authorNewVersion" type="text" :placeholder="t('author.versionTag')" class="w-full rounded-md  bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)]  focus:outline-none" />
  <textarea v-model="authorNewChangelog" :placeholder="t('author.changelog')" rows="2" class="w-full resize-y rounded-md  bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)]  focus:outline-none"></textarea>
  <button type="button" class="rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50" :disabled="authorBusy || !authorVersionFile || !authorNewVersion.trim()" @click="submitAuthorVersion">
  {{ t("author.upload") }}
  </button>
  </div>
  </section>
  </div>

  <div v-else-if="authorTab === 'news'">
  <section class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <div class="border-b border-[var(--border)]  px-3.5 py-2.5 text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("author.news") }}</div>
  <div class="divide-y divide-[var(--border)]">
  <div v-for="n in authorNews" :key="n.id" class="px-4 py-2.5">
  <div class="flex items-center gap-2">
  <span class="rounded bg-[var(--input)] px-1.5 py-0.5 text-xs uppercase text-[color:var(--tx-muted)]">{{ n.kind }}</span>
  <p class="min-w-0 flex-1 truncate text-[13px] font-medium text-[color:var(--tx)]">{{ n.title }}</p>
  <span class="shrink-0 text-[13px] text-[color:var(--tx-muted)]">{{ formatDate(n.created_at) }}</span>
  <button type="button" class="shrink-0 rounded-md  bg-[var(--input)] px-2 py-1 text-[13px] text-[#f87171] hover:bg-[#b91c1c]/20" @click="deleteAuthorNews(n.id)">
  {{ t("author.delete") }}
  </button>
  </div>
  <p v-if="n.body" class="mt-1 line-clamp-2 text-[13px] text-[color:var(--tx-muted)]">{{ n.body }}</p>
  </div>
  <div v-if="authorNews.length === 0" class="px-4 py-6 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("author.noNews") }}</div>
  </div>
  <div class="space-y-2 border-t border-[var(--border)]  p-4">
  <p class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("author.addNews") }}</p>
  <div class="flex items-center gap-2">
  <select v-model="authorNewsKind" class="rounded-md  bg-[var(--bg)] px-2 py-1.5 text-[13px] text-[color:var(--tx)]  focus:outline-none">
  <option value="post">{{ t("author.post") }}</option>
  <option value="update">{{ t("author.update") }}</option>
  </select>
  <input v-model="authorNewTitle" type="text" :placeholder="t('author.titleField')" class="min-w-0 flex-1 rounded-md  bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)]  focus:outline-none" />
  </div>
  <MdEditor v-model="authorNewBody" :rows="4" :placeholder="t('author.body')" />
  <button type="button" class="rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50" :disabled="authorBusy || !authorNewTitle.trim()" @click="addAuthorNews(authorNewTitle.trim(), authorNewBody)">
  {{ t("author.addNews") }}
  </button>
  </div>
  </section>
  </div>

  <!-- Соавторы: гранулярные права на сборку -->
  <div v-else-if="authorTab === 'collabs'">
  <section class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <div class="border-b border-[var(--border)]  px-3.5 py-2.5 text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("collabs.tab") }}</div>
  <div class="divide-y divide-[var(--border)]">
  <div v-for="c in authorCollaborators" :key="c.id" class="flex flex-wrap items-center gap-2 px-4 py-2.5">
  <button type="button" class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full  bg-[var(--input)] font-mono text-[13px] font-bold text-[color:var(--tx-strong)] transition-colors hover:text-[var(--accent)]"
  @click="openProfileView(c.user.id)">
  {{ c.user.username?.[0]?.toUpperCase() ?? "?" }}
  </button>
  <span class="font-mono text-[13px] font-medium text-[color:var(--tx-strong)]">{{ c.user.displayName || c.user.username }}</span>
  <div class="ml-auto flex flex-wrap items-center gap-2">
  <label v-for="pm in (['permEditMeta', 'permManageVersions', 'permManageNews'] as const)" :key="pm" class="flex cursor-pointer items-center gap-1 text-xs text-[color:var(--tx-muted)]">
  <input type="checkbox" class="accent-[var(--accent)]" :checked="c[pm]" :disabled="collabBusy"
  @change="updateCollaborator(authorSelected!, c.id, { [pm]: !c[pm] } as any)" />
  {{ t('collabs.' + pm) }}
  </label>
  <button type="button" class="shrink-0 rounded-md  bg-[var(--input)] px-2 py-1 text-[13px] text-[#f87171] hover:bg-[#b91c1c]/20 disabled:opacity-50" :disabled="collabBusy" @click="removeCollaborator(authorSelected!, c.id)">
  {{ t("author.delete") }}
  </button>
  </div>
  </div>
  <div v-if="authorCollaborators.length === 0" class="px-4 py-6 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("collabs.empty") }}</div>
  </div>
  <div class="space-y-2 border-t border-[var(--border)]  p-4">
  <p class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("collabs.add") }}</p>
  <input v-model="collabName" type="text" :placeholder="t('collabs.usernamePh')" class="w-full rounded-md  bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)]  focus:outline-none" />
  <div class="flex flex-wrap items-center gap-3">
  <label v-for="pm in (['permEditMeta', 'permManageVersions', 'permManageNews'] as const)" :key="pm" class="flex cursor-pointer items-center gap-1 text-xs text-[color:var(--tx-muted)]">
  <input v-model="collabPerms[pm]" type="checkbox" class="accent-[var(--accent)]" />
  {{ t('collabs.' + pm) }}
  </label>
  <button type="button" class="ml-auto rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
  :disabled="collabBusy || !collabName.trim()"
  @click="addCollaborator(authorSelected!, collabName, collabPerms.permEditMeta, collabPerms.permManageVersions, collabPerms.permManageNews); collabName = ''">
  {{ t("collabs.addBtn") }}
  </button>
  </div>
  <p class="text-xs leading-snug text-[color:var(--tx-muted)]">{{ t("collabs.hint") }}</p>
  </div>
  </section>
  </div>

  <!-- Комментарии к сборке (модерация автора) -->
  <div v-else-if="authorTab === 'comments'">
  <section class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <div class="border-b border-[var(--border)]  px-3.5 py-2.5 text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("comments.tab") }}</div>
  <div class="divide-y divide-[var(--border)]">
  <div v-if="catalogCommentsBusy && catalogComments.length === 0" class="px-4 py-6 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("catalog.loading") }}</div>
  <div v-if="!catalogCommentsBusy && catalogComments.length === 0" class="px-4 py-6 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("comments.empty") }}</div>
  <div v-for="c in catalogComments" :key="c.id">
  <div class="flex items-start gap-2 px-4 py-2.5">
  <button type="button" class="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-full  bg-[var(--input)] font-mono text-xs font-bold text-[color:var(--tx-strong)] transition-colors hover:text-[var(--accent)]"
  @click="openProfileView(c.userId)">
  {{ c.user.username?.[0]?.toUpperCase() ?? "?" }}
  </button>
  <div class="min-w-0 flex-1">
  <div class="flex items-center gap-2">
  <span class="font-mono text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ c.user.displayName || c.user.username }}</span>
  <span class="text-xs text-[color:var(--tx-muted)]">{{ formatDate(c.createdAt) }}</span>
  <span class="text-xs text-[color:var(--tx-muted)]">👍 {{ c.likes }} · 👎 {{ c.dislikes }}</span>
  </div>
  <p class="mt-0.5 text-[13px] leading-relaxed text-[color:var(--tx)] whitespace-pre-wrap">{{ c.body }}</p>
  <div v-for="r in c.replies" :key="r.id" class="mt-1.5 ml-3 border-l-2 border-[var(--border)] pl-2.5">
  <div class="flex items-center gap-2">
  <span class="font-mono text-xs font-semibold text-[color:var(--tx)]">{{ r.user.displayName || r.user.username }}</span>
  <span class="text-[11px] text-[color:var(--tx-muted)]">{{ formatDate(r.createdAt) }} · 👍 {{ r.likes }} · 👎 {{ r.dislikes }}</span>
  </div>
  <p class="text-[13px] leading-relaxed text-[color:var(--tx-muted)] whitespace-pre-wrap">{{ r.body }}</p>
  </div>
  </div>
  <button type="button" class="shrink-0 rounded-md  bg-[var(--input)] px-2 py-1 text-xs text-[#f87171] hover:bg-[#b91c1c]/20 disabled:opacity-50"
  :disabled="catalogCommentsBusy"
  @click="removeCatalogComment(authorSelected!, c.id)">
  {{ t("author.delete") }}
  </button>
  </div>
  </div>
  </div>
  </section>
  </div>
  </template>

  <div v-else-if="authorPacks.length === 0" class="rounded-xl  bg-[var(--panel)] shadow-sm p-8 text-center text-[13px] text-[color:var(--tx-muted)]">
  {{ t("author.noPacks") }}
  </div>

  <div v-else class="grid grid-cols-1 gap-3 sm:grid-cols-2">
  <div v-for="p in authorPacks" :key="p.id" class="flex items-center gap-3 rounded-xl  bg-[var(--panel)] shadow-sm p-3">
  <img v-if="p.icon_url" :src="p.icon_url" class="h-10 w-10 shrink-0 rounded-md object-cover" />
  <div v-else class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-[var(--input)] text-sm font-semibold text-[var(--accent)]">
  {{ p.name?.[0]?.toUpperCase() }}
  </div>
  <div class="min-w-0 flex-1">
  <p class="truncate text-sm font-medium text-[color:var(--tx)]">{{ p.name }}</p>
  <p class="truncate text-[13px] text-[color:var(--tx-muted)]">
  {{ p.version }} · {{ t("author.rating") }}: {{ p.likes - p.dislikes }}
  </p>
  </div>
  <button type="button" class="shrink-0 rounded-md  bg-[var(--input)] px-2 py-2 text-[13px] text-[color:var(--tx-muted)] hover:bg-[var(--hover)] hover:text-[color:var(--tx)]" :title="t('author.copyLink')" @click="copyAuthorLink(p.url)">
  <AppIcon name="link" class="h-4 w-4 fill-current" />
  </button>
  <button type="button" class="shrink-0 rounded-md  bg-[var(--input)] px-2 py-2 text-[13px] text-[color:var(--tx-muted)] hover:bg-[var(--hover)] hover:text-[color:var(--tx)]" :title="t('author.openCatalog')" @click="openCatalogPackById(p.id)">
  <AppIcon name="eye_icon" class="h-4 w-4 fill-current" />
  </button>
  <button type="button" class="shrink-0 rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="openAuthorDetail(p.id)">
  {{ t("author.edit") }}
  </button>
  </div>
  </div>
  </div>
  </div>
</template>
