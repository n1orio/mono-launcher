<template>
  <div class="overflow-hidden rounded-md bg-[var(--bg)]">
    <div class="flex items-center gap-1 border-b border-[var(--border)] px-1.5 py-1">
      <template v-if="mode === 'edit'">
        <button type="button" class="h-6 min-w-6 rounded px-1 text-xs font-bold text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]" :title="t('md.bold')" @click="wrap('**')"><span class="italic">B</span></button>
        <button type="button" class="h-6 min-w-6 rounded px-1 text-xs font-semibold italic text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]" :title="t('md.italic')" @click="wrap('_')">I</button>
        <button type="button" class="h-6 min-w-6 rounded px-1 font-mono text-[11px] font-bold text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]" :title="t('md.code')" @click="wrap('`')">&lt;&gt;</button>
        <button type="button" class="h-6 min-w-6 rounded px-1 text-sm leading-none text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]" :title="t('md.list')" @click="prefixLine('- ')">•</button>
        <button type="button" class="h-6 min-w-6 rounded px-1 text-[12px] leading-none text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]" :title="t('md.heading')" @click="prefixLine('## ')">H</button>
        <button type="button" class="h-6 min-w-6 rounded px-1 text-xs font-bold text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]" :title="t('md.link')" @click="insertLink()">🔗</button>
      </template>
      <div class="ml-auto flex items-center rounded p-0.5">
        <button type="button" class="rounded px-2 py-0.5 text-xs font-medium transition-colors"
          :class="mode === 'edit' ? 'bg-[var(--input)] text-[color:var(--tx-strong)]' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx)]'"
          @click="mode = 'edit'">{{ t("md.edit") }}</button>
        <button type="button" class="rounded px-2 py-0.5 text-xs font-medium transition-colors"
          :class="mode === 'preview' ? 'bg-[var(--input)] text-[color:var(--tx-strong)]' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx)]'"
          @click="mode = 'preview'">{{ t("md.preview") }}</button>
      </div>
    </div>
    <textarea
      v-show="mode === 'edit'"
      ref="ta"
      :value="modelValue"
      :rows="rows"
      :placeholder="placeholder"
      class="block w-full resize-y bg-transparent px-3 py-2 text-[13px] leading-relaxed text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none"
      @input="$emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    ></textarea>
    <div v-show="mode === 'preview'" class="max-h-80 overflow-y-auto px-3 py-2 nice-scrollbar">
      <Markdown v-if="modelValue?.trim()" :source="modelValue" />
      <p v-else class="text-[13px] italic text-[color:var(--tx-muted)]">{{ t("md.empty") }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "~/composables/useI18n";

withDefaults(defineProps<{ modelValue?: string; rows?: number; placeholder?: string }>(), {
  modelValue: "",
  rows: 4,
  placeholder: "",
});

const { t } = useI18n();
const mode = ref<"edit" | "preview">("edit");
const ta = ref<HTMLTextAreaElement | null>(null);

const emit = defineEmits<{ (e: "update:modelValue", v: string): void }>();

function emitValue(v: string) {
  // Обновляем и нативное значение textarea, чтобы курсор не прыгал.
  if (ta.value && ta.value.value !== v) ta.value.value = v;
  emit("update:modelValue", v);
}

function wrap(marker: string) {
  const el = ta.value;
  if (!el) return;
  const { selectionStart: s, selectionEnd: e, value } = el;
  const sel = value.slice(s, e);
  const next = value.slice(0, s) + marker + sel + marker + value.slice(e);
  emitValue(next);
  requestAnimationFrame(() => {
    el.focus();
    el.setSelectionRange(s + marker.length, e + marker.length);
  });
}

function prefixLine(prefix: string) {
  const el = ta.value;
  if (!el) return;
  const { selectionStart: s, value } = el;
  const lineStart = value.lastIndexOf("\n", Math.max(0, s - 1)) + 1;
  const next = value.slice(0, lineStart) + prefix + value.slice(lineStart);
  emitValue(next);
  requestAnimationFrame(() => {
    el.focus();
    el.setSelectionRange(s + prefix.length, s + prefix.length);
  });
}

function insertLink() {
  const el = ta.value;
  if (!el) return;
  const { selectionStart: s, selectionEnd: e, value } = el;
  const sel = value.slice(s, e) || t("md.linkText");
  const snippet = `[${sel}](https://)`;
  const next = value.slice(0, s) + snippet + value.slice(e);
  emitValue(next);
  requestAnimationFrame(() => {
    el.focus();
    const urlStart = s + sel.length + 3;
    el.setSelectionRange(urlStart, urlStart + 8);
  });
}
</script>
