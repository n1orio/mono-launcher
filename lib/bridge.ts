import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppStatus,
  DownloadProgress,
  GameFileEntry,
  GameFileIcon,
  JavaInfo,
  LaunchLogEntry,
  MsDeviceCodeInfo,
  NewsItem,
  PackDescriptor,
  PackInfo,
  PackRepoContent,
  SystemInfo,
  UpdateInfo,
  UserSession,
  VerifyResult,
  VersionsInfo,
} from "./types";

export const isTauri = () =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export function listPacks(): Promise<PackDescriptor[]> {
  return invoke("list_packs");
}

export function addPack(url: string, name?: string): Promise<PackDescriptor> {
  return invoke("add_pack_command", { url, name: name ?? null });
}

export function removePack(packId: string): Promise<void> {
  return invoke("remove_pack_command", { packId });
}

export interface PackAddedPayload {
  ok: boolean;
  already: boolean;
  id: string;
  name: string;
  error?: string;
}

export function onPackAdded(
  cb: (p: PackAddedPayload) => void
): Promise<UnlistenFn> {
  return listen<PackAddedPayload>("pack-added", (event) => cb(event.payload));
}

/** Результат добавления по deep link, если фронтенд стартовал позже события. */
export function takePendingPackAdd(): Promise<PackAddedPayload | null> {
  return invoke("take_pending_pack_add");
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

export function listJava(): Promise<JavaInfo[]> {
  return invoke("list_java_command");
}

export function setJavaPath(path: string | null): Promise<void> {
  return invoke("set_java_path_command", { path });
}

export function ensureJava(): Promise<string> {
  return invoke("ensure_java_command");
}

export function verifyGame(packId: string): Promise<VerifyResult> {
  return invoke("verify_game_command", { packId });
}

export type GameFolder =
  | "mods"
  | "screenshots"
  | "resourcepacks"
  | "shaderpacks"
  | "saves"
  | "logs";

export function openGameFolder(packId: string, folder: GameFolder): Promise<void> {
  return invoke("open_game_folder_command", { packId, folder });
}

export function getSkin(uuid: string): Promise<string | null> {
  return invoke("get_skin_command", { uuid });
}

export function onJavaProgress(cb: (e: LaunchLogEntry) => void): Promise<UnlistenFn> {
  return listen<LaunchLogEntry>("launch-log", (event) => cb(event.payload));
}

export function setDiscordRp(enabled: boolean): Promise<void> {
  return invoke("set_discord_rp_command", { enabled });
}

export function setLocale(locale: string): Promise<void> {
  return invoke("set_locale_command", { locale });
}

export function getNews(): Promise<NewsItem[]> {
  return invoke("get_news_command");
}

export function packRepoContent(packId: string): Promise<PackRepoContent> {
  return invoke("pack_repo_content_command", { packId });
}

export type GameFolderKind = "mods" | "resourcepacks" | "shaderpacks" | "saves";

export function listGameFiles(
  packId: string,
  folder: GameFolderKind
): Promise<GameFileEntry[]> {
  return invoke("list_game_files_command", { packId, folder });
}

export function toggleGameFile(
  packId: string,
  folder: GameFolderKind,
  name: string,
  enabled: boolean
): Promise<void> {
  return invoke("toggle_game_file_command", { packId, folder, name, enabled });
}

export function getGameFileIcon(
  packId: string,
  folder: GameFolderKind,
  name: string
): Promise<string | null> {
  return invoke("get_game_file_icon_command", { packId, folder, name });
}

export function getGameFileIcons(
  packId: string,
  folder: GameFolderKind,
  names: string[]
): Promise<GameFileIcon[]> {
  return invoke("get_game_file_icons_command", { packId, folder, names });
}