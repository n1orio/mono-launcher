import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppStatus,
  DownloadProgress,
  LaunchLogEntry,
  MsDeviceCodeInfo,
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

export function msDeviceCode(): Promise<MsDeviceCodeInfo> {
  return invoke("ms_device_code_command");
}

export function msPoll(
  deviceCode: string,
  interval: number,
  expiresIn: number
): Promise<UserSession> {
  return invoke("ms_poll_command", { deviceCode, interval, expiresIn });
}

export function launchGame(
  packId: string,
  ramGb: number,
  session: UserSession,
  width: number,
  height: number
): Promise<void> {
  return invoke("launch_game_command", { packId, ramGb, session, width, height });
}

export function onDownloadProgress(
  cb: (p: DownloadProgress) => void
): Promise<UnlistenFn> {
  return listen<DownloadProgress>("download-progress", (event) =>
    cb(event.payload)
  );
}

export interface PlaytimeUpdate {
  version_id: string;
  total_seconds: number;
}

export function onPlaytimeUpdated(
  cb: (p: PlaytimeUpdate) => void
): Promise<UnlistenFn> {
  return listen<PlaytimeUpdate>("playtime-updated", (event) =>
    cb(event.payload)
  );
}

export interface GameExitedPayload {
  success: boolean;
  code: number;
}

export function onGameExited(
  cb: (e: GameExitedPayload) => void
): Promise<UnlistenFn> {
  return listen<GameExitedPayload>("game-exited", (event) =>
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

export function launcherVersion(): Promise<string> {
  return invoke("launcher_version");
}

export function openExternal(url: string): Promise<void> {
  return invoke("open_url", { url });
}