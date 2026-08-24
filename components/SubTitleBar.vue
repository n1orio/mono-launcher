<template>
  <div class="flex h-9 shrink-0 items-center gap-2 border-b border-[var(--border)] bg-[var(--panel)] px-3">
    <button type="button" class="group flex h-3 w-3 items-center justify-center rounded-full bg-[var(--accent-deep)] transition-colors hover:bg-[var(--accent-hover)]" :title="t('common.close')" @click="close">
      <svg viewBox="0 0 16 16" class="h-2 w-2 opacity-0 transition-opacity fill-black/60 group-hover:opacity-100"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
    </button>
    <button type="button" class="group flex h-3 w-3 items-center justify-center rounded-full bg-[var(--accent)] transition-colors hover:bg-[var(--accent-strong)]" @click="minimize">
      <svg viewBox="0 0 16 16" class="h-2 w-2 opacity-0 transition-opacity fill-black/60 group-hover:opacity-100"><path d="M2 8a.75.75 0 0 1 .75-.75h10.5a.75.75 0 0 1 0 1.5H2.75A.75.75 0 0 1 2 8Z"/></svg>
    </button>
    <button type="button" class="group flex h-3 w-3 items-center justify-center rounded-full bg-[var(--accent-strong)] transition-colors hover:bg-[var(--accent)]" @click="toggleMaximize">
      <svg viewBox="0 0 16 16" class="h-2 w-2 opacity-0 transition-opacity fill-black/60 group-hover:opacity-100"><path d="M1.5 2.5A1.5 1.5 0 0 1 3 1h10a1.5 1.5 0 0 1 1.5 1.5v10a1.5 1.5 0 0 1-1.5 1.5H3A1.5 1.5 0 0 1 1.5 12.5v-10Zm1.5-.5a.5.5 0 0 0-.5.5v10a.5.5 0 0 0 .5.5h10a.5.5 0 0 0 .5-.5v-10a.5.5 0 0 0-.5-.5H3Z"/></svg>
    </button>
    <span v-if="title" class="ml-1 min-w-0 truncate text-xs font-semibold text-[color:var(--tx-muted)]">{{ title }}</span>
    <div data-tauri-drag-region class="h-full min-w-8 flex-1"></div>
  </div>
</template>

<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18n } from "~/composables/useI18n";

defineProps<{ title?: string }>();

const { t } = useI18n();

function close() {
  void getCurrentWindow().close();
}
function minimize() {
  void getCurrentWindow().minimize();
}
function toggleMaximize() {
  void getCurrentWindow().toggleMaximize();
}
</script>
