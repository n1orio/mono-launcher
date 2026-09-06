<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { packGradient } from "~/lib/misc";

const props = defineProps<{
  folder: { id: string; name?: string };
  folderPacks?: any[];
}>();

const emit = defineEmits<{
  (e: "click"): void;
}>();

const packsList = computed(() => props.folderPacks || []);
const count = computed(() => packsList.value.length);
const failed = ref<Record<string, boolean>>({});

watch(packsList, () => { failed.value = {}; }, { deep: true });

function getIconSrc(p: any) {
  if (!p) return null;
  const icon = p.icon || p.icon_url;
  if (!icon || typeof icon !== "string" || !icon.trim()) return null;
  return convertFileSrc(icon);
}
</script>

<template>
  <div
    class="group relative aspect-square w-full rounded-2xl transition-all duration-200 ease-out cursor-pointer select-none bg-[var(--panel)] border border-[var(--border)] hover:border-[var(--accent)]/30 hover:scale-[1.02] active:scale-95 flex items-center justify-center overflow-hidden p-2.5 shadow-sm"
    :title="folder.name || 'Категория'"
    @click="emit('click')"
  >
    <!-- Превью 2х2 с мягкими углами -->
    <div class="w-full h-full grid grid-cols-2 grid-rows-2 gap-1.5 pointer-events-none">
      <template v-for="i in 4" :key="i">
        <template v-if="i < 4 || count <= 4">
          <div
            v-if="packsList[i - 1]"
            class="w-full h-full rounded-xl overflow-hidden relative bg-black/20"
          >
            <img
              v-if="getIconSrc(packsList[i - 1]) && !failed[`${packsList[i-1].id}`]"
              :src="getIconSrc(packsList[i - 1])!"
              class="w-full h-full object-cover select-none"
              draggable="false"
              @error="failed[`${packsList[i-1].id}`] = true"
            />
            <div
              v-else
              class="w-full h-full flex items-center justify-center text-white font-black text-sm"
              :style="{ background: packGradient(packsList[i - 1].color || packsList[i - 1].name) }"
            >
              {{ (packsList[i - 1].name || '?')[0].toUpperCase() }}
            </div>
          </div>
          <div v-else class="w-full h-full rounded-xl bg-[var(--input)] border border-[var(--border)]" />
        </template>

        <!-- 4-й слот: микро-сетка, если сборок больше 4 -->
        <template v-else>
          <div class="w-full h-full rounded-xl p-0.5 grid grid-cols-2 grid-rows-2 gap-0.5 bg-black/30 overflow-hidden">
            <template v-for="j in 4" :key="j">
              <div
                v-if="packsList[3 + j - 1]"
                class="w-full h-full rounded-md overflow-hidden relative"
              >
                <img
                  v-if="getIconSrc(packsList[3 + j - 1]) && !failed[`sub_${packsList[3+j-1].id}`]"
                  :src="getIconSrc(packsList[3 + j - 1])!"
                  class="w-full h-full object-cover select-none"
                  draggable="false"
                  @error="failed[`sub_${packsList[3+j-1].id}`] = true"
                />
                <div
                  v-else
                  class="w-full h-full flex items-center justify-center text-white font-bold text-[8px]"
                  :style="{ background: packGradient(packsList[3 + j - 1].color || packsList[3 + j - 1].name) }"
                >
                  {{ (packsList[3 + j - 1].name || '?')[0].toUpperCase() }}
                </div>
              </div>
              <div v-else class="w-full h-full bg-[var(--input)] rounded-md" />
            </template>
          </div>
        </template>
      </template>
    </div>
  </div>
</template>
