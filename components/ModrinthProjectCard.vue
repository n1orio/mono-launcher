<script setup lang="ts">
import type { ModrinthProject } from "~/lib/types";
import { useI18n } from "#imports";

interface Props {
  project: ModrinthProject;
  service: "modrinth" | "curseforge";
  selected?: boolean;
  installed?: boolean;
  loading?: boolean;
  compact?: boolean;
  onSelect?: (projectId: string) => void;
  onInstall?: (project: ModrinthProject, e?: Event) => void;
  onOpenDetail?: (project: ModrinthProject) => void;
}

const props = withDefaults(defineProps<Props>(), {
  service: "modrinth",
  selected: false,
  installed: false,
  loading: false,
  compact: false,
});

const emit = defineEmits<{
  select: [projectId: string];
  install: [project: ModrinthProject, e?: Event];
  openDetail: [project: ModrinthProject];
}>();

const i18n = useI18n();
const t = i18n.t;

function handleSelect() {
  emit("select", props.project.projectId);
  props.onSelect?.(props.project.projectId);
}

function handleInstall(e: MouseEvent) {
  e.stopPropagation();
  emit("install", props.project);
  props.onInstall?.(props.project);
}

function handleOpenDetail(e: MouseEvent) {
  e.stopPropagation();
  emit("openDetail", props.project);
  props.onOpenDetail?.(props.project);
}

const iconSvg = computed(() => {
  if (props.service === "modrinth") {
    return {
      color: "#00AF5C",
      path: "M12.252.004a11.78 11.768 0 0 0-8.92 3.73 11 10.999 0 0 0-2.17 3.11 11.37 11.359 0 0 0-1.16 5.169c0 1.42.17 2.5.6 3.77.24.759.77 1.899 1.17 2.529a12.3 12.298 0 0 0 8.85 5.639c.44.05 2.54.07 2.76.02.2-.04.22.1-.26-1.7l-.36-1.37-1.01-.06a8.5 8.489 0 0 1-5.18-1.8 5.34 5.34 0 0 1-1.3-1.26c0-.05.34-.28.74-.5a37.572 37.545 0 0 1 2.88-1.629c.03 0 .5.45 1.06.98l1 .97 2.07-.43 2.06-.43 1.47-1.47c.8-.8 1.48-1.5 1.48-1.52 0-.09-.42-1.63-.46-1.7-.04-.06-.2-.03-1.02.18-.53.13-1.2.3-1.45.4l-.48.15-.53.53-.53.53-.93.1-.93.07-.52-.5a2.7 2.7 0 0 1-.96-1.7l-.13-.6.43-.57c.68-.9.68-.9 1.46-1.1.4-.1.65-.2.83-.33.13-.099.65-.579 1.14-1.069l.9-.9-.7-.7-.7-.7-1.95.54c-1.07.3-1.96.53-1.97.53-.03 0-2.23 2.48-2.63 2.97l-.29.35.28 1.03c.16.56.3 1.16.31 1.34l.03.3-.34.23c-.37.23-2.22 1.3-2.84 1.63-.36.2-.37.2-.44.1-.08-.1-.23-.6-.32-1.03-.18-.86-.17-2.75.02-3.73a8.84 8.839 0 0 1 7.9-6.93c.43-.03.77-.08.78-.1.06-.17.5-2.999.47-3.039-.01-.02-.1-.02-.2-.03Zm3.68.67c-.2 0-.3.1-.37.38-.06.23-.46 2.42-.46 2.52 0 .04.1.11.22.16a8.51 8.499 0 0 1 2.99 2 8.38 8.379 0 0 1 2.16 3.449 6.9 6.9 0 0 1 .4 2.8c0 1.07 0 1.27-.1 1.73a9.37 9.369 0 0 1-1.76 3.769c-.32.4-.98 1.06-1.37 1.38-.38.32-1.54 1.1-1.7 1.14-.1.03-.1.06-.07.26.03.18.64 2.56.7 2.78l.06.06a12.07 12.058 0 0 0 7.27-9.4c.13-.77.13-2.58 0-3.4a11.96 11.948 0 0 0-5.73-8.578c-.7-.42-2.05-1.06-2.25-1.06Z",
    };
  }
  return {
    color: "#F16436",
    path: "M18.326 9.2145S23.2261 8.4418 24 6.1882h-7.5066V4.4H0l2.0318 2.3576V9.173s5.1267-.2665 7.1098 1.2372c2.7146 2.516-3.053 5.917-3.053 5.917L5.0995 19.6c1.5465-1.4726 4.494-3.3775 9.8983-3.2857-2.0565.65-4.1245 1.6651-5.7344 3.2857h10.9248l-1.0288-3.2726s-7.918-4.6688-.8336-7.1127z",
  };
});
</script>

<template>
  <div
    :class="[
      'flex items-start gap-4 rounded-xl bg-[var(--panel)] shadow-sm transition-colors',
      'hover:bg-[var(--hover)]',
      compact ? 'px-3 py-2' : 'px-4 py-3',
      selected ? 'ring-2 ring-[var(--accent)]' : '',
    ]"
    @click="handleOpenDetail"
  >
    <!-- Selection checkbox (non-compact only) -->
    <button
      v-if="!compact"
      type="button"
      class="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-md transition-colors"
      :class="selected ? 'bg-[color-mix(in_srgb,var(--accent)_20%,transparent)]' : ''"
      :title="selected ? 'Отменить выбор' : 'Выбрать для скачивания'"
      @click.stop="handleSelect"
    >
      <AppIcon v-if="selected" name="check" class="h-3 w-3 fill-[var(--accent)]" />
    </button>

    <!-- Icon -->
    <img
      v-if="project.iconUrl"
      :src="project.iconUrl"
      :alt="project.title"
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
      {{ project.title.slice(0, 2).toUpperCase() }}
    </div>

    <!-- Content -->
    <div class="min-w-0 flex-1" :class="compact ? 'pr-2' : ''">
      <div class="flex flex-wrap items-center gap-x-2">
        <svg
          viewBox="0 0 24 24"
          :class="['shrink-0 self-center', compact ? 'h-3 w-3' : 'h-3.5 w-3.5']"
          :title="service === 'modrinth' ? 'Modrinth' : 'CurseForge'"
        >
          <path :fill="iconSvg.color" :d="iconSvg.path" />
        </svg>
        <span :class="['truncate font-semibold', compact ? 'text-xs' : 'text-sm']">
          {{ project.title }}
        </span>
        <span v-if="project.author" class="text-xs text-[color:var(--tx-muted)]">
          {{ t("mods.byAuthor", { author: project.author }) }}
        </span>
      </div>
      <p
        v-if="!compact"
        class="mt-1 line-clamp-2 text-[13px] leading-snug text-[color:var(--tx-muted)]"
      >
        {{ project.description }}
      </p>
      <div v-if="!compact" class="mt-1.5 flex items-center gap-1 text-xs text-[color:var(--tx-muted)]">
        <AppIcon name="download-bars" class="h-3 w-3 fill-current" />
        {{ project.downloads.toLocaleString() }}
      </div>
    </div>

    <!-- Action button -->
    <button
      type="button"
      class="flex shrink-0 items-center gap-1.5 self-center rounded-lg transition-colors disabled:opacity-50"
      :class="[
        'bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] text-[var(--accent)] font-semibold',
        compact ? 'px-2 py-1.5 text-xs' : 'px-3 py-2 text-sm',
      ]"
      :disabled="loading || installed"
      @click.stop="handleInstall"
    >
      <AppIcon
        v-if="loading"
        name="spinner"
        :class="'fill-current ' + (compact ? 'h-3 w-3' : 'h-3 w-3')"
      />
      <AppIcon
        v-else-if="installed"
        name="check"
        :class="'fill-current ' + (compact ? 'h-3 w-3' : 'h-3 w-3')"
      />
      <AppIcon
        v-else
        name="arrow-down"
        :class="'fill-current ' + (compact ? 'h-3 w-3' : 'h-3 w-3')"
      />
      <span v-if="!compact">
        {{ installed ? t("mods.installedBadge") : t("mods.download") }}
      </span>
    </button>

    <!-- Chevron for non-compact -->
    <AppIcon
      v-if="!compact"
      name="chevron-right"
      class="mt-1 h-4 w-4 shrink-0 fill-[var(--tx-muted)]"
    />
  </div>
</template>