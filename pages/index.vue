<template>
    <!-- Уведомления (тосты) -->
    <ToastNotifications />
  <div v-if="!isSearchWin && !isFileDetailWin" class="flex flex-col h-full w-full select-none overflow-hidden bg-[var(--panel)] text-[color:var(--tx)] font-sans">
    <!-- ==== Кастомный Titlebar (macOS Style) ==== -->
    <TitleBar />
    <div class="flex min-h-0 flex-1 w-full">
    <!-- Карточка обновления лаунчера -->
    <AppUpdateCard />
    <!-- ==== Боковая панель ==== -->
    <SideBar />

    <!-- ==== Основной контент ==== -->
    <main class="relative mx-auto h-full w-full flex-1 overflow-hidden rounded-tl-2xl border-l border-[var(--border)] bg-[var(--bg)]" style="max-width: 1500px">
      <!-- Scalable main column: width = user-draggable (наплыва inner-контента),
           по умолчанию растягивается на всю доступную ширину. -->
      <div class="relative flex h-full w-full">
        <div
          class="relative h-full min-w-0 flex-1 overflow-hidden"
          :style="mainWidth > 0 ? `max-width:${mainWidth}px` : ''"
        >
        <div class="mx-auto flex h-full w-full flex-col px-4 py-6 sm:px-6 md:px-8">
        <!-- ======= Вкладка: Релизы ======= -->
        <template v-if="tab === 'play'">
          <div class="flex min-h-0 flex-1 flex-col">
          <!-- Header сборки -->
          <div class="mb-6 shrink-0 border-b border-[var(--border)] pb-5">
            <div v-if="activeBanner && bannerOk" class="relative mb-4 h-44 w-full overflow-hidden rounded-xl border border-[var(--border)]">
              <img
                :src="activeBanner"
                :alt="activePack?.name ?? ''"
                class="h-full w-full object-cover"
                @error="bannerOk = false"
              />
              <div class="pointer-events-none absolute inset-0 bg-gradient-to-t from-[var(--bg)] via-[color-mix(in_srgb,var(--bg)_35%,transparent)] to-transparent"></div>
            </div>
            <div class="flex flex-wrap items-end justify-between gap-x-4 gap-y-3" :class="activeBanner && bannerOk ? '-mt-7 px-4' : ''">
              <!-- Левая часть: иконка сборки + название + мета -->
              <div class="flex min-w-0 flex-1 items-end gap-4">
                <img
                  v-if="activePack?.icon"
                  :src="convertFileSrc(activePack.icon)"
                  :alt="activePack.name"
                  class="h-[60px] w-[60px] shrink-0 rounded-xl border border-[var(--border)] bg-[var(--panel)] object-cover shadow-lg"
                  @error="(e: any) => (e.target.style.display = 'none')"
                />
                <div v-else class="flex h-[60px] w-[60px] shrink-0 items-center justify-center rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-lg">
                  <svg viewBox="0 0 16 16" class="h-6 w-6 fill-[var(--tx-muted)]">
                    <path d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-1 1v.878A2.25 2.25 0 1 1 2 13.378V2.5Z"/>
                  </svg>
                </div>
                <div class="min-w-0 pb-1">
                  <h1 class="truncate text-3xl font-bold leading-tight tracking-tight text-[color:var(--tx-strong)]">
                    {{ activePack?.name ?? t("pack.none") }}
                  </h1>
                  <p v-if="activePack?.author || loaderLabel" class="mt-1 flex flex-wrap items-center gap-x-2.5 gap-y-0.5 text-xs text-[color:var(--tx-muted)]">
                    <span v-if="activePack?.author" class="font-mono font-medium text-[var(--accent)]">@{{ activePack.author }}</span>
                    <span v-if="activePack?.author && loaderLabel" class="opacity-40">·</span>
                    <span v-if="loaderLabel">{{ loaderLabel }}</span>
                  </p>
                  <div class="mt-2.5 flex flex-wrap items-center gap-1.5">
                    <span
                      class="rounded-full px-2.5 py-0.5 text-[11px] font-bold uppercase tracking-wider border shadow-sm"
                      :class="status?.installed
                        ? 'border-[#238636]/40 bg-[#238636]/10 text-[#3fb950]'
                        : 'border-[var(--border)] bg-[var(--input)] text-[color:var(--tx-muted)]'"
                    >
                      {{ status?.installed ? t("pack.installed") : t("pack.notInstalled") }}
                    </span>
                    <span
                      v-if="activePack?.minRam"
                      class="inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs font-semibold"
                      :class="(ram * 1024) < activePack.minRam
                        ? 'border-[#f0883e]/50 bg-[#f0883e]/10 text-[#f0883e]'
                        : 'border-[var(--border)] bg-[var(--input)] text-[color:var(--tx-muted)]'"
                      :title="t('pack.minRamTitle', { min: activePack.minRam / 1024 })"
                    >
                      <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
                        <path d="M1 3.75C1 2.784 1.784 2 2.75 2h10.5c.966 0 1.75.784 1.75 1.75v5.5A1.75 1.75 0 0 1 13.25 11H10v1.25h.75a.75.75 0 0 1 0 1.5h-5.5a.75.75 0 0 1 0-1.5H6V11H2.75A1.75 1.75 0 0 1 1 9.25v-5.5Zm1.5 0v5.5c0 .138.112.25.25.25h10.5a.25.25 0 0 0 .25-.25v-5.5a.25.25 0 0 0-.25-.25H2.75a.25.25 0 0 0-.25.25ZM4 4.5a.75.75 0 0 1 .75-.75h6.5a.75.75 0 0 1 0 1.5h-6.5A.75.75 0 0 1 4 4.5Zm0 3a.75.75 0 0 1 .75-.75h6.5a.75.75 0 0 1 0 1.5h-6.5A.75.75 0 0 1 4 7.5Z"/>
                      </svg>
                      ≥ {{ activePack.minRam / 1024 }} {{ t("units.gb") }}
                    </span>
                    <span
                      v-if="status && status.playtime_seconds > 0"
                      class="inline-flex items-center gap-1 rounded-full border border-[var(--border)] bg-[var(--input)] px-2 py-0.5 text-xs font-semibold text-[color:var(--tx-muted)]"
                      :title="t('pack.playtimeTitle', { time: formatPlaytime(status.playtime_seconds) })"
                    >
                      <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
                        <path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0Zm0 1.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13ZM7.25 3.5a.75.75 0 0 1 .75.75V7.8l2.58 1.55a.75.75 0 1 1-.77 1.28L7.18 9.1a.75.75 0 0 1-.43-.68V4.25a.75.75 0 0 1 .75-.75Z"/>
                      </svg>
                      {{ formatPlaytimeShort(status.playtime_seconds) }}
                    </span>
                    <span
                      v-else-if="status && status.installed"
                      class="inline-flex items-center gap-1 rounded-full border border-dashed border-[var(--border)] bg-[var(--panel-soft)] px-2 py-0.5 text-xs font-medium text-[color:var(--tx-muted)]"
                      :title="t('pack.notPlayedTitle')"
                    >
                      {{ t("pack.notPlayed") }}
                    </span>
                  </div>
                </div>
              </div>

              <!-- Правая часть: главное действие + вторичные кнопки -->
              <div class="flex shrink-0 flex-col items-end gap-2 pb-1">
                <button
                  type="button"
                  class="flex items-center justify-center gap-2 rounded-xl px-6 py-2.5 text-sm font-bold tracking-wide text-white shadow-md transition-all active:scale-[0.98] focus-visible:outline focus-visible:outline-offset-2 disabled:cursor-not-allowed disabled:opacity-50 disabled:active:scale-100"
                  :class="status?.installed
                    ? gameRunning
                      ? 'bg-[#b91c1c] hover:bg-[#dc2626]'
                      : 'bg-[#238636] hover:bg-[#2ea043]'
                    : 'bg-[var(--accent-deep)] hover:bg-[var(--accent-hover)]'"
                  :disabled="busy"
                  @click="status?.installed ? (gameRunning ? handleStop() : handlePlay()) : handleInstall()"
                >
                  <svg v-if="busy" viewBox="0 0 16 16" class="h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
                  <svg v-else-if="status?.installed && !gameRunning" viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M4.5 1.94a1 1 0 0 1 1.523-.853l9.6 6.06a1 1 0 0 1 0 1.707l-9.6 6.06A1 1 0 0 1 4.5 14.06V1.94Z"/></svg>
                  <svg v-else-if="gameRunning" viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.5 3.5h9v9h-9z"/></svg>
                  <svg v-else viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M2.75 14A1.75 1.75 0 0 1 1 12.25v-2.5a.75.75 0 0 1 1.5 0v2.5c0 .138.112.25.25.25h10.5a.25.25 0 0 0 .25-.25v-2.5a.75.75 0 0 1 1.5 0v2.5A1.75 1.75 0 0 1 13.25 14Z"/><path d="M7.97 2.72a.75.75 0 0 1 1.06 0l3 3a.75.75 0 1 1-1.06 1.06l-1.72-1.72v6.69a.75.75 0 0 1-1.5 0v-6.69L6.03 6.78a.75.75 0 0 1-1.06-1.06l3-3Z"/></svg>
                  <template v-if="!status?.installed">{{ busy ? t("side.installing") : t("side.downloadPlay") }}</template>
                  <template v-else>{{ busy ? t("side.launching") : gameRunning ? t("side.stopGame") : t("side.play") }}</template>
                </button>
                <div class="flex items-center gap-1.5">
                  <button
                    type="button"
                    class="flex items-center gap-1.5 rounded-lg border border-[var(--border)] bg-[var(--input)] px-2.5 py-1.5 text-xs font-medium text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)]"
                    :title="t('pack.openDir')"
                    @click="handleOpenPackDir"
                  >
                    <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
                      <path d="M.513 1.513A1.75 1.75 0 0 1 1.75 1h3.5c.55 0 1.07.26 1.4.7l.9 1.2a.25.25 0 0 0 .2.1H13a1 1 0 0 1 1 1v.5H2.75a.75.75 0 0 0 0 1.5h11.978a1 1 0 0 1 .994 1.117L15 13.25A1.75 1.75 0 0 1 13.25 15H1.75A1.75 1.75 0 0 1 0 13.25V2.75c0-.464.184-.91.513-1.237Z"/>
                    </svg>
                    {{ t("pack.folder") }}
                  </button>
                  <template v-if="activePack?.kind === 'local' && status?.installed">
                    <div ref="exportMenuRef" class="relative">
                      <button
                        type="button"
                        class="flex items-center gap-1.5 rounded-lg border border-[var(--border)] bg-[var(--input)] px-2.5 py-1.5 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)]"
                        :title="t('pack.exportTitle')"
                        :disabled="exportBusy"
                        @click="exportMenuOpen = !exportMenuOpen"
                      >
                        <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
                          <path d="M7.97.72a.75.75 0 0 1 1.06 0l3 3a.75.75 0 1 1-1.06 1.06L9 2.81v6.94a.75.75 0 0 1-1.5 0V2.81L5.53 4.78a.75.75 0 0 1-1.06-1.06l3-3Z"/>
                          <path d="M2.5 13.25a.75.75 0 0 1 .75.75c0 .138.112.25.25.25h9a.25.25 0 0 0 .25-.25.75.75 0 0 1 1.5 0 1.75 1.75 0 0 1-1.75 1.75h-9A1.75 1.75 0 0 1 1.75 14a.75.75 0 0 1 .75-.75Z"/>
                        </svg>
                        <span>{{ t("pack.exportBtn") }}</span>
                        <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current opacity-60"><path d="m4.22 6 3.72 3.72a.75.75 0 0 0 1.06 0L12.72 6l-1.06-1.06L8 8.09 5.28 4.94 4.22 6Z"/></svg>
                      </button>
                      <div
                        v-if="exportMenuOpen"
                        class="absolute right-0 top-[calc(100%+4px)] z-50 flex w-44 flex-col overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-1 shadow-xl"
                      >
                        <button
                          type="button"
                          class="flex items-center gap-2 rounded px-2 py-1.5 text-left text-xs text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                          :disabled="exportBusy"
                          @click="exportMenuOpen = false; openExport('mrpack')"
                        >
                          <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current opacity-70"><path d="M8 1.5A2.75 2.75 0 0 0 5.5 3.25a.75.75 0 0 1-1.5 0A4.25 4.25 0 0 1 9 1.075 4.25 4.25 0 0 1 13.2 4.5a.75.75 0 0 1-1.47.27A2.751 2.751 0 0 0 8 1.5Zm-4.5 8a2.75 2.75 0 0 1 2.5-1.75h.22a.75.75 0 0 0 .71-.51A3.75 3.75 0 0 1 8 5.25a3.75 3.75 0 0 1 1.07 1.99.75.75 0 0 0 .71.51h.22A2.75 2.75 0 0 1 12.5 9.5 2.75 2.75 0 0 1 9.75 12.25h-3.5A2.75 2.75 0 0 1 3.5 9.5Z"/><path d="M8 7.25a.75.75 0 0 1 .75.75v4.19l.97-.97a.75.75 0 1 1 1.06 1.06l-2.25 2.25a.75.75 0 0 1-1.06 0l-2.25-2.25a.75.75 0 1 1 1.06-1.06l.97.97V8a.75.75 0 0 1 .75-.75Z"/></svg>
                          .mrpack
                        </button>
                        <button
                          type="button"
                          class="flex items-center gap-2 rounded px-2 py-1.5 text-left text-xs text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                          :disabled="exportBusy"
                          @click="exportMenuOpen = false; openAuthorExport()"
                        >
                          <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current opacity-70"><path d="M7.25 1.75a.75.75 0 0 1 1.5 0v5.5h5.5a.75.75 0 0 1 0 1.5h-5.5v5.5a.75.75 0 0 1-1.5 0v-5.5h-5.5a.75.75 0 0 1 0-1.5h5.5v-5.5Z"/></svg>
                          {{ t("pack.exportAuthorShort") }}
                        </button>
                        <button
                          type="button"
                          class="flex items-center gap-2 rounded px-2 py-1.5 text-left text-xs text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                          :disabled="exportBusy"
                          @click="exportMenuOpen = false; openExport('curseforge')"
                        >
                          <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current opacity-70"><path d="M8 1.5A2.75 2.75 0 0 0 5.5 3.25a.75.75 0 0 1-1.5 0A4.25 4.25 0 0 1 9 1.075 4.25 4.25 0 0 1 13.2 4.5a.75.75 0 0 1-1.47.27A2.751 2.751 0 0 0 8 1.5Zm-4.5 8a2.75 2.75 0 0 1 2.5-1.75h.22a.75.75 0 0 0 .71-.51A3.75 3.75 0 0 1 8 5.25a3.75 3.75 0 0 1 1.07 1.99.75.75 0 0 0 .71.51h.22A2.75 2.75 0 0 1 12.5 9.5 2.75 2.75 0 0 1 9.75 12.25h-3.5A2.75 2.75 0 0 1 3.5 9.5Z"/><path d="M8 7.25a.75.75 0 0 1 .75.75v4.19l.97-.97a.75.75 0 1 1 1.06 1.06l-2.25 2.25a.75.75 0 0 1-1.06 0l-2.25-2.25a.75.75 0 1 1 1.06-1.06l.97.97V8a.75.75 0 0 1 .75-.75Z"/></svg>
                          CurseForge
                        </button>
                      </div>
                    </div>
                  </template>
                </div>
              </div>
            </div>

            <p class="mt-2 text-xs text-[color:var(--tx-muted)] flex items-center gap-2">
              <span>{{ t("pack.mono") }}</span>
              <span>•</span>
              <span v-if="loaderLabel">{{ t("pack.loader", { name: loaderLabel }) }}</span>
              <button
                v-if="activePack?.kind === 'local'"
                type="button"
                class="inline-flex items-center gap-1 rounded border border-[var(--border)] bg-[var(--input)] px-1.5 py-0.5 text-[11px] font-medium text-[var(--accent)] transition-colors hover:border-[color-mix(in_srgb,var(--accent)_50%,transparent)] hover:bg-[var(--hover)]"
                :title="t('pack.versionChange')"
                @click="openEditVersion"
              >
                <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M11.013 1.427a1.75 1.75 0 0 1 2.474 0l1.086 1.086a1.75 1.75 0 0 1 0 2.474l-8.61 8.61c-.21.21-.47.364-.756.445l-3.251 1.302a.75.75 0 0 1-.993-.993l1.302-3.251a1.75 1.75 0 0 1 .445-.756l8.61-8.61Z"/></svg>
                {{ t("pack.versionChange") }}
              </button>
              <span v-if="activePack?.author">•</span>
              <span v-if="activePack?.author" class="font-mono text-[var(--accent)]">@{{ activePack.author }}</span>
              <button
                v-if="activePackRepo"
                type="button"
                class="inline-flex items-center gap-1.5 rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1 text-xs font-medium text-[color:var(--tx)] transition-colors hover:border-[color-mix(in_srgb,var(--accent)_50%,transparent)] hover:text-[var(--accent)]"
                :title="activePackRepo"
                @click="openExternal(activePackRepo)"
              >
                <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                  <path d="M6.75 2.75h2.5a.75.75 0 0 1 0 1.5h-1.72l4.29 4.29a.75.75 0 0 1-1.06 1.06L6.47 5.31v1.69a.75.75 0 0 1-1.5 0v-3.5a.75.75 0 0 1 .75-.75Z"/>
                  <path d="M2.25 5.75A1.75 1.75 0 0 1 4 4h2.75a.75.75 0 0 1 0 1.5H4v6.5h6.5v-2.5a.75.75 0 0 1 1.5 0V11A1.75 1.75 0 0 1 10.25 12.75H4A1.75 1.75 0 0 1 2.25 11V5.75Z"/>
                  <path d="M11.75 7.25a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 .75.75v1.5a.75.75 0 0 1-1.5 0V7.75h-1.5a.75.75 0 0 1-.75-.75Z"/>
                </svg>
                {{ t("pack.repo") }}
              </button>
              <button
                v-if="activePackRepo"
                type="button"
                class="inline-flex items-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_10%,transparent)] px-2.5 py-1 text-xs font-medium text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)]"
                :title="t('pack.reportBugTitle')"
                @click="reportPackBug()"
              >
                <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                  <path d="M8 1c-1.04 0-1.9.81-2 1.84-.1-.06-.21-.11-.32-.16l-.12-.05a1.75 1.75 0 0 0-1.3 3.24c-.5.6-.8 1.36-.8 2.2v.6h.75a.75.75 0 0 1 0 1.5H3a.75.75 0 0 1 0-1.5h.3v-.6a2.6 2.6 0 0 0-.2-1 .75.75 0 0 1 .9-1c.3.2.57.4.8.65V5.8c0-.33.05-.65.16-.95.07.71.63 1.29 1.34 1.38L6.2 6.2c.44.15.86.34 1.24.58.44.28.81.58 1.12.9.47.5.94 1.15 1.44 1.94.2.32.3.69.3 1.07v1.31c.48.1.94.3 1.33.58.31.22.7.32 1.07.28a.9.9 0 1 1 .1 1.8c-.8.08-1.59-.22-2.22-.76-.43-.36-.8-.56-1.14-.61v.69c0 .64-.19 1.24-.52 1.74.8.37 1.3 1.18 1.3 2.16 0 .55-.45 1-1 1H8.25c-.55 0-1-.45-1-1s.45-1 1-1H9v-2.32c-.26.2-.55.35-.87.45-.46.14-.96.14-1.42 0a2.77 2.77 0 0 1-.71-.32V15.5c0 .55-.45 1-1 1H3.75c-.55 0-1-.45-1-1s.45-1 1-1h.61c-.58-.62-1-1.09-1.3-1.42l-.18-.17a2.25 2.25 0 0 1-1.68-2.19v-5.8c0-1.14.84-2.08 1.92-2.26A2 2 0 0 1 3.96.88l.03-.02A2 2 0 0 1 6.09 1H8Z"/>
                </svg>
                {{ t("pack.reportBug") }}
              </button>
            </p>

            <div v-if="updateInfo?.has_update && updateInfo.latest_version" class="mt-4 flex items-center justify-between gap-4 rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_10%,transparent)] px-3.5 py-2.5 text-xs text-[var(--accent)]">
              <span class="min-w-0">
                {{ t("update.available") }} <strong class="text-[var(--accent-strong)]">{{ updateInfo.latest_version }}</strong>
                <span v-if="updateInfo.current_version" class="text-[color:var(--tx-muted)]">
                  {{ t("update.installed", { v: updateInfo.current_version }) }}
                </span>
              </span>
              <button
                type="button"
                class="shrink-0 rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] disabled:opacity-50"
                :disabled="busy"
                @click="handleUpdate"
              >
                {{ t("update.btn") }}
              </button>
            </div>

            <!-- Подписка Boosty: статус/привязка токена -->
            <div
              v-if="activePack?.boostyBlog"
              class="mt-4 rounded-md border px-3.5 py-2.5 text-xs"
              :class="licenseInfo?.subscribed
                ? 'border-[#238636]/40 bg-[#238636]/10 text-[#3fb950]'
                : 'border-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_10%,transparent)]'"
            >
              <div v-if="licenseInfo?.subscribed" class="flex items-center justify-between gap-3">
                <span class="flex min-w-0 items-center gap-2">
                  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current">
                    <path d="M7.75.5A4.5 4.5 0 0 1 11.5 5.5v.85A4.5 4.5 0 0 1 13 10v3A2.5 2.5 0 0 1 10.5 15.5h-6A2.5 2.5 0 0 1 2 13v-3a4.5 4.5 0 0 1 1.5-3.35V5.5A4.25 4.25 0 0 1 7.75.5Zm0 1.5a2.75 2.75 0 0 0-2.75 2.75v.5h5.5v-.5A2.75 2.75 0 0 0 7.75 2Z"/>
                  </svg>
                  <span class="min-w-0">
                    {{
                      licenseInfo.expiresAt
                        ? t("license.active", {
                            blog: licenseInfo.blog,
                            until: formatUnixDate(licenseInfo.expiresAt),
                          })
                        : t("license.activeNoExpiry", { blog: licenseInfo.blog })
                    }}
                  </span>
                </span>
                <button
                  type="button"
                  class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1 text-xs font-medium text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)] disabled:opacity-50"
                  :disabled="licenseBusy"
                  @click="removeLicense"
                >
                  {{ t("license.remove") }}
                </button>
              </div>
              <div
                v-if="licenseInfo?.requiredTiers.length"
                class="mt-1.5 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-xs text-[color:var(--tx-muted)]"
              >
                <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 shrink-0 fill-current">
                  <path d="M4.5 7.5a3.5 3.5 0 1 1 7 0v1h.75A.75.75 0 0 1 13 9.25v4A.75.75 0 0 1 12.25 14h-8.5a.75.75 0 0 1-.75-.75v-4A.75.75 0 0 1 3.75 8.5h.75v-1Zm1.5 1v-1a2 2 0 0 1 4 0v1h-4Z"/>
                </svg>
                <span>
                  {{
                    licenseInfo.tier
                      ? t("license.tierOk", {
                          tier: licenseInfo.tier,
                          list: licenseInfo.requiredTiers.join(" / "),
                        })
                      : t("license.tierList", { list: licenseInfo.requiredTiers.join(" / ") })
                  }}
                </span>
              </div>
              <template v-else>
                <div class="flex items-center gap-2">
                  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current">
                    <path d="M7.75.5A4.5 4.5 0 0 1 11.5 5.5v.85A4.5 4.5 0 0 1 13 10v3A2.5 2.5 0 0 1 10.5 15.5h-6A2.5 2.5 0 0 1 2 13v-3a4.5 4.5 0 0 1 1.5-3.35V5.5A4.25 4.25 0 0 1 7.75.5Zm0 1.5a2.75 2.75 0 0 0-2.75 2.75v.5h5.5v-.5A2.75 2.75 0 0 0 7.75 2Z"/>
                  </svg>
                  <span class="min-w-0">
                    {{
                      licenseInfo?.requiredTiers.length
                        ? t("license.requiredTier", {
                            blog: activePack.boostyBlog,
                            list: licenseInfo.requiredTiers.join(" / "),
                          })
                        : t("license.required", { blog: activePack.boostyBlog })
                    }}
                  </span>
                </div>
                <div v-if="licenseError" class="mt-1.5 text-[color:var(--tx-muted)]">
                  {{ licenseError }}
                </div>
                <div class="mt-2">
                  <button
                    v-if="!boostyAuthOpen"
                    type="button"
                    class="w-full rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] disabled:opacity-50"
                    :disabled="licenseBusy"
                    @click="startBoostyLogin()"
                  >
                    {{ t("license.oauth") }}
                  </button>
                  <div
                    v-else
                    class="flex items-center justify-between gap-3 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs text-[color:var(--tx-muted)]"
                  >
                    <span class="flex items-center gap-2">
                      <svg class="h-3.5 w-3.5 animate-spin" viewBox="0 0 24 24" fill="none">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 0 1 8-8v3a5 5 0 0 0-5 5H4z"/>
                      </svg>
                      {{ t("license.waiting") }}
                    </span>
                    <button type="button" class="text-[var(--accent)] hover:underline" @click="cancelBoostyLogin">
                      {{ t("license.cancel") }}
                    </button>
                  </div>
                  <div class="mt-2 flex items-center gap-2 text-xs text-[color:var(--tx-muted)]">
                    <span class="h-px flex-1 bg-[var(--border)]"></span>
                    <span>{{ t("license.orManual") }}</span>
                    <span class="h-px flex-1 bg-[var(--border)]"></span>
                  </div>
                  <form class="mt-2 flex gap-2" @submit.prevent="saveLicense">
                    <input
                      v-model="licenseKeyInput"
                      type="text"
                      :placeholder="t('license.placeholder')"
                      autocomplete="off"
                      spellcheck="false"
                      class="min-w-0 flex-1 rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1.5 font-mono text-xs text-[color:var(--tx)] placeholder:text-[color:var(--tx-muted)] focus:border-[color-mix(in_srgb,var(--accent)_50%,transparent)] focus:outline-none"
                    />
                    <button
                      type="submit"
                      class="shrink-0 rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] disabled:opacity-50"
                      :disabled="licenseBusy || !licenseKeyInput.trim()"
                    >
                      {{ t("license.activate") }}
                    </button>
                  </form>
                </div>
                <div class="mt-2 flex flex-wrap items-center gap-2 text-xs text-[color:var(--tx-muted)]">
                  <span>{{ t("license.howTo") }}</span>
                  <button
                    type="button"
                    class="text-[var(--accent)] hover:underline"
                    @click="openExternal(`https://boosty.to/${activePack.boostyBlog}`)"
                  >
                    {{ t("license.openBlog") }} →
                  </button>
                </div>
              </template>
            </div>

            <!-- Предупреждение о кастомных файлах (не с Modrinth/CurseForge) -->
            <div
              v-if="warnCustomMods && status?.installed && status.custom_mods.length > 0"
              class="mt-4 rounded-md border border-[#9e6a03]/40 bg-[#9e6a03]/10 px-3.5 py-2.5 text-xs text-[#d29922]"
            >
              <button
                type="button"
                class="flex w-full items-center justify-between gap-3 text-left"
                @click="customModsOpen = !customModsOpen"
              >
                <span class="flex min-w-0 items-center gap-2">
                  <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current">
                    <path d="M8 0a8 8 0 1 0 0 16A8 8 0 0 0 8 0ZM8 7.25a.74.74 0 0 1 .74.75v2.5a.74.74 0 0 1-1.48 0V8a.74.74 0 0 1 .74-.75Zm0 5.25a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z"/>
                  </svg>
                  <span class="min-w-0">
                    {{ t("warn.customMods", { n: status.custom_mods.length }) }}
                  </span>
                </span>
                <span class="shrink-0 underline decoration-dotted underline-offset-2">
                  {{ customModsOpen ? t("warn.hide") : t("warn.show") }}
                </span>
              </button>
              <div v-if="customModsOpen" class="mt-2 space-y-1 border-t border-[#9e6a03]/30 pt-2">
                <ul class="space-y-1 font-mono text-xs">
                  <li v-for="f in status.custom_mods" :key="f.path" class="flex items-start gap-2">
                    <span class="truncate" :title="f.url">{{ f.path }}</span>
                  </li>
                </ul>
                <p class="pt-1 text-[#e3b341]">{{ t("warn.note") }}</p>
              </div>
            </div>
          </div>

          <!-- Сабтабы: релизы / моды / ресурспаки / шейдеры / миры / консоль -->
          <div class="mb-5 flex shrink-0 items-center gap-1 overflow-x-auto border-b border-[var(--border)]" style="scrollbar-width: thin">
            <template v-for="st in playSubTabsVisible" :key="st.kind">
              <span v-if="st.kind === 'screenshots'" class="mx-1.5 my-2 h-5 w-px shrink-0 bg-[var(--border)]"></span>
              <button
                type="button"
                class="relative flex shrink-0 items-center gap-1.5 px-3 pb-2.5 pt-1 text-xs font-semibold transition-colors"
                :class="playSubTab === st.kind
                  ? 'text-[var(--accent)]'
                  : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
                @click="playSubTab = st.kind"
              >
                <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current" v-html="st.icon"></svg>
                <span>{{ t("sub." + st.kind) }}</span>
                <span
                  v-if="subTabCount(st.kind) > 0"
                  class="rounded-full bg-[var(--input)] px-1.5 py-px text-[10px] font-bold tabular-nums"
                  :class="playSubTab === st.kind ? 'text-[var(--accent)]' : 'text-[color:var(--tx-muted)]'"
                >{{ subTabCount(st.kind) }}</span>
                <span v-if="playSubTab === st.kind" class="absolute inset-x-2 bottom-0 h-[2.5px] rounded-t-full bg-[var(--accent)]"></span>
              </button>
            </template>
          </div>

          <!-- Список установленных версий -->
          <template v-if="playSubTab === 'releases'">
          <div v-if="versions && versions.installed.length > 0" class="min-h-0 flex-1 space-y-4 overflow-y-auto pr-1">
            <div class="flex items-center justify-between text-xs text-[color:var(--tx-muted)]">
              <span class="font-medium">{{ t("releases.count", { n: versions.installed.length }) }}</span>
            </div>

            <article
              v-for="r in versions.installed"
              :key="r.version_id"
              class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm transition-shadow hover:shadow-md"
            >
              <div class="flex items-center justify-between px-4 py-3">
                <div class="flex items-center gap-2.5 flex-wrap">
                  <span class="font-mono text-sm font-semibold text-[var(--accent)]">
                    {{ r.source_tag ?? r.version_id }}
                  </span>
                  <span v-if="r.version_id === versions.active" class="rounded-full border border-[#238636]/40 bg-[#238636]/10 px-2 py-0.2 text-[11px] font-medium text-[#3fb950]">
                    {{ t("releases.active") }}
                  </span>
                </div>

                <div class="flex items-center gap-3">
                  <span v-if="r.total_seconds > 0" class="font-mono text-xs text-[#d29922]" :title="t('releases.playtime')">
                    {{ formatPlaytime(r.total_seconds) }}
                  </span>
                  <button
                    v-if="r.version_id !== versions.active"
                    type="button"
                    class="rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] hover:text-white disabled:opacity-50"
                    :disabled="busy"
                    @click="handleSelectVersion(r.source_tag ?? r.version_id)"
                  >
                    {{ t("releases.switch") }}
                  </button>
                </div>
              </div>
            </article>
          </div>

          <div v-else class="shrink-0 rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-8 text-center text-xs text-[color:var(--tx-muted)]">
            {{ t("releases.loadError") }}
          </div>
          </template>

          <!-- Папки файлов игры: моды / ресурспаки / шейдеры / миры -->
          <div
            v-else-if="playSubTab === 'mods' || playSubTab === 'resourcepacks' || playSubTab === 'shaderpacks' || playSubTab === 'saves'"
            class="flex min-h-0 flex-1 flex-col"
          >
            <div
              v-if="packLocked"
              class="mb-3 flex shrink-0 items-center justify-between gap-3 rounded-md border border-[color-mix(in_srgb,var(--accent)_35%,transparent)] bg-[color-mix(in_srgb,var(--accent)_8%,transparent)] px-3 py-2 text-xs text-[color:var(--tx)]"
            >
              <span class="flex items-center gap-2">
                <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 shrink-0 fill-[var(--accent)]"><path d="M8 1a4.25 4.25 0 0 0-4.25 4.25V7H3.5A1.5 1.5 0 0 0 2 8.5v5A1.5 1.5 0 0 0 3.5 15h9a1.5 1.5 0 0 0 1.5-1.5v-5A1.5 1.5 0 0 0 12.5 7h-.25V5.25A4.25 4.25 0 0 0 8 1Zm2.5 6h-5V5.25a2.5 2.5 0 0 1 5 0Z"/></svg>
                {{ t("files.locked") }}
              </span>
              <button
                type="button"
                class="shrink-0 rounded px-2 py-1 font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_15%,transparent)]"
                :title="t('files.unbindHint')"
                @click="confirmUnbindPack"
              >{{ unbindArmed ? t("files.unbindConfirm") : t("files.unbind") }}</button>
            </div>
            <div class="mb-3 flex shrink-0 items-center justify-between gap-3">
              <span class="flex shrink-0 items-center gap-2 text-xs text-[color:var(--tx-muted)]">
                {{ playSubTab === "saves" ? t("files.worldsCount", { n: fileVisibleCount }) : t("files.count", { n: fileVisibleCount }) }}
                <span
                  v-if="playSubTab !== 'saves' && fileVisibleCount > 0"
                  class="rounded-full border border-[color-mix(in_srgb,var(--accent)_35%,transparent)] bg-[color-mix(in_srgb,var(--accent)_8%,transparent)] px-2 py-0.5 text-[11px] font-medium text-[var(--accent)]"
                >
                  {{ t("files.enabledOf", { n: enabledCountIn(playSubTab as GameFolderKind), m: fileVisibleCount }) }}
                </span>
              </span>
              <div class="flex min-w-0 flex-wrap items-center gap-2">
                <template v-if="playSubTab !== 'saves'">
                  <button
                    v-if="modUpdatesTab.length > 0"
                    type="button"
                    class="flex shrink-0 items-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                    :disabled="updateAllBusy || updatingMod !== null || packLocked"
                    @click="updateAllMods"
                  >
                    <svg v-if="updateAllBusy" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                      <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                    </svg>
                    <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                      <path d="M2.75 14A1.75 1.75 0 0 1 1 12.25v-2.5a.75.75 0 0 1 1.5 0v2.5c0 .138.112.25.25.25h10.5a.25.25 0 0 0 .25-.25v-2.5a.75.75 0 0 1 1.5 0v2.5A1.75 1.75 0 0 1 13.25 14Z"/>
                      <path d="M7.97 2.72a.75.75 0 0 1 1.06 0l3 3a.75.75 0 1 1-1.06 1.06l-1.72-1.72v6.69a.75.75 0 0 1-1.5 0v-6.69L6.03 6.78a.75.75 0 0 1-1.06-1.06l3-3Z"/>
                    </svg>
                    {{ t("mods.updateAll") }}
                    <span class="rounded-full bg-[var(--accent)] px-1.5 text-[11px] font-bold text-[var(--bg)]">{{ modUpdatesTab.length }}</span>
                  </button>
                  <button
                    type="button"
                    class="flex shrink-0 items-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                    :title="t('mods.addHint')"
                    :disabled="packLocked"
                    @click="openSearch((playSubTab === 'mods' ? 'mod' : playSubTab === 'resourcepacks' ? 'resourcepack' : 'shaderpack') as ModrinthSearchKind, 'modrinth')"
                  >
                    <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                      <path d="M8 2.75a.75.75 0 0 1 .75.75v3.75h3.75a.75.75 0 0 1 0 1.5h-3.75v3.75a.75.75 0 0 1-1.5 0V8.75H3.5a.75.75 0 0 1 0-1.5h3.75V3.5A.75.75 0 0 1 8 2.75Z"/>
                    </svg>
                    {{ playSubTab === 'mods' ? t("mods.add") : playSubTab === 'resourcepacks' ? t("mods.addRP") : t("mods.addShaders") }}
                  </button>
                  <button
                    v-if="playSubTab === 'mods'"
                    type="button"
                    class="flex shrink-0 items-center gap-1.5 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-1.5 text-xs font-semibold text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[var(--tx)]"
                    :title="t('scanner.hint')"
                    @click="openModScanner"
                  >
                    <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                      <path d="M8 1.25a.75.75 0 0 1 .75.75v1.5a.75.75 0 0 1-1.5 0V2A.75.75 0 0 1 8 1.25Zm0 9.75a1.75 1.75 0 1 0 0-3.5 1.75 1.75 0 0 0 0 3.5Zm0 1.5a3.25 3.25 0 1 0 0-6.5 3.25 3.25 0 0 0 0 6.5Zm6.75-4.75a.75.75 0 0 0-1.5 0V8a.75.75 0 0 0 1.5 0V7.75ZM8 12.5a.75.75 0 0 1 .75.75V14a.75.75 0 0 1-1.5 0v-.75A.75.75 0 0 1 8 12.5Zm-5.25-4.75a.75.75 0 0 1 .75.75v.25a.75.75 0 0 1-1.5 0V8.5a.75.75 0 0 1 .75-.75Zm8.96-4.46a.75.75 0 0 1 0 1.06l-1.06 1.06a.75.75 0 1 1-1.06-1.06l1.06-1.06a.75.75 0 0 1 1.06 0Zm-8.42 8.42a.75.75 0 0 1 0 1.06L2.23 14.53a.75.75 0 0 1-1.06-1.06l1.06-1.06a.75.75 0 0 1 1.06 0Zm-1.06-8.42a.75.75 0 0 1 1.06 0l1.06 1.06A.75.75 0 1 1 3.29 4.89L2.23 3.83a.75.75 0 0 1 0-1.06Z"/>
                    </svg>
                    {{ t("scanner.btn") }}
                  </button>
                </template>
                <template v-else>
                  <button
                    type="button"
                    class="flex shrink-0 items-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)]"
                    :title="t('mods.addHint')"
                    @click="openSearch('datapack', 'modrinth')"
                  >
                    <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                      <path d="M8 2.75a.75.75 0 0 1 .75.75v3.75h3.75a.75.75 0 0 1 0 1.5h-3.75v3.75a.75.75 0 0 1-1.5 0V8.75H3.5a.75.75 0 0 1 0-1.5h3.75V3.5A.75.75 0 0 1 8 2.75Z"/>
                    </svg>
                    {{ t("mods.addDatapack") }}
                  </button>
                </template>
                <div v-if="Object.keys(selectedFiles).length > 0" class="flex shrink-0 items-center gap-1.5">
                  <span class="text-xs text-[color:var(--tx-muted)]">
                    {{ t("files.selected", { n: Object.keys(selectedFiles).length }) }}
                  </span>
                  <button
                    type="button"
                    class="rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1.5 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] hover:text-white"
                    :title="t('files.enableSel')"
                    @click="setSelectedFilesEnabled(true)"
                  >
                    {{ t("files.enable") }}
                  </button>
                  <button
                    type="button"
                    class="rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1.5 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] hover:text-white"
                    :title="t('files.disableSel')"
                    @click="setSelectedFilesEnabled(false)"
                  >
                    {{ t("files.disable") }}
                  </button>
                  <button
                    type="button"
                    class="flex items-center gap-1 rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1.5 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] hover:text-white"
                    @click="openSelected('modrinth')"
                  >
                    <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M7.775 3.275a.75.75 0 0 0 1.06 1.06l1.25-1.25v11.165a.75.75 0 0 0 1.5 0V2.085l1.25 1.25a.75.75 0 0 0 1.06-1.06L9.56.53a.75.75 0 0 0-1.06 0L7.775 3.275Z"/></svg>
                    Modrinth
                  </button>
                  <button
                    type="button"
                    class="flex items-center gap-1 rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1.5 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] hover:text-white"
                    @click="openSelected('curseforge')"
                  >
                    <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M7.775 3.275a.75.75 0 0 0 1.06 1.06l1.25-1.25v11.165a.75.75 0 0 0 1.5 0V2.085l1.25 1.25a.75.75 0 0 0 1.06-1.06L9.56.53a.75.75 0 0 0-1.06 0L7.775 3.275Z"/></svg>
                    CurseForge
                  </button>
                  <button
                    type="button"
                    class="rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-xs font-medium text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-white"
                    @click="clearFileSelection()"
                  >
                    {{ t("files.clear") }}
                  </button>
                  <template v-if="!fileDeleteArmed">
                    <button
                      type="button"
                      class="flex items-center gap-1 rounded-md border border-[#f85149]/40 bg-[#f85149]/10 px-2 py-1 text-xs font-semibold text-[#f85149] transition-colors hover:bg-[#f85149]/20 disabled:opacity-50"
                      :disabled="fileDeleteBusy || packLocked"
                      :title="t('files.deleteSelHint')"
                      @click="fileDeleteArmed = true"
                    >
                      <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M5 1.75A1.75 1.75 0 0 1 6.75 0h2.5A1.75 1.75 0 0 1 11 1.75V3h3a.75.75 0 0 1 0 1.5h-.75l-.77 9.006A1.75 1.75 0 0 1 10.738 15H5.262a1.75 1.75 0 0 1-1.742-1.494L2.75 4.5H2a.75.75 0 0 1 0-1.5h3V1.75ZM6.5 3h3V1.75a.25.25 0 0 0-.25-.25h-2.5a.25.25 0 0 0-.25.25V3ZM4.26 4.5l.763 8.91a.25.25 0 0 0 .249.214h5.456a.25.25 0 0 0 .249-.214L11.74 4.5H4.26Zm2.36 2.25a.75.75 0 0 1 1.5 0v4.5a.75.75 0 0 1-1.5 0v-4.5Zm3 0a.75.75 0 0 1 1.5 0v4.5a.75.75 0 0 1-1.5 0v-4.5Z"/></svg>
                      {{ t("files.delete") }}
                    </button>
                  </template>
                  <template v-else>
                    <button
                      type="button"
                      class="flex items-center gap-1 rounded-md border border-[#f85149]/40 bg-[#f85149]/10 px-2 py-1 text-xs font-semibold text-[#f85149] transition-colors hover:bg-[#f85149]/20 disabled:opacity-50"
                      :disabled="fileDeleteBusy"
                      :title="t('files.deleteConfirm')"
                      @click="deleteSelectedFiles()"
                    >
                      <svg v-if="fileDeleteBusy" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                        <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                      </svg>
                      <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/></svg>
                      {{ t("files.deleteConfirm") }}
                    </button>
                    <button
                      type="button"
                      class="rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-xs font-medium text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-white"
                      @click="fileDeleteArmed = false"
                    >
                      {{ t("files.cancel") }}
                    </button>
                  </template>
                </div>
                <div class="flex shrink-0 items-center gap-1 rounded-md border border-[var(--border)] bg-[var(--bg)] p-0.5">
                  <button
                    type="button"
                    class="flex items-center gap-1 rounded px-2 py-1 text-xs font-semibold transition-colors"
                    :title="fileSortKey === 'name' ? (fileSortDir === 'asc' ? t('files.sortNameAsc') : t('files.sortNameDesc')) : t('files.sortNameHint')"
                    :class="fileSortKey === 'name'
                      ? 'bg-[var(--accent)] text-white'
                      : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
                    @click="toggleFileSort('name')"
                  >
                    {{ t("files.sortName") }}
                    <svg v-if="fileSortKey === 'name'" viewBox="0 0 16 16" class="h-2.5 w-2.5 fill-current" :style="{ transform: fileSortDir === 'asc' ? 'none' : 'rotate(180deg)' }">
                      <path d="M8 11.5 3.5 7h9L8 11.5Z"/>
                    </svg>
                  </button>
                  <button
                    type="button"
                    class="flex items-center gap-1 rounded px-2 py-1 text-xs font-semibold transition-colors"
                    :title="fileSortKey === 'date' ? (fileSortDir === 'desc' ? t('files.sortDateNew') : t('files.sortDateOld')) : t('files.sortDateHint')"
                    :class="fileSortKey === 'date'
                      ? 'bg-[var(--accent)] text-white'
                      : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
                    @click="toggleFileSort('date')"
                  >
                    {{ t("files.sortDate") }}
                    <svg v-if="fileSortKey === 'date'" viewBox="0 0 16 16" class="h-2.5 w-2.5 fill-current" :style="{ transform: fileSortDir === 'asc' ? 'none' : 'rotate(180deg)' }">
                      <path d="M8 11.5 3.5 7h9L8 11.5Z"/>
                    </svg>
                  </button>
                  <button
                    v-if="fileSortKey !== 'none'"
                    type="button"
                    class="rounded px-1.5 py-1 text-xs leading-none text-[color:var(--tx-muted)] transition-colors hover:text-[var(--accent)]"
                    :title="t('files.sortReset')"
                    @click="clearFileSort"
                  >
                    ×
                  </button>
                </div>
                <div class="flex shrink-0 items-center gap-0.5 rounded-md bg-[var(--input-50)] p-0.5">
                  <button
                    type="button"
                    class="rounded px-2 py-1 text-[11px] font-semibold transition-colors"
                    :title="t('files.fAllHint')"
                    :class="fileStatusFilter === 'all' ? 'bg-[var(--accent)] text-white' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
                    @click="setFileStatusFilter('all')"
                  >{{ t("files.fAll") }}</button>
                  <button
                    type="button"
                    class="rounded px-2 py-1 text-[11px] font-semibold transition-colors"
                    :title="t('files.fEnabledHint')"
                    :class="fileStatusFilter === 'enabled' ? 'bg-[var(--accent)] text-white' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
                    @click="setFileStatusFilter('enabled')"
                  >{{ t("files.fEnabled") }}</button>
                  <button
                    type="button"
                    class="rounded px-2 py-1 text-[11px] font-semibold transition-colors"
                    :title="t('files.fDisabledHint')"
                    :class="fileStatusFilter === 'disabled' ? 'bg-[var(--accent)] text-white' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
                    @click="setFileStatusFilter('disabled')"
                  >{{ t("files.fDisabled") }}</button>
                  <button
                    v-if="playSubTab !== 'saves'"
                    type="button"
                    class="rounded px-2 py-1 text-[11px] font-semibold transition-colors"
                    :title="t('files.fUpdatesHint')"
                    :class="fileStatusFilter === 'updates' ? 'bg-[var(--accent)] text-white' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
                    @click="setFileStatusFilter('updates')"
                  >{{ t("files.fUpdates") }}</button>
                  <button
                    v-if="playSubTab !== 'saves' && modUpdatesTab.length > 0 && !updateAllBusy"
                    type="button"
                    class="flex items-center gap-1 rounded px-2 py-1 text-[11px] font-semibold text-white bg-[var(--accent)] transition-colors hover:opacity-90"
                    :title="t('mods.updateAllHint', { n: modUpdatesTab.length })"
                    @click="updateAllMods"
                  >
                    <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M4.5 3.75a.75.75 0 0 0-1.5 0v2.5A.75.75 0 0 0 3.75 7h2.5a.75.75 0 0 0 0-1.5H5.07a4.5 4.5 0 1 1 .57 6.44.75.75 0 0 0-.98-1.13 6 6 0 1 0-.16-8.5v.49Z"/></svg>
                    {{ t("mods.updateAll") }} ({{ modUpdatesTab.length }})
                  </button>
                </div>
                <div class="relative min-w-0 flex-1">
                  <svg viewBox="0 0 16 16" class="pointer-events-none absolute left-2 top-1/2 h-3 w-3 -translate-y-1/2 fill-[var(--tx-muted)]">
                    <path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"/>
                  </svg>
                  <input
                    v-model="fileSearch"
                    type="text"
                    :placeholder="t('files.search')"
                    class="w-full rounded-md border border-[var(--border)] bg-[var(--bg)] py-1.5 pl-7 pr-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] outline-none transition-colors focus:border-[var(--accent)]"
                  />
                </div>
                <div ref="fileMenuRef" class="relative">
                  <button
                    type="button"
                    class="flex shrink-0 items-center rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] hover:text-white"
                    :title="t('files.more')"
                    @click="fileMenuOpen = !fileMenuOpen"
                  >
                    <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M8 4.25a1.25 1.25 0 1 1 0-2.5 1.25 1.25 0 0 1 0 2.5Zm0 5a1.25 1.25 0 1 1 0-2.5 1.25 1.25 0 0 1 0 2.5Zm0 5a1.25 1.25 0 1 1 0-2.5 1.25 1.25 0 0 1 0 2.5Z"/></svg>
                  </button>
                  <div
                    v-if="fileMenuOpen"
                    class="absolute right-0 top-[calc(100%+4px)] z-50 flex w-44 flex-col overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-1 shadow-xl"
                  >
                    <button
                      type="button"
                      class="flex items-center gap-2 rounded px-2 py-1.5 text-left text-xs text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                      @click="fileMenuOpen = false; openFolder(playSubTab as GameFolderKind)"
                    >
                      <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M.513 1.513A1.75 1.75 0 0 1 1.75 1h3.5c.55 0 1.07.26 1.4.7l.9 1.2a.25.25 0 0 0 .2.1H13a1 1 0 0 1 1 1v.5H2.75a.75.75 0 0 0 0 1.5h11.978a1 1 0 0 1 .994 1.117L15 13.25A1.75 1.75 0 0 1 13.25 15H1.75A1.75 1.75 0 0 1 0 13.25V2.75c0-.464.184-.91.513-1.237Z"/></svg>
                      {{ t("files.open") }}
                    </button>
                    <button
                      type="button"
                      class="flex items-center gap-2 rounded px-2 py-1.5 text-left text-xs text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                      @click="fileMenuOpen = false; selectAllFiles(playSubTab as GameFolderKind, fileListFiltered)"
                    >
                      <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 2A1.75 1.75 0 0 1 3.5.25h9A1.75 1.75 0 0 1 14.25 2v9A1.75 1.75 0 0 1 12.5 12.75h-9A1.75 1.75 0 0 1 1.75 11V2ZM6 4.5H4.5v1.5H6V4.5Zm0 3H4.5V9H6V7.5Zm1.25-3h4.25V4.5H7.25V4.5Z"/></svg>
                      {{ t("files.selectAll") }}
                    </button>
                  </div>
                </div>
              </div>
            </div>

            <div v-if="!gameFiles[playSubTab]" class="flex flex-1 items-center justify-center text-xs text-[color:var(--tx-muted)]">
              <svg class="mr-2 h-4 w-4 animate-spin fill-[var(--accent)]" viewBox="0 0 16 16">
                <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
              </svg>
              {{ t("files.loading") }}
            </div>
            <div v-else-if="(gameFiles[playSubTab] ?? []).length === 0" class="shrink-0 rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-8 text-center text-xs text-[color:var(--tx-muted)]">
              <p class="mb-3">{{ t("files.empty") }}</p>
              <button
                type="button"
                class="inline-flex items-center gap-1.5 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-1.5 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                @click="openFolder(playSubTab as GameFolderKind)"
              >
                <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M.513 1.513A1.75 1.75 0 0 1 1.75 1h3.5c.55 0 1.07.26 1.4.7l.9 1.2a.25.25 0 0 0 .2.1H13a1 1 0 0 1 1 1v.5H2.75a.75.75 0 0 0 0 1.5h11.978a1 1 0 0 1 .994 1.117L15 13.25A1.75 1.75 0 0 1 13.25 15H1.75A1.75 1.75 0 0 1 0 13.25V2.75c0-.464.184-.91.513-1.237Z"/></svg>
                {{ t("files.open") }}
              </button>
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
                    class="file-row flex cursor-pointer items-center gap-3 rounded-lg border px-3 py-2 transition-colors"
                    :class="[
                      isFileSelected(playSubTab, f)
                        ? 'border-[var(--accent-deep)] bg-[color-mix(in_srgb,var(--accent-deep)_10%,transparent)]'
                        : 'border-[var(--border)] bg-[var(--panel)] hover:border-[var(--tx-muted)]',
                      { 'opacity-60': !f.enabled },
                    ]"
                    @click="toggleFileSelect(playSubTab as GameFolderKind, f)"
                  >
                    <svg
                      viewBox="0 0 16 16"
                      class="h-3.5 w-3.5 shrink-0"
                      :class="isFileSelected(playSubTab, f) ? 'fill-[var(--accent)]' : 'fill-[var(--tx-muted)]'"
                    >
                      <path v-if="isFileSelected(playSubTab, f)" d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/>
                      <path v-else d="M4 1.75C4 .784 4.784 0 5.75 0h5.586c.464 0 .909.184 1.237.513l2.914 2.914a1.75 1.75 0 0 1 .513 1.237V12.25A1.75 1.75 0 0 1 14.25 14H5.75A1.75 1.75 0 0 1 4 12.25Zm1.75-.25a.25.25 0 0 0-.25.25v10.5c0 .138.112.25.25.25h8.5a.25.25 0 0 0 .25-.25V6h-2.75A1.75 1.75 0 0 1 10 4.25V1.5Z"/>
                    </svg>
                    <div class="flex h-10 w-10 shrink-0 items-center justify-center overflow-hidden rounded-md border border-[var(--border)] bg-[var(--bg)]">
                      <img
                        v-if="modrinthMetaFor(f)?.icon || curseMetaFor(f)?.icon || gameFileIcon(playSubTab, f.name)"
                        :src="modrinthMetaFor(f)?.icon || curseMetaFor(f)?.icon || gameFileIcon(playSubTab, f.name)"
                        alt=""
                        loading="lazy"
                        class="h-full w-full object-contain"
                      />
                      <svg v-else viewBox="0 0 16 16" class="h-5 w-5 fill-[var(--tx-muted)]">
                        <path d="M.75 6.25a1.75 1.75 0 0 1 1.75-1.75h2.054l1.17-1.17A1.74 1.74 0 0 1 6.902 2.75h1.536l-.055.836a1.44 1.44 0 0 0 .432 1.123l.022.022c.059.059.12.108.183.148.523.34 1.074.405 1.528.429.755.04 1.452.044 1.766.044h3.44v.586c0 .527-.211 1.032-.587 1.404l-3.318 3.318a1.5 1.5 0 0 1-1.06.44H4.78l-.824-.412a1.75 1.75 0 0 1-.736-2.383.5.5 0 0 1-.368-.454A1.75 1.75 0 0 1 .75 6.25Zm13.24 0h-3.14c-.249 0-.679-.004-1.112-.03-.36-.022-.622-.066-.783-.111.05-.066.11-.129.176-.194l.483-.483c.344-.344.416-.861.18-1.283A1.75 1.75 0 0 0 8.5 2.75H4.75A1.75 1.75 0 0 1 4.75 1.5c.692-.06 1.4-.086 2.127-.086.63 0 1.255.022 1.873.064a.75.75 0 0 1 .5.25.75.75 0 0 1 .246.5l.293 2.927.178.04c.646.147 1.548.377 2.615.614.17.038.2.07.22.098a.6.6 0 0 1 .074.233.75.75 0 0 1-.075.4.6.6 0 0 1-.235.23ZM3.75 10.75h5.38l1.5-1.5H3.75a.75.75 0 0 1-.143 1.482l-.14.014a.75.75 0 0 1 .283-.004L3.75 10.75Z"/>
                      </svg>
                    </div>
                <div class="min-w-0 flex-1">
                  <div
                    class="truncate text-xs font-medium text-[color:var(--tx)]"
                    :title="fileMetaTitle(f)"
                  >
                    {{ fileMetaTitle(f) }}
                  </div>
                  <div class="truncate text-[11px] text-[color:var(--tx-muted)]">
                    <template v-if="modrinthMetaFor(f)?.title || curseMetaFor(f)?.title">
                      <template v-if="modrinthVersionFor(f)">{{ modrinthVersionFor(f) }} · </template>
                      <template v-else-if="curseMetaFor(f)?.title">{{ f.displayName }} · </template>
                    </template>{{ f.kind === "dir" ? t("files.dir") : `${formatBytes(f.sizeBytes)} · ${formatUnixDate(f.modified)} · ${f.enabled ? t("files.enabled") : t("files.disabled")}` }}
                  </div>
                </div>
                <button
                  v-if="playSubTab !== 'saves' && f.curseforgeProjectId"
                  type="button"
                  class="flex shrink-0 items-center gap-1 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-[11px] font-medium text-[color:var(--tx-muted)] transition-colors hover:border-[color-mix(in_srgb,var(--accent)_50%,transparent)] hover:text-[var(--accent)]"
                  :title="t('files.curseforge')"
                  @click.stop="openFileOnCurseForge(playSubTab as GameFolderKind, f)"
                >
                  CurseForge
                </button>
                <button
                  v-if="playSubTab !== 'saves' && !f.curseforgeProjectId"
                  type="button"
                  class="flex shrink-0 items-center gap-1 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-[11px] font-medium text-[color:var(--tx-muted)] transition-colors hover:border-[color-mix(in_srgb,var(--accent)_50%,transparent)] hover:text-[var(--accent)]"
                  :title="t('files.modrinth')"
                  @click.stop="openFileOnModrinth(playSubTab as GameFolderKind, f)"
                >
                  Modrinth
                </button>
                <button
                  v-if="playSubTab !== 'saves' && (f.modrinthProjectId || f.modrinthUrl || f.curseforgeProjectId)"
                  type="button"
                  class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md border border-[var(--border)] bg-[var(--input)] text-[color:var(--tx-muted)] transition-colors hover:border-[color-mix(in_srgb,var(--accent)_50%,transparent)] hover:text-[var(--accent)]"
                  :title="t('files.view')"
                  @click.stop="openFileDetail(playSubTab as GameFolderKind, f)"
                >
                  <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M8 3.75a3.25 3.25 0 1 0 0 6.5 3.25 3.25 0 0 0 0-6.5Zm0 8.5A8.75 8.75 0 0 1 0 8a8.75 8.75 0 0 1 8-4.25A8.75 8.75 0 0 1 16 8a8.75 8.75 0 0 1-8 4.25Z"/></svg>
                </button>
                <button
                  v-if="playSubTab !== 'saves' && modUpdateFor(f)"
                  type="button"
                  class="flex shrink-0 items-center gap-1 rounded-md border border-[color-mix(in_srgb,var(--accent)_45%,transparent)] bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] px-2 py-1 text-[11px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_22%,transparent)] disabled:opacity-50"
                  :disabled="updatingMod !== null || packLocked"
                  :title="`${modUpdateFor(f)!.newVersion.name} (${modUpdateFor(f)!.newVersion.versionNumber})`"
                  @click.stop="updateOneMod(modUpdateFor(f)!)"
                >
                  <svg v-if="updatingMod === f.name" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                    <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                  </svg>
                  <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                    <path d="M4.5 3.75a.75.75 0 0 0-1.5 0v2.5A.75.75 0 0 0 3.75 7h2.5a.75.75 0 0 0 0-1.5H5.07a4.5 4.5 0 1 1 .57 6.44.75.75 0 0 0-.98-1.13 6 6 0 1 0-.16-8.5v.49Z"/>
                  </svg>
                  {{ t("mods.update") }}
                </button>
                <button
                  v-if="f.kind === 'file'"
                  type="button"
                  class="relative h-5 w-9 shrink-0 rounded-full transition-all duration-200"
                  :class="[
                    f.enabled ? 'bg-[#238636]' : 'bg-[var(--tx-muted)]',
                    isFileToggling(playSubTab, f) ? 'opacity-50 cursor-wait' : 'hover:ring-2 hover:ring-[color-mix(in_srgb,var(--accent)_40%,transparent)]',
                  ]"
                  role="switch"
                  :aria-checked="f.enabled"
                  :disabled="isFileToggling(playSubTab, f) || packLocked"
                  :title="isFileToggling(playSubTab, f) ? undefined : (packLocked ? t('files.locked') : (f.enabled ? t('files.disable') : t('files.enable')))"
                  @click.stop="handleToggleFile(playSubTab as GameFolderKind, f)"
                >
                  <span
                    class="absolute top-0.5 flex h-4 w-4 items-center justify-center rounded-full bg-white shadow-sm transition-all duration-200"
                    :class="f.enabled ? 'left-[18px]' : 'left-0.5'"
                  >
                    <svg v-if="isFileToggling(playSubTab, f)" viewBox="0 0 16 16" class="h-2.5 w-2.5 animate-spin fill-[var(--accent)]">
                      <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                    </svg>
                  </span>
                </button>
              </div>
            </div>
          </div>
          </div>
          </div>

          <!-- ======= Дубликаты (mods / resourcepacks / shaderpacks) ======= -->
          <template v-else-if="playSubTab === 'duplicates'">
            <div class="flex min-h-0 flex-1 flex-col">
              <div v-if="duplicatesLoading" class="flex flex-1 items-center justify-center text-xs text-[color:var(--tx-muted)]">
                <svg class="mr-2 h-4 w-4 animate-spin fill-[var(--accent)]" viewBox="0 0 16 16">
                  <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                </svg>
                {{ t("duplicates.loading") }}
              </div>
              <div v-else-if="duplicates.groups.length === 0" class="flex flex-1 items-center justify-center">
                <div class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-8 text-center text-xs text-[color:var(--tx-muted)]">
                  <p class="font-medium text-[color:var(--tx)]">{{ t("duplicates.empty") }}</p>
                </div>
              </div>
              <div v-else class="min-h-0 flex-1 space-y-3 overflow-y-auto pr-1 pb-8">
                <p class="text-xs text-[color:var(--tx-muted)]">
                  {{ t("duplicates.found", { n: duplicates.groups.length, size: formatBytes(duplicates.wasted_bytes) }) }}
                </p>
                <div v-for="(g, gi) in duplicates.groups" :key="gi" class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm">
                  <div class="flex items-center justify-between gap-2 border-b border-[var(--border)] px-3 py-2">
                    <p class="text-xs font-medium text-[color:var(--tx-strong)]">
                      {{ t("duplicates.group", { n: g.files.length, size: formatBytes(g.size_bytes) }) }}
                    </p>
                    <button
                      type="button"
                      class="rounded border border-[#f85149]/40 bg-[#f85149]/10 px-2 py-0.5 text-[11px] font-semibold text-[#f85149] transition-colors hover:bg-[#f85149]/20"
                      @click="keepOne(g)"
                    >{{ t("duplicates.keepOne") }}</button>
                  </div>
                  <ul class="space-y-1 p-2">
                    <li
                      v-for="f in g.files"
                      :key="f.path"
                      class="flex items-center gap-2 rounded px-2 py-1 text-xs hover:bg-[var(--input-50)]"
                    >
                      <span class="min-w-0 flex-1 truncate font-mono text-[color:var(--tx-muted)]" :title="f.path">{{ f.folder }} / {{ f.name }}</span>
                      <button
                        type="button"
                        class="shrink-0 rounded px-1.5 py-0.5 text-[11px] font-semibold text-[color:var(--tx-muted)] transition-colors hover:bg-[#f85149]/15 hover:text-[#f85149]"
                        :title="t('duplicates.delete')"
                        @click="removeDuplicate(packId, f)"
                      >{{ t("duplicates.delete") }}</button>
                    </li>
                  </ul>
                </div>
              </div>
            </div>
          </template>

          <!-- ======= Скриншоты сборки (папка screenshots установленной версии) ======= -->
          <template v-else-if="playSubTab === 'screenshots'">
            <div class="flex min-h-0 flex-1 flex-col">
              <div v-if="screenshotsLoading" class="flex flex-1 items-center justify-center text-xs text-[color:var(--tx-muted)]">
                <svg class="mr-2 h-4 w-4 animate-spin fill-[var(--accent)]" viewBox="0 0 16 16">
                  <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                </svg>
                {{ t("screenshots.loading") }}
              </div>
              <div v-else-if="!packScreenshotsInstalled" class="flex flex-1 items-center justify-center">
                <div class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-8 text-center text-xs text-[color:var(--tx-muted)]">
                  <p class="font-medium text-[color:var(--tx)]">{{ t("screenshots.noInstall") }}</p>
                </div>
              </div>
              <div v-else-if="packScreenshots.length === 0" class="flex flex-1 items-center justify-center">
                <div class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-8 text-center text-xs text-[color:var(--tx-muted)]">
                  <p class="font-medium text-[color:var(--tx)]">{{ t("screenshots.empty") }}</p>
                </div>
              </div>
              <div v-else class="min-h-0 flex-1 space-y-3 overflow-y-auto pr-1 pb-8">
                <p class="text-xs text-[color:var(--tx-muted)]">
                  {{ t("screenshots.count", { n: packScreenshots.length }) }}
                </p>
                <div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
                  <button
                    v-for="(shot, i) in packScreenshots"
                    :key="shot.path"
                    type="button"
                    class="group relative overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm transition-colors hover:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]"
                    @click="shotIdx = i"
                  >
                    <img
                      :src="convertFileSrc(shot.path)"
                      :alt="`${t('sub.screenshots')} ${i + 1}`"
                      loading="lazy"
                      class="aspect-video w-full object-cover transition-transform duration-300 group-hover:scale-[1.03]"
                    />
                    <span
                      v-if="shot.modified"
                      class="pointer-events-none absolute bottom-1 right-1 rounded bg-black/60 px-1.5 py-0.5 text-[11px] font-medium text-white"
                    >{{ formatUnixDate(shot.modified) }}</span>
                  </button>
                </div>
                <p class="text-[11px] text-[color:var(--tx-muted)]">{{ t("screenshots.note") }}</p>
              </div>
            </div>

            <!-- Лайтбокс -->
            <div
              v-if="shotIdx !== null"
              class="fixed inset-0 z-50 flex items-center justify-center bg-black/85 p-6"
              @click.self="shotIdx = null"
            >
              <button
                type="button"
                class="absolute right-4 top-4 rounded-md bg-[var(--panel)] px-2.5 py-1 text-xs font-semibold text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                @click="shotIdx = null"
              >
                ✕
              </button>
              <button
                v-if="packScreenshots.length > 1"
                type="button"
                class="absolute left-3 top-1/2 -translate-y-1/2 rounded-md bg-[var(--panel)] px-2.5 py-1.5 text-sm font-semibold text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                @click="shotIdx = ((shotIdx ?? 0) - 1 + packScreenshots.length) % packScreenshots.length"
              >
                ←
              </button>
              <button
                v-if="packScreenshots.length > 1"
                type="button"
                class="absolute right-3 top-1/2 -translate-y-1/2 rounded-md bg-[var(--panel)] px-2.5 py-1.5 text-sm font-semibold text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                @click="shotIdx = ((shotIdx ?? 0) + 1) % packScreenshots.length"
              >
                →
              </button>
              <img
                :src="convertFileSrc(packScreenshots[shotIdx ?? 0]?.path)"
                class="max-h-[82vh] max-w-full rounded-lg object-contain shadow-2xl"
                alt=""
              />
              <span class="absolute bottom-4 rounded bg-black/60 px-2 py-1 font-mono text-xs text-[color:var(--tx-muted)]">
                {{ (shotIdx ?? 0) + 1 }} / {{ packScreenshots.length }}
                <template v-if="packScreenshots[shotIdx ?? 0]?.modified">
                  · {{ formatUnixDate(packScreenshots[shotIdx ?? 0]!.modified) }}
                </template>
              </span>
            </div>
          </template>

          <!-- ======= Сервера: сборки (servers.json) сверху + свои (servers.dat) снизу ======= -->
          <template v-else-if="playSubTab === 'servers'">
            <div class="min-h-0 flex-1 overflow-y-auto pr-1 pb-8">
              <div v-for="group in serverGroups" :key="group.key" class="mb-8 last:mb-0">
                <div class="mb-3 flex items-center justify-between">
                  <p class="text-xs font-medium text-[color:var(--tx-strong)]">
                    {{ group.title }}
                    <span class="font-normal text-[color:var(--tx-muted)]">· {{ group.servers.length }}</span>
                  </p>

                </div>
                <p v-if="group.servers.length === 0" class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm px-4 py-3 text-xs text-[color:var(--tx-muted)]">
                  {{ group.emptyText }}
                </p>
                <div v-else class="grid gap-3 sm:grid-cols-2">
                  <div
                    v-for="s in group.servers"
                    :key="`${group.key}-${serverKey(s)}`"
                    class="flex flex-col gap-3 rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-4 transition-colors hover:border-[color-mix(in_srgb,var(--accent)_50%,transparent)]"
                  >
                    <div class="flex items-start gap-3">
                      <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md border border-[var(--border)] bg-[var(--input)]">
                        <svg viewBox="0 0 16 16" class="h-4 w-4 fill-[var(--tx-muted)]">
                          <path d="M3 1.5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-9a2 2 0 0 0-2-2ZM1.5 4.5H14.5v1.5H1.5ZM1.5 8H14.5v1.25H1.5Zm0 3.25H7v1.5H1.5A.5.5 0 0 1 1 12.25v-1ZM8.5 12.75v-1.5h6v1.5A.5.5 0 0 1 14.5 13h-5a1 1 0 0 1-1-.25ZM2 5.75a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Zm3 0a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0ZM2 9.25a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Zm3 0a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Z"/>
                        </svg>
                      </div>
                      <div class="min-w-0 flex-1">
                        <div class="flex items-center gap-2">
                          <span class="truncate text-sm font-semibold text-[color:var(--tx-strong)]">{{ s.name }}</span>
                          <span
                            class="h-1.5 w-1.5 shrink-0 rounded-full"
                            :class="serverStateOf(s) === 'online' ? 'bg-[#3fb950]' : serverStateOf(s) === 'offline' ? 'bg-[#f85149]' : 'bg-[var(--tx-muted)]'"
                            :title="serverStatusText(s)"
                          />
                          <span
                            v-if="serverPlayersOf(s).length > 0"
                            class="shrink-0 rounded-full border border-[#3fb950]/40 bg-[#3fb950]/10 px-1.5 py-0.5 text-[11px] font-semibold text-[#3fb950]"
                            :title="t('servers.players', { n: serverPlayersOf(s).length, names: serverPlayersOf(s).join(', ') })"
                          >
                            {{ serverStatuses[serverKey(s)]?.playersOnline }}/{{ serverStatuses[serverKey(s)]?.playersMax }}
                          </span>
                        </div>
                        <div v-if="s.desc" class="mt-0.5 line-clamp-2 text-xs text-[color:var(--tx-muted)]">{{ s.desc }}</div>
                        <div class="mt-1 truncate text-[11px] text-[color:var(--tx-muted)]" :title="serverStatusText(s)">
                          {{ serverStatusText(s) }}
                        </div>
                      </div>
                      <span
                        v-if="s.port"
                        class="shrink-0 rounded border border-[var(--border)] bg-[var(--input)] px-1.5 py-0.5 font-mono text-[11px] text-[color:var(--tx-muted)]"
                      >
                        :{{ s.port }}
                      </span>
                    </div>
                    <div class="mt-auto flex items-center justify-between gap-2 border-t border-[var(--border)] pt-3">
                      <code class="truncate font-mono text-xs text-[color:var(--tx)]">{{ s.ip }}{{ s.port ? `:${s.port}` : "" }}</code>
                      <div class="flex shrink-0 gap-2">
                        <button
                          type="button"
                          class="flex items-center gap-1.5 rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1 text-xs font-medium text-[color:var(--tx)] transition-colors hover:border-[color-mix(in_srgb,var(--accent)_50%,transparent)] hover:text-[var(--accent)]"
                          @click="copyServerIp(s)"
                        >
                          <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                            <path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25Z"/>
                            <path d="M5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"/>
                          </svg>
                          {{ t("servers.copy") }}
                        </button>
                        <button
                          type="button"
                          class="flex items-center gap-1.5 rounded-md bg-[#238636] px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[#2ea043] disabled:cursor-not-allowed disabled:opacity-50"
                          :disabled="gameRunning"
                          @click="playOnServer(s)"
                        >
                          <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                            <path d="M3.75 2a.75.75 0 0 1 .75.75V7h7V2.75a.75.75 0 0 1 1.5 0v10.5a.75.75 0 0 1-1.5 0V8.5h-7v4.75a.75.75 0 0 1-1.5 0V2.75a.75.75 0 0 1 .75-.75Z"/>
                          </svg>
                          {{ t("servers.play") }}
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </template>

          <!-- Настройки сборки -->
          <template v-else-if="playSubTab === 'settings'">
            <div class="min-h-0 flex-1 overflow-y-auto pr-1">
              <div class="max-w-2xl space-y-6">
                <!-- ОЗУ -->
                <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
                  <div class="border-b border-[var(--border)] px-4 py-3 flex justify-between items-center">
                    <h3 class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("settings.ram") }}</h3>
                    <span class="font-mono text-xs font-semibold text-[var(--accent)]">{{ ram }} {{ t("units.gb") }}</span>
                  </div>
                  <div class="p-4 space-y-2">
                    <input
                      type="range"
                      min="2"
                      :max="maxRam"
                      step="1"
                      v-model.number="ram"
                      class="w-full accent-[var(--accent-deep)] bg-[var(--input)] h-1.5 rounded-lg appearance-none cursor-pointer"
                    />
                    <div class="flex justify-between text-xs text-[color:var(--tx-muted)] font-mono">
                      <span>2 {{ t("units.gb") }}</span>
                      <span>{{ t("settings.ramMax", { n: maxRam }) }}</span>
                    </div>
                    <p v-if="systemRam && systemRam.total_ram_gb > 0" class="text-xs text-[color:var(--tx-muted)]">
                      {{ t("settings.ramTotal", { total: systemRam.total_ram_gb, avail: systemRam.available_ram_gb }) }}
                    </p>
                    <p
                      v-if="activePack?.minRam"
                      class="text-xs"
                      :class="(ram * 1024) < activePack.minRam ? 'font-medium text-[#f0883e]' : 'text-[color:var(--tx-muted)]'"
                    >
                      {{ t("settings.ramMin", { name: activePack.name, min: activePack.minRam / 1024, gb: ram }) }}
                    </p>
                  </div>
                </section>

                <!-- Размер окна игры -->
                <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
                  <div class="border-b border-[var(--border)] px-4 py-3 flex justify-between items-center">
                    <h3 class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("settings.win") }}</h3>
                    <span class="font-mono text-xs font-semibold text-[var(--accent)]">{{ windowWidth }}×{{ windowHeight }}</span>
                  </div>
                  <div class="p-4 space-y-2">
                    <div class="flex items-center gap-3">
                      <label class="w-16 text-xs text-[color:var(--tx-muted)]" for="win-width">{{ t("settings.width") }}</label>
                      <input
                        id="win-width"
                        type="number"
                        min="320"
                        max="7680"
                        step="1"
                        v-model.number="windowWidth"
                        class="flex-1 rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] focus:border-[var(--accent)] focus:outline-none"
                      />
                      <label class="w-16 text-xs text-[color:var(--tx-muted)]" for="win-height">{{ t("settings.height") }}</label>
                      <input
                        id="win-height"
                        type="number"
                        min="240"
                        max="4320"
                        step="1"
                        v-model.number="windowHeight"
                        class="flex-1 rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] focus:border-[var(--accent)] focus:outline-none"
                      />
                    </div>
                    <p class="text-xs text-[color:var(--tx-muted)]">
                      {{ t("settings.winNote") }}
                    </p>
                  </div>
                </section>

                <!-- Java -->
                <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
                  <div class="border-b border-[var(--border)] px-4 py-3">
                    <h3 class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("settings.java") }}</h3>
                  </div>
                  <div class="p-4 space-y-3">
                    <div class="flex items-center gap-2">
                      <select
                        :value="javaSelected"
                        class="flex-1 appearance-none rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-1.5 pr-8 text-xs text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] focus:border-[var(--accent)] focus:outline-none"
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
                        class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] disabled:opacity-50"
                        :disabled="javaBusy || busy"
                        @click="downloadJava"
                      >
                        {{ javaBusy ? t("settings.javaDownloading") : t("settings.javaDownload") }}
                      </button>
                    </div>
                    <p v-if="javaMsg" class="text-xs text-[color:var(--tx-muted)] break-all">{{ javaMsg }}</p>
                    <p class="text-xs text-[color:var(--tx-muted)]">
                      {{ t("settings.javaNote") }}
                    </p>
                  </div>
                </section>

                <!-- Discord Rich Presence -->
                <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
                  <div class="border-b border-[var(--border)] px-4 py-3">
                    <h3 class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("settings.discord") }}</h3>
                  </div>
                  <div class="p-4">
                    <label class="flex cursor-pointer items-center gap-3">
                      <input
                        type="checkbox"
                        class="h-4 w-4 accent-[#5865F2]"
                        :checked="discordRp"
                        @change="toggleDiscordRp(($event.target as HTMLInputElement).checked)"
                      />
                      <span class="text-xs text-[color:var(--tx)]">{{ t("settings.discordLabel") }}</span>
                    </label>
                    <p class="mt-2 text-xs text-[color:var(--tx-muted)]">
                      {{ t("settings.discordNote") }}
                    </p>
                  </div>
                </section>

                <!-- Предупреждение о кастомных модах -->
                <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
                  <div class="border-b border-[var(--border)] px-4 py-3">
                    <h3 class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("settings.warnCustomMods") }}</h3>
                  </div>
                  <div class="p-4">
                    <label class="flex cursor-pointer items-center gap-3">
                      <input
                        type="checkbox"
                        class="h-4 w-4 accent-[#f0883e]"
                        :checked="warnCustomMods"
                        @change="toggleWarnCustomMods(($event.target as HTMLInputElement).checked)"
                      />
                      <span class="text-xs text-[color:var(--tx)]">{{ t("settings.warnCustomModsLabel") }}</span>
                    </label>
                    <p class="mt-2 text-xs text-[color:var(--tx-muted)]">
                      {{ t("settings.warnCustomModsNote") }}
                    </p>
                  </div>
                </section>

                <!-- Проверка целостности -->
                <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
                  <div class="border-b border-[var(--border)] px-4 py-3 flex justify-between items-center">
                    <h3 class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("settings.verify") }}</h3>
                  </div>
                  <div class="p-4 space-y-3">
                    <p class="text-xs text-[color:var(--tx-muted)]">
                      {{ t("settings.verifyNote") }}
                    </p>
                    <button
                      type="button"
                      class="rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] disabled:opacity-50"
                      :disabled="verifyBusy || busy"
                      @click="handleVerify"
                    >
                      {{ verifyBusy ? t("settings.verifying") : t("settings.verifyBtn") }}
                    </button>
                    <div
                      v-if="verifyResult"
                      class="rounded-md border bg-[var(--bg-60)] p-3 text-xs"
                      :class="verifyResult.broken.length === 0 ? 'border-[#238636]/40' : 'border-[#f85149]/40'"
                    >
                      <p class="font-medium" :class="verifyResult.broken.length === 0 ? 'text-[#3fb950]' : 'text-[#f85149]'">
                        {{ verifyResult.broken.length === 0 ? t("settings.verifyOk") : t("settings.verifyBroken", { n: verifyResult.broken.length }) }}
                      </p>
                      <p class="mt-0.5 text-[color:var(--tx-muted)]">{{ t("settings.verifyStats", { checked: verifyResult.checked, ok: verifyResult.ok }) }}</p>
                      <ul v-if="verifyResult.broken.length > 0" class="mt-2 max-h-32 space-y-1 overflow-y-auto font-mono text-[11px] text-[#f85149]">
                        <li v-for="b in verifyResult.broken" :key="b">{{ b }}</li>
                      </ul>
</div>

              </div>
            </section>
          </div>
          </div>
        </template>

          <!-- Консоль / логи -->
          <section v-else class="flex h-full min-h-0 flex-1 flex-col overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm">
            <div class="flex items-center justify-between border-b border-[var(--border)] bg-[var(--input-50)] px-4 py-2">
              <h3 class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("console.title") }}</h3>
              <div class="flex items-center gap-3">
                <span class="text-[11px] tabular-nums text-[var(--tx-muted)]">
                  {{ t("console.lines", { n: logEntries.length }) }}
                </span>
                <div class="flex gap-2">
                  <button
                    type="button"
                    class="text-xs text-[color:var(--tx-muted)] hover:text-[var(--accent)]"
                    @click="handleCopyLog"
                  >
                    {{ t("console.copy") }}
                  </button>
                  <button
                    type="button"
                    class="text-xs text-[color:var(--tx-muted)] hover:text-[#f85149]"
                    @click="handleClearLog"
                  >
                    {{ t("console.clear") }}
                  </button>
                  <button
                    type="button"
                    class="text-xs text-[color:var(--tx-muted)] hover:text-[var(--accent)]"
                    @click="openFolder('logs')"
                  >
                    {{ t("console.logs") }}
                  </button>
                </div>
              </div>
            </div>
            <div
              ref="logRef"
              class="flex-1 select-text overflow-y-auto bg-[var(--bg)] p-3 font-mono text-xs leading-relaxed text-[color:var(--tx-muted)]"
            >
              <p v-if="logEntries.length === 0" class="italic text-[var(--tx-muted)]">
                {{ t("console.empty") }}
              </p>
              <div
                v-for="(e, i) in logEntries"
                :key="i"
                :class="{
                  'text-[#f85149]': e.stream === 'err',
                  'text-[var(--accent)]': e.stream === 'sys',
                  'text-[color:var(--tx)]': e.stream === 'out',
                  'font-bold !text-[#f85149]': e.fatal,
                }"
              >
                {{ e.line }}
              </div>
            </div>
          </section>
          </div>

          <!-- ======= Просмотр ресурса (страница в лаунчере): обновить + перейти на сервис ======= -->
          <div
            v-if="fileDetail && fileDetail.folder === playSubTab"
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-6"
            @click.self="fileDetail = null"
          >
            <div class="flex max-h-[82vh] w-full max-w-xl flex-col overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-2xl">
              <div class="flex shrink-0 items-center justify-between border-b border-[var(--border)] px-4 py-3">
                <h3 class="truncate text-sm font-semibold text-[color:var(--tx-strong)]">
                  {{ fileDetail.entry.displayName }}
                </h3>
                <button
                  type="button"
                  class="rounded-md p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
                  @click="fileDetail = null"
                >
                  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
                </button>
              </div>

              <div class="flex min-h-0 flex-col gap-3 overflow-y-auto p-4">
                <div class="flex items-center gap-3 rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2.5">
                  <img
                    v-if="fileDetailMr?.iconUrl"
                    :src="fileDetailMr.iconUrl"
                    :alt="fileDetailMr.title"
                    loading="lazy"
                    class="h-11 w-11 shrink-0 rounded-md object-cover"
                  />
                  <img
                    v-else-if="fileDetailCf?.iconUrl"
                    :src="fileDetailCf.iconUrl"
                    :alt="fileDetailCf.name"
                    loading="lazy"
                    class="h-11 w-11 shrink-0 rounded-md object-cover"
                  />
                  <div v-else class="flex h-11 w-11 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-xs text-[color:var(--tx-muted)]">
                    {{ (fileDetailMr?.title || fileDetailCf?.name || fileDetail.entry.displayName).slice(0, 2).toUpperCase() }}
                  </div>
                  <div class="min-w-0 flex-1">
                    <h4 class="truncate text-sm font-semibold text-[color:var(--tx-strong)]">
                      {{ fileDetailMr?.title || fileDetailCf?.name || fileDetail.entry.displayName }}
                    </h4>
                    <p class="truncate text-[11px] text-[color:var(--tx-muted)]">
                      {{ fileDetail.entry.name }}
                      <template v-if="fileDetailMr || fileDetailCf"> · {{ fileDetailMr ? t("mods.serviceModrinth") : t("mods.serviceCurseforge") }}</template>
                    </p>
                  </div>
                </div>

                <button
                  type="button"
                  class="flex shrink-0 items-center justify-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-3 py-2 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                  :disabled="fileDetailMrLoading || fileDetailCfLoading || updatingFileDetail"
                  @click="updateFileDetail()"
                >
                  <svg v-if="updatingFileDetail" viewBox="0 0 16 16" class="h-3.5 w-3.5 animate-spin fill-current">
                    <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                  </svg>
                  <svg v-else viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
                    <path d="M4.5 3.75a.75.75 0 0 0-1.5 0v2.5A.75.75 0 0 0 3.75 7h2.5a.75.75 0 0 0 0-1.5H5.07a4.5 4.5 0 1 1 .57 6.44.75.75 0 0 0-.98-1.13 6 6 0 1 0-.16-8.5v.49Z"/>
                  </svg>
                  {{ t("files.update") }}
                </button>

                <template v-if="fileDetailMr">
                  <div class="rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx-muted)]">
                    <p class="line-clamp-3">{{ fileDetailMr.description }}</p>
                    <p class="mt-1 flex flex-wrap items-center gap-3">
                      <span>{{ t("mods.byAuthor", { author: fileDetailMr.author }) }}</span>
                      <span v-if="fileDetailMr.downloads">{{ fileDetailMr.downloads.toLocaleString() }} {{ t("mods.downloads") }}</span>
                      <span v-if="fileDetailMr.categories.length">{{ fileDetailMr.categories.slice(0, 4).join(", ") }}</span>
                    </p>
                  </div>
                  <div class="mb-3 flex items-center gap-1 border-b border-[var(--border)] pb-2">
                    <button
                      v-for="tb in fileDetailTabs"
                      :key="tb.kind"
                      type="button"
                      class="rounded-md px-3 py-1.5 text-xs font-medium transition-colors"
                      :class="fileDetailTab === tb.kind
                        ? 'bg-[var(--input)] text-[color:var(--tx-strong)]'
                        : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]'"
                      @click="fileDetailTab = tb.kind"
                    >
                      {{ t("mods.tab" + tb.kind) }}
                    </button>
                  </div>
                  <div v-if="fileDetailTab === 'about'" class="max-h-[40vh] overflow-y-auto rounded-md border border-[var(--border)] bg-[var(--bg)] px-4 py-3">
                    <Markdown v-if="fileDetailMr.body" :source="fileDetailMr.body" />
                    <p v-else class="py-6 text-center text-xs italic text-[color:var(--tx-muted)]">{{ t("mods.noAbout") }}</p>
                  </div>
                  <div v-else-if="fileDetailTab === 'versions'">
                    <div class="mb-2 flex flex-wrap items-center gap-2">
                      <FilterSelect v-model="fileDetailMcSel" :options="fileDetailMcOptions" :placeholder="t('curse.fVersion')" :multiple="false" />
                      <FilterSelect v-model="fileDetailLoaderSel" :options="fileDetailLoaderOptions" :placeholder="t('mods.fLoader')" :multiple="false" />
                      <FilterSelect v-model="fileDetailTypeSel" :options="versionTypeOptions" :placeholder="t('mods.fType')" :multiple="false" />
                    </div>
                    <div v-if="fileDetailMrVersions === null" class="flex items-center justify-center py-4 text-xs text-[color:var(--tx-muted)]">
                      <svg viewBox="0 0 16 16" class="mr-2 h-3.5 w-3.5 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
                      {{ t("mods.searching") }}
                    </div>
                    <div v-else-if="fileDetailFilteredVersions.length === 0" class="rounded-md border border-[var(--border)] bg-[var(--input-50)] p-4 text-center text-xs text-[color:var(--tx-muted)]">
                      {{ t("mods.noVersions") }}
                    </div>
                    <div v-else class="space-y-1">
                      <button
                        v-for="v in fileDetailFilteredVersions"
                        :key="v.id"
                        type="button"
                        class="flex w-full items-center gap-2 rounded-md border border-[var(--border)] bg-[var(--bg)] py-1.5 pl-2.5 pr-2 text-left transition-colors hover:border-[color-mix(in_srgb,var(--accent)_50%,transparent)] disabled:opacity-50"
                        :disabled="fileDetailMrVersionBusy !== null"
                        @click="installFileDetailVersion(v)"
                      >
                        <span
                          class="h-2 w-2 shrink-0 rounded-full"
                          :style="{ backgroundColor: verTypeColor(v.versionType) }"
                          :title="t('mods.verType.' + v.versionType)"
                        ></span>
                        <span class="min-w-0 flex-1">
<span class="flex items-center gap-1.5 text-xs font-medium text-[color:var(--tx-strong)]">
                            <span class="truncate">{{ v.name }}</span>
                            <span v-if="fileDetailInstalledVersion(v)" class="shrink-0 rounded-full border border-[color-mix(in_srgb,var(--accent)_45%,transparent)] bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] px-1.5 py-px text-[10px] font-semibold text-[var(--accent)]">{{ t("mods.installedBadge") }}</span>
                          </span>
                          <span class="block truncate text-[11px] text-[color:var(--tx-muted)]">
                            {{ v.loaders.map(cap).join(" · ") || "vanilla" }} · {{ v.gameVersions.slice(0, 2).join(", ") }} · {{ formatDate(v.datePublished) }}
                            <template v-if="verInstallSize(v)"> · {{ formatBytes(verInstallSize(v)) }}</template>
                          </span>
                        </span>
                        <span class="shrink-0 rounded border border-[var(--border)] bg-[var(--input-50)] px-1.5 py-px font-mono text-[10px] text-[color:var(--tx-muted)]">{{ v.versionNumber }}</span>
                        <template v-if="fileDetailMrVersionBusy === v.id">
                          <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 shrink-0 animate-spin fill-[var(--accent)]"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
                        </template>
                        <svg v-else viewBox="0 0 16 16" class="h-3.5 w-3.5 shrink-0 fill-[var(--accent)]"><path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/></svg>
                      </button>
                    </div>
                  </div>
                  <div v-else>
                    <div v-if="fileDetailMr.gallery.length" class="grid grid-cols-2 gap-2">
                      <img
                        v-for="g in fileDetailMr.gallery"
                        :key="g.url"
                        :src="g.url"
                        :alt="g.title ?? ''"
                        loading="lazy"
                        class="h-32 w-full cursor-zoom-in rounded-md border border-[var(--border)] object-cover transition-transform hover:scale-[1.02]"
                        :title="g.title ?? undefined"
                        @click="openExternal(g.url)"
                      />
                    </div>
                    <p v-else class="py-10 text-center text-xs italic text-[color:var(--tx-muted)]">{{ t("mods.noGallery") }}</p>
                  </div>
                </template>

                <template v-else-if="fileDetailCf">
                  <div class="rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx-muted)]">
                    <p class="max-h-40 overflow-y-auto whitespace-pre-wrap">{{ fileDetailCf.description }}</p>
                  </div>
                </template>
                <p v-else-if="fileDetailMrLoading || fileDetailCfLoading" class="flex items-center justify-center py-6 text-xs text-[color:var(--tx-muted)]">
                  <svg viewBox="0 0 16 16" class="mr-2 h-3.5 w-3.5 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
                  {{ t("mods.searching") }}
                </p>
              </div>

              <div class="flex shrink-0 items-center justify-end gap-2 border-t border-[var(--border)] px-4 py-3">
                <a
                  v-if="fileDetailExternalUrl()"
                  href="#"
                  class="flex items-center gap-1.5 rounded-md border border-[var(--border)] bg-[var(--bg)] px-2.5 py-1.5 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                  @click.prevent="openExternal(fileDetailExternalUrl()!)"
                >
                  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M3.75 2h3.5a.75.75 0 0 1 0 1.5h-2l6 6V7.5a.75.75 0 0 1 1.5 0v4.5a.75.75 0 0 1-.75.75H5.5a.75.75 0 0 1 0-1.5h2l-6-6v2a.75.75 0 0 1-1.5 0V3.5A1.75 1.75 0 0 1 1.75 1.75h2a.75.75 0 0 1 0 1.5Z"/></svg>
                  {{ t("files.openPage") }}
                </a>
              </div>
            </div>
          </div>
        </template>

        <!-- ======= Вкладка: Новости ======= -->
        <template v-else-if="tab === 'news'">
          <div class="flex min-h-0 flex-1 flex-col">
            <div class="mb-6 shrink-0 border-b border-[var(--border)] pb-5">
              <h1 class="text-xl font-bold tracking-tight text-[color:var(--tx-strong)]">{{ t("news.title") }}</h1>
              <p class="mt-2 text-xs text-[color:var(--tx-muted)]">
                {{ t("news.subtitle") }}
              </p>
              <div class="mt-4 flex flex-wrap items-center gap-2">
                <button
                  v-for="src in newsSources"
                  :key="src"
                  type="button"
                  class="rounded-full border px-3.5 py-1.5 text-xs font-medium transition-colors"
                  :class="newsFilter === src
                    ? 'border-[color-mix(in_srgb,var(--accent-deep)_60%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] text-white'
                    : 'border-[var(--border)] bg-[var(--input)] text-[color:var(--tx-muted)] hover:bg-[var(--hover)] hover:text-[color:var(--tx)]'"
                  @click="newsFilter = src"
                >
                  {{ src === "launcher" ? "Mono Launcher" : packNameFor(src) }}
                </button>
                <button
                  type="button"
                  class="rounded-full border px-3.5 py-1.5 text-xs font-medium transition-colors"
                  :class="newsFilter === 'all'
                    ? 'border-[color-mix(in_srgb,var(--accent-deep)_60%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] text-white'
                    : 'border-[var(--border)] bg-[var(--input)] text-[color:var(--tx-muted)] hover:bg-[var(--hover)] hover:text-[color:var(--tx)]'"
                  @click="newsFilter = 'all'"
                >
                  {{ t("news.all") }}
                </button>
              </div>
            </div>

            <div v-if="news === null" class="flex flex-1 items-center justify-center text-xs text-[color:var(--tx-muted)]">
              <svg class="mr-2 h-4 w-4 animate-spin fill-[var(--accent)]" viewBox="0 0 16 16">
                <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
              </svg>
              {{ t("news.loading") }}
            </div>

            <div v-else-if="news.length === 0" class="shrink-0 rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-8 text-center text-xs text-[color:var(--tx-muted)]">
              {{ t("news.none") }}
            </div>

            <div v-else-if="filteredNews.length === 0" class="shrink-0 rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-8 text-center text-xs text-[color:var(--tx-muted)]">
              {{ t("news.emptyCat") }}
            </div>

            <div v-else class="min-h-0 flex-1 space-y-4 overflow-y-auto pr-1 pb-8">
              <article
                v-for="n in filteredNews"
                :key="`${n.kind}-${n.url || n.tag}`"
                class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm transition-shadow hover:shadow-md"
              >
                <div class="flex items-start justify-between gap-3 border-b border-[var(--border)] px-4 py-3">
                  <div class="min-w-0">
                    <div class="flex items-center gap-2 flex-wrap">
                      <span
                        class="rounded-full px-2 py-0.5 text-[11px] font-medium border"
                        :class="n.kind === 'update'
                          ? 'border-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_10%,transparent)] text-[var(--accent)]'
                          : 'border-[#9e6a03]/40 bg-[#9e6a03]/10 text-[#d29922]'"
                      >
                        {{ n.kind === "update" ? t("news.update") : t("news.post") }}
                      </span>
                      <span v-if="n.category" class="rounded-full border border-[var(--border)] bg-[var(--bg)] px-2 py-0.5 text-[11px] font-medium text-[color:var(--tx-muted)]">
                        {{ n.category }}
                      </span>
                      <span class="rounded-full border border-[var(--border)] bg-[var(--bg)] px-2 py-0.5 text-[11px] font-medium text-[color:var(--tx-muted)]">
                        {{ n.pack_name }}
                      </span>
                      <span v-if="n.kind === 'update' && n.tag" class="font-mono text-xs font-semibold text-[var(--accent)]">
                        {{ n.tag }}
                      </span>
                    </div>
                    <h2 class="mt-1.5 text-sm font-semibold text-[color:var(--tx-strong)] break-words">
                      {{ n.title }}
                    </h2>
                  </div>
                  <div class="flex shrink-0 flex-col items-end gap-1.5">
                    <span class="text-xs text-[color:var(--tx-muted)]">
                      {{ formatDate(n.date) }}
                    </span>
                    <div class="flex gap-1.5">
                      <button
                        v-if="n.kind === 'post' && n.url"
                        type="button"
                        class="rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] hover:text-white"
                        @click="openNewsLink(n.url)"
                      >
                        {{ t("news.open") }}
                      </button>
                      <button
                        v-else-if="n.kind === 'update' && n.pack_id === 'launcher' && n.url"
                        type="button"
                        class="rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] hover:text-white"
                        @click="openNewsLink(n.url)"
                      >
                        {{ t("news.open") }}
                      </button>
                      <button
                        v-if="n.kind === 'update' && n.pack_id !== 'launcher' && n.tag"
                        type="button"
                        class="rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] disabled:opacity-50"
                        :disabled="busy"
                        @click="installNews(n)"
                      >
                        {{ isInstalledVersion(n.tag) ? (isActiveNewsTag(n.tag) ? t("releases.selected") : t("releases.switch")) : t("releases.install") }}
                      </button>
                    </div>
                  </div>
                </div>

                <!-- Тело: ченджлог/пост -->
                <div v-if="changelogLines(n.body).length > 0" class="p-4 text-xs text-[color:var(--tx)] space-y-1.5">
                  <div class="changelog space-y-1 font-sans" @click="onChangelogLinkClick">
                    <template v-for="(line, idx) in visibleNewsLines(n)" :key="idx">
                      <div v-if="line.type === 'bullet'" class="flex items-start gap-2 text-[color:var(--tx)]">
                        <span class="text-[color:var(--tx-muted)] select-none">•</span>
                        <span v-html="renderInline(line.text)"></span>
                      </div>
                      <div v-else-if="line.type === 'body'" class="font-semibold text-[color:var(--tx-strong)] pt-1.5" v-html="renderInline(line.text)"></div>
                      <div v-else class="text-[color:var(--tx-muted)]" v-html="renderInline(line.text)"></div>
                    </template>
                  </div>
                  <button
                    v-if="isNewsExpandable(n)"
                    type="button"
                    class="mt-2 inline-block text-xs font-medium text-[var(--accent)] hover:underline"
                    @click="toggleNewsExpanded(n)"
                  >
                    {{ isNewsExpanded(n) ? t("news.collapse") : t("news.showAll") }}
                  </button>
                </div>
                <div v-else class="p-4 text-xs text-[color:var(--tx-muted)] italic">
                  {{ t("news.noText") }}
                </div>
              </article>
            </div>
          </div>
        </template>

        <!-- ======= Вкладка: Каталог сборок ======= -->
        <template v-else-if="tab === 'catalog'">
          <div class="flex min-h-0 flex-1 flex-col">
            <div class="mb-5 flex shrink-0 items-center justify-between gap-4 border-b border-[var(--border)] pb-4">
              <div>
                <h2 class="text-xl font-bold tracking-tight text-[color:var(--tx-strong)]">{{ t("catalog.title") }}</h2>
                <p class="mt-1 text-xs text-[color:var(--tx-muted)]">{{ t("catalog.subtitle") }}</p>
              </div>
              <div class="flex shrink-0 items-center gap-2">
                <button
                  type="button"
                  class="rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] disabled:opacity-50"
                  :disabled="monoCatalogLoading"
                  @click="loadMonoCatalog"
                >
                  {{ t("catalog.refresh") }}
                </button>
                <button
                  type="button"
                  class="rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_10%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)]"
                  @click="openExternal('https://github.com/n1orio/mono-launcher/issues/new?title=' + encodeURIComponent(t('catalog.proposeTitle')) + '&body=' + encodeURIComponent(t('catalog.proposeBody')))"
                >
                  {{ t("catalog.propose") }}
                </button>
              </div>
            </div>

            <div v-if="!catalogDetail" class="mb-4 flex shrink-0 items-center gap-1 rounded-xl border border-[var(--border)] bg-[var(--panel)] p-1 shadow-sm">
              <button
                v-for="src in (['mono', 'modrinth', 'curse'] as const)"
                :key="src"
                type="button"
                class="flex flex-1 items-center justify-center gap-2 rounded-lg px-3 py-2 text-xs font-semibold transition-colors"
                :class="catalogSource === src
                  ? 'bg-[var(--input)] text-[color:var(--tx-strong)]'
                  : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]'"
                @click="switchCatalogSource(src)"
              >
                <svg v-if="src === 'mono'" viewBox="0 0 24 24" class="h-4 w-4 rounded-[4px] bg-[var(--accent)] p-[3px] fill-[color:var(--panel)]"><path d="M3 8.4 8.4 3h7.2L21 8.4v7.2L15.6 21H8.4L3 15.6V8.4Zm2 1.3v4.6L8.3 19H9.7l2.5-6.2L14.7 19h1.4L19 14.3V9.7L15.7 5H9.9L5 9.7Z"/></svg>
                <svg v-else-if="src === 'modrinth'" viewBox="0 0 24 24" class="h-4 w-4 fill-current"><path fill="#00AF5C" d="M12.252.004a11.78 11.768 0 0 0-8.92 3.73 11 10.999 0 0 0-2.17 3.11 11.37 11.359 0 0 0-1.16 5.169c0 1.42.17 2.5.6 3.77.24.759.77 1.899 1.17 2.529a12.3 12.298 0 0 0 8.85 5.639c.44.05 2.54.07 2.76.02.2-.04.22.1-.26-1.7l-.36-1.37-1.01-.06a8.5 8.489 0 0 1-5.18-1.8 5.34 5.34 0 0 1-1.3-1.26c0-.05.34-.28.74-.5a37.572 37.545 0 0 1 2.88-1.629c.03 0 .5.45 1.06.98l1 .97 2.07-.43 2.06-.43 1.47-1.47c.8-.8 1.48-1.5 1.48-1.52 0-.09-.42-1.63-.46-1.7-.04-.06-.2-.03-1.02.18-.53.13-1.2.3-1.45.4l-.48.15-.53.53-.53.53-.93.1-.93.07-.52-.5a2.7 2.7 0 0 1-.96-1.7l-.13-.6.43-.57c.68-.9.68-.9 1.46-1.1.4-.1.65-.2.83-.33.13-.099.65-.579 1.14-1.069l.9-.9-.7-.7-.7-.7-1.95.54c-1.07.3-1.96.53-1.97.53-.03 0-2.23 2.48-2.63 2.97l-.29.35.28 1.03c.16.56.3 1.16.31 1.34l.03.3-.34.23c-.37.23-2.22 1.3-2.84 1.63-.36.2-.37.2-.44.1-.08-.1-.23-.6-.32-1.03-.18-.86-.17-2.75.02-3.73a8.84 8.839 0 0 1 7.9-6.93c.43-.03.77-.08.78-.1.06-.17.5-2.999.47-3.039-.01-.02-.1-.02-.2-.03Zm3.68.67c-.2 0-.3.1-.37.38-.06.23-.46 2.42-.46 2.52 0 .04.1.11.22.16a8.51 8.499 0 0 1 2.99 2 8.38 8.379 0 0 1 2.16 3.449 6.9 6.9 0 0 1 .4 2.8c0 1.07 0 1.27-.1 1.73a9.37 9.369 0 0 1-1.76 3.769c-.32.4-.98 1.06-1.37 1.38-.38.32-1.54 1.1-1.7 1.14-.1.03-.1.06-.07.26.03.18.64 2.56.7 2.78l.06.06a12.07 12.058 0 0 0 7.27-9.4c.13-.77.13-2.58 0-3.4a11.96 11.948 0 0 0-5.73-8.578c-.7-.42-2.05-1.06-2.25-1.06Z"/></svg>
                <svg v-else viewBox="0 0 24 24" class="h-4 w-4 fill-current"><path fill="#F16436" d="M18.326 9.2145S23.2261 8.4418 24 6.1882h-7.5066V4.4H0l2.0318 2.3576V9.173s5.1267-.2665 7.1098 1.2372c2.7146 2.516-3.053 5.917-3.053 5.917L5.0995 19.6c1.5465-1.4726 4.494-3.3775 9.8983-3.2857-2.0565.65-4.1245 1.6651-5.7344 3.2857h10.9248l-1.0288-3.2726s-7.918-4.6688-.8336-7.1127z"/></svg>
                {{ src === "mono" ? t("catalog.sourceMono") : src === "modrinth" ? t("mods.serviceModrinth") : t("mods.serviceCurseforge") }}
              </button>
            </div>

            <div class="min-h-0 flex-1 overflow-y-auto pb-6">
              <template v-if="catalogSource === 'mono'">
              <!-- Catalog Detail View -->
              <template v-if="catalogDetail">
                <div class="flex flex-col h-full">
                  <div class="flex items-center gap-3 mb-4">
                    <button type="button" @click="closeCatalogDetail()" class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-xs text-[color:var(--tx-muted)] hover:text-[var(--accent)] transition-colors">
                      &larr; {{ t("catalog.title") }}
                    </button>
                    <div class="flex items-center gap-2 min-w-0">
                      <img v-if="catalogDetail.icon_url" :src="catalogDetail.icon_url" class="h-8 w-8 shrink-0 rounded object-cover" />
                      <h3 class="truncate text-base font-bold text-[color:var(--tx-strong)]">{{ catalogDetail.name }}</h3>
                      <span v-if="catalogDetail.versions?.length" class="shrink-0 rounded border border-[var(--border)] bg-[var(--input-50)] px-1.5 py-0.5 text-[11px] font-mono text-[color:var(--accent)]">v{{ catalogDetail.versions[0].version }}</span>
                    </div>
                    <div class="ml-auto flex items-center gap-2 shrink-0">
                      <template v-if="catalogDetail.author_user_id">
                        <button
                          type="button"
                          class="max-w-[140px] truncate rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 font-mono text-xs text-[color:var(--tx-muted)] transition-colors hover:text-[var(--accent)]"
                          :title="t('profile.open')"
                          @click="openProfileView(catalogDetail.author_user_id!)"
                        >
                          @{{ catalogDetail.author_name ?? "?" }}
                        </button>
                      </template>
                      <button
                        type="button"
                        class="flex shrink-0 items-center gap-1 rounded-md border px-2 py-1 text-xs font-semibold transition-colors disabled:opacity-50"
                        :class="catalogDetail.my_rating === 1
                          ? 'border-[#3fb950]/50 bg-[#3fb950]/15 text-[#3fb950]'
                          : 'border-[var(--border)] bg-[var(--input)] text-[color:var(--tx-muted)] hover:text-[#3fb950]'"
                        :title="t('comments.like')"
                        :disabled="catalogCommentsBusy"
                        @click="ratePack(catalogDetail.id, 1)"
                      >
                        <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M14.25 6.5c.69 0 1.25.56 1.25 1.25v.5c0 .29-.1.57-.28.79l-3.2 3.88a1.75 1.75 0 0 1-1.35.63H6.1a1.75 1.75 0 0 1-1.35-.63L1.53 9.04a1 1 0 0 1-.28-.68V3.5c0-.675.525-1.225 1.193-1.225h2.934c.51 0 .976.285 1.2.74L8.4 5.5c.09.188.28.31.49.31h5.36ZM2.75 3.775a.225.225 0 0 0-.225.225v4.19l2.946 3.573a.25.25 0 0 0 .193.087h1.06L5.05 6.36a1.75 1.75 0 0 1-.3-.985V3.775Z"/></svg>
                        {{ catalogDetail.likes }}
                      </button>
                      <button
                        type="button"
                        class="flex shrink-0 items-center gap-1 rounded-md border px-2 py-1 text-xs font-semibold transition-colors disabled:opacity-50"
                        :class="catalogDetail.my_rating === -1
                          ? 'border-[#f85149]/50 bg-[#f85149]/15 text-[#f85149]'
                          : 'border-[var(--border)] bg-[var(--input)] text-[color:var(--tx-muted)] hover:text-[#f85149]'"
                        :title="t('comments.dislike')"
                        :disabled="catalogCommentsBusy"
                        @click="ratePack(catalogDetail.id, -1)"
                      >
                        <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 rotate-180 fill-current"><path d="M14.25 6.5c.69 0 1.25.56 1.25 1.25v.5c0 .29-.1.57-.28.79l-3.2 3.88a1.75 1.75 0 0 1-1.35.63H6.1a1.75 1.75 0 0 1-1.35-.63L1.53 9.04a1 1 0 0 1-.28-.68V3.5c0-.675.525-1.225 1.193-1.225h2.934c.51 0 .976.285 1.2.74L8.4 5.5c.09.188.28.31.49.31h5.36ZM2.75 3.775a.225.225 0 0 0-.225.225v4.19l2.946 3.573a.25.25 0 0 0 .193.087h1.06L5.05 6.36a1.75 1.75 0 0 1-.3-.985V3.775Z"/></svg>
                        {{ catalogDetail.dislikes }}
                      </button>
                      <button v-if="!packs.some((p: any) => p.url === catalogDetail!.url)" type="button" @click="addMonoPack({ url: catalogDetail!.url, name: catalogDetail!.name, boosty_blog: catalogDetail!.boosty_blog } as any)" :disabled="addingPack"
                        class="rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] disabled:opacity-50">
                        {{ addingPack ? t("dev.adding") : t("catalog.add") }}
                      </button>
                      <button v-else type="button" @click="openMonoPack({ url: catalogDetail!.url, name: catalogDetail!.name } as any)"
                        class="rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]">
                        {{ t("catalog.open") }}
                      </button>
                    </div>
                  </div>

                  <div v-if="catalogDetailBusy" class="flex items-center justify-center py-16 text-xs text-[color:var(--tx-muted)]">
                    <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
                    {{ t("catalog.loading") }}
                  </div>

                  <template v-else>
                    <!-- Detail tabs -->
                    <div class="mb-4 flex gap-1 overflow-x-auto border-b border-[var(--border)]">
                      <button type="button" @click="catalogDetailTab = 'description'"
                        class="relative shrink-0 px-3.5 pb-2.5 pt-1 text-xs font-semibold transition-colors"
                        :class="catalogDetailTab === 'description' ? 'text-[var(--accent)]' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'">
                        {{ t('pack.description') }}
                        <span v-if="catalogDetailTab === 'description'" class="absolute inset-x-2 bottom-0 h-[2.5px] rounded-t-full bg-[var(--accent)]"></span>
                      </button>
                      <button v-if="(catalogDetail.meta as any)?.screenshots?.length" type="button" @click="catalogDetailTab = 'screenshots'"
                        class="relative shrink-0 px-3.5 pb-2.5 pt-1 text-xs font-semibold transition-colors"
                        :class="catalogDetailTab === 'screenshots' ? 'text-[var(--accent)]' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'">
                        {{ t('pack.screenshots') }} ({{ ((catalogDetail.meta as any)?.screenshots ?? []).length }})
                        <span v-if="catalogDetailTab === 'screenshots'" class="absolute inset-x-2 bottom-0 h-[2.5px] rounded-t-full bg-[var(--accent)]"></span>
                      </button>
                      <button v-if="catalogDetail.versions?.length" type="button" @click="catalogDetailTab = 'versions'"
                        class="relative shrink-0 px-3.5 pb-2.5 pt-1 text-xs font-semibold transition-colors"
                        :class="catalogDetailTab === 'versions' ? 'text-[var(--accent)]' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'">
                        {{ t('pack.versions') }} ({{ catalogDetail.versions.length }})
                        <span v-if="catalogDetailTab === 'versions'" class="absolute inset-x-2 bottom-0 h-[2.5px] rounded-t-full bg-[var(--accent)]"></span>
                      </button>
                      <button v-if="catalogDetail.news?.length" type="button" @click="catalogDetailTab = 'news'"
                        class="relative shrink-0 px-3.5 pb-2.5 pt-1 text-xs font-semibold transition-colors"
                        :class="catalogDetailTab === 'news' ? 'text-[var(--accent)]' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'">
                        {{ t('pack.news') }} ({{ catalogDetail.news.length }})
                        <span v-if="catalogDetailTab === 'news'" class="absolute inset-x-2 bottom-0 h-[2.5px] rounded-t-full bg-[var(--accent)]"></span>
                      </button>
                      <button type="button" @click="catalogDetailTab = 'comments'"
                        class="relative shrink-0 px-3.5 pb-2.5 pt-1 text-xs font-semibold transition-colors"
                        :class="catalogDetailTab === 'comments' ? 'text-[var(--accent)]' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'">
                        {{ t('comments.tab') }} ({{ catalogCommentCount }})
                        <span v-if="catalogDetailTab === 'comments'" class="absolute inset-x-2 bottom-0 h-[2.5px] rounded-t-full bg-[var(--accent)]"></span>
                      </button>
                    </div>

                    <div class="min-h-0 flex-1 overflow-y-auto pr-1">
                      <!-- Description -->
                      <div v-if="catalogDetailTab === 'description'" class="space-y-3">
                        <div class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-4">
                          <div v-if="catalogDetail.description" class="text-[13px] leading-relaxed text-[color:var(--tx)] whitespace-pre-wrap">{{ catalogDetail.description }}</div>
                          <p v-else class="text-xs text-[color:var(--tx-muted)]">{{ t("common.notFound") }}</p>
                        </div>
                        <div v-if="catalogDetail.boosty_blog" class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-3 text-xs">
                          <span class="text-[color:var(--tx-muted)]">Boosty: </span>
                          <a :href="catalogDetail.boosty_blog" class="text-[var(--accent)] hover:underline" @click.prevent="openExternal(catalogDetail.boosty_blog!)">{{ catalogDetail.boosty_blog }}</a>
                        </div>
                        <div v-if="catalogDetail.min_ram_mb" class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-3 text-xs">
                          <span class="text-[color:var(--tx-muted)]">{{ t("pack.minRam") || "Мин. RAM" }}: </span>
                          <span>{{ catalogDetail.min_ram_mb }} MB</span>
                        </div>
                      </div>

                      <!-- Screenshots -->
                      <div v-if="catalogDetailTab === 'screenshots'" class="space-y-3">
                        <div v-if="!(catalogDetail.meta as any)?.screenshots?.length" class="text-center py-8 text-xs text-[color:var(--tx-muted)]">{{ t("common.notFound") }}</div>
                        <div v-for="(s, i) in ((catalogDetail.meta as any)?.screenshots || [])" :key="i" class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
                          <img :src="typeof s === 'string' ? s : s.url" :alt="`Screenshot ${i + 1}`" class="w-full object-cover max-h-64" loading="lazy" />
                        </div>
                      </div>

                      <!-- Versions -->
                      <div v-if="catalogDetailTab === 'versions'" class="space-y-2">
                        <div v-if="!catalogDetail.versions?.length" class="text-center py-8 text-xs text-[color:var(--tx-muted)]">{{ t("common.notFound") }}</div>
                        <div v-for="v in catalogDetail.versions" :key="v.id" class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-3 flex items-center justify-between gap-3">
                          <div class="min-w-0">
                            <div class="flex items-center gap-2">
                              <span class="font-mono text-xs font-bold text-[var(--accent)]">v{{ v.version }}</span>
                              <span class="text-[11px] text-[color:var(--tx-muted)]">{{ formatDate(v.created_at) }}</span>
                              <span class="text-[11px] text-[color:var(--tx-muted)]">{{ formatBytes(v.size) }}</span>
                            </div>
                            <p v-if="v.changelog" class="mt-1 text-xs text-[color:var(--tx-muted)] whitespace-pre-wrap line-clamp-2">{{ v.changelog }}</p>
                          </div>
                          <a :href="v.url" class="shrink-0 rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)]">
                            {{ t("pack.download") }}
                          </a>
                        </div>
                      </div>

                      <!-- News -->
                      <div v-if="catalogDetailTab === 'news'" class="space-y-2">
                        <div v-if="!catalogDetail.news?.length" class="text-center py-8 text-xs text-[color:var(--tx-muted)]">{{ t("common.notFound") }}</div>
                        <div v-for="n in catalogDetail.news" :key="n.id" class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-3">
                          <div class="flex items-center gap-2 mb-1">
                            <span class="px-1.5 py-0.5 rounded text-[11px] font-medium"
                              :class="n.kind === 'update' ? 'bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] text-[var(--accent)]' : 'bg-green-500/10 text-green-400'">
                              {{ n.kind === 'update' ? 'Update' : 'Post' }}
                            </span>
                            <span class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ n.title }}</span>
                            <span class="text-[11px] text-[color:var(--tx-muted)] ml-auto">{{ formatDate(n.created_at) }}</span>
                          </div>
                          <p class="text-xs text-[color:var(--tx-muted)] whitespace-pre-wrap">{{ n.body }}</p>
                        </div>
                      </div>

                      <!-- Comments -->
                      <div v-if="catalogDetailTab === 'comments'" class="space-y-3">
                        <div v-if="catalogCommentsBusy && catalogComments.length === 0" class="flex items-center justify-center py-8 text-xs text-[color:var(--tx-muted)]">
                          <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
                          {{ t("catalog.loading") }}
                        </div>

                        <!-- Композер нового комментария -->
                        <div v-if="monoProfile" class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-3">
                          <textarea v-model="commentDraft" rows="2" :placeholder="t('comments.placeholder')" class="w-full resize-y rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none"></textarea>
                          <div class="mt-2 flex justify-end">
                            <button type="button" class="rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-3 py-1 text-xs font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] disabled:opacity-50"
                              :disabled="catalogCommentsBusy || !commentDraft.trim()"
                              @click="sendCatalogComment(catalogDetail.id, commentDraft); commentDraft = ''">
                              {{ t("comments.send") }}
                            </button>
                          </div>
                        </div>
                        <div v-else class="rounded-lg border border-dashed border-[var(--border)] bg-[var(--panel)] p-3 text-center text-xs text-[color:var(--tx-muted)]">
                          {{ t("comments.needLogin") }}
                        </div>

                        <div v-if="!catalogCommentsBusy && catalogComments.length === 0" class="text-center py-8 text-xs text-[color:var(--tx-muted)]">{{ t("comments.empty") }}</div>

                        <!-- Дерево: корень + 1 уровень ответов -->
                        <div v-for="c in catalogComments" :key="c.id" class="space-y-2">
                          <div class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-3">
                            <div class="flex items-center gap-2">
                              <button type="button" class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full border border-[var(--border)] bg-[var(--input)] font-mono text-[11px] font-bold text-[color:var(--tx-strong)] transition-colors hover:text-[var(--accent)]"
                                @click="openProfileView(c.userId)">
                                {{ c.user.username?.[0]?.toUpperCase() ?? "?" }}
                              </button>
                              <button type="button" class="font-mono text-xs font-semibold text-[color:var(--tx-strong)] hover:text-[var(--accent)] transition-colors" @click="openProfileView(c.userId)">
                                {{ c.user.displayName || c.user.username }}
                              </button>
                              <span class="text-[11px] text-[color:var(--tx-muted)]">{{ formatDate(c.createdAt) }}</span>
                              <div class="ml-auto flex shrink-0 items-center gap-1">
                                <button type="button" class="flex items-center gap-1 rounded border px-1.5 py-0.5 text-[11px] font-semibold transition-colors"
                                  :class="c.myRating === 1 ? 'border-[#3fb950]/50 bg-[#3fb950]/15 text-[#3fb950]' : 'border-[var(--border)] bg-[var(--input)] text-[color:var(--tx-muted)] hover:text-[#3fb950]'"
                                  :title="t('comments.like')"
                                  @click="rateCatalogComment(catalogDetail.id, c.id, 1)">
                                  👍 {{ c.likes }}
                                </button>
                                <button type="button" class="flex items-center gap-1 rounded border px-1.5 py-0.5 text-[11px] font-semibold transition-colors"
                                  :class="c.myRating === -1 ? 'border-[#f85149]/50 bg-[#f85149]/15 text-[#f85149]' : 'border-[var(--border)] bg-[var(--input)] text-[color:var(--tx-muted)] hover:text-[#f85149]'"
                                  :title="t('comments.dislike')"
                                  @click="rateCatalogComment(catalogDetail.id, c.id, -1)">
                                  👎 {{ c.dislikes }}
                                </button>
                              </div>
                            </div>

                            <!-- Редактирование своего комментария -->
                            <template v-if="commentEditId === c.id">
                              <textarea v-model="commentEditDraft" rows="2" class="mt-2 w-full resize-y rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] focus:border-[var(--accent)] focus:outline-none"></textarea>
                              <div class="mt-1.5 flex gap-2">
                                <button type="button" class="rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-3 py-1.5 text-xs font-semibold text-white disabled:opacity-50"
                                  :disabled="catalogCommentsBusy || !commentEditDraft.trim()"
                                  @click="editCatalogComment(catalogDetail.id, c.id, commentEditDraft); commentEditId = null">
                                  {{ t("author.save") }}
                                </button>
                                <button type="button" class="rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1 text-xs text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="commentEditId = null">
                                  {{ t("author.cancel") }}
                                </button>
                              </div>
                            </template>
                            <p v-else class="mt-1.5 text-xs leading-relaxed text-[color:var(--tx)] whitespace-pre-wrap">{{ c.body }}</p>

                            <div class="mt-2 flex items-center gap-2">
                              <button v-if="monoProfile && c.parentId === null" type="button" class="text-[11px] font-medium text-[var(--accent)] hover:underline" @click="commentReplyTo = commentReplyTo === c.id ? null : c.id; commentReplyDraft = ''">
                                {{ commentReplyTo === c.id ? t("author.cancel") : t("comments.reply") }}
                              </button>
                              <button v-if="monoProfile?.uuid === c.userId" type="button" class="text-[11px] font-medium text-[color:var(--tx-muted)] hover:text-[var(--accent)]"
                                @click="commentEditId = c.id; commentEditDraft = c.body">
                                {{ t("comments.edit") }}
                              </button>
                              <button v-if="monoProfile?.uuid === c.userId || isAdmin" type="button" class="text-[11px] font-medium text-[#f87171] hover:underline" @click="removeCatalogComment(catalogDetail.id, c.id)">
                                {{ t("author.delete") }}
                              </button>
                            </div>

                            <!-- Форма ответа -->
                            <div v-if="commentReplyTo === c.id" class="mt-2 rounded-md border border-[var(--border)] bg-[var(--bg)] p-2">
                              <textarea v-model="commentReplyDraft" rows="2" :placeholder="t('comments.placeholder')" class="w-full resize-y rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none"></textarea>
                              <div class="mt-1.5 flex justify-end gap-2">
                                <button type="button" class="rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-3 py-1.5 text-xs font-semibold text-white disabled:opacity-50"
                                  :disabled="catalogCommentsBusy || !commentReplyDraft.trim()"
                                  @click="sendCatalogComment(catalogDetail.id, commentReplyDraft, c.id); commentReplyTo = null; commentReplyDraft = ''">
                                  {{ t("comments.send") }}
                                </button>
                              </div>
                            </div>
                          </div>

                          <!-- Ответы (1 уровень) -->
                          <div v-for="r in c.replies" :key="r.id" class="ml-6 rounded-lg border border-[var(--border)] bg-[var(--input-50)] p-3">
                            <div class="flex items-center gap-2">
                              <button type="button" class="flex h-5 w-5 shrink-0 items-center justify-center rounded-full border border-[var(--border)] bg-[var(--input)] font-mono text-[10px] font-bold text-[color:var(--tx-strong)] transition-colors hover:text-[var(--accent)]"
                                @click="openProfileView(r.userId)">
                                {{ r.user.username?.[0]?.toUpperCase() ?? "?" }}
                              </button>
                              <button type="button" class="font-mono text-xs font-semibold text-[color:var(--tx-strong)] hover:text-[var(--accent)] transition-colors" @click="openProfileView(r.userId)">
                                {{ r.user.displayName || r.user.username }}
                              </button>
                              <span class="text-[11px] text-[color:var(--tx-muted)]">{{ formatDate(r.createdAt) }}</span>
                              <div class="ml-auto flex shrink-0 items-center gap-1">
                                <button type="button" class="flex items-center gap-1 rounded border px-1.5 py-0.5 text-[11px] font-semibold transition-colors"
                                  :class="r.myRating === 1 ? 'border-[#3fb950]/50 bg-[#3fb950]/15 text-[#3fb950]' : 'border-[var(--border)] bg-[var(--input)] text-[color:var(--tx-muted)] hover:text-[#3fb950]'"
                                  @click="rateCatalogComment(catalogDetail.id, r.id, 1)">
                                  👍 {{ r.likes }}
                                </button>
                                <button type="button" class="flex items-center gap-1 rounded border px-1.5 py-0.5 text-[11px] font-semibold transition-colors"
                                  :class="r.myRating === -1 ? 'border-[#f85149]/50 bg-[#f85149]/15 text-[#f85149]' : 'border-[var(--border)] bg-[var(--input)] text-[color:var(--tx-muted)] hover:text-[#f85149]'"
                                  @click="rateCatalogComment(catalogDetail.id, r.id, -1)">
                                  👎 {{ r.dislikes }}
                                </button>
                              </div>
                            </div>
                            <template v-if="commentEditId === r.id">
                              <textarea v-model="commentEditDraft" rows="2" class="mt-2 w-full resize-y rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] focus:border-[var(--accent)] focus:outline-none"></textarea>
                              <div class="mt-1.5 flex gap-2">
                                <button type="button" class="rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-3 py-1.5 text-xs font-semibold text-white disabled:opacity-50"
                                  :disabled="catalogCommentsBusy || !commentEditDraft.trim()"
                                  @click="editCatalogComment(catalogDetail.id, r.id, commentEditDraft); commentEditId = null">
                                  {{ t("author.save") }}
                                </button>
                                <button type="button" class="rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1 text-xs text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="commentEditId = null">
                                  {{ t("author.cancel") }}
                                </button>
                              </div>
                            </template>
                            <p v-else class="mt-1.5 text-xs leading-relaxed text-[color:var(--tx)] whitespace-pre-wrap">{{ r.body }}</p>
                            <div class="mt-2 flex items-center gap-2">
                              <button v-if="monoProfile?.uuid === r.userId" type="button" class="text-[11px] font-medium text-[color:var(--tx-muted)] hover:text-[var(--accent)]"
                                @click="commentEditId = r.id; commentEditDraft = r.body">
                                {{ t("comments.edit") }}
                              </button>
                              <button v-if="monoProfile?.uuid === r.userId || isAdmin" type="button" class="text-[11px] font-medium text-[#f87171] hover:underline" @click="removeCatalogComment(catalogDetail.id, r.id)">
                                {{ t("author.delete") }}
                              </button>
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>
                  </template>
                </div>
              </template>

              <!-- Catalog List View -->
              <template v-else>
              <div v-if="monoCatalogLoading && monoCatalog.length === 0" class="flex items-center justify-center py-16 text-xs text-[color:var(--tx-muted)]">
                <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current">
                  <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                </svg>
                {{ t("catalog.loading") }}
              </div>
              <div v-else-if="monoCatalogError && monoCatalog.length === 0" class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-8 text-center text-xs text-[color:var(--tx-muted)]">
                <p class="mb-3">{{ t("catalog.err", { e: monoCatalogError }) }}</p>
                <button type="button" class="text-[var(--accent)] hover:underline" @click="loadMonoCatalog">
                  {{ t("catalog.retry") }}
                </button>
              </div>
              <div v-else-if="monoCatalog.length === 0" class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-8 text-center text-xs text-[color:var(--tx-muted)]">
                {{ t("catalog.emptyMono") }}
              </div>
              <div v-else class="grid grid-cols-1 gap-4 sm:grid-cols-2">
                <article
                  v-for="entry in monoCatalog"
                  :key="entry.id"
                  class="flex cursor-pointer flex-col overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm transition-all hover:border-[color-mix(in_srgb,var(--accent)_40%,transparent)] hover:shadow-md"
                  @click="openCatalogDetail(entry)"
                >
                  <div class="flex flex-1 flex-col p-5">
                  <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0">
                      <div class="flex items-center gap-2.5">
                        <img v-if="entry.icon_url" :src="entry.icon_url" :alt="entry.name" loading="lazy" @error="(e: any) => (e.target.style.display = 'none')" class="h-9 w-9 shrink-0 rounded-lg border border-[var(--border)] object-cover" />
                        <div v-else class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-[var(--border)] bg-[var(--input)] text-xs font-bold text-[var(--accent)]">{{ entry.name?.[0]?.toUpperCase() }}</div>
                        <h3 class="truncate text-[15px] font-semibold text-[color:var(--tx-strong)]">{{ entry.name }}</h3>
                      </div>
                      <div v-if="entry.author_name" class="mt-1 font-mono text-xs text-[color:var(--tx-muted)]">
                        @{{ entry.author_name }}
                      </div>
                    </div>
                    <div class="flex shrink-0 flex-wrap items-center gap-1.5">
                      <span
                        v-if="entry.boosty_blog"
                        class="inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-semibold"
                        :class="isMonoPackAdded(entry) ? 'opacity-60' : ''"
                      >
                        <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                          <path d="M7.75.5A4.5 4.5 0 0 1 11.5 5.5v.85A4.5 4.5 0 0 1 13 10v3A2.5 2.5 0 0 1 10.5 15.5h-6A2.5 2.5 0 0 1 2 13v-3a4.5 4.5 0 0 1 1.5-3.35V5.5A4.25 4.25 0 0 1 7.75.5Zm0 1.5a2.75 2.75 0 0 0-2.75 2.75v.5h5.5v-.5A2.75 2.75 0 0 0 7.75 2Z"/>
                        </svg>
                        {{ t("catalog.paid") }}
                      </span>
                      <span
                        v-if="entry.min_ram_mb"
                        class="rounded-full border border-[var(--border)] px-2 py-0.5 text-[11px] font-medium text-[color:var(--tx-muted)]"
                      >
                        ≥ {{ entry.min_ram_mb / 1024 }} {{ t("units.gb") }}
                      </span>
                    </div>
                  </div>
                  <p v-if="entry.description" class="mt-3 min-h-0 flex-1 text-[13px] leading-relaxed text-[color:var(--tx-muted)] line-clamp-3">
                    {{ entry.description }}
                  </p>
                  <div class="mt-3 flex flex-wrap items-center gap-1.5">
                    <span v-if="entry.version" class="rounded-full border border-[var(--border)] bg-[var(--input-50)] px-2 py-0.5 text-xs text-[color:var(--tx-muted)]">
                      v{{ entry.version }}
                    </span>
                    <span v-if="entry.size" class="rounded-full border border-[var(--border)] bg-[var(--input-50)] px-2 py-0.5 text-xs text-[color:var(--tx-muted)]">
                      {{ formatBytes(entry.size) }}
                    </span>
                    <span v-if="entry.rating" class="inline-flex items-center gap-1 rounded-full border border-[var(--border)] bg-[var(--input-50)] px-2 py-0.5 text-xs text-[color:var(--tx-muted)]">
                      <svg viewBox="0 0 16 16" class="h-3 w-3 fill-[var(--accent)]"><path d="M8 1.3 9.9 5l4 .56-2.9 2.8.7 4L8 10.38 4.3 12.36l.7-4L2.1 5.56 6.1 5 8 1.3Z"/></svg>
                      {{ entry.rating }}
                      <template v-if="entry.likes + entry.dislikes">({{ entry.likes }}👍/{{ entry.dislikes }}👎)</template>
                    </span>
                  </div>
                  <div class="mt-4 flex items-center gap-2 border-t border-[var(--border)] pt-3.5">
                    <button
                      type="button"
                      v-if="!isMonoPackAdded(entry)"
                      class="flex-1 rounded-lg border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-3 py-2 text-xs font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] disabled:opacity-50"
                      :disabled="addingPack"
                      @click.stop="addMonoPack(entry)"
                    >
                      {{ addingPack ? t("dev.adding") : t("catalog.add") }}
                    </button>
                    <button
                      type="button"
                      v-else
                      class="flex-1 rounded-lg border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                      @click.stop="openMonoPack(entry)"
                    >
                      {{ t("catalog.open") }}
                    </button>
                    <button
                      type="button"
                      class="shrink-0 rounded-lg border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx-muted)] transition-colors hover:text-[var(--accent)]"
                      :title="t('catalog.detailsHint')"
                      @click.stop="openCatalogDetail(entry)"
                    >
                      <span class="inline-flex items-center gap-1.5">
                        <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
                          <path d="M8 1.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13ZM7.25 5.25a.75.75 0 1 1 1.5 0 .75.75 0 0 1-1.5 0Zm.5 2.25h.5a.75.75 0 0 1 .75.75v3a.75.75 0 0 1-1.5 0V8a.75.75 0 0 1 .75-.75Z" transform="translate(0 .5)"/>
                        </svg>
                        {{ t("catalog.details") }}
                      </span>
                    </button>
                    <button
                      type="button"
                      class="shrink-0 rounded-lg border border-[var(--border)] bg-[var(--input)] px-2.5 py-2 text-xs text-[color:var(--tx-muted)] transition-colors hover:text-[var(--accent)]"
                      :title="entry.url"
                      @click.stop="openExternal(entry.url)"
                    >
                      <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
                        <path d="M3.75 2h3.5a.75.75 0 0 1 0 1.5h-2l6 6V7.5a.75.75 0 0 1 1.5 0v4.5a.75.75 0 0 1-.75.75H5.5a.75.75 0 0 1 0-1.5h2l-6-6v2a.75.75 0 0 1-1.5 0V3.5A1.75 1.75 0 0 1 1.75 1.75h2a.75.75 0 0 1 0 1.5Z"/>
                      </svg>
                    </button>
                  </div>
                  </div>
                </article>
              </div>
              </template>
              </template>

              <template v-else-if="catalogSource === 'modrinth'">
                <div class="mb-3 flex shrink-0 flex-wrap items-center gap-2">
                  <FilterSelect
                    v-model="packFilters.versions"
                    :options="packVersionOptions"
                    :placeholder="t('mods.fVersion')"
                    @change="searchPacks()"
                  />
                  <FilterSelect
                    v-model="packFilters.loaders"
                    :options="packLoaderOptions"
                    :placeholder="t('mods.fLoader')"
                    @change="searchPacks()"
                  />
                    <FilterSelect
                      v-model="packVersionTypeSel"
                      :options="versionTypeOptions"
                      :placeholder="t('mods.fType')"
                      :multiple="false"
                      @change="searchPacks()"
                    />
                  <FilterSelect
                    v-model="packFilters.categories"
                    :options="packCategoryOptions"
                    :placeholder="t('mods.fCategory')"
                    @change="searchPacks()"
                  />
                  <FilterSelect
                    v-model="packEnvSel"
                    :options="envOptions"
                    :placeholder="t('mods.fAny')"
                    :multiple="false"
                    @change="searchPacks()"
                  />
                  <FilterSelect
                    v-model="packSortSel"
                    :options="sortSelectOptions"
                    :placeholder="t('mods.fSort')"
                    :multiple="false"
                    @change="searchPacks()"
                  />
                </div>
                <div class="mb-3 flex items-center gap-2">
                  <div class="relative min-w-0 flex-1">
                    <svg viewBox="0 0 16 16" class="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 fill-[var(--tx-muted)]">
                      <path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"/>
                    </svg>
                    <input
                      v-model="modPackQuery"
                      type="text"
                      :placeholder="t('mods.packsPlaceholder')"
                      class="w-full rounded-md border border-[var(--border)] bg-[var(--bg)] py-1.5 pl-8 pr-3 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] outline-none transition-colors focus:border-[var(--accent)]"
                      @keydown.enter="searchPacks"
                    />
                  </div>
                  <button
                    type="button"
                    class="flex shrink-0 items-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                    :disabled="modPackLoading"
                    @click="searchPacks"
                  >
                    {{ t("mods.search") }}
                  </button>
                </div>
                <div v-if="modPackLoading" class="flex items-center justify-center py-16 text-xs text-[color:var(--tx-muted)]">
                  <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current">
                    <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                  </svg>
                  {{ t("mods.searchingAll") }}
                </div>
                <div v-else-if="modPackResults.length === 0" class="py-16 text-center text-xs text-[color:var(--tx-muted)]">
                  {{ modPackQuery ? t("mods.noResults") : t("mods.packsHelp") }}
                </div>
                <div v-else class="space-y-2">
                  <div
                    v-for="p in modPackResults"
                    :key="p.projectId"
                    class="flex cursor-pointer items-start gap-3 rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2.5 transition-colors hover:border-[color-mix(in_srgb,var(--accent)_50%,transparent)]"
                    @click="openCatalogModrinthDetail(p)"
                  >
                    <img v-if="p.iconUrl" :src="p.iconUrl" alt="" loading="lazy" class="h-10 w-10 shrink-0 rounded-md object-cover" />
                    <div v-else class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-[11px] text-[color:var(--tx-muted)]">
                      {{ p.title.slice(0, 2).toUpperCase() }}
                    </div>
                    <div class="min-w-0 flex-1">
                      <div class="flex flex-wrap items-center gap-x-2">
                        <svg viewBox="0 0 24 24" class="h-3 w-3 shrink-0 self-center" :title="t('mods.serviceModrinth')"><path fill="#00AF5C" d="M12.252.004a11.78 11.768 0 0 0-8.92 3.73 11 10.999 0 0 0-2.17 3.11 11.37 11.359 0 0 0-1.16 5.169c0 1.42.17 2.5.6 3.77.24.759.77 1.899 1.17 2.529a12.3 12.298 0 0 0 8.85 5.639c.44.05 2.54.07 2.76.02.2-.04.22.1-.26-1.7l-.36-1.37-1.01-.06a8.5 8.489 0 0 1-5.18-1.8 5.34 5.34 0 0 1-1.3-1.26c0-.05.34-.28.74-.5a37.572 37.545 0 0 1 2.88-1.629c.03 0 .5.45 1.06.98l1 .97 2.07-.43 2.06-.43 1.47-1.47c.8-.8 1.48-1.5 1.48-1.52 0-.09-.42-1.63-.46-1.7-.04-.06-.2-.03-1.02.18-.53.13-1.2.3-1.45.4l-.48.15-.53.53-.53.53-.93.1-.93.07-.52-.5a2.7 2.7 0 0 1-.96-1.7l-.13-.6.43-.57c.68-.9.68-.9 1.46-1.1.4-.1.65-.2.83-.33.13-.099.65-.579 1.14-1.069l.9-.9-.7-.7-.7-.7-1.95.54c-1.07.3-1.96.53-1.97.53-.03 0-2.23 2.48-2.63 2.97l-.29.35.28 1.03c.16.56.3 1.16.31 1.34l.03.3-.34.23c-.37.23-2.22 1.3-2.84 1.63-.36.2-.37.2-.44.1-.08-.1-.23-.6-.32-1.03-.18-.86-.17-2.75.02-3.73a8.84 8.839 0 0 1 7.9-6.93c.43-.03.77-.08.78-.1.06-.17.5-2.999.47-3.039-.01-.02-.1-.02-.2-.03Zm3.68.67c-.2 0-.3.1-.37.38-.06.23-.46 2.42-.46 2.52 0 .04.1.11.22.16a8.51 8.499 0 0 1 2.99 2 8.38 8.379 0 0 1 2.16 3.449 6.9 6.9 0 0 1 .4 2.8c0 1.07 0 1.27-.1 1.73a9.37 9.369 0 0 1-1.76 3.769c-.32.4-.98 1.06-1.37 1.38-.38.32-1.54 1.1-1.7 1.14-.1.03-.1.06-.07.26.03.18.64 2.56.7 2.78l.06.06a12.07 12.058 0 0 0 7.27-9.4c.13-.77.13-2.58 0-3.4a11.96 11.948 0 0 0-5.73-8.578c-.7-.42-2.05-1.06-2.25-1.06Z"/></svg>
                        <span class="truncate text-xs font-semibold text-[color:var(--tx-strong)]">{{ p.title }}</span>
                        <span class="text-[11px] text-[color:var(--tx-muted)]">{{ t("mods.byAuthor", { author: p.author }) }}</span>
                      </div>
                      <p class="mt-0.5 line-clamp-2 text-xs leading-snug text-[color:var(--tx-muted)]">{{ p.description }}</p>
                      <p class="mt-1 flex items-center gap-1 text-[11px] text-[color:var(--tx-muted)]">
                        <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1.75a.75.75 0 0 0-1.5 0v9A2.25 2.25 0 0 0 2.5 13h12.75a.75.75 0 0 0 0-1.5H2.5a.75.75 0 0 1-.75-.75v-9Zm10.75 2.5a.75.75 0 0 0-1.5 0v5a.75.75 0 0 0 1.5 0v-5Zm-3 .75a.75.75 0 0 1 1.5 0v4.25a.75.75 0 0 1-1.5 0V5Zm-3 1.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3Z"/></svg>
                        {{ p.downloads.toLocaleString() }}
                      </p>
                    </div>
                    <button
                      type="button"
                      class="flex shrink-0 items-center gap-1.5 self-center rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                      :disabled="quickPackBusy !== null"
                      :title="t('mods.downloadHint')"
                      @click.stop="quickDownloadPack(p, $event)"
                    >
                      <svg v-if="quickPackBusy === p.projectId" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                        <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                      </svg>
                      <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                        <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
                      </svg>
                      {{ t("mods.download") }}
                    </button>
                  </div>
                </div>
              </template>

              <template v-else-if="catalogSource === 'curse'">
                <div class="mb-3 flex shrink-0 flex-wrap items-center gap-2">
                  <FilterSelect
                    v-model="cpCatSel"
                    :options="cpCatOptions"
                    :placeholder="t('curse.fCategory')"
                    :multiple="false"
                    @change="searchCursePacks"
                  />
                  <FilterSelect
                    v-model="cpVerSel"
                    :options="packVersionOptions"
                    :placeholder="t('mods.fVersion')"
                    :multiple="false"
                    @change="searchCursePacks"
                  />
                  <FilterSelect
                    v-model="cpSortSel"
                    :options="curseSortOptions"
                    :placeholder="t('mods.fSort')"
                    :multiple="false"
                    @change="searchCursePacks"
                  />
                </div>
                <div v-if="!curseKeyOk" class="mb-3 rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-4 text-center text-xs text-[color:var(--tx-muted)]">
                  {{ t("curse.noKey") }}
                </div>
                <div class="mb-3 flex items-center gap-2">
                  <div class="relative min-w-0 flex-1">
                    <svg viewBox="0 0 16 16" class="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 fill-[var(--tx-muted)]">
                      <path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"/>
                    </svg>
                    <input
                      v-model="modPackQuery"
                      type="text"
                      :placeholder="t('curse.packsPlaceholder')"
                      class="w-full rounded-md border border-[var(--border)] bg-[var(--bg)] py-1.5 pl-8 pr-3 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] outline-none transition-colors focus:border-[var(--accent)]"
                      @keydown.enter="searchCursePacks"
                    />
                  </div>
                  <button
                    type="button"
                    class="flex shrink-0 items-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                    :disabled="cpLoading"
                    @click="searchCursePacks"
                  >
                    {{ t("mods.search") }}
                  </button>
                </div>
                <p v-if="!cpSearched" class="py-8 text-center text-xs text-[color:var(--tx-muted)]">{{ t("curse.packsHelp") }}</p>
                <p v-else-if="cpLoading" class="flex items-center justify-center gap-2 py-8 text-xs text-[color:var(--tx-muted)]">
                  <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 animate-spin fill-current">
                    <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                  </svg>
                  {{ t("mods.searchingAll") }}
                </p>
                <div v-else-if="cpErr" class="rounded-md border border-[var(--border)] bg-[var(--input-50)] p-6 text-center text-xs text-[color:var(--tx-muted)]">
                  <p class="mb-2 whitespace-pre-wrap">{{ cpErr }}</p>
                  <button type="button" class="text-[var(--accent)] hover:underline" @click="searchCursePacks">{{ t("catalog.retry") }}</button>
                </div>
                <div v-else-if="cpResults.length === 0" class="py-16 text-center text-xs text-[color:var(--tx-muted)]">
                  {{ modPackQuery ? t("mods.noResults") : t("curse.packsHelp") }}
                </div>
                <div v-else class="space-y-2">
                  <div
                    v-for="p in cpResults"
                    :key="p.projectId"
                    class="flex cursor-pointer items-start gap-3 rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2.5 transition-colors hover:border-[color-mix(in_srgb,var(--accent)_50%,transparent)]"
                    @click="openCatalogCurseDetail(p)"
                  >
                    <img v-if="p.iconUrl" :src="p.iconUrl" alt="" loading="lazy" class="h-10 w-10 shrink-0 rounded-md object-cover" />
                    <div v-else class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-[11px] text-[color:var(--tx-muted)]">
                      {{ p.name.slice(0, 2).toUpperCase() }}
                    </div>
                    <div class="min-w-0 flex-1">
                      <div class="flex flex-wrap items-center gap-x-2">
                        <svg viewBox="0 0 24 24" class="h-3 w-3 shrink-0 self-center" :title="t('mods.serviceCurseforge')"><path fill="#F16436" d="M18.326 9.2145S23.2261 8.4418 24 6.1882h-7.5066V4.4H0l2.0318 2.3576V9.173s5.1267-.2665 7.1098 1.2372c2.7146 2.516-3.053 5.917-3.053 5.917L5.0995 19.6c1.5465-1.4726 4.494-3.3775 9.8983-3.2857-2.0565.65-4.1245 1.6651-5.7344 3.2857h10.9248l-1.0288-3.2726s-7.918-4.6688-.8336-7.1127z"/></svg>
                        <span class="truncate text-xs font-semibold text-[color:var(--tx-strong)]">{{ p.name }}</span>
                        <span class="text-[11px] text-[color:var(--tx-muted)]">{{ t("mods.byAuthor", { author: p.author }) }}</span>
                      </div>
                      <p class="mt-0.5 line-clamp-2 text-xs leading-snug text-[color:var(--tx-muted)]">{{ p.summary }}</p>
                      <p class="mt-1 flex items-center gap-1 text-[11px] text-[color:var(--tx-muted)]">
                        <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1.75a.75.75 0 0 0-1.5 0v9A2.25 2.25 0 0 0 2.5 13h12.75a.75.75 0 0 0 0-1.5H2.5a.75.75 0 0 1-.75-.75v-9Zm10.75 2.5a.75.75 0 0 0-1.5 0v5a.75.75 0 0 0 1.5 0v-5Zm-3 .75a.75.75 0 0 1 1.5 0v4.25a.75.75 0 0 1-1.5 0V5Zm-3 1.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3Z"/></svg>
                        {{ p.downloadCount.toLocaleString() }}
                      </p>
                    </div>
                    <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 shrink-0 self-center fill-[var(--tx-muted)]"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"/></svg>
                  </div>
                </div>
              </template>
            </div>
          </div>
        </template>

        <!-- ======= Вкладка: Разработчикам ======= -->
        <template v-else-if="tab === 'dev'">
          <div class="min-h-0 flex-1 overflow-y-auto pr-1">
          <div class="space-y-6">
            <div class="border-b border-[var(--border)] pb-3">
              <h1 class="text-xl font-bold tracking-tight text-[color:var(--tx-strong)]">{{ t("dev.title") }}</h1>
              <p class="text-xs text-[color:var(--tx-muted)]">{{ t("dev.subtitle") }}</p>
            </div>

            <!-- Добавление сборки -->
            <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
              <div class="flex items-center justify-between gap-2 border-b border-[var(--border)] px-4 py-3">
                <h3 class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("dev.addTitle") }}</h3>
                <button
                  type="button"
                  class="flex items-center gap-1.5 rounded-md bg-[#238636] px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[#2ea043]"
                  @click="openExamplePack"
                >
                  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                    <path d="M2 1.75C2 .784 2.784 0 3.75 0h3.5a.75.75 0 0 1 0 1.5h-3.5a.25.25 0 0 0-.25.25v12.5c0 .138.112.25.25.25h3.5a.75.75 0 0 1 0 1.5h-3.5A1.75 1.75 0 0 1 2 14.25Zm12.28 4.97a.75.75 0 0 1 0 1.06l-4.75 4.75a.75.75 0 0 1-1.06-1.06l3.47-3.47H5.75a.75.75 0 0 1 0-1.5h6.19l-3.47-3.47a.75.75 0 0 1 1.06-1.06Z"/>
                  </svg>
                  {{ t("dev.createBtn") }}
                </button>
              </div>
              <div class="space-y-3 p-4">
                <div>
                  <label class="mb-1 block text-xs text-[color:var(--tx-muted)]" for="pack-url">{{ t("dev.addUrl") }}</label>
                  <input
                    id="pack-url"
                    v-model="packUrl"
                    type="text"
                    class="w-full rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs text-[color:var(--tx)] outline-none transition-colors placeholder:text-[color:var(--tx-muted)] focus:border-[var(--accent)]"
                    :placeholder="t('dev.addUrlPh')"
                    @keydown.enter="handleAddPack"
                  />
                </div>
                <div>
                  <label class="mb-1 block text-xs text-[color:var(--tx-muted)]" for="pack-name">{{ t("dev.addName") }}</label>
                  <input
                    id="pack-name"
                    v-model="packName"
                    type="text"
                    class="w-full rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs text-[color:var(--tx)] outline-none transition-colors placeholder:text-[color:var(--tx-muted)] focus:border-[var(--accent)]"
                    :placeholder="t('dev.addNamePh')"
                    @keydown.enter="handleAddPack"
                  />
                </div>
                <button
                  type="button"
                  class="rounded-md bg-[#238636] px-4 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[#2ea043] disabled:opacity-50"
                  :disabled="addingPack || busy"
                  @click="handleAddPack"
                >
                  {{ addingPack ? t("dev.adding") : t("dev.addBtn") }}
                </button>
              </div>
            </section>

            <!-- Подключённые сборки -->
            <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
              <div class="border-b border-[var(--border)] px-4 py-3">
                <h3 class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("dev.listTitle") }}</h3>
              </div>
              <div class="divide-y divide-[var(--border)]">
                <div v-if="packs.length === 0" class="p-4 text-xs text-[color:var(--tx-muted)]">
                  {{ t("dev.empty") }}
                </div>
                <div
                  v-for="p in packs"
                  :key="p.id"
                  class="flex items-center gap-3 px-4 py-2.5"
                >
                  <span
                    class="h-2 w-2 shrink-0 rounded-full"
                    :class="p.id === packId ? 'bg-[#3fb950]' : 'bg-[var(--tx-muted)]'"
                  />
                  <div class="min-w-0 flex-1">
                    <div class="truncate text-xs font-medium text-[color:var(--tx)]">
                      {{ p.name }}
                      <span v-if="p.author" class="font-mono text-[11px] text-[var(--accent)]">@{{ p.author }}</span>
                    </div>
                    <div class="truncate font-mono text-[11px] text-[color:var(--tx-muted)]">{{ p.id }}</div>
                  </div>
                  <span v-if="p.builtin" class="shrink-0 rounded border border-[var(--border)] px-1.5 py-0.5 text-[11px] text-[color:var(--tx-muted)]" :title="t('dev.builtinNote')">
                    {{ t("dev.builtin") }}
                  </span>
                  <button
                    v-if="!p.builtin"
                    type="button"
                    class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-[11px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                    :title="t('dev.setIconHint')"
                    @click="pickPackIcon(p.id)"
                  >
                    {{ p.icon ? t("dev.changeIcon") : t("dev.setIcon") }}
                  </button>
                  <button
                    v-else
                    type="button"
                    class="shrink-0 rounded-md border px-2 py-1 text-[11px] font-medium transition-colors disabled:opacity-50"
                    :class="removeArmed === p.id
                      ? 'border-[#f85149]/60 bg-[#f85149]/15 text-[#f85149]'
                      : 'border-[var(--border)] bg-[var(--input)] text-[color:var(--tx)] hover:bg-[var(--hover)]'"
                    :disabled="removingPack === p.id"
                    @click="handleRemovePack(p.id)"
                    @blur="resetRemoveArm"
                  >
                    {{ removingPack === p.id ? t("dev.removing") : removeArmed === p.id ? t("dev.removeConfirm") : t("dev.remove") }}
                  </button>
                </div>
              </div>
            </section>

            <!-- Мини-документация -->
            <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
              <div class="border-b border-[var(--border)] px-4 py-3">
                <h3 class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("dev.docsTitle") }}</h3>
              </div>
              <div class="space-y-2.5 p-4 text-xs leading-relaxed text-[color:var(--tx)]">
                <p>{{ t("dev.docsStep1") }}</p>
                <p>{{ t("dev.docsStep2") }}</p>
                <p>{{ t("dev.docsStep3") }}</p>
                <p>{{ t("dev.docsStep4") }}</p>
                <p>{{ t("dev.docsStep5") }}</p>
                <div class="mt-3 rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_30%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_10%,transparent)] p-3 text-xs text-[color:var(--tx)]">
                  {{ t("dev.docsFormat") }}
                </div>
                <div class="rounded-md border border-[var(--border)] bg-[var(--bg-60)] p-3">
                  <p class="mb-1.5 font-mono text-[11px] text-[color:var(--tx-muted)]">pack.json</p>
                  <pre class="overflow-x-auto text-[11px] leading-relaxed text-[color:var(--tx)]">{{ examplePackJson }}</pre>
                </div>
                <button
                  type="button"
                  class="flex items-center gap-1.5 rounded-md bg-[var(--accent-deep)] px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[var(--accent-hover)]"
                  @click="openExamplePack"
                >
                  <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
                    <path d="M8 1.5a.75.75 0 0 1 .75.75V9.44l1.72-1.72a.75.75 0 1 1 1.06 1.06l-3 3a.75.75 0 0 1-1.06 0l-3-3a.75.75 0 1 1 1.06-1.06l1.72 1.72V2.25A.75.75 0 0 1 8 1.5ZM4 12.25a.75.75 0 0 1 .75.75v.5a.5.5 0 0 0 .5.5h5.5a.5.5 0 0 0 .5-.5v-.5a.75.75 0 0 1 1.5 0v.5a2 2 0 0 1-2 2h-5.5a2 2 0 0 1-2-2v-.5a.75.75 0 0 1 .75-.75Z"/>
                  </svg>
                  {{ t("dev.docsExample") }}
                </button>
                <p class="text-xs text-[color:var(--tx-muted)]">{{ t("dev.docsNews") }}</p>
                <p class="text-xs text-[color:var(--tx-muted)]">{{ t("dev.docsBugs") }}</p>
                <p class="text-xs text-[color:var(--tx-muted)]">{{ t("dev.docsContent") }}</p>
                <div class="space-y-2 font-mono text-[11px] text-[color:var(--tx-muted)]">
                  <div class="overflow-x-auto rounded-md border border-[var(--border)] bg-[var(--bg-60)] px-3 py-2">
                    <div class="mb-1 font-semibold text-[color:var(--tx-strong)]">servers.json</div>
                    <pre class="leading-relaxed">[
  {
    "name": "Главный сервер",
    "ip": "play.example.ru",
    "port": 25565,
    "desc": "Ваниль + PvP-арена"
  }
]</pre>
                  </div>
                  <div class="overflow-x-auto rounded-md border border-[var(--border)] bg-[var(--bg-60)] px-3 py-2">
                    <div class="mb-1 font-semibold text-[color:var(--tx-strong)]">socials.json</div>
                    <pre class="leading-relaxed">{
  "discord": "https://discord.gg/example",
  "telegram": "https://t.me/example",
  "vk": "https://vk.com/example"
}</pre>
                  </div>
                  <div class="overflow-x-auto rounded-md border border-[var(--border)] bg-[var(--bg-60)] px-3 py-2">
                    <div class="mb-1 font-semibold text-[color:var(--tx-strong)]">theme.json</div>
                    <pre class="leading-relaxed">{
  "bg": "#0d1117",
  "panel": "#161b22",
  "accent": "#f0883e",
  "accentDeep": "#d06a1f",
  "tx": "#e6edf3"
}</pre>
                  </div>
                </div>
                <div class="rounded-md border border-[#238636]/30 bg-[#238636]/10 p-3 text-xs text-[color:var(--tx)]">
                  <p class="mb-2 font-semibold text-[#3fb950]">mono://</p>
                  <p class="mb-2">{{ t("dev.docsDeep") }}</p>
                  <code class="block overflow-x-auto rounded bg-[var(--bg-60)] px-2 py-1.5 font-mono text-[11px] text-[color:var(--tx-strong)]">{{ deepLinkExample }}</code>
                  <div class="mt-2.5 flex flex-wrap gap-2">
                    <button
                      type="button"
                      class="flex items-center gap-1.5 rounded-md bg-[#238636] px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[#2ea043]"
                      @click="openExampleInLauncher"
                    >
                      <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
                        <path d="M10.78 5.22a.75.75 0 0 1 0 1.06L8.56 8.5H11.5a.75.75 0 0 1 0 1.5H8.56l2.22 2.22a.75.75 0 1 1-1.06 1.06l-3.5-3.5a.75.75 0 0 1 0-1.06l3.5-3.5a.75.75 0 0 1 1.06 0ZM3.75 4A1.75 1.75 0 0 0 2 5.75v4.5c0 .966.784 1.75 1.75 1.75h3a.75.75 0 0 0 0-1.5h-3a.25.25 0 0 1-.25-.25v-4.5a.25.25 0 0 1 .25-.25h3a.75.75 0 0 0 0-1.5h-3Z"/>
                      </svg>
                      {{ t("dev.docsOpenExample") }}
                    </button>
                    <button
                      v-if="activePack"
                      type="button"
                      class="flex items-center gap-1.5 rounded-md border border-[#238636]/50 bg-[#238636]/10 px-3 py-1.5 text-xs font-semibold text-[#3fb950] transition-colors hover:bg-[#238636]/20"
                      @click="copyInviteLink"
                    >
                      <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
                        <path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25Zm10.108-9.858 3.05 3.05a.75.75 0 0 1 0 1.061l-5.25 5.25a.75.75 0 0 1-1.061 0l-3.05-3.05a.75.75 0 0 1 0-1.061l5.25-5.25a.75.75 0 0 1 1.061 0Z"/>
                      </svg>
                      {{ t("dev.copyInvite") }}
                    </button>
                  </div>
                </div>
              </div>
            </section>
          </div>
          </div>
        </template>

        <template v-else-if="tab === 'library'">
          <div class="flex min-h-0 flex-1 flex-col">
            <div class="mb-5 flex shrink-0 items-center justify-between gap-4 border-b border-[var(--border)] pb-4">
              <div>
                <h2 class="text-xl font-bold tracking-tight text-[color:var(--tx-strong)]">{{ t("nav.library") }}</h2>
                <p class="mt-1 text-xs text-[color:var(--tx-muted)]">{{ t("library.subtitle") }}</p>
              </div>
              <div class="flex shrink-0 items-center gap-1.5">
                <button
                  type="button"
                  class="rounded-md border border-[var(--border)] bg-[var(--input)] p-1.5 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)] disabled:opacity-40"
                  :title="t('library.zoomOut')"
                  :disabled="libScale <= 1"
                  @click="setLibScale(libScale - 1)"
                >
                  <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M3 8a.75.75 0 0 1 .75-.75h8.5a.75.75 0 0 1 0 1.5h-8.5A.75.75 0 0 1 3 8Z"/></svg>
                </button>
                <span class="w-11 text-center text-xs font-semibold tabular-nums text-[color:var(--tx-muted)]">{{ libPercent }}%</span>
                <button
                  type="button"
                  class="rounded-md border border-[var(--border)] bg-[var(--input)] p-1.5 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)] disabled:opacity-40"
                  :title="t('library.zoomIn')"
                  :disabled="libScale >= 4"
                  @click="setLibScale(libScale + 1)"
                >
                  <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M8 3.25a.75.75 0 0 1 .75.75v3.25H12a.75.75 0 0 1 0 1.5H8.75V12a.75.75 0 0 1-1.5 0V8.75H4a.75.75 0 0 1 0-1.5h3.25V4a.75.75 0 0 1 .75-.75Z"/></svg>
                </button>
              </div>
            </div>
            <div class="min-h-0 flex-1 overflow-y-auto pr-1">
              <template v-for="cat in PACK_CATS" :key="cat">
                <section v-if="packsBySource[cat].length > 0" class="mb-6">
                  <h3 class="mb-3 flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-[color:var(--tx-muted)]">
                    <button
                      type="button"
                      class="flex h-4 w-4 shrink-0 items-center justify-center rounded transition-colors hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]"
                      :title="t('library.toggleCat')"
                      @click="toggleSidebarCat(cat)"
                    >
                      <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current transition-transform" :class="sidebarCat[cat] ? 'rotate-90' : ''"><path d="M6 4l4 4-4 4V4Z"/></svg>
                    </button>
                    {{ t(PACK_CAT_LABELS[cat]) }}
                    <span class="rounded-full bg-[var(--input)] px-1.5 py-0.5 text-[10px] font-bold tabular-nums">{{ packsBySource[cat].length }}</span>
                  </h3>
                  <div v-if="sidebarCat[cat]" class="grid gap-3" :class="libTile.col">
                    <div
                      v-for="p in packsBySource[cat]"
                      :key="p.id"
                      class="flex aspect-square flex-col items-center justify-center gap-2 rounded-md border p-3 text-center transition-colors"
                      :class="packId === p.id
                        ? 'border-[color-mix(in_srgb,var(--accent)_45%,transparent)] bg-[color-mix(in_srgb,var(--accent)_8%,transparent)]'
                        : 'border-[var(--border)] bg-[var(--panel)] hover:border-[color-mix(in_srgb,var(--accent)_35%,transparent)] hover:bg-[var(--input-50)]'"
                      @contextmenu.prevent="openLibMenu($event, p)"
                    >
                      <button
                        type="button"
                        class="flex w-full flex-col items-center justify-center gap-2"
                        :title="p.name"
                        @click="openPackTab(p.id)"
                      >
                        <img
                          v-if="p.icon"
                          :src="convertFileSrc(p.icon)"
                          :alt="p.name"
                          class="shrink-0 aspect-square rounded-none border border-[var(--border)] object-cover"
                          :class="libTile.icon"
                        />
                        <svg v-else viewBox="0 0 16 16" class="shrink-0 rounded-none fill-current text-[var(--tx-muted)]" :class="libTile.icon">
                          <path d="M1 7.775V2.75C1 1.784 1.784 1 2.75 1h5.025c.464 0 .91.184 1.238.513l6.25 6.25a1.75 1.75 0 0 1 0 2.474l-5.026 5.026a1.75 1.75 0 0 1-2.474 0l-6.25-6.25A1.752 1.752 0 0 1 1 7.775Z"/>
                        </svg>
                        <span class="w-full min-w-0 truncate text-xs font-medium" :class="packId === p.id ? 'text-[var(--accent)]' : 'text-[color:var(--tx)]'">{{ p.name }}</span>
                      </button>
                      <button
                        type="button"
                        class="flex w-full items-center justify-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs font-semibold text-white shadow-sm transition-colors"
                        :class="libStatus[p.id]?.installed ? 'bg-[#238636] hover:bg-[#2ea043]' : 'bg-[var(--accent-deep)] hover:bg-[var(--accent-hover)]'"
                        :disabled="busy || gameRunning"
                        @click="playLibraryPack(p)"
                      >
                        <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M4.5 1.94a1 1 0 0 1 1.523-.853l9.6 6.06a1 1 0 0 1 0 1.707l-9.6 6.06A1 1 0 0 1 4.5 14.06V1.94Z"/></svg>
                        {{ libStatus[p.id]?.installed ? t("side.play") : t("side.downloadPlay") }}
                      </button>
                    </div>
                  </div>
                </section>
              </template>
              <div
                v-if="packs.length === 0"
                class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-8 text-center text-xs text-[color:var(--tx-muted)]"
              >
                {{ t("library.empty") }}
              </div>
            </div>

            <!-- Контекстное меню: ПКМ по экземпляру в библиотеке -->
            <div
              v-if="libMenuPack && libMenuPos"
              class="fixed inset-0 z-[70]"
              @mousedown="closeLibMenu"
              @contextmenu.prevent="closeLibMenu"
            >
              <div
                class="fixed z-[71] w-56 overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm py-1 shadow-2xl"
                :style="{ left: `${libMenuPos.x}px`, top: `${libMenuPos.y}px` }"
                @mousedown.stop
                @contextmenu.stop
              >
                <div class="px-3 py-1.5">
                  <div class="truncate text-xs font-semibold text-[color:var(--tx-strong)]">{{ libMenuPack.name }}</div>
                  <div class="truncate font-mono text-[11px] text-[color:var(--tx-muted)]">{{ libMenuPack.id }}</div>
                </div>
                <div class="mx-3 border-t border-[var(--border)]"></div>
                <button
                  type="button"
                  class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs transition-colors hover:bg-[var(--hover)] disabled:opacity-50"
                  :disabled="busy || gameRunning"
                  @click="libDoPlay"
                >
                  <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 shrink-0 fill-current"><path d="M4.5 1.94a1 1 0 0 1 1.523-.853l9.6 6.06a1 1 0 0 1 0 1.707l-9.6 6.06A1 1 0 0 1 4.5 14.06V1.94Z"/></svg>
                  {{ libMenuPack && libStatus[libMenuPack.id]?.installed ? t("side.play") : t("side.downloadPlay") }}
                </button>
                <button
                  type="button"
                  class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                  @click="libOpenSettings"
                >
                  <svg viewBox="0 0 24 24" class="h-3.5 w-3.5 shrink-0 fill-none stroke-current" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1Z"></path></svg>
                  {{ t("nav.settings") }}
                </button>
              </div>
            </div>
          </div>
        </template>
        <template v-else-if="tab === 'author'">
          <div class="min-h-0 flex-1 overflow-y-auto pr-1">
          <div class="space-y-6">
            <div class="border-b border-[var(--border)] pb-3 flex items-start justify-between gap-4">
              <div>
                <h1 class="text-xl font-bold tracking-tight text-[color:var(--tx-strong)]">{{ t("author.title") }}</h1>
                <p class="text-xs text-[color:var(--tx-muted)]">{{ t("author.subtitle") }}</p>
              </div>
              <button
                v-if="monoProfile"
                type="button"
                class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                :disabled="authorBusy"
                @click="pickAuthorImportFile"
              >
                {{ t("author.create") }}
              </button>
            </div>

            <div v-if="!monoProfile" class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-8 text-center text-xs text-[color:var(--tx-muted)]">
              <p>{{ t("author.needLogin") }}</p>
              <button type="button" class="mt-4 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="tab = 'settings'">
                {{ t("nav.settings") }}
              </button>
            </div>

            <template v-else-if="authorDetail">
              <button type="button" class="flex items-center gap-1 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="closeAuthorDetail()">
                <svg viewBox="0 0 16 16" class="h-3 w-3 fill-none stroke-current" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 4 6 8l4 4"/></svg>
                {{ t("author.back") }}
              </button>

              <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
                <div class="flex items-start gap-3 border-b border-[var(--border)] px-4 py-3">
                  <img v-if="authorDetail.icon_url" :src="authorDetail.icon_url" class="h-10 w-10 shrink-0 rounded-md object-cover" />
                  <div v-else class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-[var(--input)] text-sm font-semibold text-[var(--accent)]">
                    {{ authorDetail.name?.[0]?.toUpperCase() }}
                  </div>
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <h2 class="truncate text-sm font-semibold text-[color:var(--tx-strong)]">{{ authorDetail.name }}</h2>
                      <span v-if="authorVersions.length > 0" class="rounded bg-[var(--input)] px-1.5 py-0.5 text-[11px] text-[color:var(--tx-muted)]">{{ authorVersions[0].version }}</span>
                    </div>
                    <p class="truncate text-xs text-[color:var(--tx-muted)]">
                      @{{ authorDetail.author_name ?? t("author.unknown") }} · {{ t("author.rating") }}: {{ authorDetail.likes - authorDetail.dislikes }} ({{ authorDetail.likes }}👍 / {{ authorDetail.dislikes }}👎)
                    </p>
                  </div>
                </div>
              </section>

              <!-- Сабтабы панели автора -->
              <div class="flex gap-1 overflow-x-auto border-b border-[var(--border)]">
                <button v-for="st in (['overview', 'versions', 'news', 'collabs', 'comments'] as const)" :key="st" type="button"
                  class="relative shrink-0 px-3.5 pb-2.5 pt-1 text-xs font-semibold transition-colors"
                  :class="authorTab === st ? 'text-[var(--accent)]' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
                  @click="authorTab = st">
                  {{ st === 'overview' ? t('author.tabOverview') : st === 'versions' ? t('author.versions') : st === 'news' ? t('author.news') : st === 'collabs' ? t('collabs.tab') : t('comments.tab') }}
                  <template v-if="st === 'versions' && authorVersions.length"> ({{ authorVersions.length }})</template>
                  <template v-else-if="st === 'news' && authorNews.length"> ({{ authorNews.length }})</template>
                  <template v-else-if="st === 'collabs' && authorCollaborators.length"> ({{ authorCollaborators.length }})</template>
                  <template v-else-if="st === 'comments' && catalogCommentCount"> ({{ catalogCommentCount }})</template>
                  <span v-if="authorTab === st" class="absolute inset-x-2 bottom-0 h-[2.5px] rounded-t-full bg-[var(--accent)]"></span>
                </button>
              </div>

              <div v-if="authorTab === 'overview'" class="space-y-6">
              <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
                <div class="space-y-3 p-4">
                  <label class="block text-xs text-[color:var(--tx-muted)]">
                    {{ t("author.name") }}
                    <input v-model="authorDetail.name" type="text" class="mt-1 w-full rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none" />
                  </label>
                  <label class="block text-xs text-[color:var(--tx-muted)]">
                    {{ t("author.desc") }}
                    <textarea v-model="authorDetail.description" rows="3" class="mt-1 w-full resize-y rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none"></textarea>
                  </label>
                  <div class="grid grid-cols-2 gap-3">
                    <label class="block text-xs text-[color:var(--tx-muted)]">
                      {{ t("author.minRamMb") }}
                      <input v-model.number="authorDetail.min_ram_mb" type="number" min="0" class="mt-1 w-full rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] focus:border-[var(--accent)] focus:outline-none" />
                    </label>
                    <label class="block text-xs text-[color:var(--tx-muted)]">
                      {{ t("author.iconUrl") }}
                      <input v-model="authorDetail.icon_url" type="text" class="mt-1 w-full rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none" />
                    </label>
                    <label class="col-span-2 block text-xs text-[color:var(--tx-muted)]">
                      {{ t("author.boosty") }}
                      <input v-model="authorDetail.boosty_blog" type="text" class="mt-1 w-full rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none" />
                    </label>
                  </div>
                  <div class="flex gap-2">
                    <button type="button" class="rounded-lg border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50" :disabled="authorBusy" @click="updateAuthorMeta({ name: authorDetail.name, description: authorDetail.description, min_ram_mb: authorDetail.min_ram_mb, boosty_blog: authorDetail.boosty_blog, icon_url: authorDetail.icon_url })">
                      {{ t("author.save") }}
                    </button>
                  </div>
                  <button type="button" class="rounded-md border border-[#b91c1c]/40 bg-[var(--input)] px-3 py-1.5 text-xs font-medium text-[#f87171] hover:bg-[#b91c1c]/20 disabled:opacity-50" @click="deleteAuthorPack()">
                    {{ t("author.delete") }}
                  </button>
                </div>
              </section>

              <!-- Скриншоты: список в meta.screenshots (добавление/удаление по URL) -->
              <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
                <div class="border-b border-[var(--border)] px-4 py-3 text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("pack.screenshots") }}</div>
                <div class="space-y-2 p-4">
                  <div v-if="authorShots.length === 0" class="text-center text-xs text-[color:var(--tx-muted)]">{{ t("author.noShots") }}</div>
                  <div v-for="(s, i) in authorShots" :key="i" class="flex items-center gap-2 rounded-md border border-[var(--border)] bg-[var(--bg)] p-2">
                    <img :src="s" :alt="`Screenshot ${i + 1}`" class="h-10 w-16 shrink-0 rounded object-cover" loading="lazy" />
                    <span class="min-w-0 flex-1 truncate font-mono text-[11px] text-[color:var(--tx-muted)]">{{ s }}</span>
                    <button type="button" class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-xs text-[#f87171] hover:bg-[#b91c1c]/20 disabled:opacity-50"
                      :disabled="authorBusy"
                      @click="removeAuthorShot(i)">
                      {{ t("author.delete") }}
                    </button>
                  </div>
                  <div class="flex items-center gap-2 border-t border-[var(--border)] pt-3">
                    <input v-model="authorShotUrl" type="text" :placeholder="t('author.shotUrlPh')" class="min-w-0 flex-1 rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none" />
                    <button type="button" class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                      :disabled="authorBusy || !authorShotUrl.trim()"
                      @click="addAuthorShot">
                      {{ t("author.addShot") }}
                    </button>
                  </div>
                </div>
              </section>
              </div>

              <div v-else-if="authorTab === 'versions'">
              <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
                <div class="border-b border-[var(--border)] px-4 py-3 text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("author.versions") }}</div>
                <div class="divide-y divide-[var(--border)]">
                  <div v-for="v in authorVersions" :key="v.id" class="flex items-center gap-3 px-4 py-2.5">
                    <div class="min-w-0 flex-1">
                      <p class="truncate text-xs font-medium text-[color:var(--tx)]">{{ v.version }}</p>
                      <p v-if="v.changelog" class="truncate text-xs text-[color:var(--tx-muted)]">{{ v.changelog }}</p>
                    </div>
                    <span class="shrink-0 text-xs text-[color:var(--tx-muted)]">{{ formatDate(v.created_at) }}</span>
                    <button type="button" class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-xs text-[#f87171] hover:bg-[#b91c1c]/20 disabled:opacity-50" :disabled="authorBusy" @click="deleteAuthorVersion(v.id)">
                      {{ t("author.delete") }}
                    </button>
                  </div>
                </div>
                <div class="space-y-2 border-t border-[var(--border)] p-4">
                  <p class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("author.addVersion") }}</p>
                  <div class="flex items-center gap-2">
                    <button type="button" class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="pickAuthorVersionFile">
                      {{ t("author.pickFile") }}
                    </button>
                    <span class="truncate text-xs text-[color:var(--tx-muted)]">{{ authorVersionFile || t("author.noFile") }}</span>
                  </div>
                  <input v-model="authorNewVersion" type="text" :placeholder="t('author.versionTag')" class="w-full rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none" />
                  <textarea v-model="authorNewChangelog" :placeholder="t('author.changelog')" rows="2" class="w-full resize-y rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none"></textarea>
                  <button type="button" class="rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50" :disabled="authorBusy || !authorVersionFile || !authorNewVersion.trim()" @click="createAuthorVersion(authorVersionFile, authorNewVersion.trim(), authorNewChangelog)">
                    {{ t("author.upload") }}
                  </button>
                </div>
              </section>
              </div>

              <div v-else-if="authorTab === 'news'">
              <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
                <div class="border-b border-[var(--border)] px-4 py-3 text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("author.news") }}</div>
                <div class="divide-y divide-[var(--border)]">
                  <div v-for="n in authorNews" :key="n.id" class="px-4 py-2.5">
                    <div class="flex items-center gap-2">
                      <span class="rounded bg-[var(--input)] px-1.5 py-0.5 text-[11px] uppercase text-[color:var(--tx-muted)]">{{ n.kind }}</span>
                      <p class="min-w-0 flex-1 truncate text-xs font-medium text-[color:var(--tx)]">{{ n.title }}</p>
                      <span class="shrink-0 text-xs text-[color:var(--tx-muted)]">{{ formatDate(n.created_at) }}</span>
                      <button type="button" class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-xs text-[#f87171] hover:bg-[#b91c1c]/20" @click="deleteAuthorNews(n.id)">
                        {{ t("author.delete") }}
                      </button>
                    </div>
                    <p v-if="n.body" class="mt-1 line-clamp-2 text-xs text-[color:var(--tx-muted)]">{{ n.body }}</p>
                  </div>
                  <div v-if="authorNews.length === 0" class="px-4 py-6 text-center text-xs text-[color:var(--tx-muted)]">{{ t("author.noNews") }}</div>
                </div>
                <div class="space-y-2 border-t border-[var(--border)] p-4">
                  <p class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("author.addNews") }}</p>
                  <div class="flex items-center gap-2">
                    <select v-model="authorNewsKind" class="rounded-md border border-[var(--border)] bg-[var(--bg)] px-2 py-1.5 text-xs text-[color:var(--tx)] focus:border-[var(--accent)] focus:outline-none">
                      <option value="post">{{ t("author.post") }}</option>
                      <option value="update">{{ t("author.update") }}</option>
                    </select>
                    <input v-model="authorNewTitle" type="text" :placeholder="t('author.titleField')" class="min-w-0 flex-1 rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none" />
                  </div>
                  <textarea v-model="authorNewBody" :placeholder="t('author.body')" rows="2" class="w-full resize-y rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none"></textarea>
                  <button type="button" class="rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50" :disabled="authorBusy || !authorNewTitle.trim()" @click="addAuthorNews(authorNewTitle.trim(), authorNewBody)">
                    {{ t("author.addNews") }}
                  </button>
                </div>
              </section>
              </div>

              <!-- Соавторы: гранулярные права на сборку -->
              <div v-else-if="authorTab === 'collabs'">
              <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
                <div class="border-b border-[var(--border)] px-4 py-3 text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("collabs.tab") }}</div>
                <div class="divide-y divide-[var(--border)]">
                  <div v-for="c in authorCollaborators" :key="c.id" class="flex flex-wrap items-center gap-2 px-4 py-2.5">
                    <button type="button" class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full border border-[var(--border)] bg-[var(--input)] font-mono text-xs font-bold text-[color:var(--tx-strong)] transition-colors hover:text-[var(--accent)]"
                      @click="openProfileView(c.user.id)">
                      {{ c.user.username?.[0]?.toUpperCase() ?? "?" }}
                    </button>
                    <span class="font-mono text-xs font-medium text-[color:var(--tx-strong)]">{{ c.user.displayName || c.user.username }}</span>
                    <div class="ml-auto flex flex-wrap items-center gap-2">
                      <label v-for="pm in (['permEditMeta', 'permManageVersions', 'permManageNews'] as const)" :key="pm" class="flex cursor-pointer items-center gap-1 text-[11px] text-[color:var(--tx-muted)]">
                        <input type="checkbox" class="accent-[var(--accent)]" :checked="c[pm]" :disabled="collabBusy"
                          @change="updateCollaborator(authorSelected!, c.id, { [pm]: !c[pm] } as any)" />
                        {{ t('collabs.' + pm) }}
                      </label>
                      <button type="button" class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-xs text-[#f87171] hover:bg-[#b91c1c]/20 disabled:opacity-50" :disabled="collabBusy" @click="removeCollaborator(authorSelected!, c.id)">
                        {{ t("author.delete") }}
                      </button>
                    </div>
                  </div>
                  <div v-if="authorCollaborators.length === 0" class="px-4 py-6 text-center text-xs text-[color:var(--tx-muted)]">{{ t("collabs.empty") }}</div>
                </div>
                <div class="space-y-2 border-t border-[var(--border)] p-4">
                  <p class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("collabs.add") }}</p>
                  <input v-model="collabName" type="text" :placeholder="t('collabs.usernamePh')" class="w-full rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none" />
                  <div class="flex flex-wrap items-center gap-3">
                    <label v-for="pm in (['permEditMeta', 'permManageVersions', 'permManageNews'] as const)" :key="pm" class="flex cursor-pointer items-center gap-1 text-[11px] text-[color:var(--tx-muted)]">
                      <input v-model="collabPerms[pm]" type="checkbox" class="accent-[var(--accent)]" />
                      {{ t('collabs.' + pm) }}
                    </label>
                    <button type="button" class="ml-auto rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                      :disabled="collabBusy || !collabName.trim()"
                      @click="addCollaborator(authorSelected!, collabName, collabPerms.permEditMeta, collabPerms.permManageVersions, collabPerms.permManageNews); collabName = ''">
                      {{ t("collabs.addBtn") }}
                    </button>
                  </div>
                  <p class="text-[11px] leading-snug text-[color:var(--tx-muted)]">{{ t("collabs.hint") }}</p>
                </div>
              </section>
              </div>

              <!-- Комментарии к сборке (модерация автора) -->
              <div v-else-if="authorTab === 'comments'">
              <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
                <div class="border-b border-[var(--border)] px-4 py-3 text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("comments.tab") }}</div>
                <div class="divide-y divide-[var(--border)]">
                  <div v-if="catalogCommentsBusy && catalogComments.length === 0" class="px-4 py-6 text-center text-xs text-[color:var(--tx-muted)]">{{ t("catalog.loading") }}</div>
                  <div v-if="!catalogCommentsBusy && catalogComments.length === 0" class="px-4 py-6 text-center text-xs text-[color:var(--tx-muted)]">{{ t("comments.empty") }}</div>
                  <div v-for="c in catalogComments" :key="c.id">
                    <div class="flex items-start gap-2 px-4 py-2.5">
                      <button type="button" class="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-full border border-[var(--border)] bg-[var(--input)] font-mono text-[11px] font-bold text-[color:var(--tx-strong)] transition-colors hover:text-[var(--accent)]"
                        @click="openProfileView(c.userId)">
                        {{ c.user.username?.[0]?.toUpperCase() ?? "?" }}
                      </button>
                      <div class="min-w-0 flex-1">
                        <div class="flex items-center gap-2">
                          <span class="font-mono text-xs font-semibold text-[color:var(--tx-strong)]">{{ c.user.displayName || c.user.username }}</span>
                          <span class="text-[11px] text-[color:var(--tx-muted)]">{{ formatDate(c.createdAt) }}</span>
                          <span class="text-[11px] text-[color:var(--tx-muted)]">👍 {{ c.likes }} · 👎 {{ c.dislikes }}</span>
                        </div>
                        <p class="mt-0.5 text-xs leading-relaxed text-[color:var(--tx)] whitespace-pre-wrap">{{ c.body }}</p>
                        <div v-for="r in c.replies" :key="r.id" class="mt-1.5 ml-3 border-l-2 border-[var(--border)] pl-2.5">
                          <div class="flex items-center gap-2">
                            <span class="font-mono text-[11px] font-semibold text-[color:var(--tx)]">{{ r.user.displayName || r.user.username }}</span>
                            <span class="text-[10px] text-[color:var(--tx-muted)]">{{ formatDate(r.createdAt) }} · 👍 {{ r.likes }} · 👎 {{ r.dislikes }}</span>
                          </div>
                          <p class="text-xs leading-relaxed text-[color:var(--tx-muted)] whitespace-pre-wrap">{{ r.body }}</p>
                        </div>
                      </div>
                      <button type="button" class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-[11px] text-[#f87171] hover:bg-[#b91c1c]/20 disabled:opacity-50"
                        :disabled="catalogCommentsBusy"
                        @click="removeCatalogComment(authorSelected!, c.id)">
                        {{ t("author.delete") }}
                      </button>
                    </div>
                  </div>
                </div>
              </section>
              </div>
            </template>

            <div v-else-if="authorPacks.length === 0" class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-8 text-center text-xs text-[color:var(--tx-muted)]">
              {{ t("author.noPacks") }}
            </div>

            <div v-else class="grid grid-cols-1 gap-3 sm:grid-cols-2">
              <div v-for="p in authorPacks" :key="p.id" class="flex items-center gap-3 rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm p-3">
                <img v-if="p.icon_url" :src="p.icon_url" class="h-10 w-10 shrink-0 rounded-md object-cover" />
                <div v-else class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-[var(--input)] text-sm font-semibold text-[var(--accent)]">
                  {{ p.name?.[0]?.toUpperCase() }}
                </div>
                <div class="min-w-0 flex-1">
                  <p class="truncate text-sm font-medium text-[color:var(--tx)]">{{ p.name }}</p>
                  <p class="truncate text-xs text-[color:var(--tx-muted)]">
                    {{ p.version }} · {{ t("author.rating") }}: {{ p.likes - p.dislikes }}
                  </p>
                </div>
                <button type="button" class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="openAuthorDetail(p.id)">
                  {{ t("author.edit") }}
                </button>
              </div>
            </div>
          </div>
          </div>
        </template>
        <!-- ======= Вкладка: Админ-панель ======= -->
        <template v-else-if="tab === 'admin'">
          <div class="min-h-0 flex-1 overflow-y-auto pr-1">
          <div class="space-y-6">
            <div class="flex items-start justify-between gap-4 border-b border-[var(--border)] pb-3">
              <div>
                <h1 class="text-xl font-bold tracking-tight text-[color:var(--tx-strong)]">{{ t("admin.title") }}</h1>
                <p class="text-xs text-[color:var(--tx-muted)]">{{ t("admin.subtitle") }}</p>
              </div>
              <button
                type="button"
                class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                :disabled="adminBusy"
                @click="loadAdminData"
              >
                {{ t("catalog.refresh") }}
              </button>
            </div>

            <!-- Пользователи -->
            <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
              <div class="border-b border-[var(--border)] px-4 py-3 text-xs font-semibold text-[color:var(--tx-strong)]">
                {{ t("admin.users") }} ({{ adminUsers.length }})
              </div>
              <div class="divide-y divide-[var(--border)]">
                <div v-if="adminBusy && adminUsers.length === 0" class="px-4 py-6 text-center text-xs text-[color:var(--tx-muted)]">{{ t("catalog.loading") }}</div>
                <div v-for="u in adminUsers" :key="u.id" class="px-4 py-2.5">
                  <div class="flex flex-wrap items-center gap-2">
                    <button type="button" class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full border border-[var(--border)] bg-[var(--input)] font-mono text-[11px] font-bold text-[color:var(--tx-strong)] transition-colors hover:text-[var(--accent)]"
                      @click="openProfileView(u.id)">
                      {{ u.username?.[0]?.toUpperCase() ?? "?" }}
                    </button>
                    <span class="font-mono text-xs font-medium text-[color:var(--tx-strong)]">{{ u.username }}</span>
                    <span v-if="u.displayName" class="text-xs text-[color:var(--tx-muted)]">{{ u.displayName }}</span>
                    <span class="rounded px-1.5 py-0.5 text-[11px] font-semibold"
                      :class="u.role === 'admin' ? 'bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] text-[var(--accent)]' : 'bg-[var(--input)] text-[color:var(--tx-muted)]'">
                      {{ u.role }}
                    </span>
                    <span v-if="u.banned" class="rounded bg-[#f85149]/15 px-1.5 py-0.5 text-[11px] font-semibold text-[#f85149]" :title="u.banReason || ''">
                      {{ t("admin.banned") }}<template v-if="u.banReason">: {{ u.banReason }}</template>
                    </span>
                    <span v-if="u.email" class="min-w-0 flex-1 truncate text-right text-[11px] text-[color:var(--tx-muted)]">
                      {{ u.email }} <template v-if="!u.emailConfirmed">⚠️</template>
                    </span>
                    <span class="ml-auto shrink-0 text-[11px] text-[color:var(--tx-muted)]">{{ formatDate(u.createdAt) }}</span>
                    <select
                      class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--bg)] px-1.5 py-1 text-[11px] text-[color:var(--tx)] focus:border-[var(--accent)] focus:outline-none disabled:opacity-50"
                      :value="u.role"
                      :disabled="adminBusy || u.id === monoProfile?.uuid"
                      @change="adminSetRole(u.id, ($event.target as HTMLSelectElement).value)"
                    >
                      <option value="user">user</option>
                      <option value="admin">admin</option>
                    </select>
                    <button v-if="!u.banned" type="button" class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-xs font-medium text-[#f87171] hover:bg-[#b91c1c]/20 disabled:opacity-50"
                      :disabled="adminBusy || u.id === monoProfile?.uuid"
                      @click="adminBanArmed = adminBanArmed === u.id ? null : u.id; adminBanReason = ''">
                      {{ t("admin.ban") }}
                    </button>
                    <button v-else type="button" class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-xs font-medium text-[#3fb950] hover:bg-[#238636]/20 disabled:opacity-50"
                      :disabled="adminBusy"
                      @click="adminUnbanUser(u.id)">
                      {{ t("admin.unban") }}
                    </button>
                    <button type="button" class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-xs text-[#f87171] hover:bg-[#b91c1c]/20 disabled:opacity-50"
                      :disabled="adminBusy || u.id === monoProfile?.uuid"
                      @click="adminDeleteUser(u.id)">
                      {{ t("author.delete") }}
                    </button>
                  </div>
                  <!-- Причина бана (inline) -->
                  <div v-if="adminBanArmed === u.id" class="mt-2 flex items-center gap-2 rounded-md border border-[#b91c1c]/40 bg-[var(--bg)] p-2">
                    <input v-model="adminBanReason" type="text" :placeholder="t('admin.banReasonPh')" class="min-w-0 flex-1 rounded-md border border-[var(--border)] bg-[var(--bg)] px-2 py-1 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none" />
                    <button type="button" class="shrink-0 rounded-md border border-[#b91c1c]/40 bg-[#b91c1c]/15 px-3 py-1.5 text-xs font-semibold text-[#f87171] hover:bg-[#b91c1c]/25 disabled:opacity-50"
                      :disabled="adminBusy"
                      @click="adminBanUser(u.id, adminBanReason); adminBanArmed = null">
                      {{ t("admin.banConfirm") }}
                    </button>
                    <button type="button" class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1 text-xs text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="adminBanArmed = null">
                      {{ t("author.cancel") }}
                    </button>
                  </div>
                </div>
              </div>
            </section>

            <!-- Сборки -->
            <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
              <div class="border-b border-[var(--border)] px-4 py-3 text-xs font-semibold text-[color:var(--tx-strong)]">
                {{ t("admin.packs") }} ({{ adminPacks.length }})
              </div>
              <div class="divide-y divide-[var(--border)]">
                <div v-if="adminBusy && adminPacks.length === 0" class="px-4 py-6 text-center text-xs text-[color:var(--tx-muted)]">{{ t("catalog.loading") }}</div>
                <div v-for="p in adminPacks" :key="p.id" class="flex flex-wrap items-center gap-2 px-4 py-2.5">
                  <span class="min-w-0 flex-1 truncate text-xs font-medium text-[color:var(--tx-strong)]">{{ p.name }}</span>
                  <span class="shrink-0 font-mono text-[11px] text-[color:var(--tx-muted)]">@{{ p.authorName ?? "?" }}</span>
                  <span class="shrink-0 text-[11px] text-[color:var(--tx-muted)]">👍 {{ p.likes }} / 👎 {{ p.dislikes }} · {{ t("author.versions") }}: {{ p.versionsCount }}</span>
                  <span class="shrink-0 text-[11px] text-[color:var(--tx-muted)]">{{ formatDate(p.createdAt) }}</span>
                  <button type="button" class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-xs text-[#f87171] hover:bg-[#b91c1c]/20 disabled:opacity-50"
                    :disabled="adminBusy"
                    @click="adminDeletePack(p.id)">
                    {{ t("author.delete") }}
                  </button>
                </div>
              </div>
            </section>
          </div>
          </div>
        </template>
        <template v-else>
          <div class="min-h-0 flex-1 overflow-y-auto pr-1">
          <div class="space-y-6">
            <div class="border-b border-[var(--border)] pb-3">
              <h1 class="text-xl font-bold tracking-tight text-[color:var(--tx-strong)]">{{ t("settings.title") }}</h1>
              <p class="text-xs text-[color:var(--tx-muted)]">{{ t("settings.subtitle") }}</p>
            </div>

            <!-- Аккаунты: профиль Mono + игровые аккаунты (две колонки) -->
            <div class="grid gap-4 lg:grid-cols-2">
              <!-- Профиль Mono -->
              <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
                <div class="border-b border-[var(--border)] px-4 py-3 flex items-center gap-2.5">
                  <span class="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg bg-[var(--accent)]">
                    <svg viewBox="0 0 24 24" class="h-4 w-4 fill-[color:var(--panel)]"><path d="M3 8.4 8.4 3h7.2L21 8.4v7.2L15.6 21H8.4L3 15.6V8.4Zm2 1.3v4.6L8.3 19H9.7l2.5-6.2L14.7 19h1.4L19 14.3V9.7L15.7 5H9.9L5 9.7Z"/></svg>
                  </span>
                  <div class="min-w-0">
                    <h3 class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("settings.monoTitle") }}</h3>
                    <p class="text-[11px] leading-tight text-[color:var(--tx-muted)]">{{ t("settings.monoNote") }}</p>
                  </div>
                </div>

                <div class="p-4 space-y-3">
                  <template v-if="monoProfile">
                    <div class="flex items-center gap-3">
                      <span class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full border border-[color-mix(in_srgb,var(--accent)_50%,transparent)] bg-[var(--input)] font-mono text-sm font-bold text-[var(--accent)]">
                        {{ monoProfile.username?.[0]?.toUpperCase() ?? "M" }}
                      </span>
                      <p class="flex min-w-0 items-center gap-1.5 truncate text-sm font-semibold text-[color:var(--tx-strong)]">
                        {{ monoProfile.username }}
                        <svg class="h-4 w-4 shrink-0 fill-[#3fb950]" viewBox="0 0 16 16">
                          <path d="M8 15A7 7 0 1 1 8 1a7 7 0 0 1 0 14Zm-1.31-4.21 4.55-4.55-1.06-1.06-3.49 3.49-1.42-1.42-1.06 1.06 2.48 2.48Z"/>
                        </svg>
                      </p>
                    </div>
                    <div class="flex flex-wrap gap-2">
                      <button
                        type="button"
                        class="flex-1 rounded-lg border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                        :disabled="profileBusy"
                        @click="openProfileView(monoProfile.uuid)"
                      >
                        {{ t("profile.my") }}
                      </button>
                      <button
                        type="button"
                        class="flex-1 rounded-lg border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                        :disabled="monoAuthBusy"
                        :title="t('auth2.confirmHint')"
                        @click="handleMonoConfirmEmail"
                      >
                        {{ t("auth2.confirm") }}
                      </button>
                    </div>
                    <button
                      type="button"
                      class="w-full rounded-lg border border-[color-mix(in_srgb,var(--accent)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] py-2 text-xs font-semibold text-[var(--accent)] hover:bg-[color-mix(in_srgb,var(--accent)_25%,transparent)] disabled:opacity-50"
                      :disabled="busy || monoBusy"
                      @click="handleMonoLogout"
                    >
                      {{ monoBusy ? t("settings.monoWait") : t("accounts.signOut") }}
                    </button>
                  </template>

                  <template v-else>
                    <input
                      v-model="monoName"
                      :placeholder="t('settings.monoUsername')"
                      class="w-full rounded-lg border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none"
                    />
                    <input
                      v-model="monoPass"
                      type="password"
                      :placeholder="t('settings.monoPassword')"
                      @keydown.enter="handleMonoLogin"
                      class="w-full rounded-lg border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none"
                    />
                    <div class="flex gap-2">
                      <button
                        type="button"
                        class="flex-1 rounded-lg border border-[color-mix(in_srgb,var(--accent)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] py-2 text-xs font-semibold text-[var(--accent)] hover:bg-[color-mix(in_srgb,var(--accent)_25%,transparent)] disabled:opacity-50"
                        :disabled="busy || monoBusy"
                        @click="handleMonoLogin"
                      >
                        {{ monoBusy ? t("settings.monoWait") : t("settings.monoSignIn") }}
                      </button>
                      <button
                        type="button"
                        class="flex-1 rounded-lg border border-[var(--border)] bg-[var(--input)] py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                        :disabled="busy || monoBusy"
                        @click="handleMonoRegister"
                      >
                        {{ t("settings.monoRegister") }}
                      </button>
                    </div>
                    <button
                      type="button"
                      class="w-full text-center text-[11px] font-medium text-[var(--accent)] hover:underline"
                      @click="monoForgotOpen = !monoForgotOpen"
                    >
                      {{ t("auth2.forgot") }}
                    </button>

                    <!-- Восстановление пароля: письмо + сброс по токену из письма -->
                    <div v-if="monoForgotOpen" class="space-y-2 rounded-lg border border-[var(--border)] bg-[var(--bg)] p-3">
                      <div class="flex items-center gap-2">
                        <input
                          v-model="monoForgotEmail"
                          type="email"
                          :placeholder="t('auth2.emailPh')"
                          class="min-w-0 flex-1 rounded-lg border border-[var(--border)] bg-[var(--bg)] px-2.5 py-1.5 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none"
                          @keydown.enter="handleMonoForgot"
                        />
                        <button
                          type="button"
                          class="shrink-0 rounded-lg border border-[var(--border)] bg-[var(--input)] px-2.5 py-1.5 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                          :disabled="monoAuthBusy"
                          @click="handleMonoForgot"
                        >
                          {{ t("auth2.send") }}
                        </button>
                      </div>
                      <p v-if="monoForgotSent" class="text-[11px] leading-snug text-[#3fb950]">{{ t("auth2.forgotSent") }}</p>
                      <div class="space-y-1.5 border-t border-[var(--border)] pt-2">
                        <p class="text-[11px] text-[color:var(--tx-muted)]">{{ t("auth2.resetHint") }}</p>
                        <input
                          v-model="monoResetToken"
                          type="text"
                          :placeholder="t('auth2.tokenPh')"
                          class="w-full rounded-lg border border-[var(--border)] bg-[var(--bg)] px-2.5 py-1.5 font-mono text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none"
                        />
                        <input
                          v-model="monoResetPass"
                          type="password"
                          :placeholder="t('auth2.newPassPh')"
                          class="w-full rounded-lg border border-[var(--border)] bg-[var(--bg)] px-2.5 py-1.5 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none"
                          @keydown.enter="handleMonoReset"
                        />
                        <button
                          type="button"
                          class="w-full rounded-lg border border-[color-mix(in_srgb,var(--accent)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] py-1.5 text-xs font-semibold text-[var(--accent)] hover:bg-[color-mix(in_srgb,var(--accent)_25%,transparent)] disabled:opacity-50"
                          :disabled="monoAuthBusy"
                          @click="handleMonoReset"
                        >
                          {{ t("auth2.resetBtn") }}
                        </button>
                        <p v-if="monoResetDone" class="text-[11px] leading-snug text-[#3fb950]">{{ t("auth2.resetDone") }}</p>
                      </div>
                    </div>
                  </template>
                </div>
              </section>

              <!-- Игровые аккаунты -->
              <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
                <div class="border-b border-[var(--border)] px-4 py-3">
                  <h3 class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("settings.gameAccounts") }}</h3>
                </div>
                <div class="p-4 space-y-3">
                  <div class="flex gap-2">
                    <input
                      v-model="username"
                      :placeholder="t('settings.nickname')"
                      class="flex-1 rounded-lg border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none"
                    />
                    <button
                      type="button"
                      class="rounded-lg border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                      :disabled="busy"
                      @click="handleOffline"
                    >
                      {{ t("settings.save") }}
                    </button>
                  </div>

                  <div class="grid grid-cols-2 gap-2">
                    <button
                      type="button"
                      class="rounded-lg border border-[var(--border)] bg-[var(--input)] py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                      :disabled="busy || msPolling || elyPolling"
                      @click="handleMicrosoft"
                    >
                      {{ msPolling ? t("settings.msWait") : t("settings.msSignin") }}
                    </button>
                    <button
                      type="button"
                      class="rounded-lg border border-[color-mix(in_srgb,var(--accent)_30%,transparent)] bg-[color-mix(in_srgb,var(--accent)_8%,transparent)] py-2 text-xs font-medium text-[var(--accent)] hover:bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] disabled:opacity-50"
                      :disabled="busy || msPolling || elyPolling"
                      @click="handleEly"
                    >
                      {{ elyPolling ? t("settings.elyWait") : t("settings.elySignin") }}
                    </button>
                  </div>

                <!-- Device code flow: показать код и ссылку -->
                <div
                  v-if="deviceFlow"
                  class="rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] bg-[var(--bg-60)] p-3 space-y-2"
                >
                  <p class="text-xs text-[color:var(--tx-muted)]">
                    {{ msFlow ? t("settings.msCode") : t("settings.elyCode") }}
                  </p>
                  <div class="flex items-center gap-3">
                    <div
                      v-if="deviceFlow.qr_svg"
                      class="h-28 w-28 shrink-0 overflow-hidden rounded-md border border-[var(--border)] bg-white"
                      :title="t('settings.msScan')"
                    >
                      <div class="h-full w-full" v-html="sanitizeSvg(deviceFlow.qr_svg ?? '')"></div>
                    </div>
                    <div class="min-w-0 flex-1">
                    <p class="font-mono text-2xl font-bold tracking-[0.3em] text-[var(--accent-strong)] select-text">
                      {{ deviceFlow.user_code }}
                    </p>
                    <button
                      type="button"
                      class="mt-2 rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)]"
                      @click="openMsAuthPage"
                    >
                      {{ t("settings.msOpen", { uri: deviceFlow.verification_uri.replace(/^https?:\/\//, "") }) }}
                    </button>
                    </div>
                  </div>
                  <p v-if="msPolling || elyPolling" class="flex items-center gap-2 text-xs text-[color:var(--tx-muted)]">
                    <svg class="h-3 w-3 animate-spin fill-[var(--accent)]" viewBox="0 0 16 16">
                      <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                    </svg>
                    {{ t("settings.msBrowser") }}
                  </p>
                </div>

                <!-- Список сохранённых аккаунтов -->
                <div v-if="accounts.list.length" class="space-y-1.5 border-t border-[var(--border)] pt-3">
                  <div
                    v-for="a in accounts.list"
                    :key="a.id"
                    class="flex items-center gap-2 rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2"
                    :class="a.id === accounts.active ? 'border-[color-mix(in_srgb,var(--accent)_50%,transparent)]' : ''"
                  >
                    <div
                      class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full border border-[var(--border)] bg-[var(--input)] font-mono text-xs font-bold text-[color:var(--tx-strong)]"
                    >
                      {{ a.username[0]?.toUpperCase() ?? "?" }}
                    </div>
                    <div class="min-w-0 flex-1">
                      <p class="truncate text-xs font-medium text-[color:var(--tx-strong)]">{{ a.username }}</p>
                      <p class="text-[11px] text-[color:var(--tx-muted)]">
                        {{ a.user_type === "microsoft" ? t("accounts.ms") : a.user_type === "ely" ? t("accounts.ely") : t("accounts.offline") }}
                      </p>
                    </div>
                    <button
                      v-if="a.id !== accounts.active"
                      type="button"
                      class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-[11px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] disabled:opacity-50"
                      :disabled="accountBusy"
                      @click="handleSwitchAccount(a.id)"
                    >
                      {{ t("accounts.use") }}
                    </button>
                    <span
                      v-else
                      class="shrink-0 text-[11px] font-semibold text-[#3fb950]"
                    >
                      {{ t("accounts.active") }}
                    </span>
                    <button
                      type="button"
                      class="shrink-0 rounded-md border border-[#f85149]/30 bg-[#f85149]/10 p-1 text-[#f85149] transition-colors hover:bg-[#f85149]/20 disabled:opacity-50"
                      :title="t('accounts.removeTitle')"
                      :disabled="accountBusy"
                      @click="handleRemoveAccount(a.id)"
                    >
                      <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                        <path d="M4.75 1.5h6.5a.75.75 0 0 1 .75.75V3.5h2.5a.75.75 0 0 1 0 1.5h-.75v9A1.75 1.75 0 0 1 12 15.75H4A1.75 1.75 0 0 1 2.25 14V5H1.5a.75.75 0 0 1 0-1.5H4V2.25a.75.75 0 0 1 .75-.75Zm.75 5.75a.75.75 0 0 1 1.5 0v4.5a.75.75 0 0 1-1.5 0Zm3.5 0a.75.75 0 0 1 1.5 0v4.5a.75.75 0 0 1-1.5 0Z"/>
                      </svg>
                    </button>
                  </div>
                </div>
                </div>
              </section>
            </div>

            <!-- Boosty: платные сборки -->
            <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
              <div class="border-b border-[var(--border)] px-4 py-3 flex items-center gap-2">
                <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-[var(--accent)]" preserveAspectRatio="none">
                  <path d="M8 1C3.9 1 .7 4.3.7 8.4h3.1L1.6 15l7.2-7.2H6.3C6.3 5.3 7.2 2.9 9.6 2.4 11.9 2 13.7 3.6 13.7 5.8c0 .4-.1.9-.1 1.3.9.5 1.5 1.4 1.7 2.5.1-.6.2-1.2.2-1.8 0-3.8-3.2-6.8-7.5-6.8Z" transform="translate(0 -1)"/>
                </svg>
                <h3 class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("settings.boosty") }}</h3>
              </div>
              <div class="space-y-3 p-4">
                  <p class="text-xs leading-snug text-[color:var(--tx-muted)]">{{ t("settings.boostyNote") }}</p>
                  <!-- Глобальный аккаунт Boosty: работает даже без платных сборок -->
                  <div class="rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2">
                    <div class="flex items-center gap-2">
                      <div class="min-w-0 flex-1">
                        <p class="truncate text-xs font-medium text-[color:var(--tx-strong)]">{{ t("settings.boostyGlobal") }}</p>
                        <p class="truncate text-[11px] text-[color:var(--tx-muted)]">{{ t("settings.boostyGlobalNote") }}</p>
                      </div>
                      <span
                        v-if="boostyGlobalLinkedState"
                        class="shrink-0 rounded-full border border-[#3fb950]/30 bg-[#3fb950]/10 px-2 py-0.5 text-[11px] font-semibold text-[#3fb950]"
                      >
                        {{ t("settings.boostyOk") }}
                      </span>
                      <span
                        v-else
                        class="shrink-0 rounded-full border border-[var(--border)] bg-[var(--input)] px-2 py-0.5 text-[11px] font-semibold text-[color:var(--tx-muted)]"
                      >
                        {{ t("settings.boostyNo") }}
                      </span>
                    </div>
                    <template v-if="boostyGlobalLinkedState">
                      <button
                        type="button"
                        class="mt-1.5 w-full rounded-md border border-[#f85149]/30 bg-[#f85149]/10 py-1 text-[11px] font-medium text-[#f85149] transition-colors hover:bg-[#f85149]/20 disabled:opacity-50"
                        :disabled="licenseBusy"
                        @click="unlinkBoostyGlobal"
                      >
                        {{ t("accounts.signOut") }}
                      </button>
                    </template>
                    <div v-else class="mt-2 space-y-1.5">
                      <button
                        type="button"
                        class="flex w-full items-center justify-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_45%,transparent)] bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_22%,transparent)] disabled:opacity-50"
                        :disabled="licenseBusy || boostyAuthOpen"
                        @click="startBoostyGlobalLogin"
                      >
                        <svg v-if="boostyAuthOpen && boostyGlobalOpen" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
                        {{ boostyAuthOpen && boostyGlobalOpen ? t("license.waiting") : t("license.oauth") }}
                      </button>
                      <div v-if="boostyAuthOpen && boostyGlobalOpen" class="flex justify-center">
                        <button
                          type="button"
                          class="text-xs font-medium text-[color:var(--tx-muted)] hover:text-[color:var(--tx)]"
                          @click="cancelBoostyLogin"
                        >
                          {{ t("license.cancel") }}
                        </button>
                      </div>
                      <div class="flex gap-1.5">
                        <input
                          v-model="licenseKeyInput"
                          type="password"
                          :placeholder="t('license.placeholder')"
                          class="min-w-0 flex-1 rounded-md border border-[var(--border)] bg-[var(--bg)] px-2 py-1.5 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none"
                          @keydown.enter="saveBoostyGlobal(licenseKeyInput)"
                        />
                        <button
                          type="button"
                          class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1.5 text-[11px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                          :disabled="licenseBusy"
                          @click="saveBoostyGlobal(licenseKeyInput)"
                        >
                          {{ t("license.activate") }}
                        </button>
                      </div>
                    </div>
                  </div>
                  <div
                    v-for="p in paidPacks"
                    :key="p.id"
                    class="rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2"
                  >
                    <div class="flex items-center gap-2">
                      <div class="min-w-0 flex-1">
                        <p class="truncate text-xs font-medium text-[color:var(--tx-strong)]">{{ p.name }}</p>
                        <p class="truncate text-[11px] text-[color:var(--tx-muted)]">
                          boosty.to/{{ p.boostyBlog }}
                        </p>
                      </div>
                      <span
                        v-if="licenseByPack[p.id]?.subscribed"
                        class="shrink-0 rounded-full border border-[#3fb950]/30 bg-[#3fb950]/10 px-2 py-0.5 text-[11px] font-semibold text-[#3fb950]"
                      >
                        {{ t("settings.boostyOk") }}
                      </span>
                      <span
                        v-else-if="licenseByPack[p.id] && !licenseByPack[p.id]?.subscribed"
                        class="shrink-0 rounded-full border border-[var(--border)] bg-[var(--input)] px-2 py-0.5 text-[11px] font-semibold text-[color:var(--tx-muted)]"
                      >
                        {{ t("settings.boostyNo") }}
                      </span>
                    </div>
                    <template v-if="licenseByPack[p.id]?.subscribed">
                      <p v-if="licenseByPack[p.id]?.tier" class="mt-1 truncate text-[11px] text-[color:var(--tx-muted)]">
                        {{ t("license.tierList", { list: licenseByPack[p.id]?.tier ?? "" }) }}
                      </p>
                      <p v-if="licenseByPack[p.id]?.expiresAt" class="mt-0.5 text-[11px] text-[color:var(--tx-muted)]">
                        {{ t("license.active", { blog: p.boostyBlog ?? "", until: formatUnixDate(licenseByPack[p.id]?.expiresAt ?? 0) }) }}
                      </p>
                      <button
                        type="button"
                        class="mt-1.5 w-full rounded-md border border-[#f85149]/30 bg-[#f85149]/10 py-1 text-[11px] font-medium text-[#f85149] transition-colors hover:bg-[#f85149]/20 disabled:opacity-50"
                        :disabled="licenseBusyFor === p.id"
                        @click="removeLicenseFor(p.id)"
                      >
                        {{ t("accounts.signOut") }}
                      </button>
                    </template>
                    <div v-else class="mt-2 space-y-1.5">
                      <button
                        type="button"
                        class="flex w-full items-center justify-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_45%,transparent)] bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_22%,transparent)] disabled:opacity-50"
                        :disabled="licenseBusyFor === p.id || boostyAuthOpen"
                        @click="startBoostyLogin(p.id)"
                      >
                        <svg v-if="boostyAuthOpen && boostyTargetPack === p.id" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
                        {{ boostyAuthOpen && boostyTargetPack === p.id ? t("license.waiting") : t("license.oauth") }}
                      </button>
                      <div v-if="boostyAuthOpen && boostyTargetPack === p.id" class="flex justify-center">
                        <button
                          type="button"
                          class="text-xs font-medium text-[color:var(--tx-muted)] hover:text-[color:var(--tx)]"
                          @click="cancelBoostyLogin"
                        >
                          {{ t("license.cancel") }}
                        </button>
                      </div>
                      <div class="flex gap-1.5">
                        <input
                          v-model="licenseKeyInput"
                          type="password"
                          :placeholder="t('license.placeholder')"
                          class="min-w-0 flex-1 rounded-md border border-[var(--border)] bg-[var(--bg)] px-2 py-1.5 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none"
                          @keydown.enter="saveLicenseFor(p.id, licenseKeyInput)"
                        />
                        <button
                          type="button"
                          class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1.5 text-[11px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                          :disabled="licenseBusyFor === p.id"
                          @click="saveLicenseFor(p.id, licenseKeyInput)"
                        >
                          {{ t("license.activate") }}
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
              </section>

            <!-- Скин -->
            <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
              <div class="border-b border-[var(--border)] px-4 py-3">
                <h3 class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("skin.title") }}</h3>
              </div>
              <div class="p-4 space-y-3">
                <div class="flex items-center gap-3">
                  <div class="flex h-14 w-14 shrink-0 items-center justify-center overflow-hidden rounded-md border border-[var(--border)] bg-[var(--input)] font-mono text-sm font-bold text-[color:var(--tx-strong)]">
                    <img
                      v-if="localSkin?.has_skin"
                      :src="localSkin.path ? convertFileSrc(localSkin.path) : ''"
                      :alt="t('skin.title')"
                      class="h-full w-full object-cover"
                    />
                    <template v-else>{{ session?.username?.[0]?.toUpperCase() ?? "?" }}</template>
                  </div>
                  <div class="min-w-0 flex-1 space-y-1.5">
                    <select
                      v-model="skinModel"
                      class="w-full appearance-none rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-1.5 pr-8 text-xs text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] focus:border-[var(--accent)] focus:outline-none"
                      :disabled="skinBusy"
                    >
                      <option value="classic">{{ t("skin.modelClassic") }}</option>
                      <option value="slim">{{ t("skin.modelSlim") }}</option>
                    </select>
                    <p class="text-[11px] leading-relaxed text-[color:var(--tx-muted)]">
                      {{ t("skin.note") }}
                    </p>
                  </div>
                </div>
                <div class="flex gap-2">
                  <button
                    type="button"
                    class="flex-1 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                    :disabled="skinBusy"
                    @click="pickSkinFile"
                  >
                    {{ skinBusy ? t("skin.busy") : t("skin.pick") }}
                  </button>
                  <button
                    v-if="localSkin?.has_skin"
                    type="button"
                    class="rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                    :disabled="skinBusy"
                    @click="removeLocalSkin"
                  >
                    {{ t("skin.remove") }}
                  </button>
                </div>
                <div class="rounded-md border border-[var(--border)] bg-[var(--bg-60)] p-3 space-y-1.5">
                  <p class="text-xs text-[color:var(--tx-muted)]">{{ t("skin.apiHint") }}</p>
                  <div class="flex items-center gap-2">
                    <code class="min-w-0 flex-1 truncate rounded border border-[var(--border)] bg-[var(--input)] px-2 py-1 font-mono text-[11px] text-[color:var(--tx)] select-all">{{ skinApi || "…" }}</code>
                    <button
                      type="button"
                      class="rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-[11px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                      :disabled="!skinApi"
                      @click="copySkinApi"
                    >
                      {{ t("skin.copy") }}
                    </button>
                  </div>
                </div>
              </div>
            </section>

            <!-- Тема -->
            <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
              <div class="flex items-center justify-between border-b border-[var(--border)] px-4 py-3">
                <h3 class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("settings.theme") }}</h3>
                <span class="text-[11px] font-medium text-[color:var(--tx-muted)]">
                  {{ themeLevel >= 0.5 ? t("theme.dark") : t("theme.light") }}
                </span>
              </div>
              <div class="p-4 space-y-3">
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.01"
                  :value="themeLevel"
                  :disabled="packThemeActive"
                  class="w-full accent-[var(--accent-deep)] bg-[var(--input)] h-1.5 rounded-lg appearance-none cursor-pointer disabled:opacity-50"
                  @input="setThemeLevel(Number(($event.target as HTMLInputElement).value))"
                />
                <button
                  type="button"
                  class="w-full rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                  :disabled="packThemeActive"
                  @click="toggleTheme"
                >
                  {{ t("settings.themeToggle") }}
                </button>
                <p v-if="packThemeActive" class="text-xs text-[var(--accent)]">
                  {{ t("theme.disabled") }}
                </p>
              </div>
            </section>

            <!-- Язык интерфейса -->
            <section class="rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm overflow-hidden">
              <div class="border-b border-[var(--border)] px-4 py-3">
                <h3 class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ t("settings.language") }}</h3>
              </div>
              <div class="p-4 space-y-3">
                <select
                  :value="locale"
                  class="w-full appearance-none rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-1.5 pr-8 text-xs text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] focus:border-[var(--accent)] focus:outline-none"
                  @change="setLocale(($event.target as HTMLSelectElement).value)"
                >
                  <option v-for="l in locales" :key="l" :value="l">{{ localeLabel(l) }}</option>
                </select>
                <p class="flex items-center gap-1 text-xs text-[color:var(--tx-muted)]">
                  <span>{{ t("lang.byAuthor") }}</span>
                  <span class="font-medium text-[color:var(--tx)]">{{ activeLocaleAuthor || "—" }}</span>
                  <template v-if="activeLocaleVersion">
                    <span>·</span>
                    <span>{{ t("lang.launcherVer") }} {{ activeLocaleVersion }}</span>
                  </template>
                </p>
              </div>
            </section>
          </div>
          </div>
        </template>
        </div>
        </div>
        </div>
    </main>
    </div>

    <!-- Модалка: скачать сборку с Modrinth -->
    <div
      v-if="modPackOpen"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6"
      @click.self="modPackOpen = false; modPackVersions = null; modPackDetail = null"
    >
      <div class="flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-2xl">
        <div class="flex shrink-0 items-center justify-between border-b border-[var(--border)] px-4 py-3">
          <h3 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ t("mods.packsTitle") }}</h3>
          <button
            type="button"
            class="rounded-md p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
            @click="modPackOpen = false; modPackVersions = null; modPackDetail = null"
          >
            <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
          </button>
        </div>
        <div class="flex shrink-0 items-center gap-2 border-b border-[var(--border)] px-4 py-3">
          <div class="flex shrink-0 items-center gap-1 rounded-md border border-[var(--border)] bg-[var(--bg)] p-0.5">
            <button
              type="button"
              class="flex items-center gap-1.5 rounded px-3 py-1.5 text-xs font-semibold transition-colors"
              :class="modPackService === 'modrinth'
                ? 'bg-[var(--accent)] ring-2 ring-[var(--accent)] ring-offset-2 ring-offset-[var(--panel)]'
                : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
              @click="switchPackService('modrinth')"
            >
              <svg viewBox="0 0 24 24" class="h-3.5 w-3.5 shrink-0" :title="t('mods.serviceModrinth')"><path fill="#00AF5C" d="M12.252.004a11.78 11.768 0 0 0-8.92 3.73 11 10.999 0 0 0-2.17 3.11 11.37 11.359 0 0 0-1.16 5.169c0 1.42.17 2.5.6 3.77.24.759.77 1.899 1.17 2.529a12.3 12.298 0 0 0 8.85 5.639c.44.05 2.54.07 2.76.02.2-.04.22.1-.26-1.7l-.36-1.37-1.01-.06a8.5 8.489 0 0 1-5.18-1.8 5.34 5.34 0 0 1-1.3-1.26c0-.05.34-.28.74-.5a37.572 37.545 0 0 1 2.88-1.629c.03 0 .5.45 1.06.98l1 .97 2.07-.43 2.06-.43 1.47-1.47c.8-.8 1.48-1.5 1.48-1.52 0-.09-.42-1.63-.46-1.7-.04-.06-.2-.03-1.02.18-.53.13-1.2.3-1.45.4l-.48.15-.53.53-.53.53-.93.1-.93.07-.52-.5a2.7 2.7 0 0 1-.96-1.7l-.13-.6.43-.57c.68-.9.68-.9 1.46-1.1.4-.1.65-.2.83-.33.13-.099.65-.579 1.14-1.069l.9-.9-.7-.7-.7-.7-1.95.54c-1.07.3-1.96.53-1.97.53-.03 0-2.23 2.48-2.63 2.97l-.29.35.28 1.03c.16.56.3 1.16.31 1.34l.03.3-.34.23c-.37.23-2.22 1.3-2.84 1.63-.36.2-.37.2-.44.1-.08-.1-.23-.6-.32-1.03-.18-.86-.17-2.75.02-3.73a8.84 8.839 0 0 1 7.9-6.93c.43-.03.77-.08.78-.1.06-.17.5-2.999.47-3.039-.01-.02-.1-.02-.2-.03Zm3.68.67c-.2 0-.3.1-.37.38-.06.23-.46 2.42-.46 2.52 0 .04.1.11.22.16a8.51 8.499 0 0 1 2.99 2 8.38 8.379 0 0 1 2.16 3.449 6.9 6.9 0 0 1 .4 2.8c0 1.07 0 1.27-.1 1.73a9.37 9.369 0 0 1-1.76 3.769c-.32.4-.98 1.06-1.37 1.38-.38.32-1.54 1.1-1.7 1.14-.1.03-.1.06-.07.26.03.18.64 2.56.7 2.78l.06.06a12.07 12.058 0 0 0 7.27-9.4c.13-.77.13-2.58 0-3.4a11.96 11.948 0 0 0-5.73-8.578c-.7-.42-2.05-1.06-2.25-1.06Z"/></svg>
              {{ t("mods.serviceModrinth") }}
            </button>
            <button
              type="button"
              class="flex items-center gap-1.5 rounded px-3 py-1.5 text-xs font-semibold transition-colors"
              :class="modPackService === 'curseforge'
                ? 'bg-[var(--accent)] ring-2 ring-[var(--accent)] ring-offset-2 ring-offset-[var(--panel)]'
                : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
              @click="switchPackService('curseforge')"
            >
              <svg viewBox="0 0 24 24" class="h-3.5 w-3.5 shrink-0" :title="t('mods.serviceCurseforge')"><path fill="#F16436" d="M18.326 9.2145S23.2261 8.4418 24 6.1882h-7.5066V4.4H0l2.0318 2.3576V9.173s5.1267-.2665 7.1098 1.2372c2.7146 2.516-3.053 5.917-3.053 5.917L5.0995 19.6c1.5465-1.4726 4.494-3.3775 9.8983-3.2857-2.0565.65-4.1245 1.6651-5.7344 3.2857h10.9248l-1.0288-3.2726s-7.918-4.6688-.8336-7.1127z"/></svg>
              {{ t("mods.serviceCurseforge") }}
            </button>
          </div>
          <div class="relative min-w-0 flex-1">
            <svg viewBox="0 0 16 16" class="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 fill-[var(--tx-muted)]">
              <path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"/>
            </svg>
            <input
              v-model="modPackQuery"
              type="text"
              :placeholder="modPackService === 'modrinth' ? t('mods.packsPlaceholder') : t('curse.packsPlaceholder')"
              class="w-full rounded-md border border-[var(--border)] bg-[var(--bg)] py-1.5 pl-8 pr-3 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] outline-none transition-colors focus:border-[var(--accent)]"
              @keydown.enter="searchPacksOrCurse"
            />
          </div>
          <button
            type="button"
            class="flex shrink-0 items-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
            :disabled="modPackLoading || cpLoading || !modPackQuery.trim()"
            @click="searchPacksOrCurse"
          >
            <svg v-if="modPackLoading || cpLoading" viewBox="0 0 16 16" class="h-3.5 w-3.5 animate-spin fill-current">
              <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
            </svg>
            <svg v-else viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
              <path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"/>
            </svg>
            {{ t("mods.search") }}
          </button>
        </div>
        <div v-if="modPackService === 'modrinth'" class="flex shrink-0 flex-wrap items-center gap-2 border-b border-[var(--border)] px-4 py-2">
          <FilterSelect
            v-model="packFilters.versions"
            :options="packVersionOptions"
            :placeholder="t('mods.fVersion')"
            @change="searchPacks()"
          />
          <FilterSelect
            v-model="packFilters.loaders"
            :options="packLoaderOptions"
            :placeholder="t('mods.fLoader')"
            @change="searchPacks()"
          />
          <FilterSelect
            v-model="packVersionTypeSel"
            :options="versionTypeOptions"
            :placeholder="t('mods.fType')"
            :multiple="false"
            @change="searchPacks()"
          />
          <FilterSelect
            v-model="packFilters.categories"
            :options="packCategoryOptions"
            :placeholder="t('mods.fCategory')"
            @change="searchPacks()"
          />
          <FilterSelect
            v-model="packEnvSel"
            :options="envOptions"
            :placeholder="t('mods.fAny')"
            :multiple="false"
            @change="searchPacks()"
          />
          <FilterSelect
            v-model="packSortSel"
            :options="sortSelectOptions"
            :placeholder="t('mods.fSort')"
            :multiple="false"
            @change="searchPacks()"
          />
        </div>
        <div v-if="modPackService === 'modrinth'" class="min-h-0 flex-1 overflow-y-auto p-4">
          <template v-if="modPackDetail">
            <button
              type="button"
              class="mb-3 flex items-center gap-1 text-xs text-[color:var(--tx-muted)] transition-colors hover:text-[var(--accent)]"
              @click="modPackDetail = null; modPackVersions = null"
            >
              <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M7.28 3.22a.75.75 0 0 1 0 1.06L3.56 8l3.72 3.72a.75.75 0 1 1-1.06 1.06l-4.25-4.25a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0Zm4 0a.75.75 0 0 1 0 1.06L7.56 8l3.72 3.72a.75.75 0 1 1-1.06 1.06l-4.25-4.25a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0Z"/></svg>
              {{ t("mods.back") }}
            </button>
            <div class="overflow-hidden rounded-md border border-[var(--border)] bg-[var(--bg)]">
              <div class="flex items-start gap-3 px-4 py-3">
                <img
                  v-if="modPackDetail.iconUrl"
                  :src="modPackDetail.iconUrl"
                  alt=""
                  class="h-14 w-14 shrink-0 rounded-md object-cover"
                />
                <div v-else class="flex h-14 w-14 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-sm font-bold text-[color:var(--tx-muted)]">
                  {{ modPackDetail.title.slice(0, 2).toUpperCase() }}
                </div>
                <div class="min-w-0 flex-1">
                  <h4 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ modPackDetail.title }}</h4>
                  <div class="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-[11px] text-[color:var(--tx-muted)]">
                    <span>{{ t("mods.byAuthor", { author: modPackDetail.author }) }}</span>
                    <span class="flex items-center gap-1">
                      <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1.75a.75.75 0 0 0-1.5 0v9A2.25 2.25 0 0 0 2.5 13h12.75a.75.75 0 0 0 0-1.5H2.5a.75.75 0 0 1-.75-.75v-9Zm10.75 2.5a.75.75 0 0 0-1.5 0v5a.75.75 0 0 0 1.5 0v-5Zm-3 .75a.75.75 0 0 1 1.5 0v4.25a.75.75 0 0 1-1.5 0V5Zm-3 1.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3Z"/></svg>
                      {{ modPackDetail.downloads.toLocaleString() }}
                    </span>
                    <span v-if="modPackDetail.categories.length">{{ modPackDetail.categories.slice(0, 4).join(", ") }}</span>
                  </div>
                </div>
                <button
                  type="button"
                  class="flex shrink-0 items-center gap-1.5 rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1.5 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                  @click="openExternal(`https://modrinth.com/modpack/${modPackDetail!.slug}`)"
                >
                  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M3.75 2h3.5a.75.75 0 0 1 0 1.5h-2l6 6V7.5a.75.75 0 0 1 1.5 0v4.5a.75.75 0 0 1-.75.75H5.5a.75.75 0 0 1 0-1.5h2l-6-6v2a.75.75 0 0 1-1.5 0V3.5A1.75 1.75 0 0 1 1.75 1.75h2a.75.75 0 0 1 0 1.5Z"/></svg>
                  {{ t("mods.openPage") }}
                </button>
              </div>
            </div>

            <!-- Вкладки как на Modrinth: описание / версии / галерея -->
            <div class="mt-3 mb-3 flex shrink-0 items-center gap-1 border-b border-[var(--border)] pb-2">
              <button
                v-for="tb in modPackTabs"
                :key="tb.kind"
                type="button"
                class="rounded-md px-3 py-1.5 text-xs font-medium transition-colors"
                :class="modPackTab === tb.kind
                  ? 'bg-[var(--input)] text-[color:var(--tx-strong)]'
                  : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]'"
                @click="modPackTab = tb.kind"
              >
                {{ t("mods.tab" + tb.kind) }}
              </button>
            </div>

            <div v-if="modPackTab === 'about'" class="max-h-[46vh] overflow-y-auto rounded-md border border-[var(--border)] bg-[var(--bg)] px-4 py-3">
              <Markdown v-if="modPackDetail.body" :source="modPackDetail.body" />
              <p v-else class="py-6 text-center text-xs italic text-[color:var(--tx-muted)]">{{ t("mods.noAbout") }}</p>
            </div>

            <div v-else-if="modPackTab === 'versions'">
              <div v-if="modPackVersions && modPackVersions.length === 0" class="py-8 text-center text-xs text-[color:var(--tx-muted)]">{{ t("mods.noVersions") }}</div>
              <div v-else-if="modPackVersions" class="space-y-2">
                <div
                  v-for="v in modPackVersions"
                  :key="v.id"
                  class="flex items-center gap-3 rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2"
                >
                  <div class="min-w-0 flex-1">
                    <div class="flex flex-wrap items-center gap-x-2 gap-y-1">
                      <span class="truncate text-xs font-medium text-[color:var(--tx-strong)]">{{ v.name }}</span>
                      <span class="rounded border border-[var(--border)] bg-[var(--input-50)] px-1.5 py-0.5 font-mono text-[11px] text-[color:var(--tx-muted)]">{{ v.versionNumber }}</span>
                    </div>
                    <div class="mt-0.5 truncate text-[11px] text-[color:var(--tx-muted)]">
                      {{ v.gameVersions.slice(0, 2).join(", ") }} · {{ v.loaders.join(", ") }} · {{ formatDate(v.datePublished) }}
                    </div>
                  </div>
                  <button
                    type="button"
                    class="flex shrink-0 items-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                    :disabled="modPackInstalling !== null"
                    @click="installPackVersion(v)"
                  >
                    <svg v-if="modPackInstalling === v.id" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                      <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                    </svg>
                    <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                      <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
                    </svg>
                    {{ t("mods.install") }}
                  </button>
                </div>
              </div>
              <div v-else class="flex items-center justify-center py-10 text-xs text-[color:var(--tx-muted)]">
                <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current">
                  <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                </svg>
                {{ t("mods.searching") }}
              </div>
            </div>

            <div v-else>
              <div v-if="modPackDetail.gallery.length" class="grid grid-cols-2 gap-2">
                <img
                  v-for="g in modPackDetail.gallery"
                  :key="g.url"
                  :src="g.url"
                  :alt="g.title ?? ''"
                  loading="lazy"
                  class="h-32 w-full cursor-zoom-in rounded-md border border-[var(--border)] object-cover transition-transform hover:scale-[1.02]"
                  :title="g.title ?? undefined"
                  @click="openExternal(g.url)"
                />
              </div>
              <p v-else class="py-10 text-center text-xs italic text-[color:var(--tx-muted)]">{{ t("mods.noGallery") }}</p>
            </div>
          </template>
          <template v-else-if="modPackLoading">
            <div class="flex items-center justify-center py-16 text-xs text-[color:var(--tx-muted)]">
              <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current">
                <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
              </svg>
              {{ t("mods.searching") }}
            </div>
          </template>
          <template v-else-if="modPackResults.length === 0">
            <div class="py-16 text-center text-xs text-[color:var(--tx-muted)]">
              {{ modPackQuery ? t("mods.noResults") : t("mods.packsHelp") }}
            </div>
          </template>
          <template v-else>
            <div class="space-y-2">
              <div
                v-for="p in modPackResults"
                :key="p.projectId"
                class="flex cursor-pointer items-start gap-3 rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2.5 transition-colors hover:border-[color-mix(in_srgb,var(--accent)_50%,transparent)]"
                @click="openPackDetail(p)"
              >
                <img v-if="p.iconUrl" :src="p.iconUrl" alt="" loading="lazy" class="h-10 w-10 shrink-0 rounded-md object-cover" />
                <div v-else class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-[11px] text-[color:var(--tx-muted)]">
                  {{ p.title.slice(0, 2).toUpperCase() }}
                </div>
                <div class="min-w-0 flex-1">
                  <div class="flex flex-wrap items-center gap-x-2">
                    <svg viewBox="0 0 24 24" class="h-3 w-3 shrink-0 self-center" :title="t('mods.serviceModrinth')"><path fill="#00AF5C" d="M12.252.004a11.78 11.768 0 0 0-8.92 3.73 11 10.999 0 0 0-2.17 3.11 11.37 11.359 0 0 0-1.16 5.169c0 1.42.17 2.5.6 3.77.24.759.77 1.899 1.17 2.529a12.3 12.298 0 0 0 8.85 5.639c.44.05 2.54.07 2.76.02.2-.04.22.1-.26-1.7l-.36-1.37-1.01-.06a8.5 8.489 0 0 1-5.18-1.8 5.34 5.34 0 0 1-1.3-1.26c0-.05.34-.28.74-.5a37.572 37.545 0 0 1 2.88-1.629c.03 0 .5.45 1.06.98l1 .97 2.07-.43 2.06-.43 1.47-1.47c.8-.8 1.48-1.5 1.48-1.52 0-.09-.42-1.63-.46-1.7-.04-.06-.2-.03-1.02.18-.53.13-1.2.3-1.45.4l-.48.15-.53.53-.53.53-.93.1-.93.07-.52-.5a2.7 2.7 0 0 1-.96-1.7l-.13-.6.43-.57c.68-.9.68-.9 1.46-1.1.4-.1.65-.2.83-.33.13-.099.65-.579 1.14-1.069l.9-.9-.7-.7-.7-.7-1.95.54c-1.07.3-1.96.53-1.97.53-.03 0-2.23 2.48-2.63 2.97l-.29.35.28 1.03c.16.56.3 1.16.31 1.34l.03.3-.34.23c-.37.23-2.22 1.3-2.84 1.63-.36.2-.37.2-.44.1-.08-.1-.23-.6-.32-1.03-.18-.86-.17-2.75.02-3.73a8.84 8.839 0 0 1 7.9-6.93c.43-.03.77-.08.78-.1.06-.17.5-2.999.47-3.039-.01-.02-.1-.02-.2-.03Zm3.68.67c-.2 0-.3.1-.37.38-.06.23-.46 2.42-.46 2.52 0 .04.1.11.22.16a8.51 8.499 0 0 1 2.99 2 8.38 8.379 0 0 1 2.16 3.449 6.9 6.9 0 0 1 .4 2.8c0 1.07 0 1.27-.1 1.73a9.37 9.369 0 0 1-1.76 3.769c-.32.4-.98 1.06-1.37 1.38-.38.32-1.54 1.1-1.7 1.14-.1.03-.1.06-.07.26.03.18.64 2.56.7 2.78l.06.06a12.07 12.058 0 0 0 7.27-9.4c.13-.77.13-2.58 0-3.4a11.96 11.948 0 0 0-5.73-8.578c-.7-.42-2.05-1.06-2.25-1.06Z"/></svg>
                    <span class="truncate text-xs font-semibold text-[color:var(--tx-strong)]">{{ p.title }}</span>
                    <span class="text-[11px] text-[color:var(--tx-muted)]">{{ t("mods.byAuthor", { author: p.author }) }}</span>
                  </div>
                  <p class="mt-0.5 line-clamp-2 text-xs leading-snug text-[color:var(--tx-muted)]">{{ p.description }}</p>
                  <div class="mt-1 flex items-center gap-3 text-[11px] text-[color:var(--tx-muted)]">
                    <span class="flex items-center gap-1">
                      <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1.75a.75.75 0 0 0-1.5 0v9A2.25 2.25 0 0 0 2.5 13h12.75a.75.75 0 0 0 0-1.5H2.5a.75.75 0 0 1-.75-.75v-9Zm10.75 2.5a.75.75 0 0 0-1.5 0v5a.75.75 0 0 0 1.5 0v-5Zm-3 .75a.75.75 0 0 1 1.5 0v4.25a.75.75 0 0 1-1.5 0V5Zm-3 1.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3Z"/></svg>
                      {{ p.downloads.toLocaleString() }}
                    </span>
                    <span v-if="status?.minecraft_version">{{ status.minecraft_version }}</span>
                  </div>
                </div>
                <button
                  type="button"
                  class="flex shrink-0 items-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                  :disabled="quickPackBusy !== null || modPackInstalling !== null"
                  :title="t('mods.downloadHint')"
                  @click="quickDownloadPack(p, $event)"
                >
                  <svg v-if="quickPackBusy === p.projectId" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                    <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                  </svg>
                  <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                    <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
                  </svg>
                  {{ t("mods.download") }}
                </button>
                <svg viewBox="0 0 16 16" class="mt-1 h-3.5 w-3.5 shrink-0 fill-[var(--tx-muted)]"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"/></svg>
              </div>
            </div>
          </template>
        </div>
        <div v-if="modPackService === 'curseforge'" class="flex shrink-0 flex-wrap items-center gap-2 border-b border-[var(--border)] px-4 py-2">
          <FilterSelect
            v-model="cpCatSel"
            :options="cpCatOptions"
            :placeholder="t('curse.fCategory')"
            :multiple="false"
            @change="searchCursePacks"
          />
          <FilterSelect
            v-model="cpVerSel"
            :options="packVersionOptions"
            :placeholder="t('mods.fVersion')"
            :multiple="false"
            @change="searchCursePacks"
          />
          <FilterSelect
            v-model="cpSortSel"
            :options="curseSortOptions"
            :placeholder="t('mods.fSort')"
            :multiple="false"
            @change="searchCursePacks"
          />
        </div>
        <div v-if="modPackService === 'curseforge'" class="min-h-0 flex-1 overflow-y-auto px-4 py-3">
          <p v-if="!cpSearched" class="py-8 text-center text-xs text-[color:var(--tx-muted)]">{{ t("curse.packsHelp") }}</p>
          <p v-else-if="cpLoading" class="flex items-center justify-center gap-2 py-8 text-xs text-[color:var(--tx-muted)]">
            <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 animate-spin fill-current">
              <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
            </svg>
            {{ t("mods.searchingAll") }}
          </p>
          <div v-else-if="cpErr" class="rounded-md border border-[var(--border)] bg-[var(--input-50)] p-6 text-center text-xs text-[color:var(--tx-muted)]">
            <p class="mb-2 whitespace-pre-wrap">{{ cpErr }}</p>
            <button type="button" class="text-[var(--accent)] hover:underline" @click="searchCursePacks">{{ t("catalog.retry") }}</button>
          </div>
          <template v-else-if="cpProject">
            <button
              type="button"
              class="mb-3 flex items-center gap-1 text-xs text-[color:var(--tx-muted)] transition-colors hover:text-[var(--accent)]"
              @click="cpProject = null; cpFiles = null; cpDetail = null; cpErr = ''"
            >
              <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M7.28 3.22a.75.75 0 0 1 0 1.06L3.56 8l3.72 3.72a.75.75 0 1 1-1.06 1.06l-4.25-4.25a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0Zm4 0a.75.75 0 0 1 0 1.06L7.56 8l3.72 3.72a.75.75 0 1 1-1.06 1.06l-4.25-4.25a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0Z"/></svg>
              {{ t("mods.back") }}
            </button>
            <div class="mb-3 rounded-md border border-[var(--border)] bg-[var(--bg)]">
              <div class="flex items-start gap-3 px-4 py-3">
                <img
                  v-if="cpProject.iconUrl"
                  :src="cpProject.iconUrl"
                  alt=""
                  class="h-14 w-14 shrink-0 rounded-md object-cover"
                />
                <div v-else class="flex h-14 w-14 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-sm font-bold text-[color:var(--tx-muted)]">
                  {{ cpProject.name.slice(0, 2).toUpperCase() }}
                </div>
                <div class="min-w-0 flex-1">
                  <h4 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ cpProject.name }}</h4>
                  <div class="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-[11px] text-[color:var(--tx-muted)]">
                    <span>{{ t("mods.byAuthor", { author: cpProject.author }) }}</span>
                    <span class="flex items-center gap-1">
                      <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1.75a.75.75 0 0 0-1.5 0v9A2.25 2.25 0 0 0 2.5 13h12.75a.75.75 0 0 0 0-1.5H2.5a.75.75 0 0 1-.75-.75v-9Zm10.75 2.5a.75.75 0 0 0-1.5 0v5a.75.75 0 0 0 1.5 0v-5Zm-3 .75a.75.75 0 0 1 1.5 0v4.25a.75.75 0 0 1-1.5 0V5Zm-3 1.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3Z"/></svg>
                      {{ cpProject.downloadCount.toLocaleString() }}
                    </span>
                    <span v-if="cpDetail?.categories.length">{{ cpDetail.categories.slice(0, 4).join(", ") }}</span>
                  </div>
                </div>
                <a
                  v-if="cpWebsiteUrl"
                  href="#"
                  class="flex shrink-0 items-center gap-1.5 rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1.5 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                  @click.prevent="openExternal(cpWebsiteUrl)"
                >
                  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M3.75 2h3.5a.75.75 0 0 1 0 1.5h-2l6 6V7.5a.75.75 0 0 1 1.5 0v4.5a.75.75 0 0 1-.75.75H5.5a.75.75 0 0 1 0-1.5h2l-6-6v2a.75.75 0 0 1-1.5 0V3.5A1.75 1.75 0 0 1 1.75 1.75h2a.75.75 0 0 1 0 1.5Z"/></svg>
                  {{ t("mods.openPage") }}
                </a>
              </div>
            </div>

            <!-- Вкладки как на Modrinth: описание / версии / скриншоты -->
            <div class="mt-3 mb-3 flex shrink-0 items-center gap-1 border-b border-[var(--border)] pb-2">
              <button
                v-for="tb in cpTabs"
                :key="tb"
                type="button"
                class="rounded-md px-3 py-1.5 text-xs font-medium transition-colors"
                :class="cpTab === tb
                  ? 'bg-[var(--input)] text-[color:var(--tx-strong)]'
                  : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]'"
                @click="cpTab = tb"
              >
                {{ t("curse.tab" + tb) }}
              </button>
            </div>

            <div v-if="cpTab === 'about'">
              <div v-if="cpDetailLoading" class="flex items-center justify-center py-10 text-xs text-[color:var(--tx-muted)]">
                <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current">
                  <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                </svg>
                {{ t("mods.searching") }}
              </div>
              <div v-else-if="cpDetail?.description" class="max-h-[46vh] overflow-y-auto rounded-md border border-[var(--border)] bg-[var(--bg)] px-4 py-3 leading-relaxed">
                <p class="whitespace-pre-wrap text-xs text-[color:var(--tx)]">{{ cpDetail.description }}</p>
              </div>
              <div v-else class="py-8 text-center text-xs italic text-[color:var(--tx-muted)]">{{ t("mods.noAbout") }}</div>
            </div>

            <div v-else-if="cpTab === 'versions'">
              <div v-if="cpFiles === null" class="flex items-center justify-center py-10 text-xs text-[color:var(--tx-muted)]">
                <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current">
                  <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                </svg>
                {{ t("mods.searching") }}
              </div>
              <div v-else-if="cpFiles.length === 0" class="py-8 text-center text-xs text-[color:var(--tx-muted)]">{{ t("curse.noFiles") }}</div>
              <div v-else class="space-y-2">
                <div
                  v-for="f in cpFiles"
                  :key="f.fileId"
                  class="flex items-center gap-3 rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2"
                >
                  <div class="min-w-0 flex-1">
                    <div class="truncate text-xs font-medium text-[color:var(--tx-strong)]">{{ f.displayName }}</div>
                    <div class="mt-0.5 truncate text-[11px] text-[color:var(--tx-muted)]">
                      {{ f.gameVersion }} · {{ formatDate(f.fileDate) }}
                    </div>
                  </div>
                  <button
                    type="button"
                    class="flex shrink-0 items-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                    :disabled="cpBusy !== null"
                    @click="installCpPack(f)"
                  >
                    <svg v-if="cpBusy === f.fileId" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                      <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                    </svg>
                    <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                      <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
                    </svg>
                    {{ t("mods.install") }}
                  </button>
                </div>
              </div>
            </div>

            <div v-else>
              <div v-if="cpDetailLoading" class="flex items-center justify-center py-10 text-xs text-[color:var(--tx-muted)]">
                <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current">
                  <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                </svg>
                {{ t("mods.searching") }}
              </div>
              <div v-else-if="cpDetail?.screenshots.length" class="grid grid-cols-2 gap-2">
                <img
                  v-for="(s, i) in cpDetail.screenshots"
                  :key="i"
                  :src="s"
                  alt=""
                  loading="lazy"
                  class="h-32 w-full cursor-zoom-in rounded-md border border-[var(--border)] object-cover transition-transform hover:scale-[1.02]"
                  @click="openExternal(s)"
                />
              </div>
              <div v-else class="py-8 text-center text-xs text-[color:var(--tx-muted)]">{{ t("curse.noScreenshots") }}</div>
            </div>
          </template>
          <template v-else-if="cpResults.length === 0">
            <p class="py-8 text-center text-xs text-[color:var(--tx-muted)]">{{ t("mods.noResults") }}</p>
          </template>
          <template v-else>
            <div class="space-y-2">
              <div
                v-for="p in cpResults"
                :key="p.projectId"
                class="flex cursor-pointer items-start gap-3 rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2.5 transition-colors hover:border-[color-mix(in_srgb,var(--accent)_50%,transparent)]"
                @click="openCpFiles(p)"
              >
                <img v-if="p.iconUrl" :src="p.iconUrl" alt="" loading="lazy" class="h-10 w-10 shrink-0 rounded-md object-cover" />
                <div v-else class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-[11px] text-[color:var(--tx-muted)]">
                  {{ p.name.slice(0, 2).toUpperCase() }}
                </div>
                <div class="min-w-0 flex-1">
                  <div class="flex flex-wrap items-center gap-x-2">
                    <svg viewBox="0 0 24 24" class="h-3 w-3 shrink-0 self-center" :title="t('mods.serviceCurseforge')"><path fill="#F16436" d="M18.326 9.2145S23.2261 8.4418 24 6.1882h-7.5066V4.4H0l2.0318 2.3576V9.173s5.1267-.2665 7.1098 1.2372c2.7146 2.516-3.053 5.917-3.053 5.917L5.0995 19.6c1.5465-1.4726 4.494-3.3775 9.8983-3.2857-2.0565.65-4.1245 1.6651-5.7344 3.2857h10.9248l-1.0288-3.2726s-7.918-4.6688-.8336-7.1127z"/></svg>
                    <span class="truncate text-xs font-semibold text-[color:var(--tx-strong)]">{{ p.name }}</span>
                    <span class="text-[11px] text-[color:var(--tx-muted)]">{{ t("mods.byAuthor", { author: p.author }) }}</span>
                  </div>
                  <p class="mt-0.5 line-clamp-2 text-xs leading-snug text-[color:var(--tx-muted)]">{{ p.summary }}</p>
                  <p class="mt-1 flex items-center gap-1 text-[11px] text-[color:var(--tx-muted)]">
                    <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1.75a.75.75 0 0 0-1.5 0v9A2.25 2.25 0 0 0 2.5 13h12.75a.75.75 0 0 0 0-1.5H2.5a.75.75 0 0 1-.75-.75v-9Zm10.75 2.5a.75.75 0 0 0-1.5 0v5a.75.75 0 0 0 1.5 0v-5Zm-3 .75a.75.75 0 0 1 1.5 0v4.25a.75.75 0 0 1-1.5 0V5Zm-3 1.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3Z"/></svg>
                    {{ p.downloadCount.toLocaleString() }}
                  </p>
                </div>
                <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 shrink-0 fill-[var(--tx-muted)]"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"/></svg>
              </div>
            </div>
          </template>
        </div>
      </div>
    </div>

    <!-- Модалка: создание своей сборки -->
    <div
      v-if="createPackOpen"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6"
      @click.self="createPackOpen = false"
    >
      <div class="flex w-full max-w-md flex-col overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-2xl">
        <div class="flex shrink-0 items-center justify-between border-b border-[var(--border)] px-4 py-3">
          <h3 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ t("mods.createTitle") }}</h3>
          <button
            type="button"
            class="rounded-md p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
            @click="createPackOpen = false"
          >
            <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
          </button>
        </div>
        <div class="space-y-4 overflow-y-auto p-4">
          <div>
            <label class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("mods.createName") }}</label>
            <input
              v-model="createPackName"
              type="text"
              :placeholder="t('mods.createNamePlaceholder')"
              class="w-full rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] outline-none transition-colors focus:border-[var(--accent)]"
              @keydown.enter="createPack"
            />
          </div>
          <div>
            <label class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("mods.createMc") }}</label>
            <div ref="createPackVersionBox" class="relative">
              <button
                type="button"
                class="flex w-full items-center justify-between gap-2 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] focus:border-[var(--accent)]"
                @click="createPackVersionOpen = !createPackVersionOpen"
              >
                <span class="truncate">{{ createPackMc }}</span>
                <svg
                  viewBox="0 0 16 16"
                  class="h-3.5 w-3.5 shrink-0 fill-[var(--tx-muted)] transition-transform"
                  :class="createPackVersionOpen ? 'rotate-180' : ''"
                >
                  <path d="M4.22 6.22a.75.75 0 0 1 1.06 0L8 8.94l2.72-2.72a.75.75 0 1 1 1.06 1.06l-3.25 3.25a.75.75 0 0 1-1.06 0L4.22 7.28a.75.75 0 0 1 0-1.06Z"/>
                </svg>
              </button>
              <div
                v-if="createPackVersionOpen"
                class="absolute left-0 right-0 top-full z-50 mt-1 flex max-h-64 flex-col overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm shadow-2xl"
              >
                <div class="shrink-0 border-b border-[var(--border)] p-1.5">
                  <input
                    v-model="createPackVersionQuery"
                    type="text"
                    :placeholder="t('mods.createSearch')"
                    class="w-full rounded-md border border-[var(--border)] bg-[var(--bg)] px-2 py-1 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] outline-none transition-colors focus:border-[var(--accent)]"
                  />
                </div>
                <div class="flex-1 overflow-y-auto py-1">
                  <div v-for="group in createVersionGroups" :key="group.label">
                    <div
                      v-if="group.items.length"
                      class="px-3 pb-0.5 pt-1.5 text-[10px] font-semibold uppercase tracking-wider text-[color:var(--tx-muted)]"
                    >{{ group.label }}</div>
                    <button
                      v-for="v in group.items"
                      :key="v.id"
                      type="button"
                      class="flex w-full items-center gap-1.5 px-3 py-1 text-left text-xs transition-colors hover:bg-[var(--hover)]"
                      :class="createPackMc === v.id ? 'text-[var(--accent)]' : 'text-[color:var(--tx)]'"
                      @click="chooseCreateVersion(v.id)"
                    >
                      <span class="min-w-0 truncate">{{ v.id }}</span>
                      <svg v-if="v.kind === 'snapshot'" viewBox="0 0 16 16" class="ml-auto h-3 w-3 shrink-0 fill-[var(--tx-muted)]">
                        <path d="M2 3.5A1.5 1.5 0 0 1 3.5 2h1.25c.492 0 .923.24 1.184.61.39.553.997.89 2.066.89.993 0 1.613-.364 2.02-.907A1.36 1.36 0 0 1 11.25 2h1.25A1.5 1.5 0 0 1 14 3.5V5a.75.75 0 0 1-1.5 0v-.087a.471.471 0 0 0-.22-.39l-.02-.01c-.116-.08-.316-.08-.486.067-.325.28-.766.37-1.454.37h-.4c-.98 0-1.83.6-2.68 1.386l-.837.774A2.75 2.75 0 0 0 4.06 9.17l-.654 1.085A4.443 4.443 0 0 0 2.25 12.85V14a.75.75 0 0 1-1.5 0v-1.15c0-1.6.542-3.12 1.48-4.345l.713-1.183A4.25 4.25 0 0 1 4.7 6.31c.5-.683 1.07-1.31 1.608-1.56h.044l.26-.24C7.165 3.985 7.71 3.5 8.53 3.5h.72v-.027.027c.588 0 .986-.141 1.284-.286a1.52 1.52 0 0 0 .381-.287l.008-.008a.486.486 0 0 0 .085-.167.75.75 0 0 1 1.492.197V3.5A1.5 1.5 0 0 1 12.5 5a.75.75 0 0 1 0 1.5zm7.28 7.378a.75.75 0 0 1 .77.077.25.25 0 0 1 0 .03v1.6A1.5 1.5 0 0 1 8.55 14l-.3-2a.75.75 0 0 1 1.44-.22l.59 3.17Z"/>
                      </svg>
                    </button>
                  </div>
                  <div
                    v-if="!filteredCreateReleases.length && !filteredCreateSnapshots.length"
                    class="px-3 py-2 text-[11px] text-[color:var(--tx-muted)]"
                  >{{ t("mods.createNone") }}</div>
                </div>
              </div>
            </div>
          </div>
          <div>
            <label class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("mods.createLoader") }}</label>
            <div class="flex flex-wrap gap-2">
              <button
                v-for="l in CREATE_LOADERS"
                :key="l"
                type="button"
                class="flex-1 rounded-md border px-3 py-1.5 text-xs font-medium capitalize transition-colors"
                :class="createPackLoader === l
                  ? 'border-[color-mix(in_srgb,var(--accent)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] text-[var(--accent)]'
                  : 'border-[var(--border)] bg-[var(--input)] text-[color:var(--tx)] hover:bg-[var(--hover)]'"
                @click="createPackLoader = l"
              >
                {{ l }}
              </button>
            </div>
          </div>
          <div v-if="createPackLoader !== 'vanilla'">
            <label class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("mods.createLoaderVersion") }}</label>
            <div ref="createPackLvBox" class="relative">
              <button
                type="button"
                class="flex w-full items-center justify-between gap-2 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                @click="createPackLoaderLvOpen = !createPackLoaderLvOpen"
              >
                <span class="truncate">{{ createPackLoaderVersion || t("mods.createLatest") }}</span>
                <svg
                  viewBox="0 0 16 16"
                  class="h-3.5 w-3.5 shrink-0 fill-[var(--tx-muted)] transition-transform"
                  :class="createPackLoaderLvOpen ? 'rotate-180' : ''"
                >
                  <path d="M4.22 6.22a.75.75 0 0 1 1.06 0L8 8.94l2.72-2.72a.75.75 0 1 1 1.06 1.06l-3.25 3.25a.75.75 0 0 1-1.06 0L4.22 7.28a.75.75 0 0 1 0-1.06Z"/>
                </svg>
              </button>
              <div
                v-if="createPackLoaderLvOpen"
                class="absolute left-0 right-0 top-full z-50 mt-1 max-h-52 overflow-y-auto rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm py-1 shadow-2xl"
              >
                <button
                  type="button"
                  class="flex w-full items-center justify-between px-3 py-1 text-left text-xs transition-colors hover:bg-[var(--hover)]"
                  :class="createPackLoaderVersion === '' ? 'text-[var(--accent)]' : 'text-[color:var(--tx)]'"
                  @click="chooseCreateLoaderVersion('')"
                >
                  <span>{{ t("mods.createLatest") }}</span>
                  <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M12.78 4.22a.75.75 0 0 1 0 1.06l-5.78 5.78a.75.75 0 0 1-1.06 0l-2.5-2.5a.75.75 0 1 1 1.06-1.06L6.5 9.44l5.22-5.22a.75.75 0 0 1 1.06 0Z"/></svg>
                </button>
                <div v-if="!createPackLoaderVersions.length" class="px-3 py-1.5 text-[11px] text-[color:var(--tx-muted)]">{{ t("mods.createLvNone") }}</div>
                <button
                  v-for="v in createPackLoaderVersions"
                  :key="v"
                  type="button"
                  class="flex w-full items-center gap-2 px-3 py-1 text-left text-xs transition-colors hover:bg-[var(--hover)]"
                  :class="createPackLoaderVersion === v ? 'text-[var(--accent)]' : 'text-[color:var(--tx)]'"
                  @click="chooseCreateLoaderVersion(v)"
                >
                  <span class="min-w-0 truncate">{{ v }}</span>
                  <svg v-if="createPackLoaderVersion === v" viewBox="0 0 16 16" class="ml-auto h-3.5 w-3.5 shrink-0 fill-current"><path d="M12.78 4.22a.75.75 0 0 1 0 1.06l-5.78 5.78a.75.75 0 0 1-1.06 0l-2.5-2.5a.75.75 0 1 1 1.06-1.06L6.5 9.44l5.22-5.22a.75.75 0 0 1 1.06 0Z"/></svg>
                </button>
              </div>
            </div>
          </div>
          <div class="grid grid-cols-2 gap-2">
            <div>
              <label class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("mods.createIcon") }}</label>
              <button
                type="button"
                class="flex w-full items-center justify-center gap-1.5 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)]"
                @click="pickCreateFile('icon')"
              >
                <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M4.5 2.75A2.75 2.75 0 0 1 7.25 0h1.5A2.75 2.75 0 0 1 11.5 2.75 2.75 2.75 0 0 1 16 5.5v5A2.75 2.75 0 0 1 13.25 13.25V13H2.75A2.75 2.75 0 0 1 0 10.25v-4.5A2.75 2.75 0 0 1 2.75 3c1.12 0 2.097.523 1.75-1.5Z"/></svg>
                <span class="min-w-0 truncate">{{ createPackIcon ? createPackIcon.split(/[\\/]/).pop() : t("mods.createChoose") }}</span>
              </button>
              <button
                v-if="createPackIcon"
                type="button"
                class="mt-1 w-full rounded-md px-2 py-0.5 text-[11px] font-medium text-[var(--accent)] hover:opacity-80"
                @click="createPackIcon = null"
              >{{ t("mods.createRemove") }}</button>
            </div>
            <div>
              <label class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("mods.createBanner") }}</label>
              <button
                type="button"
                class="flex w-full items-center justify-center gap-1.5 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)]"
                @click="pickCreateFile('banner')"
              >
                <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M3.5 1.75A1.75 1.75 0 0 1 5.25 0h5.5c.966 0 1.75.784 1.75 1.75v12.5a.75.75 0 0 1-1.2.6L8 12.313l-3.3 2.537A.75.75 0 0 1 3.5 14.25V1.75Z"/></svg>
                <span class="min-w-0 truncate">{{ createPackBanner ? createPackBanner.split(/[\\/]/).pop() : t("mods.createChoose") }}</span>
              </button>
              <button
                v-if="createPackBanner"
                type="button"
                class="mt-1 w-full rounded-md px-2 py-0.5 text-[11px] font-medium text-[var(--accent)] hover:opacity-80"
                @click="createPackBanner = null"
              >{{ t("mods.createRemove") }}</button>
            </div>
          </div>
          <button
            type="button"
            class="flex w-full items-center justify-center gap-2 rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_10%,transparent)] px-3 py-2 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] disabled:opacity-50"
            :disabled="createPackBusy"
            @click="createPack"
          >
            <svg v-if="createPackBusy" viewBox="0 0 16 16" class="h-3.5 w-3.5 animate-spin fill-current">
              <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
            </svg>
            <svg v-else viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
              <path d="M8 2.75a.75.75 0 0 1 .75.75v3.75h3.75a.75.75 0 0 1 0 1.5h-3.75v3.75a.75.75 0 0 1-1.5 0V8.75H3.5a.75.75 0 0 1 0-1.5h3.75V3.5A.75.75 0 0 1 8 2.75Z"/>
            </svg>
            {{ t("mods.create") }}
          </button>
          <p class="text-[11px] leading-snug text-[color:var(--tx-muted)]">{{ t("mods.createHint") }}</p>
        </div>
        <button
          type="button"
          class="flex w-full items-center justify-center gap-2 px-4 pb-4 pt-0 text-xs font-medium text-[var(--accent)] transition-colors hover:opacity-80"
          @click="createPackOpen = false; openModPackModal()"
        >
          <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/></svg>
          {{ t("mods.createDownloadPack") }}
        </button>
      </div>
    </div>

    <!-- Модалка: отчёт об ошибке (превью + копирование + Issues) -->
    <div
      v-if="bugReportOpen"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6"
      @click.self="closeBugReport"
    >
      <div class="flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-2xl">
        <div class="flex shrink-0 items-center justify-between border-b border-[var(--border)] px-4 py-3">
          <h3 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ t("reportPack.modalTitle") }}</h3>
          <button
            type="button"
            class="rounded-md px-2 py-1 text-xs text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)]"
            @click="closeBugReport"
          >
            ✕
          </button>
        </div>
        <pre class="min-h-0 flex-1 overflow-y-auto whitespace-pre-wrap break-words px-4 py-3 font-mono text-xs leading-relaxed text-[color:var(--tx)]">{{ bugBody }}</pre>
        <div class="flex shrink-0 items-center justify-end gap-2 border-t border-[var(--border)] px-4 py-3">
          <span v-if="bugLog" class="mr-auto text-xs text-[color:var(--tx-muted)]">
            {{ t("reportPack.logNote", { n: bugLog.split("\n").slice(-60).length }) }}
          </span>
          <button
            type="button"
            class="rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
            @click="copyBugReport"
          >
            {{ bugCopied ? t("reportPack.copied") : t("reportPack.copy") }}
          </button>
          <button
            type="button"
            class="rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)]"
            @click="openBugReportIssue"
          >
            {{ t("reportPack.open") }}
          </button>
        </div>
      </div>
    </div>

    <!-- Модалка: вероятная причина краша (анализ crash-report / hs_err / лога) -->
    <div
      v-if="crashAnalysis"
      class="fixed inset-0 z-[55] flex items-center justify-center bg-black/50 p-6"
      @click.self="closeCrashAnalysis"
    >
      <div class="flex max-h-[80vh] w-full max-w-lg flex-col overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-2xl">
        <div class="flex shrink-0 items-start justify-between border-b border-[var(--border)] px-4 py-3">
          <div class="flex items-center gap-2">
            <svg viewBox="0 0 16 16" class="h-4 w-4 fill-[var(--accent-deep)]"><path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1Zm.75 8.5a.75.75 0 0 1-1.5 0V5.25a.75.75 0 0 1 1.5 0Zm-0.75 2.25a.9.9 0 1 1 0-1.8.9.9 0 0 1 0 1.8Z"/></svg>
            <h3 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ t("crash.title") }}</h3>
          </div>
          <button
            type="button"
            class="rounded-md px-2 py-1 text-xs text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)]"
            @click="closeCrashAnalysis"
          >
            ✕
          </button>
        </div>
        <div class="min-h-0 flex-1 space-y-3 overflow-y-auto px-4 py-3">
          <div>
            <div class="text-xs font-semibold text-[color:var(--tx-strong)]">{{ crashView(crashAnalysis).title }}</div>
            <p class="mt-0.5 text-xs leading-relaxed text-[color:var(--tx)]">{{ crashView(crashAnalysis).msg }}</p>
          </div>
          <div v-if="crashAnalysis.exception" class="rounded-md border border-[var(--border)] bg-[var(--input-50)] px-3 py-2">
            <div class="mb-0.5 text-[11px] font-medium uppercase tracking-wide text-[color:var(--tx-muted)]">{{ t("crash.exception") }}</div>
            <code class="break-words font-mono text-xs text-[color:var(--tx)]">{{ crashAnalysis.exception }}</code>
          </div>
          <div v-if="crashAnalysis.description" class="rounded-md border border-[var(--border)] bg-[var(--input-50)] px-3 py-2">
            <div class="mb-0.5 text-[11px] font-medium uppercase tracking-wide text-[color:var(--tx-muted)]">{{ t("crash.description") }}</div>
            <div class="break-words text-xs text-[color:var(--tx)]">{{ crashAnalysis.description }}</div>
          </div>
          <div v-if="crashAnalysis.suspected.length" class="rounded-md border border-[var(--border)] bg-[var(--input-50)] px-3 py-2">
            <div class="mb-1 text-[11px] font-medium uppercase tracking-wide text-[color:var(--tx-muted)]">{{ t("crash.suspected") }}</div>
            <ul class="space-y-1">
              <li v-for="m in crashAnalysis.suspected" :key="m.file" class="flex items-center justify-between gap-2 text-xs">
                <span class="min-w-0 truncate text-[color:var(--tx)]">{{ m.name }}</span>
                <code class="shrink-0 font-mono text-[11px] text-[var(--tx-muted)]">{{ m.file }}</code>
              </li>
            </ul>
          </div>
          <div class="rounded-md border border-[var(--border)] bg-[var(--input-50)] px-3 py-2">
            <div class="text-[11px] uppercase tracking-wide text-[color:var(--tx-muted)]">{{ t("crash.file") }}</div>
            <code class="font-mono text-xs text-[color:var(--tx)]">{{ crashAnalysis.file }}</code>
          </div>
        </div>
        <div class="flex shrink-0 items-center justify-between gap-2 border-t border-[var(--border)] px-4 py-3">
          <button
            type="button"
            class="rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
            @click="copyCrashAnalysis"
          >
            {{ t("reportPack.copy") }}
          </button>
          <button
            type="button"
            class="rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)]"
            @click="openCrashIssue"
          >
            {{ t("crash.report") }}
          </button>
        </div>
      </div>
    </div>

    </div>

    <!-- Модалка: сканер модов (.jar → SHA256 + опасные классы) -->
    <div v-if="scannerOpen" class="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 p-4" @click.self="scannerOpen = false">
      <div class="w-full max-w-lg overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-2xl">
        <div class="flex items-center justify-between gap-2 border-b border-[var(--border)] px-4 py-3">
          <h3 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ t("scanner.title") }}</h3>
          <button type="button" class="rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-xs text-[color:var(--tx-muted)] hover:text-[var(--accent)] transition-colors" @click="scannerOpen = false">
            <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 5.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 7l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 8.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 7 3.72 3.78a.75.75 0 0 1 0-1.06Z"/></svg>
          </button>
        </div>
        <div class="space-y-3 p-4">
          <p class="text-xs leading-snug text-[color:var(--tx-muted)]">{{ t("scanner.note") }}</p>

          <div class="flex items-center gap-2">
            <button
              type="button"
              class="flex-1 rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)] disabled:opacity-50"
              :disabled="scanBusy || !monoProfile"
              @click="pickAndScanJar"
            >
              <svg v-if="scanBusy" viewBox="0 0 16 16" class="mr-1 inline h-3 w-3 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
              {{ scanBusy ? t("scanner.scanning") : t("scanner.pick") }}
            </button>
          </div>
          <p v-if="!monoProfile" class="text-[11px] text-[color:var(--tx-muted)]">{{ t("author.needLogin") }}</p>

          <!-- Результат скана -->
          <div v-if="scanResult" class="space-y-2 rounded-lg border p-3"
            :class="scanResult.safe ? 'border-[#3fb950]/40 bg-[#3fb950]/5' : 'border-[#f85149]/50 bg-[#f85149]/10'">
            <div class="flex items-center gap-2">
              <span class="inline-flex h-6 w-6 items-center justify-center rounded-full"
                :class="scanResult.safe ? 'bg-[#3fb950]/15 text-[#3fb950]' : 'bg-[#f85149]/20 text-[#f85149]'">
                <svg v-if="scanResult.safe" viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M8 15A7 7 0 1 1 8 1a7 7 0 0 1 0 14Zm-1.31-4.21 4.55-4.55-1.06-1.06-3.49 3.49-1.42-1.42-1.06 1.06 2.48 2.48Z"/></svg>
                <svg v-else viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M8 1.5A6.5 6.5 0 0 1 14.5 8 6.5 6.5 0 0 1 8 14.5 6.5 6.5 0 0 1 1.5 8 6.5 6.5 0 0 1 8 1.5ZM7.25 9.75h1.5V4.5h-1.5v5.25Zm0 3h1.5v-1.5h-1.5v1.5Z"/></svg>
              </span>
              <p class="text-xs font-bold" :class="scanResult.safe ? 'text-[#3fb950]' : 'text-[#f85149]'">
                {{ scanResult.safe ? t("scanner.safe") : t("scanner.dangerous") }}
              </p>
              <span v-if="scanResult.cached" class="ml-auto rounded-full border border-[var(--border)] bg-[var(--input)] px-2 py-0.5 text-[10px] font-semibold uppercase text-[color:var(--tx-muted)]">{{ t("scanner.cached") }}</span>
            </div>
            <p class="text-xs text-[color:var(--tx)]">{{ scanResult.scanResult }}</p>
            <p class="break-all font-mono text-[11px] text-[color:var(--tx-muted)]">SHA-256: {{ scanResult.sha256 }}</p>
            <div v-if="scanResult.dangerousClasses" class="space-y-1">
              <p class="text-[11px] font-semibold uppercase tracking-wide text-[#f87171]">{{ t("scanner.classes") }}:</p>
              <div class="flex flex-wrap gap-1">
                <span v-for="cl in scanResult.dangerousClasses.split(',').map((s: string) => s.trim()).filter(Boolean)" :key="cl"
                  class="rounded bg-[#f85149]/15 px-1.5 py-0.5 font-mono text-[11px] text-[#f87171]">
                  {{ cl }}
                </span>
              </div>
            </div>
          </div>

          <!-- Проверка по хешу (без загрузки файла) -->
          <div class="space-y-1.5 border-t border-[var(--border)] pt-3">
            <p class="text-[11px] font-semibold uppercase tracking-wide text-[color:var(--tx-muted)]">{{ t("scanner.byHash") }}</p>
            <div class="flex items-center gap-2">
              <input
                v-model="scannerHash"
                type="text"
                :placeholder="t('scanner.hashPh')"
                class="min-w-0 flex-1 rounded-md border border-[var(--border)] bg-[var(--bg)] px-2.5 py-1.5 font-mono text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none"
                @keydown.enter="scanByHash(scannerHash)"
              />
              <button
                type="button"
                class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1.5 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                :disabled="scanBusy"
                @click="scanByHash(scannerHash)"
              >
                {{ t("scanner.checkBtn") }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Модалка: профиль пользователя (bio + сборки + комментарии) -->
    <div v-if="profileView || profileBusy" class="fixed inset-0 z-[65] flex items-center justify-center bg-black/60 p-4" @click.self="closeProfileView()">
      <div class="flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-2xl">
        <div class="flex items-center justify-between gap-2 border-b border-[var(--border)] px-4 py-3">
          <h3 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ t("profile.title") }}</h3>
          <button type="button" class="rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1 text-xs text-[color:var(--tx-muted)] hover:text-[var(--accent)] transition-colors" @click="closeProfileView()">
            <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 5.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 7l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 8.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 7 3.72 3.78a.75.75 0 0 1 0-1.06Z"/></svg>
          </button>
        </div>

        <div v-if="profileBusy && !profileView" class="flex items-center justify-center py-16 text-xs text-[color:var(--tx-muted)]">
          <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
          {{ t("catalog.loading") }}
        </div>

        <template v-if="profileView">
          <div class="flex items-center gap-3 border-b border-[var(--border)] px-4 py-3">
            <img v-if="profileView.profile.avatarUrl" :src="profileView.profile.avatarUrl" :alt="profileView.profile.user.username" class="h-12 w-12 shrink-0 rounded-full border border-[var(--border)] object-cover" />
            <div v-else class="flex h-12 w-12 shrink-0 items-center justify-center rounded-full border border-[var(--border)] bg-[var(--input)] font-mono text-lg font-bold text-[var(--accent)]">
              {{ profileView.profile.user.username?.[0]?.toUpperCase() ?? "?" }}
            </div>
            <div class="min-w-0 flex-1">
              <p class="truncate font-mono text-sm font-semibold text-[color:var(--tx-strong)]">{{ profileView.profile.user.username }}</p>
              <p class="text-xs text-[color:var(--tx-muted)]">
                {{ t("profile.joined") }}: {{ formatDate(profileView.profile.joinedAt) }} ·
                {{ t("profile.packsCount", { n: profileView.profile.packsCount }) }} ·
                {{ t("profile.commentsCount", { n: profileView.profile.commentsCount }) }}
              </p>
            </div>
          </div>

          <div class="min-h-0 flex-1 space-y-4 overflow-y-auto p-4">
            <!-- Bio -->
            <section class="rounded-lg border border-[var(--border)] bg-[var(--bg)] p-3">
              <p class="mb-1 text-[11px] font-semibold uppercase tracking-wide text-[color:var(--tx-muted)]">{{ t("profile.bio") }}</p>
              <template v-if="profileIsOwn && profileBioEditing">
                <textarea v-model="profileBioDraft" rows="3" class="w-full resize-y rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:border-[var(--accent)] focus:outline-none"></textarea>
                <div class="mt-2 flex gap-2">
                  <button type="button" class="rounded-md border border-[color-mix(in_srgb,var(--accent-deep)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-3 py-1.5 text-xs font-semibold text-white disabled:opacity-50"
                    :disabled="profileBusy"
                    @click="saveMyProfile(profileBioDraft); profileBioEditing = false">
                    {{ t("author.save") }}
                  </button>
                  <button type="button" class="rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1 text-xs text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="profileBioEditing = false">
                    {{ t("author.cancel") }}
                  </button>
                </div>
              </template>
              <template v-else>
                <p class="text-xs leading-relaxed text-[color:var(--tx)] whitespace-pre-wrap">{{ profileView.profile.bio || t("profile.noBio") }}</p>
                <button v-if="profileIsOwn" type="button" class="mt-1.5 text-[11px] font-medium text-[var(--accent)] hover:underline" @click="profileBioEditing = true; profileBioDraft = profileView!.profile.bio">
                  {{ t("profile.editBio") }}
                </button>
              </template>
            </section>

            <!-- Сборки -->
            <section class="space-y-2">
              <p class="text-[11px] font-semibold uppercase tracking-wide text-[color:var(--tx-muted)]">{{ t("profile.packs") }}</p>
              <div v-if="profileView.packs.length === 0" class="rounded-lg border border-[var(--border)] bg-[var(--bg)] p-3 text-center text-xs text-[color:var(--tx-muted)]">{{ t("author.noPacks") }}</div>
              <div v-for="p in profileView.packs" :key="p.id" class="flex items-center gap-3 rounded-lg border border-[var(--border)] bg-[var(--bg)] p-3">
                <img v-if="p.iconUrl" :src="p.iconUrl" :alt="p.name" class="h-9 w-9 shrink-0 rounded-md object-cover" />
                <div v-else class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-[var(--input)] text-xs font-semibold text-[var(--accent)]">
                  {{ p.name?.[0]?.toUpperCase() }}
                </div>
                <div class="min-w-0 flex-1">
                  <p class="truncate text-xs font-semibold text-[color:var(--tx-strong)]">{{ p.name }}</p>
                  <p class="truncate text-[11px] text-[color:var(--tx-muted)]">
                    <template v-if="p.version">v{{ p.version }} · </template>👍 {{ p.likes }} / 👎 {{ p.dislikes }} · {{ t("author.versions") }}: {{ p.versionsCount }}
                  </p>
                </div>
                <button type="button" class="shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="openCatalogPackById(p.id)">
                  {{ t("catalog.open") }}
                </button>
              </div>
            </section>

            <!-- Комментарии -->
            <section class="space-y-2">
              <p class="text-[11px] font-semibold uppercase tracking-wide text-[color:var(--tx-muted)]">{{ t("profile.comments") }}</p>
              <div v-if="profileView.comments.length === 0" class="rounded-lg border border-[var(--border)] bg-[var(--bg)] p-3 text-center text-xs text-[color:var(--tx-muted)]">{{ t("comments.empty") }}</div>
              <div v-for="cm in profileView.comments" :key="cm.id" class="rounded-lg border border-[var(--border)] bg-[var(--bg)] p-3">
                <div class="flex items-center gap-2">
                  <span class="text-xs font-semibold text-[var(--accent)]">{{ cm.packName }}</span>
                  <span class="text-[11px] text-[color:var(--tx-muted)]">{{ formatDate(cm.createdAt) }}</span>
                  <button v-if="isAdmin" type="button" class="ml-auto text-[11px] font-medium text-[#f87171] hover:underline disabled:opacity-50" :disabled="adminBusy" @click="adminDeleteComment(cm.id)">
                    {{ t("author.delete") }}
                  </button>
                </div>
                <p class="mt-1 text-xs leading-relaxed text-[color:var(--tx)] whitespace-pre-wrap">{{ cm.body }}</p>
              </div>
            </section>
          </div>
        </template>
      </div>
    </div>

    <!-- Модалка: смена версии Minecraft / загрузчика у своей сборки -->
    <div v-if="editVerOpen" class="fixed inset-0 z-[60] flex items-center justify-center bg-black/40 p-4" @click.self="editVerOpen = false">
      <div class="w-full max-w-md overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-2xl">
        <div class="flex items-center justify-between gap-2 border-b border-[var(--border)] px-4 py-3">
          <h3 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ t("pack.versionTitle") }}</h3>
          <button
            type="button"
            class="rounded-md p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
            @click="editVerOpen = false"
          >
            <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
          </button>
        </div>
        <div class="space-y-3 px-4 py-3">
          <div>
            <label class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("pack.editName") }}</label>
            <input
              v-model="editVerName"
              class="w-full rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs text-[color:var(--tx)] outline-none placeholder:text-[var(--tx-muted)] focus:border-[color-mix(in_srgb,var(--accent)_50%,transparent)]"
              :placeholder="activePack?.name || ''"
              maxlength="60"
            />
          </div>

          <div>
            <label class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("pack.editMedia") }}</label>
            <div class="flex items-center gap-2">
              <img
                v-if="activePack?.icon"
                :src="convertFileSrc(activePack.icon)"
                alt=""
                class="h-9 w-9 shrink-0 rounded-md border border-[var(--border)] bg-[var(--input)] object-cover"
              />
              <div v-else class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md border border-[var(--border)] bg-[var(--input)] text-[color:var(--tx-muted)]">
                <svg viewBox="0 0 16 16" class="h-5 w-5 fill-current"><path d="M1 7.775V2.75C1 1.784 1.784 1 2.75 1h5.025c.464 0 .91.184 1.238.513l6.25 6.25a1.75 1.75 0 0 1 0 2.474l-5.026 5.026a1.75 1.75 0 0 1-2.474 0l-6.25-6.25A1.752 1.752 0 0 1 1 7.775Zm1.5 0c0 .066.026.13.073.177l6.25 6.25a.25.25 0 0 0 .354 0l5.025-5.025a.25.25 0 0 0 0-.354l-6.25-6.25a.25.25 0 0 0-.177-.073H2.75a.25.25 0 0 0-.25.25ZM6 5a1 1 0 1 0 0 2 1 1 0 0 0 0-2Z"/></svg>
              </div>
              <button
                type="button"
                class="flex-1 rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1.5 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                @click="pickPackIcon(packId)"
              >
                {{ t("dev.changeIcon") }}
              </button>
              <span v-if="activeBanner" class="inline-block h-9 w-24 shrink-0 rounded-md border border-[var(--border)] object-cover" :style="`background-image:url('${activeBanner}');background-size:cover;background-position:center`"></span>
              <button
                type="button"
                class="flex-1 rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1.5 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                @click="pickPackBanner(packId)"
              >
                {{ activeBanner ? t("dev.changeBanner") : t("dev.setBanner") }}
              </button>
            </div>
          </div>

          <div>
            <label class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("pack.versionMc") }}</label>
            <div ref="editVerMcBox" class="relative">
              <button
                type="button"
                class="flex w-full items-center justify-between gap-2 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                @click="editVerMcOpen = !editVerMcOpen"
              >
                <span class="truncate">{{ editVerMc || t("pack.versionPick") }}</span>
                <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 shrink-0 fill-[var(--tx-muted)] transition-transform" :class="editVerMcOpen ? 'rotate-180' : ''"><path d="M4.22 6.22a.75.75 0 0 1 1.06 0L8 8.94l2.72-2.72a.75.75 0 1 1 1.06 1.06l-3.25 3.25a.75.75 0 0 1-1.06 0L4.22 7.28a.75.75 0 0 1 0-1.06Z"/></svg>
              </button>
              <div v-if="editVerMcOpen" class="absolute left-0 right-0 top-full z-50 mt-1 overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm shadow-2xl">
                <input v-model="editVerMcQuery" class="w-full border-b border-[var(--border)] bg-[var(--input)] px-3 py-1.5 text-xs outline-none placeholder:text-[var(--tx-muted)]" :placeholder="t('pack.versionSearch')" />
                <div class="max-h-52 overflow-y-auto py-1">
                  <button
                    v-for="v in editVerMcList"
                    :key="v.id"
                    type="button"
                    class="flex w-full items-center gap-2 px-3 py-1 text-left text-xs transition-colors hover:bg-[var(--hover)]"
                    :class="editVerMc === v.id ? 'text-[var(--accent)]' : 'text-[color:var(--tx)]'"
                    @click="chooseEditVerMc(v.id)"
                  >
                    <span class="min-w-0 flex-1 truncate">{{ v.id }}</span>
                    <span v-if="v.kind === 'snapshot'" class="rounded border border-[#9e6a03]/40 bg-[#9e6a03]/10 px-1.5 text-[11px] text-[#d29922]">{{ t("pack.snapshot") }}</span>
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div>
            <label class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("pack.versionLoader") }}</label>
            <div class="flex flex-wrap gap-1.5">
              <button
                v-for="l in CREATE_LOADERS"
                :key="l"
                type="button"
                class="rounded-md border px-2.5 py-1 text-xs font-medium capitalize transition-colors"
                :class="editVerLoader === l
                  ? 'border-[color-mix(in_srgb,var(--accent)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] text-[var(--accent)]'
                  : 'border-[var(--border)] bg-[var(--input)] text-[color:var(--tx)] hover:bg-[var(--hover)]'"
                @click="editVerLoader = l"
              >
                {{ l === "vanilla" ? t("pack.vanilla") : l }}
              </button>
            </div>
          </div>

          <div v-if="editVerLoader !== 'vanilla'">
            <label class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("pack.versionLoaderVer") }}</label>
            <div ref="editVerLvBox" class="relative">
              <button
                type="button"
                class="flex w-full items-center justify-between gap-2 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-2 text-xs text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                @click="editVerLvOpen = !editVerLvOpen"
              >
                <span class="truncate">{{ editVerLv || t("mods.createLatest") }}</span>
                <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 shrink-0 fill-[var(--tx-muted)] transition-transform" :class="editVerLvOpen ? 'rotate-180' : ''"><path d="M4.22 6.22a.75.75 0 0 1 1.06 0L8 8.94l2.72-2.72a.75.75 0 1 1 1.06 1.06l-3.25 3.25a.75.75 0 0 1-1.06 0L4.22 7.28a.75.75 0 0 1 0-1.06Z"/></svg>
              </button>
              <div v-if="editVerLvOpen" class="absolute left-0 right-0 top-full z-50 mt-1 max-h-52 overflow-y-auto rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-sm py-1 shadow-2xl">
                <button
                  type="button"
                  class="flex w-full items-center justify-between px-3 py-1 text-left text-xs transition-colors hover:bg-[var(--hover)]"
                  :class="editVerLv === '' ? 'text-[var(--accent)]' : 'text-[color:var(--tx)]'"
                  @click="chooseEditVerLoaderVersion('')"
                >
                  <span>{{ t("mods.createLatest") }}</span>
                  <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M12.78 4.22a.75.75 0 0 1 0 1.06l-5.78 5.78a.75.75 0 0 1-1.06 0l-2.5-2.5a.75.75 0 1 1 1.06-1.06L6.5 9.44l5.22-5.22a.75.75 0 0 1 1.06 0Z"/></svg>
                </button>
                <div v-if="!editVerLoaderVersions.length" class="px-3 py-1.5 text-[11px] text-[color:var(--tx-muted)]">{{ t("mods.createLvNone") }}</div>
                <button
                  v-for="v in editVerLoaderVersions"
                  :key="v"
                  type="button"
                  class="flex w-full items-center justify-between gap-2 px-3 py-1 text-left text-xs transition-colors hover:bg-[var(--hover)]"
                  :class="editVerLv === v ? 'text-[var(--accent)]' : 'text-[color:var(--tx)]'"
                  @click="chooseEditVerLoaderVersion(v)"
                >
                  <span class="min-w-0 truncate">{{ v }}</span>
                  <svg v-if="editVerLv === v" viewBox="0 0 16 16" class="h-3.5 w-3.5 shrink-0 fill-current"><path d="M12.78 4.22a.75.75 0 0 1 0 1.06l-5.78 5.78a.75.75 0 0 1-1.06 0l-2.5-2.5a.75.75 0 1 1 1.06-1.06L6.5 9.44l5.22-5.22a.75.75 0 0 1 1.06 0Z"/></svg>
                </button>
              </div>
            </div>
          </div>

          <div class="flex items-center justify-end gap-2 pt-1">
            <button
              type="button"
              class="rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-1.5 text-xs font-medium text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)]"
              @click="editVerOpen = false"
            >
              {{ t("files.cancel") }}
            </button>
            <button
              type="button"
              class="flex items-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_45%,transparent)] bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_22%,transparent)] disabled:opacity-50"
              :disabled="editVerBusy"
              @click="saveEditVersion"
            >
              <svg v-if="editVerBusy" viewBox="0 0 16 16" class="h-3.5 w-3.5 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
              {{ t("pack.versionSave") }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Модалка: выбор папок и файлов для экспорта сборки -->
    <div v-if="exportOpen" class="fixed inset-0 z-[60] flex items-center justify-center bg-black/40 p-4" @click.self="exportOpen = false">
      <div class="flex max-h-[80vh] w-full max-w-xl flex-col overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--panel)] shadow-2xl">
        <div class="flex items-center justify-between gap-2 border-b border-[var(--border)] px-4 py-3">
          <h3 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ authorImportMode ? t("author.importTitle") : exportFormat === "author" ? t("pack.exportAuthorTitle") : t("pack.exportTitle") }}</h3>
          <button
            type="button"
            class="rounded-md p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
            @click="exportOpen = false"
          >
            <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
          </button>
        </div>
        <div class="flex items-center justify-between gap-2 border-b border-[var(--border)] px-4 py-2">
          <p class="text-xs text-[color:var(--tx-muted)]">
            {{ authorImportMode ? t("author.importHint") : exportFormat === "curseforge" ? t("pack.exportFormatCurseforge") : exportFormat === "author" ? t("pack.exportAuthorHint") : t("pack.exportFormatMrpack") }}
          </p>
          <button
            v-if="exportFormat !== 'author'"
            type="button"
            class="text-xs font-medium text-[var(--accent)] transition-colors hover:underline disabled:opacity-50"
            :disabled="exportLoading"
            @click="toggleExportAll"
          >
            {{ exportAllChecked ? t("pack.exportNone") : t("pack.exportAll") }}
          </button>
        </div>
        <div v-if="exportFormat !== 'author'" class="grid grid-cols-2 gap-2 border-b border-[var(--border)] px-4 py-2">
          <label class="block">
            <span class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportNameLabel") }}</span>
            <input
              v-model="exportName"
              class="w-full rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1.5 text-xs text-[color:var(--tx)] outline-none transition-colors focus:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]"
              :placeholder="t('pack.exportNamePlaceholder')"
            />
          </label>
          <label class="block">
            <span class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportVersionNumLabel") }}</span>
            <input
              v-model="exportVersionNum"
              class="w-full rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1.5 text-xs text-[color:var(--tx)] outline-none transition-colors focus:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]"
              :placeholder="t('pack.exportVersionNumPlaceholder')"
            />
          </label>
        </div>
        <div class="min-h-0 flex-1 overflow-y-auto px-2 py-1">
          <template v-if="exportFormat === 'author'">
            <div class="space-y-3 px-2 py-2">
              <div v-if="authorImportMode" class="flex items-center gap-2 rounded-md border border-[var(--border)] bg-[var(--bg-30)] px-2 py-1.5">
                <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 shrink-0 fill-[var(--tx-muted)]"><path d="M9 1H4.5A1.5 1.5 0 0 0 3 2.5v11A1.5 1.5 0 0 0 4.5 15h7A1.5 1.5 0 0 0 13 13.5V5l-4-4Z"/></svg>
                <span class="min-w-0 flex-1 truncate font-mono text-xs text-[color:var(--tx)]">{{ authorImportFile || t("author.noFile") }}</span>
              </div>
              <div class="grid grid-cols-2 gap-2">
                <label :class="authorImportMode ? 'col-span-2' : ''" class="block">
                  <span class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorName") }}</span>
                  <input v-model="authorName" class="w-full rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1.5 text-xs text-[color:var(--tx)] outline-none transition-colors focus:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]" />
                </label>
                <label v-if="!authorImportMode" class="block">
                  <span class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorAuthor") }}</span>
                  <input v-model="authorAuthor" class="w-full rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1.5 text-xs text-[color:var(--tx)] outline-none transition-colors focus:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]" :placeholder="t('pack.exportAuthorAuthorPh')" />
                </label>
              </div>
              <label class="block">
                <span class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorDesc") }}</span>
                <textarea v-model="authorDesc" rows="2" class="w-full resize-none rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1.5 text-xs text-[color:var(--tx)] outline-none transition-colors focus:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]" :placeholder="t('pack.exportAuthorDescPh')"></textarea>
              </label>
              <div v-if="authorImportMode" class="grid grid-cols-2 gap-2">
                <label class="block">
                  <span class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorIcon") }}</span>
                  <input v-model="authorIcon" type="text" class="w-full rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1.5 text-xs text-[color:var(--tx)] outline-none transition-colors focus:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]" :placeholder="t('pack.exportAuthorIconPh')" />
                </label>
                <label class="block">
                  <span class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorBanner") }}</span>
                  <input v-model="authorBanner" type="text" class="w-full rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1.5 text-xs text-[color:var(--tx)] outline-none transition-colors focus:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]" :placeholder="t('pack.exportAuthorBannerPh')" />
                </label>
              </div>
              <div class="grid grid-cols-2 gap-2">
                <label class="block">
                  <span class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorBoosty") }}</span>
                  <input v-model="authorBoosty" class="w-full rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1.5 text-xs text-[color:var(--tx)] outline-none transition-colors focus:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]" :placeholder="t('pack.exportAuthorBoostyPh')" />
                </label>
                <label class="flex items-end gap-2 pb-1">
                  <span class="w-full">
                    <span class="mb-1 block text-xs font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorMinRam") }}</span>
                    <span class="flex items-center gap-2">
                      <input v-model="authorMinRam" type="checkbox" class="h-4 w-4 accent-[var(--accent)]" />
                      <input v-if="authorMinRam" v-model.number="authorMinRamMb" type="number" min="1" class="w-20 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1.5 text-xs text-[color:var(--tx)] outline-none focus:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]" />
                      <span v-else class="text-[11px] text-[color:var(--tx-muted)]">—</span>
                    </span>
                  </span>
                </label>
              </div>

              <div class="rounded-lg border border-[var(--border)] bg-[var(--bg-30)] p-2">
                <div class="mb-1.5 flex items-center justify-between px-1">
                  <span class="flex items-center gap-1.5 text-xs font-medium text-[color:var(--tx-muted)]">
                    <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M3 1.5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-9a2 2 0 0 0-2-2ZM1.5 4.5H14.5v1.5H1.5ZM1.5 8H14.5v1.25H1.5Zm0 3.25H7v1.5H1.5A.5.5 0 0 1 1 12.25v-1ZM8.5 12.75v-1.5h6v1.5A.5.5 0 0 1 14.5 13h-5a1 1 0 0 1-1-.25ZM2 5.75a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Zm3 0a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0ZM2 9.25a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Zm3 0a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Z"/></svg>
                    {{ t("pack.exportAuthorServers") }}
                  </span>
                  <div class="flex items-center gap-2">
                    <span class="text-[11px] tabular-nums text-[color:var(--tx-muted)]">{{ authorServers.length }}/{{ AUTHOR_MAX_SERVERS }}</span>
                    <button type="button" class="flex h-5 w-5 items-center justify-center rounded-md border border-[var(--border)] text-xs leading-none text-[var(--accent)] transition-colors hover:bg-[var(--input-50)] disabled:opacity-30 disabled:hover:bg-transparent" :disabled="authorServers.length >= AUTHOR_MAX_SERVERS" @click="addAuthorServer" title="+">+</button>
                  </div>
                </div>
                <div v-for="(_, i) in authorServers" :key="i" class="mb-1 flex items-center gap-1 rounded-md bg-[var(--panel)] p-1 last:mb-0">
                  <input v-model="authorServers[i].name" class="min-w-0 flex-1 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1.5 text-xs text-[color:var(--tx)] outline-none transition-colors focus:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]" :placeholder="t('pack.exportServerNamePh')" />
                  <input v-model="authorServers[i].ip" class="w-[7rem] rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1.5 font-mono text-xs text-[color:var(--tx)] outline-none transition-colors focus:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]" :placeholder="t('pack.exportServerIpPh')" />
                  <div class="relative">
                    <input v-model.number="authorServers[i].port" type="number" class="w-16 rounded-md border border-[var(--border)] bg-[var(--input)] px-1.5 py-1.5 font-mono text-xs text-[color:var(--tx)] outline-none transition-colors focus:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]" :placeholder="t('pack.exportServerPortPh')" />
                  </div>
                  <button type="button" class="flex h-6 w-6 shrink-0 items-center justify-center rounded-md p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--input-50)] hover:text-[#f85149]" :title="t('files.remove')" @click="removeAuthorServer(i)">
                    <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
                  </button>
                </div>
              </div>

              <div class="rounded-lg border border-[var(--border)] bg-[var(--bg-30)] p-2">
                <div class="mb-1.5 flex items-center justify-between px-1">
                  <span class="flex items-center gap-1.5 text-xs font-medium text-[color:var(--tx-muted)]">
                    <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M8 1a3 3 0 1 0 0 6 3 3 0 0 0 0-6Zm.75 7h-1.5A5.25 5.25 0 0 0 2 13.25c0 .414.336.75.75.75h10.5a.75.75 0 0 0 .75-.75A5.25 5.25 0 0 0 8.75 8Z"/></svg>
                    {{ t("pack.exportAuthorSocials") }}
                  </span>
                  <div class="flex items-center gap-2">
                    <span class="text-[11px] tabular-nums text-[color:var(--tx-muted)]">{{ authorSocials.length }}/{{ AUTHOR_MAX_SOCIALS }}</span>
                    <button type="button" class="flex h-5 w-5 items-center justify-center rounded-md border border-[var(--border)] text-xs leading-none text-[var(--accent)] transition-colors hover:bg-[var(--input-50)] disabled:opacity-30 disabled:hover:bg-transparent" :disabled="authorSocials.length >= AUTHOR_MAX_SOCIALS" @click="addAuthorSocial" title="+">+</button>
                  </div>
                </div>
                <div v-for="(_, i) in authorSocials" :key="i" class="mb-1 flex items-center gap-1 rounded-md bg-[var(--panel)] p-1 last:mb-0">
                  <input v-model="authorSocials[i].name" class="min-w-0 flex-1 rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1.5 text-xs text-[color:var(--tx)] outline-none transition-colors focus:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]" :placeholder="t('pack.exportSocialNamePh')" />
                  <input v-model="authorSocials[i].url" class="w-[9rem] rounded-md border border-[var(--border)] bg-[var(--input)] px-2 py-1.5 font-mono text-xs text-[color:var(--tx)] outline-none transition-colors focus:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]" :placeholder="t('pack.exportSocialUrlPh')" />
                  <div class="relative shrink-0">
                    <input v-model="authorSocials[i].color" class="w-16 rounded-md border border-[var(--border)] bg-[var(--input)] pl-6 py-1.5 font-mono text-xs text-[color:var(--tx)] outline-none transition-colors focus:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]" :placeholder="t('pack.exportSocialColorPh')" />
                    <span class="pointer-events-none absolute left-1.5 top-1/2 h-3 w-3 -translate-y-1/2 rounded-full border border-black/20" :style="{ background: themePreview(authorSocials[i].color) }"></span>
                  </div>
                  <button type="button" class="flex h-6 w-6 shrink-0 items-center justify-center rounded-md p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--input-50)] hover:text-[#f85149]" :title="t('files.remove')" @click="removeAuthorSocial(i)">
                    <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
                  </button>
                </div>
              </div>

              <div class="rounded-lg border border-[var(--border)] bg-[var(--bg-30)] p-2">
                <div class="mb-1.5 flex items-center justify-between px-1">
                  <span class="flex items-center gap-1.5 text-xs font-medium text-[color:var(--tx-muted)]">
                    <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current"><path d="M8 1a3 3 0 1 1 0 6 3 3 0 0 1 0-6Zm.75 7h-1.5A5.25 5.25 0 0 0 2 13.25c0 .414.336.75.75.75h10.5a.75.75 0 0 0 .75-.75A5.25 5.25 0 0 0 8.75 8ZM8 15a7 7 0 1 1 7-7 7 7 0 0 1-7 7Zm0-1.5a5.5 5.5 0 1 0 0-11 5.5 5.5 0 0 0 0 11Z"/></svg>
                    {{ t("pack.exportAuthorTheme") }}
                  </span>
                  <span class="text-[11px] text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorThemeAuto") }}</span>
                </div>
                <div class="mb-2 flex items-center gap-1.5 rounded-md bg-[var(--panel)] p-1.5">
                  <div class="relative shrink-0">
                    <input v-model="authorAccent" class="w-24 rounded-md border border-[var(--border)] bg-[var(--input)] pl-6 py-1.5 font-mono text-xs text-[color:var(--tx)] outline-none transition-colors focus:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]" placeholder="#rrggbb" @input="applyAuthorAccent" @change="applyAuthorAccent" />
                    <label class="absolute left-1.5 top-1/2 block h-3.5 w-3.5 -translate-y-1/2 cursor-pointer overflow-hidden rounded-full border border-black/25" :style="{ background: themePreview(authorAccent) }" :title="t('pack.exportAuthorAccentPicker')">
                      <input type="color" class="pointer-events-none absolute -left-2 -top-2 h-8 w-8 opacity-0" :value="themePreview(authorAccent) === '#000000' ? '#000000' : authorAccent" @input="applyAuthorAccentColor" />
                    </label>
                  </div>
                  <span class="min-w-0 flex-1 text-[11px] leading-tight text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorAccentHint") }}</span>
                </div>
                <div class="grid grid-cols-2 gap-1.5">
                  <label v-for="f in authorThemeFields" :key="f.key" class="rounded-md bg-[var(--panel)] px-1.5 py-1">
                    <div class="flex items-center gap-1.5">
                      <span class="pointer-events-none h-3.5 w-3.5 shrink-0 rounded-full border border-black/25" :style="{ background: themePreview(authorTheme[f.key]) }"></span>
                      <input v-model="authorTheme[f.key]" class="min-w-0 flex-1 rounded-md border border-transparent bg-transparent py-0.5 font-mono text-[11px] text-[color:var(--tx)] outline-none transition-colors focus:border-[color-mix(in_srgb,var(--accent)_60%,transparent)]" placeholder="#rrggbb" />
                    </div>
                    <span class="mt-0.5 block pl-[1.375rem] text-[10px] leading-tight text-[color:var(--tx-muted)]">{{ t(f.cap) }}</span>
                  </label>
                </div>
              </div>
            </div>
            <div v-if="!authorImportMode" class="flex items-center gap-2 border-t border-[var(--border)] px-2 pt-2 pb-1">
              <span class="shrink-0 text-xs font-medium text-[color:var(--tx-muted)]">{{ t("pack.exportAuthorFiles") }}</span>
              <span class="shrink-0 rounded bg-[var(--input-50)] px-1.5 py-0.5 text-[11px] font-bold tabular-nums">{{ exportSelected.size }}</span>
              <span class="h-px flex-1 bg-[var(--border)]"></span>
              <button type="button" class="text-xs font-medium text-[var(--accent)] hover:underline disabled:opacity-50" :disabled="exportLoading" @click="toggleExportAll">
                {{ exportAllChecked ? t("pack.exportNone") : t("pack.exportAll") }}
              </button>
            </div>
          </template>
          <div v-if="!authorImportMode && exportLoading" class="flex items-center justify-center gap-2 py-8 text-xs text-[color:var(--tx-muted)]">
            <svg viewBox="0 0 16 16" class="h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
            {{ t("pack.exportLoading") }}
          </div>
          <div v-else-if="!authorImportMode && exportItems.length === 0" class="px-2 py-8 text-center text-xs text-[color:var(--tx-muted)]">{{ t("pack.exportEmpty") }}</div>
          <div v-else-if="!authorImportMode">
            <div
              v-for="row in exportVisibleRows"
              :key="row.it.path"
              class="flex cursor-pointer items-center gap-1 rounded-md py-1 pr-2 transition-colors hover:bg-[var(--hover)]"
              :style="{ paddingLeft: `${row.depth * 16 + 4}px` }"
            >
              <button
                v-if="row.it.isDir"
                type="button"
                class="flex h-4 w-4 shrink-0 items-center justify-center rounded text-[color:var(--tx-muted)] transition-colors hover:text-[color:var(--tx)]"
                @click.stop="toggleExportExpand(row.it.path)"
              >
                <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current transition-transform" :class="exportExpanded.has(row.it.path) ? 'rotate-90' : ''"><path d="M6 4l4 4-4 4V4Z"/></svg>
              </button>
              <span v-else class="w-4 shrink-0"></span>
              <input
                type="checkbox"
                class="h-3.5 w-3.5 shrink-0 accent-[var(--accent)]"
                :checked="exportSelected.has(row.it.path)"
                :indeterminate="exportSelectedCount(row.it.path).selected > 0 && exportSelectedCount(row.it.path).selected < exportSelectedCount(row.it.path).total"
                @change="toggleExport(row.it.path)"
              />
              <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-[var(--tx-muted)]">
                <path v-if="row.it.isDir" d="M1.75 1A1.75 1.75 0 0 0 0 2.75v10.5C0 14.216.784 15 1.75 15h12.5A1.75 1.75 0 0 0 16 13.25v-8.5A1.75 1.75 0 0 0 14.25 3H7.5a.25.25 0 0 1-.2-.1l-.9-1.2A1.75 1.75 0 0 0 5 1H1.75Z"/>
                <path v-else d="M9 1H4.5A1.5 1.5 0 0 0 3 2.5v11A1.5 1.5 0 0 0 4.5 15h7A1.5 1.5 0 0 0 13 13.5V5l-4-4Z"/>
              </svg>
              <span class="min-w-0 flex-1 truncate text-xs text-[color:var(--tx)]" :class="!row.it.defaultIncluded ? 'opacity-60' : ''">{{ row.it.path.split("/").pop() }}</span>
              <span v-if="row.it.defaultIncluded" class="shrink-0 text-[11px] tabular-nums text-[color:var(--tx-muted)]">{{ formatBytes(row.it.size) }}</span>
              <span v-else class="shrink-0 rounded border border-[var(--border)] px-1.5 py-0.5 text-[10px] uppercase text-[color:var(--tx-muted)]">{{ t("pack.exportExcluded") }}</span>
            </div>
          </div>
        </div>
        <div class="flex items-center justify-end gap-2 border-t border-[var(--border)] px-4 py-3">
          <label
            v-if="monoProfile && exportFormat === 'mrpack'"
            class="mr-auto flex cursor-pointer items-center gap-1.5 text-xs text-[color:var(--tx-muted)]"
          >
            <input type="checkbox" v-model="exportUpload" class="h-3.5 w-3.5 accent-[var(--accent)]" />
            {{ t("pack.uploadMono") }}
          </label>
          <button
            type="button"
            class="rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-1.5 text-xs font-medium text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)]"
            @click="exportOpen = false"
          >
            {{ t("files.cancel") }}
          </button>
          <button
            type="button"
            class="flex items-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_45%,transparent)] bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_22%,transparent)] disabled:opacity-50"
            :disabled="exportBusy || exportLoading || (!authorImportMode && exportSelected.size === 0)"
            @click="authorImportMode ? doAuthorImport() : exportFormat === 'author' ? doAuthorExport() : doExport()"
          >
            <svg v-if="exportBusy" viewBox="0 0 16 16" class="h-3.5 w-3.5 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
            {{ authorImportMode ? t("author.upload") : exportFormat === "author" ? t("pack.exportAuthorBtn") : t("pack.exportBtn") }}
          </button>
        </div>
      </div>
    </div>

    <!-- Окно поиска файлов: Modrinth / CurseForge
         (в обычном режиме — плавающая панель, в отдельном окне — на весь экран) -->
    <div
      v-if="searchOpen"
      class="fixed z-50"
      :class="isSearchWin ? 'inset-0' : ''"
      :style="isSearchWin ? undefined : searchWinStyle"
    >
      <div
        class="flex flex-col overflow-hidden bg-[var(--panel)]"
        :class="isSearchWin
          ? 'h-full w-full'
          : 'max-h-[85vh] w-[720px] max-w-[92vw] rounded-xl border border-[var(--border)] shadow-2xl'"
      >
        <div
          class="flex shrink-0 items-center justify-between gap-2 border-b border-[var(--border)] bg-[var(--input-50)] px-3 py-2.5"
          :class="isSearchWin ? '' : 'cursor-move'"
          @pointerdown="dragSearchWin"
        >
          <div class="flex shrink-0 items-center gap-1 rounded-md border border-[var(--border)] bg-[var(--bg)] p-0.5">
            <button
              type="button"
              class="flex items-center gap-1.5 rounded px-3 py-1.5 text-xs font-semibold transition-colors"
              :class="searchService === 'modrinth'
                ? 'bg-[var(--accent)] ring-2 ring-[var(--accent)] ring-offset-2 ring-offset-[var(--panel)]'
                : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
              @click="switchSearchService('modrinth')"
            >
              <svg viewBox="0 0 24 24" class="h-3.5 w-3.5 shrink-0" :title="t('mods.serviceModrinth')"><path fill="#00AF5C" d="M12.252.004a11.78 11.768 0 0 0-8.92 3.73 11 10.999 0 0 0-2.17 3.11 11.37 11.359 0 0 0-1.16 5.169c0 1.42.17 2.5.6 3.77.24.759.77 1.899 1.17 2.529a12.3 12.298 0 0 0 8.85 5.639c.44.05 2.54.07 2.76.02.2-.04.22.1-.26-1.7l-.36-1.37-1.01-.06a8.5 8.489 0 0 1-5.18-1.8 5.34 5.34 0 0 1-1.3-1.26c0-.05.34-.28.74-.5a37.572 37.545 0 0 1 2.88-1.629c.03 0 .5.45 1.06.98l1 .97 2.07-.43 2.06-.43 1.47-1.47c.8-.8 1.48-1.5 1.48-1.52 0-.09-.42-1.63-.46-1.7-.04-.06-.2-.03-1.02.18-.53.13-1.2.3-1.45.4l-.48.15-.53.53-.53.53-.93.1-.93.07-.52-.5a2.7 2.7 0 0 1-.96-1.7l-.13-.6.43-.57c.68-.9.68-.9 1.46-1.1.4-.1.65-.2.83-.33.13-.099.65-.579 1.14-1.069l.9-.9-.7-.7-.7-.7-1.95.54c-1.07.3-1.96.53-1.97.53-.03 0-2.23 2.48-2.63 2.97l-.29.35.28 1.03c.16.56.3 1.16.31 1.34l.03.3-.34.23c-.37.23-2.22 1.3-2.84 1.63-.36.2-.37.2-.44.1-.08-.1-.23-.6-.32-1.03-.18-.86-.17-2.75.02-3.73a8.84 8.839 0 0 1 7.9-6.93c.43-.03.77-.08.78-.1.06-.17.5-2.999.47-3.039-.01-.02-.1-.02-.2-.03Zm3.68.67c-.2 0-.3.1-.37.38-.06.23-.46 2.42-.46 2.52 0 .04.1.11.22.16a8.51 8.499 0 0 1 2.99 2 8.38 8.379 0 0 1 2.16 3.449 6.9 6.9 0 0 1 .4 2.8c0 1.07 0 1.27-.1 1.73a9.37 9.369 0 0 1-1.76 3.769c-.32.4-.98 1.06-1.37 1.38-.38.32-1.54 1.1-1.7 1.14-.1.03-.1.06-.07.26.03.18.64 2.56.7 2.78l.06.06a12.07 12.058 0 0 0 7.27-9.4c.13-.77.13-2.58 0-3.4a11.96 11.948 0 0 0-5.73-8.578c-.7-.42-2.05-1.06-2.25-1.06Z"/></svg>
              {{ t("mods.serviceModrinth") }}
            </button>
            <button
              v-if="modSearchKind !== 'datapack'"
              type="button"
              class="flex items-center gap-1.5 rounded px-3 py-1.5 text-xs font-semibold transition-colors"
              :class="searchService === 'curseforge'
                ? 'bg-[var(--accent)] ring-2 ring-[var(--accent)] ring-offset-2 ring-offset-[var(--panel)]'
                : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
              @click="switchSearchService('curseforge')"
            >
              <svg viewBox="0 0 24 24" class="h-3.5 w-3.5 shrink-0" :title="t('mods.serviceCurseforge')"><path fill="#F16436" d="M18.326 9.2145S23.2261 8.4418 24 6.1882h-7.5066V4.4H0l2.0318 2.3576V9.173s5.1267-.2665 7.1098 1.2372c2.7146 2.516-3.053 5.917-3.053 5.917L5.0995 19.6c1.5465-1.4726 4.494-3.3775 9.8983-3.2857-2.0565.65-4.1245 1.6651-5.7344 3.2857h10.9248l-1.0288-3.2726s-7.918-4.6688-.8336-7.1127z"/></svg>
              {{ t("mods.serviceCurseforge") }}
            </button>
          </div>
          <h3 class="min-w-0 flex-1 truncate text-sm font-semibold text-[color:var(--tx-strong)]">
            {{ searchTitle }}
          </h3>
          <button
            type="button"
            class="rounded-md p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
            @click="closeSearch"
          >
            <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
          </button>
        </div>
        <div v-if="searchService === 'curseforge' && !curseKeyOk" class="border-b border-[var(--border)] px-4 py-2.5">
          <p class="text-xs text-[color:var(--tx-muted)]">{{ t("curse.noKey") }}</p>
        </div>
        <div class="flex shrink-0 items-center gap-2 border-b border-[var(--border)] px-4 py-3">
          <div class="relative min-w-0 flex-1">
            <svg viewBox="0 0 16 16" class="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 fill-[var(--tx-muted)]">
              <path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"/>
            </svg>
            <input
              v-model="searchInput"
              type="text"
              :placeholder="searchService === 'modrinth' ? t('mods.searchPlaceholder') : t('curse.searchPlaceholder')"
              class="w-full rounded-md border border-[var(--border)] bg-[var(--bg)] py-1.5 pl-8 pr-3 text-xs text-[color:var(--tx)] placeholder-[var(--tx-muted)] outline-none transition-colors focus:border-[var(--accent)]"
              @keydown.enter="doSearch"
            />
          </div>
          <button
            type="button"
            class="flex shrink-0 items-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
            :disabled="searchLoading || !searchInput.trim()"
            @click="doSearch"
          >
            <svg v-if="searchLoading" viewBox="0 0 16 16" class="h-3.5 w-3.5 animate-spin fill-current">
              <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
            </svg>
            <svg v-else viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
              <path d="M10.68 11.74a6 6 0 0 1-7.922-8.982 6 6 0 0 1 8.982 7.922l3.04 3.04a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215ZM11.5 7a4.499 4.499 0 1 0-8.997 0A4.499 4.499 0 0 0 11.5 7Z"/>
            </svg>
            {{ searchService === 'modrinth' ? t("mods.search") : t("curse.search") }}
          </button>
        </div>
        <div v-if="searchService === 'curseforge'" class="flex shrink-0 flex-wrap items-center gap-2 border-b border-[var(--border)] px-4 py-2">
          <FilterSelect
            v-model="curseLoaderSel"
            :options="curseLoaderOptions"
            :placeholder="t('curse.fLoader')"
            :multiple="false"
            @change="searchCurse()"
          />
          <FilterSelect
            v-model="curseCatSel"
            :options="curseCatOptions"
            :placeholder="t('curse.fCategory')"
            :multiple="false"
            @change="searchCurse()"
          />
          <FilterSelect
            v-model="curseVerSel"
            :options="versionOptions"
            :placeholder="t('curse.fVersion')"
            :multiple="false"
            @change="searchCurse()"
          />
          <FilterSelect
            v-model="curseSortSel"
            :options="curseSortOptions"
            :placeholder="t('curse.fSort')"
            :multiple="false"
            @change="searchCurse()"
          />
        </div>
        <div v-if="searchService === 'modrinth'" class="flex shrink-0 flex-wrap items-center gap-2 border-b border-[var(--border)] px-4 py-2">
          <FilterSelect
            v-if="modSearchKind === 'datapack'"
            v-model="modDatapackWorldSel"
            :options="worldOptions"
            :placeholder="t('mods.fWorld')"
            :multiple="false"
          />
          <FilterSelect
            v-model="modFilters.versions"
            :options="versionOptions"
            :placeholder="t('mods.fVersion')"
            @change="searchMods()"
          />
<FilterSelect
              v-model="modFilters.loaders"
              :options="loaderOptions"
              :placeholder="t('mods.fLoader')"
              @change="searchMods()"
            />
            <FilterSelect
              v-model="modVersionTypeSel"
              :options="versionTypeOptions"
              :placeholder="t('mods.fType')"
              :multiple="false"
              @change="searchMods()"
            />
          <FilterSelect
            v-model="modFilters.categories"
            :options="categoryOptions"
            :placeholder="t('mods.fCategory')"
            @change="searchMods()"
          />
          <FilterSelect
            v-model="modEnvSel"
            :options="envOptions"
            :placeholder="t('mods.fAny')"
            :multiple="false"
            @change="searchMods()"
          />
          <FilterSelect
            v-model="modSortSel"
            :options="sortSelectOptions"
            :placeholder="t('mods.fSort')"
            :multiple="false"
            @change="searchMods()"
          />
        </div>
        <div v-if="searchService === 'modrinth'" ref="modSearchBox" class="min-h-0 flex-1 overflow-y-auto p-4" @scroll.passive="onModSearchScroll">
          <div
            v-if="selModrinth.size > 0 && !modSearchLoading && !modDetail"
            class="mb-3 flex flex-wrap items-center gap-2 rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_8%,transparent)] px-3 py-2"
          >
            <span class="text-xs font-medium text-[var(--accent)]">
              {{ t("mods.selected", { n: selModrinth.size }) }}
            </span>
            <button
              type="button"
              class="flex shrink-0 items-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_25%,transparent)] disabled:opacity-50"
              :disabled="multiSelBusy || quickModBusy !== null || modInstallBusy !== null"
              @click="downloadSelectedMods"
            >
              <svg v-if="multiSelBusy" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
              </svg>
              <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
              </svg>
              {{ multiSelBusy ? t("mods.installingSel") : t("mods.downloadSel") }}
            </button>
            <button
              type="button"
              class="rounded-md px-1.5 py-0.5 text-xs text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
              :title="t('files.clear')"
              @click="clearSelAll"
            >
              ×
            </button>
          </div>
          <div v-if="modSearchErr" class="rounded-md border border-[var(--border)] bg-[var(--input-50)] p-6 text-center text-xs text-[color:var(--tx-muted)]">
            <p class="mb-2">{{ modSearchErr }}</p>
            <button type="button" class="text-[var(--accent)] hover:underline" @click="searchMods">{{ t("catalog.retry") }}</button>
          </div>
          <template v-else-if="modDetail">
            <button
              type="button"
              class="mb-3 flex items-center gap-1 text-xs text-[color:var(--tx-muted)] transition-colors hover:text-[var(--accent)]"
              @click="modDetail = null; modVersions = null"
            >
              <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M7.28 3.22a.75.75 0 0 1 0 1.06L3.56 8l3.72 3.72a.75.75 0 1 1-1.06 1.06l-4.25-4.25a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0Zm4 0a.75.75 0 0 1 0 1.06L7.56 8l3.72 3.72a.75.75 0 1 1-1.06 1.06l-4.25-4.25a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0Z"/></svg>
              {{ t("mods.back") }}
            </button>
            <div class="mb-3 flex items-center gap-3 rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2.5">
              <img v-if="modDetail.iconUrl" :src="searchIconUrl(modDetail.iconUrl)" :alt="modDetail.title" loading="lazy" class="h-11 w-11 shrink-0 rounded-md object-cover" />
              <div v-else class="flex h-11 w-11 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-xs text-[color:var(--tx-muted)]">
                {{ modDetail.title.slice(0, 2).toUpperCase() }}
              </div>
              <div class="min-w-0 flex-1">
                <h4 class="truncate text-sm font-semibold text-[color:var(--tx-strong)]">{{ modDetail.title }}</h4>
                <div class="mt-0.5 flex flex-wrap items-center gap-3 text-[11px] text-[color:var(--tx-muted)]">
                  <span>{{ t("mods.byAuthor", { author: modDetail.author }) }}</span>
                  <span class="flex items-center gap-1">
                    <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1.75a.75.75 0 0 0-1.5 0v9A2.25 2.25 0 0 0 2.5 13h12.75a.75.75 0 0 0 0-1.5H2.5a.75.75 0 0 1-.75-.75v-9Zm10.75 2.5a.75.75 0 0 0-1.5 0v5a.75.75 0 0 0 1.5 0v-5Zm-3 .75a.75.75 0 0 1 1.5 0v4.25a.75.75 0 0 1-1.5 0V5Zm-3 1.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3Z"/></svg>
                    {{ modDetail.downloads.toLocaleString() }}
                  </span>
                  <span v-if="modDetail.categories.length">{{ modDetail.categories.slice(0, 4).join(", ") }}</span>
                  <button
                    v-if="modDetail.slug"
                    type="button"
                    class="text-[var(--accent)] hover:underline"
                    @click="openExternal(`https://modrinth.com/mod/${modDetail!.slug}`)"
                  >
                    {{ t("mods.openPage") }}
                  </button>
                </div>
              </div>
              <button
                type="button"
                class="flex shrink-0 items-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                :disabled="quickModBusy !== null || modInstallBusy !== null"
                :title="t('mods.downloadHint')"
                @click="quickDownloadMod(modDetail, $event)"
              >
                <svg v-if="quickModBusy === modDetail.projectId" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                  <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                </svg>
                <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                  <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
                </svg>
                {{ t("mods.download") }}
              </button>
            </div>
            <!-- Вкладки как на Modrinth: описание / версии / галерея -->
            <div class="mb-3 flex shrink-0 items-center gap-1 border-b border-[var(--border)] pb-2">
              <button
                v-for="tb in modDetailTabs"
                :key="tb.kind"
                type="button"
                class="rounded-md px-3 py-1.5 text-xs font-medium transition-colors"
                :class="modDetailTab === tb.kind
                  ? 'bg-[var(--input)] text-[color:var(--tx-strong)]'
                  : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]'"
                @click="modDetailTab = tb.kind"
              >
                {{ t("mods.tab" + tb.kind) }}
              </button>
            </div>
            <div v-if="modDetailTab === 'about'" class="max-h-[46vh] overflow-y-auto rounded-md border border-[var(--border)] bg-[var(--bg)] px-4 py-3">
              <Markdown v-if="modDetail.body" :source="modDetail.body" />
              <p v-else class="py-6 text-center text-xs italic text-[color:var(--tx-muted)]">{{ t("mods.noAbout") }}</p>
            </div>
            <div v-else-if="modDetailTab === 'versions'">
              <div v-if="modVersions === null" class="flex items-center justify-center py-10 text-xs text-[color:var(--tx-muted)]">
                <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current">
                  <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                </svg>
                {{ t("mods.searching") }}
              </div>
              <div v-else-if="modVersions.length === 0" class="py-8 text-center text-xs text-[color:var(--tx-muted)]">{{ t("mods.noVersions") }}</div>
              <div v-else class="space-y-2">
                <div v-if="modVersionsRaw.length > 1" class="flex flex-wrap items-center gap-2">
                  <FilterSelect v-model="verFilterMcSel" :options="verFilterMcOptions" :placeholder="t('curse.fVersion')" :multiple="false" />
                  <FilterSelect v-model="verFilterLoaderSel" :options="verFilterLoaderOptions" :placeholder="t('curse.fLoader')" :multiple="false" />
                  <FilterSelect v-model="verFilterTypeSel" :options="verFilterTypeOptions" :placeholder="t('mods.fType')" :multiple="false" />
                </div>
                <p v-if="filteredModVersions.length === 0" class="py-6 text-center text-xs text-[color:var(--tx-muted)]">{{ t("mods.noVersions") }}</p>
                <div
                  v-for="v in filteredModVersions"
                  :key="v.id"
                  class="flex items-center gap-2 rounded-md border border-[var(--border)] bg-[var(--bg)] py-1.5 pl-2.5 pr-1.5"
                >
                  <span
                    class="h-2 w-2 shrink-0 rounded-full"
                    :style="{ backgroundColor: verTypeColor(v.versionType) }"
                    :title="t('mods.verType.' + v.versionType)"
                  ></span>
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-1.5">
                      <span class="truncate text-xs font-medium text-[color:var(--tx-strong)]">{{ v.name }}</span>
                      <span
                        v-if="status?.minecraft_version && v.gameVersions.includes(status.minecraft_version)"
                        class="shrink-0 rounded-full border border-[color-mix(in_srgb,var(--accent)_35%,transparent)] bg-[color-mix(in_srgb,var(--accent)_8%,transparent)] px-1.5 py-px text-[10px] font-medium text-[var(--accent)]"
                      >
                        {{ t("mods.versionMatch") }}
                      </span>
                    </div>
                    <div class="truncate text-[11px] text-[color:var(--tx-muted)]">
                      {{ v.loaders.map(cap).join(" · ") || "vanilla" }} · {{ v.gameVersions.slice(0, 2).join(", ") }} · {{ formatDate(v.datePublished) }}
                    </div>
                  </div>
                  <span class="shrink-0 rounded border border-[var(--border)] bg-[var(--input-50)] px-1.5 py-px font-mono text-[10px] text-[color:var(--tx-muted)]">{{ v.versionNumber }}</span>
                  <button
                    type="button"
                    class="flex shrink-0 items-center gap-1 rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-2 py-1 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                    :disabled="modInstallBusy !== null"
                    :title="v.versionNumber"
                    @click="installModVersion(v)"
                  >
                    <svg v-if="modInstallBusy === v.id" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                      <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                    </svg>
                    <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                      <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
                    </svg>
                  </button>
                </div>
              </div>
            </div>
            <div v-else>
              <div v-if="modDetail.gallery.length" class="grid grid-cols-2 gap-2">
                <img
                  v-for="g in modDetail.gallery"
                  :key="g.url"
                  :src="g.url"
                  :alt="g.title ?? ''"
                  loading="lazy"
                  class="h-32 w-full cursor-zoom-in rounded-md border border-[var(--border)] object-cover transition-transform hover:scale-[1.02]"
                  :title="g.title ?? undefined"
                  @click="openExternal(g.url)"
                />
              </div>
              <p v-else class="py-10 text-center text-xs italic text-[color:var(--tx-muted)]">{{ t("mods.noGallery") }}</p>
            </div>
          </template>
          <template v-else-if="modSearchLoading">
            <div class="flex items-center justify-center py-16 text-xs text-[color:var(--tx-muted)]">
              <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current">
                <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
              </svg>
              {{ t("mods.searching") }}
            </div>
          </template>
          <template v-else-if="modSearchResults.length === 0">
            <div class="py-16 text-center text-xs text-[color:var(--tx-muted)]">
              <p class="mb-3">{{ modSearchQuery ? t("mods.noResults") : t("mods.help") }}</p>
              <button
                v-if="modFiltersActive"
                type="button"
                class="inline-flex items-center gap-1.5 rounded-md border border-[var(--border)] bg-[var(--input)] px-3 py-1.5 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
                @click="resetModFiltersAndSearch"
              >
                <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1A1.75 1.75 0 0 0 0 2.75v3.5C0 7.216.784 8 1.75 8h3.5A1.75 1.75 0 0 0 7 6.25v-3.5A1.75 1.75 0 0 0 5.25 1h-3.5ZM1 2.75a.75.75 0 0 1 .75-.75h3.5a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-.75.75h-3.5a.75.75 0 0 1-.75-.75v-3.5Zm9-1.75A1.75 1.75 0 0 0 8.25 2.75v3.5A1.75 1.75 0 0 0 10 8h3.5A1.75 1.75 0 0 0 15.25 6.25v-3.5A1.75 1.75 0 0 0 13.5 1H10Zm-.75 1.75a.75.75 0 0 1 .75-.75h3.5a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-.75.75H10a.75.75 0 0 1-.75-.75v-3.5ZM1.75 9A1.75 1.75 0 0 0 0 10.75v3.5A1.75 1.75 0 0 0 1.75 16h3.5A1.75 1.75 0 0 0 7 14.25v-3.5A1.75 1.75 0 0 0 5.25 9h-3.5Zm-.75 1.75a.75.75 0 0 1 .75-.75h3.5a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-.75.75h-3.5a.75.75 0 0 1-.75-.75v-3.5ZM10 9A1.75 1.75 0 0 0 8.25 10.75v3.5A1.75 1.75 0 0 0 10 16h3.5A1.75 1.75 0 0 0 15.25 14.25v-3.5A1.75 1.75 0 0 0 13.5 9H10Zm-.75 1.75a.75.75 0 0 1 .75-.75h3.5a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-.75.75H10a.75.75 0 0 1-.75-.75v-3.5Z"/></svg>
                {{ t("mods.resetFilters") }}
              </button>
            </div>
          </template>
          <template v-else>
            <div class="space-y-2">
              <div
                v-for="p in modSearchResults"
                :key="p.projectId"
                class="flex cursor-pointer items-start gap-3 rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2.5 transition-colors hover:border-[color-mix(in_srgb,var(--accent)_50%,transparent)]"
                @click="openModDetail(p)"
              >
                <button
                  type="button"
                  class="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-md border transition-colors"
                  :class="selModrinth.has(p.projectId)
                    ? 'border-[var(--accent)] bg-[color-mix(in_srgb,var(--accent)_20%,transparent)]'
                    : 'border-[var(--border)] hover:border-[var(--tx-muted)]'"
                  :title="selModrinth.has(p.projectId) ? t('files.clear') : t('mods.selForDownload')"
                  @click.stop="toggleModrinthSel(p.projectId)"
                >
                  <svg v-if="selModrinth.has(p.projectId)" viewBox="0 0 16 16" class="h-3 w-3 fill-[var(--accent)]">
                    <path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/>
                  </svg>
                </button>
                <img v-if="p.iconUrl" :src="searchIconUrl(p.iconUrl)" alt="" loading="lazy" class="h-10 w-10 shrink-0 rounded-md object-cover" />
                <div v-else class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-[11px] text-[color:var(--tx-muted)]">
                  {{ p.title.slice(0, 2).toUpperCase() }}
                </div>
                <div class="min-w-0 flex-1">
                  <div class="flex flex-wrap items-center gap-x-2">
                    <svg viewBox="0 0 24 24" class="h-3 w-3 shrink-0 self-center" :title="t('mods.serviceModrinth')"><path fill="#00AF5C" d="M12.252.004a11.78 11.768 0 0 0-8.92 3.73 11 10.999 0 0 0-2.17 3.11 11.37 11.359 0 0 0-1.16 5.169c0 1.42.17 2.5.6 3.77.24.759.77 1.899 1.17 2.529a12.3 12.298 0 0 0 8.85 5.639c.44.05 2.54.07 2.76.02.2-.04.22.1-.26-1.7l-.36-1.37-1.01-.06a8.5 8.489 0 0 1-5.18-1.8 5.34 5.34 0 0 1-1.3-1.26c0-.05.34-.28.74-.5a37.572 37.545 0 0 1 2.88-1.629c.03 0 .5.45 1.06.98l1 .97 2.07-.43 2.06-.43 1.47-1.47c.8-.8 1.48-1.5 1.48-1.52 0-.09-.42-1.63-.46-1.7-.04-.06-.2-.03-1.02.18-.53.13-1.2.3-1.45.4l-.48.15-.53.53-.53.53-.93.1-.93.07-.52-.5a2.7 2.7 0 0 1-.96-1.7l-.13-.6.43-.57c.68-.9.68-.9 1.46-1.1.4-.1.65-.2.83-.33.13-.099.65-.579 1.14-1.069l.9-.9-.7-.7-.7-.7-1.95.54c-1.07.3-1.96.53-1.97.53-.03 0-2.23 2.48-2.63 2.97l-.29.35.28 1.03c.16.56.3 1.16.31 1.34l.03.3-.34.23c-.37.23-2.22 1.3-2.84 1.63-.36.2-.37.2-.44.1-.08-.1-.23-.6-.32-1.03-.18-.86-.17-2.75.02-3.73a8.84 8.839 0 0 1 7.9-6.93c.43-.03.77-.08.78-.1.06-.17.5-2.999.47-3.039-.01-.02-.1-.02-.2-.03Zm3.68.67c-.2 0-.3.1-.37.38-.06.23-.46 2.42-.46 2.52 0 .04.1.11.22.16a8.51 8.499 0 0 1 2.99 2 8.38 8.379 0 0 1 2.16 3.449 6.9 6.9 0 0 1 .4 2.8c0 1.07 0 1.27-.1 1.73a9.37 9.369 0 0 1-1.76 3.769c-.32.4-.98 1.06-1.37 1.38-.38.32-1.54 1.1-1.7 1.14-.1.03-.1.06-.07.26.03.18.64 2.56.7 2.78l.06.06a12.07 12.058 0 0 0 7.27-9.4c.13-.77.13-2.58 0-3.4a11.96 11.948 0 0 0-5.73-8.578c-.7-.42-2.05-1.06-2.25-1.06Z"/></svg>
                    <span class="truncate text-xs font-semibold text-[color:var(--tx-strong)]">{{ p.title }}</span>
                    <span class="text-[11px] text-[color:var(--tx-muted)]">{{ t("mods.byAuthor", { author: p.author }) }}</span>
                  </div>
                  <p class="mt-0.5 line-clamp-2 text-xs leading-snug text-[color:var(--tx-muted)]">{{ p.description }}</p>
                  <div class="mt-1 flex items-center gap-3 text-[11px] text-[color:var(--tx-muted)]">
                    <span class="flex items-center gap-1">
                      <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1.75a.75.75 0 0 0-1.5 0v9A2.25 2.25 0 0 0 2.5 13h12.75a.75.75 0 0 0 0-1.5H2.5a.75.75 0 0 1-.75-.75v-9Zm10.75 2.5a.75.75 0 0 0-1.5 0v5a.75.75 0 0 0 1.5 0v-5Zm-3 .75a.75.75 0 0 1 1.5 0v4.25a.75.75 0 0 1-1.5 0V5Zm-3 1.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3Z"/></svg>
                      {{ p.downloads.toLocaleString() }}
                    </span>
                    <span v-if="status?.minecraft_version">{{ status.minecraft_version }}</span>
                  </div>
                </div>
                <button
                  type="button"
                  class="flex shrink-0 items-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                  :disabled="quickModBusy !== null || modInstallBusy !== null || installedModrinthSlugs.has(p.slug)"
                  :title="t('mods.downloadHint')"
                  @click="quickDownloadMod(p, $event)"
                >
                  <svg v-if="quickModBusy === p.projectId" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                    <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                  </svg>
                  <svg v-else-if="installedModrinthSlugs.has(p.slug)" viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                    <path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/>
                  </svg>
                  <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                    <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
                  </svg>
                  {{ installedModrinthSlugs.has(p.slug) ? t("mods.installedBadge") : t("mods.download") }}
                </button>
                <svg viewBox="0 0 16 16" class="mt-1 h-3.5 w-3.5 shrink-0 fill-[var(--tx-muted)]"><path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"/></svg>
              </div>
            </div>
          </template>
          <div v-if="modSearchMore || modSearchMoreBusy" class="flex justify-center py-4">
            <svg v-if="modSearchMoreBusy" viewBox="0 0 16 16" class="h-4 w-4 animate-spin fill-[var(--tx-muted)]">
              <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
            </svg>
            <button
              v-else
              type="button"
              class="text-xs font-medium text-[var(--accent)] hover:underline"
              @click="loadMoreMods"
            >{{ t("mods.loadMore") }}</button>
          </div>
        </div>
        <div v-else class="min-h-0 flex-1 overflow-y-auto px-4 py-3">
          <div
            v-if="selCurse.size > 0 && !curseLoading"
            class="mb-3 flex flex-wrap items-center gap-2 rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_8%,transparent)] px-3 py-2"
          >
            <span class="text-xs font-medium text-[var(--accent)]">
              {{ t("mods.selected", { n: selCurse.size }) }}
            </span>
            <button
              type="button"
              class="flex shrink-0 items-center gap-1.5 rounded-md border border-[color-mix(in_srgb,var(--accent)_50%,transparent)] bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_25%,transparent)] disabled:opacity-50"
              :disabled="multiSelBusy || curseInstallBusy !== null"
              @click="downloadSelectedCurse"
            >
              <svg v-if="multiSelBusy" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current">
                <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
              </svg>
              <svg v-else viewBox="0 0 16 16" class="h-3 w-3 fill-current">
                <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/>
              </svg>
              {{ multiSelBusy ? t("mods.installingSel") : t("mods.downloadSel") }}
            </button>
            <button
              type="button"
              class="rounded-md px-1.5 py-0.5 text-xs text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
              :title="t('files.clear')"
              @click="clearSelAll"
            >
              ×
            </button>
          </div>
          <p v-if="!curseSearched" class="py-8 text-center text-xs text-[color:var(--tx-muted)]">{{ t("curse.help") }}</p>
          <p v-else-if="curseLoading" class="flex items-center justify-center gap-2 py-8 text-xs text-[color:var(--tx-muted)]">
            <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 animate-spin fill-current">
              <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
            </svg>
            {{ t("mods.searchingAll") }}
          </p>
          <div v-else-if="curseErr" class="rounded-md border border-[var(--border)] bg-[var(--input-50)] p-6 text-center text-xs text-[color:var(--tx-muted)]">
            <p class="mb-2 whitespace-pre-wrap">{{ curseErr }}</p>
            <button type="button" class="text-[var(--accent)] hover:underline" @click="searchCurse">{{ t("catalog.retry") }}</button>
          </div>
          <p v-else-if="curseResults.length === 0" class="py-8 text-center text-xs text-[color:var(--tx-muted)]">{{ t("mods.noResults") }}</p>
          <div v-else class="space-y-2">
            <div
              v-for="p in curseResults"
              :key="p.projectId"
              class="flex items-center gap-3 rounded-md border border-[var(--border)] bg-[var(--bg)] px-3 py-2"
            >
              <button
                type="button"
                class="flex h-6 w-6 shrink-0 items-center justify-center rounded-md border transition-colors"
                :class="selCurse.has(p.projectId)
                  ? 'border-[var(--accent)] bg-[color-mix(in_srgb,var(--accent)_20%,transparent)]'
                  : 'border-[var(--border)] hover:border-[var(--tx-muted)]'"
                :title="selCurse.has(p.projectId) ? t('files.clear') : t('mods.selForDownload')"
                @click="toggleCurseSel(p.projectId)"
              >
                <svg v-if="selCurse.has(p.projectId)" viewBox="0 0 16 16" class="h-3 w-3 fill-[var(--accent)]">
                  <path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/>
                </svg>
              </button>
              <img
                v-if="p.iconUrl"
                :src="searchIconUrl(p.iconUrl)"
                :alt="p.name"
                loading="lazy"
                class="h-10 w-10 shrink-0 rounded-md object-cover"
              />
              <div v-else class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-[11px] text-[color:var(--tx-muted)]">
                {{ p.name.slice(0, 2).toUpperCase() }}
              </div>
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <svg viewBox="0 0 24 24" class="h-3 w-3 shrink-0" :title="t('mods.serviceCurseforge')"><path fill="#F16436" d="M18.326 9.2145S23.2261 8.4418 24 6.1882h-7.5066V4.4H0l2.0318 2.3576V9.173s5.1267-.2665 7.1098 1.2372c2.7146 2.516-3.053 5.917-3.053 5.917L5.0995 19.6c1.5465-1.4726 4.494-3.3775 9.8983-3.2857-2.0565.65-4.1245 1.6651-5.7344 3.2857h10.9248l-1.0288-3.2726s-7.918-4.6688-.8336-7.1127z"/></svg>
                  <span class="truncate text-xs font-semibold text-[color:var(--tx-strong)]">{{ p.name }}</span>
                  <span v-if="curseInstallBusy === p.projectId" class="text-[11px] text-[var(--accent)]">{{ t("curse.installing") }}</span>
                </div>
                <p class="line-clamp-1 text-[11px] text-[color:var(--tx-muted)]">{{ p.summary }}</p>
                <p class="mt-0.5 flex items-center gap-2 text-[11px] text-[color:var(--tx-muted)]">
                  <span>{{ t("mods.byAuthor", { author: p.author }) }}</span>
                  <span>{{ p.downloadCount.toLocaleString() }}</span>
                </p>
              </div>
              <button
                type="button"
                class="shrink-0 rounded-md border border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[color-mix(in_srgb,var(--accent)_10%,transparent)] px-3 py-1.5 text-xs font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_20%,transparent)] disabled:opacity-50"
                :disabled="curseInstallBusy !== null && curseInstallBusy !== p.projectId || installedCurseIds.has(p.projectId)"
                @click="installCurse(p)"
              >
                <template v-if="installedCurseIds.has(p.projectId)">
                  <svg viewBox="0 0 16 16" class="mr-1 inline h-3 w-3 fill-current"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/></svg>
                  {{ t("mods.installedBadge") }}
                </template>
                <template v-else>{{ t("mods.download") }}</template>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
    <!-- Отдельное окно просмотра ресурса (win=filedetail): на весь экран -->
    <div v-if="isFileDetailWin" class="fixed inset-0 z-50 flex flex-col overflow-hidden bg-[var(--bg)] text-[color:var(--tx)] font-sans">
      <div class="flex shrink-0 items-center justify-between gap-3 border-b border-[var(--border)] bg-[var(--panel)] px-4 py-2.5">
        <div class="flex min-w-0 flex-1 items-center gap-3">
          <img v-if="fileDetailMr?.iconUrl" :src="searchIconUrl(fileDetailMr.iconUrl)" :alt="fileDetailMr.title" loading="lazy" class="h-10 w-10 shrink-0 rounded-md object-cover" />
          <div v-else-if="fileDetailMr" class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-[var(--input-50)] text-xs text-[color:var(--tx-muted)]">
            {{ fileDetailMr.title.slice(0, 2).toUpperCase() }}
          </div>
          <div class="min-w-0">
            <h2 class="truncate text-sm font-semibold text-[color:var(--tx-strong)]">{{ fileDetailMr?.title ?? fileDetailTitle }}</h2>
            <div class="flex flex-wrap items-center gap-x-3 gap-y-0.5 text-[11px] text-[color:var(--tx-muted)]">
              <template v-if="fileDetailMr">
                <span>{{ t("mods.byAuthor", { author: fileDetailMr.author }) }}</span>
                <span class="flex items-center gap-1">
                  <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M1.75 1.75a.75.75 0 0 0-1.5 0v9A2.25 2.25 0 0 0 2.5 13h12.75a.75.75 0 0 0 0-1.5H2.5a.75.75 0 0 1-.75-.75v-9Zm10.75 2.5a.75.75 0 0 0-1.5 0v5a.75.75 0 0 0 1.5 0v-5Zm-3 .75a.75.75 0 0 1 1.5 0v4.25a.75.75 0 0 1-1.5 0V5Zm-3 1.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3Z"/></svg>
                  {{ fileDetailMr.downloads.toLocaleString() }}
                </span>
                <span v-if="fileDetailMr.categories.length">{{ fileDetailMr.categories.slice(0, 4).join(", ") }}</span>
              </template>
            </div>
          </div>
        </div>
        <div class="flex shrink-0 items-center gap-2">
          <button
            v-if="fileDetailExternalUrl()"
            type="button"
            class="rounded-md border border-[var(--border)] bg-[var(--input)] px-2.5 py-1.5 text-xs font-medium text-[color:var(--tx-muted)] transition-colors hover:border-[color-mix(in_srgb,var(--accent)_50%,transparent)] hover:text-[var(--accent)]"
            @click="openExternal(fileDetailExternalUrl()!)"
          >
            {{ t("mods.openPage") }}
          </button>
          <button
            type="button"
            class="rounded-md p-1.5 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
            @click="closeFileDetailWin"
          >
            <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
          </button>
        </div>
      </div>
      <div class="flex shrink-0 items-center gap-1 border-b border-[var(--border)] px-4 pb-2 pt-3">
        <button
          v-for="tb in fileDetailTabs"
          :key="tb.kind"
          type="button"
          class="rounded-md px-3 py-1.5 text-xs font-medium transition-colors"
          :class="fileDetailTab === tb.kind
            ? 'bg-[var(--input)] text-[color:var(--tx-strong)]'
            : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]'"
          @click="fileDetailTab = tb.kind"
        >
          {{ t("mods.tab" + tb.kind) }}
        </button>
      </div>
      <div v-if="fileDetailMrLoading" class="flex min-h-0 flex-1 items-center justify-center text-xs text-[color:var(--tx-muted)]">
        <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
        {{ t("mods.searching") }}
      </div>
      <div v-else-if="fileDetailMr" class="min-h-0 flex-1 overflow-y-auto px-4 py-3">
        <div v-if="fileDetailTab === 'about'" class="rounded-md border border-[var(--border)] bg-[var(--bg)] px-4 py-3">
          <Markdown v-if="fileDetailMr.body" :source="fileDetailMr.body" />
          <p v-else class="py-6 text-center text-xs italic text-[color:var(--tx-muted)]">{{ t("mods.noAbout") }}</p>
        </div>
        <div v-else-if="fileDetailTab === 'versions'">
          <div v-if="fileDetailMrVersions !== null" class="mb-2 flex flex-wrap items-center gap-2">
            <FilterSelect v-model="fileDetailMcSel" :options="fileDetailMcOptions" :placeholder="t('curse.fVersion')" :multiple="false" />
            <FilterSelect v-model="fileDetailLoaderSel" :options="fileDetailLoaderOptions" :placeholder="t('mods.fLoader')" :multiple="false" />
            <FilterSelect v-model="fileDetailTypeSel" :options="versionTypeOptions" :placeholder="t('mods.fType')" :multiple="false" />
          </div>
          <div v-if="fileDetailMrVersions === null" class="flex items-center justify-center py-10 text-xs text-[color:var(--tx-muted)]">
            <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
            {{ t("mods.searching") }}
          </div>
          <div v-else-if="fileDetailFilteredVersions.length === 0" class="rounded-md border border-[var(--border)] bg-[var(--input-50)] p-6 text-center text-xs text-[color:var(--tx-muted)]">
            {{ t("mods.noVersions") }}
          </div>
          <div v-else class="space-y-1.5">
            <button
              v-for="v in fileDetailFilteredVersions"
              :key="v.id"
              type="button"
              class="flex w-full items-center gap-2 rounded-md border border-[var(--border)] bg-[var(--bg)] py-1.5 pl-2.5 pr-2 text-left transition-colors hover:border-[color-mix(in_srgb,var(--accent)_50%,transparent)] disabled:opacity-50"
              :disabled="fileDetailMrVersionBusy !== null"
              @click="installFileDetailVersion(v)"
            >
              <span
                class="h-2 w-2 shrink-0 rounded-full"
                :style="{ backgroundColor: verTypeColor(v.versionType) }"
                :title="t('mods.verType.' + v.versionType)"
              ></span>
              <span class="min-w-0 flex-1">
                <span class="flex items-center gap-1.5 text-xs font-medium text-[color:var(--tx-strong)]">
                  <span class="truncate">{{ v.name }}</span>
                  <span v-if="fileDetailInstalledVersion(v)" class="shrink-0 rounded-full border border-[color-mix(in_srgb,var(--accent)_45%,transparent)] bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] px-1.5 py-px text-[10px] font-semibold text-[var(--accent)]">{{ t("mods.installedBadge") }}</span>
                </span>
                <span class="block truncate text-[11px] text-[color:var(--tx-muted)]">
                  {{ v.loaders.map(cap).join(" · ") || "vanilla" }} · {{ v.gameVersions.slice(0, 2).join(", ") }} · {{ formatDate(v.datePublished) }}
                  <template v-if="verInstallSize(v)"> · {{ formatBytes(verInstallSize(v)) }}</template>
                </span>
              </span>
              <span class="shrink-0 rounded border border-[var(--border)] bg-[var(--input-50)] px-1.5 py-px font-mono text-[10px] text-[color:var(--tx-muted)]">{{ v.versionNumber }}</span>
              <template v-if="fileDetailMrVersionBusy === v.id">
                <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 shrink-0 animate-spin fill-[var(--accent)]"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
              </template>
              <svg v-else viewBox="0 0 16 16" class="h-3.5 w-3.5 shrink-0 fill-[var(--accent)]"><path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/></svg>
            </button>
          </div>
        </div>
        <div v-else>
          <div v-if="fileDetailMr.gallery.length" class="grid grid-cols-2 gap-2">
            <img
              v-for="g in fileDetailMr.gallery"
              :key="g.url"
              :src="g.url"
              :alt="g.title ?? ''"
              loading="lazy"
              class="h-40 w-full cursor-zoom-in rounded-md border border-[var(--border)] object-cover transition-transform hover:scale-[1.02]"
              :title="g.title ?? undefined"
              @click="openExternal(g.url)"
            />
          </div>
          <p v-else class="py-10 text-center text-xs italic text-[color:var(--tx-muted)]">{{ t("mods.noGallery") }}</p>
        </div>
      </div>
      <div v-else class="flex min-h-0 flex-1 items-center justify-center px-4 py-10">
        <p class="text-xs text-[color:var(--tx-muted)]">{{ t("mods.noAbout") }}</p>
      </div>
    </div>
</template>

<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { open as openDialog, save } from "@tauri-apps/plugin-dialog";
import { computed, nextTick, onBeforeUnmount, onMounted, provide, reactive, ref, watch } from "vue";
import { useRoute } from "vue-router";
import { isTauri, openExternal, pingServer, createLocalPack, localLoaderVersions, minecraftVersions, editPackVersion, exportPack as exportPackFn, exportSourceList, exportAuthorPack, uploadPack, modrinthCheckUpdates, modrinthInstallMod, modrinthInstallPack, modrinthProject, modrinthProjectVersions, modrinthSearch, modrinthTags as fetchModrinthTags, modrinthUpdateMod, installedModSha1, setPackIcon, setPackBanner, setPackName, elyDeviceCode, elyPoll, curseforgeSearch, curseforgeCategories, curseforgeLatestFile, curseforgeInstallFile, curseforgeModpackFiles, curseforgeInstallPack, curseforgeKeyConfigured, curseforgeProjectDetail, deleteGameFiles, getStatus } from "~/lib/bridge";
import type { GameFolderKind, ModrinthInstallFolder, ModrinthSearchKind, CurseSearchHit, CursePackFile, CurseProjectDetail } from "~/lib/bridge";
import type { AuthorPackConfig, AuthorServer, AuthorSocial, AuthorTheme, CrashAnalysis, CurseInstallResult, ExportSourceItem, GameFileEntry, McVersionInfo, ModrinthProject, ModrinthTags, ModrinthVersion, ModUpdate, NewsItem, PackCatalog, PackDescriptor, ServerStatus, TrackedMod, AppStatus, DuplicateGroup } from "~/lib/types";
import { useLauncher } from "~/composables/useLauncher";
import { useI18n, getLocaleMeta } from "~/composables/useI18n";
import { LauncherCtxKey } from "~/composables/useLauncherContext";
import { getCachedIcon, setCachedIcon } from "~/lib/iconCache";
import {
  changelogLines,
  CHANGELOG_PREVIEW_LINES,
  onChangelogLinkClick,
  renderInline,
  type ChangelogLine,
} from "~/lib/changelog";
import { formatPlaytimeShort as _formatPlaytimeShort } from "~/lib/format";
import { phaseLabel as _phaseLabel, javaArchLabel as _javaArchLabel, localeLabel as _localeLabel } from "~/lib/labels";
import { verCmp, cap, sanitizeSvg, themeFromAccent, normalizeHex } from "~/lib/misc";

/** Этот экземпляр страницы открыт как отдельное окно поиска файлов. */
function isSearchWindowQuery() {
  return (
    typeof window !== "undefined" &&
    new URLSearchParams(window.location.search).get("win") === "search"
  );
}
/** Этот экземпляр страницы открыт как отдельное окно просмотра ресурса. */
function isFileDetailWindowQuery() {
  return (
    typeof window !== "undefined" &&
    new URLSearchParams(window.location.search).get("win") === "filedetail"
  );
}
const route = useRoute();
const isSearchWin = computed(() => route.query.win === "search");
const isFileDetailWin = computed(() => route.query.win === "filedetail");

function appClose() { if (isTauri()) getCurrentWindow().close(); }
function appMinimize() { if (isTauri()) getCurrentWindow().minimize(); }
function appToggleMaximize() { if (isTauri()) getCurrentWindow().toggleMaximize(); }

const __launcher = useLauncher({ keepPackId: isSearchWindowQuery() || isFileDetailWindowQuery() });
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
  gameRunning,
  progress,
  updateInfo,
  launcherVer,
  versions,
  logEntries,
  logRef,
  tab,
  packs,
  packId,
  activePack,
  sidebarRecentPacks,
  percent,
  filePercent,
  filesDone,
  loaderLabel,
  formatBytes,
  formatDate,
  formatUnixDate,
  formatPlaytime,
  isInstalledVersion,
  handleInstall,
  handleUpdate,
  handleSelectVersion,
  handleOffline,
  handleMicrosoft,
  handleEly,
  openMsAuthPage,
  msFlow,
  msPolling,
  elyFlow,
  elyPolling,
  deviceFlow,
  accounts,
  accountBusy,
  handleSwitchAccount,
  handleRemoveAccount,
  monoName,
  monoPass,
  monoBusy,
  monoProfile,
  handleMonoLogin,
  handleMonoRegister,
  handleMonoLogout,
  authorPacks,
  authorDetail,
  authorNews,
  authorBusy,
  authorSelected,
  authorVersions,
  authorTab,
  authorNewsKind,
  loadAuthorPacks,
  openAuthorDetail,
  closeAuthorDetail,
  createAuthorVersion,
  importAuthorPack,
  deleteAuthorVersion,
  addAuthorNews,
  deleteAuthorNews,
  updateAuthorMeta,
  deleteAuthorPack,
  ratePack,
  handlePlay,
  playOnServer,
  handleStop,
  handleClearLog,
  handleCopyLog,
  handleOpenPackDir,
  selectPack,
  addPack,
  refreshVersions,
  notifications,
  notify,
  dismissNotification,
  reportError,
  reportPackBug,
  appUpdate,
  appUpdating,
  appUpdateProgress,
  installAppUpdate,
  skinUrl,
  localSkin,
  skinModel,
  skinBusy,
  skinApi,
  loadLocalSkin,
  applyLocalSkin,
  removeLocalSkin,
  licenseInfo,
  licenseKeyInput,
  licenseBusy,
  licenseError,
  saveLicense,
  removeLicense,
  boostyAuthOpen,
  startBoostyLogin,
  cancelBoostyLogin,
  boostyTargetPack,
  boostyGlobalLinkedState,
  boostyGlobalOpen,
  loadBoostyGlobal,
  saveBoostyGlobal,
  startBoostyGlobalLogin,
  unlinkBoostyGlobal,
  licenseByPack,
  licenseBusyFor,
  loadLicenseStatusFor,
  loadAllLicenses,
  saveLicenseFor,
  removeLicenseFor,
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
  warnCustomMods,
  toggleWarnCustomMods,
  news,
  newsFilter,
  newsSources,
  filteredNews,
  playSubTab,
  gameFiles,
  fileIcons,
  fileSearch,
  fileToggling,
  selectedFiles,
  toggleFileSelect,
  clearFileSelection,
  selectAllFiles,
  enabledCountIn,
  setSelectedFilesEnabled,
  openFileOnModrinth,
  openFileOnCurseForge,
  handleToggleFile,
  packUrl,
  packName,
  addingPack,
  removingPack,
  removeArmed,
  themeLevel,
  setThemeLevel,
  packThemeActive,
  packLocked,
  setActivePackLocked,
  toggleTheme,
  handleAddPack,
  handleRemovePack,
  resetRemoveArm,
  packScreenshots,
  packScreenshotsInstalled,
  screenshotsLoading,
  loadPackScreenshots,
  duplicates,
  duplicatesLoading,
  loadDuplicates,
  removeDuplicate,
  myServers,
  myServersInstalled,
  loadMyServers,
  bugReportOpen,
  bugBody,
  bugLog,
  bugCopied,
  closeBugReport,
  copyBugReport,
  openBugReportIssue,
  crashAnalysis,
  closeCrashAnalysis,
  monoCatalog,
  monoCatalogLoading,
  monoCatalogError,
  loadMonoCatalog,
  loadPacks,
  loadGameFiles,
  load,
  catalogDetail,
  catalogDetailBusy,
  catalogDetailTab,
  openCatalogDetail,
  closeCatalogDetail,
  catalogComments,
  catalogCommentsBusy,
  catalogCommentCount,
  sendCatalogComment,
  editCatalogComment,
  removeCatalogComment,
  rateCatalogComment,
  profileView,
  profileBusy,
  profileIsOwn,
  openProfileView,
  closeProfileView,
  saveMyProfile,
  scanResult,
  scanBusy,
  scanModFile,
  scanByHash,
  authorCollaborators,
  collabBusy,
  addCollaborator,
  updateCollaborator,
  removeCollaborator,
  isAdmin,
  adminUsers,
  adminPacks,
  adminBusy,
  loadAdminData,
  adminBanUser,
  adminUnbanUser,
  adminDeleteUser,
  adminSetRole,
  adminDeletePack,
  adminDeleteComment,
  monoForgotOpen,
  monoForgotEmail,
  monoForgotSent,
  monoResetToken,
  monoResetPass,
  monoResetDone,
  monoAuthBusy,
  handleMonoForgot,
  handleMonoReset,
  handleMonoConfirmEmail,
} = __launcher;

const { t, locale, locales, setLocale } = useI18n();

/** Компактное время в игре (часы/минуты), привязано к локали. */
const formatPlaytimeShort = (seconds: number) => _formatPlaytimeShort(seconds, t);
const phaseLabel = (p: string) => _phaseLabel(p, t);
const javaArchLabel = (a: string) => _javaArchLabel(a, t);
const localeLabel = (code: string) => _localeLabel(code, getLocaleMeta);

/** Автор и версия активного перевода — для строки «Перевод: …» в настройках лаунчера. */
const activeLocaleAuthor = computed(() => getLocaleMeta(locale.value).author ?? "");
const activeLocaleVersion = computed(() => getLocaleMeta(locale.value).version ?? "");

/** Локализованный заголовок и совет для краш-анализа по его kind. */
function crashView(a: CrashAnalysis) {
  const k = `crash.kind.${a.kind}`;
  if (a.kind === "oom") {
    return { title: t(`${k}.title`), msg: t(`${k}.msg`, { ram: ram.value }) };
  }
  if (a.kind === "javaVersion") {
    return {
      title: t(`${k}.title`),
      msg: a.javaHint ? t(`${k}.msg`, { java: a.javaHint }) : t(`${k}.msgAuto`),
    };
  }
  return { title: t(`${k}.title`), msg: t(`${k}.msg`) };
}

/** Копирует текстовое резюме краш-анализа в буфер. */
async function copyCrashAnalysis() {
  const a = crashAnalysis.value;
  if (!a) return;
  const v = crashView(a);
  const lines = [
    v.title,
    v.msg,
    "",
    a.exception ? `Exception: ${a.exception}` : "",
    a.description ? `Description: ${a.description}` : "",
    a.suspected.length
      ? `Suspected: ${a.suspected.map((m) => `${m.name} (${m.file})`).join(", ")}`
      : "",
    `File: ${a.file}`,
  ].filter(Boolean);
  try {
    await navigator.clipboard.writeText(lines.join("\n"));
  } catch {
    const ta = document.createElement("textarea");
    ta.value = lines.join("\n");
    document.body.appendChild(ta);
    ta.select();
    document.execCommand("copy");
    ta.remove();
  }
}

/** Открывает GitHub Issues сборки с предзаполненным краш-анализом. */
function openCrashIssue() {
  const pack = activePack.value;
  const rest = (pack?.url || "").replace(/^https?:\/\/github\.com\//, "");
  const [owner, repo] = rest.split("/");
  if (!owner || !repo || owner === "USER" || repo === "REPO") return;
  const a = crashAnalysis.value;
  if (!a) return;
  const v = crashView(a);
  const body = [
    v.title,
    v.msg,
    "",
    a.exception ? `Exception: ${a.exception}` : "",
    a.description ? `Description: ${a.description}` : "",
    a.suspected.length
      ? `Suspected: ${a.suspected.map((m) => `${m.name} (${m.file})`).join(", ")}`
      : "",
    `File: ${a.file}`,
  ]
    .filter(Boolean)
    .join("\n");
  const url = `https://github.com/${owner}/${repo}/issues/new?title=${encodeURIComponent(t("reportPack.title", { name: pack?.name ?? "?" }))}&body=${encodeURIComponent(body)}`;
  if (isTauri()) openExternal(url).catch(() => window.open(url, "_blank"));
  else window.open(url, "_blank");
}

const customModsOpen = ref(false);

const catalogSource = ref<"mono" | "modrinth" | "curse">("mono");

function switchCatalogSource(s: "mono" | "modrinth" | "curse") {
  if (catalogSource.value === s) return;
  catalogSource.value = s;
  modPackDetail.value = null;
  modPackVersions.value = null;
  cpProject.value = null;
  cpFiles.value = null;
  cpDetail.value = null;
  cpErr.value = "";
  cpSearched.value = false;
  if (s === "modrinth") {
    void (async () => {
      await loadModrinthTags("modpack");
      applyPackAutoFilters();
      await searchPacks();
    })();
  } else if (s === "curse") {
    void (async () => {
      await loadCurseKeyStatus();
      await loadCpCategories();
      await searchCursePacks();
    })();
  }
}

async function openCatalogModrinthDetail(p: ModrinthProject) {
  modPackService.value = "modrinth";
  modPackOpen.value = true;
  await openPackDetail(p);
}

async function openCatalogCurseDetail(p: CurseSearchHit) {
  modPackService.value = "curseforge";
  modPackOpen.value = true;
  await openCpFiles(p);
}

/** Категории сборок в сайдбаре: авторские (GitHub), свои, с Modrinth, с CurseForge. */
type PackCat = "github" | "custom" | "modrinth" | "curseforge";
const PACK_CATS: PackCat[] = ["github", "custom", "modrinth", "curseforge"];
/** Ключи переводов для названий вкладок. */
const PACK_CAT_LABELS: Record<PackCat, string> = {
  github: "side.catGitHub",
  custom: "side.catCustom",
  modrinth: "side.catModrinth",
  curseforge: "side.catCurse",
};

/** Сворачиваемые категории сайдбара (состояние в localStorage). */
const SIDEBAR_CATS_KEY = "mono.sidebarCats";
const sidebarCat = reactive<Record<PackCat, boolean>>({
  github: true,
  custom: true,
  modrinth: true,
  curseforge: true,
});
{
  const saved =
    typeof localStorage !== "undefined"
      ? (JSON.parse(localStorage.getItem(SIDEBAR_CATS_KEY) || "{}") as Partial<Record<PackCat, boolean>>)
      : {};
  for (const k of PACK_CATS) if (typeof saved[k] === "boolean") sidebarCat[k] = saved[k];
}
function toggleSidebarCat(k: PackCat) {
  sidebarCat[k] = !sidebarCat[k];
  localStorage.setItem(SIDEBAR_CATS_KEY, JSON.stringify(sidebarCat));
}

/** Порядок вкладок категорий — меняется перетаскиванием (localStorage). */
const PACK_TABS_KEY = "mono.packTabs";
const packTabs = ref<PackCat[]>([...PACK_CATS]);
{
  const saved: unknown =
    typeof localStorage !== "undefined"
      ? JSON.parse(localStorage.getItem(PACK_TABS_KEY) || "null")
      : null;
  if (Array.isArray(saved)) {
    const order = saved.filter((k): k is PackCat => PACK_CATS.includes(k as PackCat));
    for (const k of PACK_CATS) if (!order.includes(k)) order.push(k);
    packTabs.value = order;
  }
}
function persistPackTabs() {
  localStorage.setItem(PACK_TABS_KEY, JSON.stringify(packTabs.value));
}

/** Перетаскиваемая вкладка (для drag&drop перестановки). */
const dragPackTab = ref<PackCat | null>(null);
function packTabDragStart(e: DragEvent, cat: PackCat) {
  dragPackTab.value = cat;
  e.dataTransfer?.setData("text/plain", cat);
  if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
  e.dataTransfer?.setDragImage(e.currentTarget as Element, 12, 12);
}
function packTabDragOver(e: DragEvent) {
  e.preventDefault();
  if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
}
function packTabDrop(e: DragEvent, target: PackCat) {
  e.preventDefault();
  const cat = dragPackTab.value;
  dragPackTab.value = null;
  if (!cat || cat === target) return;
  const from = packTabs.value.indexOf(cat);
  const to = packTabs.value.indexOf(target);
  if (from < 0 || to < 0) return;
  packTabs.value.splice(from, 1);
  packTabs.value.splice(to, 0, cat);
  persistPackTabs();
}
function packTabDragEnd() {
  dragPackTab.value = null;
}

/** Разбивка сборок по источнику: id mrn-* → Modrinth, cf-* → CurseForge,
 *  local-* / local:// → свои, остальные (встроенные и GitHub) → авторские. */
type PacksBySource = Record<PackCat, PackDescriptor[]>;

/** Платные сборки (с привязанным Boosty-блогом) — для панели Boosty в Настройках. */
const paidPacks = computed<PackDescriptor[]>(() => packs.value.filter((p) => Boolean(p.boostyBlog)));

const packsBySource = computed<PacksBySource>(() => {
  const out: PacksBySource = { github: [], custom: [], modrinth: [], curseforge: [] };
  for (const p of packs.value) {
    const group: PackCat = p.id.startsWith("mrn-")
      ? "modrinth"
      : p.id.startsWith("cf-")
        ? "curseforge"
        : p.id.startsWith("local-") || p.url.startsWith("local://")
          ? "custom"
          : "github";
    out[group].push(p);
  }
  return out;
});

const EXAMPLE_PACK_REPO = "https://github.com/n1orio/mono-pack-example";

const examplePackJson = `{
  "name": "Example Pack",
  "id": "example-pack",
  "version": "1.0.0",
  "description": "Минимальная сборка-пример"
}`;

async function openExamplePack() {
  try {
    await openExternal(EXAMPLE_PACK_REPO);
  } catch {
    notify(t("dev.errOpen", { url: EXAMPLE_PACK_REPO }), "error");
  }
}

const deepLinkExample =
  "https://n1orio.github.io/mono-launcher/?url=" +
  encodeURIComponent("https://github.com/n1orio/mono-pack-example") +
  "&name=" +
  encodeURIComponent("Example Pack");

/** URL репозитория GitHub активной сборки (из mrpack-ссылки). */
const activePackRepo = computed(() => {
  const url = activePack.value?.url?.replace(/^https?:\/\/github\.com\//, "") ?? "";
  const [owner, repo] = url.split("/");
  if (!owner || !repo) return "";
  return `https://github.com/${owner}/${repo}`;
});

async function openExampleInLauncher() {
  try {
    await openExternal(deepLinkExample);
  } catch {
    notify(t("dev.errOpen", { url: deepLinkExample }), "error");
  }
}

/** Строит универсальную ссылку-приглашение для любой сборки
 *  (blog — ник блога Boosty издателя, чтобы сборка пришла платной). */
function inviteLinkFor(pack: { name: string; url: string; boostyBlog?: string | null }): string {
  let link =
    "https://n1orio.github.io/mono-launcher/?url=" + encodeURIComponent(pack.url);
  if (pack.name) link += "&name=" + encodeURIComponent(pack.name);
  if (pack.boostyBlog) link += "&blog=" + encodeURIComponent(pack.boostyBlog);
  return link;
}

async function copyInviteLink() {
  const pack = activePack.value;
  if (!pack) return;
  const link = inviteLinkFor(pack);
  try {
    await navigator.clipboard.writeText(link);
    notify(t("dev.copyInviteDone"), "success");
  } catch {
    notify(`${t("dev.copyInviteFail")}: ${link}`, "error");
  }
}

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
const ICON_IMAGE =
  "M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-1 1v.878A2.25 2.25 0 1 1 2 13.378V2.5ZM5.5 1a1.5 1.5 0 1 0 0 3 1.5 1.5 0 0 0 0-3Zm5.912.5a.75.75 0 0 1 .232 1.136l-3.75 4.5a.75.75 0 0 1-1.136.029L4.22 4.441a.75.75 0 0 0-1.014.023L.22 7.341A.75.75 0 0 1-.252 6.22l3.47-3.47a2.25 2.25 0 0 1 3.043-.07l1.714 1.53 3.15-3.781a.75.75 0 0 1 1.087-.071Z";
const ICON_DUP =
  "M5 1h7.75A2.25 2.25 0 0 1 15 3.25v7.75a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2V6.75a.75.75 0 0 1 .75-.75H9a2 2 0 0 1-2-2V3.25A2.25 2.25 0 0 1 5 1Zm3.25 5H7V3.25a.25.25 0 0 1 .5 0V4.5h1.5a.5.5 0 0 1 0 1h-.25a.75.75 0 0 0 0 1.5ZM2.5 4.5h.25v3h4V9H2.5A.5.5 0 0 1 2 8.5v-3.5a.5.5 0 0 1 .5-.5Z";
const ICON_SERVER =
  "M3 1.5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-9a2 2 0 0 0-2-2ZM1.5 4.5H14.5v1.5H1.5ZM1.5 8H14.5v1.25H1.5Zm0 3.25H7v1.5H1.5A.5.5 0 0 1 1 12.25v-1ZM8.5 12.75v-1.5h6v1.5A.5.5 0 0 1 14.5 13h-5a1 1 0 0 1-1-.25ZM2 5.75a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Zm3 0a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0ZM2 9.25a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Zm3 0a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Z";
const ICON_GEAR =
  "M8 0a1.5 1.5 0 0 1 1.5 1.5v.364a4.98 4.98 0 0 1 1.424.845l.319-.19a1.5 1.5 0 0 1 1.5 2.598l-.322.19a4.97 4.97 0 0 1 0 1.784l.322.19a1.5 1.5 0 0 1-1.5 2.598l-.319-.19a4.98 4.98 0 0 1-1.424.845V13a1.5 1.5 0 0 1-3 0v-.364a4.98 4.98 0 0 1-1.424-.845l-.319.19a1.5 1.5 0 0 1-1.5-2.598l.322-.19a4.97 4.97 0 0 1 0-1.784l-.322-.19a1.5 1.5 0 0 1 1.5-2.598l.319.19A4.98 4.98 0 0 1 6.5 1.864V1.5A1.5 1.5 0 0 1 8 0Zm0 4a2 2 0 1 0 0 4 2 2 0 0 0 0-4Z";

const playSubTabs = [
  { kind: "releases" as const, icon: ICON_TAG },
  { kind: "mods" as const, icon: ICON_PACKAGE },
  { kind: "resourcepacks" as const, icon: ICON_PAINT },
  { kind: "shaderpacks" as const, icon: ICON_SUN },
  { kind: "saves" as const, icon: ICON_FOLDER },
  { kind: "duplicates" as const, icon: ICON_DUP },
  { kind: "screenshots" as const, icon: ICON_IMAGE },
  { kind: "servers" as const, icon: ICON_SERVER },
  { kind: "console" as const, icon: ICON_TERMINAL },
  { kind: "settings" as const, icon: ICON_GEAR },
];

/** Счётчик для бейджа сабтаба (файлы папок / версии). */
function subTabCount(kind: string): number {
  if (kind === "mods" || kind === "resourcepacks" || kind === "shaderpacks" || kind === "saves") {
    return (gameFiles.value[kind as GameFolderKind] ?? []).filter((f) => f.kind === "file").length;
  }
  if (kind === "releases") {
    return versions.value?.installed.length ?? 0;
  }
  return 0;
}

/** Релизы GitHub есть только у авторских сборок (kind "remote" = GitHub-репозиторий). */
const playSubTabsVisible = computed(() =>
  activePack.value?.kind === "remote"
    ? playSubTabs
    : playSubTabs.filter((st) => st.kind !== "releases")
);

/** Иконки глобальных вкладок правого верхнего угла (Сборка/Новости/Каталог/Настройки/Разработчикам). */
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
const fileSortKey = ref<"none" | "name" | "date">("none");
const fileSortDir = ref<"asc" | "desc">("asc");
function toggleFileSort(k: "name" | "date") {
  if (fileSortKey.value === k) {
    // Повторный клик — инвертируем направление.
    fileSortDir.value = fileSortDir.value === "asc" ? "desc" : "asc";
  } else {
    fileSortKey.value = k;
    // Дата — новые сверху, имя — A→Z; повторный клик перевернёт.
    fileSortDir.value = k === "date" ? "desc" : "asc";
  }
}
function clearFileSort() {
  fileSortKey.value = "none";
  fileSortDir.value = "asc";
}
const fileStatusFilter = ref<"all" | "enabled" | "disabled" | "updates">("all");
function setFileStatusFilter(k: "all" | "enabled" | "disabled" | "updates") {
  fileStatusFilter.value = fileStatusFilter.value === k ? "all" : k;
}
const fileMenuOpen = ref(false);
const fileMenuRef = ref<HTMLElement | null>(null);
const exportMenuOpen = ref(false);
const exportMenuRef = ref<HTMLElement | null>(null);
onMounted(() => {
  document.addEventListener("mousedown", onFileMenuDoc);
  document.addEventListener("keydown", onFileMenuKey);
  document.addEventListener("mousedown", onExportMenuDoc);
  document.addEventListener("keydown", onExportMenuKey);
  document.addEventListener("keydown", onGlobalEscapeKey);
  void loadCurseKeyStatus();
});
onBeforeUnmount(() => {
  document.removeEventListener("mousedown", onFileMenuDoc);
  document.removeEventListener("keydown", onFileMenuKey);
  document.removeEventListener("mousedown", onExportMenuDoc);
  document.removeEventListener("keydown", onExportMenuKey);
  document.removeEventListener("keydown", onGlobalEscapeKey);
});
function onFileMenuDoc(e: MouseEvent) {
  if (!fileMenuOpen.value) return;
  if (fileMenuRef.value && fileMenuRef.value.contains(e.target as Node)) return;
  fileMenuOpen.value = false;
}
function onFileMenuKey(e: KeyboardEvent) {
  if (e.key === "Escape") fileMenuOpen.value = false;
}
function onExportMenuDoc(e: MouseEvent) {
  if (!exportMenuOpen.value) return;
  if (exportMenuRef.value && exportMenuRef.value.contains(e.target as Node)) return;
  exportMenuOpen.value = false;
}
function onExportMenuKey(e: KeyboardEvent) {
  if (e.key === "Escape") exportMenuOpen.value = false;
}
/** ESC закрывает верхнюю открытую модалку (единый порядок для всех окон). */
function onGlobalEscapeKey(e: KeyboardEvent) {
  if (e.key !== "Escape") return;
  if (exportOpen.value) {
    exportOpen.value = false;
  } else if (editVerOpen.value) {
    editVerOpen.value = false;
  } else if (crashAnalysis.value) {
    closeCrashAnalysis();
  } else if (modPackOpen.value) {
    modPackOpen.value = false;
    modPackVersions.value = null;
    modPackDetail.value = null;
  } else if (createPackOpen.value) {
    createPackOpen.value = false;
  } else if (bugReportOpen.value) {
    closeBugReport();
  } else if (searchOpen.value) {
    closeSearch();
  } else if (fileDetail.value !== null) {
    fileDetail.value = null;
  } else if (shotIdx.value !== null) {
    shotIdx.value = null;
  }
}

/** Прогресс/запуск отражаем в заголовке окна — видно даже в свёрнутом виде. */
const mainBaseTitle = document.title || "Mono Launcher";
watch([gameRunning, progress, percent, activePack], () => {
  if (isSearchWin.value || isFileDetailWin.value || !isTauri()) return;
  const win = getCurrentWindow();
  let title = mainBaseTitle;
  if (gameRunning.value && activePack.value?.name) {
    title = `▶ ${activePack.value.name}`;
  } else if (progress.value) {
    const p = percent.value;
    const phase = String(progress.value.phase ?? "").trim();
    title = p > 0 ? `${p}%${phase ? ` · ${phase}` : ""}` : (phase || mainBaseTitle);
  }
  void win.setTitle(title);
});
const fileListFiltered = computed(() => {
  let list = gameFiles.value[playSubTab.value as GameFolderKind] ?? [];
  if (fileSortKey.value === "name") {
    const c = fileSortDir.value === "asc" ? 1 : -1;
    list = [...list].sort((a, b) =>
      c * a.displayName.toLowerCase().localeCompare(b.displayName.toLowerCase())
    );
  } else if (fileSortKey.value === "date") {
    const c = fileSortDir.value === "asc" ? 1 : -1;
    list = [...list].sort(
      (a, b) =>
        c *
        (a.modified - b.modified ||
          a.displayName.toLowerCase().localeCompare(b.displayName.toLowerCase()))
    );
  }
  // "none" — порядок с бэка (включённые сверху, затем алфавит).
  const q = fileSearch.value.trim().toLowerCase();
  if (q) {
    list = list.filter(
      (f) => f.displayName.toLowerCase().includes(q) || f.name.toLowerCase().includes(q)
    );
  }
  const st = fileStatusFilter.value;
  if (st === "enabled") return list.filter((f) => f.enabled);
  if (st === "disabled") return list.filter((f) => !f.enabled);
  if (st === "updates" && playSubTab.value !== "saves") {
    return list.filter((f) => !!modUpdateFor(f));
  }
  return list;
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

function isFileToggling(folder: string, entry: GameFileEntry): boolean {
  return fileToggling.value.has(`${folder}/${entry.name}`);
}

// ---- Метаданные Modrinth (название + аватар проекта), подгружаются лениво -----
const modrinthMeta = ref<Record<string, { title: string; icon: string; version?: string }>>({});

// ---- Метаданные CurseForge (название + иконка проекта), подгружаются лениво ----
const curseMeta = ref<Record<number, { title: string; icon: string }>>({});

function curseMetaFor(f: GameFileEntry) {
  // Мета напрямую из трекера (уже в GameFileEntry) — без API-запроса.
  if (f.curseforgeTitle || f.curseforgeIcon) {
    return { title: f.curseforgeTitle ?? "", icon: f.curseforgeIcon ?? "" };
  }
  if (!f.curseforgeProjectId) return undefined;
  return curseMeta.value[f.curseforgeProjectId];
}

/** Человекочитаемое название файла в списке: мета Modrinth или CurseForge, иначе имя файла. */
function fileMetaTitle(f: GameFileEntry): string {
  const mr = modrinthMetaFor(f)?.title;
  if (mr) return mr;
  const cf = curseMetaFor(f)?.title;
  if (cf) return cf;
  return f.name;
}

async function fetchCurseMeta(projectId: number) {
  if (!projectId || curseMeta.value[projectId]) return;
  const cacheKey = `cf:${projectId}`;
  const cached = getCachedIcon(cacheKey);
  if (cached) {
    try {
      const j = JSON.parse(cached.data) as { title?: string; icon?: string };
      if (j && typeof j.title === "string") {
        curseMeta.value = {
          ...curseMeta.value,
          [projectId]: { title: j.title, icon: typeof j.icon === "string" ? j.icon : "" },
        };
        if (!cached.stale) return;
      }
    } catch {
      /* повреждённая запись — перезагрузим с API */
    }
  }
  try {
    const d = await curseforgeProjectDetail(projectId);
    const meta = { title: d.name, icon: d.iconUrl ?? "" };
    curseMeta.value = { ...curseMeta.value, [projectId]: meta };
    setCachedIcon(cacheKey, JSON.stringify(meta));
  } catch {
    curseMeta.value = { ...curseMeta.value, [projectId]: { title: "", icon: "" } };
  }
}

// ---- Мультивыбор-удаление файлов (моды/ресурспаки/шейдеры/миры) ----
const fileDeleteArmed = ref(false);
const fileDeleteBusy = ref(false);

/** Удаляет выделенные файлы/папки текущей вкладки (с двойным подтверждением). */
async function deleteSelectedFiles() {
  if (!isTauri() || !packId.value || fileDeleteBusy.value) return;
  const folder = playSubTab.value as GameFolderKind;
  const targets = Object.values(selectedFiles.value).filter(
    (s) => s.folder === folder && isFileSafeToDelete(folder, s.entry)
  );
  if (targets.length === 0) {
    fileDeleteArmed.value = false;
    return;
  }
  fileDeleteBusy.value = true;
  try {
    const n = await deleteGameFiles(packId.value, folder, targets.map((s) => s.entry.name));
    clearFileSelection();
    fileDeleteArmed.value = false;
    notify(t("files.deleted", { n }), "success");
    await loadGameFiles(folder, true);
    await refreshModUpdates(true);
  } catch (e) {
    notify(t("err.deleteFiles", { e }));
  } finally {
    fileDeleteBusy.value = false;
  }
}

/** Не даём выбрать для удаления мусорные записи (папки-миры можно, но без точек). */
function isFileSafeToDelete(_folder: GameFolderKind, entry: GameFileEntry): boolean {
  const n = entry.name;
  return n !== "" && n !== "." && n !== ".." && !n.includes("/") && !n.includes("\\") && !n.startsWith(".");
}

function modrinthProjectId(url: string): string | null {
  return url.match(/\/mod\/([^/]+)/)?.[1] ?? null;
}

/** ID проекта Modrinth файла: из трекинга (скачан вручную) либо из URL индекса сборки. */
function modrinthProjectIdFor(f: GameFileEntry): string | null {
  if (f.modrinthProjectId) return f.modrinthProjectId;
  if (!f.modrinthUrl) return null;
  return modrinthProjectId(f.modrinthUrl) ?? null;
}

function modrinthMetaFor(f: GameFileEntry) {
  const id = modrinthProjectIdFor(f);
  return id ? modrinthMeta.value[id] : undefined;
}

/** Версия Modrinth файла (числовое имя версии), если известна. */
function modrinthVersionFor(f: GameFileEntry): string | undefined {
  return modrinthMetaFor(f)?.version;
}

async function fetchModrinthMeta(f: GameFileEntry) {
  const id = modrinthProjectIdFor(f);
  if (!id || modrinthMeta.value[id]) return;
  const versionId = f.modrinthVersionId;
  const cacheKey = `mr:${id}`;
  const cached = getCachedIcon(cacheKey);
  if (cached) {
    try {
      const j = JSON.parse(cached.data) as { title?: string; icon?: string; version?: string };
      if (j && typeof j.title === "string") {
        modrinthMeta.value = {
          ...modrinthMeta.value,
          [id]: {
            title: j.title,
            icon: typeof j.icon === "string" ? j.icon : "",
            version: typeof j.version === "string" ? j.version : undefined,
          },
        };
        if (!cached.stale) return;
      }
    } catch {
      /* повреждённая запись — перезагрузим с API */
    }
  }
  try {
    const [proj, ver] = await Promise.all([
      fetch(`https://api.modrinth.com/v2/project/${id}?fields=title,icon_url`),
      versionId
        ? fetch(`https://api.modrinth.com/v2/version/${versionId}?fields=version_number`)
        : Promise.resolve(null),
    ]);
    const meta: { title: string; icon: string; version?: string } = { title: "", icon: "" };
    if (proj.ok) {
      const j = await proj.json();
      if (typeof j?.title !== "string") return;
      meta.title = j.title;
      meta.icon = typeof j.icon_url === "string" ? j.icon_url : "";
    }
    if (ver?.ok) {
      const j = await ver.json();
      if (typeof j?.version_number === "string") meta.version = j.version_number;
    }
    if (!meta.title) return;
    modrinthMeta.value = { ...modrinthMeta.value, [id]: meta };
    setCachedIcon(cacheKey, JSON.stringify(meta));
  } catch {
    /* метаданные некритичны */
  }
}

// ---- Иконки из поиска: кеш data-URL, чтобы повторные поиски не дёргали сеть ----
const searchIconData = ref<Record<string, string>>({});

function searchIconUrl(url: string): string {
  return searchIconData.value[url] ?? url;
}

async function warmSearchIcon(url: string) {
  if (!url || searchIconData.value[url]) return;
  const cached = getCachedIcon(`pic:${url}`);
  if (cached) {
    searchIconData.value = { ...searchIconData.value, [url]: cached.data };
    if (!cached.stale) return;
  }
  try {
    const res = await fetch(url);
    if (!res.ok) return;
    const blob = await res.blob();
    const dataUrl = await new Promise<string>((resolve, reject) => {
      const r = new FileReader();
      r.onload = () => resolve(String(r.result));
      r.onerror = () => reject(new Error("file read"));
      r.readAsDataURL(blob);
    });
    searchIconData.value = { ...searchIconData.value, [url]: dataUrl };
    setCachedIcon(`pic:${url}`, dataUrl);
  } catch {
    /* иконки некритичны */
  }
}

// Тянем мету только для видимых строк (виртуализированный список).
watch(fileListVisible, (rows) => {
  for (const f of rows) {
    if (f.modrinthProjectId || f.modrinthUrl) fetchModrinthMeta(f);
    if (f.curseforgeProjectId) fetchCurseMeta(f.curseforgeProjectId);
  }
});

// ---- Поиск файлов: Modrinth / CurseForge ----
type SearchService = "modrinth" | "curseforge";
const searchService = ref<SearchService>("modrinth");
const searchOpen = ref(false);
const searchPos = ref<{ x: number | null; y: number | null }>({ x: null, y: null });
const searchWinStyle = computed(() => ({
  right: searchPos.value.x === null ? "2rem" : undefined,
  bottom: searchPos.value.y === null ? "2rem" : undefined,
  left: searchPos.value.x === null ? undefined : `${searchPos.value.x}px`,
  top: searchPos.value.y === null ? undefined : `${searchPos.value.y}px`,
}));
let searchDrag: { dx: number; dy: number } | null = null;
function dragSearchWin(e: PointerEvent) {
  if (isSearchWin.value) return;
  if ((e.target as HTMLElement).closest("button")) return;
  const x = searchPos.value.x ?? window.innerWidth - 768;
  const y = searchPos.value.y ?? window.innerHeight - 480;
  searchDrag = { dx: e.clientX - x, dy: e.clientY - y };
  window.addEventListener("pointermove", moveSearchWin);
  window.addEventListener("pointerup", endSearchDrag, { once: true });
}
function moveSearchWin(e: PointerEvent) {
  if (!searchDrag) return;
  searchPos.value = {
    x: Math.max(0, Math.min(window.innerWidth - 120, e.clientX - searchDrag.dx)),
    y: Math.max(0, Math.min(window.innerHeight - 64, e.clientY - searchDrag.dy)),
  };
}
function endSearchDrag() {
  searchDrag = null;
  window.removeEventListener("pointermove", moveSearchWin);
}
function closeSearch() {
  if (isSearchWin.value && isTauri()) {
    getCurrentWindow().close();
    return;
  }
  searchOpen.value = false;
  modVersions.value = null;
  modDetail.value = null;
  clearSelAll();
  window.removeEventListener("pointermove", moveSearchWin);
}
/** Открывает поиск файлов. В Tauri — настоящее отдельное окно, в браузере — плавающая панель. */
async function openSearch(kind: ModrinthSearchKind, service: SearchService = "modrinth") {
  modSearchKind.value = kind;
  if (isTauri()) {
    if (!packId.value) return;
    const existing = await WebviewWindow.getByLabel("search");
    if (existing) {
      try {
        await existing.close();
      } catch {
        /* окно уже закрывается */
      }
    }
    try {
      const devBase = import.meta.env.DEV ? "http://localhost:1420/" : "";
      new WebviewWindow("search", {
        url: `${devBase}?win=search&kind=${kind}&service=${service}&packId=${encodeURIComponent(packId.value)}`,
        title: searchTitle.value,
        width: 760,
        height: 640,
        minWidth: 520,
        minHeight: 400,
        resizable: true,
      });
    } catch (e) {
      notify(t("mods.windowErr", { e }), "error");
    }
    return;
  }
  searchService.value = service;
  modSearchQuery.value = "";
  modSearchResults.value = [];
  modSearchErr.value = "";
  modVersions.value = null;
  modDetail.value = null;
  modFilters.versions = [];
  modFilters.loaders = [];
  modFilters.categories = [];
  modFilters.env = "";
  modFilters.sort = "relevance";
  modDatapackWorld.value = null;
  curseQuery.value = "";
  curseResults.value = [];
  curseSearched.value = false;
  curseErr.value = "";
  searchOpen.value = true;
  autoFiltersDone.value = false;
  if (kind === "datapack" && !gameFiles.value.saves) {
    void loadGameFiles("saves");
  }
  // Сразу подгружаем фильтры/теги и запускаем поиск, чтобы не ждать Enter.
  await runInitialSearch();
}
const searchInput = computed({
  get: () => (searchService.value === "modrinth" ? modSearchQuery.value : curseQuery.value),
  set: (v: string) => {
    if (searchService.value === "modrinth") modSearchQuery.value = v;
    else curseQuery.value = v;
  },
});
const searchLoading = computed(() =>
  searchService.value === "modrinth" ? modSearchLoading.value : curseLoading.value
);
const searchTitle = computed(() => {
  const kind = modSearchKind.value;
  switch (kind) {
    case "mod":
      return t("mods.title");
    case "resourcepack":
      return t("mods.titleRP");
    case "shaderpack":
      return t("mods.titleShaders");
    case "datapack":
      return t("mods.titleDatapack");
    default:
      return t("mods.title");
  }
});
async function switchSearchService(s: SearchService) {
  if (s === searchService.value) return;
  if (s === "curseforge" && modSearchKind.value === "datapack") return;
  searchService.value = s;
  // Сразу грузим фильтры/ключ и запускаем первичный поиск, чтобы не ждать Enter.
  await runInitialSearch();
}
function doSearch() {
  if (searchService.value === "modrinth") void searchMods();
  else void searchCurse();
}
const modSearchKind = ref<ModrinthSearchKind>("mod");
const modSearchQuery = ref("");
const modSearchLoading = ref(false);
const modSearchResults = ref<ModrinthProject[]>([]);
const modSearchErr = ref("");
const modSearchOffset = ref(0);
const modSearchMore = ref(false);
const modSearchMoreBusy = ref(false);
const modSearchBox = ref<HTMLElement | null>(null);
let modSearchGen = 0;
const MOD_SEARCH_PAGE = 20;
const modVersions = ref<ModrinthVersion[] | null>(null);
const modVersionsRaw = ref<ModrinthVersion[]>([]);
const verFilterMc = ref("");
const verFilterLoader = ref("");
const verFilterType = ref("");

const verFilterMcOptions = computed(() =>
  Array.from(new Set(modVersionsRaw.value.flatMap((v) => v.gameVersions)))
    .sort((a, b) => verCmp(b, a))
    .map((v) => ({ value: v, label: v }))
);
const verFilterLoaderOptions = computed(() =>
  Array.from(new Set(modVersionsRaw.value.flatMap((v) => v.loaders)))
    .map((v) => ({ value: v, label: cap(v) }))
    .sort((a, b) => a.label.localeCompare(b.label))
);
const verFilterTypeOptions = computed(() =>
  Array.from(new Set(modVersionsRaw.value.map((v) => v.versionType)))
    .sort((a, b) => a.localeCompare(b))
    .map((v) => ({ value: v, label: t("mods.verType." + v) }))
);
const filteredModVersions = computed(() =>
  modVersionsRaw.value.filter((v) => {
    if (verFilterMc.value && !v.gameVersions.includes(verFilterMc.value)) return false;
    if (verFilterLoader.value && !v.loaders.includes(verFilterLoader.value)) return false;
    if (verFilterType.value && v.versionType !== verFilterType.value) return false;
    return true;
  })
);
const verFilterMcSel = computed({
  get: () => (verFilterMc.value ? [verFilterMc.value] : []),
  set: (v: string[]) => { verFilterMc.value = v[0] ?? ""; },
});
const verFilterLoaderSel = computed({
  get: () => (verFilterLoader.value ? [verFilterLoader.value] : []),
  set: (v: string[]) => { verFilterLoader.value = v[0] ?? ""; },
});
const verFilterTypeSel = computed({
  get: () => (verFilterType.value ? [verFilterType.value] : []),
  set: (v: string[]) => { verFilterType.value = v[0] ?? ""; },
});
const verTypeColor = (ty: string) =>
  ty === "beta" ? "#f59e0b" : ty === "alpha" ? "#ef4444" : "#22c55e";
const verInstallSize = (v: ModrinthVersion) =>
  v.files.reduce((m, f) => Math.max(m, f.size ?? 0), 0);
const modInstallBusy = ref<string | null>(null);
const modUpdates = ref<ModUpdate[]>([]);
const trackedMods = ref<TrackedMod[]>([]);
const updatingMod = ref<string | null>(null);
const updateAllBusy = ref(false);

/** Обновления только текущей вкладки (моды / ресурспаки / шейдеры). */
const modUpdatesTab = computed(() =>
  playSubTab.value === "mods" ||
  playSubTab.value === "resourcepacks" ||
  playSubTab.value === "shaderpacks"
    ? modUpdates.value.filter((u) => u.folder === playSubTab.value)
    : []
);

const modPackOpen = ref(false);
const modPackService = ref<"modrinth" | "curseforge">("modrinth");
const modPackQuery = ref("");
const modPackLoading = ref(false);
const modPackResults = ref<ModrinthProject[]>([]);
const modPackVersions = ref<ModrinthVersion[] | null>(null);
const modPackInstalling = ref<string | null>(null);
const modPackDetail = ref<ModrinthProject | null>(null);
const modPackTab = ref<"about" | "versions" | "gallery">("about");

/** Поиск сборок на CurseForge (отдельное состояние от поиска файлов). */
const cpSearched = ref(false);
const cpLoading = ref(false);
const cpErr = ref("");
const cpResults = ref<CurseSearchHit[]>([]);
const cpProject = ref<CurseSearchHit | null>(null);
const cpFiles = ref<CursePackFile[] | null>(null);
const cpBusy = ref<number | null>(null);
const cpDetail = ref<CurseProjectDetail | null>(null);
const cpDetailLoading = ref(false);
const cpTab = ref<"about" | "versions" | "screenshots">("about");
const cpTabs: ("about" | "versions" | "screenshots")[] = ["about", "versions", "screenshots"];
const cpTabBusy = ref(false);
/** Ссылка на сайт сборки для кнопки «Открыть страницу». */
const cpWebsiteUrl = ref("");

// ---- Просмотр ресурса из списка установленных файлов (страница в лаунчере) ----
const fileDetail = ref<{ folder: GameFolderKind; entry: GameFileEntry } | null>(null);
const fileDetailMrLoading = ref(false);
const fileDetailMr = ref<ModrinthProject | null>(null);
const fileDetailMrVersions = ref<ModrinthVersion[] | null>(null);
const fileDetailInstalledSha = ref<string | null>(null);
const fileDetailMcFilter = ref<string | null>(null);
const fileDetailTab = ref<"about" | "versions" | "gallery">("about");
const fileDetailTabs: { kind: "about" | "versions" | "gallery" }[] = [
  { kind: "about" },
  { kind: "versions" },
  { kind: "gallery" },
];
const fileDetailMrVersionBusy = ref<string | null>(null);
const fileDetailCfLoading = ref(false);
const updatingFileDetail = ref(false);
const fileDetailCf = ref<CurseProjectDetail | null>(null);
/** Строка — с какого проекта CurseForge открыт просмотр (для кнопки «обновить»). */
const fileDetailFolder = ref<GameFolderKind>("mods");
/** Заголовок окна просмотра ресурса (из имени файла), пока проект не подгружен. */
const fileDetailTitle = ref("");

/** Закрывает отдельное окно просмотра ресурса. */
async function closeFileDetailWin() {
  if (isTauri()) {
    try {
      await getCurrentWindow().close();
    } catch {
      /* окно уже закрывается */
    }
  }
}

async function openFileDetail(folder: GameFolderKind, entry: GameFileEntry) {
  let slug = entry.modrinthProjectId || "";
  const m = /\/mod\/([^/]+)\/?$/.exec(entry.modrinthUrl ?? "");
  if (!slug && m) slug = m[1];
  // В Tauri — настоящее отдельное окно просмотра ресурса (как окно скачки мода).
  if (isTauri() && (slug || entry.curseforgeProjectId)) {
    if (!packId.value) return;
    const existing = await WebviewWindow.getByLabel("filedetail");
    if (existing) {
      try {
        await existing.close();
      } catch {
        /* окно уже закрывается */
      }
    }
    const devBase = import.meta.env.DEV ? "http://localhost:1420/" : "";
    try {
      new WebviewWindow("filedetail", {
        url: `${devBase}?win=filedetail&slug=${encodeURIComponent(slug)}&cfid=${entry.curseforgeProjectId || ""}&folder=${folder}&packId=${encodeURIComponent(packId.value)}&name=${encodeURIComponent(entry.displayName || entry.name || "")}`,
        title: entry.displayName || entry.name || t("files.view"),
        width: 820,
        height: 660,
        minWidth: 560,
        minHeight: 420,
        resizable: true,
      });
    } catch (e) {
      notify(t("mods.windowErr", { e }), "error");
    }
    return;
  }
  fileDetail.value = { folder, entry };
  fileDetailFolder.value = folder;
  fileDetailMr.value = null;
  fileDetailMrVersions.value = null;
  fileDetailCf.value = null;
  fileDetailMcFilter.value = status.value?.minecraft_version || null;
  fileDetailLoaderFilter.value = "";
  fileDetailTypeFilter.value = "";
  fileDetailTab.value = "about";
  if (slug) {
    fileDetailMrLoading.value = true;
    try {
      fileDetailMr.value = await modrinthProject(slug);
      const fl = folder === "saves" && (entry.kind === "dir" ? true : false) ? "mods" : folder;
      void loadFileDetailVersions(slug, fl);
      void loadFileDetailInstalledSha(fileDetailMr.value.projectId);
    } catch {
      /* не удалось — остаётся placeholder проекта */
    } finally {
      fileDetailMrLoading.value = false;
    }
  } else if (entry.curseforgeProjectId) {
    fileDetailCfLoading.value = true;
    try {
      fileDetailCf.value = await curseforgeProjectDetail(entry.curseforgeProjectId);
    } catch {
      fileDetailCf.value = null;
    } finally {
      fileDetailCfLoading.value = false;
    }
  }
}

async function loadFileDetailVersions(slug: string, folder: GameFolderKind) {
  try {
    fileDetailMrVersions.value = await modrinthProjectVersions(slug, undefined, undefined);
  } catch {
    fileDetailMrVersions.value = [];
  }
}

/** Загружает sha1 установленного файла проекта, чтобы отметить текущую версию. */
async function loadFileDetailInstalledSha(projectId: string) {
  fileDetailInstalledSha.value = null;
  if (!isTauri() || !packId.value || !projectId) return;
  try {
    fileDetailInstalledSha.value = (await installedModSha1(packId.value, projectId)) ?? null;
  } catch {
    fileDetailInstalledSha.value = null;
  }
}

/** Установлена ли уже эта версия (по sha1 файла среди файлов версии). */
const fileDetailInstalledVersion = (v: ModrinthVersion) =>
  !!fileDetailInstalledSha.value && v.files.some((f) => {
    const h = f.hashes?.["sha1"];
    return !!h && h.toLowerCase() === fileDetailInstalledSha.value;
  });

/** Все версии игры, встречающиеся у файла (для фильтра версии). */
const fileDetailMcOptions = computed(() =>
  Array.from(new Set((fileDetailMrVersions.value ?? []).flatMap((v) => v.gameVersions)))
    .sort(verCmpDesc)
    .map((mc) => ({ value: mc, label: mc }))
);
/** Платформы/загрузчики, встречающиеся у файла. */
const fileDetailLoaderOptions = computed(() =>
  Array.from(new Set((fileDetailMrVersions.value ?? []).flatMap((v) => v.loaders)))
    .map((l) => ({ value: l, label: cap(l) }))
    .sort((a, b) => a.label.localeCompare(b.label))
);

const fileDetailMcSel = computed({
  get: () => (fileDetailMcFilter.value ? [fileDetailMcFilter.value] : []),
  set: (v: string[]) => { fileDetailMcFilter.value = v[0] ?? null; },
});
const fileDetailLoaderFilter = ref("");
const fileDetailTypeFilter = ref("");
const fileDetailLoaderSel = computed({
  get: () => (fileDetailLoaderFilter.value ? [fileDetailLoaderFilter.value] : []),
  set: (v: string[]) => { fileDetailLoaderFilter.value = v[0] ?? ""; },
});
const fileDetailTypeSel = computed({
  get: () => (fileDetailTypeFilter.value ? [fileDetailTypeFilter.value] : []),
  set: (v: string[]) => { fileDetailTypeFilter.value = v[0] ?? ""; },
});

/** Версии с применёнными фильтрами (версия игры, загрузчик, канал):
 *  сначала подходящие под выбранную версию сборки, затем остальные. */
const fileDetailFilteredVersions = computed<ModrinthVersion[]>(() => {
  const all = fileDetailMrVersions.value ?? [];
  const explicit = all.filter((v) =>
    (!fileDetailMcFilter.value || v.gameVersions.includes(fileDetailMcFilter.value)) &&
    (!fileDetailLoaderFilter.value || v.loaders.includes(fileDetailLoaderFilter.value)) &&
    (!fileDetailTypeFilter.value || v.versionType === fileDetailTypeFilter.value)
  );
  const mc = status.value?.minecraft_version;
  const loader = status.value?.loader?.replace("-loader", "");
  const isMod = fileDetailFolder.value !== "saves";
  const matchLoader = isMod && loader ? (v: ModrinthVersion) => v.loaders.includes(loader) : () => true;
  const match = mc ? explicit.filter((v) => v.gameVersions.includes(mc) && matchLoader(v)) : explicit;
  const rest = mc ? explicit.filter((v) => !match.includes(v)) : [];
  return [...match, ...rest];
});

/** Сортировка версий «1.21.1» по убыванию (новые сверху). */
function verCmpDesc(a: string, b: string): number {
  const pa = a.split(".").map((x) => parseInt(x, 10) || 0);
  const pb = b.split(".").map((x) => parseInt(x, 10) || 0);
  const n = Math.max(pa.length, pb.length);
  for (let i = 0; i < n; i++) {
    const da = pa[i] ?? 0;
    const db = pb[i] ?? 0;
    if (da !== db) return db - da;
  }
  return 0;
}

/** Кнопка «открыть страницу» внешнего сервиса. */
function fileDetailExternalUrl(): string | null {
  const d = fileDetail.value;
  if (!d) return null;
  if (d.entry.curseforgeProjectId && fileDetailCf.value?.websiteUrl) {
    return fileDetailCf.value.websiteUrl;
  }
  const slug = fileDetailMr.value?.slug || d.entry.modrinthProjectId;
  const m = /\/mod\/([^/]+)\/?$/.exec(d.entry.modrinthUrl ?? "");
  if (slug) return `https://modrinth.com/mod/${slug}`;
  if (m) return `https://modrinth.com/mod/${m[1]}`;
  return null;
}

/** Обновление: Modrinth — текущая версия через update, CurseForge — последняя версия. */
async function updateFileDetail() {
  const d = fileDetail.value;
  if (!d || !packId.value || updatingFileDetail.value) return;
  const folder = fileDetailFolder.value === "saves" ? "mods" : fileDetailFolder.value;
  updatingFileDetail.value = true;
  try {
    if (d.entry.curseforgeProjectId) {
      const file = await curseforgeLatestFile(packId.value, d.entry.curseforgeProjectId);
      await curseforgeInstallFile(packId.value, file, folder);
    } else if (d.entry.modrinthProjectId && d.entry.name) {
      await modrinthUpdateMod(packId.value, d.entry.name);
    } else if (fileDetailMr.value) {
      // Нет трекера, но есть slug — установим последнюю подходящую версию
      // под версию сборки (через серверную фильтрацию по MC).
      const mc = status.value?.minecraft_version || undefined;
      const versions = fileDetailMr.value
        ? await modrinthProjectVersions(fileDetailMr.value.projectId, mc, undefined)
        : [];
      const target = versions[0] ?? fileDetailMrVersions.value?.[0];
      if (target) {
        await modrinthInstallMod(packId.value, target.id, folder as ModrinthInstallFolder);
      }
    }
    await loadGameFiles(fileDetailFolder.value, true);
    await refreshModUpdates(true);
    notify(t("files.updated"), "success");
  } catch (e) {
    notify(t("files.updateErr", { e }));
  } finally {
    updatingFileDetail.value = false;
  }
}

/** Установка конкретной версии из просмотра ресурса (Modrinth). */
async function installFileDetailVersion(v: ModrinthVersion) {
  const d = fileDetail.value;
  if (!d || !packId.value || fileDetailMrVersionBusy.value) return;
  if (d.entry.curseforgeProjectId) return;
  const folder = (fileDetailFolder.value === "saves" ? "mods" : fileDetailFolder.value) as ModrinthInstallFolder;
  fileDetailMrVersionBusy.value = v.id;
  try {
    await modrinthInstallMod(packId.value, v.id, folder);
    await loadGameFiles(fileDetailFolder.value, true);
    await refreshModUpdates(true);
    notify(t("mods.installed", { kind: kindNoun(folder), name: v.name }), "success");
  } catch (e) {
    notify(t("files.updateErr", { e }));
  } finally {
    fileDetailMrVersionBusy.value = null;
  }
}

/** Фильтры поиска сборок на CurseForge (категория/версия/сортировка). */
const cpCatOptions = ref<{ value: string; label: string }[]>([]);
const cpCatId = ref<number | null>(null);
const cpCatSel = computed({
  get: () => (cpCatId.value !== null ? [String(cpCatId.value)] : []),
  set: (v: string[]) => {
    cpCatId.value = v[0] ? Number(v[0]) : null;
  },
});
const cpVersion = ref("");
const cpVerSel = computed({
  get: () => (cpVersion.value ? [cpVersion.value] : []),
  set: (v: string[]) => {
    cpVersion.value = v[0] ?? "";
  },
});
const cpSortField = ref("2");
const cpSortSel = computed({
  get: () => [cpSortField.value],
  set: (v: string[]) => {
    cpSortField.value = v[0] ?? "2";
  },
});
async function loadCpCategories() {
  if (!isTauri()) return;
  cpCatOptions.value = [];
  cpCatId.value = null;
  try {
    const cats = await curseforgeCategories(4471);
    cpCatOptions.value = cats.map((c) => ({ value: String(c.id), label: c.name }));
  } catch {
    /* фильтр просто не появится */
  }
}

/** Открывает модалку скачивания сборки (Modrinth по умолчанию, либо CurseForge).
 *  Сразу грузит теги, проставляет автофильтры и запускает поиск — чтобы не ждать Enter. */
async function openModPackModal(service: "modrinth" | "curseforge" = "modrinth") {
  modPackOpen.value = true;
  modPackQuery.value = "";
  modPackService.value = service;
  modPackVersions.value = null;
  modPackDetail.value = null;
  modPackTab.value = "about";
  cpProject.value = null;
  cpFiles.value = null;
  cpDetail.value = null;
  cpErr.value = "";
  cpSearched.value = false;
  await loadModrinthTags("modpack");
  applyPackAutoFilters();
  if (service === "modrinth") {
    await searchPacks();
  } else {
    await loadCpCategories();
    await searchCursePacks();
  }
}

function switchPackService(s: "modrinth" | "curseforge") {
  if (s === modPackService.value) return;
  modPackService.value = s;
  modPackDetail.value = null;
  modPackVersions.value = null;
  cpProject.value = null;
  cpFiles.value = null;
  cpDetail.value = null;
  cpErr.value = "";
  cpVersion.value = "";
  if (s === "curseforge") {
    void loadCpCategories();
    void searchCursePacks();
  } else {
    void searchPacks();
  }
}

function searchPacksOrCurse() {
  if (modPackService.value === "modrinth") void searchPacks();
  else void searchCursePacks();
}

/** Поиск сборок на CurseForge (класс modpacks). */
async function searchCursePacks() {
  if (!isTauri() || cpLoading.value) return;
  cpLoading.value = true;
  cpSearched.value = true;
  cpErr.value = "";
  cpProject.value = null;
  cpFiles.value = null;
  cpDetail.value = null;
  try {
    cpResults.value = await curseforgeSearch(
      modPackQuery.value.trim(),
      4471,
      cpCatId.value,
      cpVersion.value || undefined,
      cpSortField.value
    );
  } catch (e) {
    cpResults.value = [];
    cpErr.value = String(e);
  } finally {
    cpLoading.value = false;
  }
}

/** Файлы сборки CurseForge (выбор версии). */
async function openCpFiles(p: CurseSearchHit) {
  cpProject.value = p;
  cpFiles.value = null;
  cpErr.value = "";
  cpTab.value = "about";
  // Деталка (описание/скриншоты/категории) подгружается независимо и не блокирует список файлов.
  void loadCpDetail(p.projectId);
  try {
    cpFiles.value = await curseforgeModpackFiles(p.projectId);
  } catch (e) {
    cpErr.value = String(e);
    cpFiles.value = [];
  }
}

/** Загружает полное описание проекта CurseForge (описание/скриншоты/категории). */
async function loadCpDetail(projectId: number) {
  cpDetailLoading.value = true;
  cpDetail.value = null;
  cpWebsiteUrl.value = "";
  try {
    const d = await curseforgeProjectDetail(projectId);
    cpDetail.value = d;
    cpWebsiteUrl.value = d.websiteUrl;
  } catch (e) {
    cpDetail.value = null;
    notify(t("err.curseDetail", { e }));
  } finally {
    cpDetailLoading.value = false;
  }
}

/** Скачивает и устанавливает сборку CurseForge как отдельную сборку. */
async function installCpPack(f: CursePackFile) {
  if (!cpProject.value || cpBusy.value !== null) return;
  cpBusy.value = f.fileId;
  try {
    const pack = await curseforgeInstallPack(cpProject.value.projectId, f.fileId);
    notify(t("mods.packInstalled", { name: pack.name }), "success");
    modPackOpen.value = false;
    cpProject.value = null;
    cpFiles.value = null;
    cpDetail.value = null;
    await loadPacks();
    await nextTick();
    openPackTab(pack.id);
  } catch (e) {
    notify(t("mods.packInstallErr", { e }), "error");
  } finally {
    cpBusy.value = null;
  }
}
const modPackTabs: { kind: "about" | "versions" | "gallery"; icon: string }[] = [
  { kind: "about", icon: '<path d="M3.5 2.75A1.75 1.75 0 0 1 5.25 1h5.5c.966 0 1.75.784 1.75 1.75v10.5A1.75 1.75 0 0 1 10.75 15h-5.5a1.75 1.75 0 0 1-1.75-1.75V2.75ZM5.25 2.5a.25.25 0 0 0-.25.25v10.5c0 .138.112.25.25.25h5.5a.25.25 0 0 0 .25-.25V2.75a.25.25 0 0 0-.25-.25h-5.5ZM6.5 5.75a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 0 1.5h-1.5a.75.75 0 0 1-.75-.75Zm0 3a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 0 1.5h-1.5a.75.75 0 0 1-.75-.75Zm0 3a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 0 1.5h-1.5a.75.75 0 0 1-.75-.75Z"/>' },
  { kind: "versions", icon: '<path d="M2.22 3.305l5.25-2.625a1.75 1.75 0 0 1 1.56 0l5.25 2.625c.511.255.722.862.61 1.41L12.53 10.65c-.121.6-.416 1.154-.836 1.57l-3.117 3.09a.75.75 0 0 1-1.056 0l-3.117-3.09a3.25 3.25 0 0 1-.836-1.57L1.61 4.715a1.75 1.75 0 0 1 .61-1.41Zm7.78 2.195a1.75 1.75 0 0 1 .75-1.415l4.925-2.462L11.15 1.5h-6.3L1.075 1.623l4.925 2.462a1.75 1.75 0 0 1 .75 1.415v4.837c0 .034.001.068.004.102l3.647-1.462L10 5.5Z"/>' },
  { kind: "gallery", icon: '<path d="M1.75 1.75A1.75 1.75 0 0 0 0 3.5v9A1.75 1.75 0 0 0 1.75 14.25h12.5A1.75 1.75 0 0 0 16 12.5v-9a1.75 1.75 0 0 0-1.75-1.75H1.75ZM1.5 3.5a.25.25 0 0 1 .25-.25h12.5a.25.25 0 0 1 .25.25v9a.25.25 0 0 1-.25.25H1.75a.25.25 0 0 1-.25-.25v-9ZM2.5 12.25v-2.5h11v2.5h-11Zm.83-3.5h9.34a2.75 2.75 0 0 0-2.24-1.25h-4.86a2.75 2.75 0 0 0-2.24 1.25Zm.8-2a1.5 1.5 0 1 0-1.5-1.5 1.5 1.5 0 0 0 1.5 1.5Z"/>' },
];

const createPackOpen = ref(false);
const CREATE_LOADERS = ["vanilla", "fabric", "quilt", "forge", "neoforge"] as const;
type LoaderKey = (typeof CREATE_LOADERS)[number];
const createPackName = ref("");
const createPackMc = ref("1.21.4");
const createPackLoader = ref<LoaderKey>("fabric");
const createPackBusy = ref(false);
/** "" = последняя версия загрузчика ("Latest"). */
const createPackLoaderVersion = ref("");
const createPackLoaderVersions = ref<string[]>([]);
const createPackLoaderLvOpen = ref(false);
const createPackLvBox = ref<HTMLElement | null>(null);
let createPackLvClose: ((e: MouseEvent) => void) | null = null;
const createPackVersions = ref<McVersionInfo[]>([]);
const createPackVersionOpen = ref(false);
const createPackVersionQuery = ref("");
const createPackVersionBox = ref<HTMLElement | null>(null);
let createPackVersionClose: ((e: MouseEvent) => void) | null = null;
const createPackIcon = ref<string | null>(null);
const createPackBanner = ref<string | null>(null);

/** Фильтры поиска Modrinth (теги грузятся по типам проектов). */
const modrinthTagsMap = ref<Record<string, ModrinthTags | null>>({});
const modrinthTags = computed(() => modrinthTagsMap.value[modSearchKind.value] ?? null);
interface SearchFilterState {
  versions: string[];
  loaders: string[];
  categories: string[];
  versionType: string;
  sort: string;
  env: string;
}
const modFilters = reactive<SearchFilterState>({ versions: [], loaders: [], categories: [], versionType: "", sort: "relevance", env: "" });
const packFilters = reactive<SearchFilterState>({ versions: [], loaders: [], categories: [], versionType: "", sort: "relevance", env: "" });

const versionTypeOptions = [
  { value: "release", label: t("mods.verType.release") },
  { value: "beta", label: t("mods.verType.beta") },
  { value: "alpha", label: t("mods.verType.alpha") },
];
const modVersionTypeSel = computed({
  get: () => (modFilters.versionType ? [modFilters.versionType] : []),
  set: (v: string[]) => { modFilters.versionType = v[0] ?? ""; },
});
const packVersionTypeSel = computed({
  get: () => (packFilters.versionType ? [packFilters.versionType] : []),
  set: (v: string[]) => { packFilters.versionType = v[0] ?? ""; },
});

/** Теги модпаков (для фильтров окна поиска сборок). */
const packTags = computed(() => modrinthTagsMap.value["modpack"] ?? null);
const packVersionOptions = computed(() =>
  [...(packTags.value?.versions ?? [])].sort((a, b) => verCmp(b, a)).map((v) => ({ value: v, label: v }))
);
const packLoaderOptions = computed(() =>
  (packTags.value?.loaders ?? []).map((l) => ({ value: l, label: cap(l) }))
);
const packCategoryOptions = computed(() =>
  (packTags.value?.categories ?? []).map((c) => ({ value: c, label: cap(c) }))
);
function applyPackAutoFilters() {
  const mc = status.value?.minecraft_version;
  if (mc && packVersionOptions.value.some((o) => o.value === mc)) {
    packFilters.versions = [mc];
    cpVersion.value = mc;
  } else {
    packFilters.versions = [];
    cpVersion.value = "";
  }
}

const versionOptions = computed(() =>
  [...(modrinthTags.value?.versions ?? [])].sort((a, b) => verCmp(b, a)).map((v) => ({ value: v, label: v }))
);
const loaderOptions = computed(() =>
  (modrinthTags.value?.loaders ?? [])
    .filter((l) => !SERVER_PLATFORMS.has(l))
    .filter((l) => !(modSearchKind.value === "mod" && l === "datapack"))
    .map((l) => ({ value: l, label: cap(l) }))
);
const autoFiltersDone = ref(false);
/** Автофильтры при открытии поиска: версия Minecraft и загрузчик активной сборки.
 *  Загрузчик учитываем только для модов (ресурспаки/шейдеры часто только vanilla). */
function applyAutoFilters() {
  const mc = status.value?.minecraft_version;
  const loader = status.value?.loader?.replace("-loader", "");
  if (mc && versionOptions.value.some((o) => o.value === mc)) {
    modFilters.versions = [mc];
    curseVersion.value = mc;
  } else {
    modFilters.versions = [];
    curseVersion.value = "";
  }
  if (modSearchKind.value === "mod" && loader && loaderOptions.value.some((o) => o.value === loader)) {
    modFilters.loaders = [loader];
  } else {
    modFilters.loaders = [];
  }
}
/** Запускает первичный поиск при открытии окна: подгружает фильтры/теги,
 *  проставляет автофильтры и наполняет список, чтобы не ждать Enter. */
async function runInitialSearch() {
  if (!isTauri() || !packId.value) return;
  if (searchService.value === "modrinth") {
    await loadModrinthTags(modSearchKind.value);
    applyAutoFilters();
    await searchMods();
  } else {
    // Теги Modrinth нужны как источник списка версий Minecraft для CF-фильтра.
    await loadModrinthTags(modSearchKind.value);
    await loadCurseKeyStatus();
    await loadCurseCategories();
    if (!curseKeyOk.value) return;
    await searchCurse();
  }
}
// В отдельном окне поиска статус и теги грузятся асинхронно — применяем автофильтры
// один раз, когда оба готовы, и сразу запускаем первичный поиск.
watch([status, modrinthTags], () => {
  if (!isSearchWin.value || autoFiltersDone.value) return;
  if (!status.value || !modrinthTags.value) return;
  autoFiltersDone.value = true;
  applyAutoFilters();
  void runInitialSearch();
});
const categoryOptions = computed(() =>
  (modrinthTags.value?.categories ?? [])
    .filter((c) => !(modSearchKind.value === "mod" && SERVER_PLATFORMS.has(c)))
    .map((c) => ({ value: c, label: cap(c) }))
);
/* Серверные платформы/плагины Modrinth (paper/velocity/spigot/... отдаются как загрузчики
 * и категории для типа "mod"). Это серверная, а не клиентская сторона — убираем из фильтров модов. */
const SERVER_PLATFORMS = new Set([
  "spigot",
  "paper",
  "purpur",
  "folia",
  "bukkit",
  "velocity",
  "waterfall",
  "bungeecord",
  "sponge",
  "geyser",
]);
const envOptions = [
  { value: "client", label: t("mods.fClient") },
  { value: "server", label: t("mods.fServer") },
];
const sortOptions = [
  { id: "relevance", labelKey: "mods.sortRelevance" },
  { id: "downloads", labelKey: "mods.sortDownloads" },
  { id: "follows", labelKey: "mods.sortFollows" },
  { id: "newest", labelKey: "mods.sortNewest" },
  { id: "updated", labelKey: "mods.sortUpdated" },
];
const sortSelectOptions = sortOptions.map((s) => ({ value: s.id, label: t(s.labelKey) }));
const modEnvSel = computed({
  get: () => (modFilters.env ? [modFilters.env] : []),
  set: (v: string[]) => {
    modFilters.env = v[0] ?? "";
  },
});
const packEnvSel = computed({
  get: () => (packFilters.env ? [packFilters.env] : []),
  set: (v: string[]) => {
    packFilters.env = v[0] ?? "";
  },
});
const modSortSel = computed({
  get: () => [modFilters.sort],
  set: (v: string[]) => {
    modFilters.sort = v[0] ?? "relevance";
  },
});
const packSortSel = computed({
  get: () => [packFilters.sort],
  set: (v: string[]) => {
    packFilters.sort = v[0] ?? "relevance";
  },
});

/** Общие опции поиска из фильтров. */
function searchOpts(f: SearchFilterState) {
  const opts: { categories?: string[]; loaders?: string[]; versionType?: string; versions?: string[]; environment?: string; index?: string } = {};
  if (f.categories.length) opts.categories = f.categories;
  if (f.loaders.length) opts.loaders = f.loaders;
  if (f.versionType) opts.versionType = f.versionType;
  if (f.versions.length) opts.versions = f.versions;
  if (f.env) opts.environment = f.env;
  if (f.sort && f.sort !== "relevance") opts.index = f.sort;
  return opts;
}

/** Загружает теги Modrinth для типа проекта (по одному разу за сессию). */
async function loadModrinthTags(kind: ModrinthSearchKind = modSearchKind.value) {
  if (!isTauri() || modrinthTagsMap.value[kind]) return;
  try {
    modrinthTagsMap.value = { ...modrinthTagsMap.value, [kind]: await fetchModrinthTags(kind) };
  } catch {
    /* фильтры просто не появятся */
  }
}

/** Открывает окно поиска файлов (Modrinth по умолчанию). */
async function openModSearch(kind: ModrinthSearchKind) {
  await openSearch(kind, "modrinth");
}

/** Папка игры для типа проекта Modrinth. */
const MOD_KIND_FOLDER: Record<ModrinthSearchKind, ModrinthInstallFolder> = {
  mod: "mods",
  modpack: "mods",
  resourcepack: "resourcepacks",
  shaderpack: "shaderpacks",
  datapack: "datapacks",
};

/** Имя типа проекта для сообщений («мод»/«ресурспак»/«шейдер»/«датапак»); по kind или папке. */
function kindNoun(v: ModrinthSearchKind | ModrinthInstallFolder): string {
  switch (v) {
    case "mod":
    case "mods":
      return t("mods.kindMod");
    case "resourcepack":
    case "resourcepacks":
      return t("mods.kindRP");
    case "shaderpack":
    case "shaderpacks":
      return t("mods.kindShaders");
    default:
      return t("mods.kindDatapack");
  }
}

/** Класс проектов CurseForge для типа проекта (моды/ресурспаки/шейдеры). */
const CURSE_CLASS: Partial<Record<ModrinthSearchKind, number>> = {
  mod: 6,
  resourcepack: 12,
  shaderpack: 6552,
};

/** Папка игры для типа проекта CurseForge. */
const CURSE_FOLDER: Partial<Record<ModrinthSearchKind, ModrinthInstallFolder>> = {
  mod: "mods",
  resourcepack: "resourcepacks",
  shaderpack: "shaderpacks",
};

const curseQuery = ref("");
const curseLoading = ref(false);
const curseSearched = ref(false);
const curseResults = ref<CurseSearchHit[]>([]);

// Тёплый кеш иконок для видимых результатов поиска (оба сервиса). Объявлен после
// результатов: watch исполняет геттер сразу при setup и не должен ссылаться
// на ещё не инициализированные ref'ы (TDZ).
watch(
  () => [modSearchResults.value.map((r) => r.iconUrl), curseResults.value.map((r) => r.iconUrl)],
  ([m, c]) => {
    for (const u of [...m, ...c]) {
      if (u) warmSearchIcon(u);
    }
  },
  { deep: true }
);
const curseErr = ref("");
const curseInstallBusy = ref<number | null>(null);
const curseKeyOk = ref(true);

// Установленные в активной сборке проекты — для плашки «Установлено» в поиске.
// Modrinth: slug из трекера (.mono-modrinth.json) или из modrinth_url (`/mod/{slug}`);
// CurseForge: project_id из трекера (.mono-curseforge.json).
const installedModrinthSlugs = computed(() => {
  const set = new Set<string>();
  for (const list of Object.values(gameFiles.value)) {
    for (const f of list) {
      if (f.modrinthProjectId) set.add(f.modrinthProjectId);
      const m = /\/mod\/([^/]+)\/?$/.exec(f.modrinthUrl ?? "");
      if (m) set.add(m[1]);
    }
  }
  return set;
});
const installedCurseIds = computed(() => {
  const set = new Set<number>();
  for (const list of Object.values(gameFiles.value)) {
    for (const f of list) {
      if (f.curseforgeProjectId) set.add(f.curseforgeProjectId);
    }
  }
  return set;
});

async function loadCurseKeyStatus() {
  if (!isTauri()) return;
  try {
    curseKeyOk.value = await curseforgeKeyConfigured();
  } catch {
    curseKeyOk.value = false;
  }
}

/** Категории CurseForge для фильтра (грузим по классу проекта). */
const curseCatOptions = ref<{ value: string; label: string }[]>([]);
const curseCatId = ref<number | null>(null);
const curseCatSel = computed({
  get: () => (curseCatId.value !== null ? [String(curseCatId.value)] : []),
  set: (v: string[]) => {
    curseCatId.value = v[0] ? Number(v[0]) : null;
  },
});
/** Загрузчики CurseForge (выделенные из категорий). */
const curseLoaderOptions = ref<{ value: string; label: string }[]>([]);
const curseLoaderId = ref<number | null>(null);
const curseLoaderSel = computed({
  get: () => (curseLoaderId.value !== null ? [String(curseLoaderId.value)] : []),
  set: (v: string[]) => {
    curseLoaderId.value = v[0] ? Number(v[0]) : null;
  },
});
async function loadCurseCategories() {
  if (!isTauri()) return;
  const cls = CURSE_CLASS[modSearchKind.value] ?? 6;
  curseCatOptions.value = [];
  curseLoaderOptions.value = [];
  curseCatId.value = null;
  curseLoaderId.value = null;
  try {
    const cats = await curseforgeCategories(cls);
    const loaderKeywords = ["forge", "fabric", "neoforge", "quilt", "liteloader", "rift", "modloader", "mcp"];
    for (const c of cats) {
      const lower = c.name.toLowerCase();
      const isLoader = loaderKeywords.some((k) => lower.includes(k));
      if (isLoader) {
        curseLoaderOptions.value.push({ value: String(c.id), label: c.name });
      } else {
        curseCatOptions.value.push({ value: String(c.id), label: c.name });
      }
    }
  } catch {
    /* фильтр просто не появится */
  }
}

/** Фильтры CurseForge: версия Minecraft и сортировка. */
const curseVersion = ref("");
const curseVerSel = computed({
  get: () => (curseVersion.value ? [curseVersion.value] : []),
  set: (v: string[]) => {
    curseVersion.value = v[0] ?? "";
  },
});
const curseSortField = ref<string>("2");
const curseSortSel = computed({
  get: () => [curseSortField.value],
  set: (v: string[]) => {
    curseSortField.value = v[0] ?? "2";
  },
});
/* CurseForge ModsSearchSortField: 1 Featured, 2 Popularity, 3 LastUpdated, 4 Name, 6 TotalDownloads. */
const CURSE_SORT = [
  { value: "2", labelKey: "mods.sortRelevance" },
  { value: "6", labelKey: "mods.sortDownloads" },
  { value: "3", labelKey: "mods.sortNewest" },
  { value: "4", labelKey: "mods.sortName" },
];
const curseSortOptions = CURSE_SORT.map((s) => ({ value: s.value, label: t(s.labelKey) }));

/** Поиск на CurseForge. */
async function searchCurse() {
  if (!isTauri() || !packId.value) return;
  modDetail.value = null;
  curseLoading.value = true;
  curseSearched.value = true;
  curseErr.value = "";
  try {
    const categoryId = curseLoaderId.value ?? curseCatId.value;
    curseResults.value = await curseforgeSearch(
      curseQuery.value.trim(),
      CURSE_CLASS[modSearchKind.value] ?? 6,
      categoryId,
      curseVersion.value || undefined,
      curseSortField.value
    );
  } catch (e) {
    curseResults.value = [];
    curseErr.value = String(e);
  } finally {
    curseLoading.value = false;
  }
}

/** Скачивает последний подходящий файл проекта CurseForge в папку вкладки.
 *  Возвращает результат установки (false — ошибка/нет данных). */
async function installCurseCore(p: CurseSearchHit): Promise<CurseInstallResult | null> {
  if (!isTauri() || !packId.value) return null;
  curseInstallBusy.value = p.projectId;
  try {
    const file = await curseforgeLatestFile(packId.value, p.projectId);
    const folder = (CURSE_FOLDER[modSearchKind.value] ?? "mods") as GameFolderKind;
    return await curseforgeInstallFile(packId.value, file, folder, p.name, p.iconUrl);
  } finally {
    curseInstallBusy.value = null;
  }
}

/** Скачивает один проект CurseForge с уведомлением и обновлением списка файлов. */
async function installCurse(p: CurseSearchHit) {
  if (!isTauri() || !packId.value || curseInstallBusy.value !== null) return;
  try {
    const res = await installCurseCore(p);
    if (!res) {
      notify(t("curse.installErr", { e: "unknown" }));
      return;
    }
    notify(
      res.depsInstalled > 0
        ? t("curse.installedDeps", { name: p.name, deps: res.depsInstalled })
        : t("curse.installed", { name: p.name }),
      "success"
    );
    // Сразу сохраняем мету из поискового хита (название + иконка) в кеш, чтобы
    // main-окно показало их без отдельного API-запроса project_detail.
    curseMeta.value = { ...curseMeta.value, [p.projectId]: { title: p.name, icon: p.iconUrl ?? "" } };
    setCachedIcon(`cf:${p.projectId}`, JSON.stringify({ title: p.name, icon: p.iconUrl ?? "" }));
    closeSearch();
    const folder = (CURSE_FOLDER[modSearchKind.value] ?? "mods") as GameFolderKind;
    await loadGameFiles(folder, true);
    await refreshModUpdates(true);
  } catch (e) {
    notify(t("curse.installErr", { e }));
  }
}

/** Активны ли фильтры/запрос поиска (для показа кнопки «сбросить» в пустом списке). */
const modFiltersActive = computed(() => {
  const f = modFilters;
  return (
    !!modSearchQuery.value.trim() ||
    f.categories.length > 0 ||
    f.loaders.length > 0 ||
    f.versions.length > 0 ||
    !!f.versionType ||
    f.sort !== "relevance" ||
    !!f.env
  );
});

/** Сбрасывает фильтры и запрос поиска Modrinth и запускает поиск заново. */
function resetModFiltersAndSearch() {
  modFilters.versions = [];
  modFilters.loaders = [];
  modFilters.categories = [];
  modFilters.env = "";
  modFilters.sort = "relevance";
  modFilters.versionType = "";
  modSearchQuery.value = "";
  modSearchErr.value = "";
  void searchMods();
}

/** Поиск модов/ресурспаков/шейдеров/датапаков для добавления в сборку. */
async function searchMods() {
  if (!isTauri() || !packId.value) return;
  modDetail.value = null;
  const gen = ++modSearchGen;
  modSearchLoading.value = true;
  modSearchErr.value = "";
  modSearchMore.value = false;
  try {
    modSearchResults.value = await modrinthSearch(
      modSearchQuery.value.trim(),
      modSearchKind.value,
      MOD_SEARCH_PAGE,
      searchOpts(modFilters),
      0
    );
    if (gen !== modSearchGen) return;
    modSearchOffset.value = modSearchResults.value.length;
    modSearchMore.value = modSearchResults.value.length >= MOD_SEARCH_PAGE;
    modSearchBox.value?.scrollTo({ top: 0 });
  } catch (e) {
    if (gen === modSearchGen) modSearchErr.value = String(e);
  } finally {
    if (gen === modSearchGen) modSearchLoading.value = false;
  }
}

/** Догрузка следующей страницы результатов (бесконечный поиск). */
async function loadMoreMods() {
  if (modSearchMoreBusy.value || modSearchLoading.value || !modSearchMore.value) return;
  const gen = modSearchGen;
  modSearchMoreBusy.value = true;
  try {
    const page = await modrinthSearch(
      modSearchQuery.value.trim(),
      modSearchKind.value,
      MOD_SEARCH_PAGE,
      searchOpts(modFilters),
      modSearchOffset.value
    );
    if (gen !== modSearchGen) return;
    if (page.length) modSearchResults.value.push(...page);
    modSearchOffset.value += page.length;
    modSearchMore.value = page.length >= MOD_SEARCH_PAGE;
  } catch {
    /* не критично: пользователь увидит кнопку «Ещё» и сможет повторить */
  } finally {
    modSearchMoreBusy.value = false;
  }
}

function onModSearchScroll() {
  const el = modSearchBox.value;
  if (!el || modSearchMoreBusy.value || !modSearchMore.value) return;
  if (el.scrollTop + el.clientHeight >= el.scrollHeight - 160) loadMoreMods();
}

function modUpdateFor(f: GameFileEntry): ModUpdate | undefined {
  return updatesByFile.value.get(f.name);
}

/** Индекс обновлений по имени файла (O(1) вместо линейного поиска на строку). */
const updatesByFile = computed(() => {
  const map = new Map<string, ModUpdate>();
  for (const u of modUpdates.value) map.set(u.fileName, u);
  return map;
});

/** Версии мода: сперва подходящие под версию сборки, остальные ниже.
 *  По загрузчику фильтруем только моды — ресурспаки/шейдеры/датапаки
 *  часто поддержаны только на vanilla, даже в fabric-сборках. */
async function openModVersions(p: ModrinthProject) {
  modVersions.value = null;
  verFilterMc.value = "";
  verFilterLoader.value = "";
  verFilterType.value = "";
  try {
    const all = await modrinthProjectVersions(p.projectId);
    modVersionsRaw.value = all;
    const mc = status.value?.minecraft_version;
    const loader = status.value?.loader?.replace("-loader", "");
    const kind = modSearchKind.value;
    const matchLoader = kind === "mod" && loader ? (v: ModrinthVersion) => v.loaders.includes(loader) : () => true;
    const match = mc
      ? all.filter((v) => v.gameVersions.includes(mc) && matchLoader(v))
      : all;
    const rest = mc ? all.filter((v) => !match.includes(v)) : [];
    modVersions.value = [...match, ...rest];
  } catch (e) {
    modSearchErr.value = String(e);
    modVersions.value = [];
    modVersionsRaw.value = [];
  }
}

/** Открывает «страницу» ресурса: вкладки описание/версии/галерея (как в сборках). */
const modDetail = ref<ModrinthProject | null>(null);
const modDetailTab = ref<"about" | "versions" | "gallery">("about");
const modDetailTabs: { kind: "about" | "versions" | "gallery" }[] = [
  { kind: "about" },
  { kind: "versions" },
  { kind: "gallery" },
];

async function openModDetail(p: ModrinthProject) {
  modDetail.value = p;
  modDetailTab.value = "about";
  modVersions.value = null;
  openModVersions(p);
  if (!p.body) {
    try {
      modDetail.value = await modrinthProject(p.projectId);
    } catch {
      /* остаётся карточка из поиска */
    }
  }
}

/** Мир, в который ставятся датапаки. */
const modDatapackWorld = ref<string | null>(null);
const datapackWorlds = computed(() => (gameFiles.value.saves ?? []).filter((s) => s.kind === "dir").map((s) => s.name));
const worldOptions = computed(() => datapackWorlds.value.map((w) => ({ value: w, label: w })));
const modDatapackWorldSel = computed({
  get: () => (modDatapackWorld.value ? [modDatapackWorld.value] : []),
  set: (v: string[]) => {
    modDatapackWorld.value = v[0] ?? null;
  },
});

/** Устанавливает выбранную версию в папку активной сборки
 *  (датапаки — в saves/<мир>/datapacks). */
async function installModVersion(v: ModrinthVersion, closeAfter = true) {
  if (modInstallBusy.value) return;
  if (!packId.value) {
    notify(t("mods.noPack"), "error");
    return;
  }
  const folder = MOD_KIND_FOLDER[modSearchKind.value];
  const world = modSearchKind.value === "datapack" ? (modDatapackWorld.value ?? undefined) : undefined;
  if (modSearchKind.value === "datapack" && !world) {
    notify(t("mods.pickWorld"), "info");
    return false;
  }
  modInstallBusy.value = v.id;
  try {
    await modrinthInstallMod(packId.value, v.id, folder, world);
    notify(t("mods.installed", { kind: kindNoun(modSearchKind.value), name: v.name }), "success");
    if (closeAfter) closeSearch();
    modVersions.value = null;
    if (folder !== "datapacks") {
      await loadGameFiles(folder, true);
    } else {
      await loadGameFiles("saves", true);
    }
    await refreshModUpdates(true);
    return true;
  } catch (e) {
    notify(t("mods.installErr", { kind: kindNoun(modSearchKind.value), e }));
    return false;
  } finally {
    modInstallBusy.value = null;
  }
}

/** Подбирает версию проекта под версию Minecraft и загрузчик сборки
 *  (загрузчик учитываем только для модов). */
async function pickModVersion(p: ModrinthProject): Promise<ModrinthVersion | null> {
  const isMod = modSearchKind.value === "mod";
  const mc = status.value?.minecraft_version || undefined;
  // Без активной версии Minecraft нельзя выбрать «самую свежую» версию мода —
  // это приведёт к установке файла под более новую (или иную) версию игры.
  if (!mc) return null;
  const loader = status.value?.loader?.replace("-loader", "");
  // Версии под текущую версию Minecraft (и загрузчик для модов) фильтруем на
  // сервере — так не получится скачать файл под другую версию игры.
  const reqLoader = isMod && loader ? loader : undefined;
  let all = await modrinthProjectVersions(p.projectId, mc, reqLoader);
  if (all.length === 0 && mc) {
    // Под текущую MC нет версии с нужным загрузчиком — пробуем любой загрузчик.
    all = await modrinthProjectVersions(p.projectId, mc, undefined);
  }
  if (all.length === 0) return null;
  // project_versions возвращает от новых к старым — берём самую свежую под MC.
  return all[0];
}

/** Быстрое скачивание мода: последняя версия под MC и загрузчик сборки. */
const quickModBusy = ref<string | null>(null);
async function quickDownloadMod(p: ModrinthProject, ev: Event) {
  ev.stopPropagation();
  if (quickModBusy.value) return;
  if (!packId.value) {
    notify(t("mods.noPack"), "error");
    return;
  }
  quickModBusy.value = p.projectId;
  try {
    const pick = await pickModVersion(p);
    if (!pick) {
      notify(t("mods.noMatchVersion"), "info");
      return;
    }
    await installModVersion(pick);
  } catch (e) {
    notify(t("mods.installErr", { kind: kindNoun(modSearchKind.value), e }));
  } finally {
    quickModBusy.value = null;
  }
}

// ---- Мультивыбор в поиске: скачивание сразу нескольких ресурсов ----
const selModrinth = ref<Set<string>>(new Set());
const selCurse = ref<Set<number>>(new Set());
const multiSelBusy = ref(false);

function toggleModrinthSel(id: string) {
  const s = new Set(selModrinth.value);
  if (s.has(id)) s.delete(id);
  else s.add(id);
  selModrinth.value = s;
}

function toggleCurseSel(id: number) {
  const s = new Set(selCurse.value);
  if (s.has(id)) s.delete(id);
  else s.add(id);
  selCurse.value = s;
}

function clearSelAll() {
  selModrinth.value = new Set();
  selCurse.value = new Set();
}

/** Скачивает все выделенные проекты Modrinth подряд (последние подходящие версии). */
async function downloadSelectedMods() {
  if (!isTauri() || !packId.value || multiSelBusy.value) return;
  if (modSearchKind.value === "datapack" && !modDatapackWorld.value) {
    notify(t("mods.pickWorld"), "info");
    return;
  }
  const ids = [...selModrinth.value];
  if (ids.length === 0) return;
  multiSelBusy.value = true;
  let ok = 0;
  for (const id of ids) {
    const p = modSearchResults.value.find((r) => r.projectId === id);
    if (!p) continue;
    try {
      const pick = await pickModVersion(p);
      if (!pick) continue;
      if (await installModVersion(pick, false)) ok++;
    } catch (e) {
      notify(t("mods.installErr", { kind: kindNoun(modSearchKind.value), e }));
    }
  }
  multiSelBusy.value = false;
  clearSelAll();
  closeSearch();
  if (ok > 0) notify(t("mods.installedSel", { n: ok }), "success");
}

/** Скачивает все выделенные проекты CurseForge подряд (последние подходящие файлы). */
async function downloadSelectedCurse() {
  if (!isTauri() || !packId.value || multiSelBusy.value) return;
  const ids = [...selCurse.value];
  if (ids.length === 0) return;
  multiSelBusy.value = true;
  let ok = 0;
  for (const id of ids) {
    const p = curseResults.value.find((r) => r.projectId === id);
    if (!p) continue;
    try {
      if ((await installCurseCore(p)) !== null) ok++;
    } catch (e) {
      notify(t("curse.installErr", { e }));
    }
  }
  multiSelBusy.value = false;
  clearSelAll();
  closeSearch();
  if (ok > 0) notify(t("mods.installedSel", { n: ok }), "success");
}

/** Быстрое скачивание модпака: последняя версия (с учётом выбранного загрузчика). */
const quickPackBusy = ref<string | null>(null);
async function quickDownloadPack(p: ModrinthProject, ev: Event) {
  ev.stopPropagation();
  if (quickPackBusy.value || modPackInstalling.value) return;
  quickPackBusy.value = p.projectId;
  try {
    const all = await modrinthProjectVersions(p.projectId);
    const sorted = [...all].sort((a, b) => Date.parse(b.datePublished) - Date.parse(a.datePublished));
    const pick = packFilters.loaders[0]
      ? sorted.find((v) => v.loaders.includes(packFilters.loaders[0]))
      : sorted[0];
    if (!pick) {
      notify(t("mods.noMatchVersion"), "info");
      return;
    }
    await installPackVersion(pick);
  } catch (e) {
    notify(t("mods.packInstallErr", { e }));
  } finally {
    quickPackBusy.value = null;
  }
}

/** Проверяет обновления установленных из Modrinth модов (с кешем на 5 минут). */
const updatesCheckedAt = ref(0);
const UPDATES_TTL_MS = 5 * 60 * 1000;
async function refreshModUpdates(force = false) {
  if (!isTauri() || !packId.value || !status.value?.installed) {
    modUpdates.value = [];
    trackedMods.value = [];
    return;
  }
  if (!force && updatesCheckedAt.value && Date.now() - updatesCheckedAt.value < UPDATES_TTL_MS) return;
  try {
    modUpdates.value = await modrinthCheckUpdates(packId.value);
    updatesCheckedAt.value = Date.now();
  } catch {
    modUpdates.value = [];
  }
}

/** Обновляет один мод (папку берём из записи обновления). */
async function updateOneMod(u: ModUpdate) {
  if (!packId.value || updatingMod.value) return;
  updatingMod.value = u.fileName;
  try {
    await modrinthUpdateMod(packId.value, u.fileName);
    notify(t("mods.updated", { kind: kindNoun(u.folder as ModrinthInstallFolder), name: u.newVersion.name }), "success");
    await loadGameFiles(u.folder === "datapacks" ? "saves" : (u.folder as GameFolderKind), true);
    await refreshModUpdates(true);
  } catch (e) {
    notify(t("mods.updateErr", { kind: kindNoun(u.folder as ModrinthInstallFolder), e }));
  } finally {
    updatingMod.value = null;
  }
}

/** Обновляет все моды текущей вкладки (последовательно). */
async function updateAllMods() {
  if (!packId.value || updateAllBusy.value || modUpdatesTab.value.length === 0) return;
  updateAllBusy.value = true;
  let ok = 0;
  let fail = 0;
  for (const u of [...modUpdatesTab.value]) {
    try {
      await modrinthUpdateMod(packId.value, u.fileName);
      ok++;
    } catch {
      fail++;
    }
  }
  notify(
    ok > 0
      ? t("mods.updatedCount", { ok, fail })
      : t("mods.updateAllFail", { fail }),
    fail > 0 && ok === 0 ? "error" : "success"
  );
  const tabs: GameFolderKind[] = ["mods", "resourcepacks", "shaderpacks", "saves"];
  if ((tabs as string[]).includes(playSubTab.value)) {
    await loadGameFiles(playSubTab.value as GameFolderKind, true);
  }
  await refreshModUpdates(true);
  updateAllBusy.value = false;
}

/** Поиск модпаков на Modrinth для установки как сборки. */
async function searchPacks() {
  if (!isTauri()) return;
  modPackLoading.value = true;
  modPackDetail.value = null;
  modPackVersions.value = null;
  try {
    modPackResults.value = await modrinthSearch(
      modPackQuery.value.trim(),
      "modpack",
      20,
      searchOpts(packFilters)
    );
  } catch (e) {
    notify(t("mods.packsSearchErr", { e }));
  } finally {
    modPackLoading.value = false;
  }
}

/** Версии модпака (по убыванию даты). */
async function openPackVersions(p: ModrinthProject) {
  modPackVersions.value = null;
  try {
    const all = await modrinthProjectVersions(p.projectId);
    modPackVersions.value = all.sort(
      (a, b) => Date.parse(b.datePublished) - Date.parse(a.datePublished)
    );
  } catch (e) {
    notify(t("mods.packsSearchErr", { e }));
    modPackVersions.value = [];
  }
}

/** Открывает «страницу» сборки: вкладки описание/версии/галерея. */
async function openPackDetail(p: ModrinthProject) {
  modPackDetail.value = p;
  modPackTab.value = "about";
  modPackVersions.value = null;
  openPackVersions(p);
  if (!p.body) {
    try {
      modPackDetail.value = await modrinthProject(p.projectId);
    } catch {
      /* оставляем карточку из поиска */
    }
  }
}

/** Скачивает и устанавливает модпак с Modrinth. */
async function installPackVersion(v: ModrinthVersion) {
  if (modPackInstalling.value) return;
  modPackInstalling.value = v.id;
  try {
    const pack = await modrinthInstallPack(v.id);
    notify(t("mods.packInstalled", { name: pack.name }), "success");
    modPackOpen.value = false;
    modPackVersions.value = null;
    modPackDetail.value = null;
    await loadPacks();
    await nextTick();
    openPackTab(pack.id);
  } catch (e) {
    notify(t("mods.packInstallErr", { e }));
  } finally {
    modPackInstalling.value = null;
  }
}

/** Создаёт свою (локальную) сборку: база + загрузчик. */
async function createPack() {
  if (createPackBusy.value) return;
  const name = createPackName.value.trim();
  if (!name) {
    notify(t("mods.createName"), "info");
    return;
  }
  createPackBusy.value = true;
  try {
    const pack = await createLocalPack(
      name,
      createPackMc.value.trim(),
      createPackLoader.value,
      createPackIcon.value,
      createPackBanner.value,
      createPackLoaderVersion.value || null
    );
    notify(t("mods.packCreated", { name: pack.name }), "success");
    createPackOpen.value = false;
    createPackName.value = "";
    createPackIcon.value = null;
    createPackBanner.value = null;
    await loadPacks();
    await nextTick();
    openPackTab(pack.id);
  } catch (e) {
    notify(t("mods.createErr", { e }));
  } finally {
    createPackBusy.value = false;
  }
}

// ---- Смена версии Minecraft / загрузчика / версии загрузчика у своей сборки ----
const editVerOpen = ref(false);
const editVerBusy = ref(false);
const editVerName = ref("");
const exportBusy = ref(false);
const exportUpload = ref(false);
const authorNewTitle = ref("");
const authorNewBody = ref("");
const authorNewVersion = ref("");
const authorNewChangelog = ref("");
const authorVersionFile = ref("");
async function pickAuthorVersionFile() {
  const picked = await openDialog({
    filters: [{ name: "Modrinth Pack", extensions: ["mrpack", "zip"] }],
  });
  if (typeof picked === "string") authorVersionFile.value = picked;
}

// ---- Комментарии каталога: черновики/ответы/редактирование ----
const commentDraft = ref("");
const commentReplyTo = ref<string | null>(null);
const commentReplyDraft = ref("");
const commentEditId = ref<string | null>(null);
const commentEditDraft = ref("");

// ---- Сканер модов ----
const scannerOpen = ref(false);
const scannerHash = ref("");

function openModScanner() {
  scannerHash.value = "";
  scannerOpen.value = true;
}

async function pickAndScanJar() {
  const picked = await openDialog({
    filters: [{ name: "Java Archive", extensions: ["jar"] }],
  });
  if (typeof picked === "string") await scanModFile(picked);
}

// ---- Профиль: редактирование bio ----
const profileBioEditing = ref(false);
const profileBioDraft = ref("");

/** Открыть сборку из профиля в каталоге (если она есть в каталоге Mono). */
async function openCatalogPackById(id: string) {
  if (monoCatalog.value.length === 0) {
    await loadMonoCatalog();
  }
  const entry = monoCatalog.value.find((p) => p.id === id);
  if (!entry) {
    notify(t("profile.packMissing"), "info");
    return;
  }
  closeProfileView();
  tab.value = "catalog";
  void openCatalogDetail(entry);
}

// ---- Соавторы: форма добавления ----
const collabName = ref("");
const collabPerms = reactive({ permEditMeta: true, permManageVersions: true, permManageNews: false });

// ---- Скриншоты в панели автора (meta.screenshots) ----
const authorShotUrl = ref("");

const authorShots = computed<string[]>(() => {
  const meta = authorDetail.value?.meta as Record<string, unknown> | null | undefined;
  const shots = meta && Array.isArray(meta.screenshots) ? meta.screenshots : [];
  return (shots as unknown[])
    .map((s) => (typeof s === "string" ? s : (s as { url?: string } | null)?.url ?? ""))
    .filter(Boolean);
});

function authorShotsPayload(next: string[]): Record<string, unknown> {
  return { ...(authorDetail.value?.meta ?? {}), screenshots: next };
}

async function addAuthorShot() {
  const url = authorShotUrl.value.trim();
  if (!url || !authorDetail.value) return;
  authorShotUrl.value = "";
  await updateAuthorMeta({ meta: authorShotsPayload([...authorShots.value, url]) });
}

async function removeAuthorShot(index: number) {
  if (!authorDetail.value) return;
  const next = authorShots.value.filter((_, i) => i !== index);
  await updateAuthorMeta({ meta: authorShotsPayload(next) });
}

// ---- Админ: inline-причина бана ----
const adminBanArmed = ref<string | null>(null);
const adminBanReason = ref("");

const authorImportFile = ref("");
const authorImportVersion = ref("");
const authorImportChangelog = ref("");
const authorImportMode = ref(false);
async function pickAuthorImportFile() {
  const picked = await openDialog({
    filters: [{ name: "Modrinth Pack", extensions: ["mrpack", "zip"] }],
  });
  if (typeof picked === "string") {
    authorImportFile.value = picked;
    const base = picked.split("/").pop()?.replace(/\.(mrpack|zip)$/i, "") ?? "";
    authorName.value = base;
    authorAuthor.value = "";
    authorDesc.value = "";
    authorBoosty.value = "";
    authorIcon.value = "";
    authorBanner.value = "";
    authorMinRam.value = false;
    authorMinRamMb.value = null;
    authorServers.value = [{ name: "", ip: "", port: null, desc: "" }];
    authorSocials.value = [{ name: "", url: "", color: "" }];
    authorTheme.value = {};
    authorAccent.value = "";
    authorImportVersion.value = "";
    authorImportChangelog.value = "";
    authorImportMode.value = true;
    exportFormat.value = "author";
    exportOpen.value = true;
  }
}

async function doAuthorImport() {
  if (!authorImportFile.value || exportBusy.value) return;
  const cfg = authorConfig();
  const meta: Record<string, unknown> = {
    theme: cfg.theme,
    servers: cfg.servers,
    socials: cfg.socials,
  };
  if (authorBanner.value.trim()) meta.banner = authorBanner.value.trim();
  const ok = await importAuthorPack(
    authorImportFile.value,
    cfg.name,
    cfg.description ?? "",
    authorImportVersion.value,
    authorImportChangelog.value,
    {
      minRamMb: cfg.minRam,
      boostyBlog: cfg.boostyBlog,
      meta,
      iconUrl: authorIcon.value.trim() ? authorIcon.value.trim() : null,
    }
  );
  if (ok) exportOpen.value = false;
}
const exportOpen = ref(false);
const exportFormat = ref<"mrpack" | "curseforge" | "author">("mrpack");
const exportLoading = ref(false);
const exportItems = ref<ExportSourceItem[]>([]);
const exportSelected = ref(new Set<string>());
const exportExpanded = ref(new Set<string>());
const exportVersionNum = ref("1.0.0");
const exportName = ref("");
const exportAllChecked = computed(() => exportItems.value.length > 0 && exportItems.value.every((it) => exportSelected.value.has(it.path)));

/** Дети узла дерева экспорта (по префиксу пути). */
function exportChildrenOf(parent: string): ExportSourceItem[] {
  const pref = parent ? `${parent}/` : "";
  return exportItems.value
    .filter((it) => it.path.startsWith(pref) && it.path.slice(pref.length).indexOf("/") === -1)
    .sort((a, b) => Number(b.isDir) - Number(a.isDir) || a.path.localeCompare(b.path));
}

/** Все потомки узла (включая сам узел). */
function exportDescendantsOf(path: string): string[] {
  const pref = path ? `${path}/` : "";
  return exportItems.value
    .filter((it) => it.path === path || it.path.startsWith(pref))
    .map((it) => it.path);
}

/** Видимые строки дерева (preorder, с учётом развёрнутых папок). */
const exportVisibleRows = computed(() => {
  const rows: { it: ExportSourceItem; depth: number }[] = [];
  const walk = (parent: string, depth: number) => {
    for (const it of exportChildrenOf(parent)) {
      rows.push({ it, depth });
      if (it.isDir && exportExpanded.value.has(it.path)) walk(it.path, depth + 1);
    }
  };
  walk("", 0);
  return rows;
});

/** Множество выбранных потомков узла (для неопределённого состояния чекбокса папки). */
function exportSelectedCount(path: string): { selected: number; total: number } {
  const kids = exportChildrenOf(path);
  if (!kids.length) {
    return exportSelected.value.has(path) ? { selected: 1, total: 1 } : { selected: 0, total: 1 };
  }
  let selected = 0;
  let total = 0;
  for (const k of kids) {
    const [s, t] = k.isDir ? [exportSelectedCount(k.path).selected, exportSelectedCount(k.path).total] : [Number(exportSelected.value.has(k.path)), 1];
    selected += s;
    total += t;
  }
  return { selected, total };
}
const editVerMc = ref("");
const editVerLoader = ref<LoaderKey>("vanilla");
const editVerLv = ref("");
const editVerMcVersions = ref<McVersionInfo[]>([]);
const editVerLoaderVersions = ref<string[]>([]);
const editVerMcOpen = ref(false);
const editVerMcQuery = ref("");
const editVerMcBox = ref<HTMLElement | null>(null);
let editVerMcClose: ((e: MouseEvent) => void) | null = null;
const editVerLvOpen = ref(false);
const editVerLvBox = ref<HTMLElement | null>(null);
let editVerLvClose: ((e: MouseEvent) => void) | null = null;
const editVerMcList = computed(() => {
  const q = editVerMcQuery.value.trim().toLowerCase();
  const list = editVerMcVersions.value.filter((v) => !q || v.id.toLowerCase().includes(q));
  return list;
});

function openEditVersion() {
  editVerMc.value = status.value?.minecraft_version || "";
  editVerLoader.value = (status.value?.loader as LoaderKey) || "vanilla";
  editVerLv.value = status.value?.loader_version || "";
  editVerName.value = activePack?.value?.name || "";
  editVerOpen.value = true;
  if (editVerMcVersions.value.length) return;
  void minecraftVersions()
    .then((v) => (editVerMcVersions.value = v))
    .catch(() => {});
}

function chooseEditVerMc(id: string) {
  editVerMc.value = id;
  editVerMcOpen.value = false;
  editVerMcQuery.value = "";
}

function chooseEditVerLoaderVersion(v: string) {
  editVerLv.value = v;
  editVerLvOpen.value = false;
}

/** При смене версии/загрузчика — грузим доступные версии загрузчика. */
watch([editVerLoader, editVerMc], async ([loader, mc]) => {
  editVerLv.value = "";
  editVerLoaderVersions.value = [];
  if (editVerOpen.value && loader !== "vanilla" && mc.trim()) {
    try {
      editVerLoaderVersions.value = await localLoaderVersions(loader, mc.trim());
    } catch {
      editVerLoaderVersions.value = [];
    }
  }
});

/** Закрываем выпадашки при клике вне их. */
watch(editVerMcOpen, (open) => {
  if (editVerMcClose) {
    document.removeEventListener("mousedown", editVerMcClose);
    editVerMcClose = null;
  }
  if (open) {
    editVerMcClose = (e: MouseEvent) => {
      if (editVerMcBox.value && !editVerMcBox.value.contains(e.target as Node)) {
        editVerMcOpen.value = false;
        editVerMcQuery.value = "";
      }
    };
    document.addEventListener("mousedown", editVerMcClose);
  }
});
watch(editVerLvOpen, (open) => {
  if (editVerLvClose) {
    document.removeEventListener("mousedown", editVerLvClose);
    editVerLvClose = null;
  }
  if (open) {
    editVerLvClose = (e: MouseEvent) => {
      if (editVerLvBox.value && !editVerLvBox.value.contains(e.target as Node)) {
        editVerLvOpen.value = false;
      }
    };
    document.addEventListener("mousedown", editVerLvClose);
  }
});

/** Открывает диалог выбора папок/файлов, версии и имени перед экспортом сборки. */
async function openExport(format: "mrpack" | "curseforge") {
  if (exportBusy.value || !packId.value || !isTauri()) return;
  exportFormat.value = format;
  exportOpen.value = true;
  exportName.value = activePack?.value?.name || "pack";
  exportExpanded.value = new Set();
  await loadExportList();
}

async function loadExportList() {
  if (!packId.value) return;
  exportLoading.value = true;
  try {
    const items = await exportSourceList(packId.value, "");
    exportItems.value = items;
    const sel = new Set<string>();
    for (const it of items) if (it.defaultIncluded) sel.add(it.path);
    exportSelected.value = sel;
  } catch (e) {
    notify(t("pack.exportListErr", { e }));
  } finally {
    exportLoading.value = false;
  }
}

/** Развернуть/свернуть папку в дереве. */
function toggleExportExpand(path: string) {
  const ex = new Set(exportExpanded.value);
  if (ex.has(path)) ex.delete(path);
  else ex.add(path);
  exportExpanded.value = ex;
}

function toggleExport(path: string) {
  const it = exportItems.value.find((x) => x.path === path);
  if (!it) return;
  const sel = new Set(exportSelected.value);
  if (it.isDir) {
    const all = exportDescendantsOf(path);
    if (all.every((p) => sel.has(p))) for (const p of all) sel.delete(p);
    else for (const p of all) sel.add(p);
  } else {
    if (sel.has(path)) sel.delete(path);
    else sel.add(path);
  }
  exportSelected.value = sel;
}

function toggleExportAll() {
  if (exportAllChecked.value) exportSelected.value = new Set();
  else exportSelected.value = new Set(exportItems.value.map((it) => it.path));
}

/** Подтверждает выбор, показывает диалог сохранения и запускает экспорт. */
async function doExport() {
  if (exportBusy.value || !packId.value || !isTauri()) return;
  const format = exportFormat.value === "author" ? "mrpack" : exportFormat.value;
  const include = [...exportSelected.value];
  const name = exportName.value.trim() || activePack?.value?.name || "pack";
  const ext = format === "mrpack" ? "mrpack" : "zip";
  const dest = await save({
    defaultPath: `${name}.${ext}`,
    filters: format === "mrpack"
      ? [{ name: "MRPack", extensions: ["mrpack"] }]
      : [{ name: "ZIP", extensions: ["zip"] }],
  });
  if (!dest) return;
  exportBusy.value = true;
  try {
    await exportPackFn(packId.value, "", format, dest, include, name, exportVersionNum.value.trim() || "1.0.0");
    if (exportUpload.value && monoProfile.value) {
      try {
        const pack = await uploadPack(monoProfile.value.access_token, dest, name, "");
        notify(t("pack.uploadDone"), "success");
        void openExternal(pack.url);
      } catch (e) {
        notify(t("pack.uploadErr", { e }), "error");
      }
    } else {
      notify(t("pack.exportDone"), "success");
    }
    exportOpen.value = false;
  } catch (e) {
    notify(t("pack.exportErr", { e }));
  } finally {
    exportBusy.value = false;
    exportUpload.value = false;
  }
}

/* ================= Экспорт «авторской» сборки ================= */
const authorName = ref("");
const authorAuthor = ref("");
const authorDesc = ref("");
const authorBoosty = ref("");
const authorIcon = ref("");
const authorBanner = ref("");
const authorMinRam = ref(false);
const authorMinRamMb = ref<number | null>(null);
const authorServers = ref<AuthorServer[]>([{ name: "", ip: "", port: null, desc: "" }]);
const authorSocials = ref<AuthorSocial[]>([{ name: "", url: "", color: "" }]);
const authorTheme = ref<AuthorTheme>({});
const authorAccent = ref("");
const AUTHOR_MAX_SERVERS = 5;
const AUTHOR_MAX_SOCIALS = 4;

function openAuthorExport() {
  if (exportBusy.value || !packId.value || !isTauri()) return;
  authorImportMode.value = false;
  exportFormat.value = "author";
  authorName.value = activePack?.value?.name || "pack";
  authorAuthor.value = activePack?.value?.author || "";
  authorDesc.value = "";
  authorBoosty.value = activePack?.value?.boostyBlog || "";
  const mr = activePack?.value?.minRam ?? null;
  authorMinRam.value = !!mr;
  authorMinRamMb.value = mr ? Math.round(mr / 1024) : null;
  authorServers.value = [{ name: "", ip: "", port: null, desc: "" }];
  authorSocials.value = [{ name: "", url: "", color: "" }];
  authorTheme.value = {};
  authorAccent.value = "";
  exportOpen.value = true;
  exportExpanded.value = new Set();
  void loadExportList();
}

const authorThemeFields: Array<{ key: keyof AuthorTheme; cap: string }> = [
  { key: "accent", cap: "pack.exportThemeAccent" },
  { key: "accentStrong", cap: "pack.exportThemeAccentStrong" },
  { key: "accentHover", cap: "pack.exportThemeAccentHover" },
  { key: "accentDeep", cap: "pack.exportThemeAccentDeep" },
  { key: "bg", cap: "pack.exportThemeBg" },
  { key: "panel", cap: "pack.exportThemePanel" },
  { key: "input", cap: "pack.exportThemeInput" },
  { key: "border", cap: "pack.exportThemeBorder" },
  { key: "tx", cap: "pack.exportThemeTx" },
  { key: "txStrong", cap: "pack.exportThemeTxStrong" },
  { key: "txMuted", cap: "pack.exportThemeTxMuted" },
];

function addAuthorServer() { if (authorServers.value.length < AUTHOR_MAX_SERVERS) authorServers.value.push({ name: "", ip: "", port: null, desc: "" }); }
function removeAuthorServer(i: number) { if (authorServers.value.length > 1) authorServers.value.splice(i, 1); else authorServers.value = [{ name: "", ip: "", port: null, desc: "" }]; }
function addAuthorSocial() { if (authorSocials.value.length < AUTHOR_MAX_SOCIALS) authorSocials.value.push({ name: "", url: "", color: "" }); }
function removeAuthorSocial(i: number) { if (authorSocials.value.length > 1) authorSocials.value.splice(i, 1); else authorSocials.value = [{ name: "", url: "", color: "" }]; }

/** По одному введённому цвету автозаполняет акцентную тему. */
function applyAuthorAccent() {
  const t = themeFromAccent(authorAccent.value);
  if (t) authorTheme.value = t;
}

/** Событие нативного color-picker: ставит валидный hex и применяет тему. */
function applyAuthorAccentColor(ev: Event) {
  const v = (ev.target as HTMLInputElement).value;
  if (!v) return;
  authorAccent.value = v;
  applyAuthorAccent();
}

function themePreview(hex?: string | null): string {
  return normalizeHex(hex ?? "") ?? "#000";
}

function authorConfig(): AuthorPackConfig {
  return {
    name: authorName.value.trim() || activePack?.value?.name || "pack",
    author: authorAuthor.value.trim(),
    description: authorDesc.value.trim() ? authorDesc.value.trim() : null,
    boostyBlog: authorBoosty.value.trim() ? authorBoosty.value.trim() : null,
    minRam: authorMinRam.value ? (authorMinRamMb.value ?? null) : null,
    servers: authorServers.value
      .filter((s) => s.name.trim() || s.ip.trim())
      .map((s) => ({ name: s.name.trim(), ip: s.ip.trim(), port: s.port ?? null, desc: s.desc?.trim() ? s.desc.trim() : null })),
    socials: authorSocials.value
      .filter((s) => s.name.trim() && s.url.trim())
      .map((s) => ({ name: s.name.trim(), url: s.url.trim(), color: s.color?.trim() ? s.color.trim() : null })),
    theme: authorThemeFields.some((f) => (authorTheme.value[f.key] ?? "").trim())
      ? Object.fromEntries(
          authorThemeFields
            .filter((f) => (authorTheme.value[f.key] ?? "").trim())
            .map((f) => [f.key, authorTheme.value[f.key]!.trim()])
        ) as AuthorTheme
      : null,
  };
}

async function doAuthorExport() {
  if (exportBusy.value || !packId.value || !isTauri()) return;
  const include = [...exportSelected.value];
  if (include.length === 0) {
    notify(t("pack.exportEmpty"), "info");
    return;
  }
  const cfg = authorConfig();
  const dest = await save({
    defaultPath: `${cfg.name.replace(/[^a-zа-яё0-9-]+/gi, "-").toLowerCase() || "pack"}.zip`,
    filters: [{ name: "ZIP", extensions: ["zip"] }],
  });
  if (!dest) return;
  exportBusy.value = true;
  try {
    await exportAuthorPack(packId.value, "", dest, include, cfg);
    notify(t("pack.exportAuthorDone"), "success");
    exportOpen.value = false;
  } catch (e) {
    notify(t("pack.exportAuthorErr", { e }));
  } finally {
    exportBusy.value = false;
  }
}

async function saveEditVersion() {
  if (editVerBusy.value || !packId.value) return;
  const mc = editVerMc.value.trim();
  if (!mc) {
    notify(t("mods.createVersionReq"), "info");
    return;
  }
  editVerBusy.value = true;
  try {
    await editPackVersion(packId.value, mc, editVerLoader.value, editVerLv.value);
    if (editVerName.value.trim() && editVerName.value.trim() !== activePack?.value?.name) {
      await setPackName(packId.value, editVerName.value.trim());
    }
    await load();
    refreshModUpdates(true);
    notify(t("pack.versionSaved"), "success");
    editVerOpen.value = false;
  } catch (e) {
    notify(t("pack.versionSaveErr", { e }));
  } finally {
    editVerBusy.value = false;
  }
}

/** При открытии модалки создания — грузим список версий Minecraft и сбрасываем выбор файлов. */
watch(createPackOpen, async (open) => {
  if (!open) return;
  createPackIcon.value = null;
  createPackBanner.value = null;
  createPackLoaderVersion.value = "";
  if (createPackVersions.value.length) return;
  try {
    createPackVersions.value = await minecraftVersions();
    if (createPackVersions.value.length) {
      const cur = createPackMc.value;
      const has = (id: string) => createPackVersions.value.some((v) => v.id === id);
      createPackMc.value = has(cur) ? cur : has("1.21.4") ? "1.21.4" : createPackVersions.value[0].id;
    }
  } catch (e) {
    notify(t("mods.createErr", { e }));
  }
});

/** При смене загрузчика/версии — заново грузим доступные версии загрузчика. */
watch([createPackLoader, createPackMc], async ([loader, mc]) => {
  createPackLoaderVersion.value = "";
  createPackLoaderVersions.value = [];
  if (createPackOpen.value && loader !== "vanilla") {
    try {
      createPackLoaderVersions.value = await localLoaderVersions(loader, mc.trim());
    } catch (e) {
      createPackLoaderVersions.value = [];
      notify(t("mods.createErr", { e }));
    }
  }
});

/** Закрываем выпадающий список версий загрузчика при клике вне его. */
watch(createPackLoaderLvOpen, (open) => {
  if (createPackLvClose) {
    document.removeEventListener("mousedown", createPackLvClose);
    createPackLvClose = null;
  }
  if (open) {
    createPackLvClose = (e) => {
      if (!createPackLvBox.value?.contains(e.target as Node)) createPackLoaderLvOpen.value = false;
    };
    document.addEventListener("mousedown", createPackLvClose);
  }
});

function chooseCreateLoaderVersion(v: string) {
  createPackLoaderVersion.value = v;
  createPackLoaderLvOpen.value = false;
}

/** Отфильтрованные по запросу подгруппы версий для выпадающего списка. */
const filteredCreateReleases = computed(() =>
  createPackVersions.value
    .filter((v) => v.kind !== "snapshot" && v.id.toLowerCase().includes(createPackVersionQuery.value.toLowerCase()))
);
const filteredCreateSnapshots = computed(() =>
  createPackVersions.value
    .filter((v) => v.kind === "snapshot" && v.id.toLowerCase().includes(createPackVersionQuery.value.toLowerCase()))
);
const createVersionGroups = computed(() => [
  { label: t("mods.createReleases"), items: filteredCreateReleases.value },
  { label: t("mods.createSnapshots"), items: filteredCreateSnapshots.value },
]);

function chooseCreateVersion(id: string) {
  createPackMc.value = id;
  createPackVersionOpen.value = false;
  createPackVersionQuery.value = "";
}

/** Закрываем выпадающий список при клике вне его. */
watch(createPackVersionOpen, (open) => {
  if (createPackVersionClose) {
    document.removeEventListener("mousedown", createPackVersionClose);
    createPackVersionClose = null;
  }
  if (open) {
    createPackVersionClose = (e) => {
      if (!createPackVersionBox.value?.contains(e.target as Node)) {
        createPackVersionOpen.value = false;
        createPackVersionQuery.value = "";
      }
    };
    document.addEventListener("mousedown", createPackVersionClose);
  }
});

async function pickCreateFile(target: "icon" | "banner") {
  const path = await pickImage();
  if (!path) return;
  if (target === "icon") createPackIcon.value = path;
  else createPackBanner.value = path;
}

// При открытии сабтаба файлов — проверяем обновления установленных из Modrinth файлов.
watch(playSubTab, (tab) => {
  if (tab === "mods" || tab === "resourcepacks" || tab === "shaderpacks") refreshModUpdates();
  if (tab === "duplicates" && activePack.value) loadDuplicates(packId.value);
});

// При загрузке сборки / после установки — сразу проверяем обновления,
// иначе кнопка «обновить» и фильтр «есть обновления» не появляются до смены сабтаба.
watch(
  () => [packId.value, status.value?.installed] as const,
  ([id, inst]) => {
    if (id && inst) refreshModUpdates();
  }
);

// Если вкладка релизов скрыта (не авторская сборка), уводим с неё.
watch(
  () => activePack.value?.kind,
  (kind) => {
    if (kind && kind !== "remote" && playSubTab.value === "releases") playSubTab.value = "mods";
  },
  { immediate: true }
);

function openSelected(site: "modrinth" | "curseforge") {
  const sel = Object.values(selectedFiles.value);
  for (const s of sel) {
    if (site === "modrinth") openFileOnModrinth(s.folder, s.entry);
    else openFileOnCurseForge(s.folder, s.entry);
  }
}

watch(playSubTab, () => {
  if (fileListRef.value) fileListRef.value.scrollTop = 0;
  if (
    (playSubTab.value === "screenshots" || playSubTab.value === "servers") &&
    activePack.value
  ) {
  }
  if (playSubTab.value === "screenshots" && activePack.value) {
    loadPackScreenshots(activePack.value.id);
  }
});

// Таймер пинга серверов живёт только пока экран «Серверы» реально активен
// (вкладка play + серверный сабтаб + выбранная сборка). Иначе — гасим, чтобы
// не пинговать в фоне при переключении сборок/вкладок.
let serverPingTimer: ReturnType<typeof setInterval> | null = null;
watch(
  () => [tab.value, playSubTab.value, packId.value],
  () => {
    if (tab.value === "play" && playSubTab.value === "servers" && activePack.value) {
      loadMyServers(activePack.value.id);
      pingActiveServers();
      stopServerPingTimer();
      serverPingTimer = setInterval(pingActiveServers, 45000);
    } else {
      stopServerPingTimer();
    }
  },
  { immediate: true }
);

function gameFileIcon(folder: string, name: string): string {
  return fileIcons.value[`${folder}/${name}`] ?? "";
}

const unbindArmed = ref(false);
async function confirmUnbindPack() {
  if (!unbindArmed.value) {
    unbindArmed.value = true;
    return;
  }
  unbindArmed.value = false;
  await setActivePackLocked(false);
}

function onJavaChange(e: Event) {
  const val = (e.target as HTMLSelectElement).value;
  selectJava(val);
}

const expanded = ref<Record<string, boolean>>({});

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

/** Баннер сборки: скрываем, если картинка не загрузилась. */
const bannerOk = ref(true);
/** Баннер: локальный из папки сборки. */
const activeBanner = computed(() => {
  if (activePack.value?.banner) return convertFileSrc(activePack.value.banner);
  return null;
});
watch(
  () => activePack.value?.id,
  () => {
    bannerOk.value = true;
  }
);

/** Время в игре: короткий формат для бейджа («3 ч» / «12 мин»). */
const shotIdx = ref<number | null>(null);

/** Удалить все файлы группы, кроме первого (оставить «оригинал»). */
async function keepOne(g: DuplicateGroup) {
  for (const f of g.files.slice(1)) {
    await removeDuplicate(packId.value, f);
  }
  await loadDuplicates(packId.value);
}

// ==== Тема сборки удалена (GitHub repo content больше не загружается) ====

onBeforeUnmount(() => {
  stopServerPingTimer();
});

/** Разбирает адрес "host" или "host:port" из servers.dat. */
function splitServerAddress(address: string): { ip: string; port: number | null } {
  const idx = address.lastIndexOf(":");
  if (idx > 0 && /^\d+$/.test(address.slice(idx + 1))) {
    return { ip: address.slice(0, idx), port: Number(address.slice(idx + 1)) };
  }
  return { ip: address, port: null };
}

type ServerGroup = {
  key: "mine";
  title: string;
  servers: { name: string; ip: string; port: number | null; desc: string | null }[];
  emptyText: string;
};

/** Группы серверов: свои (servers.dat). */
const serverGroups = computed<ServerGroup[]>(() => {
  const mine = myServers.value.map((s) => {
    const { ip, port } = splitServerAddress(s.address);
    return { name: s.name, ip, port, desc: null };
  });
  return [
    {
      key: "mine",
      title: t("servers.myTitle"),
      servers: mine,
      emptyText: myServersInstalled.value ? t("servers.myEmpty") : t("servers.noInstall"),
    },
  ];
});

async function copyServerIp(srv: { ip: string; port: number | null }) {
  const text = `${srv.ip}${srv.port ? `:${srv.port}` : ""}`;
  try {
    await navigator.clipboard.writeText(text);
    notify(t("servers.copied", { ip: text }), "success");
  } catch {
    notify(`${t("servers.copyFail")}: ${text}`, "error");
  }
}

const sidebarWidth = ref(readSidebarWidth());
const sidebarDragging = ref(false);
const SIDEBAR_COLLAPSE = 260;
const SIDEBAR_ICON = 68;
const sidebarCollapsed = computed(() => sidebarWidth.value < SIDEBAR_COLLAPSE);

/** Масштаб плиток «Библиотеки» (1–4), сохраняется в localStorage. */
const LIB_SCALE_KEY = "mono.libScale";
function readLibScale(): number {
  const n = parseInt(localStorage.getItem(LIB_SCALE_KEY) ?? "", 10);
  return Number.isFinite(n) ? Math.min(4, Math.max(1, n)) : 2;
}
const libScale = ref(readLibScale());
function setLibScale(n: number) {
  libScale.value = Math.min(4, Math.max(1, n));
  localStorage.setItem(LIB_SCALE_KEY, String(libScale.value));
}
const LIB_TILES: Record<number, { col: string; icon: string }> = {
  1: { col: "grid-cols-[repeat(auto-fill,minmax(92px,1fr))]", icon: "h-10 w-10" },
  2: { col: "grid-cols-[repeat(auto-fill,minmax(124px,1fr))]", icon: "h-14 w-14" },
  3: { col: "grid-cols-[repeat(auto-fill,minmax(164px,1fr))]", icon: "h-20 w-20" },
  4: { col: "grid-cols-[repeat(auto-fill,minmax(220px,1fr))]", icon: "h-28 w-28" },
};
const libTile = computed(() => LIB_TILES[libScale.value] ?? LIB_TILES[2]);
const libPercent = computed(() => 75 + libScale.value * 25);

/** Кэш статусов сборок для «Библиотеки» (установлена ли, версия). */
const libStatus = reactive<Record<string, AppStatus | null>>({});
let libStatusLoading = false;
async function loadLibraryStatus() {
  if (!isTauri() || libStatusLoading) return;
  libStatusLoading = true;
  try {
    await Promise.all(
      packs.value.map(async (p) => {
        libStatus[p.id] = await getStatus(p.id).catch(() => null);
      })
    );
  } finally {
    libStatusLoading = false;
  }
}
watch(
  () => tab.value,
  (t) => {
    if (t === "library") loadLibraryStatus();
    if (t === "settings") loadAllLicenses();
  }
);
watch(
  () => packs.value.length,
  () => loadAllLicenses()
);
onMounted(loadLibraryStatus);
onMounted(loadAllLicenses);

/** Запуск (или установка) конкретной сборки из плитки «Библиотеки». */
async function playLibraryPack(p: PackDescriptor) {
  if (!isTauri()) return;
  await openPackTab(p.id);
  if (status.value?.installed) await handlePlay();
  else await handleInstall();
}

/** Контекстное меню «Библиотеки» (ПКМ по экземпляру). */
const libMenuPack = ref<PackDescriptor | null>(null);
const libMenuPos = ref<{ x: number; y: number } | null>(null);

function openLibMenu(e: MouseEvent, p: PackDescriptor) {
  const menuW = 224;
  const menuH = 120;
  const x = Math.min(e.clientX, window.innerWidth - menuW - 8);
  const y = Math.min(e.clientY, window.innerHeight - menuH - 8);
  libMenuPack.value = p;
  libMenuPos.value = { x: Math.max(8, x), y: Math.max(8, y) };
}

function closeLibMenu() {
  libMenuPack.value = null;
  libMenuPos.value = null;
}

async function libDoPlay() {
  const p = libMenuPack.value;
  closeLibMenu();
  if (p) await playLibraryPack(p);
}

function libOpenSettings() {
  const p = libMenuPack.value;
  closeLibMenu();
  if (p) openPackTab(p.id);
}

/** Пользовательская ширина основного контента (0 = авто, растягивается). */
const mainWidth = ref(readMainWidth());
let mainDrag = { active: false, left: 0, max: 0 };

/** Вкладка правой панели (глобальные разделы). */
const rightTab = ref<"settings" | "dev" | "news" | "catalog">("settings");

function readMainWidth(): number {
  const saved = parseInt(localStorage.getItem("mono.mainWidth") ?? "", 10);
  return Number.isFinite(saved) ? Math.max(0, saved) : 0;
}

function startMainDrag(e: PointerEvent) {
  const row = (e.currentTarget as HTMLElement).parentElement;
  if (!row) return;
  const r = row.getBoundingClientRect();
  mainDrag = { active: true, left: r.left, max: r.right - 10 };
  mainWidth.value = Math.max(420, Math.min(mainDrag.max, e.clientX - r.left));
  (e.target as HTMLElement).setPointerCapture(e.pointerId);
}

function onMainDrag(e: PointerEvent) {
  if (!mainDrag.active) return;
  mainWidth.value = Math.max(420, Math.min(mainDrag.max, e.clientX - mainDrag.left));
}

function endMainDrag(e: PointerEvent) {
  if (!mainDrag.active) return;
  mainDrag.active = false;
  try {
    (e.target as HTMLElement).releasePointerCapture(e.pointerId);
  } catch {
    /* ignore */
  }
  if (mainWidth.value > 0) localStorage.setItem("mono.mainWidth", String(mainWidth.value));
}

function resetMainWidth() {
  mainWidth.value = 0;
  localStorage.setItem("mono.mainWidth", "0");
}

function readSidebarWidth(): number {
  const saved = parseInt(localStorage.getItem("mono.sidebarWidth") ?? "", 10);
  return Number.isFinite(saved) ? Math.min(420, Math.max(SIDEBAR_ICON, saved)) : 272;
}

function startSidebarDrag(e: PointerEvent) {
  sidebarDragging.value = true;
  (e.target as HTMLElement).setPointerCapture(e.pointerId);
}

function onSidebarDrag(e: PointerEvent) {
  if (!sidebarDragging.value) return;
  let w = Math.min(420, Math.max(SIDEBAR_ICON, e.clientX));
  if (w < SIDEBAR_COLLAPSE) w = SIDEBAR_ICON;
  sidebarWidth.value = w;
}

function endSidebarDrag(e: PointerEvent) {
  if (!sidebarDragging.value) return;
  sidebarDragging.value = false;
  try {
    (e.target as HTMLElement).releasePointerCapture(e.pointerId);
  } catch {
    /* ignore */
  }
  let w = sidebarWidth.value;
  if (w < SIDEBAR_COLLAPSE) w = SIDEBAR_ICON;
  else if (w < SIDEBAR_COLLAPSE + 40) w = SIDEBAR_COLLAPSE;
  sidebarWidth.value = w;
  localStorage.setItem("mono.sidebarWidth", String(sidebarWidth.value));
}

async function pickImage(): Promise<string | null> {
  if (!isTauri()) {
    notify(t("skin.tauriOnly"), "info");
    return null;
  }
  try {
    const p = await openDialog({
      multiple: false,
      filters: [
        { name: "Изображение", extensions: ["png", "jpg", "jpeg", "webp"] },
      ],
    });
    return typeof p === "string" ? p : null;
  } catch {
    notify(t("skin.readFail"), "error");
    return null;
  }
}

async function pickSkinFile() {
  const path = await pickImage();
  if (path) await applyLocalSkin(path);
}

async function pickPackIcon(packId: string) {
  const path = await pickImage();
  if (!path) return;
  try {
    await setPackIcon(packId, path);
    notify(t("dev.iconSet"), "success");
    await loadPacks();
  } catch (e) {
    notify(t("dev.iconErr", { e }));
  }
}

async function pickPackBanner(packId: string) {
  const path = await pickImage();
  if (!path) return;
  try {
    await setPackBanner(packId, path);
    notify(t("dev.bannerSet"), "success");
    await loadPacks();
  } catch (e) {
    notify(t("dev.bannerErr", { e }));
  }
}

async function copySkinApi() {
  try {
    await navigator.clipboard.writeText(skinApi.value);
    notify(t("skin.copied"), "success");
  } catch {
    notify(t("servers.copyFail"), "error");
  }
}

/** Статусы серверов активной сборки: key "host:port" → результат пинга. */
const serverStatuses = ref<Record<string, ServerStatus>>({});
const serverPinging = ref<Record<string, boolean>>({});

function serverKey(srv: { ip: string; port: number | null }): string {
  return `${srv.ip}:${srv.port ?? 25565}`;
}

function stopServerPingTimer() {
  if (serverPingTimer) {
    clearInterval(serverPingTimer);
    serverPingTimer = null;
  }
}

async function pingOneServer(srv: { ip: string; port: number | null }) {
  const key = serverKey(srv);
  if (serverPinging.value[key]) return;
  serverPinging.value[key] = true;
  try {
    serverStatuses.value[key] = await pingServer(srv.ip, srv.port ?? null);
  } catch {
    serverStatuses.value[key] = { online: false, version: null, motd: null, playersOnline: null, playersMax: null, players: [], latencyMs: null };
  } finally {
    serverPinging.value[key] = false;
  }
}

function serverPlayersOf(srv: { ip: string; port: number | null }): string[] {
  return serverStatuses.value[serverKey(srv)]?.players ?? [];
}

function pingActiveServers() {
  serverGroups.value.forEach((g) => g.servers.forEach((srv) => void pingOneServer(srv)));
}

type ServerState = "online" | "offline" | "checking" | "unknown";

function serverStateOf(srv: { ip: string; port: number | null }): ServerState {
  const key = serverKey(srv);
  if (serverPinging.value[key]) return "checking";
  const st = serverStatuses.value[key];
  if (!st) return "unknown";
  return st.online ? "online" : "offline";
}

function serverStatusText(srv: { ip: string; port: number | null }): string {
  const key = serverKey(srv);
  const st = serverStatuses.value[key];
  switch (serverStateOf(srv)) {
    case "checking":
      return t("servers.checking");
    case "unknown":
      return t("servers.unknown");
    case "offline":
      return t("servers.offline");
    default: {
      const parts = [t("servers.online")];
      if (st?.playersOnline != null) parts.push(`${st.playersOnline}/${st.playersMax ?? "?"}`);
      if (st?.version) parts.push(st.version);
      if (st?.latencyMs != null) parts.push(`${st.latencyMs}мс`);
      return parts.join(" · ");
    }
  }
}

/** Открывает вкладку сборки: выбирает её и показывает play-вид. */
async function openPackTab(id: string) {
  if (packId.value !== id) await selectPack(id);
  tab.value = "play";
}

/** Сборка из каталога Mono уже добавлена в лаунчер? */
function isMonoPackAdded(entry: PackCatalog): boolean {
  return packs.value.some((p) => p.url === entry.url);
}

/** Добавить сборку из каталога Mono в библиотеку. */
async function addMonoPack(entry: PackCatalog) {
  if (addingPack.value || busy.value) return;
  addingPack.value = true;
  try {
    const added = await addPack(entry.url, entry.name, entry.boosty_blog ?? undefined);
    await loadPacks();
    await load();
    refreshVersions();
    if (packId.value !== added.id) {
      await selectPack(added.id);
    }
    notify(t("catalog.added", { name: added.name }), "success");
  } catch (e) {
    notify(t("dev.errAdd", { e }), "error");
  } finally {
    addingPack.value = false;
  }
}

/** Открыть уже добавленную сборку из каталога Mono. */
async function openMonoPack(entry: PackCatalog) {
  const pack = packs.value.find((p) => p.url === entry.url);
  if (pack) await openPackTab(pack.id);
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

// ---- Отдельное окно поиска файлов (win=search) ----
if (isSearchWin.value) {
  const q = route.query;
  const kind =
    (["mod", "resourcepack", "shaderpack", "datapack"] as const).find((k) => k === q.kind) ??
    "mod";
  modSearchKind.value = kind;
  searchService.value = q.service === "curseforge" ? "curseforge" : "modrinth";
  searchOpen.value = true;
  if (typeof q.packId === "string" && q.packId) packId.value = q.packId;
  // Загружаем статус сборки (нужен для автофильтров по версии/загрузчику), теги и запускаем поиск.
  void (async () => {
    await load();
    await loadModrinthTags(kind);
    applyAutoFilters();
    if (searchService.value === "curseforge") {
      await loadCurseKeyStatus();
      await loadCurseCategories();
      if (!curseKeyOk.value) return;
    }
    if (kind === "datapack") await loadGameFiles("saves");
    else
      await loadGameFiles(
        (CURSE_FOLDER[(kind as ModrinthSearchKind)] ?? "mods") as GameFolderKind
      );
    await runInitialSearch();
  })();
}

// ---- Отдельное окно просмотра ресурса (win=filedetail) ----
if (isFileDetailWin.value) {
  const q = route.query;
  const folder = (q.folder === "saves" ? "saves" : (q.folder as GameFolderKind) || "mods") as GameFolderKind;
  fileDetailTitle.value = typeof q.name === "string" ? q.name : "";
  const slug = typeof q.slug === "string" ? q.slug : "";
  const cfid = typeof q.cfid === "string" && q.cfid ? q.cfid : "";
  fileDetailFolder.value = folder;
  fileDetailMcFilter.value = status.value?.minecraft_version || null;
  fileDetailLoaderFilter.value = "";
  fileDetailTypeFilter.value = "";
  fileDetailTab.value = "about";
  if (slug) {
    fileDetail.value = {
      folder,
      entry: { name: "", displayName: fileDetailTitle.value, kind: "file", enabled: true, sizeBytes: 0, modified: 0, modrinthProjectId: slug },
    };
    void (async () => {
      await load();
      if (typeof q.packId === "string" && q.packId) packId.value = q.packId;
      fileDetailMcFilter.value = status.value?.minecraft_version || null;
      fileDetailMrLoading.value = true;
      try {
        fileDetailMr.value = await modrinthProject(slug);
        const fl = folder === "saves" ? "mods" : folder;
        await loadFileDetailVersions(slug, fl);
        await loadFileDetailInstalledSha(fileDetailMr.value.projectId);
      } catch {
        fileDetailMr.value = null;
      } finally {
        fileDetailMrLoading.value = false;
      }
    })();
  } else if (cfid) {
    const cfIdNum = Number(cfid) || 0;
    fileDetail.value = {
      folder,
      entry: { name: "", displayName: fileDetailTitle.value, kind: "file", enabled: true, sizeBytes: 0, modified: 0, curseforgeProjectId: cfIdNum },
    };
    void (async () => {
      await load();
      if (typeof q.packId === "string" && q.packId) packId.value = q.packId;
      fileDetailCfLoading.value = true;
      try {
        fileDetailCf.value = await curseforgeProjectDetail(cfIdNum);
      } catch {
        fileDetailCf.value = null;
      } finally {
        fileDetailCfLoading.value = false;
      }
    })();
  }
}

provide(LauncherCtxKey, {
  ...__launcher,
  t,
  locale,
  locales,
  setLocale,
  getLocaleMeta,
  formatPlaytimeShort,
  phaseLabel,
  javaArchLabel,
  localeLabel,
  activeLocaleAuthor,
  activeLocaleVersion,
  crashView,
  appClose,
  appMinimize,
  appToggleMaximize,
  sidebarWidth,
  sidebarDragging,
  SIDEBAR_COLLAPSE,
  SIDEBAR_ICON,
  sidebarCollapsed,
  startSidebarDrag,
  onSidebarDrag,
  endSidebarDrag,
  mainWidth,
  startMainDrag,
  onMainDrag,
  endMainDrag,
  resetMainWidth,
  createPackOpen,
  customModsOpen,
  openEditVersion,
  openPackTab,
  activePackRepo,
});
</script>