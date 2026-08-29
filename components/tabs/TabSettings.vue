<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useLauncherCtx } from "~/composables/useLauncherContext";
import {
  isTauri,
  setCloseToTray,
  autostartSet,
  autostartGet,
  getUserJvmArgs,
  setUserJvmArgs,
} from "~/lib/bridge";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { sanitizeSvg } from "~/lib/misc";

const ctx = useLauncherCtx();
const {
  t,
  locale,
  locales,
  setLocale,
  getLocaleMeta,
  javaArchLabel,
  localeLabel,
  monoProfile,
  monoName,
  monoPass,
  monoBusy,
  monoAuthBusy,
  handleMonoLogin,
  handleMonoRegister,
  handleMonoLogout,
  handleMonoConfirmEmail,
  monoForgotOpen,
  monoForgotEmail,
  monoForgotSent,
  handleMonoForgot,
  monoResetToken,
  monoResetPass,
  monoResetDone,
  handleMonoReset,
  busy,
  profileBusy,
  openProfileView,
  username,
  handleOffline,
  msPolling,
  elyPolling,
  handleMicrosoft,
  handleEly,
  deviceFlow,
  msFlow,
  openMsAuthPage,
  accounts,
  accountBusy,
  handleSwitchAccount,
  handleRemoveAccount,
  boostyGlobalLinkedState,
  boostyAuthOpen,
  boostyGlobalOpen,
  startBoostyGlobalLogin,
  cancelBoostyLogin,
  unlinkBoostyGlobal,
  licenseKeyInput,
  licenseBusy,
  saveBoostyGlobal,
  paidPacks,
  licenseByPack,
  licenseBusyFor,
  startBoostyLogin,
  saveLicenseFor,
  removeLicenseFor,
  formatUnixDate,
  boostyTargetPack,
  localSkin,
  skinModel,
  skinBusy,
  skinApi,
  applyLocalSkin,
  removeLocalSkin,
  session,
  notify,
  themeLevel,
  packThemeActive,
  setThemeLevel,
  toggleTheme,
  ram,
  maxRam,
  systemRam,
  activePack,
  windowWidth,
  windowHeight,
  javaList,
  javaSelected,
  javaBusy,
  javaMsg,
  downloadJava,
  selectJava,
  discordRp,
  toggleDiscordRp,
  warnCustomMods,
  toggleWarnCustomMods,
  verifyBusy,
  verifyResult,
  handleVerify,
} = ctx;

const activeLocaleAuthor = computed(() => getLocaleMeta(locale.value).author ?? "");
const activeLocaleVersion = computed(() => getLocaleMeta(locale.value).version ?? "");

// --- Settings sub-tabs ---

const settingsTab = ref<"accounts" | "appearance">("accounts");

const SETTINGS_TAB_ICONS: Record<"accounts" | "appearance", string> = {
  accounts:
    '<path d="M8 1a3 3 0 1 0 0 6 3 3 0 0 0 0-6ZM2 13.25C2 10.75 4.46 9.25 8 9.25s6 1.5 6 4V14a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1v-.75Z"/>',
  appearance:
    '<path fill-rule="evenodd" d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1Zm0 1.5v11a5.5 5.5 0 0 1 0-11Z"/>',
};

// --- System: tray + autostart ---

const closeToTray = ref(false);
const autostartOn = ref(false);

async function toggleCloseToTray(on: boolean) {
  closeToTray.value = on;
  localStorage.setItem("mono.closeToTray", on ? "1" : "0");
  if (!isTauri()) return;
  try {
    await setCloseToTray(on);
  } catch (e) {
    notify(String(e));
  }
}

async function toggleAutostart(on: boolean) {
  autostartOn.value = on;
  if (!isTauri()) return;
  try {
    await autostartSet(on);
  } catch (e) {
    notify(String(e));
    autostartOn.value = !on;
  }
}

(async () => {
  closeToTray.value = localStorage.getItem("mono.closeToTray") === "1";
  if (isTauri()) {
    try {
      await setCloseToTray(closeToTray.value);
      autostartOn.value = await autostartGet();
    } catch {
      // плагин недоступен
    }
  }
})();

// --- JVM arguments ---

const jvmArgs = ref("");
const jvmArgsSaving = ref(false);

async function loadJvmArgs() {
  if (!isTauri()) return;
  try {
    jvmArgs.value = await getUserJvmArgs();
  } catch {
    /* ignore */
  }
}

async function saveJvmArgs() {
  if (!isTauri() || jvmArgsSaving.value) return;
  jvmArgsSaving.value = true;
  try {
    await setUserJvmArgs(jvmArgs.value.trim());
    notify(t("settings.jvmArgsSaved"), "success");
  } catch (e) {
    notify(t("files.updateErr", { e }), "error");
  } finally {
    jvmArgsSaving.value = false;
  }
}

onMounted(() => {
  void loadJvmArgs();
});

// --- Java change ---

function onJavaChange(e: Event) {
  const val = (e.target as HTMLSelectElement).value;
  selectJava(val);
}

// --- Skin helpers ---

async function pickImage(): Promise<string | null> {
  if (!isTauri()) {
    notify(t("skin.tauriOnly"), "info");
    return null;
  }
  try {
    const p = await openDialog({
      multiple: false,
      filters: [{ name: "Изображение", extensions: ["png", "jpg", "jpeg", "webp"] }],
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

async function copySkinApi() {
  try {
    await navigator.clipboard.writeText(skinApi.value);
    notify(t("skin.copied"), "success");
  } catch {
    notify(t("servers.copyFail"), "error");
  }
}
</script>

<template>
  <div class="min-h-0 flex-1 overflow-y-auto pr-1">
    <div class="space-y-6">
      <div class="border-b border-[var(--border)] pb-3">
        <h1 class="text-xl font-bold tracking-tight text-[color:var(--tx-strong)]">{{ t("settings.title") }}</h1>
        <p class="text-[13px] text-[color:var(--tx-muted)]">{{ t("settings.subtitle") }}</p>
        <div class="mt-3 flex gap-1">
          <button
            v-for="st in ([['accounts', t('settings.tabAccounts')], ['appearance', t('settings.tabAppearance')]] as const)"
            :key="st[0]"
            type="button"
            class="relative inline-flex items-center gap-1.5 px-3 pb-2 pt-1 text-[13px] font-semibold transition-colors"
            :class="settingsTab === st[0] ? 'text-[var(--accent)]' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx-strong)]'"
            @click="settingsTab = st[0]"
          >
            <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current" v-html="SETTINGS_TAB_ICONS[st[0]]"></svg>
            {{ st[1] }}
            <span v-if="settingsTab === st[0]" class="absolute inset-x-2 bottom-0 h-[2.5px] rounded-t-full bg-[var(--accent)]"></span>
          </button>
        </div>
      </div>

      <template v-if="settingsTab === 'accounts'">
        <div class="space-y-4">
          <!-- Аккаунты: профиль Mono + игровые аккаунты (две колонки) -->
          <div class="grid gap-4 lg:grid-cols-2">
            <!-- Профиль Mono -->
            <section class="rounded-xl bg-[var(--panel)] shadow-sm overflow-hidden">
              <div class="border-b border-[var(--border)] px-3.5 py-2.5 flex items-center gap-2.5">
                <span class="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg bg-[var(--accent)]">
                  <svg viewBox="0 0 24 24" class="h-4 w-4 fill-[color:var(--panel)]"><path d="M3 8.4 8.4 3h7.2L21 8.4v7.2L15.6 21H8.4L3 15.6V8.4Zm2 1.3v4.6L8.3 19H9.7l2.5-6.2L14.7 19h1.4L19 14.3V9.7L15.7 5H9.9L5 9.7Z"/></svg>
                </span>
                <div class="min-w-0">
                  <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.monoTitle") }}</h3>
                  <p class="text-xs leading-tight text-[color:var(--tx-muted)]">{{ t("settings.monoNote") }}</p>
                </div>
              </div>

              <div class="p-4 space-y-3">
                <template v-if="monoProfile">
                  <div class="flex items-center gap-3">
                    <span class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-[var(--input)] font-mono text-sm font-bold text-[var(--accent)]">
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
                      class="flex-1 rounded-lg bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                      :disabled="profileBusy"
                      @click="openProfileView(monoProfile.uuid)"
                    >
                      {{ t("profile.my") }}
                    </button>
                    <button
                      type="button"
                      class="flex-1 rounded-lg bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                      :disabled="monoAuthBusy"
                      :title="t('auth2.confirmHint')"
                      @click="handleMonoConfirmEmail"
                    >
                      {{ t("auth2.confirm") }}
                    </button>
                  </div>
                  <button
                    type="button"
                    class="w-full rounded-lg bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] py-2 text-[13px] font-semibold text-[var(--accent)] hover:bg-[color-mix(in_srgb,var(--accent)_25%,transparent)] disabled:opacity-50"
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
                    class="w-full rounded-lg bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none"
                  />
                  <input
                    v-model="monoPass"
                    type="password"
                    :placeholder="t('settings.monoPassword')"
                    @keydown.enter="handleMonoLogin"
                    class="w-full rounded-lg bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none"
                  />
                  <div class="flex gap-2">
                    <button
                      type="button"
                      class="flex-1 rounded-lg bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] py-2 text-[13px] font-semibold text-[var(--accent)] hover:bg-[color-mix(in_srgb,var(--accent)_25%,transparent)] disabled:opacity-50"
                      :disabled="busy || monoBusy"
                      @click="handleMonoLogin"
                    >
                      {{ monoBusy ? t("settings.monoWait") : t("settings.monoSignIn") }}
                    </button>
                    <button
                      type="button"
                      class="flex-1 rounded-lg bg-[var(--input)] py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                      :disabled="busy || monoBusy"
                      @click="handleMonoRegister"
                    >
                      {{ t("settings.monoRegister") }}
                    </button>
                  </div>
                  <button
                    type="button"
                    class="w-full text-center text-xs font-medium text-[var(--accent)] hover:underline"
                    @click="monoForgotOpen = !monoForgotOpen"
                  >
                    {{ t("auth2.forgot") }}
                  </button>

                  <!-- Восстановление пароля: письмо + сброс по токену из письма -->
                  <div v-if="monoForgotOpen" class="space-y-2 rounded-lg bg-[var(--bg)] p-3">
                    <div class="flex items-center gap-2">
                      <input
                        v-model="monoForgotEmail"
                        type="email"
                        :placeholder="t('auth2.emailPh')"
                        class="min-w-0 flex-1 rounded-lg bg-[var(--bg)] px-2.5 py-1.5 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none"
                        @keydown.enter="handleMonoForgot"
                      />
                      <button
                        type="button"
                        class="shrink-0 rounded-lg bg-[var(--input)] px-2.5 py-1.5 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                        :disabled="monoAuthBusy"
                        @click="handleMonoForgot"
                      >
                        {{ t("auth2.send") }}
                      </button>
                    </div>
                    <p v-if="monoForgotSent" class="text-xs leading-snug text-[#3fb950]">{{ t("auth2.forgotSent") }}</p>
                    <div class="space-y-1.5 border-t border-[var(--border)] pt-2">
                      <p class="text-xs text-[color:var(--tx-muted)]">{{ t("auth2.resetHint") }}</p>
                      <input
                        v-model="monoResetToken"
                        type="text"
                        :placeholder="t('auth2.tokenPh')"
                        class="w-full rounded-lg bg-[var(--bg)] px-2.5 py-1.5 font-mono text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none"
                      />
                      <input
                        v-model="monoResetPass"
                        type="password"
                        :placeholder="t('auth2.newPassPh')"
                        class="w-full rounded-lg bg-[var(--bg)] px-2.5 py-1.5 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none"
                        @keydown.enter="handleMonoReset"
                      />
                      <button
                        type="button"
                        class="w-full rounded-lg bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] py-1.5 text-[13px] font-semibold text-[var(--accent)] hover:bg-[color-mix(in_srgb,var(--accent)_25%,transparent)] disabled:opacity-50"
                        :disabled="monoAuthBusy"
                        @click="handleMonoReset"
                      >
                        {{ t("auth2.resetBtn") }}
                      </button>
                      <p v-if="monoResetDone" class="text-xs leading-snug text-[#3fb950]">{{ t("auth2.resetDone") }}</p>
                    </div>
                  </div>
                </template>
              </div>
            </section>

            <!-- Игровые аккаунты -->
            <section class="rounded-xl bg-[var(--panel)] shadow-sm overflow-hidden">
              <div class="border-b border-[var(--border)] px-3.5 py-2.5">
                <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.gameAccounts") }}</h3>
              </div>
              <div class="p-4 space-y-3">
                <div class="flex gap-2">
                  <input
                    v-model="username"
                    :placeholder="t('settings.nickname')"
                    class="flex-1 rounded-lg bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none"
                  />
                  <button
                    type="button"
                    class="rounded-lg bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                    :disabled="busy"
                    @click="handleOffline"
                  >
                    {{ t("settings.save") }}
                  </button>
                </div>

                <div class="grid grid-cols-2 gap-2">
                  <button
                    type="button"
                    class="rounded-lg bg-[var(--input)] py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                    :disabled="busy || msPolling || elyPolling"
                    @click="handleMicrosoft"
                  >
                    {{ msPolling ? t("settings.msWait") : t("settings.msSignin") }}
                  </button>
                  <button
                    type="button"
                    class="rounded-lg bg-[color-mix(in_srgb,var(--accent)_8%,transparent)] py-2 text-[13px] font-medium text-[var(--accent)] hover:bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] disabled:opacity-50"
                    :disabled="busy || msPolling || elyPolling"
                    @click="handleEly"
                  >
                    {{ elyPolling ? t("settings.elyWait") : t("settings.elySignin") }}
                  </button>
                </div>

                <!-- Device code flow: показать код и ссылку -->
                <div
                  v-if="deviceFlow"
                  class="rounded-md bg-[var(--bg-60)] p-3 space-y-2"
                >
                  <p class="text-[13px] text-[color:var(--tx-muted)]">
                    {{ msFlow ? t("settings.msCode") : t("settings.elyCode") }}
                  </p>
                  <div class="flex items-center gap-3">
                    <div
                      v-if="deviceFlow.qr_svg"
                      class="h-28 w-28 shrink-0 overflow-hidden rounded-md bg-white"
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
                        class="mt-2 rounded-md bg-[color-mix(in_srgb,var(--accent-deep)_20%,transparent)] px-2.5 py-1.5 text-[13px] font-semibold text-white transition-colors hover:bg-[color-mix(in_srgb,var(--accent-deep)_40%,transparent)]"
                        @click="openMsAuthPage"
                      >
                        {{ t("settings.msOpen", { uri: deviceFlow.verification_uri.replace(/^https?:\/\//, "") }) }}
                      </button>
                    </div>
                  </div>
                  <p v-if="msPolling || elyPolling" class="flex items-center gap-2 text-[13px] text-[color:var(--tx-muted)]">
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
                    class="flex items-center gap-2 rounded-md bg-[var(--bg)] px-3 py-2"
                  >
                    <div
                      class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-[var(--input)] font-mono text-[13px] font-bold text-[color:var(--tx-strong)]"
                    >
                      {{ a.username[0]?.toUpperCase() ?? "?" }}
                    </div>
                    <div class="min-w-0 flex-1">
                      <p class="truncate text-[13px] font-medium text-[color:var(--tx-strong)]">{{ a.username }}</p>
                      <p class="text-xs text-[color:var(--tx-muted)]">
                        {{ a.user_type === "microsoft" ? t("accounts.ms") : a.user_type === "ely" ? t("accounts.ely") : t("accounts.offline") }}
                      </p>
                    </div>
                    <button
                      v-if="a.id !== accounts.active"
                      type="button"
                      class="shrink-0 rounded-md bg-[var(--input)] px-2 py-1 text-xs font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] disabled:opacity-50"
                      :disabled="accountBusy"
                      @click="handleSwitchAccount(a.id)"
                    >
                      {{ t("accounts.use") }}
                    </button>
                    <span
                      v-else
                      class="shrink-0 text-xs font-semibold text-[#3fb950]"
                    >
                      {{ t("accounts.active") }}
                    </span>
                    <button
                      type="button"
                      class="shrink-0 rounded-md bg-[#f85149]/10 p-1 text-[#f85149] transition-colors hover:bg-[#f85149]/20 disabled:opacity-50"
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
          <section class="rounded-xl bg-[var(--panel)] shadow-sm overflow-hidden">
            <div class="border-b border-[var(--border)] px-3.5 py-2.5 flex items-center gap-2">
              <svg viewBox="0 0 16 16" class="h-4 w-4 shrink-0 fill-[var(--accent)]" preserveAspectRatio="none">
                <path d="M8 1C3.9 1 .7 4.3.7 8.4h3.1L1.6 15l7.2-7.2H6.3C6.3 5.3 7.2 2.9 9.6 2.4 11.9 2 13.7 3.6 13.7 5.8c0 .4-.1.9-.1 1.3.9.5 1.5 1.4 1.7 2.5.1-.6.2-1.2.2-1.8 0-3.8-3.2-6.8-7.5-6.8Z" transform="translate(0 -1)"/>
              </svg>
              <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.boosty") }}</h3>
            </div>
            <div class="space-y-3 p-4">
              <p class="text-[13px] leading-snug text-[color:var(--tx-muted)]">{{ t("settings.boostyNote") }}</p>
              <!-- Глобальный аккаунт Boosty: работает даже без платных сборок -->
              <div class="rounded-md bg-[var(--bg)] px-3 py-2">
                <div class="flex items-center gap-2">
                  <div class="min-w-0 flex-1">
                    <p class="truncate text-[13px] font-medium text-[color:var(--tx-strong)]">{{ t("settings.boostyGlobal") }}</p>
                    <p class="truncate text-xs text-[color:var(--tx-muted)]">{{ t("settings.boostyGlobalNote") }}</p>
                  </div>
                  <span
                    v-if="boostyGlobalLinkedState"
                    class="shrink-0 rounded-full bg-[#3fb950]/10 px-2 py-0.5 text-xs font-semibold text-[#3fb950]"
                  >
                    {{ t("settings.boostyOk") }}
                  </span>
                  <span
                    v-else
                    class="shrink-0 rounded-full bg-[var(--input)] px-2 py-0.5 text-xs font-semibold text-[color:var(--tx-muted)]"
                  >
                    {{ t("settings.boostyNo") }}
                  </span>
                </div>
                <template v-if="boostyGlobalLinkedState">
                  <button
                    type="button"
                    class="mt-1.5 w-full rounded-md bg-[#f85149]/10 py-1 text-xs font-medium text-[#f85149] transition-colors hover:bg-[#f85149]/20 disabled:opacity-50"
                    :disabled="licenseBusy"
                    @click="unlinkBoostyGlobal"
                  >
                    {{ t("accounts.signOut") }}
                  </button>
                </template>
                <div v-else class="mt-2 space-y-1.5">
                  <button
                    type="button"
                    class="flex w-full items-center justify-center gap-1.5 rounded-md bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] py-1.5 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_22%,transparent)] disabled:opacity-50"
                    :disabled="licenseBusy || boostyAuthOpen"
                    @click="startBoostyGlobalLogin"
                  >
                    <svg v-if="boostyAuthOpen && boostyGlobalOpen" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
                    {{ boostyAuthOpen && boostyGlobalOpen ? t("license.waiting") : t("license.oauth") }}
                  </button>
                  <div v-if="boostyAuthOpen && boostyGlobalOpen" class="flex justify-center">
                    <button
                      type="button"
                      class="text-[13px] font-medium text-[color:var(--tx-muted)] hover:text-[color:var(--tx)]"
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
                      class="min-w-0 flex-1 rounded-md bg-[var(--bg)] px-2 py-1.5 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none"
                      @keydown.enter="saveBoostyGlobal(licenseKeyInput)"
                    />
                    <button
                      type="button"
                      class="shrink-0 rounded-md bg-[var(--input)] px-2 py-1.5 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
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
                class="rounded-md bg-[var(--bg)] px-3 py-2"
              >
                <div class="flex items-center gap-2">
                  <div class="min-w-0 flex-1">
                    <p class="truncate text-[13px] font-medium text-[color:var(--tx-strong)]">{{ p.name }}</p>
                    <p class="truncate text-xs text-[color:var(--tx-muted)]">
                      boosty.to/{{ p.boostyBlog }}
                    </p>
                  </div>
                  <span
                    v-if="licenseByPack[p.id]?.subscribed"
                    class="shrink-0 rounded-full bg-[#3fb950]/10 px-2 py-0.5 text-xs font-semibold text-[#3fb950]"
                  >
                    {{ t("settings.boostyOk") }}
                  </span>
                  <span
                    v-else-if="licenseByPack[p.id] && !licenseByPack[p.id]?.subscribed"
                    class="shrink-0 rounded-full bg-[var(--input)] px-2 py-0.5 text-xs font-semibold text-[color:var(--tx-muted)]"
                  >
                    {{ t("settings.boostyNo") }}
                  </span>
                </div>
                <template v-if="licenseByPack[p.id]?.subscribed">
                  <p v-if="licenseByPack[p.id]?.tier" class="mt-1 truncate text-xs text-[color:var(--tx-muted)]">
                    {{ t("license.tierList", { list: licenseByPack[p.id]?.tier ?? "" }) }}
                  </p>
                  <p v-if="licenseByPack[p.id]?.expiresAt" class="mt-0.5 text-xs text-[color:var(--tx-muted)]">
                    {{ t("license.active", { blog: p.boostyBlog ?? "", until: formatUnixDate(licenseByPack[p.id]?.expiresAt ?? 0) }) }}
                  </p>
                  <button
                    type="button"
                    class="mt-1.5 w-full rounded-md bg-[#f85149]/10 py-1 text-xs font-medium text-[#f85149] transition-colors hover:bg-[#f85149]/20 disabled:opacity-50"
                    :disabled="licenseBusyFor === p.id"
                    @click="removeLicenseFor(p.id)"
                  >
                    {{ t("accounts.signOut") }}
                  </button>
                </template>
                <div v-else class="mt-2 space-y-1.5">
                  <button
                    type="button"
                    class="flex w-full items-center justify-center gap-1.5 rounded-md bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] py-1.5 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_22%,transparent)] disabled:opacity-50"
                    :disabled="licenseBusyFor === p.id || boostyAuthOpen"
                    @click="startBoostyLogin(p.id)"
                  >
                    <svg v-if="boostyAuthOpen && boostyTargetPack === p.id" viewBox="0 0 16 16" class="h-3 w-3 animate-spin fill-current"><path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/></svg>
                    {{ boostyAuthOpen && boostyTargetPack === p.id ? t("license.waiting") : t("license.oauth") }}
                  </button>
                  <div v-if="boostyAuthOpen && boostyTargetPack === p.id" class="flex justify-center">
                    <button
                      type="button"
                      class="text-[13px] font-medium text-[color:var(--tx-muted)] hover:text-[color:var(--tx)]"
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
                      class="min-w-0 flex-1 rounded-md bg-[var(--bg)] px-2 py-1.5 text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none"
                      @keydown.enter="saveLicenseFor(p.id, licenseKeyInput)"
                    />
                    <button
                      type="button"
                      class="shrink-0 rounded-md bg-[var(--input)] px-2 py-1.5 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
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
          <section class="rounded-xl bg-[var(--panel)] shadow-sm overflow-hidden">
            <div class="border-b border-[var(--border)] px-3.5 py-2.5">
              <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("skin.title") }}</h3>
            </div>
            <div class="p-4 space-y-3">
              <div class="flex items-center gap-3">
                <div class="flex h-14 w-14 shrink-0 items-center justify-center overflow-hidden rounded-md bg-[var(--input)] font-mono text-sm font-bold text-[color:var(--tx-strong)]">
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
                    class="w-full appearance-none rounded-md bg-[var(--input)] px-2.5 py-1.5 pr-8 text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] focus:outline-none"
                    :disabled="skinBusy"
                  >
                    <option value="classic">{{ t("skin.modelClassic") }}</option>
                    <option value="slim">{{ t("skin.modelSlim") }}</option>
                  </select>
                  <p class="text-xs leading-relaxed text-[color:var(--tx-muted)]">
                    {{ t("skin.note") }}
                  </p>
                </div>
              </div>
              <div class="flex gap-2">
                <button
                  type="button"
                  class="flex-1 rounded-md bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                  :disabled="skinBusy"
                  @click="pickSkinFile"
                >
                  {{ skinBusy ? t("skin.busy") : t("skin.pick") }}
                </button>
                <button
                  v-if="localSkin?.has_skin"
                  type="button"
                  class="rounded-md bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                  :disabled="skinBusy"
                  @click="removeLocalSkin"
                >
                  {{ t("skin.remove") }}
                </button>
              </div>
              <div class="rounded-md bg-[var(--bg-60)] p-3 space-y-1.5">
                <p class="text-[13px] text-[color:var(--tx-muted)]">{{ t("skin.apiHint") }}</p>
                <div class="flex items-center gap-2">
                  <code class="min-w-0 flex-1 truncate rounded bg-[var(--input)] px-2 py-1 font-mono text-xs text-[color:var(--tx)] select-all">{{ skinApi || "…" }}</code>
                  <button
                    type="button"
                    class="rounded-md bg-[var(--input)] px-2 py-1 text-xs font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                    :disabled="!skinApi"
                    @click="copySkinApi"
                  >
                    {{ t("skin.copy") }}
                  </button>
                </div>
              </div>
            </div>
          </section>
        </div>
      </template>

      <template v-else>
        <div class="space-y-4">
          <!-- Тема -->
          <section class="rounded-xl bg-[var(--panel)] shadow-sm overflow-hidden">
            <div class="flex items-center justify-between border-b border-[var(--border)] px-3.5 py-2.5">
              <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.theme") }}</h3>
              <span class="text-xs font-medium text-[color:var(--tx-muted)]">
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
                class="w-full rounded-md bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] hover:bg-[var(--hover)] disabled:opacity-50"
                :disabled="packThemeActive"
                @click="toggleTheme"
              >
                {{ t("settings.themeToggle") }}
              </button>
              <p v-if="packThemeActive" class="text-[13px] text-[var(--accent)]">
                {{ t("theme.disabled") }}
              </p>
            </div>
          </section>

          <!-- Язык интерфейса -->
          <section class="rounded-xl bg-[var(--panel)] shadow-sm overflow-hidden">
            <div class="border-b border-[var(--border)] px-3.5 py-2.5">
              <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.language") }}</h3>
            </div>
            <div class="p-4 space-y-3">
              <select
                :value="locale"
                class="w-full appearance-none rounded-md bg-[var(--input)] px-2.5 py-1.5 pr-8 text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] focus:outline-none"
                @change="setLocale(($event.target as HTMLSelectElement).value)"
              >
                <option v-for="l in locales" :key="l" :value="l">{{ localeLabel(l) }}</option>
              </select>
              <p class="flex items-center gap-1 text-[13px] text-[color:var(--tx-muted)]">
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
      </template>

      <!-- System settings (always visible below sub-tabs) -->
      <div class="space-y-4">
        <!-- ОЗУ -->
        <section class="rounded-xl bg-[var(--panel)] shadow-sm overflow-hidden">
          <div class="border-b border-[var(--border)] px-3.5 py-2.5 flex justify-between items-center">
            <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.ram") }}</h3>
            <span class="font-mono text-[13px] font-semibold text-[var(--accent)]">{{ ram }} {{ t("units.gb") }}</span>
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
            <div class="flex justify-between text-[13px] text-[color:var(--tx-muted)] font-mono">
              <span>2 {{ t("units.gb") }}</span>
              <span>{{ t("settings.ramMax", { n: maxRam }) }}</span>
            </div>
            <p v-if="systemRam && systemRam.total_ram_gb > 0" class="text-[13px] text-[color:var(--tx-muted)]">
              {{ t("settings.ramTotal", { total: systemRam.total_ram_gb, avail: systemRam.available_ram_gb }) }}
            </p>
            <p
              v-if="activePack?.minRam"
              class="text-[13px]"
              :class="(ram * 1024) < activePack.minRam ? 'font-medium text-[#f0883e]' : 'text-[color:var(--tx-muted)]'"
            >
              {{ t("settings.ramMin", { name: activePack.name, min: activePack.minRam / 1024, gb: ram }) }}
            </p>
          </div>
        </section>

        <!-- JVM-аргументы -->
        <section class="rounded-xl bg-[var(--panel)] shadow-sm overflow-hidden">
          <div class="border-b border-[var(--border)] px-3.5 py-2.5 flex justify-between items-center">
            <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.jvmArgs") }}</h3>
            <button
              type="button"
              class="text-[13px] underline decoration-dotted underline-offset-2 disabled:opacity-50"
              :disabled="jvmArgsSaving"
              @click="saveJvmArgs"
            >
              {{ jvmArgsSaving ? t("common.saving") : t("common.save") }}
            </button>
          </div>
          <div class="p-4 space-y-2">
            <textarea
              v-model="jvmArgs"
              rows="3"
              spellcheck="false"
              class="w-full rounded-md bg-[var(--input)] border border-[var(--border)] px-3 py-2 font-mono text-[13px] text-[color:var(--tx)] focus:outline-none focus:border-[var(--accent)]"
              :placeholder="t('settings.jvmArgsHint')"
            ></textarea>
            <p class="text-[13px] text-[color:var(--tx-muted)]">{{ t("settings.jvmArgsNote") }}</p>
          </div>
        </section>

        <!-- Размер окна игры -->
        <section class="rounded-xl bg-[var(--panel)] shadow-sm overflow-hidden">
          <div class="border-b border-[var(--border)] px-3.5 py-2.5 flex justify-between items-center">
            <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.win") }}</h3>
            <span class="font-mono text-[13px] font-semibold text-[var(--accent)]">{{ windowWidth }}×{{ windowHeight }}</span>
          </div>
          <div class="p-4 space-y-2">
            <div class="flex items-center gap-3">
              <label class="w-16 text-[13px] text-[color:var(--tx-muted)]" for="ts-win-width">{{ t("settings.width") }}</label>
              <input
                id="ts-win-width"
                type="number"
                min="320"
                max="7680"
                step="1"
                v-model.number="windowWidth"
                class="flex-1 rounded-md bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)] focus:outline-none"
              />
              <label class="w-16 text-[13px] text-[color:var(--tx-muted)]" for="ts-win-height">{{ t("settings.height") }}</label>
              <input
                id="ts-win-height"
                type="number"
                min="240"
                max="4320"
                step="1"
                v-model.number="windowHeight"
                class="flex-1 rounded-md bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)] focus:outline-none"
              />
            </div>
            <p class="text-[13px] text-[color:var(--tx-muted)]">
              {{ t("settings.winNote") }}
            </p>
          </div>
        </section>

        <!-- Java -->
        <section class="rounded-xl bg-[var(--panel)] shadow-sm overflow-hidden">
          <div class="border-b border-[var(--border)] px-3.5 py-2.5">
            <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.java") }}</h3>
          </div>
          <div class="p-4 space-y-3">
            <div class="flex items-center gap-2">
              <select
                :value="javaSelected"
                class="flex-1 appearance-none rounded-md bg-[var(--input)] px-2.5 py-1.5 pr-8 text-[13px] text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] focus:outline-none"
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
                class="shrink-0 rounded-md bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] disabled:opacity-50"
                :disabled="javaBusy || busy"
                @click="downloadJava"
              >
                {{ javaBusy ? t("settings.javaDownloading") : t("settings.javaDownload") }}
              </button>
            </div>
            <p v-if="javaMsg" class="text-[13px] text-[color:var(--tx-muted)] break-all">{{ javaMsg }}</p>
            <p class="text-[13px] text-[color:var(--tx-muted)]">
              {{ t("settings.javaNote") }}
            </p>
          </div>
        </section>

        <!-- Discord Rich Presence -->
        <section class="rounded-xl bg-[var(--panel)] shadow-sm overflow-hidden">
          <div class="border-b border-[var(--border)] px-3.5 py-2.5">
            <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.discord") }}</h3>
          </div>
          <div class="p-4">
            <label class="flex cursor-pointer items-center gap-3">
              <input
                type="checkbox"
                class="h-4 w-4 accent-[#5865F2]"
                :checked="discordRp"
                @change="toggleDiscordRp(($event.target as HTMLInputElement).checked)"
              />
              <span class="text-[13px] text-[color:var(--tx)]">{{ t("settings.discordLabel") }}</span>
            </label>
            <p class="mt-2 text-[13px] text-[color:var(--tx-muted)]">
              {{ t("settings.discordNote") }}
            </p>
          </div>
        </section>

        <!-- Система: трей + автозапуск -->
        <section class="rounded-xl bg-[var(--panel)] shadow-sm overflow-hidden">
          <div class="border-b border-[var(--border)] px-3.5 py-2.5">
            <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.system") }}</h3>
          </div>
          <div class="space-y-3 p-4">
            <label class="flex cursor-pointer items-center gap-3">
              <input
                type="checkbox"
                class="h-4 w-4 accent-[#5865F2]"
                :checked="closeToTray"
                @change="toggleCloseToTray(($event.target as HTMLInputElement).checked)"
              />
              <span class="text-[13px] text-[color:var(--tx)]">{{ t("settings.closeToTray") }}</span>
            </label>
            <label class="flex cursor-pointer items-center gap-3">
              <input
                type="checkbox"
                class="h-4 w-4 accent-[#5865F2]"
                :checked="autostartOn"
                @change="toggleAutostart(($event.target as HTMLInputElement).checked)"
              />
              <span class="text-[13px] text-[color:var(--tx)]">{{ t("settings.autostart") }}</span>
            </label>
          </div>
        </section>

        <!-- Предупреждение о кастомных модах -->
        <section class="rounded-xl bg-[var(--panel)] shadow-sm overflow-hidden">
          <div class="border-b border-[var(--border)] px-3.5 py-2.5">
            <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.warnCustomMods") }}</h3>
          </div>
          <div class="p-4">
            <label class="flex cursor-pointer items-center gap-3">
              <input
                type="checkbox"
                class="h-4 w-4 accent-[#f0883e]"
                :checked="warnCustomMods"
                @change="toggleWarnCustomMods(($event.target as HTMLInputElement).checked)"
              />
              <span class="text-[13px] text-[color:var(--tx)]">{{ t("settings.warnCustomModsLabel") }}</span>
            </label>
            <p class="mt-2 text-[13px] text-[color:var(--tx-muted)]">
              {{ t("settings.warnCustomModsNote") }}
            </p>
          </div>
        </section>

        <!-- Проверка целостности -->
        <section class="rounded-xl bg-[var(--panel)] shadow-sm overflow-hidden">
          <div class="border-b border-[var(--border)] px-3.5 py-2.5 flex justify-between items-center">
            <h3 class="text-[13px] font-semibold text-[color:var(--tx-strong)]">{{ t("settings.verify") }}</h3>
          </div>
          <div class="p-4 space-y-3">
            <p class="text-[13px] text-[color:var(--tx-muted)]">
              {{ t("settings.verifyNote") }}
            </p>
            <button
              type="button"
              class="rounded-md bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] disabled:opacity-50"
              :disabled="verifyBusy || busy"
              @click="handleVerify"
            >
              {{ verifyBusy ? t("settings.verifying") : t("settings.verifyBtn") }}
            </button>
            <div
              v-if="verifyResult"
              class="rounded-md bg-[var(--bg-60)] p-3 text-[13px]"
            >
              <p class="font-medium" :class="verifyResult.broken.length === 0 ? 'text-[#3fb950]' : 'text-[#f85149]'">
                {{ verifyResult.broken.length === 0 ? t("settings.verifyOk") : t("settings.verifyBroken", { n: verifyResult.broken.length }) }}
              </p>
              <p class="mt-0.5 text-[color:var(--tx-muted)]">{{ t("settings.verifyStats", { checked: verifyResult.checked, ok: verifyResult.ok }) }}</p>
              <ul v-if="verifyResult.broken.length > 0" class="mt-2 max-h-32 space-y-1 overflow-y-auto font-mono text-xs text-[#f85149]">
                <li v-for="b in verifyResult.broken" :key="b">{{ b }}</li>
              </ul>
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>
