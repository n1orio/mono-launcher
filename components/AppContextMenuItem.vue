<template>
  <button
    type="button"
    class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[13px] transition-colors"
    :class="[
      danger ? 'text-[#f85149] hover:bg-[#f85149]/10' : 'text-[color:var(--tx)] hover:bg-[var(--hover)]',
      disabled ? 'opacity-50' : ''
    ]"
    :disabled="disabled"
    @click.stop="$emit('click', $event)"
  >
    <slot name="icon" />
    <span class="min-w-0 truncate">{{ label }}</span>
  </button>
</template>

<script setup lang="ts">
defineProps<{
  label: string;
  danger?: boolean;
  disabled?: boolean;
}>();

defineEmits<{ click: [e: MouseEvent] }>();
</script>
