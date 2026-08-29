<script setup lang="ts">
import { useLauncherCtx } from "~/composables/useLauncherContext";
import { useI18n } from "#imports";

const { t } = useI18n();
const {
  createPackOpen,
  createPackName,
  createPackMc,
  createPackVersionOpen,
  createPackVersionQuery,
  createPackVersionBox,
  createVersionGroups,
  filteredCreateReleases,
  filteredCreateSnapshots,
  createPackLoader,
  createPackLoaderVersion,
  createPackLoaderLvOpen,
  createPackLvBox,
  createPackLoaderVersions,
  createPackIcon,
  createPackBanner,
  createPackBusy,
  CREATE_LOADERS,
  chooseCreateVersion,
  chooseCreateLoaderVersion,
  pickCreateFile,
  createPack,
  openModPackModal,
} = useLauncherCtx();
</script>

<template>
  <div
    v-if="createPackOpen"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6"
    @click.self="createPackOpen = false"
  >
    <div class="flex w-full max-w-md flex-col overflow-hidden rounded-xl  bg-[var(--panel)] shadow-2xl">
      <div class="flex shrink-0 items-center justify-between border-b border-[var(--border)]  px-3.5 py-2.5">
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
          <label class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("mods.createName") }}</label>
          <input
            v-model="createPackName"
            type="text"
            :placeholder="t('mods.createNamePlaceholder')"
            class="w-full rounded-md  bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)] outline-none transition-colors "
            @keydown.enter="createPack"
          />
        </div>
        <div>
          <label class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("mods.createMc") }}</label>
          <div ref="createPackVersionBox" class="relative">
            <button
              type="button"
              class="flex w-full items-center justify-between gap-2 rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] "
              @click="createPackVersionOpen = !createPackVersionOpen"
            >
              <span class="truncate">{{ createPackMc }}</span>
              <svg
                viewBox="0 0 16 16"
                class="h-4 w-4 shrink-0 fill-[var(--tx-muted)] transition-transform"
                :class="createPackVersionOpen ? 'rotate-180' : ''"
              >
                <path d="M4.22 6.22a.75.75 0 0 1 1.06 0L8 8.94l2.72-2.72a.75.75 0 1 1 1.06 1.06l-3.25 3.25a.75.75 0 0 1-1.06 0L4.22 7.28a.75.75 0 0 1 0-1.06Z"/>
              </svg>
            </button>
            <div
              v-if="createPackVersionOpen"
              class="absolute left-0 right-0 top-full z-50 mt-1 flex max-h-64 flex-col overflow-hidden rounded-xl  bg-[var(--panel)] shadow-sm shadow-2xl"
            >
              <div class="shrink-0 border-b border-[var(--border)]  p-1.5">
                <input
                  v-model="createPackVersionQuery"
                  type="text"
                  :placeholder="t('mods.createSearch')"
                  class="w-full rounded-md  bg-[var(--bg)] px-2 py-1 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)] outline-none transition-colors "
                />
              </div>
              <div class="flex-1 overflow-y-auto py-1">
                <div v-for="group in createVersionGroups" :key="group.label">
                  <div
                    v-if="group.items.length"
                    class="px-3 pb-0.5 pt-1.5 text-[11px] font-semibold uppercase tracking-wider text-[color:var(--tx-muted)]"
                  >{{ group.label }}</div>
                  <button
                    v-for="v in group.items"
                    :key="v.id"
                    type="button"
                    class="flex w-full items-center gap-1.5 px-3 py-1 text-left text-[13px] transition-colors hover:bg-[var(--hover)]"
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
                  class="px-3 py-2 text-xs text-[color:var(--tx-muted)]"
                >{{ t("mods.createNone") }}</div>
              </div>
            </div>
          </div>
        </div>
        <div>
          <label class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("mods.createLoader") }}</label>
          <div class="flex flex-wrap gap-2">
            <button
              v-for="l in CREATE_LOADERS"
              :key="l"
              type="button"
              class="flex-1 rounded-md  px-2.5 py-1.5 text-[13px] font-medium capitalize transition-colors"
              :class="createPackLoader === l
                ? ' bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] text-[var(--accent)]'
                : ' bg-[var(--input)] text-[color:var(--tx)] hover:bg-[var(--hover)]'"
              @click="createPackLoader = l"
            >
              {{ l }}
            </button>
          </div>
        </div>
        <div v-if="createPackLoader !== 'vanilla'">
          <label class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("mods.createLoaderVersion") }}</label>
          <div ref="createPackLvBox" class="relative">
            <button
              type="button"
              class="flex w-full items-center justify-between gap-2 rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
              @click="createPackLoaderLvOpen = !createPackLoaderLvOpen"
            >
              <span class="truncate">{{ createPackLoaderVersion || t("mods.createLatest") }}</span>
              <svg
                viewBox="0 0 16 16"
                class="h-4 w-4 shrink-0 fill-[var(--tx-muted)] transition-transform"
                :class="createPackLoaderLvOpen ? 'rotate-180' : ''"
              >
                <path d="M4.22 6.22a.75.75 0 0 1 1.06 0L8 8.94l2.72-2.72a.75.75 0 1 1 1.06 1.06l-3.25 3.25a.75.75 0 0 1-1.06 0L4.22 7.28a.75.75 0 0 1 0-1.06Z"/>
              </svg>
            </button>
            <div
              v-if="createPackLoaderLvOpen"
              class="absolute left-0 right-0 top-full z-50 mt-1 max-h-52 overflow-y-auto rounded-xl  bg-[var(--panel)] shadow-sm py-1 shadow-2xl"
            >
              <button
                type="button"
                class="flex w-full items-center justify-between px-3 py-1 text-left text-[13px] transition-colors hover:bg-[var(--hover)]"
                :class="createPackLoaderVersion === '' ? 'text-[var(--accent)]' : 'text-[color:var(--tx)]'"
                @click="chooseCreateLoaderVersion('')"
              >
                <span>{{ t("mods.createLatest") }}</span>
                <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M12.78 4.22a.75.75 0 0 1 0 1.06l-5.78 5.78a.75.75 0 0 1-1.06 0l-2.5-2.5a.75.75 0 1 1 1.06-1.06L6.5 9.44l5.22-5.22a.75.75 0 0 1 1.06 0Z"/></svg>
              </button>
              <div v-if="!createPackLoaderVersions.length" class="px-2.5 py-1.5 text-xs text-[color:var(--tx-muted)]">{{ t("mods.createLvNone") }}</div>
              <button
                v-for="v in createPackLoaderVersions"
                :key="v"
                type="button"
                class="flex w-full items-center gap-2 px-3 py-1 text-left text-[13px] transition-colors hover:bg-[var(--hover)]"
                :class="createPackLoaderVersion === v ? 'text-[var(--accent)]' : 'text-[color:var(--tx)]'"
                @click="chooseCreateLoaderVersion(v)"
              >
                <span class="min-w-0 truncate">{{ v }}</span>
                <svg v-if="createPackLoaderVersion === v" viewBox="0 0 16 16" class="ml-auto h-4 w-4 shrink-0 fill-current"><path d="M12.78 4.22a.75.75 0 0 1 0 1.06l-5.78 5.78a.75.75 0 0 1-1.06 0l-2.5-2.5a.75.75 0 1 1 1.06-1.06L6.5 9.44l5.22-5.22a.75.75 0 0 1 1.06 0Z"/></svg>
              </button>
            </div>
          </div>
        </div>
        <div class="grid grid-cols-2 gap-2">
          <div>
            <label class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("mods.createIcon") }}</label>
            <button
              type="button"
              class="flex w-full items-center justify-center gap-1.5 rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)]"
              @click="pickCreateFile('icon')"
            >
              <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M4.5 2.75A2.75 2.75 0 0 1 7.25 0h1.5A2.75 2.75 0 0 1 11.5 2.75 2.75 2.75 0 0 1 16 5.5v5A2.75 2.75 0 0 1 13.25 13.25V13H2.75A2.75 2.75 0 0 1 0 10.25v-4.5A2.75 2.75 0 0 1 2.75 3c1.12 0 2.097.523 1.75-1.5Z"/></svg>
              <span class="min-w-0 truncate">{{ createPackIcon ? createPackIcon.split(/[\\/]/).pop() : t("mods.createChoose") }}</span>
            </button>
            <button
              v-if="createPackIcon"
              type="button"
              class="mt-1 w-full rounded-md px-2 py-0.5 text-xs font-medium text-[var(--accent)] hover:opacity-80"
              @click="createPackIcon = null"
            >{{ t("mods.createRemove") }}</button>
          </div>
          <div>
            <label class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("mods.createBanner") }}</label>
            <button
              type="button"
              class="flex w-full items-center justify-center gap-1.5 rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)]"
              @click="pickCreateFile('banner')"
            >
              <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.5 1.75A1.75 1.75 0 0 1 5.25 0h5.5c.966 0 1.75.784 1.75 1.75v12.5a.75.75 0 0 1-1.2.6L8 12.313l-3.3 2.537A.75.75 0 0 1 3.5 14.25V1.75Z"/></svg>
              <span class="min-w-0 truncate">{{ createPackBanner ? createPackBanner.split(/[\\/]/).pop() : t("mods.createChoose") }}</span>
            </button>
            <button
              v-if="createPackBanner"
              type="button"
              class="mt-1 w-full rounded-md px-2 py-0.5 text-xs font-medium text-[var(--accent)] hover:opacity-80"
              @click="createPackBanner = null"
            >{{ t("mods.createRemove") }}</button>
          </div>
        </div>
        <button
          type="button"
          class="flex w-full items-center justify-center gap-2 rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_10%,transparent)] px-3 py-2 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] disabled:opacity-50"
          :disabled="createPackBusy"
          @click="createPack"
        >
          <svg v-if="createPackBusy" viewBox="0 0 16 16" class="h-4 w-4 animate-spin fill-current">
            <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
          </svg>
          <svg v-else viewBox="0 0 16 16" class="h-4 w-4 fill-current">
            <path d="M8 2.75a.75.75 0 0 1 .75.75v3.75h3.75a.75.75 0 0 1 0 1.5h-3.75v3.75a.75.75 0 0 1-1.5 0V8.75H3.5a.75.75 0 0 1 0-1.5h3.75V3.5A.75.75 0 0 1 8 2.75Z"/>
          </svg>
          {{ t("mods.create") }}
        </button>
        <p class="text-xs leading-snug text-[color:var(--tx-muted)]">{{ t("mods.createHint") }}</p>
      </div>
      <button
        type="button"
        class="flex w-full items-center justify-center gap-2 px-4 pb-4 pt-0 text-[13px] font-medium text-[var(--accent)] transition-colors hover:opacity-80"
        @click="createPackOpen = false; openModPackModal()"
      >
        <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M7.25 1.75a.75.75 0 0 1 1.5 0v8.5l3.22-3.22a.75.75 0 1 1 1.06 1.06l-4.5 4.5a.75.75 0 0 1-1.06 0l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.22 3.22v-8.5Z"/></svg>
        {{ t("mods.createDownloadPack") }}
      </button>
    </div>
  </div>
</template>
