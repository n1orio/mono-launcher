<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { openPackDir } from "~/lib/bridge";
import { useLauncherCtx } from "~/composables/useLauncherContext";
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

const { copyPackDeepLink } = useLauncherCtx();

const imgFailed = ref(false);
const menuOpen = ref(false);
const showBadges = ref(true);

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

const isManaged = computed(() => props.pack?.builtin === true || props.pack?.kind === "remote");
const hasAuthlib = computed(() => props.pack?.meta?.use_authlib === true);

function handleMenu(e: MouseEvent) {
  e.stopPropagation();
  menuOpen.value = !menuOpen.value;
}

function closeMenu() {
  menuOpen.value = false;
}

function handleFolder() {
  closeMenu();
  if (props.pack?.id) openPackDir(props.pack.id);
}

function handleCopyLink() {
  closeMenu();
  copyPackDeepLink(props.pack);
}

function toggleBadges() {
  showBadges.value = !showBadges.value;
}

function handleDocClick(e: MouseEvent) {
  const el = e.target as HTMLElement;
  if (!el.closest("[data-modpack-card]")) {
    closeMenu();
  }
}

onMounted(() => {
  document.addEventListener("click", handleDocClick);
});
onBeforeUnmount(() => {
  document.removeEventListener("click", handleDocClick);
});
</script>

<template>
  <div
    data-modpack-card
    class="group relative flex flex-col rounded-2xl transition-all duration-200 ease-out cursor-pointer select-none bg-[var(--panel)] border border-[var(--border)] hover:scale-[1.02] active:scale-95 overflow-hidden shadow-sm"
    :title="pack?.name || 'Сборка'"
    @pointerdown="emit('pointerdown', $event)"
    @click="emit('click', $event, pack.id)"
    @contextmenu.prevent="emit('contextmenu', $event, pack)"
  >
    <!-- Иконка -->
    <div class="flex-1 w-full flex items-center justify-center overflow-hidden relative pointer-events-none min-h-[80px]">
      <img
        v-if="iconSrc"
        :src="iconSrc"
        draggable="false"
        class="w-full h-full object-cover select-none"
        @error="imgFailed = true"
      />
      <div
        v-else
        class="w-full h-full flex items-center justify-center text-white font-black text-5xl leading-none tracking-tighter select-none"
        :style="{ background: packGradient(props.pack?.color || packName) }"
      >
        {{ packName[0].toUpperCase() }}
      </div>

      <!-- Плашки (managed / authlib) -->
      <div v-if="showBadges" class="absolute top-1.5 left-1.5 z-10 flex flex-col gap-1">
        <span v-if="isManaged" class="inline-flex items-center gap-1 rounded-full px-1.5 py-0.5 text-[10px] font-semibold bg-[#f0883e]/20 text-[#f0883e]">
          managed
        </span>
        <span v-if="hasAuthlib" class="inline-flex items-center gap-1 rounded-full px-1.5 py-0.5 text-[10px] font-semibold bg-[#627eea]/20 text-[#627eea]">
          authlib
        </span>
      </div>

      <!-- Кнопка «Играть» -->
      <div
        class="absolute inset-0 bg-black/40 backdrop-blur-[2px] rounded-2xl opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center p-2 z-20 pointer-events-none group-hover:pointer-events-auto"
      >
        <button
          type="button"
          class="px-5 py-3 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white text-sm font-bold uppercase tracking-wider shadow-xl flex items-center gap-2 transform active:scale-95 transition-all disabled:opacity-50 pointer-events-auto"
          :disabled="busy || gameRunning"
          @click.stop="emit('play', pack.id)"
        >
          <svg viewBox="0 0 16 16" class="w-4 h-4 fill-current"><path d="M3.5 2.5a.75.75 0 0 1 1.25-.64l7.5 4.5a.75.75 0 0 1 0 1.28l-7.5 4.5a.75.75 0 0 1-1.25-.64V2.5Z"/></svg>
          Играть
        </button>
      </div>
    </div>

    <!-- Название -->
    <div class="px-2 py-1.5 text-center border-t border-[var(--border)]">
      <span class="text-[12px] font-semibold text-[color:var(--tx)] truncate block">{{ pack?.name ?? "?" }}</span>
    </div>

    <!-- Кнопки внизу -->
    <div class="flex items-center gap-1 px-1.5 pb-1.5">
      <button
        type="button"
        class="flex-1 px-2 py-1.5 rounded-lg bg-[var(--input)] hover:bg-[var(--hover)] text-[11px] font-semibold text-[color:var(--tx)] active:scale-95 transition-all"
        @click.stop="handleFolder"
        title="Открыть папку сборки"
      >
        📁
      </button>
      <button
        type="button"
        class="flex-1 px-2 py-1.5 rounded-lg bg-[var(--input)] hover:bg-[var(--hover)] text-[11px] font-semibold text-[color:var(--tx)] active:scale-95 transition-all"
        @click.stop="handleCopyLink"
        title="Скопировать диплинк"
      >
        🔗
      </button>
      <button
        type="button"
        class="px-2 py-1.5 rounded-lg bg-[var(--input)] hover:bg-[var(--hover)] text-[11px] font-semibold text-[color:var(--tx)] active:scale-95 transition-all relative"
        @click.stop="handleMenu"
        title="Меню"
      >
        ⋯
        <!-- Выпадающее меню -->
        <div v-if="menuOpen" class="absolute bottom-full left-0 mb-1 w-44 rounded-xl bg-[var(--panel)] border border-[var(--border)] shadow-xl z-50 overflow-hidden">
          <button
            type="button"
            class="w-full px-3 py-2 text-[12px] text-left hover:bg-[var(--hover)] text-[color:var(--tx)] transition-colors flex items-center gap-2"
            @click.stop="handleFolder"
          >
            📁 Открыть папку сборки
          </button>
          <button
            type="button"
            class="w-full px-3 py-2 text-[12px] text-left hover:bg-[var(--hover)] text-[color:var(--tx)] transition-colors flex items-center gap-2"
            @click.stop="handleCopyLink"
          >
            🔗 Скопировать диплинк
          </button>
          <button
            type="button"
            class="w-full px-3 py-2 text-[12px] text-left hover:bg-[var(--hover)] text-[color:var(--tx)] transition-colors flex items-center gap-2 border-t border-[var(--border)]"
            @click.stop="toggleBadges"
          >
            {{ showBadges ? '👁️ Скрыть плашки' : '📋 Показать плашки' }}
          </button>
        </div>
      </button>
    </div>
  </div>
</template>