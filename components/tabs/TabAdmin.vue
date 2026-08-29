<script setup lang="ts">
import { useLauncherCtx } from '~/composables/useLauncherContext';

const {
  t,
  formatDate,
  monoProfile,
  adminUsers,
  adminPacks,
  adminBusy,
  loadAdminData,
  adminBanUser,
  adminUnbanUser,
  adminDeleteUser,
  adminSetRole,
  adminDeletePack,
  openProfileView,
} = useLauncherCtx();

const adminBanArmed = ref<string | null>(null);
const adminBanReason = ref('');
</script>

<template>
  <div class="min-h-0 flex-1 overflow-y-auto pr-1">
  <div class="space-y-6">
  <div class="flex items-start justify-between gap-4 border-b border-[var(--border)]  pb-3">
  <div>
  <h1 class="text-xl font-bold tracking-tight text-[color:var(--tx-strong)]">{{ t("admin.title") }}</h1>
  <p class="text-[13px] text-[color:var(--tx-muted)]">{{ t("admin.subtitle") }}</p>
  </div>
  <button
  type="button"
  class="shrink-0 rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
  :disabled="adminBusy"
  @click="loadAdminData"
  >
  {{ t("catalog.refresh") }}
  </button>
  </div>

  <!-- Пользователи -->
  <section class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <div class="border-b border-[var(--border)]  px-3.5 py-2.5 text-[13px] font-semibold text-[color:var(--tx-strong)]">
  {{ t("admin.users") }} ({{ adminUsers.length }})
  </div>
  <div class="divide-y divide-[var(--border)]">
  <div v-if="adminBusy && adminUsers.length === 0" class="px-4 py-6 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("catalog.loading") }}</div>
  <div v-for="u in adminUsers" :key="u.id" class="px-4 py-2.5">
  <div class="flex flex-wrap items-center gap-2">
  <button type="button" class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full  bg-[var(--input)] font-mono text-xs font-bold text-[color:var(--tx-strong)] transition-colors hover:text-[var(--accent)]"
  @click="openProfileView(u.id)">
  {{ u.username?.[0]?.toUpperCase() ?? "?" }}
  </button>
  <span class="font-mono text-[13px] font-medium text-[color:var(--tx-strong)]">{{ u.username }}</span>
  <span v-if="u.displayName" class="text-[13px] text-[color:var(--tx-muted)]">{{ u.displayName }}</span>
  <span class="rounded px-1.5 py-0.5 text-xs font-semibold"
  :class="u.role === 'admin' ? 'bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] text-[var(--accent)]' : 'bg-[var(--input)] text-[color:var(--tx-muted)]'">
  {{ u.role }}
  </span>
  <span v-if="u.banned" class="rounded bg-[#f85149]/15 px-1.5 py-0.5 text-xs font-semibold text-[#f85149]" :title="u.banReason || ''">
  {{ t("admin.banned") }}<template v-if="u.banReason">: {{ u.banReason }}</template>
  </span>
  <span v-if="u.email" class="min-w-0 flex-1 truncate text-right text-xs text-[color:var(--tx-muted)]">
  {{ u.email }} <template v-if="!u.emailConfirmed">⚠️</template>
  </span>
  <span class="ml-auto shrink-0 text-xs text-[color:var(--tx-muted)]">{{ formatDate(u.createdAt) }}</span>
  <select
  class="shrink-0 rounded-md  bg-[var(--bg)] px-1.5 py-1 text-xs text-[color:var(--tx)]  focus:outline-none disabled:opacity-50"
  :value="u.role"
  :disabled="adminBusy || u.id === monoProfile?.uuid"
  @change="adminSetRole(u.id, ($event.target as HTMLSelectElement).value)"
  >
  <option value="user">user</option>
  <option value="admin">admin</option>
  </select>
  <button v-if="!u.banned" type="button" class="shrink-0 rounded-md  bg-[var(--input)] px-2 py-1 text-[13px] font-medium text-[#f87171] hover:bg-[#b91c1c]/20 disabled:opacity-50"
  :disabled="adminBusy || u.id === monoProfile?.uuid"
  @click="adminBanArmed = adminBanArmed === u.id ? null : u.id; adminBanReason = ''">
  {{ t("admin.ban") }}
  </button>
  <button v-else type="button" class="shrink-0 rounded-md  bg-[var(--input)] px-2 py-1 text-[13px] font-medium text-[#3fb950] hover:bg-[#238636]/20 disabled:opacity-50"
  :disabled="adminBusy"
  @click="adminUnbanUser(u.id)">
  {{ t("admin.unban") }}
  </button>
  <button type="button" class="shrink-0 rounded-md  bg-[var(--input)] px-2 py-1 text-[13px] text-[#f87171] hover:bg-[#b91c1c]/20 disabled:opacity-50"
  :disabled="adminBusy || u.id === monoProfile?.uuid"
  @click="adminDeleteUser(u.id)">
  {{ t("author.delete") }}
  </button>
  </div>
  <!-- Причина бана (inline) -->
  <div v-if="adminBanArmed === u.id" class="mt-2 flex items-center gap-2 rounded-md  bg-[var(--bg)] p-2">
  <input v-model="adminBanReason" type="text" :placeholder="t('admin.banReasonPh')" class="min-w-0 flex-1 rounded-md  bg-[var(--bg)] px-2 py-1 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)]  focus:outline-none" />
  <button type="button" class="shrink-0 rounded-md  bg-[#b91c1c]/15 px-2.5 py-1.5 text-[13px] font-semibold text-[#f87171] hover:bg-[#b91c1c]/25 disabled:opacity-50"
  :disabled="adminBusy"
  @click="adminBanUser(u.id, adminBanReason); adminBanArmed = null">
  {{ t("admin.banConfirm") }}
  </button>
  <button type="button" class="shrink-0 rounded-md  bg-[var(--input)] px-2.5 py-1 text-[13px] text-[color:var(--tx)] hover:bg-[var(--hover)]" @click="adminBanArmed = null">
  {{ t("author.cancel") }}
  </button>
  </div>
  </div>
  </div>
  </section>

  <!-- Сборки -->
  <section class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <div class="border-b border-[var(--border)]  px-3.5 py-2.5 text-[13px] font-semibold text-[color:var(--tx-strong)]">
  {{ t("admin.packs") }} ({{ adminPacks.length }})
  </div>
  <div class="divide-y divide-[var(--border)]">
  <div v-if="adminBusy && adminPacks.length === 0" class="px-4 py-6 text-center text-[13px] text-[color:var(--tx-muted)]">{{ t("catalog.loading") }}</div>
  <div v-for="p in adminPacks" :key="p.id" class="flex flex-wrap items-center gap-2 px-4 py-2.5">
  <span class="min-w-0 flex-1 truncate text-[13px] font-medium text-[color:var(--tx-strong)]">{{ p.name }}</span>
  <span class="shrink-0 font-mono text-xs text-[color:var(--tx-muted)]">@{{ p.authorName ?? "?" }}</span>
  <span class="shrink-0 text-xs text-[color:var(--tx-muted)]">👍 {{ p.likes }} / 👎 {{ p.dislikes }} · {{ t("author.versions") }}: {{ p.versionsCount }}</span>
  <span class="shrink-0 text-xs text-[color:var(--tx-muted)]">{{ formatDate(p.createdAt) }}</span>
  <button type="button" class="shrink-0 rounded-md  bg-[var(--input)] px-2 py-1 text-[13px] text-[#f87171] hover:bg-[#b91c1c]/20 disabled:opacity-50"
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
