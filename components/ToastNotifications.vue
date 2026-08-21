<template>
  <div class="pointer-events-none fixed right-4 top-4 z-50 flex w-80 max-w-[calc(100vw-2rem)] flex-col gap-2">
    <TransitionGroup name="toast">
      <div
        v-for="n in notifications"
        :key="n.id"
        class="pointer-events-auto flex items-start gap-2.5 rounded-md border bg-[var(--panel)] px-3.5 py-2.5 text-xs shadow-lg shadow-black/40"
        :class="{
          'border-[#f85149]/50': n.type === 'error',
          'border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)]': n.type === 'info',
          'border-[#238636]/50': n.type === 'success',
        }"
      >
        <svg
          viewBox="0 0 16 16"
          class="mt-0.5 h-3.5 w-3.5 shrink-0 fill-current"
          :class="{
            'text-[#f85149]': n.type === 'error',
            'text-[var(--accent)]': n.type === 'info',
            'text-[#3fb950]': n.type === 'success',
          }"
        >
          <path v-if="n.type === 'error'" d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM4.97 4.97a.749.749 0 0 0-1.06 1.06L6.94 8l-3.03 3.03a.749.749 0 1 0 1.06 1.06L8 9.06l3.03 3.03a.749.749 0 1 0 1.06-1.06L9.06 8l3.03-3.03a.749.749 0 0 0-1.06-1.06L8 6.94Z"/>
          <path v-else-if="n.type === 'info'" d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM7.25 3.5a.75.75 0 0 0 0 1.5h.008a.75.75 0 0 0 0-1.5ZM7 7.25a.75.75 0 0 0 0 1.5h.25V12H7a.75.75 0 0 0 0 1.5h.75a.75.75 0 0 0 .75-.75v-5.5A.75.75 0 0 0 7.5 6.5H7a.75.75 0 0 0 0 .75Z"/>
          <path v-else d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0Zm3.03 5.03a.75.75 0 0 0-1.06-1.06L6.5 7.44l-1.47-1.47a.75.75 0 0 0-1.06 1.06l2 2a.75.75 0 0 0 1.06 0Z"/>
        </svg>
        <p class="min-w-0 break-words leading-relaxed text-[color:var(--tx)]">{{ n.text }}</p>
        <div class="ml-auto flex shrink-0 items-center gap-1.5">
          <button
            v-if="n.reportable"
            type="button"
            class="flex items-center gap-1 rounded border border-[#f85149]/40 bg-[#f85149]/10 px-2 py-0.5 text-[10px] font-semibold text-[#f85149] transition-colors hover:bg-[#f85149]/20"
            :title="t('toast.report')"
            @click="reportError(n.text)"
          >
            <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8Z"/>
            </svg>
            GitHub Issue
          </button>
          <button
            type="button"
            class="shrink-0 text-[color:var(--tx-muted)] transition-colors hover:text-[color:var(--tx-strong)]"
            :title="t('toast.close')"
            @click="dismissNotification(n.id)"
          >
            <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
              <path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.749.749 0 0 1 1.06 1.06L9.06 8l3.22 3.22a.749.749 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.749.749 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.749.749 0 0 1 0-1.06Z"/>
            </svg>
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