<script setup lang="ts">
import { useLauncherCtx } from '~/composables/useLauncherContext';
const ctx = useLauncherCtx();
const { t, status, packId, activePack, isAdmin, packUrl, packName, addingPack, packs, busy, handleAddPack, handleRemovePack, resetRemoveArm, removingPack, removeArmed, pickPackIcon, examplePackJson, deepLinkExample, openExampleInLauncher, copyInviteLink, openExamplePack } = ctx;
</script>

<template>
  <div class="min-h-0 flex-1 overflow-y-auto pr-1">
  <div class="space-y-6">
  <div class="border-b border-[var(--border)]  pb-3">
  <h1 class="text-xl font-bold tracking-tight text-[color:var(--tx-strong)]">{{ t("dev.title") }}</h1>
  <p class="text-[13px] text-[color:var(--tx-muted)]">{{ t("dev.subtitle") }}</p>
  </div>

  <!-- Добавление сборки -->
  <section class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <div class="flex items-center justify-between gap-2 border-b border-[var(--border)]  px-3.5 py-2.5">
  <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("dev.addTitle") }}</h3>
  <button
  type="button"
  class="flex items-center gap-1.5 rounded-md bg-[#238636] px-2.5 py-1.5 text-[13px] font-semibold text-white transition-colors hover:bg-[#2ea043]"
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
  <label class="mb-1 block text-[13px] text-[color:var(--tx-muted)]" for="pack-url">{{ t("dev.addUrl") }}</label>
  <input
  id="pack-url"
  v-model="packUrl"
  type="text"
  class="w-full rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] text-[color:var(--tx)] outline-none transition-colors placeholder:text-[color:var(--tx-muted)] "
  :placeholder="t('dev.addUrlPh')"
  @keydown.enter="handleAddPack"
  />
  </div>
  <div>
  <label class="mb-1 block text-[13px] text-[color:var(--tx-muted)]" for="pack-name">{{ t("dev.addName") }}</label>
  <input
  id="pack-name"
  v-model="packName"
  type="text"
  class="w-full rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] text-[color:var(--tx)] outline-none transition-colors placeholder:text-[color:var(--tx-muted)] "
  :placeholder="t('dev.addNamePh')"
  @keydown.enter="handleAddPack"
  />
  </div>
  <button
  type="button"
  class="rounded-md bg-[#238636] px-4 py-1.5 text-[13px] font-semibold text-white transition-colors hover:bg-[#2ea043] disabled:opacity-50"
  :disabled="addingPack || busy"
  @click="handleAddPack"
  >
  {{ addingPack ? t("dev.adding") : t("dev.addBtn") }}
  </button>
  </div>
  </section>

  <!-- Подключённые сборки -->
  <section class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <div class="border-b border-[var(--border)]  px-3.5 py-2.5">
  <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("dev.listTitle") }}</h3>
  </div>
  <div class="divide-y divide-[var(--border)]">
  <div v-if="packs.length === 0" class="p-4 text-[13px] text-[color:var(--tx-muted)]">
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
  <div class="truncate text-[13px] font-medium text-[color:var(--tx)]">
  {{ p.name }}
  <span v-if="p.author" class="font-mono text-xs text-[var(--accent)]">@{{ p.author }}</span>
  </div>
  <div class="truncate font-mono text-xs text-[color:var(--tx-muted)]">{{ p.id }}</div>
  </div>
  <span v-if="p.builtin" class="shrink-0 rounded  px-1.5 py-0.5 text-xs text-[color:var(--tx-muted)]" :title="t('dev.builtinNote')">
  {{ t("dev.builtin") }}
  </span>
  <button
  v-if="!p.builtin"
  type="button"
  class="shrink-0 rounded-md  bg-[var(--input)] px-2 py-1 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)]"
  :title="t('dev.setIconHint')"
  @click="pickPackIcon(p.id)"
  >
  {{ p.icon ? t("dev.changeIcon") : t("dev.setIcon") }}
  </button>
  <button
  v-else
  type="button"
  class="shrink-0 rounded-md  px-2 py-1 text-xs font-medium transition-colors disabled:opacity-50"
  :class="removeArmed === p.id
  ? 'bg-[#f85149]/15 text-[#f85149]'
  : ' bg-[var(--input)] text-[color:var(--tx)] hover:bg-[var(--hover)]'"
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
  <section class="rounded-xl  bg-[var(--panel)] shadow-sm overflow-hidden">
  <div class="border-b border-[var(--border)]  px-3.5 py-2.5">
  <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("dev.docsTitle") }}</h3>
  </div>
  <div class="space-y-2.5 p-4 text-[13px] leading-relaxed text-[color:var(--tx)]">
  <p>{{ t("dev.docsStep1") }}</p>
  <p>{{ t("dev.docsStep2") }}</p>
  <p>{{ t("dev.docsStep3") }}</p>
  <p>{{ t("dev.docsStep4") }}</p>
  <p>{{ t("dev.docsStep5") }}</p>
  <div class="mt-3 rounded-md  bg-[color-mix(in_srgb,var(--accent-deep)_10%,transparent)] p-3 text-[13px] text-[color:var(--tx)]">
  {{ t("dev.docsFormat") }}
  </div>
  <div class="rounded-md  bg-[var(--bg-60)] p-3">
  <p class="mb-1.5 font-mono text-xs text-[color:var(--tx-muted)]">pack.json</p>
  <pre class="overflow-x-auto text-xs leading-relaxed text-[color:var(--tx)]">{{ examplePackJson }}</pre>
  </div>
  <button
  type="button"
  class="flex items-center gap-1.5 rounded-md bg-[var(--accent-deep)] px-2.5 py-1.5 text-[13px] font-semibold text-white transition-colors hover:bg-[var(--accent-hover)]"
  @click="openExamplePack"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current">
  <path d="M8 1.5a.75.75 0 0 1 .75.75V9.44l1.72-1.72a.75.75 0 1 1 1.06 1.06l-3 3a.75.75 0 0 1-1.06 0l-3-3a.75.75 0 1 1 1.06-1.06l1.72 1.72V2.25A.75.75 0 0 1 8 1.5ZM4 12.25a.75.75 0 0 1 .75.75v.5a.5.5 0 0 0 .5.5h5.5a.5.5 0 0 0 .5-.5v-.5a.75.75 0 0 1 1.5 0v.5a2 2 0 0 1-2 2h-5.5a2 2 0 0 1-2-2v-.5a.75.75 0 0 1 .75-.75Z"/>
  </svg>
  {{ t("dev.docsExample") }}
  </button>
  <p class="text-[13px] text-[color:var(--tx-muted)]">{{ t("dev.docsNews") }}</p>
  <p class="text-[13px] text-[color:var(--tx-muted)]">{{ t("dev.docsBugs") }}</p>
  <p class="text-[13px] text-[color:var(--tx-muted)]">{{ t("dev.docsContent") }}</p>
  <div class="space-y-2 font-mono text-xs text-[color:var(--tx-muted)]">
  <div class="overflow-x-auto rounded-md  bg-[var(--bg-60)] px-3 py-2">
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
  <div class="overflow-x-auto rounded-md  bg-[var(--bg-60)] px-3 py-2">
  <div class="mb-1 font-semibold text-[color:var(--tx-strong)]">socials.json</div>
  <pre class="leading-relaxed">{
  "discord": "https://discord.gg/example",
  "telegram": "https://t.me/example",
  "vk": "https://vk.com/example"
}</pre>
  </div>
  <div class="overflow-x-auto rounded-md  bg-[var(--bg-60)] px-3 py-2">
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
  <div class="rounded-md  bg-[#238636]/10 p-3 text-[13px] text-[color:var(--tx)]">
  <p class="mb-2 font-semibold text-[#3fb950]">mono://</p>
  <p class="mb-2">{{ t("dev.docsDeep") }}</p>
  <code class="block overflow-x-auto rounded bg-[var(--bg-60)] px-2 py-1.5 font-mono text-xs text-[color:var(--tx-strong)]">{{ deepLinkExample }}</code>
  <div class="mt-2.5 flex flex-wrap gap-2">
  <button
  type="button"
  class="flex items-center gap-1.5 rounded-md bg-[#238636] px-2.5 py-1.5 text-[13px] font-semibold text-white transition-colors hover:bg-[#2ea043]"
  @click="openExampleInLauncher"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current">
  <path d="M10.78 5.22a.75.75 0 0 1 0 1.06L8.56 8.5H11.5a.75.75 0 0 1 0 1.5H8.56l2.22 2.22a.75.75 0 1 1-1.06 1.06l-3.5-3.5a.75.75 0 0 1 0-1.06l3.5-3.5a.75.75 0 0 1 1.06 0ZM3.75 4A1.75 1.75 0 0 0 2 5.75v4.5c0 .966.784 1.75 1.75 1.75h3a.75.75 0 0 0 0-1.5h-3a.25.25 0 0 1-.25-.25v-4.5a.25.25 0 0 1 .25-.25h3a.75.75 0 0 0 0-1.5h-3Z"/>
  </svg>
  {{ t("dev.docsOpenExample") }}
  </button>
  <button
  v-if="activePack"
  type="button"
  class="flex items-center gap-1.5 rounded-md  bg-[#238636]/10 px-2.5 py-1.5 text-[13px] font-semibold text-[#3fb950] transition-colors hover:bg-[#238636]/20"
  @click="copyInviteLink"
  >
  <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current">
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
