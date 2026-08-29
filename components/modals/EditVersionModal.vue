<script setup lang="ts">
import { useLauncherCtx } from "~/composables/useLauncherContext";
import { useI18n } from "#imports";

const { t } = useI18n();
const {
  editVerOpen,
  editVerName,
  editVerMc,
  editVerMcOpen,
  editVerMcQuery,
  editVerMcBox,
  editVerMcList,
  editVerLoader,
  editVerLv,
  editVerLvOpen,
  editVerLvBox,
  editVerLoaderVersions,
  editVerBusy,
  activePack,
  activeBanner,
  activePackRepo,
  CREATE_LOADERS,
  chooseEditVerMc,
  chooseEditVerLoaderVersion,
  pickPackIcon,
  pickPackBanner,
  saveEditVersion,
  packId,
  convertFileSrc,
} = useLauncherCtx();
</script>

<template>
  <div v-if="editVerOpen" class="fixed inset-0 z-[60] flex items-center justify-center bg-black/40 p-4" @click.self="editVerOpen = false">
    <div class="w-full max-w-md overflow-hidden rounded-xl  bg-[var(--panel)] shadow-2xl">
      <div class="flex items-center justify-between gap-2 border-b border-[var(--border)]  px-3.5 py-2.5">
        <h3 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ t("pack.versionTitle") }}</h3>
        <button
          type="button"
          class="rounded-md p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
          @click="editVerOpen = false"
        >
          <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
        </button>
      </div>
      <div class="space-y-3 px-3.5 py-2.5">
        <div>
          <label class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("pack.editName") }}</label>
          <input
            v-model="editVerName"
            class="w-full rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] text-[color:var(--tx)] outline-none placeholder:text-[var(--tx-muted)] "
            :placeholder="activePack?.name || ''"
            maxlength="60"
          />
        </div>

        <div>
          <label class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("pack.editMedia") }}</label>
          <div class="flex items-center gap-2">
            <img
              v-if="activePack?.icon"
              :src="convertFileSrc(activePack.icon)"
              alt=""
              class="h-9 w-9 shrink-0 rounded-md  bg-[var(--input)] object-cover"
            />
            <div v-else class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md  bg-[var(--input)] text-[color:var(--tx-muted)]">
              <svg viewBox="0 0 16 16" class="h-5 w-5 fill-current"><path d="M1 7.775V2.75C1 1.784 1.784 1 2.75 1h5.025c.464 0 .91.184 1.238.513l6.25 6.25a1.75 1.75 0 0 1 0 2.474l-5.026 5.026a1.75 1.75 0 0 1-2.474 0l-6.25-6.25A1.752 1.752 0 0 1 1 7.775Zm1.5 0c0 .066.026.13.073.177l6.25 6.25a.25.25 0 0 0 .354 0l5.025-5.025a.25.25 0 0 0 0-.354l-6.25-6.25a.25.25 0 0 0-.177-.073H2.75a.25.25 0 0 0-.25.25ZM6 5a1 1 0 1 0 0 2 1 1 0 0 0 0-2Z"/></svg>
            </div>
            <button
              type="button"
              class="flex-1 rounded-md  bg-[var(--input)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
              @click="pickPackIcon(packId)"
            >
              {{ t("dev.changeIcon") }}
            </button>
            <span v-if="activeBanner" class="inline-block h-9 w-24 shrink-0 rounded-md  object-cover" :style="`background-image:url('${activeBanner}');background-size:cover;background-position:center`"></span>
            <button
              type="button"
              class="flex-1 rounded-md  bg-[var(--input)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
              @click="pickPackBanner(packId)"
            >
              {{ activeBanner ? t("dev.changeBanner") : t("dev.setBanner") }}
            </button>
          </div>
        </div>

        <div>
          <label class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("pack.versionMc") }}</label>
          <div ref="editVerMcBox" class="relative">
            <button
              type="button"
              class="flex w-full items-center justify-between gap-2 rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
              @click="editVerMcOpen = !editVerMcOpen"
            >
              <span class="truncate">{{ editVerMc || t("pack.versionPick") }}</span>
              <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-[var(--tx-muted)] transition-transform" :class="editVerMcOpen ? 'rotate-180' : ''"><path d="M4.22 6.22a.75.75 0 0 1 1.06 0L8 8.94l2.72-2.72a.75.75 0 1 1 1.06 1.06l-3.25 3.25a.75.75 0 0 1-1.06 0L4.22 7.28a.75.75 0 0 1 0-1.06Z"/></svg>
            </button>
            <div v-if="editVerMcOpen" class="absolute left-0 right-0 top-full z-50 mt-1 overflow-hidden rounded-xl  bg-[var(--panel)] shadow-sm shadow-2xl">
              <input v-model="editVerMcQuery" class="w-full border-b border-[var(--border)]  bg-[var(--input)] px-2.5 py-1.5 text-[13px] outline-none placeholder:text-[var(--tx-muted)]" :placeholder="t('pack.versionSearch')" />
              <div class="max-h-52 overflow-y-auto py-1">
                <button
                  v-for="v in editVerMcList"
                  :key="v.id"
                  type="button"
                  class="flex w-full items-center gap-2 px-3 py-1 text-left text-[13px] transition-colors hover:bg-[var(--hover)]"
                  :class="editVerMc === v.id ? 'text-[var(--accent)]' : 'text-[color:var(--tx)]'"
                  @click="chooseEditVerMc(v.id)"
                >
                  <span class="min-w-0 flex-1 truncate">{{ v.id }}</span>
                  <span v-if="v.kind === 'snapshot'" class="rounded  bg-[#9e6a03]/10 px-1.5 text-xs text-[#d29922]">{{ t("pack.snapshot") }}</span>
                </button>
              </div>
            </div>
          </div>
        </div>

        <div>
          <label class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("pack.versionLoader") }}</label>
          <div class="flex flex-wrap gap-1.5">
            <button
              v-for="l in CREATE_LOADERS"
              :key="l"
              type="button"
              class="rounded-md  px-2.5 py-1 text-[13px] font-medium capitalize transition-colors"
              :class="editVerLoader === l
                ? ' bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] text-[var(--accent)]'
                : ' bg-[var(--input)] text-[color:var(--tx)] hover:bg-[var(--hover)]'"
              @click="editVerLoader = l"
            >
              {{ l === "vanilla" ? t("pack.vanilla") : l }}
            </button>
          </div>
        </div>

        <div v-if="editVerLoader !== 'vanilla'">
          <label class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("pack.versionLoaderVer") }}</label>
          <div ref="editVerLvBox" class="relative">
            <button
              type="button"
              class="flex w-full items-center justify-between gap-2 rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
              @click="editVerLvOpen = !editVerLvOpen"
            >
              <span class="truncate">{{ editVerLv || t("mods.createLatest") }}</span>
              <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-[var(--tx-muted)] transition-transform" :class="editVerLvOpen ? 'rotate-180' : ''"><path d="M4.22 6.22a.75.75 0 0 1 1.06 0L8 8.94l2.72-2.72a.75.75 0 1 1 1.06 1.06l-3.25 3.25a.75.75 0 0 1-1.06 0L4.22 7.28a.75.75 0 0 1 0-1.06Z"/></svg>
            </button>
            <div v-if="editVerLvOpen" class="absolute left-0 right-0 top-full z-50 mt-1 max-h-52 overflow-y-auto rounded-xl  bg-[var(--panel)] shadow-sm py-1 shadow-2xl">
              <button
                type="button"
                class="flex w-full items-center justify-between px-3 py-1 text-left text-[13px] transition-colors hover:bg-[var(--hover)]"
                :class="editVerLv === '' ? 'text-[var(--accent)]' : 'text-[color:var(--tx)]'"
                @click="chooseEditVerLoaderVersion('')"
              >
                <span>{{ t("mods.createLatest") }}</span>
                <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M12.78 4.22a.75.75 0 0 1 0 1.06l-5.78 5.78a.75.75 0 0 1-1.06 0l-2.5-2.5a.75.75 0 1 1 1.06-1.06L6.5 9.44l5.22-5.22a.75.75 0 0 1 1.06 0Z"/></svg>
              </button>
              <div v-if="!editVerLoaderVersions.length" class="px-2.5 py-1.5 text-xs text-[color:var(--tx-muted)]">{{ t("mods.createLvNone") }}</div>
              <button
                v-for="v in editVerLoaderVersions"
                :key="v"
                type="button"
                class="flex w-full items-center justify-between gap-2 px-3 py-1 text-left text-[13px] transition-colors hover:bg-[var(--hover)]"
                :class="editVerLv === v ? 'text-[var(--accent)]' : 'text-[color:var(--tx)]'"
                @click="chooseEditVerLoaderVersion(v)"
              >
                <span class="min-w-0 truncate">{{ v }}</span>
                <svg v-if="editVerLv === v" viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-current"><path d="M12.78 4.22a.75.75 0 0 1 0 1.06l-5.78 5.78a.75.75 0 0 1-1.06 0l-2.5-2.5a.75.75 0 1 1 1.06-1.06L6.5 9.44l5.22-5.22a.75.75 0 0 1 1.06 0Z"/></svg>
              </button>
            </div>
          </div>
        </div>

        <div class="flex items-center justify-end gap-2 pt-1">
          <button
            type="button"
            class="rounded-md  bg-[var(--input)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx)]"
            @click="editVerOpen = false"
          >
            {{ t("files.cancel") }}
          </button>
          <button
            type="button"
            class="flex items-center gap-1.5 rounded-md  bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_22%,transparent)] disabled:opacity-50"
            :disabled="editVerBusy"
            @click="saveEditVersion"
          >
            <svg v-if="editVerBusy" viewBox="0 0 16 16" class="h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
            {{ t("pack.versionSave") }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
