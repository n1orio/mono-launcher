import { ref, watch } from "vue";
import type { Ref } from "vue";
import {
  isTauri,
  setCloseToTray,
  autostartSet,
  autostartGet,
  getUserJvmArgs,
  setUserJvmArgs,
} from "~/lib/bridge";

export interface UseSystemSettingsDeps {
  notify: (text: string, type?: string) => void;
  playSubTab: Ref<string>;
  t: (key: string, params?: Record<string, unknown>) => string;
}

/**
 * Системные настройки: сворачивание в трей, автозапуск и пользовательские JVM-аргументы.
 * Извлечено из index.vue (строки 6557-6578, 6865-6887, 7098-7109).
 */
export function useSystemSettings(deps: UseSystemSettingsDeps) {
  const { notify, playSubTab, t } = deps;

  // ---- Сворачивание в трей + автозапуск (Настройки → Система) ----

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

  /** Восстановить настройки трея/автозапуска. */
  async function initSystemPrefs() {
    closeToTray.value = localStorage.getItem("mono.closeToTray") === "1";
    if (isTauri()) {
      try {
        await setCloseToTray(closeToTray.value);
        autostartOn.value = await autostartGet();
      } catch {
        // плагин недоступен
      }
    }
  }

  // ---- Пользовательские JVM-аргументы (настройки сборки) ----

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

  // Автозагрузка JVM-аргументов при переключении на вкладку "Настройки"
  watch(playSubTab, (tab) => {
    if (tab === "settings") void loadJvmArgs();
  }, { immediate: true });

  return {
    closeToTray,
    autostartOn,
    toggleCloseToTray,
    toggleAutostart,
    initSystemPrefs,
    jvmArgs,
    jvmArgsSaving,
    loadJvmArgs,
    saveJvmArgs,
  };
}
