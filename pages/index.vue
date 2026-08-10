<template>
  <div class="flex h-full w-full select-none overflow-hidden bg-[#0d1117] text-[#c9d1d9] font-sans">
    <!-- Уведомления (тосты) -->
    <div class="pointer-events-none fixed right-4 top-4 z-50 flex w-80 max-w-[calc(100vw-2rem)] flex-col gap-2">
      <TransitionGroup name="toast">
        <div
          v-for="n in notifications"
          :key="n.id"
          class="pointer-events-auto flex items-start gap-2.5 rounded-md border bg-[#161b22] px-3.5 py-2.5 text-xs shadow-lg shadow-black/40"
          :class="{
            'border-[#f85149]/50': n.type === 'error',
            'border-[#1f6beb]/50': n.type === 'info',
            'border-[#238636]/50': n.type === 'success',
          }"
        >
          <svg
            viewBox="0 0 16 16"
            class="mt-0.5 h-3.5 w-3.5 shrink-0 fill-current"
            :class="{
              'text-[#f85149]': n.type === 'error',
              'text-[#58a6ff]': n.type === 'info',
              'text-[#3fb950]': n.type === 'success',
            }"
          >
            <path v-if="n.type === 'error'" d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM4.97 4.97a.749.749 0 0 0-1.06 1.06L6.94 8l-3.03 3.03a.749.749 0 1 0 1.06 1.06L8 9.06l3.03 3.03a.749.749 0 1 0 1.06-1.06L9.06 8l3.03-3.03a.749.749 0 0 0-1.06-1.06L8 6.94Z"/>
            <path v-else-if="n.type === 'info'" d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM7.25 3.5a.75.75 0 0 0 0 1.5h.008a.75.75 0 0 0 0-1.5ZM7 7.25a.75.75 0 0 0 0 1.5h.25V12H7a.75.75 0 0 0 0 1.5h.75a.75.75 0 0 0 .75-.75v-5.5A.75.75 0 0 0 7.5 6.5H7a.75.75 0 0 0 0 .75Z"/>
            <path v-else d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0Zm3.03 5.03a.75.75 0 0 0-1.06-1.06L6.5 7.44l-1.47-1.47a.75.75 0 0 0-1.06 1.06l2 2a.75.75 0 0 0 1.06 0Z"/>
          </svg>
          <p class="min-w-0 break-words leading-relaxed text-[#c9d1d9]">{{ n.text }}</p>
          <div class="ml-auto flex shrink-0 items-center gap-1.5">
            <button
              v-if="n.reportable"
              type="button"
              class="flex items-center gap-1 rounded border border-[#f85149]/40 bg-[#f85149]/10 px-2 py-0.5 text-[10px] font-semibold text-[#f85149] transition-colors hover:bg-[#f85149]/20"
              :title="t('toast.report')"
              @click="reportError(n.text)"
            >
              <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8Z"/>
              </svg>
              GitHub Issue
            </button>
            <button
              type="button"
              class="shrink-0 text-[#8b949e] transition-colors hover:text-[#f0f6fc]"
              :title="t('toast.close')"
              @click="dismissNotification(n.id)"
            >
              <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                <path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.749.749 0 0 1 1.06 1.06L9.06 8l3.22 3.22a.749.749 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.749.749 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.749.749 0 0 1 0-1.06Z"/>
              </svg>
            </button>
          </div>
        </div>
      </TransitionGroup>
    </div>
    <!-- Карточка обновления лаунчера -->
    <div
      v-if="appUpdate && !appUpdating"
      class="fixed bottom-4 right-4 z-40 w-80 max-w-[calc(100vw-2rem)] rounded-md border border-[#1f6beb]/50 bg-[#161b22] p-3.5 shadow-lg shadow-black/40"
    >
      <div class="flex items-start gap-2.5">
        <svg viewBox="0 0 16 16" class="mt-0.5 h-4 w-4 shrink-0 fill-[#58a6ff]">
          <path d="M8 1.5a.75.75 0 0 1 .75.75V2.5H14a1 1 0 0 1 1 1v2.75A1.75 1.75 0 0 1 13.25 8H8.75v5.75a1.75 1.75 0 0 1-3.5 0V8H2A1.75 1.75 0 0 1 .25 6.25V3.5a1 1 0 0 1 1-1h5.25v-.25A.75.75 0 0 1 8 1.5Z"/>
        </svg>
        <div class="min-w-0 flex-1">
          <div class="text-xs font-semibold text-[#f0f6fc]">
            {{ t("appUpdate.title") }}
          </div>
          <div class="mt-0.5 truncate text-[11px] text-[#8b949e]">
            {{ t("appUpdate.version", { v: appUpdate.version }) }}
          </div>
          <p v-if="appUpdate.notes" class="mt-1 max-h-12 overflow-hidden text-[11px] leading-snug text-[#c9d1d9]">
            {{ appUpdate.notes.slice(0, 180) }}{{ appUpdate.notes.length > 180 ? "…" : "" }}
          </p>
          <button
            type="button"
            class="mt-2.5 w-full rounded-md bg-[#1f6beb] px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[#388bfd]"
            @click="installAppUpdate"
          >
            {{ t("appUpdate.install") }}
          </button>
        </div>
      </div>
    </div>
    <!-- Прогресс обновления лаунчера -->
    <div
      v-if="appUpdating"
      class="fixed bottom-4 right-4 z-40 w-80 max-w-[calc(100vw-2rem)] rounded-md border border-[#1f6beb]/50 bg-[#161b22] p-3.5 shadow-lg shadow-black/40"
    >
      <div class="mb-1.5 flex items-center justify-between text-[11px]">
        <span class="font-medium text-[#c9d1d9]">{{ t("appUpdate.progress") }}</span>
        <span class="tabular-nums font-mono text-[10px] text-[#8b949e]">
          {{ appUpdateProgress ?? 0 }}%
        </span>
      </div>
      <div class="h-1.5 w-full overflow-hidden rounded-full bg-[#21262d]">
        <div
          class="h-full bg-[#2f81f7] transition-all duration-200"
          :style="{ width: `${appUpdateProgress ?? 0}%` }"
        />
      </div>
      <div class="mt-1.5 text-[10px] text-[#8b949e]">
        {{ t("appUpdate.restart") }}
      </div>
    </div>
    <!-- ==== Боковая панель ==== -->
    <aside class="flex w-64 shrink-0 flex-col border-r border-[#30363d] bg-[#161b22]">
      <!-- Выбор сборки (стилизован под репозиторий GitHub) -->
      <div class="p-3.5 border-b border-[#30363d]">
        <label class="mb-1.5 block text-[11px] font-semibold uppercase tracking-wider text-[#8b949e]">
          {{ t("side.packRepo") }}
        </label>
        <div class="relative">
          <select
            :value="packId"
            :disabled="busy"
            @change="onPackChange"
            class="w-full appearance-none rounded-md border border-[#30363d] bg-[#21262d] px-3 py-1.5 pr-8 text-xs font-medium text-[#c9d1d9] transition-colors hover:bg-[#30363d] focus:border-[#58a6ff] focus:outline-none disabled:opacity-50"
          >
            <option v-for="p in packs" :key="p.id" :value="p.id">
              {{ p.name }}
            </option>
          </select>
          <svg viewBox="0 0 16 16" class="pointer-events-none absolute right-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 fill-[#8b949e]">
            <path d="m4.427 6.427 3.396 3.396a.25.25 0 0 0 .354 0l3.396-3.396A.25.25 0 0 0 11.396 6H4.604a.25.25 0 0 0-.177.427Z"/>
          </svg>
        </div>
        <div class="mt-2 flex items-center gap-1.5 text-[11px] text-[#8b949e]">
          <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-[#8b949e] shrink-0">
            <path d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-1 1v.878A2.25 2.25 0 1 1 2 13.378V2.5Z"/>
          </svg>
          <span class="truncate font-mono">{{ activePack?.name ?? t("side.notSelected") }}</span>
        </div>
      </div>

      <!-- Навигация -->
      <nav class="flex flex-col gap-0.5 p-2 border-b border-[#30363d]">
        <button
          type="button"
          class="flex items-center gap-2 rounded-md px-3 py-1.5 text-xs font-medium transition-colors"
          :class="tab === 'play' ? 'bg-[#21262d] text-[#f0f6fc]' : 'text-[#8b949e] hover:bg-[#21262d]/50 hover:text-[#c9d1d9]'"
          @click="tab = 'play'"
        >
          <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current">
            <path d="M0 1.75C0 .784.784 0 1.75 0h12.5C15.216 0 16 .784 16 1.75v12.5A1.75 1.75 0 0 1 14.25 16H1.75A1.75 1.75 0 0 1 0 14.25Zm1.75-.25a.25.25 0 0 0-.25.25v12.5c0 .138.112.25.25.25h12.5a.25.25 0 0 0 .25-.25V1.75a.25.25 0 0 0-.25-.25Z"/>
            <path d="M6.25 4.25a.75.75 0 0 1 1.15-.632l4.5 3a.75.75 0 0 1 0 1.264l-4.5 3A.75.75 0 0 1 6.25 10.25Z"/>
          </svg>
          {{ t("nav.releases") }}
        </button>
        <button
          type="button"
          class="flex items-center gap-2 rounded-md px-3 py-1.5 text-xs font-medium transition-colors"
          :class="tab === 'news' ? 'bg-[#21262d] text-[#f0f6fc]' : 'text-[#8b949e] hover:bg-[#21262d]/50 hover:text-[#c9d1d9]'"
          @click="tab = 'news'"
        >
          <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current">
            <path d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-1 1v.878A2.25 2.25 0 1 1 2 13.378V2.5Z"/>
            <path d="M5.25 3.75a.75.75 0 0 1 .75-.75h4.5a.75.75 0 0 1 0 1.5h-4.5a.75.75 0 0 1-.75-.75Zm0 3a.75.75 0 0 1 .75-.75h4.5a.75.75 0 0 1 0 1.5h-4.5a.75.75 0 0 1-.75-.75Z"/>
          </svg>
          {{ t("nav.news") }}
        </button>
        <button
          type="button"
          class="flex items-center gap-2 rounded-md px-3 py-1.5 text-xs font-medium transition-colors"
          :class="tab === 'settings' ? 'bg-[#21262d] text-[#f0f6fc]' : 'text-[#8b949e] hover:bg-[#21262d]/50 hover:text-[#c9d1d9]'"
          @click="tab = 'settings'"
        >
          <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current">
            <path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM1.5 8a6.5 6.5 0 1 0 13 0 6.5 6.5 0 0 0-13 0Zm7-3.25a.75.75 0 0 0-1.5 0v3.25H4.25a.75.75 0 0 0 0 1.5h3.5a.75.75 0 0 0 .75-.75V4.75Z"/>
          </svg>
          {{ t("nav.settings") }}
        </button>
      </nav>

      <!-- Сводка статуса -->
      <div class="space-y-2 p-3.5 text-xs text-[#8b949e]">
        <div class="flex items-center justify-between">
          <span>{{ t("side.status") }}</span>
          <span class="inline-flex items-center gap-1.5 font-medium">
            <span class="h-2 w-2 rounded-full" :class="status?.installed ? 'bg-[#3fb950]' : 'bg-[#8b949e]'"></span>
            <span :class="status?.installed ? 'text-[#f0f6fc]' : 'text-[#8b949e]'">
              {{ status?.installed ? t("side.installed") : t("side.notInstalled") }}
            </span>
          </span>
        </div>
        <div class="flex items-center justify-between">
          <span>{{ t("side.version") }}</span>
          <span class="font-mono font-medium text-[#c9d1d9] truncate max-w-[110px]" :title="status?.active_version ? `versionId: ${status.active_version}` : undefined">
            {{ status?.active_source_tag ?? status?.active_version ?? "—" }}
          </span>
        </div>
        <div class="flex items-center justify-between">
          <span>{{ t("side.memory") }}</span>
          <span class="font-mono font-medium text-[#c9d1d9]">{{ ram }} {{ t("units.gb") }}</span>
        </div>
      </div>

      <!-- Глобальный прогресс установки/скачивания -->
      <div v-if="progress && busy" class="border-t border-[#30363d] p-3 bg-[#0d1117]/50">
        <div class="mb-1 flex items-center justify-between text-[11px] text-[#8b949e]">
          <span class="truncate pr-2 font-medium text-[#c9d1d9]">{{ phaseLabel(progress.phase) }}</span>
          <span class="tabular-nums font-mono text-[10px]">{{ percent }}%</span>
        </div>
        <div class="h-1.5 w-full overflow-hidden rounded-full bg-[#21262d]">
          <div
            class="h-full bg-[#2f81f7] transition-all duration-200"
            :style="{ width: `${percent}%` }"
          />
        </div>
        <div class="mt-1 flex items-center justify-between text-[10px] text-[#8b949e]">
          <span class="truncate max-w-[120px]">{{ progress.currentFile || t("side.preparing") }}</span>
          <span class="tabular-nums font-mono">{{ progress.speed > 0 ? `${formatBytes(progress.speed)}${t("units.perSec")}` : "" }}</span>
        </div>
      </div>

      <div class="flex-1" />

      <!-- Учётная запись -->
      <div class="flex items-center gap-2.5 border-t border-[#30363d] p-3 bg-[#0d1117]/30">
        <div class="flex h-7 w-7 shrink-0 items-center justify-center overflow-hidden rounded-full border border-[#30363d] bg-[#21262d] font-mono text-xs font-bold text-[#f0f6fc]">
          <img v-if="skinUrl" :src="skinUrl" :alt="t('side.skin')" class="h-full w-full object-cover" />
          <template v-else>{{ session?.username?.[0]?.toUpperCase() ?? "?" }}</template>
        </div>
        <div class="min-w-0 flex-1">
          <div class="truncate text-xs font-medium text-[#c9d1d9]">
            {{ session?.username ?? t("side.guest") }}
          </div>
          <div class="truncate text-[10px] text-[#8b949e]">
            {{ session ? session.user_type : t("side.offline") }}
          </div>
        </div>
      </div>

      <!-- Главное действие (Кнопка запуска) -->
      <div class="p-3 border-t border-[#30363d] bg-[#161b22]">
        <button
          type="button"
          class="w-full rounded-md py-2 px-3 text-xs font-semibold text-white shadow-sm transition-all focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
          :class="status?.installed
            ? 'bg-[#238636] hover:bg-[#2ea043] focus-visible:outline-[#2ea043]'
            : 'bg-[#1f6beb] hover:bg-[#388bfd] focus-visible:outline-[#388bfd]'"
          :disabled="busy"
          @click="status?.installed ? handlePlay() : handleInstall()"
        >
          <template v-if="!status?.installed">
            {{ busy ? t("side.installing") : t("side.downloadPlay") }}
          </template>
          <template v-else>
            {{ busy ? t("side.launching") : t("side.play") }}
          </template>
        </button>
      </div>
    </aside>

    <!-- ==== Основной контент ==== -->
    <main class="relative flex-1 overflow-hidden bg-[#0d1117]">
      <div class="mx-auto flex h-full w-full max-w-4xl flex-col px-8 py-6">
        <!-- ======= Вкладка: Релизы ======= -->
        <template v-if="tab === 'play'">
          <div class="flex min-h-0 flex-1 flex-col">
          <!-- Header сборки -->
          <div class="mb-6 shrink-0 border-b border-[#30363d] pb-5">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <svg viewBox="0 0 16 16" class="h-5 w-5 fill-[#8b949e]">
                  <path d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-1 1v.878A2.25 2.25 0 1 1 2 13.378V2.5Z"/>
                </svg>
                <h1 class="text-xl font-semibold text-[#f0f6fc]">
                  {{ activePack?.name ?? t("pack.none") }}
                </h1>
                                <span
                  class="ml-2 rounded-full px-2 py-0.5 text-[11px] font-medium border"
                  :class="status?.installed
                    ? 'border-[#238636]/40 bg-[#238636]/10 text-[#3fb950]'
                    : 'border-[#30363d] bg-[#21262d] text-[#8b949e]'"
                >
                  {{ status?.installed ? t("pack.installed") : t("pack.notInstalled") }}
                </span>
                <button
                  type="button"
                  class="ml-1 flex items-center gap-1.5 rounded-md border border-[#30363d] bg-[#21262d] px-2.5 py-1 text-[11px] font-medium text-[#8b949e] transition-colors hover:bg-[#30363d] hover:text-[#c9d1d9]"
                  :title="t('pack.openDir')"
                  @click="handleOpenPackDir"
                >
                  <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
                    <path d="M.513 1.513A1.75 1.75 0 0 1 1.75 1h3.5c.55 0 1.07.26 1.4.7l.9 1.2a.25.25 0 0 0 .2.1H13a1 1 0 0 1 1 1v.5H2.75a.75.75 0 0 0 0 1.5h11.978a1 1 0 0 1 .994 1.117L15 13.25A1.75 1.75 0 0 1 13.25 15H1.75A1.75 1.75 0 0 1 0 13.25V2.75c0-.464.184-.91.513-1.237Z"/>
                  </svg>
                  {{ t("pack.folder") }}
                </button>
              </div>
            </div>

            <p class="mt-2 text-xs text-[#8b949e] flex items-center gap-2">
              <span>{{ t("pack.mono") }}</span>
              <span>•</span>
              <span v-if="loaderLabel">{{ t("pack.loader", { name: loaderLabel }) }}</span>
            </p>

            <div v-if="updateInfo?.has_update && updateInfo.latest_version" class="mt-4 flex items-center justify-between gap-4 rounded-md border border-[#1f6beb]/40 bg-[#1f6beb]/10 px-3.5 py-2.5 text-xs text-[#58a6ff]">
              <span class="min-w-0">
                {{ t("update.available") }} <strong class="text-[#79c0ff]">{{ updateInfo.latest_version }}</strong>
                <span v-if="updateInfo.current_version" class="text-[#8b949e]">
                  {{ t("update.installed", { v: updateInfo.current_version }) }}
                </span>
              </span>
              <button
                type="button"
                class="shrink-0 rounded-md border border-[#1f6beb]/50 bg-[#1f6beb]/20 px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[#1f6beb]/40 disabled:opacity-50"
                :disabled="busy"
                @click="handleUpdate"
              >
                {{ t("update.btn") }}
              </button>
            </div>
          </div>

          <!-- Сабтабы: релизы / моды / ресурспаки / шейдеры / миры / консоль -->
          <div class="mb-4 flex shrink-0 flex-wrap items-center gap-1 border-b border-[#30363d] pb-2">
            <button
              v-for="st in playSubTabs"
              :key="st.kind"
              type="button"
              class="flex items-center gap-1.5 rounded-md px-3 py-1.5 text-[11px] font-medium transition-colors"
              :class="playSubTab === st.kind
                ? 'bg-[#21262d] text-[#f0f6fc]'
                : 'text-[#8b949e] hover:bg-[#21262d]/50 hover:text-[#c9d1d9]'"
              @click="playSubTab = st.kind"
            >
              <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 shrink-0 fill-current" v-html="st.icon"></svg>
              <span>{{ t("sub." + st.kind) }}</span>
            </button>
          </div>

          <!-- Список релизов GitHub -->
          <template v-if="playSubTab === 'releases'">
          <div v-if="versions && versions.github.length > 0" class="min-h-0 flex-1 space-y-4 overflow-y-auto pr-1">
            <div class="flex items-center justify-between text-xs text-[#8b949e]">
              <span class="font-medium">{{ t("releases.count", { n: versions.github.length }) }}</span>
            </div>

            <article
              v-for="r in versions.github"
              :key="r.tag"
              class="rounded-md border border-[#30363d] bg-[#161b22]"
            >
              <!-- Шапка релиза -->
              <div class="flex items-center justify-between border-b border-[#30363d] bg-[#21262d]/50 px-4 py-3">
                <div class="flex items-center gap-2.5 flex-wrap">
                  <span class="font-mono text-sm font-semibold text-[#58a6ff] hover:underline cursor-pointer">
                    {{ r.tag }}
                  </span>
                  <span v-if="r.name && r.name !== r.tag && !r.name.toLowerCase().startsWith(r.tag.toLowerCase())" class="text-xs text-[#8b949e]">
                    {{ r.name }}
                  </span>
                  <span v-if="r.prerelease" class="rounded-full border border-[#9e6a03]/40 bg-[#9e6a03]/10 px-2 py-0.2 text-[10px] font-medium text-[#d29922]">
                    {{ t("releases.pre") }}
                  </span>
                  <span v-if="isActiveRelease(r.tag)" class="rounded-full border border-[#238636]/40 bg-[#238636]/10 px-2 py-0.2 text-[10px] font-medium text-[#3fb950]">
                    {{ t("releases.active") }}
                  </span>
                </div>

                <div class="flex items-center gap-3">
                  <span class="text-[11px] text-[#8b949e]">
                    {{ formatDate(r.published_at) }}
                  </span>
                  <span
                    v-if="playtimeForRelease(r.tag) > 0"
                    class="font-mono text-[11px] text-[#d29922]"
                    :title="t('releases.playtime')"
                  >
                    {{ formatPlaytime(playtimeForRelease(r.tag)) }}
                  </span>
                  <button
                    type="button"
                    class="rounded-md border border-[#30363d] bg-[#21262d] px-2.5 py-1 text-xs font-medium text-[#c9d1d9] transition-colors hover:bg-[#30363d] hover:text-white disabled:opacity-50"
                    :disabled="busy"
                    @click="handleSelectVersion(r.tag)"
                  >
                    <template v-if="isInstalledVersion(r.tag)">
                      {{ isActiveRelease(r.tag) ? t("releases.selected") : t("releases.switch") }}
                    </template>
                    <template v-else>
                      {{ t("releases.install") }}
                    </template>
                  </button>
                </div>
              </div>

              <!-- Ченджлог -->
              <div class="p-4 text-xs text-[#c9d1d9] space-y-1.5">
                <div
                  v-if="changelogLines(r.body).length > 0"
                  class="changelog space-y-1 font-sans"
                  @click="onChangelogLinkClick"
                >
                  <template v-for="(line, idx) in visibleLines(r.body)" :key="idx">
                    <div v-if="line.type === 'bullet'" class="flex items-start gap-2 text-[#c9d1d9]">
                      <span class="text-[#8b949e] select-none">•</span>
                      <span v-html="renderInline(line.text)"></span>
                    </div>
                    <div v-else-if="line.type === 'body'" class="font-semibold text-[#f0f6fc] pt-1.5" v-html="renderInline(line.text)"></div>
                    <div v-else class="text-[#8b949e]" v-html="renderInline(line.text)"></div>
                  </template>
                </div>
                <p v-else class="text-[#8b949e] italic">{{ t("releases.noChangelog") }}</p>

                <button
                  v-if="isExpandable(r.body)"
                  type="button"
                  class="mt-2 inline-block text-xs font-medium text-[#58a6ff] hover:underline"
                  @click="toggleExpanded(r.tag)"
                >
                  {{ isExpanded(r.tag) ? t("releases.collapse") : t("releases.showAll") }}
                </button>
              </div>
            </article>
          </div>

          <div v-else class="shrink-0 rounded-md border border-[#30363d] bg-[#161b22] p-8 text-center text-xs text-[#8b949e]">
            {{ t("releases.loadError") }}
          </div>
          </template>

          <!-- Папки файлов игры: моды / ресурспаки / шейдеры / миры -->
          <div
            v-else-if="playSubTab === 'mods' || playSubTab === 'resourcepacks' || playSubTab === 'shaderpacks' || playSubTab === 'saves'"
            class="flex min-h-0 flex-1 flex-col"
          >
            <div class="mb-3 flex shrink-0 items-center justify-between gap-3">
              <span class="shrink-0 text-xs text-[#8b949e]">
                {{ playSubTab === "saves" ? t("files.worldsCount", { n: fileVisibleCount }) : t("files.count", { n: fileVisibleCount }) }}
              </span>
              <div class="flex min-w-0 items-center gap-2">
                <div v-if="Object.keys(selectedFiles).length > 0" class="flex shrink-0 items-center gap-1.5">
                  <span class="text-[11px] text-[#8b949e]">
                    {{ t("files.selected", { n: Object.keys(selectedFiles).length }) }}
                  </span>
                  <button
                    type="button"
                    class="rounded-md border border-[#30363d] bg-[#21262d] px-2 py-1 text-[11px] font-medium text-[#c9d1d9] transition-colors hover:bg-[#30363d] hover:text-white"
                    :title="t('files.enableSel')"
                    @click="setSelectedFilesEnabled(true)"
                  >
                    {{ t("files.enable") }}
                  </button>
                  <button
                    type="button"
                    class="rounded-md border border-[#30363d] bg-[#21262d] px-2 py-1 text-[11px] font-medium text-[#c9d1d9] transition-colors hover:bg-[#30363d] hover:text-white"
                    :title="t('files.disableSel')"
                    @click="setSelectedFilesEnabled(false)"
                  >
                    {{ t("files.disable") }}
                  </button>
                  <button
                    type="button"
                    class="flex items-center gap-1 rounded-md border border-[#30363d] bg-[#21262d] px-2 py-1 text-[11px] font-medium text-[#c9d1d9] transition-colors hover:bg-[#30363d] hover:text-white"
                    @click="openSelected('modrinth')"
                  >
                    <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M7.775 3.275a.75.75 0 0 0 1.06 1.06l1.25-1.25v11.165a.75.75 0 0 0 1.5 0V2.085l1.25 1.25a.75.75 0 0 0 1.06-1.06L9.56.53a.75.75 0 0 0-1.06 0L7.775 3.275Z"/></svg>
                    Modrinth
                  </button>
                  <button
                    type="button"
                    class="flex items-center gap-1 rounded-md border border-[#30363d] bg-[#21262d] px-2 py-1 text-[11px] font-medium text-[#c9d1d9] transition-colors hover:bg-[#30363d] hover:text-white"
                    @click="openSelected('curseforge')"
                  >
                    <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M7.775 3.275a.75.75 0 0 0 1.06 1.06l1.25-1.25v11.165a.75.75 0 0 0 1.5 0V2.085l1.25 1.25a.75.75 0 0 0 1.06-1.06L9.56.53a.75.75 0 0 0-1.06 0L7.775 3.275Z"/></svg>
                    CurseForge
                  </button>
                  <button
                    type="button"
                    class="rounded-md border border-[#30363d] bg-[#21262d] px-2 py-1 text-[11px] font-medium text-[#8b949e] transition-colors hover:bg-[#30363d] hover:text-white"
                    @click="clearFileSelection()"
                  >
                    {{ t("files.clear") }}
                  </button>
                </div>
                <div class="relative min-w-0 flex-1">
                  <svg viewBox="0 0 16 16" class="pointer-events-none absolute left-2 top-1/2 h-3 w-3 -translate-y-1/2 fill-[#484f58]">
                    <path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"/>
                  </svg>
                  <input
                    v-model="fileSearch"
                    type="text"
                    :placeholder="t('files.search')"
                    class="w-full rounded-md border border-[#30363d] bg-[#0d1117] py-1.5 pl-7 pr-2 text-[11px] text-[#c9d1d9] placeholder-[#484f58] outline-none transition-colors focus:border-[#58a6ff]"
                  />
                </div>
                <button
                  type="button"
                  class="flex shrink-0 items-center gap-1.5 rounded-md border border-[#30363d] bg-[#21262d] px-2.5 py-1 text-[11px] font-medium text-[#c9d1d9] transition-colors hover:bg-[#30363d] hover:text-white"
                  @click="openFolder(playSubTab as GameFolderKind)"
                >
                  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                    <path d="M.513 1.513A1.75 1.75 0 0 1 1.75 1h3.5c.55 0 1.07.26 1.4.7l.9 1.2a.25.25 0 0 0 .2.1H13a1 1 0 0 1 1 1v.5H2.75a.75.75 0 0 0 0 1.5h11.978a1 1 0 0 1 .994 1.117L15 13.25A1.75 1.75 0 0 1 13.25 15H1.75A1.75 1.75 0 0 1 0 13.25V2.75c0-.464.184-.91.513-1.237Z"/>
                  </svg>
                  {{ t("files.open") }}
                </button>
              </div>
            </div>

            <div v-if="!gameFiles[playSubTab]" class="flex flex-1 items-center justify-center text-xs text-[#8b949e]">
              <svg class="mr-2 h-4 w-4 animate-spin fill-[#58a6ff]" viewBox="0 0 16 16">
                <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
              </svg>
              {{ t("files.loading") }}
            </div>
            <div v-else-if="(gameFiles[playSubTab] ?? []).length === 0" class="shrink-0 rounded-md border border-[#30363d] bg-[#161b22] p-8 text-center text-xs text-[#8b949e]">
              {{ t("files.empty") }}
            </div>
            <div
              v-else
              ref="fileListRef"
              class="min-h-0 flex-1 overflow-y-auto overscroll-contain pr-1 pb-8"
              @scroll="fileListScroll"
            >
              <div class="relative" :style="{ height: `${fileListTotal}px` }">
                <div
                  class="absolute left-0 right-0 space-y-2"
                  :style="{ transform: `translateY(${fileListStart * fileRowStride}px)` }"
                >
<div
                    v-for="f in fileListVisible"
                    :key="f.name"
                    class="file-row flex cursor-pointer items-center gap-3 rounded-md border px-3 py-2 transition-colors"
                    :class="[
                      isFileSelected(playSubTab, f)
                        ? 'border-[#1f6beb] bg-[#1f6beb]/10'
                        : 'border-[#30363d] bg-[#161b22] hover:border-[#484f58]',
                      { 'opacity-60': !f.enabled },
                    ]"
                    @click="toggleFileSelect(playSubTab as GameFolderKind, f)"
                  >
                    <svg
                      viewBox="0 0 16 16"
                      class="h-3.5 w-3.5 shrink-0"
                      :class="isFileSelected(playSubTab, f) ? 'fill-[#58a6ff]' : 'fill-[#484f58]'"
                    >
                      <path v-if="isFileSelected(playSubTab, f)" d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/>
                      <path v-else d="M4 1.75C4 .784 4.784 0 5.75 0h5.586c.464 0 .909.184 1.237.513l2.914 2.914a1.75 1.75 0 0 1 .513 1.237V12.25A1.75 1.75 0 0 1 14.25 14H5.75A1.75 1.75 0 0 1 4 12.25Zm1.75-.25a.25.25 0 0 0-.25.25v10.5c0 .138.112.25.25.25h8.5a.25.25 0 0 0 .25-.25V6h-2.75A1.75 1.75 0 0 1 10 4.25V1.5Z"/>
                    </svg>
                    <div class="flex h-10 w-10 shrink-0 items-center justify-center overflow-hidden rounded-md border border-[#30363d] bg-[#0d1117]">
                      <img
                        v-if="gameFileIcon(playSubTab, f.name)"
                        :src="gameFileIcon(playSubTab, f.name)"
                        alt=""
                        loading="lazy"
                        class="h-full w-full object-contain"
                      />
                      <svg v-else viewBox="0 0 16 16" class="h-5 w-5 fill-[#8b949e]">
                        <path d="M.75 6.25a1.75 1.75 0 0 1 1.75-1.75h2.054l1.17-1.17A1.74 1.74 0 0 1 6.902 2.75h1.536l-.055.836a1.44 1.44 0 0 0 .432 1.123l.022.022c.059.059.12.108.183.148.523.34 1.074.405 1.528.429.755.04 1.452.044 1.766.044h3.44v.586c0 .527-.211 1.032-.587 1.404l-3.318 3.318a1.5 1.5 0 0 1-1.06.44H4.78l-.824-.412a1.75 1.75 0 0 1-.736-2.383.5.5 0 0 1-.368-.454A1.75 1.75 0 0 1 .75 6.25Zm13.24 0h-3.14c-.249 0-.679-.004-1.112-.03-.36-.022-.622-.066-.783-.111.05-.066.11-.129.176-.194l.483-.483c.344-.344.416-.861.18-1.283A1.75 1.75 0 0 0 8.5 2.75H4.75A1.75 1.75 0 0 1 4.75 1.5c.692-.06 1.4-.086 2.127-.086.63 0 1.255.022 1.873.064a.75.75 0 0 1 .5.25.75.75 0 0 1 .246.5l.293 2.927.178.04c.646.147 1.548.377 2.615.614.17.038.2.07.22.098a.6.6 0 0 1 .074.233.75.75 0 0 1-.075.4.6.6 0 0 1-.235.23ZM3.75 10.75h5.38l1.5-1.5H3.75a.75.75 0 0 1-.143 1.482l-.14.014a.75.75 0 0 1 .283-.004L3.75 10.75Z"/>
                      </svg>
                    </div>
                <div class="min-w-0 flex-1">
                  <div class="truncate text-xs font-medium text-[#c9d1d9]" :title="f.name">
                    {{ f.displayName }}
                  </div>
                  <div class="text-[10px] text-[#8b949e]">
                    {{ f.kind === "dir" ? t("files.dir") : `${formatBytes(f.sizeBytes)} · ${f.enabled ? t("files.enabled") : t("files.disabled")}` }}
                  </div>
                </div>
                <button
                  type="button"
                  class="flex shrink-0 items-center gap-1 rounded-md border border-[#30363d] bg-[#21262d] px-2 py-1 text-[10px] font-medium text-[#8b949e] transition-colors hover:border-[#58a6ff]/50 hover:text-[#58a6ff]"
                  :title="t('files.modrinth')"
                  @click.stop="openFileOnModrinth(playSubTab as GameFolderKind, f)"
                >
                  Modrinth
                </button>
                <button
                  v-if="f.kind === 'file'"
                  type="button"
                  class="relative h-5 w-9 shrink-0 rounded-full transition-colors"
                  :class="f.enabled ? 'bg-[#238636]' : 'bg-[#484f58]'"
                  role="switch"
                  :aria-checked="f.enabled"
                  :title="f.enabled ? t('files.disable') : t('files.enable')"
                  @click.stop="handleToggleFile(playSubTab as GameFolderKind, f)"
                >
                  <span
                    class="absolute top-0.5 h-4 w-4 rounded-full bg-white transition-all"
                    :class="f.enabled ? 'left-[18px]' : 'left-0.5'"
                  />
                </button>
              </div>
            </div>
          </div>
          </div>
          </div>

          <!-- Консоль / логи -->
          <section v-else class="flex h-full min-h-0 flex-1 flex-col overflow-hidden rounded-md border border-[#30363d] bg-[#161b22]">
            <div class="flex items-center justify-between border-b border-[#30363d] bg-[#21262d]/50 px-4 py-2">
              <h3 class="text-xs font-semibold text-[#f0f6fc]">{{ t("console.title") }}</h3>
              <div class="flex items-center gap-3">
                <span class="text-[10px] tabular-nums text-[#484f58]">
                  {{ t("console.lines", { n: logEntries.length }) }}
                </span>
                <div class="flex gap-2">
                  <button
                    type="button"
                    class="text-[11px] text-[#8b949e] hover:text-[#58a6ff]"
                    @click="handleCopyLog"
                  >
                    {{ t("console.copy") }}
                  </button>
                  <button
                    type="button"
                    class="text-[11px] text-[#8b949e] hover:text-[#f85149]"
                    @click="handleClearLog"
                  >
                    {{ t("console.clear") }}
                  </button>
                  <button
                    type="button"
                    class="text-[11px] text-[#8b949e] hover:text-[#58a6ff]"
                    @click="openFolder('logs')"
                  >
                    {{ t("console.logs") }}
                  </button>
                </div>
              </div>
            </div>
            <div
              ref="logRef"
              class="flex-1 select-text overflow-y-auto bg-[#0d1117] p-3 font-mono text-[11px] leading-relaxed text-[#8b949e]"
            >
              <p v-if="logEntries.length === 0" class="italic text-[#484f58]">
                {{ t("console.empty") }}
              </p>
              <div
                v-for="(e, i) in logEntries"
                :key="i"
                :class="{
                  'text-[#f85149]': e.stream === 'err',
                  'text-[#58a6ff]': e.stream === 'sys',
                  'text-[#c9d1d9]': e.stream === 'out',
                }"
              >
                {{ e.line }}
              </div>
            </div>
          </section>
          </div>
        </template>

        <!-- ======= Вкладка: Новости ======= -->
        <template v-else-if="tab === 'news'">
          <div class="flex min-h-0 flex-1 flex-col">
            <div class="mb-6 shrink-0 border-b border-[#30363d] pb-5">
              <h1 class="text-xl font-semibold text-[#f0f6fc]">{{ t("news.title") }}</h1>
              <p class="mt-2 text-xs text-[#8b949e]">
                {{ t("news.subtitle") }}
              </p>
              <div class="mt-4 flex flex-wrap items-center gap-2">
                <button
                  v-for="src in newsSources"
                  :key="src"
                  type="button"
                  class="rounded-full border px-3 py-1 text-[11px] font-medium transition-colors"
                  :class="newsFilter === src
                    ? 'border-[#1f6beb]/60 bg-[#1f6beb]/20 text-white'
                    : 'border-[#30363d] bg-[#21262d] text-[#8b949e] hover:bg-[#30363d] hover:text-[#c9d1d9]'"
                  @click="newsFilter = src"
                >
                  {{ src === "launcher" ? "NIO Launcher" : packNameFor(src) }}
                </button>
                <button
                  type="button"
                  class="rounded-full border px-3 py-1 text-[11px] font-medium transition-colors"
                  :class="newsFilter === 'all'
                    ? 'border-[#1f6beb]/60 bg-[#1f6beb]/20 text-white'
                    : 'border-[#30363d] bg-[#21262d] text-[#8b949e] hover:bg-[#30363d] hover:text-[#c9d1d9]'"
                  @click="newsFilter = 'all'"
                >
                  {{ t("news.all") }}
                </button>
              </div>
            </div>

            <div v-if="news === null" class="flex flex-1 items-center justify-center text-xs text-[#8b949e]">
              <svg class="mr-2 h-4 w-4 animate-spin fill-[#58a6ff]" viewBox="0 0 16 16">
                <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
              </svg>
              {{ t("news.loading") }}
            </div>

            <div v-else-if="news.length === 0" class="shrink-0 rounded-md border border-[#30363d] bg-[#161b22] p-8 text-center text-xs text-[#8b949e]">
              {{ t("news.none") }}
            </div>

            <div v-else-if="filteredNews.length === 0" class="shrink-0 rounded-md border border-[#30363d] bg-[#161b22] p-8 text-center text-xs text-[#8b949e]">
              {{ t("news.emptyCat") }}
            </div>

            <div v-else class="min-h-0 flex-1 space-y-4 overflow-y-auto pr-1 pb-8">
              <article
                v-for="n in filteredNews"
                :key="`${n.kind}-${n.url || n.tag}`"
                class="rounded-md border border-[#30363d] bg-[#161b22]"
              >
                <div class="flex items-start justify-between gap-3 border-b border-[#30363d] bg-[#21262d]/50 px-4 py-3">
                  <div class="min-w-0">
                    <div class="flex items-center gap-2 flex-wrap">
                      <span
                        class="rounded-full px-2 py-0.5 text-[10px] font-medium border"
                        :class="n.kind === 'update'
                          ? 'border-[#1f6beb]/40 bg-[#1f6beb]/10 text-[#58a6ff]'
                          : 'border-[#9e6a03]/40 bg-[#9e6a03]/10 text-[#d29922]'"
                      >
                        {{ n.kind === "update" ? t("news.update") : t("news.post") }}
                      </span>
                      <span v-if="n.category" class="rounded-full border border-[#30363d] bg-[#0d1117] px-2 py-0.5 text-[10px] font-medium text-[#8b949e]">
                        {{ n.category }}
                      </span>
                      <span class="rounded-full border border-[#30363d] bg-[#0d1117] px-2 py-0.5 text-[10px] font-medium text-[#8b949e]">
                        {{ n.pack_name }}
                      </span>
                      <span v-if="n.kind === 'update' && n.tag" class="font-mono text-xs font-semibold text-[#58a6ff]">
                        {{ n.tag }}
                      </span>
                    </div>
                    <h2 class="mt-1.5 text-sm font-semibold text-[#f0f6fc] break-words">
                      {{ n.title }}
                    </h2>
                  </div>
                  <div class="flex shrink-0 flex-col items-end gap-1.5">
                    <span class="text-[11px] text-[#8b949e]">
                      {{ formatDate(n.date) }}
                    </span>
                    <div class="flex gap-1.5">
                      <button
                        v-if="n.kind === 'post' && n.url"
                        type="button"
                        class="rounded-md border border-[#30363d] bg-[#21262d] px-2.5 py-1 text-[11px] font-medium text-[#c9d1d9] transition-colors hover:bg-[#30363d] hover:text-white"
                        @click="openNewsLink(n.url)"
                      >
                        {{ t("news.open") }}
                      </button>
                      <button
                        v-if="n.kind === 'update' && n.tag"
                        type="button"
                        class="rounded-md border border-[#1f6beb]/50 bg-[#1f6beb]/20 px-2.5 py-1 text-[11px] font-semibold text-white transition-colors hover:bg-[#1f6beb]/40 disabled:opacity-50"
                        :disabled="busy"
                        @click="installNews(n)"
                      >
                        {{ isInstalledVersion(n.tag) ? (isActiveNewsTag(n.tag) ? t("releases.selected") : t("releases.switch")) : t("releases.install") }}
                      </button>
                    </div>
                  </div>
                </div>

                <!-- Тело: ченджлог/пост -->
                <div v-if="changelogLines(n.body).length > 0" class="p-4 text-xs text-[#c9d1d9] space-y-1.5">
                  <div class="changelog space-y-1 font-sans" @click="onChangelogLinkClick">
                    <template v-for="(line, idx) in visibleNewsLines(n)" :key="idx">
                      <div v-if="line.type === 'bullet'" class="flex items-start gap-2 text-[#c9d1d9]">
                        <span class="text-[#8b949e] select-none">•</span>
                        <span v-html="renderInline(line.text)"></span>
                      </div>
                      <div v-else-if="line.type === 'body'" class="font-semibold text-[#f0f6fc] pt-1.5" v-html="renderInline(line.text)"></div>
                      <div v-else class="text-[#8b949e]" v-html="renderInline(line.text)"></div>
                    </template>
                  </div>
                  <button
                    v-if="isNewsExpandable(n)"
                    type="button"
                    class="mt-2 inline-block text-xs font-medium text-[#58a6ff] hover:underline"
                    @click="toggleNewsExpanded(n)"
                  >
                    {{ isNewsExpanded(n) ? t("news.collapse") : t("news.showAll") }}
                  </button>
                </div>
                <div v-else class="p-4 text-xs text-[#8b949e] italic">
                  {{ t("news.noText") }}
                </div>
              </article>
            </div>
          </div>
        </template>

        <!-- ======= Вкладка: Настройки ======= -->
        <template v-else>
          <div class="min-h-0 flex-1 overflow-y-auto pr-1">
          <div class="space-y-6">
            <div class="border-b border-[#30363d] pb-3">
              <h1 class="text-lg font-semibold text-[#f0f6fc]">{{ t("settings.title") }}</h1>
              <p class="text-xs text-[#8b949e]">{{ t("settings.subtitle") }}</p>
            </div>

            <!-- Учётная запись -->
            <section class="rounded-md border border-[#30363d] bg-[#161b22] overflow-hidden">
              <div class="border-b border-[#30363d] bg-[#21262d]/50 px-4 py-2.5">
                <h3 class="text-xs font-semibold text-[#f0f6fc]">{{ t("settings.account") }}</h3>
              </div>
              <div class="p-4 space-y-3">
                <div class="flex gap-2">
                  <input
                    v-model="username"
                    :placeholder="t('settings.nickname')"
                    class="flex-1 rounded-md border border-[#30363d] bg-[#0d1117] px-3 py-1.5 text-xs text-[#c9d1d9] placeholder-[#8b949e] focus:border-[#58a6ff] focus:outline-none"
                  />
                  <button
                    type="button"
                    class="rounded-md border border-[#30363d] bg-[#21262d] px-3 py-1.5 text-xs font-medium text-[#c9d1d9] hover:bg-[#30363d] disabled:opacity-50"
                    :disabled="busy"
                    @click="handleOffline"
                  >
                    {{ t("settings.save") }}
                  </button>
                </div>

                <div class="relative flex items-center justify-center my-2">
                  <div class="border-t border-[#30363d] w-full"></div>
                  <span class="bg-[#161b22] px-2 text-[10px] uppercase text-[#8b949e] absolute">{{ t("settings.or") }}</span>
                </div>

                <button
                  type="button"
                  class="w-full rounded-md border border-[#30363d] bg-[#21262d] py-1.5 text-xs font-medium text-[#c9d1d9] hover:bg-[#30363d] disabled:opacity-50"
                  :disabled="busy || msPolling"
                  @click="handleMicrosoft"
                >
                  {{ msPolling ? t("settings.msWait") : t("settings.msSignin") }}
                </button>

                <!-- Device code flow: показать код и ссылку -->
                <div
                  v-if="msFlow"
                  class="rounded-md border border-[#1f6beb]/40 bg-[#0d1117]/60 p-3 space-y-2"
                >
                  <p class="text-[11px] text-[#8b949e]">
                    {{ t("settings.msCode") }}
                  </p>
                  <div class="flex items-center gap-3">
                    <div
                      v-if="msFlow.qr_svg"
                      class="h-28 w-28 shrink-0 overflow-hidden rounded-md border border-[#30363d] bg-white"
                      :title="t('settings.msScan')"
                    >
                      <div class="h-full w-full" v-html="msFlow.qr_svg"></div>
                    </div>
                    <div class="min-w-0 flex-1">
                    <p class="font-mono text-2xl font-bold tracking-[0.3em] text-[#79c0ff] select-text">
                      {{ msFlow.user_code }}
                    </p>
                    <button
                      type="button"
                      class="mt-2 rounded-md border border-[#1f6beb]/50 bg-[#1f6beb]/20 px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[#1f6beb]/40"
                      @click="openMsAuthPage"
                    >
                      {{ t("settings.msOpen", { uri: msFlow.verification_uri.replace(/^https?:\/\//, "") }) }}
                    </button>
                    </div>
                  </div>
                  <p v-if="msPolling" class="flex items-center gap-2 text-[11px] text-[#8b949e]">
                    <svg class="h-3 w-3 animate-spin fill-[#58a6ff]" viewBox="0 0 16 16">
                      <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                    </svg>
                    {{ t("settings.msBrowser") }}
                  </p>
                </div>
              </div>
            </section>

            <!-- ОЗУ -->
            <section class="rounded-md border border-[#30363d] bg-[#161b22] overflow-hidden">
              <div class="border-b border-[#30363d] bg-[#21262d]/50 px-4 py-2.5 flex justify-between items-center">
                <h3 class="text-xs font-semibold text-[#f0f6fc]">{{ t("settings.ram") }}</h3>
                <span class="font-mono text-xs font-semibold text-[#58a6ff]">{{ ram }} {{ t("units.gb") }}</span>
              </div>
              <div class="p-4 space-y-2">
                <input
                  type="range"
                  min="2"
                  :max="maxRam"
                  step="1"
                  v-model.number="ram"
                  class="w-full accent-[#1f6beb] bg-[#21262d] h-1.5 rounded-lg appearance-none cursor-pointer"
                />
                <div class="flex justify-between text-[11px] text-[#8b949e] font-mono">
                  <span>2 {{ t("units.gb") }}</span>
                  <span>{{ t("settings.ramMax", { n: maxRam }) }}</span>
                </div>
                <p v-if="systemRam && systemRam.total_ram_gb > 0" class="text-[11px] text-[#8b949e]">
                  {{ t("settings.ramTotal", { total: systemRam.total_ram_gb, avail: systemRam.available_ram_gb }) }}
                </p>
              </div>
            </section>

            <!-- Размер окна игры -->
            <section class="rounded-md border border-[#30363d] bg-[#161b22] overflow-hidden">
              <div class="border-b border-[#30363d] bg-[#21262d]/50 px-4 py-2.5 flex justify-between items-center">
                <h3 class="text-xs font-semibold text-[#f0f6fc]">{{ t("settings.win") }}</h3>
                <span class="font-mono text-xs font-semibold text-[#58a6ff]">{{ windowWidth }}×{{ windowHeight }}</span>
              </div>
              <div class="p-4 space-y-2">
                <div class="flex items-center gap-3">
                  <label class="w-16 text-[11px] text-[#8b949e]" for="win-width">{{ t("settings.width") }}</label>
                  <input
                    id="win-width"
                    type="number"
                    min="320"
                    max="7680"
                    step="1"
                    v-model.number="windowWidth"
                    class="flex-1 rounded-md border border-[#30363d] bg-[#0d1117] px-3 py-1.5 text-xs text-[#c9d1d9] focus:border-[#58a6ff] focus:outline-none"
                  />
                  <label class="w-16 text-[11px] text-[#8b949e]" for="win-height">{{ t("settings.height") }}</label>
                  <input
                    id="win-height"
                    type="number"
                    min="240"
                    max="4320"
                    step="1"
                    v-model.number="windowHeight"
                    class="flex-1 rounded-md border border-[#30363d] bg-[#0d1117] px-3 py-1.5 text-xs text-[#c9d1d9] focus:border-[#58a6ff] focus:outline-none"
                  />
                </div>
                <p class="text-[11px] text-[#8b949e]">
                  {{ t("settings.winNote") }}
                </p>
              </div>
            </section>

            <!-- Java -->
            <section class="rounded-md border border-[#30363d] bg-[#161b22] overflow-hidden">
              <div class="border-b border-[#30363d] bg-[#21262d]/50 px-4 py-2.5">
                <h3 class="text-xs font-semibold text-[#f0f6fc]">{{ t("settings.java") }}</h3>
              </div>
              <div class="p-4 space-y-3">
                <div class="flex items-center gap-2">
                  <select
                    :value="javaSelected"
                    class="flex-1 appearance-none rounded-md border border-[#30363d] bg-[#21262d] px-3 py-1.5 pr-8 text-xs text-[#c9d1d9] transition-colors hover:bg-[#30363d] focus:border-[#58a6ff] focus:outline-none"
                    :disabled="javaBusy || busy"
                    @change="onJavaChange"
                  >
                    <option value="">{{ t("settings.javaAuto") }}</option>
                    <option v-for="j in javaList" :key="j.path" :value="j.path">
                      {{ j.label }} — {{ j.version }} [{{ javaArchLabel(j.arch) }}]
                    </option>
                  </select>
                  <button
                    type="button"
                    class="shrink-0 rounded-md border border-[#30363d] bg-[#21262d] px-3 py-1.5 text-xs font-medium text-[#c9d1d9] transition-colors hover:bg-[#30363d] disabled:opacity-50"
                    :disabled="javaBusy || busy"
                    @click="downloadJava"
                  >
                    {{ javaBusy ? t("settings.javaDownloading") : t("settings.javaDownload") }}
                  </button>
                </div>
                <p v-if="javaMsg" class="text-[11px] text-[#8b949e] break-all">{{ javaMsg }}</p>
                <p class="text-[11px] text-[#8b949e]">
                  {{ t("settings.javaNote") }}
                </p>
              </div>
            </section>

            <!-- Discord Rich Presence -->
            <section class="rounded-md border border-[#30363d] bg-[#161b22] overflow-hidden">
              <div class="border-b border-[#30363d] bg-[#21262d]/50 px-4 py-2.5">
                <h3 class="text-xs font-semibold text-[#f0f6fc]">{{ t("settings.discord") }}</h3>
              </div>
              <div class="p-4">
                <label class="flex cursor-pointer items-center gap-3">
                  <input
                    type="checkbox"
                    class="h-4 w-4 accent-[#5865F2]"
                    :checked="discordRp"
                    @change="toggleDiscordRp(($event.target as HTMLInputElement).checked)"
                  />
                  <span class="text-xs text-[#c9d1d9]">{{ t("settings.discordLabel") }}</span>
                </label>
                <p class="mt-2 text-[11px] text-[#8b949e]">
                  {{ t("settings.discordNote") }}
                </p>
              </div>
            </section>

            <!-- Язык интерфейса -->
            <section class="rounded-md border border-[#30363d] bg-[#161b22] overflow-hidden">
              <div class="border-b border-[#30363d] bg-[#21262d]/50 px-4 py-2.5">
                <h3 class="text-xs font-semibold text-[#f0f6fc]">{{ t("settings.language") }}</h3>
              </div>
              <div class="flex gap-2 p-4">
                <button
                  type="button"
                  class="rounded-md border px-4 py-1.5 text-xs font-medium transition-colors"
                  :class="locale === 'ru'
                    ? 'border-[#1f6beb]/60 bg-[#1f6beb]/20 text-white'
                    : 'border-[#30363d] bg-[#21262d] text-[#c9d1d9] hover:bg-[#30363d]'"
                  @click="setLocale('ru')"
                >
                  Русский
                </button>
                <button
                  type="button"
                  class="rounded-md border px-4 py-1.5 text-xs font-medium transition-colors"
                  :class="locale === 'en'
                    ? 'border-[#1f6beb]/60 bg-[#1f6beb]/20 text-white'
                    : 'border-[#30363d] bg-[#21262d] text-[#c9d1d9] hover:bg-[#30363d]'"
                  @click="setLocale('en')"
                >
                  English
                </button>
              </div>
            </section>

            <!-- Проверка целостности -->
            <section class="rounded-md border border-[#30363d] bg-[#161b22] overflow-hidden">
              <div class="border-b border-[#30363d] bg-[#21262d]/50 px-4 py-2.5 flex justify-between items-center">
                <h3 class="text-xs font-semibold text-[#f0f6fc]">{{ t("settings.verify") }}</h3>
              </div>
              <div class="p-4 space-y-3">
                <p class="text-[11px] text-[#8b949e]">
                  {{ t("settings.verifyNote") }}
                </p>
                <button
                  type="button"
                  class="rounded-md border border-[#30363d] bg-[#21262d] px-3 py-1.5 text-xs font-medium text-[#c9d1d9] transition-colors hover:bg-[#30363d] disabled:opacity-50"
                  :disabled="verifyBusy || busy"
                  @click="handleVerify"
                >
                  {{ verifyBusy ? t("settings.verifying") : t("settings.verifyBtn") }}
                </button>
                <div
                  v-if="verifyResult"
                  class="rounded-md border bg-[#0d1117]/60 p-3 text-[11px]"
                  :class="verifyResult.broken.length === 0 ? 'border-[#238636]/40' : 'border-[#f85149]/40'"
                >
                  <p class="font-medium" :class="verifyResult.broken.length === 0 ? 'text-[#3fb950]' : 'text-[#f85149]'">
                    {{ verifyResult.broken.length === 0 ? t("settings.verifyOk") : t("settings.verifyBroken", { n: verifyResult.broken.length }) }}
                  </p>
                  <p class="mt-0.5 text-[#8b949e]">{{ t("settings.verifyStats", { checked: verifyResult.checked, ok: verifyResult.ok }) }}</p>
                  <ul v-if="verifyResult.broken.length > 0" class="mt-2 max-h-32 space-y-1 overflow-y-auto font-mono text-[10px] text-[#f85149]">
                    <li v-for="b in verifyResult.broken" :key="b">{{ b }}</li>
                  </ul>
                </div>
              </div>
            </section>
          </div>
          </div>
        </template>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { nextTick, ref } from "vue";
import { isTauri, openExternal } from "~/lib/bridge";
import type { GameFolderKind } from "~/lib/bridge";
import type { GameFileEntry, NewsItem } from "~/lib/types";
import { useLauncher } from "~/composables/useLauncher";
import { useI18n } from "~/composables/useI18n";

const {
  status,
  username,
  ram,
  maxRam,
  systemRam,
  windowWidth,
  windowHeight,
  session,
  busy,
  progress,
  updateInfo,
  versions,
  logEntries,
  logRef,
  tab,
  packs,
  packId,
  activePack,
  percent,
  loaderLabel,
  formatBytes,
  formatDate,
  formatPlaytime,
  isInstalledVersion,
  handleInstall,
  handleUpdate,
  handleSelectVersion,
  handleOffline,
  handleMicrosoft,
  openMsAuthPage,
  msFlow,
  msPolling,
  handlePlay,
  handleClearLog,
  handleCopyLog,
  handleOpenPackDir,
  selectPack,
  notifications,
  dismissNotification,
  reportError,
  appUpdate,
  appUpdating,
  appUpdateProgress,
  installAppUpdate,
  skinUrl,
  javaList,
  javaSelected,
  javaBusy,
  javaMsg,
  selectJava,
  downloadJava,
  verifyBusy,
  verifyResult,
  handleVerify,
  openFolder,
  discordRp,
  toggleDiscordRp,
  news,
  newsFilter,
  newsSources,
  filteredNews,
  playSubTab,
  gameFiles,
  fileIcons,
  fileSearch,
  selectedFiles,
  toggleFileSelect,
  clearFileSelection,
  setSelectedFilesEnabled,
  openFileOnModrinth,
  openFileOnCurseForge,
  handleToggleFile,
} = useLauncher();

const { t, locale, setLocale } = useI18n();

const ICON_TAG =
  "M1 7.775V2.75C1 1.784 1.784 1 2.75 1h5.025c.464 0 .91.184 1.238.513l6.25 6.25a1.75 1.75 0 0 1 0 2.474l-5.026 5.026a1.75 1.75 0 0 1-2.474 0l-6.25-6.25A1.752 1.752 0 0 1 1 7.775Zm1.5 0c0 .066.026.13.073.177l6.25 6.25a.25.25 0 0 0 .354 0l5.025-5.025a.25.25 0 0 0 0-.354l-6.25-6.25a.25.25 0 0 0-.177-.073H2.75a.25.25 0 0 0-.25.25ZM6 5a1 1 0 1 0 0 2 1 1 0 0 0 0-2Z";
const ICON_PACKAGE =
  "M8.878.392a1.75 1.75 0 0 0-1.756 0l-6.065 3.685A1.75 1.75 0 0 0 .25 5.607v4.786c0 .649.353 1.247.925 1.562l6.065 3.653a1.75 1.75 0 0 0 1.72 0l6.065-3.653a1.75 1.75 0 0 0 .925-1.562V5.607a1.75 1.75 0 0 0-.807-1.53ZM5.5 2.8h5l.972.972H4.528ZM3.747 2.2h2.109l-.972.972H2.775Zm.903 3.547 3.35 2.034 3.35-2.034.14 6.994H4.51Zm-1.564.913-.972.972-.43.005L2 4.814Zm10.828-.972.143 6.988-.43-.005-.972-.972L11.25 5.841Z";
const ICON_PAINT =
  "M.75 7.5a.75.75 0 0 1 .75-.75h13a.75.75 0 0 1 .75.75v6a1.75 1.75 0 0 1-1.75 1.75H2.5A1.75 1.75 0 0 1 .75 13.5ZM2.5 8.25v5.25h11V8.25ZM4 5.25a.75.75 0 0 1-.75-.75V2.75A1.75 1.75 0 0 1 5 1h2.5A1.75 1.75 0 0 1 9.25 2.75L11 4.5h.75V7H6.25a.75.75 0 0 1-.53-.22L4.22 5.28a.75.75 0 0 1-.22-.53Z";
const ICON_SUN =
  "M8 5.5a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5ZM2.5 8a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 0 1.5h-1.5A.75.75 0 0 1 2.5 8Zm7.75 0a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 0 1.5h-1.5A.75.75 0 0 1 10.25 8Zm-5.53 2.72a.75.75 0 0 1 0 1.061l-1.06 1.061a.75.75 0 0 1-1.061-1.061l1.06-1.061a.75.75 0 0 1 1.061 0Zm5.657 0a.75.75 0 0 1 1.061 0l1.06.53a.75.75 0 1 1-.53 1.404l-1.06-.53a.75.75 0 0 1-.53-.531.75.75 0 0 1 0-.53.75.75 0 0 1 .53-.53Zm-5.657 4.803a.75.75 0 0 1 0-1.061l1.06-1.061a.75.75 0 0 1 1.061 1.061l-1.06 1.061a.75.75 0 0 1-1.061 0Zm5.657 0a.75.75 0 0 1 1.061-1.061l1.06 1.061a.75.75 0 0 1-1.06 1.061Z";
const ICON_FOLDER =
  "M.513 1.513A1.75 1.75 0 0 1 1.75 1h3.5c.55 0 1.07.26 1.4.7l.9 1.2a.25.25 0 0 0 .2.1H13a1 1 0 0 1 1 1v.5H2.75a.75.75 0 0 0 0 1.5h11.978a1 1 0 0 1 .994 1.117L15 13.25A1.75 1.75 0 0 1 13.25 15H1.75A1.75 1.75 0 0 1 0 13.25V2.75c0-.464.184-.91.513-1.237Z";
const ICON_TERMINAL =
  "M0 2.75C0 1.784.784 1 1.75 1h12.5c.966 0 1.75.784 1.75 1.75v10.5A1.75 1.75 0 0 1 14.25 15H1.75A1.75 1.75 0 0 1 0 13.25Zm1.75-.25a.25.25 0 0 0-.25.25v10.5c0 .138.112.25.25.25h12.5a.25.25 0 0 0 .25-.25V2.75a.25.25 0 0 0-.25-.25ZM7.25 8a.75.75 0 0 1-.22.53l-2.25 2.25a.75.75 0 0 1-1.06-1.06L5.44 8 3.72 6.28a.75.75 0 1 1 1.06-1.06l2.25 2.25c.141.14.22.331.22.53Zm1.5 1.5a.75.75 0 0 1 0-1.5h3a.75.75 0 0 1 0 1.5Z";

const playSubTabs = [
  { kind: "releases" as const, icon: ICON_TAG },
  { kind: "mods" as const, icon: ICON_PACKAGE },
  { kind: "resourcepacks" as const, icon: ICON_PAINT },
  { kind: "shaderpacks" as const, icon: ICON_SUN },
  { kind: "saves" as const, icon: ICON_FOLDER },
  { kind: "console" as const, icon: ICON_TERMINAL },
];

const PHASE_KEYS: Record<string, string> = {
  "Подготовка...": "phase.prepare",
  "Скачивание сборки": "phase.download",
  "Распаковка архива": "phase.extract",
  "Установка модов": "phase.mods",
  "Применение overrides": "phase.overrides",
};

function phaseLabel(phase: string): string {
  const key = PHASE_KEYS[phase];
  return key ? t(key) : phase;
}

const ARCH_KEYS: Record<string, string> = {
  "64-бит": "java.arch64",
  "32-бит": "java.arch32",
  "недоступна": "java.archUnknown",
};

function javaArchLabel(arch: string): string {
  const key = ARCH_KEYS[arch];
  return key ? t(key) : arch;
}

const FILE_OVERSCAN = 10;
const fileListRef = ref<HTMLElement | null>(null);
const fileListScrollTop = ref(0);
const fileListViewportH = ref(480);
const fileRowStride = ref(64);

function measureFileRow() {
  const row = fileListRef.value?.querySelector(".file-row") as HTMLElement | null;
  if (row && row.offsetHeight > 0) {
    fileRowStride.value = row.offsetHeight + 8;
  }
}

function fileListScroll(e: Event) {
  const el = e.target as HTMLElement;
  fileListScrollTop.value = el.scrollTop;
  fileListViewportH.value = el.clientHeight;
  measureFileRow();
}

const fileListTotal = computed(
  () => fileListFiltered.value.length * fileRowStride.value - 8
);
function resetFileListScroll() {
  const el = fileListRef.value;
  if (el) el.scrollTop = 0;
}
const fileListStart = computed(() =>
  Math.max(0, Math.floor(fileListScrollTop.value / fileRowStride.value) - FILE_OVERSCAN)
);
const fileListFiltered = computed(() => {
  const list = gameFiles.value[playSubTab.value as GameFolderKind] ?? [];
  const q = fileSearch.value.trim().toLowerCase();
  if (!q) return list;
  return list.filter((f) => f.displayName.toLowerCase().includes(q) || f.name.toLowerCase().includes(q));
});
const fileListVisible = computed(() => {
  const list = fileListFiltered.value;
  if (!list.length) return list;
  const start = fileListStart.value;
  const end = Math.min(
    list.length,
    Math.ceil((fileListScrollTop.value + fileListViewportH.value) / fileRowStride.value) +
      FILE_OVERSCAN
  );
  return list.slice(start, Math.max(end, start + 1));
});
const fileVisibleCount = computed(() => fileListFiltered.value.length);

watch(
  () => [fileListFiltered.value.length, fileSearch.value],
  () => {
    fileListScrollTop.value = 0;
    nextTick(() => {
      resetFileListScroll();
      measureFileRow();
    });
  }
);

function isFileSelected(folder: string, entry: GameFileEntry): boolean {
  return selectedFiles.value[`${folder}/${entry.name}`] !== undefined;
}

function openSelected(site: "modrinth" | "curseforge") {
  const sel = Object.values(selectedFiles.value);
  for (const s of sel) {
    if (site === "modrinth") openFileOnModrinth(s.folder, s.entry);
    else openFileOnCurseForge(s.folder, s.entry);
  }
}

watch(playSubTab, () => {
  if (fileListRef.value) fileListRef.value.scrollTop = 0;
});

function gameFileIcon(folder: string, name: string): string {
  return fileIcons.value[`${folder}/${name}`] ?? "";
}

function onJavaChange(e: Event) {
  const val = (e.target as HTMLSelectElement).value;
  selectJava(val);
}

const expanded = ref<Record<string, boolean>>({});

type ChangelogLine = { type: "bullet" | "body" | "text"; text: string };

const CHANGELOG_PREVIEW_LINES = 8;

function changelogLines(body: string): ChangelogLine[] {
  if (!body) return [];
  return body
    .split("\n")
    .map((raw) => {
      const line = raw.replace(/\r$/, "").trim();
      if (!line) return null;
      if (/^[-*]\s+/.test(line)) {
        return { type: "bullet" as const, text: line.replace(/^[-*]\s+/, "") };
      }
      if (/^#+\s+/.test(line)) {
        return { type: "body" as const, text: line.replace(/^#+\s+/, "") };
      }
      return { type: "text" as const, text: line };
    })
    .filter((l): l is ChangelogLine => l !== null);
}

function isExpandable(body: string): boolean {
  return changelogLines(body).length > CHANGELOG_PREVIEW_LINES;
}

function isExpanded(tag: string): boolean {
  return expanded.value[tag] ?? false;
}

function toggleExpanded(tag: string) {
  expanded.value = { ...expanded.value, [tag]: !expanded.value[tag] };
}

function visibleLines(body: string): ChangelogLine[] {
  const lines = changelogLines(body);
  if (!isExpandable(body)) return lines;
  return isExpanded(body) ? lines : lines.slice(0, CHANGELOG_PREVIEW_LINES);
}

function isActiveRelease(tag: string): boolean {
  const v = versions.value;
  const active = v?.active;
  if (!active) return false;
  return v.installed.some((iv) => iv.version_id === active && iv.source_tag === tag) ?? false;
}

function playtimeForRelease(tag: string): number {
  return (
    versions.value?.installed.find((iv) => iv.source_tag === tag)?.total_seconds ?? 0
  );
}

// --- Рендер inline-markdown в ченджлоге (ссылки, жирный, код, зачёркнутый) ---

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function inlineStyle(s: string): string {
  return s
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
    .replace(/`([^`]+)`/g, "<code>$1</code>")
    .replace(/\*([^*]+)\*/g, "<em>$1</em>")
    .replace(/~~([^~]+)~~/g, "<del>$1</del>");
}

function renderInline(raw: string): string {
  let t = escapeHtml(raw.trim());
  t = inlineStyle(t);
  // Ссылки [текст](http...)
  t = t.replace(
    /\[([^\]]+)\]\((https?:\/\/[^\s)"<>]+)\)/g,
    (_, text: string, url: string) => `<a href="${url}">${text}</a>`
  );
  // Голые ссылки (минуя href уже вставленных <a>)
  t = t.replace(/(https?:\/\/[^\s)"<>]+)/g, (m: string, _g: string, offset: number) => {
    const before = t.slice(0, offset);
    const opens = before.match(/<a /g)?.length ?? 0;
    const closes = before.match(/<\/a>/g)?.length ?? 0;
    return opens > closes ? m : `<a href="${m}">${m}</a>`;
  });
  return t;
}

function onChangelogLinkClick(e: MouseEvent) {
  const target = e.target as HTMLElement;
  const anchor = target.closest("a");
  if (!anchor) return;
  const href = anchor.getAttribute("href");
  if (!href || !/^https?:\/\//i.test(href)) return;
  e.preventDefault();
  if (isTauri()) {
    openExternal(href).catch(() => window.open(href, "_blank"));
  } else {
    window.open(href, "_blank");
  }
}

function onPackChange(e: Event) {
  const val = (e.target as HTMLSelectElement).value;
  if (val) selectPack(val);
}

function newsKey(n: NewsItem): string {
  return `${n.kind}-${n.url || n.tag || n.title}`;
}

function visibleNewsLines(n: NewsItem) {
  const lines = changelogLines(n.body);
  if (!isNewsExpandable(n)) return lines;
  return isNewsExpanded(n) ? lines : lines.slice(0, CHANGELOG_PREVIEW_LINES);
}

function isNewsExpandable(n: NewsItem): boolean {
  return n.body.trim().length > 0 && changelogLines(n.body).length > CHANGELOG_PREVIEW_LINES;
}

function isNewsExpanded(n: NewsItem): boolean {
  return expanded.value[newsKey(n)] ?? false;
}

function toggleNewsExpanded(n: NewsItem) {
  expanded.value = { ...expanded.value, [newsKey(n)]: !isNewsExpanded(n) };
}

function isActiveNewsTag(tag: string): boolean {
  return isActiveRelease(tag);
}

function openNewsLink(url: string) {
  if (isTauri()) {
    openExternal(url).catch(() => window.open(url, "_blank"));
  } else {
    window.open(url, "_blank");
  }
}

function installNews(n: NewsItem) {
  if (!n.tag) return;
  if (n.pack_id !== packId.value) {
    selectPack(n.pack_id);
  }
  handleSelectVersion(n.tag);
}

function packNameFor(id: string): string {
  return packs.value.find((p) => p.id === id)?.name ?? id;
}
</script>
