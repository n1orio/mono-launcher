<script setup lang="ts">
import type { CurseProjectDetail, CurseSearchHit } from "~/lib/bridge";
import { useI18n } from "#imports";

type CurseProject = CurseProjectDetail | CurseSearchHit;

interface Props {
  project: CurseProject;
  selected?: boolean;
  installed?: boolean;
  loading?: boolean;
  compact?: boolean;
  onSelect?: (projectId: number) => void;
  onInstall?: (project: CurseProject, e?: Event) => void;
  onOpenDetail?: (project: CurseProject) => void;
}

const props = withDefaults(defineProps<Props>(), {
  selected: false,
  installed: false,
  loading: false,
  compact: false,
});

const emit = defineEmits<{
  select: [projectId: number];
  install: [project: CurseProject, e?: Event];
  openDetail: [project: CurseProject];
}>();

const i18n = useI18n();
const t = i18n.t;

const iconSvg = {
  color: "#F16436",
  path: "M18.326 9.2145S23.2261 8.4418 24 6.1882h-7.5066V4.4H0l2.0318 2.3576V9.173s5.1267-.2665 7.1098 1.2372c2.7146 2.516-3.053 5.917-3.053 5.917L5.0995 19.6c1.5465-1.4726 4.494-3.3775 9.8983-3.2857-2.0565.65-4.1245 1.6651-5.7344 3.2857h10.9248l-1.0288-3.2726s-7.918-4.6688-.8336-7.1127z",
};

function handleSelect() {
  emit("select", props.project.projectId);
  props.onSelect?.(props.project.projectId);
}

function handleInstall(e: MouseEvent) {
  e.stopPropagation();
  emit("install", props.project);
  props.onInstall?.(props.project, e);
}

function handleOpenDetail(e: MouseEvent) {
  e.stopPropagation();
  emit("openDetail", props.project);
  props.onOpenDetail?.(props.project);
}
</script>

<template>
  <div
    :class="[
      'flex items-start gap-4 rounded-xl bg-[var(--panel)] shadow-sm transition-colors',
      'hover:bg-[var(--hover)]',
      compact ? 'px-3 py-2' : 'px-4 py-3',
      props.selected ? 'ring-2 ring-[var(--accent)]' : '',
    ]"
    @click="handleOpenDetail"
  >
    <!-- Selection checkbox (non-compact only) -->
    <button
      v-if="!compact"
      type="button"
      class="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-md transition-colors"
      :class="props.selected ? 'bg-[color-mix(in_srgb,var(--accent)_20%,transparent)]' : ''"
      :title="props.selected ? 'Отменить выбор' : 'Выбрать для скачивания'"
      @click.stop="handleSelect"
    >
      <svg v-if="props.selected" viewBox="0 0 16 16" class="h-3 w-3 fill-[var(--accent)]">
        <path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z" />
      </svg>
    </button>

    <!-- Icon -->
    <img
      v-if="project.iconUrl"
      :src="project.iconUrl"
      :alt="project.name"
      loading="lazy"
      :class="[
        'shrink-0 rounded-xl object-cover',
        compact ? 'h-10 w-10' : 'h-14 w-14',
      ]"
    />
    <div
      v-else
      :class="[
        'flex shrink-0 items-center justify-center rounded-xl bg-[var(--input-50)] text-sm text-[color:var(--tx-muted)]',
        compact ? 'h-10 w-10' : 'h-14 w-14',
      ]"
    >
      {{ project.name.slice(0, 2).toUpperCase() }}
    </div>

    <!-- Content -->
    <div class="min-w-0 flex-1" :class="compact ? 'pr-2' : ''">
      <div class="flex flex-wrap items-center gap-x-2">
        <svg
          viewBox="0 0 24 24"
          :class="['shrink-0 self-center', compact ? 'h-3 w-3' : 'h-3.5 w-3.5']"
          :title="t('mods.serviceCurseforge')"
        >
          <path :fill="iconSvg.color" :d="iconSvg.path" />
        </svg>
        <span :class="['truncate font-semibold', compact ? 'text-xs' : 'text-sm']">
          {{ project.name }}
        </span>
        <span v-if="project.author" class="text-xs text-[color:var(--tx-muted)]">
          {{ t("mods.byAuthor", { author: project.author }) }}
        </span>
      </div>
      <!-- Полное описание (description) или summary если description отсутствует -->
      <p
        v-if="!compact && (project.description || project.summary)"
        class="mt-1 line-clamp-2 text-[13px] leading-snug text-[color:var(--tx-muted)]"
      >
        {{ project.description || project.summary }}
      </p>
      <div v-if="!compact" class="mt-1.5 flex items-center gap-1 text-xs text-[color:var(--tx-muted)]">
        <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
          <path d="M1.75 1.75a.75.75 0 0 0-1.5 0v9A2.25 2.25 0 0 0 2.5 13h12.75a.75.75 0 0 0 0-1.5H2.5a.75.75 0 0 1-.75-.75v-9Zm10.75 2.5a.75.75 0 0 0-1.5 0v5a.75.75 0 0 0 1.5 0v-5Zm-3 .75a.75.75 0 0 1 1.5 0v4.25a.75.75 0 0 1-1.5 0V5Zm-3 1.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3Z" />
        </svg>
        {{ project.downloadCount.toLocaleString() }}
      </div>
    </div>

    <!-- Action button (only shown when onInstall is provided) -->
    <button
      v-if="props.onInstall"
      type="button"
      class="flex shrink-0 items-center gap-1.5 self-center rounded-lg transition-colors disabled:opacity-50"
      :class="[
        'bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] text-[var(--accent)] font-semibold',
        compact ? 'px-2 py-1.5 text-xs' : 'px-3 py-2 text-sm',
      ]"
      :disabled="props.loading || props.installed"
      @click.stop="handleInstall"
    >
      <svg
        v-if="props.loading"
        viewBox="0 0 16 16"
        :class="['animate-spin fill-current', compact ? 'h-3 w-3' : 'h-3 w-3']"
      >
        <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z" />
      </svg>
      <svg
        v-else-if="props.installed"
        viewBox="0 0 16 16"
        :class="['fill-current', compact ? 'h-3 w-3' : 'h-3 w-3']"
      >
        <path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z" />
      </svg>
      <svg
        v-else
        viewBox="0 0 16 16"
        :class="['fill-current', compact ? 'h-3 w-3' : 'h-3 w-3']"
      >
        <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z" />
      </svg>
      <span v-if="!compact">
        {{ props.installed ? t("mods.installedBadge") : t("mods.download") }}
      </span>
    </button>

    <!-- Chevron for non-compact -->
    <svg
      v-if="!compact"
      viewBox="0 0 16 16"
      class="mt-1 h-4 w-4 shrink-0 fill-[var(--tx-muted)]"
    >
      <path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z" />
    </svg>
  </div>
</template>