import { inject, type InjectionKey } from "vue";

/**
 * Общий контекст главной страницы: всё состояние и хелперы, которые
 * «живут» в index.vue (один вызов useLauncher + инлайновая логика).
 * Компоненты НЕ вызывают useLauncher() сами (он не синглтон и каждый вызов
 * регистрирует Tauri IPC-слушатели) — они получают объект через inject().
 */
export interface LauncherCtx {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [key: string]: any;
}

export const LauncherCtxKey: InjectionKey<LauncherCtx> = Symbol("launcher-ctx");

export function useLauncherCtx(): LauncherCtx {
  return inject(LauncherCtxKey) as LauncherCtx;
}