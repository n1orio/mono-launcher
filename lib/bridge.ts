import { invoke } from "@tauri-apps/api/core";
import { emit, listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  Accounts,
  AppStatus,
  CurseCategory,
  CurseFile,
  CurseInstallResult,
  CursePackFile,
  CurseProjectDetail,
  CurseSearchHit,
  DownloadProgress,
  GameFileEntry,
  GameFileIcon,
  JavaInfo,
  LaunchLogEntry,
  BoostyAuth,
  LicenseInfo,
  McVersionInfo,
  ModUpdate,
  ModrinthProject,
  ModrinthTags,
  ModrinthVersion,
  MsDeviceCodeInfo,
  NewsItem,
  PackDescriptor,
  PackInfo,
  SavedServer,
  SavedServersList,
  ScreenshotList,
  DuplicatesResult,
  ServerStatus,
  SkinInfo,
  SystemInfo,
  TrackedMod,
  UpdateInfo,
  UserSession,
  VerifyResult,
  VersionsInfo,
  ExportSourceItem,
  CrashAnalysis,
  AuthorPackConfig,
  MonoPackPublic,
  MonoProfile,
  PackCatalog,
  PackDetail,
  PackNewsPublic,
  PackVersionPublic,
  UpdatePackRequest,
} from "./types";

export const isTauri = () =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export function listPacks(): Promise<PackDescriptor[]> {
  return invoke("list_packs");
}

export function recentPacks(): Promise<string[]> {
  return invoke("recent_packs_command");
}

export function addPack(url: string, name?: string, blog?: string): Promise<PackDescriptor> {
  return invoke("add_pack_command", { url, name: name ?? null, blog: blog ?? null });
}

export function addPackFile(path: string, name?: string): Promise<PackDescriptor> {
  return invoke("add_pack_file_command", { path, name: name ?? null });
}

export function setCloseToTray(enabled: boolean): Promise<void> {
  return invoke("set_close_to_tray_command", { enabled });
}

export function autostartSet(enabled: boolean): Promise<void> {
  return invoke("autostart_set_command", { enabled });
}

export function autostartGet(): Promise<boolean> {
  return invoke("autostart_get_command");
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

/** Список файлов сборки изменился (установка/удаление файла) — обновить UI. */
export function onModsChanged(cb: () => void): Promise<UnlistenFn> {
  return listen("mods-changed", () => cb());
}

/** Уровень темы изменился в другом окне — применить локально. */
export function onThemeChanged(cb: (level: number) => void): Promise<UnlistenFn> {
  return listen<{ level: number }>("theme-changed", (event) => cb(event.payload.level));
}

/** Рассылает уровень темы всем окнам (в т.ч. текущему — применяется идемпотентно). */
export function emitThemeChanged(level: number): void {
  void emit("theme-changed", { level });
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
/** Тихо обновляет Microsoft-сессию (refresh_token), возвращает актуальную сессию или null. */
export function msRefreshSession(): Promise<unknown> {
  return invoke("ms_refresh_session_command");
}

export function elyDeviceCode(): Promise<MsDeviceCodeInfo> {
  return invoke("ely_device_code_command");
}

export function elyPoll(
  deviceCode: string,
  interval: number,
  expiresIn: number
): Promise<UserSession> {
  return invoke("ely_poll_command", { deviceCode, interval, expiresIn });
}

export function monoRegister(username: string, password: string): Promise<MonoProfile> {
  return invoke("mono_register_command", { username, password });
}

export function monoLogin(username: string, password: string): Promise<MonoProfile> {
  return invoke("mono_login_command", { username, password });
}

export function monoProfile(): Promise<MonoProfile | null> {
  return invoke("mono_profile_command");
}

export function monoLogout(): Promise<MonoProfile | null> {
  return invoke("mono_logout_command");
}

export function curseforgeSearch(
  query: string,
  classId: number,
  categoryId: number | null = null,
  gameVersion?: string,
  sort?: string
): Promise<CurseSearchHit[]> {
  return invoke("curseforge_search_command", { query, classId, categoryId, gameVersion, sort });
}

export function curseforgeCategories(classId: number): Promise<CurseCategory[]> {
  return invoke("curseforge_categories_command", { classId });
}

export function curseforgeLatestFile(packId: string, projectId: number): Promise<CurseFile> {
  return invoke("curseforge_latest_file_command", { packId, projectId });
}

export function curseforgeInstallFile(
  packId: string,
  file: CurseFile,
  folder: string,
  title?: string,
  icon?: string
): Promise<CurseInstallResult> {
  return invoke("curseforge_install_command", { packId, file, folder, title, icon });
}

export function curseforgeModpackFiles(projectId: number): Promise<CursePackFile[]> {
  return invoke("curseforge_modpack_files_command", { projectId });
}

export function curseforgeProjectDetail(projectId: number): Promise<CurseProjectDetail> {
  return invoke("curseforge_project_detail_command", { projectId });
}

export function curseforgeInstallPack(projectId: number, fileId: number): Promise<PackDescriptor> {
  return invoke("curseforge_install_pack_command", { projectId, fileId });
}

export function curseforgeKeyConfigured(): Promise<boolean> {
  return invoke("curseforge_key_configured_command");
}

export function launchGame(
  packId: string,
  ramGb: number,
  session: UserSession,
  width: number,
  height: number,
  serverAddress: string | null = null
): Promise<void> {
  return invoke("launch_game_command", { packId, ramGb, session, width, height, serverAddress });
}

export function stopGame(): Promise<void> {
  return invoke("stop_game_command");
}

export function pingServer(address: string, port: number | null = null): Promise<ServerStatus> {
  return invoke("ping_server_command", { address, port });
}

export function listScreenshots(packId: string | null = null): Promise<ScreenshotList> {
  return invoke("list_screenshots_command", { packId });
}

export function analyzeDuplicates(packId: string | null = null): Promise<DuplicatesResult> {
  return invoke("analyze_duplicates_command", { packId });
}

export function listSavedServers(packId: string | null = null): Promise<SavedServersList> {
  return invoke("list_servers_command", { packId });
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

export function analyzeCrash(packId: string): Promise<CrashAnalysis> {
  return invoke("analyze_crash_command", { packId });
}

export function onCrashAnalyzed(
  cb: (a: CrashAnalysis) => void
): Promise<UnlistenFn> {
  return listen<CrashAnalysis>("crash-analyzed", (event) => cb(event.payload));
}

export function openPackDir(packId: string): Promise<void> {
  return invoke("open_pack_dir", { packId });
}

/** Путь к папке сборки (для UI-подсказок; без открытия проводника). */
export function getPackDir(packId: string): Promise<string> {
  return invoke("get_pack_dir_command", { packId });
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

/** Версия Java, заданная для сборки (мажорный номер; null — авто). */
export function getPackJava(packId: string): Promise<number | null> {
  return invoke("get_pack_java_command", { packId });
}

/** Задаёт версию Java для сборки (null — авто по версии Minecraft). */
export function setPackJava(packId: string, version: number | null): Promise<void> {
  return invoke("set_pack_java_command", { packId, version });
}

export function ensureJava(major?: number): Promise<string> {
  return invoke("ensure_java_command", { major: major ?? null });
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

/** Удаляет файлы/папки игры (моды/ресурспаки/шейдеры/миры) по именам. */
export function deleteGameFiles(packId: string, folder: GameFolder, names: string[]): Promise<number> {
  return invoke("delete_game_files_command", { packId, folder, names });
}

export function packLocked(packId: string | null | undefined): Promise<boolean> {
  return invoke("pack_locked_command", { packId: packId || null });
}

export function setPackLocked(packId: string, locked: boolean): Promise<boolean> {
  return invoke("set_pack_locked_command", { packId, locked });
}

export function getSkin(uuid: string): Promise<string | null> {
  return invoke("get_skin_command", { uuid });
}

export function getLocalSkin(): Promise<SkinInfo> {
  return invoke("get_local_skin_command");
}

export function setLocalSkin(path: string, model: string, nick: string): Promise<SkinInfo> {
  return invoke("set_local_skin_command", { path, model, nick });
}

export function clearLocalSkin(nick: string): Promise<void> {
  return invoke("clear_local_skin_command", { nick });
}

export function skinApiUrl(): Promise<string> {
  return invoke("skin_api_url_command");
}

export function setBoosty(
  packId: string,
  token: string,
  refreshToken?: string,
  deviceId?: string,
  tokenExpiresAt?: number
): Promise<LicenseInfo> {
  return invoke("set_boosty_command", {
    packId,
    token,
    refreshToken,
    deviceId,
    tokenExpiresAt,
  });
}

export function licenseStatus(packId: string): Promise<LicenseInfo> {
  return invoke("license_status_command", { packId });
}

export function clearLicense(packId: string): Promise<void> {
  return invoke("clear_license_command", { packId });
}

/** Глобальная привязка Boosty (на весь лаунчер, не для конкретной сборки). */
export function setBoostyGlobal(
  token: string,
  refreshToken?: string,
  deviceId?: string,
  tokenExpiresAt?: number
): Promise<void> {
  return invoke("set_global_boosty_command", {
    token,
    refreshToken,
    deviceId,
    tokenExpiresAt,
  });
}

/** Привязан ли глобальный аккаунт Boosty. */
export function boostyGlobalLinked(): Promise<boolean> {
  return invoke("global_boosty_linked_command");
}

/** Отвязывает глобальный аккаунт Boosty. */
export function clearBoostyGlobal(): Promise<void> {
  return invoke("clear_global_boosty_command");
}

/** Открывает отдельное окно входа Boosty (захват токенов из localStorage). */
export function boostyLoginBegin(): Promise<void> {
  return invoke("boosty_login_begin_command");
}

/** Опрашивает окно входа Boosty: вернул токены — забирает и закрывает окно. */
export function boostyPoll(): Promise<BoostyAuth | null> {
  return invoke("boosty_poll_command");
}

/** Закрывает окно входа Boosty (отмена). */
export function boostyLoginCancel(): Promise<void> {
  return invoke("boosty_login_cancel_command");
}

export function onJavaProgress(cb: (e: LaunchLogEntry) => void): Promise<UnlistenFn> {
  return listen<LaunchLogEntry>("launch-log", (event) => cb(event.payload));
}

export function setDiscordRp(enabled: boolean): Promise<void> {
  return invoke("set_discord_rp_command", { enabled });
}

export function setWarnCustomMods(enabled: boolean): Promise<void> {
  return invoke("set_warn_custom_mods_command", { enabled });
}

export function listAccounts(): Promise<Accounts> {
  return invoke("list_accounts_command");
}

export function switchAccount(id: string): Promise<UserSession> {
  return invoke("switch_account_command", { id });
}

export function removeAccount(id: string): Promise<UserSession | null> {
  return invoke("remove_account_command", { id });
}

export function setLocale(locale: string): Promise<void> {
  return invoke("set_locale_command", { locale });
}

export function getNews(locale: string): Promise<NewsItem[]> {
  return invoke("get_news_command", { locale });
}

/** Стриминг новостей: батч по мере подгрузки источников (свежие сверху). */
export function onNewsChunk(cb: (items: NewsItem[]) => void): Promise<UnlistenFn> {
  return listen<NewsItem[]>("news-chunk", (event) => cb(event.payload));
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

// ==== Modrinth: свои сборки, моды, обновления ====

export interface ModrinthSearchOpts {
  /** Категории (facets categories). */
  categories?: string[];
  /** Загрузчики/платформы (facets loaders). */
  loaders?: string[];
  /** Канал версии (facets version_type): release | beta | alpha. */
  versionType?: string;
  /** Версии игры (facets versions). */
  versions?: string[];
  /** Окружение: "client" | "server". */
  environment?: string;
  /** Сортировка: relevance | downloads | follows | newest | updated. */
  index?: string;
}

/** Тип проекта Modrinth, по которому идёт поиск. */
export type ModrinthSearchKind = "mod" | "modpack" | "resourcepack" | "shaderpack" | "datapack";

export type { CurseCategory, CurseFile, CurseInstallResult, CursePackFile, CurseProjectDetail, CurseSearchHit } from "./types";

/** Папка игры, куда ставится файл с Modrinth. */
export type ModrinthInstallFolder = "mods" | "resourcepacks" | "shaderpacks" | "datapacks";

export function modrinthSearch(
  query: string,
  kind: ModrinthSearchKind,
  limit?: number,
  opts?: ModrinthSearchOpts,
  offset?: number
): Promise<ModrinthProject[]> {
  return invoke("modrinth_search_command", {
    query,
    kind,
    limit,
    offset,
    filters: {
      categories: opts?.categories,
      loaders: opts?.loaders,
      version_type: opts?.versionType,
      versions: opts?.versions,
      environment: opts?.environment,
      index: opts?.index,
    },
  });
}

export function modrinthTags(kind: ModrinthSearchKind): Promise<ModrinthTags> {
  return invoke("modrinth_tags_command", { kind });
}

export function modrinthProjectVersions(
  projectId: string,
  gameVersion?: string,
  loader?: string
): Promise<ModrinthVersion[]> {
  return invoke("modrinth_project_versions_command", {
    projectId,
    gameVersion,
    loader,
  });
}

export function modrinthVersion(versionId: string): Promise<ModrinthVersion> {
  return invoke("modrinth_version_command", { versionId });
}

export function modrinthProject(projectId: string): Promise<ModrinthProject> {
  return invoke("modrinth_project_command", { projectId });
}

export function setPackIcon(packId: string, path: string): Promise<void> {
  return invoke("set_pack_icon_command", { packId, path });
}

export function fetchPackIcon(packId: string): Promise<boolean> {
  return invoke("fetch_pack_icon_command", { packId });
}

export function setPackBanner(packId: string, path: string): Promise<void> {
  return invoke("set_pack_banner_command", { packId, path });
}

export function setPackUrl(packId: string, url: string): Promise<void> {
  return invoke("set_pack_url_command", { packId, url });
}

export function setPackName(packId: string, name: string): Promise<void> {
  return invoke("set_pack_name_command", { packId, name });
}

export function modrinthInstallMod(
  packId: string,
  versionId: string,
  folder: ModrinthInstallFolder,
  world?: string
): Promise<TrackedMod> {
  return invoke("modrinth_install_mod_command", { packId, versionId, folder, world });
}

export function modrinthCheckUpdates(packId: string): Promise<ModUpdate[]> {
  return invoke("modrinth_check_updates_command", { packId });
}

export function modrinthUpdateMod(
  packId: string,
  fileName: string
): Promise<TrackedMod> {
  return invoke("modrinth_update_mod_command", { packId, fileName });
}

export function installedModSha1(packId: string, projectId: string): Promise<string | null> {
  return invoke("installed_mod_sha1_command", { packId, projectId });
}

export function modrinthRemoveMod(packId: string, fileName: string): Promise<void> {
  return invoke("modrinth_remove_mod_command", { packId, fileName });
}

export function modrinthInstallPack(versionId: string): Promise<PackDescriptor> {
  return invoke("modrinth_install_pack_command", { versionId });
}

export function createLocalPack(
  name: string,
  minecraftVersion: string,
  loader: string | null,
  icon: string | null = null,
  banner: string | null = null,
  loaderVersion: string | null = null
): Promise<PackDescriptor> {
  return invoke("create_local_pack_command", {
    name,
    minecraftVersion,
    loader,
    loaderVersion,
    icon,
    banner,
  });
}

/** Доступные версии модлоадера под версию Minecraft (для выбора при создании своей сборки). */
export function localLoaderVersions(
  loader: string,
  minecraftVersion: string
): Promise<string[]> {
  return invoke("local_loader_versions_command", { loader, minecraftVersion });
}

/** Релизные и снапшот-версии Minecraft (для выбора при создании своей сборки). */
export function minecraftVersions(): Promise<McVersionInfo[]> {
  return invoke("minecraft_versions_command");
}

/** Меняет версию Minecraft / загрузчик / версию загрузчика у активной версии своей сборки. */
export function editPackVersion(
  packId: string,
  minecraftVersion: string,
  loader: string,
  loaderVersion: string
): Promise<void> {
  return invoke("edit_pack_version_command", {
    packId,
    minecraftVersion,
    loader,
    loaderVersion,
  });
}

/** Экспортирует сборку (указанной версии, с выбором файлов) в архив заданного формата. */
export function exportPack(
  packId: string,
  versionId: string,
  format: "mrpack" | "curseforge",
  destPath: string,
  include: string[],
  name: string,
  version: string
): Promise<void> {
  return invoke("export_pack_command", {
    packId,
    versionId,
    format,
    destPath,
    include,
    name,
    version,
  });
}

/** Папки и файлы папки игры выбранной версии (плоский список дерева) для выбора при экспорте. */
export function exportSourceList(packId: string, versionId: string): Promise<ExportSourceItem[]> {
  return invoke("export_list_command", { packId, versionId });
}

/** Экспортирует «авторскую» сборку: zip с .mrpack, pack.json, servers/socials/theme.json и README. */
export function exportAuthorPack(
  packId: string,
  versionId: string,
  destPath: string,
  include: string[],
  config: AuthorPackConfig
): Promise<void> {
  return invoke("export_author_pack_command", {
    packId,
    versionId,
    destPath,
    include,
    config,
  });
}

/** Загружает .mrpack на бэкенд Mono (multipart; бэкенд проверит мат и перешлёт файл на storage). */
export function uploadPack(
  accessToken: string,
  filePath: string,
  name: string,
  description: string,
  version?: string,
  changelog?: string,
  minRamMb?: number | null,
  boostyBlog?: string | null,
  meta?: Record<string, unknown> | null,
  iconUrl?: string | null
): Promise<MonoPackPublic> {
  return invoke("upload_pack_command", {
    accessToken,
    filePath,
    name,
    description,
    version: version ?? "",
    changelog: changelog ?? "",
    minRamMb: minRamMb ?? null,
    boostyBlog: boostyBlog ?? null,
    meta: meta ?? null,
    iconUrl: iconUrl ?? null,
  });
}

// ==== Панель автора: управление сборками на бэкенде Mono ====

/** Каталог сборок Mono (без авторизации). */
export function packCatalog(): Promise<PackCatalog[]> {
  return invoke("pack_catalog_command");
}

/** Сборки, автором которых является текущий пользователь. */
export function packMine(accessToken: string): Promise<PackCatalog[]> {
  return invoke("pack_mine_command", { accessToken });
}

/** Новости Mono (глобальные и по сборкам, свежие сверху). */
export function packNews(): Promise<PackNewsPublic[]> {
  return invoke("pack_news_command");
}

/** Деталь сборки Mono; пустой accessToken — запрос без авторизации. */
export function packDetail(
  accessToken: string,
  id: string
): Promise<PackDetail> {
  return invoke("pack_detail_command", { accessToken, id });
}

/** Частичное обновление описания сборки (PUT /packs/{id}). */
export function packUpdate(
  accessToken: string,
  id: string,
  body: UpdatePackRequest
): Promise<PackDetail> {
  return invoke("pack_update_command", { accessToken, id, body });
}

/** Удаляет сборку с бэкенда Mono и storage (DELETE /packs/{id}). */
export function packDelete(accessToken: string, id: string): Promise<void> {
  return invoke("pack_delete_command", { accessToken, id });
}

/** Загружает новую версию .mrpack для сборки. */
export function packAddVersion(
  accessToken: string,
  id: string,
  filePath: string,
  version: string,
  changelog: string
): Promise<PackVersionPublic> {
  return invoke("pack_add_version_command", {
    accessToken,
    id,
    filePath,
    version,
    changelog,
  });
}

/** Удаляет версию сборки. */
export function packDeleteVersion(
  accessToken: string,
  id: string,
  versionId: string
): Promise<void> {
  return invoke("pack_delete_version_command", {
    accessToken,
    id,
    versionId,
  });
}

/** Загружает скриншот сборки на storage (возвращает обновлённую meta). */
export function packUploadScreenshot(
  accessToken: string,
  id: string,
  filePath: string,
  caption?: string
): Promise<Record<string, unknown>> {
  return invoke("pack_upload_screenshot_command", { accessToken, id, filePath, caption: caption ?? "" });
}

/** Удаляет скриншот сборки по индексу (возвращает обновлённую meta). */
/** Резолвит id сборки на бэкенде по URL файла (null если не найдена). */
export function packIdByUrl(url: string): Promise<string | null> {
  return invoke("pack_id_by_url_command", { url });
}

export function packDeleteScreenshot(
  accessToken: string,
  id: string,
  index: number
): Promise<Record<string, unknown>> {
  return invoke("pack_delete_screenshot_command", { accessToken, id, index });
}

/** Добавляет новость к сборке. */
export function packAddNews(
  accessToken: string,
  id: string,
  kind: string,
  title: string,
  body: string
): Promise<PackNewsPublic> {
  return invoke("pack_add_news_command", {
    accessToken,
    id,
    kind,
    title,
    body,
  });
}

/** Удаляет новость сборки. */
export function packDeleteNews(
  accessToken: string,
  id: string,
  newsId: string
): Promise<void> {
  return invoke("pack_delete_news_command", {
    accessToken,
    id,
    newsId,
  });
}

/** Оценивает сборку (value: 1 или -1). */
export function packRate(
  accessToken: string,
  id: string,
  value: number
): Promise<{ likes: number; dislikes: number; rating: number; myRating: number | null }> {
  return invoke("pack_rate_command", { accessToken, id, value });
}

// ==== Комментарии ====

import type {
  CommentPublic,
  CommentWithReplies,
  ProfilePublic,
  ProfileDetail,
  ScanResult,
  CollaboratorPublic,
  AdminUser,
  AdminPack,
  AdminComment,
  AdminCreateUser,
} from "./types";

export function monoListComments(packId: string): Promise<CommentWithReplies[]> {
  return invoke("mono_list_comments_command", { packId });
}

export function monoCreateComment(
  accessToken: string,
  packId: string,
  body: string,
  parentId?: string
): Promise<CommentPublic> {
  return invoke("mono_create_comment_command", { accessToken, packId, body, parentId: parentId ?? null });
}

export function monoUpdateComment(
  accessToken: string,
  packId: string,
  commentId: string,
  body: string
): Promise<CommentPublic> {
  return invoke("mono_update_comment_command", { accessToken, packId, commentId, body });
}

export function monoDeleteComment(
  accessToken: string,
  packId: string,
  commentId: string
): Promise<void> {
  return invoke("mono_delete_comment_command", { accessToken, packId, commentId });
}

export function monoRateComment(
  accessToken: string,
  packId: string,
  commentId: string,
  value: number
): Promise<{ likes: number; dislikes: number; myRating: number | null }> {
  return invoke("mono_rate_comment_command", { accessToken, packId, commentId, value });
}

// ==== Профили ====

export function monoGetProfile(userId: string): Promise<ProfilePublic> {
  return invoke("mono_get_profile_command", { userId });
}

export function monoGetProfileFull(userId: string): Promise<ProfileDetail> {
  return invoke("mono_get_profile_full_command", { userId });
}

export function monoUpdateProfile(
  accessToken: string,
  bio?: string,
  avatarUrl?: string
): Promise<ProfilePublic> {
  return invoke("mono_update_profile_command", { accessToken, bio: bio ?? null, avatarUrl: avatarUrl ?? null });
}

// ==== Сканер модов ====

export function monoScanMod(accessToken: string, filePath: string): Promise<ScanResult> {
  return invoke("mono_scan_mod_command", { accessToken, filePath });
}

export function monoCheckHash(sha256: string): Promise<ScanResult> {
  return invoke("mono_check_hash_command", { sha256 });
}

// ==== Соавторы ====

export function monoListCollaborators(
  accessToken: string,
  packId: string
): Promise<CollaboratorPublic[]> {
  return invoke("mono_list_collaborators_command", { accessToken, packId });
}

export function monoAddCollaborator(
  accessToken: string,
  packId: string,
  username: string,
  permEditMeta: boolean,
  permManageVersions: boolean,
  permManageNews: boolean
): Promise<CollaboratorPublic> {
  return invoke("mono_add_collaborator_command", {
    accessToken, packId, username,
    permEditMeta, permManageVersions, permManageNews,
  });
}

export function monoUpdateCollaborator(
  accessToken: string,
  packId: string,
  collabId: string,
  permEditMeta?: boolean,
  permManageVersions?: boolean,
  permManageNews?: boolean
): Promise<CollaboratorPublic> {
  return invoke("mono_update_collaborator_command", {
    accessToken, packId, collabId,
    permEditMeta: permEditMeta ?? null,
    permManageVersions: permManageVersions ?? null,
    permManageNews: permManageNews ?? null,
  });
}

export function monoRemoveCollaborator(
  accessToken: string,
  packId: string,
  collabId: string
): Promise<void> {
  return invoke("mono_remove_collaborator_command", { accessToken, packId, collabId });
}

// ==== Админ ====

export function monoAdminListUsers(accessToken: string): Promise<AdminUser[]> {
  return invoke("mono_admin_list_users_command", { accessToken });
}

export function monoAdminListPacks(accessToken: string): Promise<AdminPack[]> {
  return invoke("mono_admin_list_packs_command", { accessToken });
}

export function monoAdminListComments(accessToken: string): Promise<AdminComment[]> {
  return invoke("mono_admin_list_comments_command", { accessToken });
}

export function monoAdminCreateUser(
  accessToken: string,
  payload: AdminCreateUser
): Promise<AdminUser> {
  return invoke("mono_admin_create_user_command", { accessToken, payload });
}

export function monoAdminBanUser(
  accessToken: string,
  userId: string,
  reason?: string
): Promise<void> {
  return invoke("mono_admin_ban_user_command", { accessToken, userId, reason: reason ?? null });
}

export function monoAdminUnbanUser(accessToken: string, userId: string): Promise<void> {
  return invoke("mono_admin_unban_user_command", { accessToken, userId });
}

export function monoAdminDeleteUser(accessToken: string, userId: string): Promise<void> {
  return invoke("mono_admin_delete_user_command", { accessToken, userId });
}

export function monoAdminDeletePack(accessToken: string, packId: string): Promise<void> {
  return invoke("mono_admin_delete_pack_command", { accessToken, packId });
}

export function monoAdminDeleteComment(accessToken: string, commentId: string): Promise<void> {
  return invoke("mono_admin_delete_comment_command", { accessToken, commentId });
}

export function monoAdminSetRole(
  accessToken: string,
  userId: string,
  role: string
): Promise<void> {
  return invoke("mono_admin_set_role_command", { accessToken, userId, role });
}

// ==== Auth v2 ====

export function monoForgotPassword(email: string): Promise<void> {
  return invoke("mono_forgot_password_command", { email });
}

export function monoResetPassword(token: string, password: string): Promise<void> {
  return invoke("mono_reset_password_command", { token, password });
}

export function monoConfirmEmail(accessToken: string): Promise<void> {
  return invoke("mono_confirm_email_command", { accessToken });
}