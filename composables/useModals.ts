import { ref } from "vue";
import type { CrashAnalysis } from "~/lib/types";

export type ModalKey =
  | "modPack"
  | "createPack"
  | "export"
  | "editVersion"
  | "bugReport"
  | "crashAnalysis"
  | "modScanner"
  | "userProfile"
  | "fileDetail"
  | "search"
  | "dragDrop"
  | "customMods"
  | "loginMono"
  | "loginMicrosoft"
  | "loginEly"
  | "settings";

/** Централизованное управление модальными окнами лаунчера. */
export function useModals() {
  const modals = ref<Record<ModalKey, boolean>>({
    modPack: false,
    createPack: false,
    export: false,
    editVersion: false,
    bugReport: false,
    crashAnalysis: false,
    modScanner: false,
    userProfile: false,
    fileDetail: false,
    search: false,
    dragDrop: false,
    customMods: false,
    loginMono: false,
    loginMicrosoft: false,
    loginEly: false,
    settings: false,
  });

  /** Данные краш-анализа: если не null — модалка считается открытой. */
  const crashAnalysis = ref<CrashAnalysis | null>(null);

  function openModal(key: ModalKey) {
    modals.value[key] = true;
  }

  function closeModal(key: ModalKey) {
    modals.value[key] = false;
  }

  /** Закрыть модалку краш-анализа. */
  function closeCrashAnalysis() {
    crashAnalysis.value = null;
  }

  /** Закрыть верхнюю открытую модалку (порядок приоритета: ESC). */
  function closeTopModal() {
    if (modals.value.export) {
      modals.value.export = false;
    } else if (modals.value.editVersion) {
      modals.value.editVersion = false;
    } else if (crashAnalysis.value) {
      closeCrashAnalysis();
    } else if (modals.value.modPack) {
      modals.value.modPack = false;
    } else if (modals.value.createPack) {
      modals.value.createPack = false;
    } else if (modals.value.bugReport) {
      modals.value.bugReport = false;
    } else if (modals.value.search) {
      modals.value.search = false;
    } else if (modals.value.fileDetail) {
      modals.value.fileDetail = false;
    } else if (modals.value.userProfile) {
      modals.value.userProfile = false;
    }
  }

  return {
    modals,
    crashAnalysis,
    openModal,
    closeModal,
    closeCrashAnalysis,
    closeTopModal,
  };
}
