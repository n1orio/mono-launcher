<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { packGradient } from "~/lib/misc";

const props = defineProps<{
  pack: any;
  busy?: boolean;
  gameRunning?: boolean;
}>();

const emit = defineEmits<{
  (e: "click", ev: MouseEvent, id: string): void;
  (e: "pointerdown", ev: PointerEvent): void;
  (e: "contextmenu", ev: MouseEvent, pack: any): void;
  (e: "play", id: string): void;
}>();

const imgFailed = ref(false);

watch(() => props.pack?.icon || props.pack?.icon_url, () => {
  imgFailed.value = false;
});

const iconSrc = computed(() => {
  if (imgFailed.value) return null;
  const i = props.pack?.icon || props.pack?.icon_url;
  if (!i || typeof i !== "string" || !i.trim()) return null;
  return convertFileSrc(i);
});

const packName = computed(() => props.pack?.name ?? "M");
</script>

<template>
  <div
    class="group relative aspect-square w-full rounded-2xl transition-all duration-200 ease-out cursor-pointer select-none bg-[var(--panel)] border border-[var(--border)] hover:scale-[1.02] active:scale-95 flex items-center justify-center overflow-hidden shadow-sm"
    :title="pack?.name || 'Сборка'"
    @pointerdown="emit('pointerdown', $event)"
    @click="emit('click', $event, pack.id)"
    @contextmenu.prevent="emit('contextmenu', $event, pack)"
  >
    <div class="w-full h-full flex items-center justify-center overflow-hidden relative pointer-events-none">
      <img
        v-if="iconSrc"
        :src="iconSrc"
        draggable="false"
        class="w-full h-full object-cover select-none"
        @error="imgFailed = true"
      />
      <div
        v-else
        class="w-full h-full flex items-center justify-center text-white font-black text-4xl leading-none tracking-tighter select-none"
        :style="{ background: packGradient(props.pack?.color || packName) }"
      >
        {{ packName[0].toUpperCase() }}
      </div>
    </div>

    <!-- Кнопка «Играть» -->
    <div
      class="absolute inset-0 bg-black/50 backdrop-blur-[2px] rounded-2xl opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center p-2 z-20 pointer-events-none group-hover:pointer-events-auto"
    >
      <button
        type="button"
        class="px-3.5 py-2 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-bold uppercase tracking-wider shadow-xl flex items-center gap-1.5 transform active:scale-95 transition-all disabled:opacity-50 pointer-events-auto"
        :disabled="busy || gameRunning"
        @click.stop="emit('play', pack.id)"
      >
        <svg viewBox="0 0 16 16" class="w-3.5 h-3.5 fill-current"><path d="M3.5 2.5a.75.75 0 0 1 1.25-.64l7.5 4.5a.75.75 0 0 1 0 1.28l-7.5 4.5a.75.75 0 0 1-1.25-.64V2.5Z"/></svg>
        Играть
      </button>
    </div>
  </div>
</template>
