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
              title="Открыть GitHub Issues с этим сообщением и логом запуска"
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
              title="Закрыть"
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
    <!-- ==== Боковая панель ==== -->
    <aside class="flex w-64 shrink-0 flex-col border-r border-[#30363d] bg-[#161b22]">
      <!-- Выбор сборки (стилизован под репозиторий GitHub) -->
      <div class="p-3.5 border-b border-[#30363d]">
        <label class="mb-1.5 block text-[11px] font-semibold uppercase tracking-wider text-[#8b949e]">
          Сборка / Репозиторий
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
          <span class="truncate font-mono">{{ activePack?.name ?? "не выбрано" }}</span>
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
          Релизы и запуск
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
          Параметры
        </button>
      </nav>

      <!-- Сводка статуса -->
      <div class="space-y-2 p-3.5 text-xs text-[#8b949e]">
        <div class="flex items-center justify-between">
          <span>Состояние</span>
          <span class="inline-flex items-center gap-1.5 font-medium">
            <span class="h-2 w-2 rounded-full" :class="status?.installed ? 'bg-[#3fb950]' : 'bg-[#8b949e]'"></span>
            <span :class="status?.installed ? 'text-[#f0f6fc]' : 'text-[#8b949e]'">
              {{ status?.installed ? "Установлена" : "Не установлена" }}
            </span>
          </span>
        </div>
        <div class="flex items-center justify-between">
          <span>Версия</span>
          <span class="font-mono font-medium text-[#c9d1d9] truncate max-w-[110px]" :title="status?.active_version ? `versionId: ${status.active_version}` : undefined">
            {{ status?.active_source_tag ?? status?.active_version ?? "—" }}
          </span>
        </div>
        <div class="flex items-center justify-between">
          <span>Память</span>
          <span class="font-mono font-medium text-[#c9d1d9]">{{ ram }} ГБ</span>
        </div>
      </div>

      <!-- Глобальный прогресс установки/скачивания -->
      <div v-if="progress && busy" class="border-t border-[#30363d] p-3 bg-[#0d1117]/50">
        <div class="mb-1 flex items-center justify-between text-[11px] text-[#8b949e]">
          <span class="truncate pr-2 font-medium text-[#c9d1d9]">{{ progress.phase }}</span>
          <span class="tabular-nums font-mono text-[10px]">{{ percent }}%</span>
        </div>
        <div class="h-1.5 w-full overflow-hidden rounded-full bg-[#21262d]">
          <div
            class="h-full bg-[#2f81f7] transition-all duration-200"
            :style="{ width: `${percent}%` }"
          />
        </div>
        <div class="mt-1 flex items-center justify-between text-[10px] text-[#8b949e]">
          <span class="truncate max-w-[120px]">{{ progress.currentFile || "Подготовка…" }}</span>
          <span class="tabular-nums font-mono">{{ progress.speed > 0 ? `${formatBytes(progress.speed)}/с` : "" }}</span>
        </div>
      </div>

      <div class="flex-1" />

      <!-- Учётная запись -->
      <div class="flex items-center gap-2.5 border-t border-[#30363d] p-3 bg-[#0d1117]/30">
        <div class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-[#21262d] border border-[#30363d] font-mono text-xs font-bold text-[#f0f6fc]">
          {{ session?.username?.[0]?.toUpperCase() ?? "?" }}
        </div>
        <div class="min-w-0 flex-1">
          <div class="truncate text-xs font-medium text-[#c9d1d9]">
            {{ session?.username ?? "Гость" }}
          </div>
          <div class="truncate text-[10px] text-[#8b949e]">
            {{ session ? session.user_type : "Оффлайн профиль" }}
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
            {{ busy ? "Установка…" : "Cкачать и играть" }}
          </template>
          <template v-else>
            {{ busy ? "Запуск игры…" : "ИГРАТЬ" }}
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
                  {{ activePack?.name ?? "Сборка не выбрана" }}
                </h1>
                                <span
                  class="ml-2 rounded-full px-2 py-0.5 text-[11px] font-medium border"
                  :class="status?.installed
                    ? 'border-[#238636]/40 bg-[#238636]/10 text-[#3fb950]'
                    : 'border-[#30363d] bg-[#21262d] text-[#8b949e]'"
                >
                  {{ status?.installed ? "установлена" : "не установлена" }}
                </span>
                <button
                  type="button"
                  class="ml-1 flex items-center gap-1.5 rounded-md border border-[#30363d] bg-[#21262d] px-2.5 py-1 text-[11px] font-medium text-[#8b949e] transition-colors hover:bg-[#30363d] hover:text-[#c9d1d9]"
                  title="Открыть папку активной версии сборки"
                  @click="handleOpenPackDir"
                >
                  <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
                    <path d="M.513 1.513A1.75 1.75 0 0 1 1.75 1h3.5c.55 0 1.07.26 1.4.7l.9 1.2a.25.25 0 0 0 .2.1H13a1 1 0 0 1 1 1v.5H2.75a.75.75 0 0 0 0 1.5h11.978a1 1 0 0 1 .994 1.117L15 13.25A1.75 1.75 0 0 1 13.25 15H1.75A1.75 1.75 0 0 1 0 13.25V2.75c0-.464.184-.91.513-1.237Z"/>
                  </svg>
                  Папка сборки
                </button>
              </div>
            </div>

            <p class="mt-2 text-xs text-[#8b949e] flex items-center gap-2">
              <span>Монолаунчер</span>
              <span>•</span>
              <span v-if="loaderLabel">Загрузчик: <strong class="text-[#c9d1d9] font-normal">{{ loaderLabel }}</strong></span>
            </p>

            <div v-if="updateInfo?.has_update && updateInfo.latest_version" class="mt-4 flex items-center justify-between gap-4 rounded-md border border-[#1f6beb]/40 bg-[#1f6beb]/10 px-3.5 py-2.5 text-xs text-[#58a6ff]">
              <span class="min-w-0">
                Доступна новая версия: <strong class="text-[#79c0ff]">{{ updateInfo.latest_version }}</strong>
                <span v-if="updateInfo.current_version" class="text-[#8b949e]">
                  · установлена {{ updateInfo.current_version }}
                </span>
              </span>
              <button
                type="button"
                class="shrink-0 rounded-md border border-[#1f6beb]/50 bg-[#1f6beb]/20 px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[#1f6beb]/40 disabled:opacity-50"
                :disabled="busy"
                @click="handleUpdate"
              >
                Обновить
              </button>
            </div>
          </div>

          <!-- Список релизов GitHub -->
          <div v-if="versions && versions.github.length > 0" class="min-h-0 flex-1 space-y-4 overflow-y-auto pr-1">
            <div class="flex items-center justify-between text-xs text-[#8b949e]">
              <span class="font-medium">Версии ({{ versions.github.length }})</span>
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
                    Pre-release
                  </span>
                  <span v-if="isActiveRelease(r.tag)" class="rounded-full border border-[#238636]/40 bg-[#238636]/10 px-2 py-0.2 text-[10px] font-medium text-[#3fb950]">
                    Активная
                  </span>
                </div>

                <div class="flex items-center gap-3">
                  <span class="text-[11px] text-[#8b949e]">
                    {{ formatDate(r.published_at) }}
                  </span>
                  <span
                    v-if="playtimeForRelease(r.tag) > 0"
                    class="font-mono text-[11px] text-[#d29922]"
                    title="Наиграно в этом экземпляре"
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
                      {{ isActiveRelease(r.tag) ? "Выбрано" : "Переключить" }}
                    </template>
                    <template v-else>
                      Установить
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
                <p v-else class="text-[#8b949e] italic">Нет описания изменений в этом релизе.</p>

                <button
                  v-if="isExpandable(r.body)"
                  type="button"
                  class="mt-2 inline-block text-xs font-medium text-[#58a6ff] hover:underline"
                  @click="toggleExpanded(r.tag)"
                >
                  {{ isExpanded(r.tag) ? "Свернуть ченджлог" : "Показать весь ченджлог" }}
                </button>
              </div>
            </article>
          </div>

          <div v-else class="shrink-0 rounded-md border border-[#30363d] bg-[#161b22] p-8 text-center text-xs text-[#8b949e]">
            Не удалось загрузить список релизов. Проверьте соединение с сетью или параметры репозитория.
          </div>

          <!-- Консоль -->
          <section class="mt-5 flex h-[24rem] shrink-0 flex-col overflow-hidden rounded-md border border-[#30363d] bg-[#161b22]">
            <div class="flex items-center justify-between border-b border-[#30363d] bg-[#21262d]/50 px-4 py-2">
              <h3 class="text-xs font-semibold text-[#f0f6fc]">Консоль / Логи работы</h3>
              <div class="flex items-center gap-3">
                <span class="text-[10px] tabular-nums text-[#484f58]">
                  {{ logEntries.length }} строк
                </span>
                <div class="flex gap-2">
                  <button
                    type="button"
                    class="text-[11px] text-[#8b949e] hover:text-[#58a6ff]"
                    @click="handleCopyLog"
                  >
                    Скопировать
                  </button>
                  <button
                    type="button"
                    class="text-[11px] text-[#8b949e] hover:text-[#f85149]"
                    @click="handleClearLog"
                  >
                    Очистить
                  </button>
                </div>
              </div>
            </div>
            <div
              ref="logRef"
              class="flex-1 select-text overflow-y-auto bg-[#0d1117] p-3 font-mono text-[11px] leading-relaxed text-[#8b949e]"
            >
              <p v-if="logEntries.length === 0" class="italic text-[#484f58]">
                Логи отсутствуют. Запустите игру для просмотра вывода.
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

        <!-- ======= Вкладка: Настройки ======= -->
        <template v-else>
          <div class="min-h-0 flex-1 overflow-y-auto pr-1">
          <div class="space-y-6">
            <div class="border-b border-[#30363d] pb-3">
              <h1 class="text-lg font-semibold text-[#f0f6fc]">Параметры лаунчера</h1>
              <p class="text-xs text-[#8b949e]">Управление аккаунтом, памятью и логами запуска</p>
            </div>

            <!-- Учётная запись -->
            <section class="rounded-md border border-[#30363d] bg-[#161b22] overflow-hidden">
              <div class="border-b border-[#30363d] bg-[#21262d]/50 px-4 py-2.5">
                <h3 class="text-xs font-semibold text-[#f0f6fc]">Учётная запись</h3>
              </div>
              <div class="p-4 space-y-3">
                <div class="flex gap-2">
                  <input
                    v-model="username"
                    placeholder="Никнейм (оффлайн режим)"
                    class="flex-1 rounded-md border border-[#30363d] bg-[#0d1117] px-3 py-1.5 text-xs text-[#c9d1d9] placeholder-[#8b949e] focus:border-[#58a6ff] focus:outline-none"
                  />
                  <button
                    type="button"
                    class="rounded-md border border-[#30363d] bg-[#21262d] px-3 py-1.5 text-xs font-medium text-[#c9d1d9] hover:bg-[#30363d] disabled:opacity-50"
                    :disabled="busy"
                    @click="handleOffline"
                  >
                    Сохранить
                  </button>
                </div>

                <div class="relative flex items-center justify-center my-2">
                  <div class="border-t border-[#30363d] w-full"></div>
                  <span class="bg-[#161b22] px-2 text-[10px] uppercase text-[#8b949e] absolute">или</span>
                </div>

                <button
                  type="button"
                  class="w-full rounded-md border border-[#30363d] bg-[#21262d] py-1.5 text-xs font-medium text-[#c9d1d9] hover:bg-[#30363d] disabled:opacity-50"
                  :disabled="busy || msPolling"
                  @click="handleMicrosoft"
                >
                  {{ msPolling ? "Ожидание подтверждения…" : "Войти через Microsoft" }}
                </button>

                <!-- Device code flow: показать код и ссылку -->
                <div
                  v-if="msFlow"
                  class="rounded-md border border-[#1f6beb]/40 bg-[#0d1117]/60 p-3 space-y-2"
                >
                  <p class="text-[11px] text-[#8b949e]">
                    Откройте страницу и введите код:
                  </p>
                  <p class="font-mono text-2xl font-bold tracking-[0.3em] text-[#79c0ff] select-text">
                    {{ msFlow.user_code }}
                  </p>
                  <button
                    type="button"
                    class="rounded-md border border-[#1f6beb]/50 bg-[#1f6beb]/20 px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-[#1f6beb]/40"
                    @click="openMsAuthPage"
                  >
                    Открыть страницу {{ msFlow.verification_uri.replace(/^https?:\/\//, "") }}
                  </button>
                  <p v-if="msPolling" class="flex items-center gap-2 text-[11px] text-[#8b949e]">
                    <svg class="h-3 w-3 animate-spin fill-[#58a6ff]" viewBox="0 0 16 16">
                      <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
                    </svg>
                    Ждём подтверждения в браузере…
                  </p>
                </div>
              </div>
            </section>

            <!-- ОЗУ -->
            <section class="rounded-md border border-[#30363d] bg-[#161b22] overflow-hidden">
              <div class="border-b border-[#30363d] bg-[#21262d]/50 px-4 py-2.5 flex justify-between items-center">
                <h3 class="text-xs font-semibold text-[#f0f6fc]">Выделение оперативной памяти</h3>
                <span class="font-mono text-xs font-semibold text-[#58a6ff]">{{ ram }} ГБ</span>
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
                  <span>2 ГБ</span>
                  <span>{{ maxRam }} ГБ (Макс)</span>
                </div>
                <p v-if="systemRam && systemRam.total_ram_gb > 0" class="text-[11px] text-[#8b949e]">
                  Всего в системе: {{ systemRam.total_ram_gb }} ГБ | Доступно: {{ systemRam.available_ram_gb }} ГБ
                </p>
              </div>
            </section>

            <!-- Размер окна игры -->
            <section class="rounded-md border border-[#30363d] bg-[#161b22] overflow-hidden">
              <div class="border-b border-[#30363d] bg-[#21262d]/50 px-4 py-2.5 flex justify-between items-center">
                <h3 class="text-xs font-semibold text-[#f0f6fc]">Размер окна игры</h3>
                <span class="font-mono text-xs font-semibold text-[#58a6ff]">{{ windowWidth }}×{{ windowHeight }}</span>
              </div>
              <div class="p-4 space-y-2">
                <div class="flex items-center gap-3">
                  <label class="w-16 text-[11px] text-[#8b949e]" for="win-width">Ширина</label>
                  <input
                    id="win-width"
                    type="number"
                    min="320"
                    max="7680"
                    step="1"
                    v-model.number="windowWidth"
                    class="flex-1 rounded-md border border-[#30363d] bg-[#0d1117] px-3 py-1.5 text-xs text-[#c9d1d9] focus:border-[#58a6ff] focus:outline-none"
                  />
                  <label class="w-16 text-[11px] text-[#8b949e]" for="win-height">Высота</label>
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
                  Применяется при следующем запуске игры.
                </p>
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
import { ref } from "vue";
import { isTauri, openExternal } from "~/lib/bridge";
import { useLauncher } from "~/composables/useLauncher";

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
} = useLauncher();

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
</script>
