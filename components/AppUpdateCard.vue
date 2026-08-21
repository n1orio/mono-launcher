<template>
  <div>
    <!-- Карточка обновления лаунчера -->
    <div
      v-if="appUpdate && !appUpdating"
      class="fixed bottom-4 right-4 z-40 w-80 max-w-[calc(100vw-2rem)] rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[var(--panel)] p-3.5 shadow-lg shadow-black/40"
    >
      <div class="flex items-start gap-2.5">
        <svg viewBox="0 0 16 16" class="mt-0.5 h-4 w-4 shrink-0 fill-[var(--accent)]">
          <path d="M8 1.5a.75.75 0 0 1 .75.75V2.5H14a1 1 0 0 1 1 1v2.75A1.75 1.75 0 0 1 13.25 8H8.75v5.75a1.75 1.75 0 0 1-3.5 0V8H2A1.75 1.75 0 0 1 .25 6.25V3.5a1 1 0 0 1 1-1h5.25v-.25A.75.75 0 0 1 8 1.5Z"/>
        </svg>
        <div class="min-w-0 flex-1">
          <div class="text-xs font-semibold text-[color:var(--tx-strong)]">
            {{ t("appUpdate.title") }}
          </div>
          <div class="mt-0.5 truncate text-[11px] text-[color:var(--tx-muted)]">
            {{ t("appUpdate.version", { v: appUpdate.version }) }}
          </div>
          <p v-if="appUpdate.notes" class="mt-1 max-h-12 overflow-hidden text-[11px] leading-snug text-[color:var(--tx)]">
            {{ appUpdate.notes.slice(0, 180) }}{{ appUpdate.notes.length > 180 ? "…" : "" }}
          </p>
          <button
            type="button"
            class="mt-2.5 w-full rounded-md bg-[var(--accent-deep)] px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[var(--accent-hover)]"
            @click="installAppUpdate"
          >
            {{ t("appUpdate.install") }}
          </button>
        </div>
      </div>
    </div>
    <!-- Прогресс обновления лаунчера -->
    <div
      v-if="appUpdating"
      class="fixed bottom-4 right-4 z-40 w-80 max-w-[calc(100vw-2rem)] rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[var(--panel)] p-3.5 shadow-lg shadow-black/40"
    >
      <div class="mb-1.5 flex items-center justify-between text-[11px]">
        <span class="font-medium text-[color:var(--tx)]">{{ t("appUpdate.progress") }}</span>
        <span class="tabular-nums font-mono text-[10px] text-[color:var(--tx-muted)]">
          {{ appUpdateProgress ?? 0 }}%
        </span>
      </div>
      <div class="h-1.5 w-full overflow-hidden rounded-full bg-[var(--input)]">
        <div
          class="h-full bg-[#2f81f7] transition-all duration-200"
          :style="{ width: `${appUpdateProgress ?? 0}%` }"
        />
      </div>
      <div class="mt-1.5 text-[10px] text-[color:var(--tx-muted)]">
        {{ t("appUpdate.restart") }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useLauncherCtx } from "~/composables/useLauncherContext";
import { useI18n } from "~/composables/useI18n";

const { appUpdate, appUpdating, appUpdateProgress, installAppUpdate } = useLauncherCtx();
const { t } = useI18n();
</script>