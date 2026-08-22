<template>
  <aside
    class="relative flex shrink-0 flex-col bg-[var(--panel)]"
    :class="[sidebarDragging ? '' : 'transition-[width] duration-150', sidebarCollapsed ? 'items-center' : '']"
    :style="{ width: `${sidebarWidth}px` }"
  >
<!-- Выбор сборки (вкладка каждого репозитория) -->
    <div v-if="!sidebarCollapsed" class="relative p-3.5 border-b border-[var(--border)]">
      <div class="flex items-center justify-between gap-2">
        <label class="text-[11px] font-semibold uppercase tracking-wider text-[color:var(--tx-muted)]">
          {{ t("side.packRepo") }}
        </label>
        <div class="flex items-center gap-1.5">
          <button
            v-if="activePack && !activePack.builtin"
            type="button"
            class="shrink-0 rounded-md border px-2 py-1.5 leading-none transition-colors disabled:opacity-50"
            :class="removeArmed === activePack.id
              ? 'border-[#f85149]/60 bg-[#f85149]/15 text-[#f85149]'
              : 'border-[var(--border)] bg-[var(--input)] text-[var(--tx-muted)] hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]'"
            :title="removeArmed === activePack.id ? t('dev.removeConfirm') : t('dev.remove')"
            :disabled="busy || removingPack === activePack.id"
            @click.stop="handleRemovePack(activePack.id)"
          >
            <svg v-if="removingPack === activePack.id" viewBox="0 0 16 16" class="h-3.5 w-3.5 animate-spin fill-current">
              <path d="M8 1.5a.75.75 0 0 1 .75.75V8a.75.75 0 0 1-1.5 0V2.25A.75.75 0 0 1 8 1.5Zm3.36 2.14a.75.75 0 0 1 0 1.06 4 4 0 1 1-6.72 0 .75.75 0 0 1 1.06-1.06 2.5 2.5 0 1 0 4.6 0 .75.75 0 0 1 1.06-1.06Z"/>
            </svg>
            <svg v-else viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-current">
              <path d="M6 1.75a.75.75 0 0 1 .75-.75h2.5a.75.75 0 0 1 .75.75V2h3.5a.75.75 0 0 1 0 1.5h-.38l-.89 10.055A1.75 1.75 0 0 1 10.495 15H5.505a1.75 1.75 0 0 1-1.735-1.445L2.88 3.5H2.5a.75.75 0 0 1 0-1.5H6v-.25ZM4.416 3.5l.864 9.9A.25.25 0 0 0 5.525 13.5h4.95a.25.25 0 0 0 .245-.22l.864-9.78H4.416Z"/>
            </svg>
          </button>
        </div>
        </div>
      <div class="mt-2 flex items-center gap-1.5 text-[11px] text-[color:var(--tx-muted)]">
        <svg viewBox="0 0 16 16" class="h-3.5 w-3.5 fill-[var(--tx-muted)] shrink-0">
          <path d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-1 1v.878A2.25 2.25 0 1 1 2 13.378V2.5Z"/>
        </svg>
        <span v-if="activePack?.author" class="truncate font-mono text-[var(--accent)]">@{{ activePack.author }}</span>
        <button
          v-if="activePackRepo"
          type="button"
          class="shrink-0 truncate font-mono text-[color:var(--tx-muted)] transition-colors hover:text-[color:var(--tx)] hover:underline"
          :title="activePackRepo"
          @click="openExternal(activePackRepo)"
        >
          открыть репозиторий
        </button>
      </div>
    </div>

    <!-- Навигация -->
    <nav class="flex min-h-0 flex-1 flex-col overflow-y-auto border-b border-[var(--border)] p-2" :class="sidebarCollapsed ? 'gap-1.5' : 'gap-0.5'">
      <!-- Вкладки категорий сборок: авторские / свои / Modrinth / CurseForge (перетаскиваются) -->
      <template v-if="!sidebarCollapsed">
        <button
          type="button"
          class="flex w-full items-center gap-1 px-3 pb-1 pt-2 text-left text-[10px] font-semibold uppercase tracking-wider text-[color:var(--tx-muted)]"
          disabled
        >
          {{ t("side.recent") }}
        </button>
        <template v-for="rp in sidebarRecentPacks" :key="rp.pack.id">
          <button
            type="button"
            class="flex items-center gap-2 rounded-md border border-[var(--border)] px-3 py-1.5 text-xs font-medium transition-colors"
            :class="tab === 'play' && packId === rp.pack.id ? 'border-[color-mix(in_srgb,var(--accent)_40%,transparent)] bg-[var(--input)] text-[color:var(--tx-strong)]' : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]'"
            @click="openPackTab(rp.pack.id)"
          >
            <img
              v-if="rp.pack.icon"
              :src="convertFileSrc(rp.pack.icon)"
              alt=""
              class="h-4 w-4 shrink-0 rounded object-cover"
            />
            <svg v-else viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current">
              <path d="M1 7.775V2.75C1 1.784 1.784 1 2.75 1h5.025c.464 0 .91.184 1.238.513l6.25 6.25a1.75 1.75 0 0 1 0 2.474l-5.026 5.026a1.75 1.75 0 0 1-2.474 0l-6.25-6.25A1.752 1.752 0 0 1 1 7.775Zm1.5 0c0 .066.026.13.073.177l6.25 6.25a.25.25 0 0 0 .354 0l5.025-5.025a.25.25 0 0 0 0-.354l-6.25-6.25a.25.25 0 0 0-.177-.073H2.75a.25.25 0 0 0-.25.25ZM6 5a1 1 0 1 0 0 2 1 1 0 0 0 0-2Z"/>
            </svg>
            <span class="min-w-0 flex-1 truncate text-left">{{ rp.pack.name }}</span>
            <span v-if="rp.pack.id === packId" class="h-2 w-2 shrink-0 rounded-full" :class="status?.installed ? 'bg-[#3fb950]' : 'bg-[var(--tx-muted)]'"></span>
          </button>
        </template>
        <p v-if="sidebarRecentPacks.length === 0" class="px-3 py-1 text-[11px] text-[color:var(--tx-muted)]">
          {{ t("side.recentEmpty") }}
        </p>
      </template>
      <!-- Свернутый режим: только иконки недавних сборок -->
      <template v-else>
        <button
          v-for="rp in sidebarRecentPacks"
          :key="rp.pack.id"
          type="button"
          class="flex items-center justify-center rounded-md p-1.5 transition-colors"
          :class="tab === 'play' && packId === rp.pack.id ? 'bg-[var(--input)] text-[var(--accent)]' : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]'"
          :title="rp.pack.name"
          @click="openPackTab(rp.pack.id)"
        >
          <img
            v-if="rp.pack.icon"
            :src="convertFileSrc(rp.pack.icon)"
            alt=""
            class="h-6 w-6 shrink-0 rounded object-cover"
          />
          <svg v-else viewBox="0 0 16 16" class="h-5 w-5 shrink-0 fill-current">
            <path d="M1 7.775V2.75C1 1.784 1.784 1 2.75 1h5.025c.464 0 .91.184 1.238.513l6.25 6.25a1.75 1.75 0 0 1 0 2.474l-5.026 5.026a1.75 1.75 0 0 1-2.474 0l-6.25-6.25A1.752 1.752 0 0 1 1 7.775Zm1.5 0c0 .066.026.13.073.177l6.25 6.25a.25.25 0 0 0 .354 0l5.025-5.025a.25.25 0 0 0 0-.354l-6.25-6.25a.25.25 0 0 0-.177-.073H2.75a.25.25 0 0 0-.25.25ZM6 5a1 1 0 1 0 0 2 1 1 0 0 0 0-2Z"/>
          </svg>
        </button>
      </template>
    </nav>

    <nav class="flex flex-col gap-0.5 p-2 border-b border-[var(--border)]">
      <button
        type="button"
        class="flex items-center rounded-md py-1.5 text-xs font-medium transition-colors"
        :class="[
          sidebarCollapsed ? 'justify-center px-1.5' : 'justify-start gap-2 px-3',
          tab === 'news' ? 'bg-[var(--input)] text-[color:var(--tx-strong)]' : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]',
        ]"
        :title="t('nav.news')"
        @click="tab = 'news'"
      >
        <svg viewBox="0 0 16 16" class="shrink-0 fill-current" :class="sidebarCollapsed ? 'h-[18px] w-[18px]' : 'h-4 w-4'">
          <path d="M1.5 3.25A2.25 2.25 0 0 1 3.75 1h8.5A2.25 2.25 0 0 1 14.5 3.25v9.5A2.25 2.25 0 0 1 12.25 15H3.75a2.25 2.25 0 0 1-2.25-2.25v-9.5Zm1.5 0v9.5c0 .414.336.75.75.75h8.5a.75.75 0 0 0 .75-.75v-9.5a.75.75 0 0 0-.75-.75h-8.5a.75.75 0 0 0-.75.75ZM4 5.5A.75.75 0 0 1 4.75 4.75h1.5a.75.75 0 0 1 0 1.5h-1.5A.75.75 0 0 1 4 5.5Zm3.75 0a.75.75 0 0 1 .75-.75h3.75a.75.75 0 0 1 0 1.5H8.5a.75.75 0 0 1-.75-.75ZM4 8.5a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 0 1.5h-1.5A.75.75 0 0 1 4 8.5Zm3.75 0a.75.75 0 0 1 .75-.75h3.75a.75.75 0 0 1 0 1.5H8.5a.75.75 0 0 1-.75-.75Zm-3.75 3a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 0 1.5h-1.5a.75.75 0 0 1-.75-.75Z"/>
        </svg>
        <span v-if="!sidebarCollapsed">{{ t("nav.news") }}</span>
      </button>
      <button
        type="button"
        class="flex items-center rounded-md py-1.5 text-xs font-medium transition-colors"
        :class="[
          sidebarCollapsed ? 'justify-center px-1.5' : 'justify-start gap-2 px-3',
          tab === 'catalog' ? 'bg-[var(--input)] text-[color:var(--tx-strong)]' : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]',
        ]"
        :title="t('nav.catalog')"
        @click="tab = 'catalog'"
      >
        <svg viewBox="0 0 16 16" class="shrink-0 fill-current" :class="sidebarCollapsed ? 'h-[18px] w-[18px]' : 'h-4 w-4'">
          <path d="M1.75 2A1.75 1.75 0 0 0 0 3.75v3.5C0 8.216.784 9 1.75 9h3.5A1.75 1.75 0 0 0 7 7.25v-3.5A1.75 1.75 0 0 0 5.25 2h-3.5Zm0 1.5h3.5c.138 0 .25.112.25.25v3.5c0 .138-.112.25-.25.25h-3.5a.25.25 0 0 1-.25-.25v-3.5c0-.138.112-.25.25-.25ZM10.75 2A1.75 1.75 0 0 0 9 3.75v3.5c0 .966.784 1.75 1.75 1.75h3.5A1.75 1.75 0 0 0 16 7.25v-3.5A1.75 1.75 0 0 0 14.25 2h-3.5Zm0 1.5h3.5c.138 0 .25.112.25.25v3.5c0 .138-.112.25-.25.25h-3.5a.25.25 0 0 1-.25-.25v-3.5c0-.138.112-.25.25-.25ZM1.75 10A1.75 1.75 0 0 0 0 11.75v.5C0 13.216.784 14 1.75 14h3.5A1.75 1.75 0 0 0 7 12.25v-.5A1.75 1.75 0 0 0 5.25 10h-3.5Zm0 1.5h3.5c.138 0 .25.112.25.25v.5c0 .138-.112.25-.25.25h-3.5a.25.25 0 0 1-.25-.25v-.5c0-.138.112-.25.25-.25ZM10.75 10A1.75 1.75 0 0 0 9 11.75v.5c0 .966.784 1.75 1.75 1.75h3.5A1.75 1.75 0 0 0 16 12.25v-.5A1.75 1.75 0 0 0 14.25 10h-3.5Zm0 1.5h3.5c.138 0 .25.112.25.25v.5c0 .138-.112.25-.25.25h-3.5a.25.25 0 0 1-.25-.25v-.5c0-.138.112-.25.25-.25Z"/>
        </svg>
        <span v-if="!sidebarCollapsed">{{ t("nav.catalog") }}</span>
      </button>
      <button
        type="button"
        class="flex items-center rounded-md py-1.5 text-xs font-medium transition-colors"
        :class="[
          sidebarCollapsed ? 'justify-center px-1.5' : 'justify-start gap-2 px-3',
          tab === 'library' ? 'bg-[var(--input)] text-[color:var(--tx-strong)]' : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]',
        ]"
        :title="t('nav.library')"
        @click="tab = 'library'"
      >
        <svg viewBox="0 0 16 16" class="shrink-0 fill-current" :class="sidebarCollapsed ? 'h-[18px] w-[18px]' : 'h-4 w-4'">
          <path d="M0 1.75A.75.75 0 0 1 .75 1h4.253c1.227 0 2.317.59 3 1.501A3.744 3.744 0 0 1 11.006 1h4.245a.75.75 0 0 1 .75.75v10.5a.75.75 0 0 1-.75.75h-4.507a2.25 2.25 0 0 0-1.591.659l-.622.621a.75.75 0 0 1-1.06 0l-.622-.621A2.25 2.25 0 0 0 5.258 13H.75a.75.75 0 0 1-.75-.75Zm7.251 10.324.004-5.073-.002-2.253A2.25 2.25 0 0 0 5.003 2.5H1.5v9h3.757a3.75 3.75 0 0 1 1.994.574ZM8.755 4.846V7.06h7.745V2.5h-3.496a2.249 2.249 0 0 0-2.24 2.236l-.009.11Zm-.001 7.003a3.752 3.752 0 0 1 2.003-.575H14.5v-9h-3.495a2.249 2.249 0 0 0-2.24 2.236l-.009.111-.001 5.228Z"/>
        </svg>
        <span v-if="!sidebarCollapsed">{{ t("nav.library") }}</span>
      </button>
      <button
        type="button"
        class="flex items-center rounded-md py-1.5 text-xs font-medium transition-colors"
        :class="sidebarCollapsed ? 'justify-center px-1.5' : 'justify-start gap-2 px-3'"
        :title="t('side.createInstance')"
        @click="createPackOpen = true"
      >
        <svg viewBox="0 0 16 16" class="shrink-0 fill-[var(--accent)]" :class="sidebarCollapsed ? 'h-[18px] w-[18px]' : 'h-4 w-4'">
          <path d="M8 0a1 1 0 0 1 1 1v6h6a1 1 0 1 1 0 2H9v6a1 1 0 1 1-2 0V9H1a1 1 0 0 1 0-2h6V1a1 1 0 0 1 1-1Z"/>
        </svg>
        <span v-if="!sidebarCollapsed">{{ t("side.createInstance") }}</span>
      </button>
      <button
        type="button"
        class="flex items-center rounded-md py-1.5 text-xs font-medium transition-colors"
        :class="[
          sidebarCollapsed ? 'justify-center px-1.5' : 'justify-start gap-2 px-3',
          tab === 'settings' ? 'bg-[var(--input)] text-[color:var(--tx-strong)]' : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]',
        ]"
        :title="t('nav.settings')"
        @click="tab = 'settings'"
      >
        <svg viewBox="0 0 24 24" class="shrink-0 fill-none stroke-current" :class="sidebarCollapsed ? 'h-[18px] w-[18px]' : 'h-4 w-4'" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3"></circle>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1Z"></path>
        </svg>
        <span v-if="!sidebarCollapsed">{{ t("nav.settings") }}</span>
      </button>
      <button
        type="button"
        class="flex items-center rounded-md py-1.5 text-xs font-medium transition-colors"
        :class="[
          sidebarCollapsed ? 'justify-center px-1.5' : 'justify-start gap-2 px-3',
          tab === 'author' ? 'bg-[var(--input)] text-[color:var(--tx-strong)]' : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]',
        ]"
        :title="t('nav.author')"
        @click="tab = 'author'"
      >
        <svg viewBox="0 0 16 16" class="shrink-0 fill-current" :class="sidebarCollapsed ? 'h-[18px] w-[18px]' : 'h-4 w-4'">
          <path d="M8 1a3 3 0 1 0 0 6 3 3 0 0 0 0-6ZM2 13.25C2 10.75 4.46 9.25 8 9.25s6 1.5 6 4V14a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1v-.75Z"/>
        </svg>
        <span v-if="!sidebarCollapsed">{{ t("nav.author") }}</span>
      </button>
      <button
        v-if="isAdmin"
        type="button"
        class="flex items-center rounded-md py-1.5 text-xs font-medium transition-colors"
        :class="[
          sidebarCollapsed ? 'justify-center px-1.5' : 'justify-start gap-2 px-3',
          tab === 'admin' ? 'bg-[var(--input)] text-[color:var(--tx-strong)]' : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]',
        ]"
        :title="t('nav.admin')"
        @click="tab = 'admin'"
      >
        <svg viewBox="0 0 16 16" class="shrink-0 fill-current" :class="sidebarCollapsed ? 'h-[18px] w-[18px]' : 'h-4 w-4'">
          <path d="M8 1.5 9.3 4.4l3.2.35-2.4 2.15.7 3.1L8 8.55l-2.8 1.45.7-3.1L3.5 4.75l3.2-.35ZM2.5 11.5a.75.75 0 0 1 .75-.75h2a.75.75 0 0 1 0 1.5h-2a.75.75 0 0 1-.75-.75Zm7.5 0a.75.75 0 0 1 .75-.75h2a.75.75 0 0 1 0 1.5h-2a.75.75 0 0 1-.75-.75ZM4.25 13.75a.75.75 0 0 0 0 1.5h7.5a.75.75 0 0 0 0-1.5Z"/>
        </svg>
        <span v-if="!sidebarCollapsed">{{ t("nav.admin") }}</span>
      </button>
      <button
        type="button"
        class="flex items-center rounded-md py-1.5 text-xs font-medium transition-colors"
        :class="[
          sidebarCollapsed ? 'justify-center px-1.5' : 'justify-start gap-2 px-3',
          tab === 'dev' ? 'bg-[var(--input)] text-[color:var(--tx-strong)]' : 'text-[color:var(--tx-muted)] hover:bg-[var(--input-50)] hover:text-[color:var(--tx)]',
        ]"
        :title="t('side.dev')"
        @click="tab = 'dev'"
      >
        <svg viewBox="0 0 16 16" class="shrink-0 fill-current" :class="sidebarCollapsed ? 'h-[18px] w-[18px]' : 'h-4 w-4'">
          <path d="M2 1.75C2 .784 2.784 0 3.75 0h8.5C13.216 0 14 .784 14 1.75v12.5A1.75 1.75 0 0 1 12.25 16h-8.5A1.75 1.75 0 0 1 2 14.25Zm1.69 1.884a.75.75 0 0 1 .79.075l4.244 3.253a.75.75 0 0 1 0 1.13L4.48 11.345a.75.75 0 0 1-.79.075.75.75 0 0 1-.388-.67v-6.5a.75.75 0 0 1 .388-.547ZM10.5 8.75h3a.75.75 0 0 0 0-1.5h-3a.75.75 0 0 0 0 1.5Z"/>
        </svg>
        <span v-if="!sidebarCollapsed">{{ t("side.dev") }}</span>
      </button>
    </nav>

    <!-- Сводка статуса -->
    <div v-if="!sidebarCollapsed" class="space-y-2 p-3.5 text-xs text-[color:var(--tx-muted)]">
      <div class="flex items-center justify-between">
        <span>{{ t("side.status") }}</span>
        <span class="inline-flex items-center gap-1.5 font-medium">
          <span class="h-2 w-2 rounded-full" :class="status?.installed ? 'bg-[#3fb950]' : 'bg-[var(--tx-muted)]'"></span>
          <span :class="status?.installed ? 'text-[color:var(--tx-strong)]' : 'text-[color:var(--tx-muted)]'">
            {{ status?.installed ? t("side.installed") : t("side.notInstalled") }}
          </span>
        </span>
      </div>
      <div class="flex items-center justify-between">
        <span>{{ t("side.memory") }}</span>
        <span class="font-mono font-medium text-[color:var(--tx)]">{{ ram }} {{ t("units.gb") }}</span>
      </div>
      <div class="flex items-center justify-between">
        <span>{{ t("side.launcherVersion") }}</span>
        <span class="font-mono font-medium text-[color:var(--tx)]">v{{ launcherVer || "?" }}</span>
      </div>
    </div>

    <!-- Глобальный прогресс установки/скачивания -->
    <div v-if="progress && busy" class="border-t border-[var(--border)] p-3 bg-[var(--panel-soft)]">
      <div class="mb-1 flex items-center justify-between text-[11px] text-[color:var(--tx-muted)]">
        <span class="truncate pr-2 font-medium text-[color:var(--tx)]">{{ phaseLabel(progress.phase) }}</span>
        <span v-if="progress.fileTotal > 1" class="tabular-nums font-mono text-[10px]">{{ t("progress.files", { n: filesDone, m: progress.fileTotal }) }}</span>
        <span v-else class="tabular-nums font-mono text-[10px]">{{ percent }}%</span>
      </div>
      <div class="h-1.5 w-full overflow-hidden rounded-full bg-[var(--input)]">
        <div
          class="h-full bg-[#2f81f7] transition-all duration-200"
          :style="{ width: `${percent}%` }"
        />
      </div>
      <div class="mt-1 flex items-center justify-between text-[10px] text-[color:var(--tx-muted)]">
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

    <div class="flex-1" />

    <!-- Учётная запись -->
    <div class="flex items-center gap-2.5 border-t border-[var(--border)] p-3 bg-[var(--bg-30)]" :class="sidebarCollapsed ? 'justify-center p-2' : ''">
      <div class="flex h-7 w-7 shrink-0 items-center justify-center overflow-hidden rounded-full border border-[var(--border)] bg-[var(--input)] font-mono text-xs font-bold text-[color:var(--tx-strong)]">
        <img v-if="skinUrl" :src="skinUrl" :alt="t('side.skin')" class="h-full w-full object-cover" />
        <template v-else>{{ session?.username?.[0]?.toUpperCase() ?? "?" }}</template>
      </div>
      <div v-if="!sidebarCollapsed" class="min-w-0 flex-1">
        <div class="truncate text-xs font-medium text-[color:var(--tx)]">
          {{ session?.username ?? t("side.guest") }}
        </div>
        <div class="truncate text-[10px] text-[color:var(--tx-muted)]">
          {{ session ? session.user_type : t("side.offline") }}
        </div>
      </div>
    </div>

    <!-- Главное действие (Кнопка запуска) — только во вкладке сборки -->
    <div v-if="tab === 'play'" class="p-3 border-t border-[var(--border)] bg-[var(--panel)]">
      <button
        type="button"
        class="flex items-center justify-center rounded-md py-2.5 text-sm font-bold tracking-wide text-white shadow-sm transition-all active:scale-[0.98] focus-visible:outline focus-visible:outline-offset-2 disabled:opacity-50 disabled:cursor-not-allowed disabled:active:scale-100"
        :class="[
          sidebarCollapsed ? 'px-1.5' : 'w-full px-4',
          status?.installed
            ? gameRunning
              ? 'bg-[#b91c1c] hover:bg-[#dc2626] focus-visible:outline-[#dc2626]'
              : 'bg-[#238636] hover:bg-[#2ea043] focus-visible:outline-[#2ea043]'
            : 'bg-[var(--accent-deep)] hover:bg-[var(--accent-hover)] focus-visible:outline-[var(--accent-hover)]',
        ]"
        :title="status?.installed ? (gameRunning ? t('side.stopGame') : t('side.play')) : t('side.downloadPlay')"
        :disabled="busy"
        @click="status?.installed ? (gameRunning ? handleStop() : handlePlay()) : handleInstall()"
      >
        <template v-if="!sidebarCollapsed">
          <template v-if="!status?.installed">
            {{ busy ? t("side.installing") : t("side.downloadPlay") }}
          </template>
          <template v-else>
            {{ busy ? t("side.launching") : gameRunning ? t("side.stopGame") : t("side.play") }}
          </template>
        </template>
        <svg v-else viewBox="0 0 16 16" class="h-4 w-4 fill-current">
          <path
            v-if="gameRunning"
            d="M3.5 3.5h9v9h-9z"
          />
          <path
            v-else
            d="M4.5 1.94a1 1 0 0 1 1.523-.853l9.6 6.06a1 1 0 0 1 0 1.707l-9.6 6.06A1 1 0 0 1 4.5 14.06V1.94Z"
          />
        </svg>
      </button>
    </div>

    <!-- Версия и перевод лаунчера -->
    <div v-if="!sidebarCollapsed" class="flex items-center justify-between gap-2 border-t border-[var(--border)] bg-[var(--panel)] px-3 py-2 text-[9px] text-[var(--tx-muted)]">
      <span class="min-w-0 truncate">
        {{ t("lang.byAuthor") }}
        <span class="font-semibold" :class="activeLocaleAuthor ? 'text-[color:var(--tx)]' : ''">{{ activeLocaleAuthor || "—" }}</span>
        <template v-if="activeLocaleVersion"> · v{{ activeLocaleVersion }}</template>
      </span>
      <span class="shrink-0 tabular-nums">{{ t("lang.launcherVer") }} v{{ launcherVer || "?" }}</span>
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
  activePack,
  sidebarRecentPacks,
  percent,
  filePercent,
  filesDone,
  handleInstall,
  handlePlay,
  handleStop,
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
  activePackRepo,
  activeLocaleAuthor,
  activeLocaleVersion,
  formatBytes,
  phaseLabel,
  startSidebarDrag,
  onSidebarDrag,
  endSidebarDrag,
} = useLauncherCtx();
</script>