<script setup lang="ts">
import { useLauncherCtx } from "~/composables/useLauncherContext";
import { useI18n } from "#imports";

const { t } = useI18n();
const {
  exportOpen,
  exportFormat,
  exportName,
  exportVersionNum,
  exportAllChecked,
  exportLoading,
  exportBusy,
  exportSelected,
  exportItems,
  exportVisibleRows,
  exportExpanded,
  exportUpload,
  authorImportMode,
  authorImportFile,
  authorName,
  authorAuthor,
  authorDesc,
  authorIcon,
  authorBanner,
  authorBoosty,
  authorMinRam,
  authorMinRamMb,
  authorServers,
  authorSocials,
  authorAccent,
  authorTheme,
  authorThemeFields,
  monoProfile,
  toggleExportAll,
  toggleExportExpand,
  toggleExport,
  exportSelectedCount,
  addAuthorServer,
  removeAuthorServer,
  addAuthorSocial,
  removeAuthorSocial,
  applyAuthorAccent,
  applyAuthorAccentColor,
  doExport,
  doAuthorExport,
  doAuthorImport,
  themePreview,
  AUTHOR_MAX_SERVERS,
  AUTHOR_MAX_SOCIALS,
  formatBytes,
} = useLauncherCtx();
</script>

<template>
  <div v-if="exportOpen" class="fixed inset-0 z-[60] flex items-center justify-center bg-black/40 p-4" @click.self="exportOpen = false">
    <div class="flex max-h-[80vh] w-full max-w-xl flex-col overflow-hidden rounded-xl bg-[var(--panel)] shadow-2xl">
      <div class="flex items-center justify-between gap-2 border-b border-[var(--border)] px-3.5 py-2.5">
        <h3 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ authorImportMode ? t("author.importTitle") : exportFormat === "author" ? t("pack.exportAuthorTitle") : t("pack.exportTitle") }}</h3>
        <button
          type="button"
          class="rounded-md p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
          @click="exportOpen = false"
        >
          <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
        </button>
      </div>
      <div class="flex items-center justify-between gap-2 border-b border-[var(--border)] px-4 py-2">
        <p class="text-[13px] text-[color:var(--tx-muted)]">
          {{ authorImportMode ? t("author.importHint") : exportFormat === "curseforge" ? t("pack.exportFormatCurseforge") : exportFormat === "author" ? t("pack.exportAuthorHint") : t("pack.exportFormatMrpack") }}
        </p>
        <button
          v-if="exportFormat !== 'author'"
          type="button"
          class="text-[13px] font-medium text-[var(--accent)] transition-colors hover:underline disabled:opacity-50"
          :disabled="exportLoading"
          @click="toggleExportAll"
        >
          {{ exportAllChecked ? t("pack.exportNone") : t("pack.exportAll") }}
        </button>
      </div>
      <div v-if="exportFormat !== 'author'" class="grid grid-cols-2 gap-2 border-b border-[var(--border)] px-4 py-2">
        <label class="block">
          <span class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportNameLabel") }}</span>
          <input
            v-model="exportName"
            class="w-full rounded-md bg-[var(--input)] px-2 py-1.5 text-[13px] text-[color:var(--tx)] outline-none transition-colors"
            :placeholder="t('pack.exportNamePlaceholder')"
          />
        </label>
        <label class="block">
          <span class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportVersionNumLabel") }}</span>
          <input
            v-model="exportVersionNum"
            class="w-full rounded-md bg-[var(--input)] px-2 py-1.5 text-[13px] text-[color:var(--tx)] outline-none transition-colors"
            :placeholder="t('pack.exportVersionNumPlaceholder')"
          />
        </label>
      </div>
      <div class="min-h-0 flex-1 overflow-y-auto px-2 py-1">
        <template v-if="exportFormat === 'author'">
          <div class="space-y-3 px-2 py-2">
            <div v-if="authorImportMode" class="flex items-center gap-2 rounded-md bg-[var(--bg-30)] px-2 py-1.5">
              <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-[var(--tx-muted)]"><path d="M9 1H4.5A1.5 1.5 0 0 0 3 2.5v11A1.5 1.5 0 0 0 4.5 15h7A1.5 1.5 0 0 0 13 13.5V5l-4-4Z"/></svg>
              <span class="min-w-0 flex-1 truncate font-mono text-[13px] text-[color:var(--tx)]">{{ authorImportFile || t("author.noFile") }}</span>
            </div>
            <div class="grid grid-cols-2 gap-2">
              <label :class="authorImportMode ? 'col-span-2' : ''" class="block">
                <span class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorName") }}</span>
                <input v-model="authorName" class="w-full rounded-md bg-[var(--input)] px-2 py-1.5 text-[13px] text-[color:var(--tx)] outline-none transition-colors" />
              </label>
              <label v-if="!authorImportMode" class="block">
                <span class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorAuthor") }}</span>
                <input v-model="authorAuthor" class="w-full rounded-md bg-[var(--input)] px-2 py-1.5 text-[13px] text-[color:var(--tx)] outline-none transition-colors" :placeholder="t('pack.exportAuthorAuthorPh')" />
              </label>
            </div>
            <label class="block">
              <span class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorDesc") }}</span>
              <textarea v-model="authorDesc" rows="2" class="w-full resize-none rounded-md bg-[var(--input)] px-2 py-1.5 text-[13px] text-[color:var(--tx)] outline-none transition-colors" :placeholder="t('pack.exportAuthorDescPh')"></textarea>
            </label>
            <div v-if="authorImportMode" class="grid grid-cols-2 gap-2">
              <label class="block">
                <span class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorIcon") }}</span>
                <input v-model="authorIcon" type="text" class="w-full rounded-md bg-[var(--input)] px-2 py-1.5 text-[13px] text-[color:var(--tx)] outline-none transition-colors" :placeholder="t('pack.exportAuthorIconPh')" />
              </label>
              <label class="block">
                <span class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorBanner") }}</span>
                <input v-model="authorBanner" type="text" class="w-full rounded-md bg-[var(--input)] px-2 py-1.5 text-[13px] text-[color:var(--tx)] outline-none transition-colors" :placeholder="t('pack.exportAuthorBannerPh')" />
              </label>
            </div>
            <div class="grid grid-cols-2 gap-2">
              <label class="block">
                <span class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorBoosty") }}</span>
                <input v-model="authorBoosty" class="w-full rounded-md bg-[var(--input)] px-2 py-1.5 text-[13px] text-[color:var(--tx)] outline-none transition-colors" :placeholder="t('pack.exportAuthorBoostyPh')" />
              </label>
              <label class="flex items-end gap-2 pb-1">
                <span class="w-full">
                  <span class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorMinRam") }}</span>
                  <span class="flex items-center gap-2">
                    <input v-model="authorMinRam" type="checkbox" class="h-4 w-4 accent-[var(--accent)]" />
                    <input v-if="authorMinRam" v-model.number="authorMinRamMb" type="number" min="1" class="w-20 rounded-md bg-[var(--input)] px-2 py-1.5 text-[13px] text-[color:var(--tx)] outline-none" />
                    <span v-else class="text-xs text-[color:var(--tx-muted)]">—</span>
                  </span>
                </span>
              </label>
            </div>

            <div class="rounded-lg bg-[var(--bg-30)] p-2">
              <div class="mb-1.5 flex items-center justify-between px-1">
                <span class="flex items-center gap-1.5 text-[13px] font-medium text-[color:var(--tx-muted)]">
                  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3 1.5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-9a2 2 0 0 0-2-2ZM1.5 4.5H14.5v1.5H1.5ZM1.5 8H14.5v1.25H1.5Zm0 3.25H7v1.5H1.5A.5.5 0 0 1 1 12.25v-1ZM8.5 12.75v-1.5h6v1.5A.5.5 0 0 1 14.5 13h-5a1 1 0 0 1-1-.25ZM2 5.75a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Zm3 0a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0ZM2 9.25a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Zm3 0a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Z"/></svg>
                  {{ t("pack.exportAuthorServers") }}
                </span>
                <div class="flex items-center gap-2">
                  <span class="text-xs tabular-nums text-[color:var(--tx-muted)]">{{ authorServers.length }}/{{ AUTHOR_MAX_SERVERS }}</span>
                  <button type="button" class="flex h-5 w-5 items-center justify-center rounded-md text-[13px] leading-none text-[var(--accent)] transition-colors hover:bg-[var(--input-50)] disabled:opacity-30 disabled:hover:bg-transparent" :disabled="authorServers.length >= AUTHOR_MAX_SERVERS" @click="addAuthorServer" title="+">+</button>
                </div>
              </div>
              <div v-for="(_, i) in authorServers" :key="i" class="mb-1 flex items-center gap-1 rounded-md bg-[var(--panel)] p-1 last:mb-0">
                <input v-model="authorServers[i].name" class="min-w-0 flex-1 rounded-md bg-[var(--input)] px-2 py-1.5 text-[13px] text-[color:var(--tx)] outline-none transition-colors" :placeholder="t('pack.exportServerNamePh')" />
                <input v-model="authorServers[i].ip" class="w-[7rem] rounded-md bg-[var(--input)] px-2 py-1.5 font-mono text-[13px] text-[color:var(--tx)] outline-none transition-colors" :placeholder="t('pack.exportServerIpPh')" />
                <div class="relative">
                  <input v-model.number="authorServers[i].port" type="number" class="w-16 rounded-md bg-[var(--input)] px-1.5 py-1.5 font-mono text-[13px] text-[color:var(--tx)] outline-none transition-colors" :placeholder="t('pack.exportServerPortPh')" />
                </div>
                <button type="button" class="flex h-6 w-6 shrink-0 items-center justify-center rounded-md p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--input-50)] hover:text-[#f85149]" :title="t('files.remove')" @click="removeAuthorServer(i)">
                  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
                </button>
              </div>
            </div>

            <div class="rounded-lg bg-[var(--bg-30)] p-2">
              <div class="mb-1.5 flex items-center justify-between px-1">
                <span class="flex items-center gap-1.5 text-[13px] font-medium text-[color:var(--tx-muted)]">
                  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M8 1a3 3 0 1 0 0 6 3 3 0 0 0 0-6Zm.75 7h-1.5A5.25 5.25 0 0 0 2 13.25c0 .414.336.75.75.75h10.5a.75.75 0 0 0 .75-.75A5.25 5.25 0 0 0 8.75 8Z"/></svg>
                  {{ t("pack.exportAuthorSocials") }}
                </span>
                <div class="flex items-center gap-2">
                  <span class="text-xs tabular-nums text-[color:var(--tx-muted)]">{{ authorSocials.length }}/{{ AUTHOR_MAX_SOCIALS }}</span>
                  <button type="button" class="flex h-5 w-5 items-center justify-center rounded-md text-[13px] leading-none text-[var(--accent)] transition-colors hover:bg-[var(--input-50)] disabled:opacity-30 disabled:hover:bg-transparent" :disabled="authorSocials.length >= AUTHOR_MAX_SOCIALS" @click="addAuthorSocial" title="+">+</button>
                </div>
              </div>
              <div v-for="(_, i) in authorSocials" :key="i" class="mb-1 flex items-center gap-1 rounded-md bg-[var(--panel)] p-1 last:mb-0">
                <input v-model="authorSocials[i].name" class="min-w-0 flex-1 rounded-md bg-[var(--input)] px-2 py-1.5 text-[13px] text-[color:var(--tx)] outline-none transition-colors" :placeholder="t('pack.exportSocialNamePh')" />
                <input v-model="authorSocials[i].url" class="w-[9rem] rounded-md bg-[var(--input)] px-2 py-1.5 font-mono text-[13px] text-[color:var(--tx)] outline-none transition-colors" :placeholder="t('pack.exportSocialUrlPh')" />
                <div class="relative shrink-0">
                  <input v-model="authorSocials[i].color" class="w-16 rounded-md bg-[var(--input)] pl-6 py-1.5 font-mono text-[13px] text-[color:var(--tx)] outline-none transition-colors" :placeholder="t('pack.exportSocialColorPh')" />
                  <span class="pointer-events-none absolute left-1.5 top-1/2 h-3 w-3 -translate-y-1/2 rounded-full" :style="{ background: themePreview(authorSocials[i].color) }"></span>
                </div>
                <button type="button" class="flex h-6 w-6 shrink-0 items-center justify-center rounded-md p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--input-50)] hover:text-[#f85149]" :title="t('files.remove')" @click="removeAuthorSocial(i)">
                  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
                </button>
              </div>
            </div>

            <div class="rounded-lg bg-[var(--bg-30)] p-2">
              <div class="mb-1.5 flex items-center justify-between px-1">
                <span class="flex items-center gap-1.5 text-[13px] font-medium text-[color:var(--tx-muted)]">
                  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M8 1a3 3 0 1 1 0 6 3 3 0 0 1 0-6Zm.75 7h-1.5A5.25 5.25 0 0 0 2 13.25c0 .414.336.75.75.75h10.5a.75.75 0 0 0 .75-.75A5.25 5.25 0 0 0 8.75 8ZM8 15a7 7 0 1 1 7-7 7 7 0 0 1-7 7Zm0-1.5a5.5 5.5 0 1 0 0-11 5.5 5.5 0 0 0 0 11Z"/></svg>
                  {{ t("pack.exportAuthorTheme") }}
                </span>
                <span class="text-xs text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorThemeAuto") }}</span>
              </div>
              <div class="mb-2 flex items-center gap-1.5 rounded-md bg-[var(--panel)] p-1.5">
                <div class="relative shrink-0">
                  <input v-model="authorAccent" class="w-24 rounded-md bg-[var(--input)] pl-6 py-1.5 font-mono text-[13px] text-[color:var(--tx)] outline-none transition-colors" placeholder="#rrggbb" @input="applyAuthorAccent" @change="applyAuthorAccent" />
                  <label class="absolute left-1.5 top-1/2 block h-3.5 w-3.5 -translate-y-1/2 cursor-pointer overflow-hidden rounded-full" :style="{ background: themePreview(authorAccent) }" :title="t('pack.exportAuthorAccentPicker')">
                    <input type="color" class="pointer-events-none absolute -left-2 -top-2 h-8 w-8 opacity-0" :value="themePreview(authorAccent) === '#000000' ? '#000000' : authorAccent" @input="applyAuthorAccentColor" />
                  </label>
                </div>
                <span class="min-w-0 flex-1 text-xs leading-tight text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorAccentHint") }}</span>
              </div>
              <div class="grid grid-cols-2 gap-1.5">
                <label v-for="f in authorThemeFields" :key="f.key" class="rounded-md bg-[var(--panel)] px-1.5 py-1">
                  <div class="flex items-center gap-1.5">
                    <span class="pointer-events-none h-3.5 w-3.5 shrink-0 rounded-full" :style="{ background: themePreview(authorTheme[f.key]) }"></span>
                    <input v-model="authorTheme[f.key]" class="min-w-0 flex-1 rounded-md bg-transparent py-0.5 font-mono text-xs text-[color:var(--tx)] outline-none transition-colors" placeholder="#rrggbb" />
                  </div>
                  <span class="mt-0.5 block pl-[1.375rem] text-[11px] leading-tight text-[color:var(--tx-muted)]">{{ t(f.cap) }}</span>
                </label>
              </div>
            </div>
          </div>
          <div v-if="!authorImportMode" class="flex items-center gap-2 border-t border-[var(--border)] px-2 pt-2 pb-1">
            <span class="shrink-0 text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorFiles") }}</span>
            <span class="shrink-0 rounded bg-[var(--input-50)] px-1.5 py-0.5 text-xs font-bold tabular-nums">{{ exportSelected.size }}</span>
            <span class="h-px flex-1 bg-[var(--border)]"></span>
            <button type="button" class="text-[13px] font-medium text-[var(--accent)] hover:underline disabled:opacity-50" :disabled="exportLoading" @click="toggleExportAll">
              {{ exportAllChecked ? t("pack.exportNone") : t("pack.exportAll") }}
            </button>
          </div>
        </template>
        <div v-if="!authorImportMode && exportLoading" class="flex items-center justify-center gap-2 py-8 text-[13px] text-[color:var(--tx-muted)]">
          <svg viewBox="0 0 16 16" class="h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
          {{ t("pack.exportLoading") }}
        </div>
        <div v-else-if="!authorImportMode && exportItems.length === 0" class="px-2 py-8 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("pack.exportEmpty") }}</div>
        <div v-else-if="!authorImportMode">
          <div
            v-for="row in exportVisibleRows"
            :key="row.it.path"
            class="flex cursor-pointer items-center gap-1 rounded-md py-1 pr-2 transition-colors hover:bg-[var(--hover)]"
            :style="{ paddingLeft: `${row.depth * 16 + 4}px` }"
          >
            <button
              v-if="row.it.isDir"
              type="button"
              class="flex h-4 w-4 shrink-0 items-center justify-center rounded text-[color:var(--tx-muted)] transition-colors hover:text-[color:var(--tx)]"
              @click.stop="toggleExportExpand(row.it.path)"
            >
              <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current transition-transform" :class="exportExpanded.has(row.it.path) ? 'rotate-90' : ''"><path d="M6 4l4 4-4 4V4Z"/></svg>
            </button>
            <span v-else class="w-4 shrink-0"></span>
            <input
              type="checkbox"
              class="h-3.5 w-3.5 shrink-0 accent-[var(--accent)]"
              :checked="exportSelected.has(row.it.path)"
              :indeterminate="exportSelectedCount(row.it.path).selected > 0 && exportSelectedCount(row.it.path).selected < exportSelectedCount(row.it.path).total"
              @change="toggleExport(row.it.path)"
            />
            <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-[var(--tx-muted)]">
              <path v-if="row.it.isDir" d="M1.75 1A1.75 1.75 0 0 0 0 2.75v10.5C0 14.216.784 15 1.75 15h12.5A1.75 1.75 0 0 0 16 13.25v-8.5A1.75 1.75 0 0 0 14.25 3H7.5a.25.25 0 0 1-.2-.1l-.9-1.2A1.75 1.75 0 0 0 5 1H1.75Z"/>
              <path v-else d="M9 1H4.5A1.5 1.5 0 0 0 3 2.5v11A1.5 1.5 0 0 0 4.5 15h7A1.5 1.5 0 0 0 13 13.5V5l-4-4Z"/>
            </svg>
            <span class="min-w-0 flex-1 truncate text-[13px] text-[color:var(--tx)]" :class="!row.it.defaultIncluded ? 'opacity-60' : ''">{{ row.it.path.split("/").pop() }}</span>
            <span v-if="row.it.defaultIncluded" class="shrink-0 text-xs tabular-nums text-[color:var(--tx-muted)]">{{ formatBytes(row.it.size) }}</span>
            <span v-else class="shrink-0 rounded px-1.5 py-0.5 text-[11px] uppercase text-[color:var(--tx-muted)]">{{ t("pack.exportExcluded") }}</span>
          </div>
        </div>
      </div>
      <div class="flex items-center justify-end gap-2 border-t border-[var(--border)] px-3.5 py-2.5">
        <label
          v-if="monoProfile && exportFormat === 'mrpack'"
          class="mr-auto flex cursor-pointer items-center gap-1.5 text-[13px] text-[color:var(--tx-muted)]"
        >
          <input type="checkbox" v-model="exportUpload" class="h-3.5 w-3.5 accent-[var(--accent)]" />
          {{ t("pack.uploadMono") }}
        </label>
        <button
          type="button"
          class="rounded-md bg-[var(--input)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)]"
          @click="exportOpen = false"
        >
          {{ t("files.cancel") }}
        </button>
        <button
          type="button"
          class="flex items-center gap-1.5 rounded-md bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_22%,transparent)] disabled:opacity-50"
          :disabled="exportBusy || exportLoading || (!authorImportMode && exportSelected.size === 0)"
          @click="authorImportMode ? doAuthorImport() : exportFormat === 'author' ? doAuthorExport() : doExport()"
        >
          <svg v-if="exportBusy" viewBox="0 0 16 16" class="h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
          {{ authorImportMode ? t("author.upload") : exportFormat === "author" ? t("pack.exportAuthorBtn") : t("pack.exportBtn") }}
        </button>
      </div>
    </div>
  </div>
</template>
