<template>
  <div class="flex h-10 shrink-0 items-center gap-2 px-4">
    <button type="button" class="group flex h-3 w-3 items-center justify-center rounded-full bg-[var(--accent-deep)] transition-colors hover:bg-[var(--accent-deep)]/80" @click="appClose">
      <svg viewBox="0 0 16 16" class="h-2 w-2 opacity-0 transition-opacity fill-black/60 group-hover:opacity-100"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
    </button>
    <button type="button" class="group flex h-3 w-3 items-center justify-center rounded-full bg-[var(--accent)] transition-colors hover:bg-[var(--accent)]/80" @click="appMinimize">
      <svg viewBox="0 0 16 16" class="h-2 w-2 opacity-0 transition-opacity fill-black/60 group-hover:opacity-100"><path d="M2 8a.75.75 0 0 1 .75-.75h10.5a.75.75 0 0 1 0 1.5H2.75A.75.75 0 0 1 2 8Z"/></svg>
    </button>
    <button type="button" class="group flex h-3 w-3 items-center justify-center rounded-full bg-[var(--accent-strong)] transition-colors hover:bg-[var(--accent-strong)]/80" @click="appToggleMaximize">
      <svg viewBox="0 0 16 16" class="h-2 w-2 opacity-0 transition-opacity fill-black/60 group-hover:opacity-100"><path d="M1.5 2.5A1.5 1.5 0 0 1 3 1h10a1.5 1.5 0 0 1 1.5 1.5v10a1.5 1.5 0 0 1-1.5 1.5H3A1.5 1.5 0 0 1 1.5 12.5v-10Zm1.5-.5a.5.5 0 0 0-.5.5v10a.5.5 0 0 0 .5.5h10a.5.5 0 0 0 .5-.5v-10a.5.5 0 0 0-.5-.5H3Z"/></svg>
    </button>
    <div data-tauri-drag-region class="h-full flex-1"></div>
  </div>
</template>

<script setup lang="ts">
import { useLauncherCtx } from "~/composables/useLauncherContext";

const { appClose, appMinimize, appToggleMaximize } = useLauncherCtx();
</script>