/**
 * useCustomModsChecker — мини-движок управления плашкой непроверенных модов.
 *
 * Состояния:
 *   'none'       — custom-файлов нет (чистая сборка)
 *   'loading'    — проверяем данные
 *   'unchecked'  — есть непроверенные файлы (safe === undefined)
 *   'scanning'   — идёт сканирование
 *   'safe'       — все файлы безопасны
 *   'dangerous'  — есть файлы с опасными классами
 *   'error'      — ошибка сканирования
 */

import { ref, watch, type Ref } from "vue";
import { scanCustomMods, isTauri } from "~/lib/bridge";

export type CustomModsState = "none" | "loading" | "unchecked" | "scanning" | "safe" | "dangerous" | "error";

export interface CustomModsChecker {
  /** Текущее состояние плашки */
  state: Ref<CustomModsState>;
  /** Количество непроверенных файлов */
  uncheckedCount: Ref<number>;
  /** Количество опасных файлов */
  dangerousCount: Ref<number>;
  /** Количество безопасных файлов */
  safeCount: Ref<number>;
  /** Общее количество custom-файлов */
  totalCount: Ref<number>;
  /** Список файлов с деталями */
  files: Ref<any[]>;
  /** Флаг, что идёт сканирование */
  scanning: Ref<boolean>;
  /** Сообщение об ошибке */
  errorMessage: Ref<string | null>;
  /** Запустить сканирование вручную */
  runScan: () => Promise<void>;
  /** Сбросить состояние */
  reset: () => void;
}

interface CustomModsFile {
  path: string;
  url: string;
  sha256?: string;
  safe?: boolean;
  scan_result?: string;
}

export function useCustomModsChecker(deps: {
  status: Ref<{ custom_mods?: CustomModsFile[]; active_version?: string | null } | null>;
  packId: Ref<string | null>;
  accessToken?: Ref<string | null>;
}): CustomModsChecker {
  const state = ref<CustomModsState>("none");
  const files = ref<any[]>([]);
  const scanning = ref(false);
  const errorMessage = ref<string | null>(null);

  const uncheckedCount = ref(0);
  const dangerousCount = ref(0);
  const safeCount = ref(0);
  const totalCount = ref(0);

  /** Обновить состояние на основе массива custom-файлов */
  function updateFromFiles(raw: CustomModsFile[] | null | undefined) {
    const arr = raw ?? [];
    files.value = arr;
    totalCount.value = arr.length;

    if (arr.length === 0) {
      state.value = "none";
      uncheckedCount.value = 0;
      dangerousCount.value = 0;
      safeCount.value = 0;
      return;
    }

    const unchecked = arr.filter((f) => f.safe !== true && f.safe !== false);
    const dangerous = arr.filter((f) => f.safe === false);
    const safe = arr.filter((f) => f.safe === true);

    uncheckedCount.value = unchecked.length;
    dangerousCount.value = dangerous.length;
    safeCount.value = safe.length;

    if (dangerous.length > 0) {
      state.value = "dangerous";
    } else if (unchecked.length > 0) {
      state.value = "unchecked";
    } else if (safe.length > 0) {
      state.value = "safe";
    } else {
      state.value = "none";
    }
  }

  /** Авто-скан при обнаружении непроверенных файлов */
  async function runScan(): Promise<void> {
    if (scanning.value) return;
    if (!isTauri()) return;
    const s = deps.status.value;
    const av = s?.active_version;
    if (!av || !deps.packId.value) return;

    scanning.value = true;
    state.value = "scanning";
    errorMessage.value = null;

    try {
      const token = deps.accessToken?.value ?? null;
      const [result, errors] = await scanCustomMods(deps.packId.value, av, token);
      if (errors.length > 0) {
        errorMessage.value = errors.join("; ");
      }
      const mapped = (result as CustomModsFile[]) ?? [];
      updateFromFiles(mapped);
    } catch (e: any) {
      state.value = "error";
      errorMessage.value = String(e?.message ?? e ?? "Unknown error");
    } finally {
      scanning.value = false;
    }
  }

  function reset() {
    state.value = "none";
    files.value = [];
    uncheckedCount.value = 0;
    dangerousCount.value = 0;
    safeCount.value = 0;
    totalCount.value = 0;
    scanning.value = false;
    errorMessage.value = null;
  }

  // Следим за статусом — при появлении custom_mods обновляем состояние
  watch(
    () => deps.status.value?.custom_mods,
    (mods) => {
      updateFromFiles(mods ?? []);
      // Автоскан: если есть непроверенные — запускаем
      if (
        mods &&
        mods.length > 0 &&
        deps.status.value?.active_version &&
        mods.some((f: CustomModsFile) => f.safe !== true && f.safe !== false)
      ) {
        setTimeout(() => {
          if (state.value === "unchecked" && !scanning.value) {
            void runScan();
          }
        }, 1000);
      }
    },
    { immediate: true }
  );

  return {
    state,
    uncheckedCount,
    dangerousCount,
    safeCount,
    totalCount,
    files,
    scanning,
    errorMessage,
    runScan,
    reset,
  };
}