<script setup lang="ts">
import { useLauncherCtx } from "~/composables/useLauncherContext";
import { useI18n } from "#imports";

const { t } = useI18n();
const {
  scannerOpen,
  scanBusy,
  monoProfile,
  pickAndScanJar,
  scanResult,
  scannerHash,
  scanByHash,
} = useLauncherCtx();
</script>

<template>
  <div v-if="scannerOpen" class="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 p-4" @click.self="scannerOpen = false">
    <div class="w-full max-w-lg overflow-hidden rounded-xl  bg-[var(--panel)] shadow-2xl">
      <div class="flex items-center justify-between gap-2 border-b border-[var(--border)]  px-3.5 py-2.5">
        <h3 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ t("scanner.title") }}</h3>
        <button type="button" class="rounded-md  bg-[var(--input)] px-2 py-1 text-[13px] text-[color:var(--tx-muted)] hover:text-[var(--accent)] transition-colors" @click="scannerOpen = false">
          <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 5.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 7l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 8.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 7 3.72 3.78a.75.75 0 0 1 0-1.06Z"/></svg>
        </button>
      </div>
      <div class="space-y-3 p-4">
        <p class="text-[13px] leading-snug text-[color:var(--tx-muted)]">{{ t("scanner.note") }}</p>

        <div class="flex items-center gap-2">
          <button
            type="button"
            class="flex-1 rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] disabled:opacity-50"
            :disabled="scanBusy || !monoProfile"
            @click="pickAndScanJar"
          >
            <AppIcon v-if="scanBusy" name="spinner" class="mr-1 inline h-3 w-3 fill-current" />
            {{ scanBusy ? t("scanner.scanning") : t("scanner.pick") }}
          </button>
        </div>
        <p v-if="!monoProfile" class="text-xs text-[color:var(--tx-muted)]">{{ t("author.needLogin") }}</p>

        <!-- Результат скана -->
        <div v-if="scanResult" class="space-y-2 rounded-lg  p-3"
          :class="scanResult.safe ? 'bg-[#3fb950]/5' : 'bg-[#f85149]/10'">
          <div class="flex items-center gap-2">
            <span class="inline-flex h-6 w-6 items-center justify-center rounded-full"
              :class="scanResult.safe ? 'bg-[#3fb950]/15 text-[#3fb950]' : 'bg-[#f85149]/20 text-[#f85149]'">
              <AppIcon v-if="scanResult.safe" name="shield-check" class="h-4 w-4 fill-current" />
              <AppIcon v-else name="shield_alert" class="h-4 w-4 fill-current" />
            </span>
            <p class="text-[13px] font-bold" :class="scanResult.safe ? 'text-[#3fb950]' : 'text-[#f85149]'">
              {{ scanResult.safe ? t("scanner.safe") : t("scanner.dangerous") }}
            </p>
            <span v-if="scanResult.cached" class="ml-auto rounded-full  bg-[var(--input)] px-2 py-0.5 text-[11px] font-semibold uppercase text-[color:var(--tx-muted)]">{{ t("scanner.cached") }}</span>
          </div>
          <p class="text-[13px] text-[color:var(--tx)]">{{ scanResult.scanResult }}</p>
          <p class="break-all font-mono text-xs text-[color:var(--tx-muted)]">SHA-256: {{ scanResult.sha256 }}</p>
          <div v-if="scanResult.dangerousClasses" class="space-y-1">
            <p class="text-xs font-semibold uppercase tracking-wide text-[#f87171]">{{ t("scanner.classes") }}:</p>
            <div class="flex flex-wrap gap-1">
              <span v-for="cl in scanResult.dangerousClasses.split(',').map((s: string) => s.trim()).filter(Boolean)" :key="cl"
                class="rounded bg-[#f85149]/15 px-1.5 py-0.5 font-mono text-xs text-[#f87171]">
                {{ cl }}
              </span>
            </div>
          </div>
        </div>

        <!-- Проверка по хешу (без загрузки файла) -->
        <div class="space-y-1.5 border-t border-[var(--border)]  pt-3">
          <p class="text-xs font-semibold uppercase tracking-wide text-[color:var(--tx-muted)]">{{ t("scanner.byHash") }}</p>
          <div class="flex items-center gap-2">
            <input
              v-model="scannerHash"
              type="text"
              :placeholder="t('scanner.hashPh')"
              class="min-w-0 flex-1 rounded-md  bg-[var(--bg)] px-2.5 py-1.5 font-mono text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)]  focus:outline-none"
              @keydown.enter="scanByHash(scannerHash)"
            />
            <button
              type="button"
              class="shrink-0 rounded-md  bg-[var(--input)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
              :disabled="scanBusy"
              @click="scanByHash(scannerHash)"
            >
              {{ t("scanner.checkBtn") }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
