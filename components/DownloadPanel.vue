<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from "vue";
import { useLauncherCtx } from "~/composables/useLauncherContext";
import { pauseDownload, resumeDownload, cancelDownload, isDownloadPaused } from "~/lib/bridge";

const ctx = useLauncherCtx();
const {
  t,
  busy,
  progress,
  percent,
  filePercent,
  eta,
  speedHistorySmooth,
  filesDone,
  formatBytes,
  phaseLabel,
} = ctx;

const expanded = ref(false);
const paused = ref(false);

function formatEta(seconds: number | null): string {
  if (seconds === null || seconds <= 0) return "";
  if (seconds < 60) return `~${seconds} ${t("units.s")}`;
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  if (m < 60) return `~${m} ${t("units.min")} ${s > 0 ? `${s} ${t("units.s")}` : ""}`;
  const h = Math.floor(m / 60);
  const rm = m % 60;
  return `~${h} ${t("units.h")} ${rm > 0 ? `${rm} ${t("units.min")}` : ""}`;
}

/** Build SVG path for speed sparkline */
const sparklinePath = computed(() => {
  const data = speedHistorySmooth.value;
  if (data.length < 2) return "";
  const max = Math.max(...data, 1);
  const w = 120;
  const h = 28;
  const step = w / (data.length - 1);
  const points = data.map((v, i) => [i * step, h - (v / max) * h]);
  return points.map((p, i) => `${i === 0 ? "M" : "L"}${p[0].toFixed(1)},${p[1].toFixed(1)}`).join(" ");
});

const sparklineFill = computed(() => {
  const path = sparklinePath.value;
  if (!path) return "";
  const w = 120;
  const h = 28;
  return `${path} L${w},${h} L0,${h} Z`;
});

const showPanel = computed(() => progress.value && busy.value);

async function togglePause() {
  if (paused.value) {
    await resumeDownload();
    paused.value = false;
  } else {
    await pauseDownload();
    paused.value = true;
  }
}

async function handleCancel() {
  await cancelDownload();
  paused.value = false;
}

watch(showPanel, (v) => {
  if (v) paused.value = false;
});
</script>

<template>
  <Transition name="dlpanel">
    <div v-if="showPanel" class="absolute bottom-0 left-0 right-0 z-30 border-t border-[var(--border)] bg-[var(--panel)] shadow-[0_-4px_20px_rgba(0,0,0,0.4)]">
      <!-- Compact header (always visible) -->
      <div class="px-3 pt-2.5 pb-2">
        <div class="mb-1.5 flex items-center justify-between">
          <span class="truncate pr-2 text-[13px] font-semibold text-[color:var(--tx)]">
            {{ phaseLabel(progress!.phase) }}
          </span>
          <div class="flex items-center gap-2">
            <span v-if="progress!.fileTotal > 1" class="tabular-nums font-mono text-xs text-[color:var(--tx-muted)]">
              {{ t("progress.files", { n: filesDone, m: progress!.fileTotal }) }}
            </span>
            <span class="tabular-nums font-mono text-xs font-semibold text-[var(--accent)]">
              {{ percent }}%
            </span>
          </div>
        </div>

        <!-- Main progress bar with gradient -->
        <div class="relative h-2 w-full overflow-hidden rounded-full bg-[var(--input)]">
          <div
            class="absolute inset-y-0 left-0 rounded-full transition-all duration-300 ease-out"
            :style="{
              width: `${percent}%`,
              background: 'linear-gradient(90deg, #2f81f7, #58a6ff)',
            }"
          />
          <!-- Animated shine effect -->
          <div
            v-if="percent > 0 && percent < 100"
            class="absolute inset-y-0 left-0 overflow-hidden rounded-full"
            :style="{ width: `${percent}%` }"
          >
            <div class="dl-shine h-full w-full" />
          </div>
        </div>

        <!-- Stats row -->
        <div class="mt-1.5 flex items-center justify-between text-xs text-[color:var(--tx-muted)]">
          <span class="truncate max-w-[140px]" :title="progress!.currentFile">
            {{ progress!.currentFile || t("side.preparing") }}
          </span>
          <div class="flex items-center gap-2">
            <span v-if="progress!.speed > 0" class="tabular-nums font-mono">
              {{ formatBytes(progress!.speed) }}{{ t("units.perSec") }}
            </span>
            <span v-if="eta !== null" class="tabular-nums font-mono text-[color:var(--tx-muted)]">
              {{ formatEta(eta) }}
            </span>
          </div>
        </div>

        <!-- Per-file progress bar (during multi-file installs) -->
        <div v-if="progress!.fileTotal > 1 && filePercent > 0" class="mt-1.5">
          <div class="flex items-center justify-between mb-0.5">
            <span class="text-[11px] text-[color:var(--tx-muted)]">{{ t("progress.currentFile") }}</span>
            <span class="tabular-nums font-mono text-[11px] text-[color:var(--tx-muted)]">{{ filePercent }}%</span>
          </div>
          <div class="h-1 w-full overflow-hidden rounded-full bg-[var(--input)]">
            <div
              class="h-full rounded-full transition-all duration-200"
              :style="{
                width: `${filePercent}%`,
                background: 'linear-gradient(90deg, color-mix(in srgb, var(--accent) 70%, transparent), var(--accent))',
              }"
            />
          </div>
        </div>
      </div>

      <!-- Controls row: expand + pause + cancel -->
      <div class="flex items-center border-t border-[var(--border)]">
        <button
          type="button"
          class="flex flex-1 items-center justify-center gap-1 py-1 text-[11px] text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]"
          @click="expanded = !expanded"
        >
          <AppIcon
            name="chevron-down"
            class="h-3 w-3 fill-current transition-transform"
            :class="expanded ? 'rotate-180' : ''"
          />
          {{ expanded ? t("progress.hideDetails") : t("progress.showDetails") }}
        </button>
        <div class="h-4 w-px bg-[var(--border)]"></div>
        <button
          type="button"
          class="flex items-center justify-center gap-1 px-3 py-1 text-[11px] transition-colors"
          :class="paused ? 'text-[#3fb950] hover:bg-[#3fb950]/10' : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]'"
          :title="paused ? t('progress.resume') : t('progress.pause')"
          @click="togglePause"
        >
          <AppIcon v-if="!paused" name="pause" class="h-3 w-3 fill-current" />
          <AppIcon v-else name="play" class="h-3 w-3 fill-current" />
          {{ paused ? t("progress.resume") : t("progress.pause") }}
        </button>
        <div class="h-4 w-px bg-[var(--border)]"></div>
        <button
          type="button"
          class="flex items-center justify-center gap-1 px-3 py-1 text-[11px] text-[color:var(--tx-muted)] transition-colors hover:bg-[#f85149]/10 hover:text-[#f85149]"
          :title="t('progress.cancel')"
          @click="handleCancel"
        >
          <AppIcon name="x" class="h-3 w-3 fill-current" />
          {{ t("progress.cancel") }}
        </button>
      </div>

      <Transition name="dlpanel-expand">
        <div v-if="expanded" class="border-t border-[var(--border)] px-3 py-2.5 space-y-2">
          <!-- Speed graph -->
          <div v-if="speedHistorySmooth.length > 1">
            <p class="mb-1 text-[11px] font-medium text-[color:var(--tx-muted)]">{{ t("progress.speedGraph") }}</p>
            <div class="relative h-8 w-full rounded-md bg-[var(--bg)] overflow-hidden">
              <svg viewBox="0 0 120 28" preserveAspectRatio="none" class="h-full w-full">
                <path :d="sparklineFill" fill="rgba(47,129,247,0.15)" />
                <path :d="sparklinePath" fill="none" stroke="#2f81f7" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            </div>
          </div>

          <!-- Downloaded bytes -->
          <div v-if="progress!.total > 0" class="flex items-center justify-between text-[12px]">
            <span class="text-[color:var(--tx-muted)]">{{ t("progress.downloaded") }}</span>
            <span class="tabular-nums font-mono text-[color:var(--tx)]">
              {{ formatBytes(progress!.current) }} / {{ formatBytes(progress!.total) }}
            </span>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<style scoped>
.dl-shine {
  background: linear-gradient(
    90deg,
    transparent 0%,
    rgba(255, 255, 255, 0.15) 50%,
    transparent 100%
  );
  animation: dl-shine 2s ease-in-out infinite;
}

@keyframes dl-shine {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(200%); }
}

.dlpanel-enter-active,
.dlpanel-leave-active {
  transition: transform 0.25s ease, opacity 0.25s ease;
}
.dlpanel-enter-from,
.dlpanel-leave-to {
  opacity: 0;
  transform: translateY(100%);
}
.dlpanel-enter-to,
.dlpanel-leave-from {
  opacity: 1;
  transform: translateY(0);
}

.dlpanel-expand-enter-active,
.dlpanel-expand-leave-active {
  transition: all 0.2s ease;
}
.dlpanel-expand-enter-from,
.dlpanel-expand-leave-to {
  opacity: 0;
  max-height: 0;
  overflow: hidden;
}
.dlpanel-expand-enter-to,
.dlpanel-expand-leave-from {
  opacity: 1;
  max-height: 300px;
}
</style>
