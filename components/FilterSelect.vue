<template>
  <div ref="root" class="relative">
  <button
  type="button"
  class="flex max-w-[180px] items-center gap-1.5 rounded-md  bg-[var(--bg)] px-2.5 py-1.5 text-xs text-[color:var(--tx)] transition-colors  disabled:opacity-50"
  :disabled="disabled || !options.length"
  @click="open = !open"
  >
  <span class="flex min-w-0 items-center gap-1.5">
  <span v-if="selectedLabel" class="flex min-w-0 items-center gap-1.5">
  <span class="truncate">{{ selectedLabel }}</span>
  <span
  v-if="multiple && modelValue.length > 1"
  class="shrink-0 rounded-full bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] px-1.5 text-[11px] font-semibold text-[var(--accent)]"
  >{{ modelValue.length }}</span>
  </span>
  <span v-else class="truncate text-[color:var(--tx-muted)]">{{ placeholder }}</span>
  </span>
  <svg
  viewBox="0 0 16 16"
  class="ml-auto h-3 w-3 shrink-0 fill-[var(--tx-muted)] transition-transform"
  :class="{ 'rotate-180': open }"
  >
  <path d="M3.22 5.78a.75.75 0 0 1 1.06 0L8 9.44l3.72-3.66a.75.75 0 1 1 1.06 1.08l-4.25 4.18a.75.75 0 0 1-1.06 0L3.22 6.86a.75.75 0 0 1 0-1.08Z"/>
  </svg>
  </button>
  <div
  v-if="open"
  class="absolute left-0 top-[calc(100%+4px)] z-50 flex max-h-64 w-52 flex-col overflow-hidden rounded-md  bg-[var(--panel)] shadow-xl"
  >
  <div class="flex shrink-0 items-center gap-2 border-b border-[var(--border)]  px-2 py-1.5">
  <svg viewBox="0 0 16 16" class="h-3 w-3 shrink-0 fill-[var(--tx-muted)]"><path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"/></svg>
  <input
  v-model="query"
  type="text"
  :placeholder="t('mods.filterSearch')"
  class="w-full bg-transparent text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] outline-none"
  />
  <button
  v-if="modelValue.length"
  type="button"
  class="shrink-0 text-[11px] text-[var(--accent)] hover:underline"
  @click="reset"
  >{{ t("mods.reset") }}</button>
  </div>
  <div class="min-h-0 flex-1 overflow-y-auto p-1">
  <button
  v-for="o in filtered"
  :key="o.value"
  type="button"
  class="flex w-full items-center gap-2 rounded px-2 py-1 text-left text-xs text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  @click="toggle(o)"
  >
  <span
  class="flex h-3.5 w-3.5 shrink-0 items-center justify-center rounded"
  :class="isOn(o.value) ? ' bg-[var(--accent)]' : ' bg-[var(--input)]'"
  >
  <svg v-if="isOn(o.value)" viewBox="0 0 16 16" class="h-2.5 w-2.5 fill-white"><path d="M12.7 4.7a.75.75 0 0 1 0 1.06l-5.5 5.5a.75.75 0 0 1-1.06 0l-2.5-2.5a.75.75 0 1 1 1.06-1.06l1.97 1.97 4.97-4.97a.75.75 0 0 1 1.06 0Z"/></svg>
  </span>
  <span class="truncate" :class="{ 'font-semibold text-[var(--accent)]': isOn(o.value) }">{{ o.label }}</span>
  </button>
  <p v-if="!filtered.length" class="px-2 py-1.5 text-xs text-[color:var(--tx-muted)]">{{ t("mods.noResults") }}</p>
  </div>
  </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { useI18n } from "~/composables/useI18n";

const props = withDefaults(
  defineProps<{
  modelValue: string[];
  options: { value: string; label: string }[];
  placeholder: string;
  multiple?: boolean;
  disabled?: boolean;
  }>(),
  { multiple: true, disabled: false }
);

const emit = defineEmits<{
  "update:modelValue": [v: string[]];
  change: [];
}>();

const { t } = useI18n();

const open = ref(false);
const query = ref("");
const root = ref<HTMLElement | null>(null);

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase();
  if (!q) return props.options;
  return props.options.filter((o) => o.label.toLowerCase().includes(q));
});

const selectedLabel = computed(() => {
  if (!props.modelValue.length) return "";
  if (!props.multiple) {
  const on = props.options.filter((o) => props.modelValue.includes(o.value));
  return on.map((o) => o.label).join(", ");
  }
  const first = props.options.find((o) => o.value === props.modelValue[0]);
  return first?.label ?? props.modelValue[0];
});

function isOn(value: string) {
  return props.modelValue.includes(value);
}

function toggle(o: { value: string; label: string }) {
  const on = props.modelValue.includes(o.value);
  const next = on
  ? props.modelValue.filter((v) => v !== o.value)
  : props.multiple
  ? [...props.modelValue, o.value]
  : [o.value];
  emit("update:modelValue", next);
  emit("change");
  if (!props.multiple) open.value = false;
}

function reset() {
  emit("update:modelValue", []);
  emit("change");
}

function onDocDown(ev: MouseEvent) {
  if (!open.value) return;
  if (root.value && root.value.contains(ev.target as Node)) return;
  open.value = false;
}

function onKey(ev: KeyboardEvent) {
  if (ev.key === "Escape") open.value = false;
}

watch(open, (v) => {
  query.value = "";
  if (v) {
  document.addEventListener("mousedown", onDocDown);
  document.addEventListener("keydown", onKey);
  } else {
  document.removeEventListener("mousedown", onDocDown);
  document.removeEventListener("keydown", onKey);
  }
});

onBeforeUnmount(() => {
  document.removeEventListener("mousedown", onDocDown);
  document.removeEventListener("keydown", onKey);
});
</script>
