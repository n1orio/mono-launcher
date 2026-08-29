<script setup lang="ts">
import { useLauncherCtx } from "~/composables/useLauncherContext";
import { useI18n } from "#imports";

const { t } = useI18n();
const {
  loginOpen,
  loginBusy,
  loginLogin,
  loginPassword,
  loginPasswordVisible,
  loginError,
  loginMethod,
  doLogin,
} = useLauncherCtx();
</script>

<template>
  <div
    v-if="loginOpen"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6"
    @click.self="loginOpen = false"
  >
    <div class="flex w-full max-w-sm flex-col overflow-hidden rounded-xl  bg-[var(--panel)] shadow-2xl">
      <div class="flex items-center justify-between border-b border-[var(--border)]  px-3.5 py-2.5">
        <h3 class="text-sm font-semibold text-[color:var(--tx-strong)]">{{ t("auth.login") }}</h3>
        <button
          type="button"
          class="rounded-md p-1 text-[color:var(--tx-muted)] transition-colors hover:bg-[var(--hover)] hover:text-[color:var(--tx-strong)]"
          @click="loginOpen = false"
        >
          <svg viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
        </button>
      </div>
      <form class="space-y-3 p-4" @submit.prevent="doLogin">
        <div v-if="loginError" class="rounded-md bg-red-500/10 px-3 py-2 text-xs text-red-500">{{ loginError }}</div>
        <label class="block">
          <span class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("auth.emailOrLogin") }}</span>
          <input
            v-model="loginLogin"
            type="text"
            autocomplete="username"
            class="w-full rounded-md  bg-[var(--input)] px-3 py-2 text-[13px] text-[color:var(--tx)] outline-none transition-colors "
            @keydown.enter="doLogin"
          />
        </label>
        <label class="block">
          <span class="mb-1 block text-[13px] font-medium text-[color:var(--tx-muted)]">{{ t("auth.password") }}</span>
          <div class="relative">
            <input
              v-model="loginPassword"
              :type="loginPasswordVisible ? 'text' : 'password'"
              autocomplete="current-password"
              class="w-full rounded-md  bg-[var(--input)] px-3 py-2 pr-9 text-[13px] text-[color:var(--tx)] outline-none transition-colors "
              @keydown.enter="doLogin"
            />
            <button
              type="button"
              class="absolute right-2.5 top-1/2 -translate-y-1/2 text-[color:var(--tx-muted)] transition-colors hover:text-[color:var(--tx)]"
              tabindex="-1"
              @click="loginPasswordVisible = !loginPasswordVisible"
            >
              <svg v-if="!loginPasswordVisible" viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M8 2c-2.837 0-5.352 1.615-6.868 3.653a.5.5 0 0 0 0 .694C2.648 8.385 5.163 10 8 10s5.352-1.615 6.868-3.653a.5.5 0 0 0 0-.694C13.352 3.615 10.837 2 8 2Zm0 5.5a2 2 0 1 1 0-4 2 2 0 0 1 0 4Z"/></svg>
              <svg v-else viewBox="0 0 16 16" class="h-4 w-4 fill-current"><path d="M1.053 3.213A11.5 11.5 0 0 1 8 1.5c2.53 0 4.903.6 6.947 1.713.567.316 1.053.646 1.053 1.084v.001c0 .438-.486.768-1.053 1.084A11.5 11.5 0 0 1 8 7.5a11.5 11.5 0 0 1-6.947-3.118C.486 4.052 0 3.722 0 3.284v-.001c0-.438.486-.768 1.053-1.07ZM8 10a2 2 0 1 1 0-4 2 2 0 0 1 0 4Z"/></svg>
            </button>
          </div>
        </label>
        <button
          type="submit"
          class="flex w-full items-center justify-center gap-2 rounded-md  bg-[color-mix(in_srgb,var(--accent)_12%,transparent)] px-3 py-2 text-[13px] font-semibold text-[var(--accent)] transition-colors hover:bg-[color-mix(in_srgb,var(--accent)_22%,transparent)] disabled:opacity-50"
          :disabled="loginBusy || !loginLogin.trim() || !loginPassword.trim()"
        >
          <svg v-if="loginBusy" viewBox="0 0 16 16" class="h-4 w-4 animate-spin fill-current">
            <path d="M8 1a7 7 0 1 0 7 7h-1.5A5.5 5.5 0 1 1 8 2.5V1Z"/>
          </svg>
          {{ loginBusy ? t("auth.loggingIn") : t("auth.loginBtn") }}
        </button>
      </form>
    </div>
  </div>
</template>
