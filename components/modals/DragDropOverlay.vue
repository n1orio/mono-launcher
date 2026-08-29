<script setup lang="ts">
import { useLauncherCtx } from "~/composables/useLauncherContext";
import { useI18n } from "#imports";

const { t } = useI18n();
const { dragMrpack } = useLauncherCtx();
</script>

<template>
  <div
    v-if="dragMrpack"
    class="pointer-events-none fixed inset-0 z-[90] flex items-center justify-center bg-[color-mix(in_srgb,var(--bg)_75%,transparent)]"
  >
    <div class="flex flex-col items-center gap-3 rounded-xl border-2 border-dashed border-[var(--border)] bg-[var(--panel)] px-10 py-8 shadow-2xl">
      <svg viewBox="0 0 16 16" class="h-8 w-8 fill-current text-[var(--accent)]"><path d="M1 7.775V2.75C1 1.784 1.784 1 2.75 1h5.025c.464 0 .91.184 1.238.513l6.25 6.25a1.75 1.75 0 0 1 0 2.474l-5.026 5.026a1.75 1.75 0 0 1-2.474 0l-6.25-6.25A1.752 1.752 0 0 1 1 7.775Zm1.5 0c0 .066.026.13.073.177l6.25 6.25a.25.25 0 0 0 .354 0l5.025-5.025a.25.25 0 0 0 0-.354l-6.25-6.25a.25.25 0 0 0-.177-.073H2.75a.25.25 0 0 0-.25.25ZM6 5a1 1 0 1 0 0 2 1 1 0 0 0 0-2Z"/></svg>
      <span class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ t("library.dropMrpack") }}</span>
    </div>
  </div>
</template>
