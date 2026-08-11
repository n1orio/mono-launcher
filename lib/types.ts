export interface PackDescriptor {
  id: string;
  name: string;
  /** Прямой URL на .mrpack файл сборки. */
  url: string;
  builtin: boolean;
  /** Владелец GitHub-репозитория сборки (если это github-сборка). */
  author: string | null;
  /** Ник блога на Boosty: задан → сборка платная (подписка обязательна). */
  boostyBlog: string | null;
  /** Минимальная оперативка для запуска (МБ), из pack.json автора. */
  minRam: number | null;
}

/** Запись каталога сборок (catalog.json в репозитории лаунчера). */
export interface CatalogEntry {
  name: string;
  url: string;
  description: string | null;
  author: string | null;
  boostyBlog: string | null;
  minRam: number | null;
  tags: string[];
}

/** Статус лицензии сборки (привязка Boosty). */
export interface LicenseInfo {
  /** Ник блога издателя, на который проверялась подписка. */
  blog: string;
  /** Есть активная подписка на блог (по последней проверке). */
  subscribed: boolean;
  /** Дата окончания подписки по Boosty (unix-секунды). */
  expiresAt: number | null;
  /** До какого момента действует локальная льгота без сети (unix-секунды). */
  cachedUntil: number;
}

export interface UserSession {
  username: string;
  uuid: string;
  access_token: string;
  user_type: string;
}

export interface AppStatus {
  installed: boolean;
  minecraft_version: string | null;
  loader: string | null;
  pack_name: string | null;
  session: UserSession | null;
  mrpack_url: string;
  active_version: string | null;
  active_source_tag: string | null;
  installed_versions: string[];
  discord_rp_enabled: boolean;
  /** Файлы активной версии, скачанные не с доверенных CDN (кастомные моды). */
  custom_mods: CustomFile[];
}

/** Файл сборки, скачанный не с Modrinth/CurseForge CDN. */
export interface CustomFile {
  path: string;
  url: string;
}

export interface PackInfo {
  name: string;
  summary: string | null;
  version_id: string;
  minecraft_version: string;
  loader: string;
  loader_version: string | null;
  file_count: number;
}

export interface UpdateInfo {
  current_version: string | null;
  latest_version: string | null;
  has_update: boolean;
}

export interface DownloadProgress {
  phase: string;
  current: number;
  total: number;
  file_index: number;
  file_total: number;
  current_file: string;
  bytes_per_sec: number;
}

export interface GhVersion {
  tag: string;
  name: string;
  url: string;
  prerelease: boolean;
  published_at: string | null;
  body: string;
  assets: string[];
}

export interface InstalledVersion {
  version_id: string;
  name: string;
  source_tag: string | null;
  total_seconds: number;
}

export interface VersionsInfo {
  github: GhVersion[];
  installed: InstalledVersion[];
  active: string | null;
}

export interface SystemInfo {
  total_ram_gb: number;
  available_ram_gb: number;
}

export interface LaunchLogEntry {
  stream: "out" | "err" | "sys";
  line: string;
}

export interface MsDeviceCodeInfo {
  user_code: string;
  verification_uri: string;
  device_code: string;
  interval: number;
  expires_in: number;
  qr_svg: string;
}

export interface JavaInfo {
  path: string;
  label: string;
  version: string;
  arch: "64-бит" | "32-бит" | "недоступна";
  is_bundled: boolean;
  selected: boolean;
}

export interface VerifyResult {
  checked: number;
  ok: number;
  broken: string[];
}

export interface NewsItem {
  kind: "update" | "post";
  pack_id: string;
  pack_name: string;
  title: string;
  body: string;
  url: string;
  tag: string | null;
  category: string | null;
  date: string | null;
}

export interface GameFileEntry {
  name: string;
  displayName: string;
  kind: "file" | "dir";
  enabled: boolean;
  sizeBytes: number;
  modrinthUrl?: string | null;
}

export interface GameFileIcon {
  name: string;
  data: string | null;
}

/** Сервер сборки из servers.json в репозитории. */
export interface PackServer {
  name: string;
  ip: string;
  port: number | null;
  desc: string | null;
}

/** Сервер из servers.dat игрока (camelCase из Rust). */
export interface SavedServer {
  name: string;
  address: string;
}

/** Ответ list_screenshots_command. */
export interface ScreenshotList {
  installed: boolean;
  screenshots: string[];
}

/** Ответ list_servers_command. */
export interface SavedServersList {
  installed: boolean;
  servers: SavedServer[];
}

/** Локальный скин игрока (ответ get_local_skin_command). */
export interface SkinInfo {
  has_skin: boolean;
  model: string;
  path: string | null;
}

/** Статус Minecraft-сервера (ответ ping_server_command). */
export interface ServerStatus {
  online: boolean;
  version: string | null;
  motd: string | null;
  playersOnline: number | null;
  playersMax: number | null;
  latencyMs: number | null;
}

/** Соцсеть сборки из socials.json в корне репозитория. */
export interface PackSocial {
  name: string;
  url: string;
  /** Цвет кнопки (#rrggbb) — задаётся автором в socials.json; иначе акцент темы. */
  color: string | null;
}

/** Тема лаунчера из theme.json сборки (все поля — hex-цвета `#rrggbb`). */
export interface PackTheme {
  bg?: string | null;
  panel?: string | null;
  input?: string | null;
  border?: string | null;
  tx?: string | null;
  txStrong?: string | null;
  txMuted?: string | null;
  accent?: string | null;
  accentStrong?: string | null;
  accentHover?: string | null;
  accentDeep?: string | null;
}

/** Контент репозитория сборки: звёзды, сервера, соцсети. */
export interface PackRepoContent {
  stars: number | null;
  servers: PackServer[];
  socials: PackSocial[];
  theme: PackTheme | null;
}

export interface GameExited {
  success: boolean;
  code: number;
}