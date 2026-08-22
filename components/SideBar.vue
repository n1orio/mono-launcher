<template>
  <aside
    class="relative flex shrink-0 flex-col bg-[var(--panel)]"
    :class="[sidebarDragging ? '' : 'transition-[width] duration-150', sidebarCollapsed ? 'items-center' : '']"
    :style="{ width: `${sidebarWidth}px` }"
  >
    <!-- Шапка: лого + добавление сборки -->
    <div class="flex items-center gap-2 border-b border-[var(--border)] px-3 py-2.5" :class="sidebarCollapsed ? 'justify-center' : ''">
      <div class="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg bg-[var(--accent)]">
        <svg viewBox="0 0 24 24" class="h-4 w-4 fill-[color:var(--panel)]"><path d="M3 8.4 8.4 3h7.2L21 8.4v7.2L15.6 21H8.4L3 15.6V8.4Zm2 1.3v4.6L8.3 19H9.7l2.5-6.2L14.7 19h1.4L19 14.3V9.7L15.7 5H9.9L5 9.7Z"/></svg>
      </div>
      <span v-if="!sidebarCollapsed" class="min-w-0 flex-1 truncate text-sm font-bold tracking-tight text-[color:var(--tx-strong)]">Mono</span>
      <button
        v-if="!sidebarCollapsed"
        type="button"
        class="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg border border-[var(--border)] bg-[var(--input)] text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[var(--accent)]"
        :title="t('side.createInstance')"
        @click="createPackOpen = true"
      >
        <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M8 0a1 1 0 0 1 1 1v6h6a1 1 0 1 1 0 2H9v6a1 1 0 1 1-2 0V9H1a1 1 0 0 1 0-2h6V1a1 1 0 0 1 1-1Z"/></svg>
      </button>
    </div>

    <!-- Список сборок (инстансов) -->
    <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
      <div v-if="!sidebarCollapsed" class="flex shrink-0 items-center justify-between px-4 pb-1.5 pt-3">
        <span class="text-[10px] font-semibold uppercase tracking-wider text-[color:var(--tx-muted)]">
          {{ t("side.packs") }}
          <span v-if="packs.length" class="ml-1 rounded-full bg-[var(--input)] px-1.5 py-0.5 text-[10px] font-bold tabular-nums text-[color:var(--tx-muted)]">{{ packs.length }}</span>
        </span>
      </div>

      <nav class="min-h-0 flex-1 overflow-y-auto px-2" :class="sidebarCollapsed ? 'pt-2' : 'pb-2'" style="scrollbar-width: thin">
        <template v-if="!sidebarCollapsed">
          <div
            v-for="p in packs"
            :key="p.id"
            class="group relative mb-0.5 flex cursor-pointer items-center gap-2.5 rounded-lg border px-2.5 py-1.5 transition-all duration-150"
            :class="p.id === packId
              ? 'border-[color-mix(in_srgb,var(--accent)_45%,transparent)] bg-gradient-to-r from-[color-mix(in_srgb,var(--accent)_14%,transparent)] to-transparent shadow-sm'
              : 'border-transparent hover:border-[var(--border)] hover:bg-[var(--input-50)]'"
            @click="openPackTab(p.id)"
          >
            <span
              v-if="p.id === packId"
              class="absolute -left-2 top-1/2 h-6 w-[3px] -translate-y-1/2 rounded-r-full bg-[var(--accent)]"
            ></span>
            <div class="relative shrink-0">
              <img
                v-if="p.icon"
                :src="convertFileSrc(p.icon)"
                alt=""
                class="h-9 w-9 rounded-lg border border-[var(--border)] object-cover"
              />
              <div
                v-else
                class="flex h-9 w-9 items-center justify-center rounded-lg border border-[var(--border)] bg-[var(--input)] text-xs font-bold text-[var(--accent)]"
              >
                {{ p.name?.[0]?.toUpperCase() ?? "?" }}
              </div>
              <span
                v-if="p.id === packId"
                class="absolute -bottom-0.5 -right-0.5 h-3 w-3 rounded-full border-2 border-[var(--panel)]"
                :class="status?.installed ? 'bg-[#3fb950]' : 'bg-[var(--tx-muted)]'"
                :title="status?.installed ? t('side.installed') : t('side.notInstalled')"
              ></span>
            </div>
            <div class="min-w-0 flex-1">
              <p class="truncate text-xs font-semibold leading-tight" :class="p.id === packId ? 'text-[color:var(--tx-strong)]' : 'text-[color:var(--tx)]'">{{ p.name }}</p>
              <p class="mt-0.5 truncate text-[11px] leading-tight text-[color:var(--tx-muted)]">
                <span v-if="p.author" class="font-mono text-[var(--accent)]">@{{ p.author }}</span>
                <span v-else>{{ p.kind === "local" ? t("side.createInstance") : "—" }}</span>
              </p>
            </div>

            <!-- Действия строки: появляются при наведении -->
            <div class="absolute right-1.5 flex items-center gap-0.5 rounded-lg bg-[var(--panel)]/95 py-0.5 pl-2.5 pr-0.5 opacity-0 shadow-lg backdrop-blur-sm transition-opacity group-hover:opacity-100">
              <button
                type="button"
                class="flex h-7 w-7 items-center justify-center rounded-md bg-[#238636] text-white shadow-sm transition-all hover:scale-105 hover:bg-[#2ea043] disabled:opacity-50 disabled:hover:scale-100"
                :title="t('side.play')"
                :disabled="busy || gameRunning"
                @click.stop="playFromSidebar(p.id)"
              >
                <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M4.5 1.94a1 1 0 0 1 1.523-.853l9.6 6.06a1 1 0 0 1 0 1.707l-9.6 6.06A1 1 0 0 1 4.5 14.06V1.94Z"/></svg>
              </button>
              <button
                type="button"
                class="flex h-7 w-7 items-center justify-center rounded-md transition-colors disabled:opacity-50"
                :class="removeArmed === p.id ? 'bg-[#f85149]/15 text-[#f85149]' : 'text-[color:var(--tx-muted)] hover:bg-[#f85149]/10 hover:text-[#f85149]'"
                :title="removeArmed === p.id ? t('dev.removeConfirm') : t('dev.remove')"
                :disabled="busy || removingPack === p.id"
                @click.stop="handleRemovePack(p.id)"
              >
                <svg v-if="removingPack === p.id" viewBox="0 0 16 16" class="h-3.5 w-3.5 animate-spin fill-current"><path d="M8 1.5a.75.75 0 0 1 .75.75V8a.75.75 0 0 1-1.5 0V2.25A.75.75 0 0 1 8 1.5Zm3.36 2.14a.75.75 0 0 1 0 1.06 4 4 0 1 1-6.72 0 .75.75 0 0 1 1.06-1.06 2.5 2.5 0 1 0 4.6 0 .75.75 0 0 1 1.06-1.06Z"/></svg>
                <svg v-else viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M6 1.75a.75.75 0 0 1 .75-.75h2.5a.75.75 0 0 1 .75.75V2h3.5a.75.75 0 0 1 0 1.5h-.38l-.89 10.055A1.75 1.75 0 0 1 10.495 15H5.505a1.75 1.75 0 0 1-1.735-1.445L2.88 3.5H2.5a.75.75 0 0 1 0-1.5H6v-.25ZM4.416 3.5l.864 9.9A.25.25 0 0 0 5.525 13.5h4.95a.25.25 0 0 0 .245-.22l.864-9.78H4.416Z"/></svg>
              </button>
            </div>
          </div>
          <p v-if="packs.length === 0" class="px-3 py-4 text-center text-xs leading-relaxed text-[color:var(--tx-muted)]">
            {{ t("side.recentEmpty") }}
          </p>
        </template>

        <!-- Свернутый режим: только иконки -->
        <template v-else>
          <button
            v-for="p in packs"
            :key="p.id"
            type="button"
            class="relative mb-1.5 flex w-full items-center justify-center rounded-xl p-1 transition-colors"
            :class="p.id === packId ? 'bg-[color-mix(in_srgb,var(--accent)_14%,transparent)]' : 'hover:bg-[var(--input-50)]'"
            :title="p.name"
            @click="openPackTab(p.id)"
          >
            <img
              v-if="p.icon"
              :src="convertFileSrc(p.icon)"
              alt=""
              class="h-9 w-9 rounded-lg border border-[var(--border)] object-cover"
            />
            <div
              v-else
              class="flex h-9 w-9 items-center justify-center rounded-lg border border-[var(--border)] bg-[var(--input)] text-xs font-bold text-[var(--accent)]"
            >
              {{ p.name?.[0]?.toUpperCase() ?? "?" }}
            </div>
            <span
              v-if="p.id === packId"
              class="absolute -right-1 top-1/2 h-6 w-[3px] -translate-y-1/2 rounded-full bg-[var(--accent)]"
            ></span>
          </button>
        </template>
      </nav>
    </div>

    <!-- Навигация -->
    <nav class="flex flex-col gap-0.5 border-t border-[var(--border)] p-2">
      <button
        v-for="item in navItems"
        :key="item.id"
        v-show="item.id !== 'admin' || isAdmin"
        type="button"
        class="flex items-center rounded-lg py-2.5 text-[13px] font-semibold transition-colors"
        :class="[
          sidebarCollapsed ? 'justify-center px-1.5' : 'justify-start gap-3 px-3.5',
          tab === item.id ? 'bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] text-[var(--accent)]' : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]',
        ]"
        :title="item.title"
        @click="tab = item.id"
      >
        <svg viewBox="0 0 16 16" class="shrink-0 fill-current" :class="sidebarCollapsed ? 'h-5 w-5' : 'h-[18px] w-[18px]'" v-html="item.icon"></svg>
        <span v-if="!sidebarCollapsed">{{ item.label }}</span>
      </button>
    </nav>

    <div class="flex-1" />

    <!-- Глобальный прогресс установки/скачивания -->
    <div v-if="progress && busy" class="border-t border-[var(--border)] p-3 bg-[var(--panel-soft)]">
      <div class="mb-1 flex items-center justify-between text-xs text-[color:var(--tx-muted)]">
        <span class="truncate pr-2 font-medium text-[color:var(--tx)]">{{ phaseLabel(progress.phase) }}</span>
        <span v-if="progress.fileTotal > 1" class="tabular-nums font-mono text-[11px]">{{ t("progress.files", { n: filesDone, m: progress.fileTotal }) }}</span>
        <span v-else class="tabular-nums font-mono text-[11px]">{{ percent }}%</span>
      </div>
      <div class="h-1.5 w-full overflow-hidden rounded-full bg-[var(--input)]">
        <div
          class="h-full bg-[#2f81f7] transition-all duration-200"
          :style="{ width: `${percent}%` }"
        />
      </div>
      <div class="mt-1 flex items-center justify-between text-[11px] text-[color:var(--tx-muted)]">
        <span class="truncate max-w-[120px]">{{ progress.currentFile || t("side.preparing") }}</span>
        <span class="tabular-nums font-mono">{{ progress.speed > 0 ? `${formatBytes(progress.speed)}${t("units.perSec")}` : "" }}</span>
      </div>
      <div v-if="progress.fileTotal > 1 && filePercent > 0" class="mt-1">
        <div class="h-1 w-full overflow-hidden rounded-full bg-[var(--input)]">
          <div
            class="h-full bg-[color-mix(in_srgb,var(--accent)_60%,transparent)]"
            :style="{ width: `${filePercent}%` }"
          />
        </div>
      </div>
    </div>

    <!-- Учётная запись -->
    <div class="flex items-center gap-2.5 border-t border-[var(--border)] p-3" :class="sidebarCollapsed ? 'justify-center p-2' : ''">
      <div class="flex h-9 w-9 shrink-0 items-center justify-center overflow-hidden rounded-full border border-[var(--border)] bg-[var(--input)] font-mono text-sm font-bold text-[color:var(--tx-strong)]">
        <img v-if="skinUrl" :src="skinUrl" :alt="t('side.skin')" class="h-full w-full object-cover" />
        <template v-else>{{ session?.username?.[0]?.toUpperCase() ?? "?" }}</template>
      </div>
      <div v-if="!sidebarCollapsed" class="min-w-0 flex-1">
        <div class="truncate text-[13px] font-semibold leading-tight text-[color:var(--tx)]">
          {{ session?.username ?? t("side.guest") }}
        </div>
        <div class="truncate text-[11px] leading-tight text-[color:var(--tx-muted)]">
          {{ session ? session.user_type : t("side.offline") }}
        </div>
      </div>
      <button
        v-if="!sidebarCollapsed"
        type="button"
        class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--input-50)] hover:text-[color:var(--tx-strong)]"
        :title="t('nav.settings')"
        @click="tab = 'settings'"
      >
        <svg viewBox="0 0 24 24" class="h-[18px] w-[18px] fill-none stroke-current" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3"></circle>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1Z"></path>
        </svg>
      </button>
    </div>

    <!-- Версия и перевод лаунчера -->
    <div v-if="!sidebarCollapsed" class="flex items-center justify-between gap-2 border-t border-[var(--border)] bg-[var(--panel)] px-3 py-2 text-[10px] text-[var(--tx-muted)]">
      <span class="min-w-0 truncate">
        {{ t("lang.byAuthor") }}
        <span class="font-semibold" :class="activeLocaleAuthor ? 'text-[color:var(--tx)]' : ''">{{ activeLocaleAuthor || "—" }}</span>
        <template v-if="activeLocaleVersion"> · v{{ activeLocaleVersion }}</template>
      </span>
      <span class="shrink-0 tabular-nums font-mono">{{ ram }} {{ t("units.gb") }} · v{{ launcherVer || "?" }}</span>
    </div>

    <!-- Ручка изменения ширины панели -->
    <div
      class="absolute inset-y-0 -right-[3px] z-40 w-[6px] cursor-col-resize transition-colors hover:bg-[var(--accent)] active:bg-[var(--accent-strong)]"
      @pointerdown="startSidebarDrag"
      @pointermove="onSidebarDrag"
      @pointerup="endSidebarDrag"
    ></div>
  </aside>
</template>

<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed } from "vue";
import { useLauncherCtx } from "~/composables/useLauncherContext";
import { useI18n } from "~/composables/useI18n";

const {
  status,
  ram,
  session,
  busy,
  gameRunning,
  progress,
  launcherVer,
  tab,
  packId,
  packs,
  activePack,
  percent,
  filePercent,
  filesDone,
  handleInstall,
  handlePlay,
  selectPack,
  openExternal,
  skinUrl,
  removingPack,
  removeArmed,
  openPackTab,
  handleRemovePack,
  isAdmin,
} = useLauncherCtx();
const { t } = useI18n();

const {
  sidebarWidth,
  sidebarDragging,
  sidebarCollapsed,
  createPackOpen,
  activeLocaleAuthor,
  activeLocaleVersion,
  formatBytes,
  phaseLabel,
  startSidebarDrag,
  onSidebarDrag,
  endSidebarDrag,
} = useLauncherCtx();

/** Запуск сборки прямо из сайдбара: выбрать → играть/установить. */
async function playFromSidebar(id: string) {
  if (busy.value || gameRunning.value) return;
  if (id !== packId.value) await selectPack(id);
  tab.value = "play";
  if (status.value?.installed) {
    await handlePlay();
  } else {
    await handleInstall();
  }
}

/** Иконки навигации (в стиле Modrinth app — крупные, единый стиль). */
const NAV_ICONS: Record<string, string> = {
  news: '<path d="M1.5 3.25A2.25 2.25 0 0 1 3.75 1h8.5A2.25 2.25 0 0 1 14.5 3.25v9.5A2.25 2.25 0 0 1 12.25 15H3.75a2.25 2.25 0 0 1-2.25-2.25v-9.5Zm1.5 0v9.5c0 .414.336.75.75.75h8.5a.75.75 0 0 0 .75-.75v-9.5a.75.75 0 0 0-.75-.75h-8.5a.75.75 0 0 0-.75.75ZM4 5.5A.75.75 0 0 1 4.75 4.75h1.5a.75.75 0 0 1 0 1.5h-1.5A.75.75 0 0 1 4 5.5Zm3.75 0a.75.75 0 0 1 .75-.75h3.75a.75.75 0 0 1 0 1.5H8.5a.75.75 0 0 1-.75-.75ZM4 8.5a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 0 1.5h-1.5A.75.75 0 0 1 4 8.5Zm3.75 0a.75.75 0 0 1 .75-.75h3.75a.75.75 0 0 1 0 1.5H8.5a.75.75 0 0 1-.75-.75Zm-3.75 3a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 0 1.5h-1.5a.75.75 0 0 1-.75-.75Z"/>',
  catalog: '<path d="M1.75 2A1.75 1.75 0 0 0 0 3.75v3.5C0 8.216.784 9 1.75 9h3.5A1.75 1.75 0 0 0 7 7.25v-3.5A1.75 1.75 0 0 0 5.25 2h-3.5Zm0 1.5h3.5c.138 0 .25.112.25.25v3.5c0 .138-.112.25-.25.25h-3.5a.25.25 0 0 1-.25-.25v-3.5c0-.138.112-.25.25-.25ZM10.75 2A1.75 1.75 0 0 0 9 3.75v3.5c0 .966.784 1.75 1.75 1.75h3.5A1.75 1.75 0 0 0 16 7.25v-3.5A1.75 1.75 0 0 0 14.25 2h-3.5Zm0 1.5h3.5c.138 0 .25.112.25.25v3.5c0 .138-.112.25-.25.25h-3.5a.25.25 0 0 1-.25-.25v-3.5c0-.138.112-.25.25-.25ZM1.75 10A1.75 1.75 0 0 0 0 11.75v.5C0 13.216.784 14 1.75 14h3.5A1.75 1.75 0 0 0 7 12.25v-.5A1.75 1.75 0 0 0 5.25 10h-3.5Zm0 1.5h3.5c.138 0 .25.112.25.25v.5c0 .138-.112.25-.25.25h-3.5a.25.25 0 0 1-.25-.25v-.5c0-.138.112-.25.25-.25ZM10.75 10A1.75 1.75 0 0 0 9 11.75v.5c0 .966.784 1.75 1.75 1.75h3.5A1.75 1.75 0 0 0 16 12.25v-.5A1.75 1.75 0 0 0 14.25 10h-3.5Zm0 1.5h3.5c.138 0 .25.112.25.25v.5c0 .138-.112.25-.25.25h-3.5a.25.25 0 0 1-.25-.25v-.5c0-.138.112-.25.25-.25Z"/>',
  library: '<path d="M0 1.75A.75.75 0 0 1 .75 1h4.253c1.227 0 2.317.59 3 1.501A3.744 3.744 0 0 1 11.006 1h4.245a.75.75 0 0 1 .75.75v10.5a.75.75 0 0 1-.75.75h-4.507a2.25 2.25 0 0 0-1.591.659l-.622.621a.75.75 0 0 1-1.06 0l-.622-.621A2.25 2.25 0 0 0 5.258 13H.75a.75.75 0 0 1-.75-.75Zm7.251 10.324.004-5.073-.002-2.253A2.25 2.25 0 0 0 5.003 2.5H1.5v9h3.757a3.75 3.75 0 0 1 1.994.574ZM8.755 4.846V7.06h7.745V2.5h-3.496a2.249 2.249 0 0 0-2.24 2.236l-.009.11Zm-.001 7.003a3.752 3.752 0 0 1 2.003-.575H14.5v-9h-3.495a2.249 2.249 0 0 0-2.24 2.236l-.009.111-.001 5.228Z"/>',
  settings: '<path d="M8 0a8 8 0 0 1 .8 15.96l-.4-1.98A6 6 0 1 0 2.7 6.5L1 5.6A8 8 0 0 1 8 0Zm0 3.5a4.5 4.5 0 0 1 4.47 4H15A7 7 0 0 0 1.2 7.9l1.9.62A4.5 4.5 0 0 1 8 3.5ZM6.2 6.2 9.8 9.8A2.5 2.5 0 1 0 6.2 6.2Z"/>',
  author: '<path d="M8 1a3 3 0 1 0 0 6 3 3 0 0 0 0-6ZM2 13.25C2 10.75 4.46 9.25 8 9.25s6 1.5 6 4V14a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1v-.75Z"/>',
  admin: '<path d="M8 1.5 9.3 4.4l3.2.35-2.4 2.15.7 3.1L8 8.55l-2.8 1.45.7-3.1L3.5 4.75l3.2-.35ZM2.5 11.5a.75.75 0 0 1 .75-.75h2a.75.75 0 0 1 0 1.5h-2a.75.75 0 0 1-.75-.75Zm7.5 0a.75.75 0 0 1 .75-.75h2a.75.75 0 0 1 0 1.5h-2a.75.75 0 0 1-.75-.75ZM4.25 13.75a.75.75 0 0 0 0 1.5h7.5a.75.75 0 0 0 0-1.5Z"/>',
  dev: '<path d="M2 1.75C2 .784 2.784 0 3.75 0h8.5C13.216 0 14 .784 14 1.75v12.5A1.75 1.75 0 0 1 12.25 16h-8.5A1.75 1.75 0 0 1 2 14.25Zm1.69 1.884a.75.75 0 0 1 .79.075l4.244 3.253a.75.75 0 0 1 0 1.13L4.48 11.345a.75.75 0 0 1-.79.075.75.75 0 0 1-.388-.67v-6.5a.75.75 0 0 1 .388-.547ZM10.5 8.75h3a.75.75 0 0 0 0-1.5h-3a.75.75 0 0 0 0 1.5Z"/>',
};

const navItems = computed(() => [
  { id: "news", label: t("nav.news"), title: t("nav.news"), icon: NAV_ICONS.news! },
  { id: "catalog", label: t("nav.catalog"), title: t("nav.catalog"), icon: NAV_ICONS.catalog! },
  { id: "library", label: t("nav.library"), title: t("nav.library"), icon: NAV_ICONS.library! },
  { id: "author", label: t("nav.author"), title: t("nav.author"), icon: NAV_ICONS.author! },
  { id: "admin", label: t("nav.admin"), title: t("nav.admin"), icon: NAV_ICONS.admin! },
  { id: "settings", label: t("nav.settings"), title: t("nav.settings"), icon: NAV_ICONS.settings! },
  { id: "dev", label: t("side.dev"), title: t("side.dev"), icon: NAV_ICONS.dev! },
]);
</script>
