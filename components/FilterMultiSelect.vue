<script setup lang="ts">
interface Props {
  modelValue: string[];
  options: Array<{ value: string; label: string; count?: number }>;
  placeholder: string;
  multiple?: boolean;
  disabled?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  multiple: true,
  disabled: false,
});

const emit = defineEmits<{
  "update:modelValue": [value: string[]];
  change: [value: string[]];
}>();

const isOpen = ref(false);
const localSearch = ref("");

const filteredOptions = computed(() => {
  if (!localSearch.value) return props.options;
  const q = localSearch.value.toLowerCase();
  return props.options.filter((o) =>
    o.label.toLowerCase().includes(q) || o.value.toLowerCase().includes(q)
  );
});

function toggleOpen() {
  if (!props.disabled) isOpen.value = !isOpen.value;
}

function close() {
  isOpen.value = false;
}

function toggleOption(value: string) {
  const arr = [...props.modelValue];
  const idx = arr.indexOf(value);
  if (idx >= 0) arr.splice(idx, 1);
  else arr.push(value);
  emit("update:modelValue", arr);
  emit("change", arr);
  if (!props.multiple) close();
}

function selectAll() {
  emit("update:modelValue", props.options.map((o) => o.value));
  emit("change", props.options.map((o) => o.value));
}

function clearAll() {
  emit("update:modelValue", []);
  emit("change", []);
}
</script>

<template>
  <div class="relative" @click.outside="close">
    <button
      type="button"
      class="flex items-center gap-1.5 w-full min-w-[140px] rounded-lg bg-[var(--input)] px-3 py-2 text-[13px] transition-colors"
      :class="[
        'border',
        modelValue.length > 0
          ? 'border-[var(--accent)] text-[color:var(--tx-strong)]'
          : 'border-[var(--border)] text-[color:var(--tx-muted)]',
        disabled ? 'opacity-50 cursor-not-allowed' : 'hover:border-[var(--accent)/50]',
      ]"
      @click="toggleOpen"
      :disabled="disabled"
    >
      <span v-if="modelValue.length === 0" class="truncate">
        {{ placeholder }}
      </span>
      <span v-else class="truncate">
        {{ modelValue.length }} {{ modelValue.length === 1 ? "выбрано" : "выбрано" }}
      </span>
      <svg
        class="h-3.5 w-3.5 shrink-0 ml-1 fill-[var(--tx-muted)] transition-transform"
        :class="{ 'rotate-180': isOpen }"
        viewBox="0 0 16 16"
      >
        <path d="M12.53 5.47l-4.25 4.25a.4.4 0 0 1-.56 0L3.47 5.47a.75.75 0 0 0-1.06 1.06l4.25 4.25a1.9 1.9 0 0 0 2.68 0l4.25-4.25a.75.75 0 1 0-1.06-1.06Z" />
      </svg>
    </button>

    <Transition
      enter-active-class="transition duration-150 ease-out"
      enter-from-class="opacity-0 -translate-y-1"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition duration-100 ease-in"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-1"
    >
      <div
        v-if="isOpen && !disabled"
        class="absolute z-50 mt-1 w-full min-w-[200px] max-h-60 overflow-auto rounded-lg border border-[var(--border)] bg-[var(--bg)] shadow-xl py-1.5"
      >
        <div v-if="modelValue.length > 0 && multiple" class="px-2 py-1.5 border-b border-[var(--border)] flex gap-1.5">
          <button
            type="button"
            class="text-[11px] font-medium text-[var(--accent)] hover:underline"
            @click.stop="selectAll"
          >
            Все
          </button>
          <button
            type="button"
            class="text-[11px] font-medium text-[var(--accent)] hover:underline"
            @click.stop="clearAll"
          >
            Очистить
          </button>
        </div>
        <input
          v-if="options.length > 10"
          type="text"
          v-model="localSearch"
          placeholder="Поиск..."
          class="w-full px-2 py-1.5 text-[13px] bg-[var(--input)] border-none outline-none placeholder-[var(--tx-muted)]"
        />
        <div class="max-h-[300px] overflow-y-auto">
          <label
            v-for="opt in filteredOptions"
            :key="opt.value"
            class="flex items-center gap-2 px-2 py-1.5 text-[13px] hover:bg-[var(--hover)] cursor-pointer"
          >
            <input
              type="checkbox"
              :checked="modelValue.includes(opt.value)"
              @change="toggleOption(opt.value)"
              class="h-3.5 w-3.5 rounded border-[var(--border)] bg-[var(--input)] text-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)] focus:ring-offset-2 focus:ring-offset-[var(--bg)]"
            />
            <span class="truncate">{{ opt.label }}</span>
            <span v-if="opt.count" class="ml-auto text-xs text-[color:var(--tx-muted)]">
              {{ opt.count }}
            </span>
          </label>
          <p v-if="filteredOptions.length === 0" class="px-2 py-2 text-center text-[13px] text-[color:var(--tx-muted)]">
            Ничего не найдено
          </p>
        </div>
      </div>
    </Transition>
  </div>
</template>