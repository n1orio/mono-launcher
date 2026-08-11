<template>
  <div class="markdown" v-html="html" @click="onClick"></div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { marked } from "marked";
import DOMPurify from "dompurify";
import { isTauri, openExternal } from "~/lib/bridge";

const props = defineProps<{ source?: string | null }>();

marked.setOptions({ gfm: true, breaks: true });

DOMPurify.addHook("afterSanitizeAttributes", (node) => {
  if (node.tagName === "IMG") node.setAttribute("loading", "lazy");
});

const html = computed(() => {
  if (!props.source) return "";
  const raw = marked.parse(props.source, { async: false }) as string;
  return DOMPurify.sanitize(raw);
});

function onClick(ev: MouseEvent) {
  const a = (ev.target as HTMLElement).closest("a");
  if (!a) return;
  const href = a.getAttribute("href");
  if (!href || href.startsWith("#")) return;
  ev.preventDefault();
  if (isTauri()) openExternal(href);
  else window.open(href, "_blank", "noopener");
}
</script>
