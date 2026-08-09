import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppStatus,
  DownloadProgress,
  LaunchLogEntry,
  PackDescriptor,
  PackInfo,
  SystemInfo,
  UpdateInfo,
  UserSession,
  VersionsInfo,
} from "./types";

export const isTauri = () =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export function listPacks(): Promise<PackDescriptor[]> {
  return invoke("list_packs");
}

export function getStatus(packId: string): Promise<AppStatus> {
  return invoke("get_status", { packId });
}

export function checkForUpdates(packId: string): Promise<UpdateInfo> {
  return invoke("check_for_updates", { packId });
}

export function listVersions(packId: string): Promise<VersionsInfo> {
  return invoke("list_versions", { packId });
}

export function switchVersion(packId: string, versionId: string): Promise<void> {
  return invoke("switch_version", { packId, versionId });
}

export function getSystemInfo(): Promise<SystemInfo> {
  return invoke("system_info");
}

export function installMrpack(packId: string, tag?: string): Promise<PackInfo> {
  return invoke("install_mrpack", { packId, tag: tag ?? null });
}

export function loginOffline(username: string): Promise<UserSession> {
  return invoke("login_offline_command", { username });
}

export function loginMicrosoft(): Promise<UserSession> {
  return invoke("login_microsoft_command");
}

export function launchGame(
  packId: string,
  ramGb: number,
  session: UserSession
): Promise<void> {
  return invoke("launch_game_command", { packId, ramGb, session });
}

export function onDownloadProgress(
  cb: (p: DownloadProgress) => void
): Promise<UnlistenFn> {
  return listen<DownloadProgress>("download-progress", (event) =>
    cb(event.payload)
  );
}

export function onLaunchLog(
  cb: (e: LaunchLogEntry) => void
): Promise<UnlistenFn> {
  return listen<LaunchLogEntry>("launch-log", (event) => cb(event.payload));
}

export function getLaunchLog(): Promise<string> {
  return invoke("get_launch_log");
}

export function clearLaunchLog(): Promise<void> {
  return invoke("clear_launch_log");
}

export function openPackDir(packId: string): Promise<void> {
  return invoke("open_pack_dir", { packId });
}