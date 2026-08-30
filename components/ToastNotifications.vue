<template>
  <div class="pointer-events-none fixed right-4 top-4 z-50 flex w-80 max-w-[calc(100vw-2rem)] flex-col gap-2">
  <TransitionGroup name="toast">
  <div
  v-for="n in notifications"
  :key="n.id"
  class="pointer-events-auto flex items-start gap-2.5 rounded-md bg-[var(--panel)] px-3.5 py-2.5 text-[13px] shadow-lg shadow-black/40"
>
  <AppIcon v-if="n.type === 'error'" name="alert-circle" class="mt-0.5 h-4 w-4 shrink-0 fill-current text-[#f85149]" />
  <AppIcon v-else-if="n.type === 'info'" name="alert-info" class="mt-0.5 h-4 w-4 shrink-0 fill-current text-[var(--accent)]" />
  <AppIcon v-else name="alert-success" class="mt-0.5 h-4 w-4 shrink-0 fill-current text-[#3fb950]" />
  <p class="min-w-0 break-words leading-relaxed text-[color:var(--tx)]">{{ n.text }}</p>
  <div class="ml-auto flex shrink-0 items-center gap-1.5">
  <button
  v-if="n.reportable"
  type="button"
  class="flex items-center gap-1 rounded  bg-[#f85149]/10 px-2 py-0.5 text-xs font-semibold text-[#f85149] transition-colors hover:bg-[#f85149]/20"
  :title="t('toast.report')"
  @click="reportError(n.text)"
  >
  <AppIcon name="github" class="h-3 w-3 fill-current" />
  GitHub Issue
  </button>
  <button
  type="button"
  class="shrink-0 text-[color:var(--tx-muted)] transition-colors hover:text-[color:var(--tx-strong)]"
  :title="t('toast.close')"
  @click="dismissNotification(n.id)"
  >
  <AppIcon name="x" class="h-3 w-3 fill-current" />
  </button>
  </div>
  </div>
  </TransitionGroup>
  </div>
</template>

<script setup lang="ts">
import { useLauncherCtx } from "~/composables/useLauncherContext";
import { useI18n } from "~/composables/useI18n";

const { notifications, dismissNotification, reportError } = useLauncherCtx();
const { t } = useI18n();
</script>