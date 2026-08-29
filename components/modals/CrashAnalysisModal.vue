<script setup lang="ts">
import { useLauncherCtx } from "~/composables/useLauncherContext";
import { useI18n } from "#imports";

const { t } = useI18n();
const {
  crashAnalysis,
  closeCrashAnalysis,
  crashView,
  copyCrashAnalysis,
  openCrashIssue,
} = useLauncherCtx();
</script>

<template>
  <div
    v-if="crashAnalysis"
    class="fixed inset-0 z-[55] flex items-center justify-center bg-black/50 p-6"
    @click.self="closeCrashAnalysis"
  >
    <div class="flex max-h-[80vh] w-full max-w-lg flex-col overflow-hidden rounded-xl  bg-[var(--panel)] shadow-2xl">
      <div class="flex shrink-0 items-start justify-between border-b border-[var(--border)]  px-3.5 py-2.5">
        <div class="flex items-center gap-2">
          <svg viewBox="0 0 16 16" class="h-4 w-4 fill-[var(--accent-deep)]"><path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1Zm.75 8.5a.75.75 0 0 1-1.5 0V5.25a.75.75 0 0 1 1.5 0Zm-0.75 2.25a.9.9 0 1 1 0-1.8.9.9 0 0 1 0 1.8Z"/></svg>
          <h3 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ t("crash.title") }}</h3>
        </div>
        <button
          type="button"
          class="rounded-md px-2 py-1 text-[13px] text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)]"
          @click="closeCrashAnalysis"
        >
          ✕
        </button>
      </div>
      <div class="min-h-0 flex-1 space-y-3 overflow-y-auto px-3.5 py-2.5">
        <div>
          <div class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ crashView(crashAnalysis).title }}</div>
          <p class="mt-0.5 text-[13px] leading-relaxed text-[color:var(--tx)]">{{ crashView(crashAnalysis).msg }}</p>
        </div>
        <div v-if="crashAnalysis.exception" class="rounded-md  bg-[var(--input-50)] px-3 py-2">
          <div class="mb-0.5 text-xs font-medium uppercase tracking-wide text-[color:var(--tx-muted)]">{{ t("crash.exception") }}</div>
          <code class="break-words font-mono text-[13px] text-[color:var(--tx)]">{{ crashAnalysis.exception }}</code>
        </div>
        <div v-if="crashAnalysis.description" class="rounded-md  bg-[var(--input-50)] px-3 py-2">
          <div class="mb-0.5 text-xs font-medium uppercase tracking-wide text-[color:var(--tx-muted)]">{{ t("crash.description") }}</div>
          <div class="break-words text-[13px] text-[color:var(--tx)]">{{ crashAnalysis.description }}</div>
        </div>
        <div v-if="crashAnalysis.suspected.length" class="rounded-md  bg-[var(--input-50)] px-3 py-2">
          <div class="mb-1 text-xs font-medium uppercase tracking-wide text-[color:var(--tx-muted)]">{{ t("crash.suspected") }}</div>
          <ul class="space-y-1">
            <li v-for="m in crashAnalysis.suspected" :key="m.file" class="flex items-center justify-between gap-2 text-[13px]">
              <span class="min-w-0 truncate text-[color:var(--tx)]">{{ m.name }}</span>
              <code class="shrink-0 font-mono text-xs text-[var(--tx-muted)]">{{ m.file }}</code>
            </li>
          </ul>
        </div>
        <div class="rounded-md  bg-[var(--input-50)] px-3 py-2">
          <div class="text-xs uppercase tracking-wide text-[color:var(--tx-muted)]">{{ t("crash.file") }}</div>
          <code class="font-mono text-[13px] text-[color:var(--tx)]">{{ crashAnalysis.file }}</code>
        </div>
      </div>
      <div class="flex shrink-0 items-center justify-between gap-2 border-t border-[var(--border)]  px-3.5 py-2.5">
        <button
          type="button"
          class="rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
          @click="copyCrashAnalysis"
        >
          {{ t("reportPack.copy") }}
        </button>
        <button
          type="button"
          class="rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)]"
          @click="openCrashIssue"
        >
          {{ t("crash.report") }}
        </button>
      </div>
    </div>
  </div>
</template>
