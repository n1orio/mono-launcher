<template>
  <Teleport to="body">
    <div v-if="modelValue" class="fixed inset-0 z-[70]" @mousedown="$emit('update:modelValue', false)" @contextmenu.prevent="$emit('update:modelValue', false)">
      <div
        class="fixed z-[71] w-52 overflow-hidden rounded-xl bg-[var(--panel)] py-1 shadow-2xl"
        :style="menuStyle"
        @mousedown.stop
        @contextmenu.stop
      >
        <div v-if="title" class="px-2.5 py-1.5">
          <div class="truncate text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ title }}</div>
          <div v-if="subtitle" class="truncate text-xs text-[color:var(--tx-muted)]">{{ subtitle }}</div>
        </div>
        <div v-if="title" class="mx-3 border-t border-[var(--border)]"></div>
        <slot />
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  modelValue: boolean;
  x: number;
  y: number;
  title?: string;
  subtitle?: string;
}>();

defineEmits<{ "update:modelValue": [value: boolean] }>();

const menuStyle = computed(() => {
  const mw = 220;
  const mh = 300;
  let x = props.x;
  let y = props.y;
  if (x + mw > window.innerWidth) x = window.innerWidth - mw - 8;
  if (y + mh > window.innerHeight) y = window.innerHeight - mh - 8;
  if (x < 4) x = 4;
  if (y < 4) y = 4;
  return { left: `${x}px`, top: `${y}px` };
});
</script>
