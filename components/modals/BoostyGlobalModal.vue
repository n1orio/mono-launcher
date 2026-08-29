<script setup lang="ts">
import { useLauncherCtx } from "~/composables/useLauncherContext";
import { useI18n } from "#imports";

const { t } = useI18n();
const {
  boostyGlobalOpen,
  boostyBusy,
  boostyError,
  boostyToken,
  boostySecret,
  boostyKey,
  doBoostyAuth,
} = useLauncherCtx();
</script>

<template>
  <div
    v-if="boostyGlobalOpen"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6"
    @click.self="boostyGlobalOpen = false"
  >
    <div class="flex w-full max-w-sm flex-col overflow-hidden rounded-xl  bg-[var(--panel)] shadow-2xl">
      <div class="flex items-center justify-between border-b border-[var(--border)]  px-3.5 py-2.5">
        <h3 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ t("auth.boostyTitle") }}</h3>
        <button
          type="button"
          class="rounded-md p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
          @click="boostyGlobalOpen = false"
        >
          <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
        </button>
      </div>
      <div class="space-y-3 p-4">
        <p class="text-[13px] text-[color:var(--tx-muted)]">{{ t("auth.boostyHint") }}</p>
        <div v-if="boostyError" class="rounded-md bg-red-500/10 px-3 py-2 text-xs text-red-500">{{ boostyError }}</div>
        <label class="block">
          <span class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("auth.boostyToken") }}</span>
          <input
            v-model="boostyToken"
            type="text"
            class="w-full rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] text-[color:var(--tx)] outline-none transition-colors "
            @keydown.enter="doBoostyAuth"
          />
        </label>
        <label class="block">
          <span class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("auth.boostySecret") }}</span>
          <input
            v-model="boostySecret"
            type="text"
            class="w-full rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] text-[color:var(--tx)] outline-none transition-colors "
            @keydown.enter="doBoostyAuth"
          />
        </label>
        <label class="block">
          <span class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("auth.boostyKey") }}</span>
          <input
            v-model="boostyKey"
            type="text"
            class="w-full rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] text-[color:var(--tx)] outline-none transition-colors "
            @keydown.enter="doBoostyAuth"
          />
        </label>
        <button
          type="button"
          class="flex w-full items-center justify-center gap-2 rounded-md  bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] px-3 py-2 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_22%,transparent)] disabled:opacity-50"
          :disabled="boostyBusy || !boostyToken.trim() || !boostySecret.trim() || !boostyKey.trim()"
          @click="doBoostyAuth"
        >
          <svg v-if="boostyBusy" viewBox="0 0 16 16" class="h-4 w-4 animate-spin fill-current">
            <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
          </svg>
          {{ boostyBusy ? t("auth.boostyBusy") : t("auth.boostyBtn") }}
        </button>
      </div>
    </div>
  </div>
</template>
