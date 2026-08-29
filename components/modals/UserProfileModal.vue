<script setup lang="ts">
import { useLauncherCtx } from "~/composables/useLauncherContext";
import { useI18n } from "#imports";

const { t } = useI18n();
const {
  profileView,
  profileBusy,
  closeProfileView,
  profileIsOwn,
  profileBioEditing,
  profileBioDraft,
  saveMyProfile,
  formatDate,
  openCatalogPackById,
  isAdmin,
  adminBusy,
  adminDeleteComment,
} = useLauncherCtx();
</script>

<template>
  <div v-if="profileView || profileBusy" class="fixed inset-0 z-[65] flex items-center justify-center bg-black/60 p-4" @click.self="closeProfileView()">
    <div class="flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-xl  bg-[var(--panel)] shadow-2xl">
      <div class="flex items-center justify-between gap-2 border-b border-[var(--border)]  px-3.5 py-2.5">
        <h3 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ t("profile.title") }}</h3>
        <button type="button" class="rounded-md  bg-[var(--input)] px-2 py-1 text-[13px] text-[color:var(--tx-muted)] hover:text-[var(--accent)] transition-colors" @click="closeProfileView()">
          <svg viewBox="0 0 16 16" class="h-3 w-3 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 5.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 7l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 8.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 7 3.72 3.78a.75.75 0 0 1 0-1.06Z"/></svg>
        </button>
      </div>

      <div v-if="profileBusy && !profileView" class="flex items-center justify-center py-16 text-[13px] text-[color:var(--tx-muted)]">
        <svg viewBox="0 0 16 16" class="mr-2 h-4 w-4 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
        {{ t("catalog.loading") }}
      </div>

      <template v-if="profileView">
        <div class="flex items-center gap-3 border-b border-[var(--border)]  px-3.5 py-2.5">
          <img v-if="profileView.profile.avatarUrl" :src="profileView.profile.avatarUrl" :alt="profileView.profile.user.username" class="h-12 w-12 shrink-0 rounded-full  object-cover" />
          <div v-else class="flex h-12 w-12 shrink-0 items-center justify-center rounded-full  bg-[var(--input)] font-mono text-lg font-bold text-[var(--accent)]">
            {{ profileView.profile.user.username?.[0]?.toUpperCase() ?? "?" }}
          </div>
          <div class="min-w-0 flex-1">
            <p class="truncate font-mono text-sm font-semibold text-[color:var(--tx-strong)]">{{ profileView.profile.user.username }}</p>
            <p class="text-[13px] text-[color:var(--tx-muted)]">
              {{ t("profile.joined") }}: {{ formatDate(profileView.profile.joinedAt) }} ·
              {{ t("profile.packsCount", { n: profileView.profile.packsCount }) }} ·
              {{ t("profile.commentsCount", { n: profileView.profile.commentsCount }) }}
            </p>
          </div>
        </div>

        <div class="min-h-0 flex-1 space-y-4 overflow-y-auto p-4">
          <!-- Bio -->
          <section class="rounded-lg  bg-[var(--bg)] p-3">
            <p class="mb-1 text-xs font-semibold uppercase tracking-wide text-[color:var(--tx-muted)]">{{ t("profile.bio") }}</p>
            <template v-if="profileIsOwn && profileBioEditing">
              <textarea v-model="profileBioDraft" rows="3" class="w-full resize-y rounded-md  bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)]  focus:outline-none"></textarea>
              <div class="mt-2 flex gap-2">
                <button type="button" class="rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-white disabled:opacity-50"
                  :disabled="profileBusy"
                  @click="saveMyProfile(profileBioDraft); profileBioEditing = false">
                  {{ t("author.save") }}
                </button>
                <button type="button" class="rounded-md  bg-[var(--input)] px-2.5 py-1 text-[13px] text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="profileBioEditing = false">
                  {{ t("author.cancel") }}
                </button>
              </div>
            </template>
            <template v-else>
              <p class="text-[13px] leading-relaxed text-[color:var(--tx)] whitespace-pre-wrap">{{ profileView.profile.bio || t("profile.noBio") }}</p>
              <button v-if="profileIsOwn" type="button" class="mt-1.5 text-xs font-medium text-[var(--accent)] hover:underline" @click="profileBioEditing = true; profileBioDraft = profileView!.profile.bio">
                {{ t("profile.editBio") }}
              </button>
            </template>
          </section>

          <!-- Сборки -->
          <section class="space-y-2">
            <p class="text-xs font-semibold uppercase tracking-wide text-[color:var(--tx-muted)]">{{ t("profile.packs") }}</p>
            <div v-if="profileView.packs.length === 0" class="rounded-lg  bg-[var(--bg)] p-3 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("author.noPacks") }}</div>
            <div v-for="p in profileView.packs" :key="p.id" class="flex items-center gap-3 rounded-lg  bg-[var(--bg)] p-3">
              <img v-if="p.iconUrl" :src="p.iconUrl" :alt="p.name" class="h-9 w-9 shrink-0 rounded-md object-cover" />
              <div v-else class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-[var(--input)] text-[13px] font-semibold text-[var(--accent)]">
                {{ p.name?.[0]?.toUpperCase() }}
              </div>
              <div class="min-w-0 flex-1">
                <p class="truncate text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ p.name }}</p>
                <p class="truncate text-xs text-[color:var(--tx-muted)]">
                  <template v-if="p.version">v{{ p.version }} · </template>👍 {{ p.likes }} / 👎 {{ p.dislikes }} · {{ t("author.versions") }}: {{ p.versionsCount }}
                </p>
              </div>
              <button type="button" class="shrink-0 rounded-md  bg-[var(--input)] px-2.5 py-1 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="openCatalogPackById(p.id)">
                {{ t("catalog.open") }}
              </button>
            </div>
          </section>

          <!-- Комментарии -->
          <section class="space-y-2">
            <p class="text-xs font-semibold uppercase tracking-wide text-[color:var(--tx-muted)]">{{ t("profile.comments") }}</p>
            <div v-if="profileView.comments.length === 0" class="rounded-lg  bg-[var(--bg)] p-3 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("comments.empty") }}</div>
            <div v-for="cm in profileView.comments" :key="cm.id" class="rounded-lg  bg-[var(--bg)] p-3">
              <div class="flex items-center gap-2">
                <span class="text-[13px] font-semibold text-[var(--accent)]">{{ cm.packName }}</span>
                <span class="text-xs text-[color:var(--tx-muted)]">{{ formatDate(cm.createdAt) }}</span>
                <button v-if="isAdmin" type="button" class="ml-auto text-xs font-medium text-[#f87171] hover:underline disabled:opacity-50" :disabled="adminBusy" @click="adminDeleteComment(cm.id)">
                  {{ t("author.delete") }}
                </button>
              </div>
              <p class="mt-1 text-[13px] leading-relaxed text-[color:var(--tx)] whitespace-pre-wrap">{{ cm.body }}</p>
            </div>
          </section>
        </div>
      </template>
    </div>
  </div>
</template>
