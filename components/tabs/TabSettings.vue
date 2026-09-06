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
  getNetworkSettings,
  setNetworkSettings,
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

type SettingsTab = "accounts" | "game" | "appearance" | "network";
const settingsTab = ref<SettingsTab>("accounts");

const SETTINGS_TAB_ICONS: Record<SettingsTab, string> = {
  accounts: 'user',
  game: 'bolt',
  appearance: 'half-circle',
  network: 'bars',
};

const SETTINGS_TABS: { id: SettingsTab; label: string; icon: string }[] = [
  { id: "accounts", label: "Аккаунты", icon: "user" },
  { id: "game", label: "Запуск игры", icon: "bolt" },
  { id: "appearance", label: "Интерфейс", icon: "half-circle" },
  { id: "network", label: "Сеть и система", icon: "bars" },
];

// --- RAM helpers ---
const totalSystemRam = computed(() => systemRam?.value?.total_ram_gb ?? maxRam?.value ?? 16);
const ramPct = computed(() => (totalSystemRam.value > 0 ? (ram.value / totalSystemRam.value) * 100 : 0));
const ramOver = computed(() => ramPct.value > 70);

// --- Resolution presets ---
function setResolution(w: number, h: number) {
  windowWidth.value = w;
  windowHeight.value = h;
}

// --- JVM reset ---
const DEFAULT_JVM_ARGS = "";
function resetJvmArgs() {
  jvmArgs.value = DEFAULT_JVM_ARGS;
  void saveJvmArgs();
}

// --- Java browse ---
async function browseJava() {
  if (!isTauri()) { notify(t("skin.tauriOnly"), "info"); return; }
  try {
    const p = await openDialog({ multiple: false, filters: [{ name: "Java", extensions: ["exe", "bin", ""] }] });
    if (typeof p === "string" && p) selectJava(p);
  } catch (e) { notify(String(e)); }
}

// --- Proxy split fields ---
const proxyProto = ref("HTTP");
const proxyHost = ref("");
const proxyPort = ref("");
function syncProxyFromRaw() {
  const raw = netProxy.value.trim();
  const m = raw.match(/^(https?|socks5?):\/\/([^:]+)(?::(\d+))?/i);
  if (m) {
    proxyProto.value = m[1].toLowerCase().startsWith("socks") ? "SOCKS5" : "HTTP";
    proxyHost.value = m[2];
    proxyPort.value = m[3] ?? "";
  } else if (raw) {
    const hp = raw.split(":");
    proxyHost.value = hp[0] ?? "";
    proxyPort.value = hp[1] ?? "";
  }
}
function syncProxyToRaw() {
  if (!proxyHost.value.trim()) { netProxy.value = ""; return; }
  const scheme = proxyProto.value === "SOCKS5" ? "socks5" : "http";
  netProxy.value = `${scheme}://${proxyHost.value.trim()}${proxyPort.value ? ":" + proxyPort.value : ""}`;
}

// --- Speed unlimited ---
const speedUnlimited = computed(() => netSpeedLimit.value === 0);
function setSpeedUnlimited(on: boolean) {
  netSpeedLimit.value = on ? 0 : 1024;
}

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

// --- Network settings ---

const netConcurrent = ref(8);
const netSpeedLimit = ref(0);
const netProxy = ref("");
const netForceIpv4 = ref(true);
const netSaving = ref(false);

async function loadNetworkSettings() {
  if (!isTauri()) return;
  try {
    const s = await getNetworkSettings();
    netConcurrent.value = s.concurrent;
    netSpeedLimit.value = s.speed_limit_kb;
    netProxy.value = s.proxy;
    netForceIpv4.value = s.force_ipv4;
    syncProxyFromRaw();
  } catch {
    /* ignore */
  }
}

async function saveNetworkSettings() {
  if (!isTauri() || netSaving.value) return;
  netSaving.value = true;
  try {
    syncProxyToRaw();
    await setNetworkSettings({
      concurrent: netConcurrent.value,
      speed_limit_kb: netSpeedLimit.value,
      proxy: netProxy.value,
      force_ipv4: netForceIpv4.value,
    });
    notify(t("settings.netSaved"), "success");
  } catch (e) {
    notify(t("files.updateErr", { e }), "error");
  } finally {
    netSaving.value = false;
  }
}

onMounted(() => {
  void loadNetworkSettings();
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
        <div class="mt-3 flex items-center gap-1.5 p-1 rounded-2xl bg-[var(--input)]/40 border border-[var(--border)] w-fit mb-5">
          <button
            v-for="st in SETTINGS_TABS"
            :key="st.id"
            type="button"
            class="inline-flex items-center gap-1.5 rounded-xl px-4 py-2 text-xs transition-all"
            :class="settingsTab === st.id ? 'bg-[var(--panel)] text-[color:var(--tx)] font-semibold shadow-sm' : 'text-[color:var(--tx-muted)] hover:text-[color:var(--tx)]'"
            @click="settingsTab = st.id"
          >
            <AppIcon :name="st.icon" class="h-3.5 w-3.5 fill-current" />
            {{ st.label }}
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
                      <AppIcon name="check-circle" class="h-4 w-4 shrink-0 fill-[#3fb950]" />
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
                    <AppIcon name="spinner" class="h-3 w-3 fill-[var(--accent)]" />
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
                      <AppIcon name="trash" class="h-3 w-3 fill-current" />
                    </button>
                  </div>
                </div>
              </div>
            </section>
          </div>

          <!-- Boosty: платные сборки -->
          <section class="rounded-xl bg-[var(--panel)] shadow-sm overflow-hidden">
            <div class="border-b border-[var(--border)] px-3.5 py-2.5 flex items-center gap-2">
              <AppIcon name="bolt" class="h-4 w-4 shrink-0 fill-[var(--accent)]" />
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
                    <AppIcon v-if="boostyAuthOpen && boostyGlobalOpen" name="spinner" class="h-3 w-3 fill-current" />
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
                    <AppIcon v-if="boostyAuthOpen && boostyTargetPack === p.id" name="spinner" class="h-3 w-3 fill-current" />
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

      <template v-else-if="settingsTab === 'appearance'">
        <div class="space-y-4">
          <!-- Тема: слайдер -->
          <section class="rounded-2xl bg-[var(--input)]/25 border border-[var(--border)] p-4 mb-4">
            <div class="flex items-center justify-between border-b border-[var(--border)] px-3.5 py-2.5">
              <h3 class="text-[13px] font-semibold text-[color:var(--tx)]">{{ t("settings.theme") }}</h3>
              <span class="inline-flex items-center gap-1 text-xs font-medium text-[color:var(--tx-muted)]">
                <AppIcon :name="themeLevel >= 0.5 ? 'moon' : 'sun'" class="h-3.5 w-3.5 fill-current" />
                {{ themeLevel >= 0.5 ? t("theme.dark") : t("theme.light") }}
              </span>
            </div>
            <div class="space-y-3 p-4">
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
              <p v-if="packThemeActive" class="mt-2 text-[13px] text-[var(--accent)]">
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

          <!-- Discord Rich Presence toggle row -->
          <section class="rounded-2xl bg-[var(--input)]/25 border border-[var(--border)] p-4 mb-4">
            <label class="flex cursor-pointer items-center justify-between gap-3">
              <span>
                <span class="block text-[13px] font-medium text-[color:var(--tx)]">Discord Rich Presence</span>
                <span class="block text-xs text-[color:var(--tx-muted)]">{{ t("settings.discordLabel") }}</span>
              </span>
              <input
                type="checkbox"
                class="h-4 w-4 accent-[#5865F2]"
                :checked="discordRp"
                @change="toggleDiscordRp(($event.target as HTMLInputElement).checked)"
              />
            </label>
          </section>

          <!-- Custom mods warning toggle row -->
          <section class="rounded-2xl bg-[var(--input)]/25 border border-[var(--border)] p-4 mb-4">
            <label class="flex cursor-pointer items-center justify-between gap-3">
              <span>
                <span class="block text-[13px] font-medium text-[color:var(--tx)]">{{ t("settings.warnCustomMods") }}</span>
                <span class="block text-xs text-[color:var(--tx-muted)]">{{ t("settings.warnCustomModsLabel") }}</span>
              </span>
              <input
                type="checkbox"
                class="h-4 w-4 accent-[#f0883e]"
                :checked="warnCustomMods"
                @change="toggleWarnCustomMods(($event.target as HTMLInputElement).checked)"
              />
            </label>
          </section>
        </div>
      </template>

      <template v-else-if="settingsTab === 'network'">
        <div class="space-y-4">
          <!-- Одновременные скачивания -->
          <section class="rounded-2xl bg-[var(--input)]/25 border border-[var(--border)] p-4 mb-4">
            <div class="flex justify-between items-center">
              <h3 class="text-[13px] font-semibold text-[color:var(--tx)]">{{ t("settings.netConcurrent") }}</h3>
              <span class="font-mono text-[13px] font-semibold text-[var(--accent)]">{{ netConcurrent }}</span>
            </div>
            <div class="p-4 space-y-2">
              <input
                type="range"
                min="1"
                max="32"
                step="1"
                v-model.number="netConcurrent"
                class="w-full accent-[var(--accent-deep)] bg-[var(--input)] h-1.5 rounded-lg appearance-none cursor-pointer"
              />
              <p class="text-[13px] text-[color:var(--tx-muted)]">{{ t("settings.netConcurrentNote") }}</p>
            </div>
          </section>

          <!-- Ограничение скорости -->
          <section class="rounded-2xl bg-[var(--input)]/25 border border-[var(--border)] p-4 mb-4">
            <div class="flex justify-between items-center">
              <h3 class="text-[13px] font-semibold text-[color:var(--tx)]">{{ t("settings.netSpeedLimit") }}</h3>
              <span class="font-mono text-[13px] font-semibold text-[var(--accent)]">
                {{ netSpeedLimit === 0 ? t("settings.netSpeedLimitUnlimited") : netSpeedLimit + " КБ/с" }}
              </span>
            </div>
            <div class="p-4 space-y-2">
              <label class="flex cursor-pointer items-center justify-between gap-3">
                <span class="text-[13px] text-[color:var(--tx)]">[ Без ограничений ]</span>
                <input
                  type="checkbox"
                  class="h-4 w-4 accent-[var(--accent-deep)]"
                  :checked="speedUnlimited"
                  @change="setSpeedUnlimited(($event.target as HTMLInputElement).checked)"
                />
              </label>
              <input
                type="range"
                min="0"
                max="10240"
                step="64"
                v-model.number="netSpeedLimit"
                class="w-full accent-[var(--accent-deep)] bg-[var(--input)] h-1.5 rounded-lg appearance-none cursor-pointer"
              />
              <p class="text-[13px] text-[color:var(--tx-muted)]">{{ t("settings.netSpeedLimitNote") }}</p>
            </div>
          </section>

          <!-- Прокси: protocol + host/port -->
          <section class="rounded-2xl bg-[var(--input)]/25 border border-[var(--border)] p-4 mb-4">
            <div class="border-b border-[var(--border)] px-3.5 py-2.5">
              <h3 class="text-[13px] font-semibold text-[color:var(--tx)]">{{ t("settings.netProxy") }}</h3>
            </div>
            <div class="p-4 space-y-2">
              <div class="flex gap-2">
                <select
                  v-model="proxyProto"
                  class="shrink-0 rounded-md bg-[var(--input)] px-2.5 py-2 text-[13px] text-[color:var(--tx)] focus:outline-none"
                  @change="syncProxyToRaw"
                >
                  <option value="HTTP">HTTP</option>
                  <option value="SOCKS5">SOCKS5</option>
                </select>
                <input
                  v-model="proxyHost"
                  type="text"
                  placeholder="proxy.example.com"
                  class="min-w-0 flex-1 rounded-md bg-[var(--input)] border border-[var(--border)] px-3 py-2 font-mono text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none focus:border-[var(--accent)]"
                  @input="syncProxyToRaw"
                />
                <input
                  v-model="proxyPort"
                  type="number"
                  min="1"
                  max="65535"
                  placeholder="8080"
                  class="w-24 shrink-0 rounded-md bg-[var(--input)] border border-[var(--border)] px-3 py-2 font-mono text-[13px] text-[color:var(--tx)] placeholder-[var(--tx-muted)] focus:outline-none focus:border-[var(--accent)]"
                  @input="syncProxyToRaw"
                />
              </div>
              <p class="text-[13px] text-[color:var(--tx-muted)]">{{ t("settings.netProxyNote") }}</p>
            </div>
          </section>

          <!-- IPv4 -->
          <section class="rounded-2xl bg-[var(--input)]/25 border border-[var(--border)] p-4 mb-4">
            <div class="border-b border-[var(--border)] px-3.5 py-2.5">
              <h3 class="text-[13px] font-semibold text-[color:var(--tx)]">{{ t("settings.netIpv4") }}</h3>
            </div>
            <div class="p-4">
              <label class="flex cursor-pointer items-center gap-3">
                <input
                  type="checkbox"
                  class="h-4 w-4 accent-[var(--accent-deep)]"
                  :checked="netForceIpv4"
                  @change="netForceIpv4 = ($event.target as HTMLInputElement).checked"
                />
                <span class="text-[13px] text-[color:var(--tx)]">{{ t("settings.netIpv4Note") }}</span>
              </label>
            </div>
          </section>

          <!-- System behavior toggles -->
          <section class="rounded-2xl bg-[var(--input)]/25 border border-[var(--border)] p-4 mb-4">
            <div class="border-b border-[var(--border)] px-3.5 py-2.5">
              <h3 class="text-[13px] font-semibold text-[color:var(--tx)]">{{ t("settings.system") }}</h3>
            </div>
            <div class="space-y-3 p-4">
              <label class="flex cursor-pointer items-center justify-between gap-3">
                <span class="text-[13px] text-[color:var(--tx)]">{{ t("settings.closeToTray") }}</span>
                <input
                  type="checkbox"
                  class="h-4 w-4 accent-[#5865F2]"
                  :checked="closeToTray"
                  @change="toggleCloseToTray(($event.target as HTMLInputElement).checked)"
                />
              </label>
              <label class="flex cursor-pointer items-center justify-between gap-3">
                <span class="text-[13px] text-[color:var(--tx)]">{{ t("settings.autostart") }}</span>
                <input
                  type="checkbox"
                  class="h-4 w-4 accent-[#5865F2]"
                  :checked="autostartOn"
                  @change="toggleAutostart(($event.target as HTMLInputElement).checked)"
                />
              </label>
            </div>
          </section>

          <!-- Кнопка сохранить -->
          <button
            type="button"
            class="w-full rounded-lg bg-[color-mix(in_srgb,var(--accent)_15%,transparent)] py-2.5 text-[13px] font-semibold text-[var(--accent)] hover:bg-[color-mix(in_srgb,var(--accent)_25%,transparent)] disabled:opacity-50"
            :disabled="netSaving"
            @click="saveNetworkSettings"
          >
            {{ netSaving ? t("common.saving") : t("common.save") }}
          </button>
        </div>
      </template>

      <!-- GAME: only when game tab active (no longer duplicated) -->
      <template v-else-if="settingsTab === 'game'">
      <div class="space-y-4">
        <!-- ОЗУ -->
        <section class="rounded-2xl bg-[var(--input)]/25 border border-[var(--border)] p-4 mb-4">
          <div class="flex justify-between items-center">
            <h3 class="text-[13px] font-semibold text-[color:var(--tx)]">{{ t("settings.ram") }}</h3>
            <span class="rounded-xl bg-[var(--accent)] px-3 py-1 font-mono text-[13px] font-semibold text-white">[ {{ ram }} {{ t("units.gb") }} ]</span>
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
            <div class="flex justify-between text-[13px] font-mono">
              <span class="text-[color:var(--tx-muted)]">Мин: 2 ГБ</span>
              <span class="text-[color:var(--tx-muted)]">Рекомендуется: 4–8 ГБ</span>
              <span class="text-[color:var(--tx-muted)]">Макс: {{ totalSystemRam }} ГБ</span>
            </div>
            <p v-if="ramOver" class="flex items-center gap-1.5 rounded-xl bg-[#f0883e]/10 px-3 py-1.5 text-xs font-medium text-[#f0883e]">
              <AppIcon name="alert-circle" class="h-3.5 w-3.5 fill-current" />
              <span>Выделено больше 70% системной памяти ({{ Math.round(ramPct) }}%)</span>
            </p>
            <p v-if="systemRam && systemRam.total_ram_gb > 0" class="text-[13px] text-[color:var(--tx-muted)]">
              {{ t("settings.ramTotal", { total: systemRam.total_ram_gb, avail: systemRam.available_ram_gb }) }}
            </p>
            <p
              v-if="activePack?.minRam"
              class="text-[13px]"
              :class="(ram * 1024) < (activePack?.minRam ?? 0) ? 'font-medium text-[#f0883e]' : 'text-[color:var(--tx-muted)]'"
            >
              {{ t("settings.ramMin", { name: activePack?.name ?? '', min: (activePack?.minRam ?? 0) / 1024, gb: ram }) }}
            </p>
          </div>
        </section>

        <!-- JVM-аргументы -->
        <section class="rounded-2xl bg-[var(--input)]/25 border border-[var(--border)] p-4 mb-4">
          <div class="flex justify-between items-center">
            <h3 class="text-[13px] font-semibold text-[color:var(--tx)]">{{ t("settings.jvmArgs") }}</h3>
            <div class="flex gap-2">
              <button
                type="button"
                class="rounded-xl px-3 py-1 text-xs text-[color:var(--tx-muted)] hover:text-[color:var(--tx)] transition-all"
                :disabled="jvmArgsSaving"
                @click="resetJvmArgs"
              >
                [ Сбросить по умолчанию ]
              </button>
              <button
                type="button"
                class="text-[13px] underline decoration-dotted underline-offset-2 disabled:opacity-50"
                :disabled="jvmArgsSaving"
                @click="saveJvmArgs"
              >
                {{ jvmArgsSaving ? t("common.saving") : t("common.save") }}
              </button>
            </div>
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
        <section class="rounded-2xl bg-[var(--input)]/25 border border-[var(--border)] p-4 mb-4">
          <div class="flex justify-between items-center">
            <h3 class="text-[13px] font-semibold text-[color:var(--tx)]">{{ t("settings.win") }}</h3>
            <span class="font-mono text-[13px] font-semibold text-[var(--accent)]">{{ windowWidth }}×{{ windowHeight }}</span>
          </div>
          <div class="p-4 space-y-2">
            <div class="flex items-center gap-2">
              <span class="text-[13px] text-[color:var(--tx-muted)]">Ширина</span>
              <input
                id="ts-win-width"
                type="number"
                min="320"
                max="7680"
                step="1"
                v-model.number="windowWidth"
                placeholder="854"
                class="flex-1 rounded-md bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)] focus:outline-none"
              />
              <span class="text-[color:var(--tx-muted)]">×</span>
              <span class="text-[13px] text-[color:var(--tx-muted)]">Высота</span>
              <input
                id="ts-win-height"
                type="number"
                min="240"
                max="4320"
                step="1"
                v-model.number="windowHeight"
                placeholder="480"
                class="flex-1 rounded-md bg-[var(--bg)] px-3 py-2 text-[13px] text-[color:var(--tx)] focus:outline-none"
              />
            </div>
            <div class="flex gap-2">
              <button type="button" class="rounded-xl px-3 py-1.5 text-xs text-[color:var(--tx-muted)] hover:text-[color:var(--tx)] bg-[var(--input)]/40 border border-[var(--border)] transition-all" @click="setResolution(1920, 1080)">[ 1080p ]</button>
              <button type="button" class="rounded-xl px-3 py-1.5 text-xs text-[color:var(--tx-muted)] hover:text-[color:var(--tx)] bg-[var(--input)]/40 border border-[var(--border)] transition-all" @click="setResolution(1280, 720)">[ 720p ]</button>
              <button type="button" class="rounded-xl px-3 py-1.5 text-xs text-[color:var(--tx-muted)] hover:text-[color:var(--tx)] bg-[var(--input)]/40 border border-[var(--border)] transition-all" @click="setResolution(854, 480)">[ Полный экран ]</button>
            </div>
            <p class="text-[13px] text-[color:var(--tx-muted)]">
              {{ t("settings.winNote") }}
            </p>
          </div>
        </section>

        <!-- Java -->
        <section class="rounded-2xl bg-[var(--input)]/25 border border-[var(--border)] p-4 mb-4">
          <div class="border-b border-[var(--border)] px-3.5 py-2.5">
            <h3 class="text-[13px] font-semibold text-[color:var(--tx)]">{{ t("settings.java") }}</h3>
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
                @click="browseJava"
              >
                [ Обзор... ]
              </button>
              <button
                type="button"
                class="shrink-0 rounded-md bg-[var(--input)] px-3 py-2 text-[13px] font-medium text-[color:var(--tx)] transition-colors hover:bg-[var(--hover)] disabled:opacity-50"
                :disabled="javaBusy || busy"
                @click="downloadJava"
              >
                {{ javaBusy ? t("settings.javaDownloading") : "[ Скачать JRE ]" }}
              </button>
            </div>
            <p v-if="javaMsg" class="text-[13px] text-[color:var(--tx-muted)] break-all">{{ javaMsg }}</p>
            <p class="text-[13px] text-[color:var(--tx-muted)]">
              {{ t("settings.javaNote") }}
            </p>
          </div>
        </section>

        <!-- Проверка целостности -->
        <section class="rounded-2xl bg-[var(--input)]/25 border border-[var(--border)] p-4 mb-4">
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
      </template>
    </div>
  </div>
</template>
