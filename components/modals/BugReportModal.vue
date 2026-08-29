<script setup lang="ts">
import { useLauncherCtx } from "~/composables/useLauncherContext";
import { useI18n } from "#imports";

const { t } = useI18n();
const {
  bugReportOpen,
  closeBugReport,
  bugBody,
  bugLog,
  bugCopied,
  copyBugReport,
  openBugReportIssue,
} = useLauncherCtx();
</script>

<template>
  <div
    v-if="bugReportOpen"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6"
    @click.self="closeBugReport"
  >
    <div class="flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-xl  bg-[var(--panel)] shadow-2xl">
      <div class="flex shrink-0 items-center justify-between border-b border-[var(--border)]  px-3.5 py-2.5">
        <h3 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ t("reportPack.modalTitle") }}</h3>
        <button
          type="button"
          class="rounded-md px-2 py-1 text-[13px] text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)]"
          @click="closeBugReport"
        >
          ✕
        </button>
      </div>
      <pre class="min-h-0 flex-1 overflow-y-auto whitespace-pre-wrap break-words px-3.5 py-2.5 font-mono text-[13px] leading-relaxed text-[color:var(--tx)]">{{ bugBody }}</pre>
      <div class="flex shrink-0 items-center justify-end gap-2 border-t border-[var(--border)]  px-3.5 py-2.5">
        <span v-if="bugLog" class="mr-auto text-[13px] text-[color:var(--tx-muted)]">
          {{ t("reportPack.logNote", { n: bugLog.split("\n").slice(-60).length }) }}
        </span>
        <button
          type="button"
          class="rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
          @click="copyBugReport"
        >
          {{ bugCopied ? t("reportPack.copied") : t("reportPack.copy") }}
        </button>
        <button
          type="button"
          class="rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)]"
          @click="openBugReportIssue"
        >
          {{ t("reportPack.open") }}
        </button>
      </div>
    </div>
  </div>
</template>
