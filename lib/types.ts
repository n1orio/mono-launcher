export interface PackDescriptor {
  id: string;
  name: string;
  /** Прямой URL на .mrpack файл сборки. */
  url: string;
  builtin: boolean;
  /** "remote" — сборка из GitHub Releases; "local" — своя сборка или с Modrinth. */
  kind: "remote" | "local";
  /** Владелец GitHub-репозитория сборки (если это github-сборка). */
  author: string | null;
  /** Ник блога на Boosty: задан → сборка платная (подписка обязательна). */
  boostyBlog: string | null;
  /** Минимальная оперативка для запуска (МБ), из pack.json автора. */
  minRam: number | null;
  /** Локальная иконка сборки (путь к packs/<id>/icon.png), если есть. */
  icon: string | null;
  /** Локальный баннер сборки (путь к packs/<id>/banner.png), если есть. */
  banner: string | null;
}

/** Карточка проекта Modrinth (мод или модпак). */
export interface ModrinthProject {
  projectId: string;
  slug: string;
  projectType: string;
  title: string;
  description: string;
  author: string;
  iconUrl: string | null;
  downloads: number;
  categories: string[];
  latestVersion: string | null;
  /** Полное описание (markdown) — только у полного проекта. */
  body: string | null;
  /** Галерея скриншотов — только у полного проекта. */
  gallery: ModrinthGalleryItem[];
}

/** Скриншот из галереи проекта Modrinth. */
export interface ModrinthGalleryItem {
  url: string;
  title: string | null;
  description: string | null;
  featured: boolean | null;
}

/** Теги Modrinth для фильтров поиска (загрузчики, категории, версии игры). */
export interface ModrinthTags {
  loaders: string[];
  categories: string[];
  versions: string[];
}

/** Файл версии Modrinth (jar мода или .mrpack модпака). */
export interface ModrinthFile {
  hashes: Record<string, string>;
  url: string;
  filename: string;
  primary: boolean | null;
  size: number;
}

/** Версия проекта Modrinth. */
export interface ModrinthVersion {
  id: string;
  projectId: string;
  name: string;
  versionNumber: string;
  versionType: string;
  gameVersions: string[];
  loaders: string[];
  datePublished: string;
  changelog: string | null;
  files: ModrinthFile[];
  dependencies: ModrinthDependency[];
}

export interface ModrinthDependency {
  projectId: string | null;
  versionId: string | null;
  dependencyType: string;
}

/** Установленный из Modrinth файл (трекинг обновлений). */
export interface TrackedMod {
  fileName: string;
  /** Папка игры: mods / resourcepacks / shaderpacks / datapacks. */
  folder: string;
  /** Для датапаков — мир, куда установлен файл. */
  world: string | null;
  versionId: string;
  projectId: string;
  sha1: string;
  gameVersion: string;
  loader: string;
}

/** Элемент верхнего уровня папки игры, который можно выбрать при экспорте. */
export interface ExportSourceItem {
  path: string;
  isDir: boolean;
  size: number;
  defaultIncluded: boolean;
}

/** Конфигурация «авторской» сборки для экспорта (мастер автора). */
export interface AuthorServer {
  name: string;
  ip: string;
  port?: number | null;
  desc?: string | null;
}

export interface AuthorSocial {
  name: string;
  url: string;
  color?: string | null;
}

export interface AuthorTheme {
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

export interface AuthorPackConfig {
  name: string;
  author: string;
  description?: string | null;
  boostyBlog?: string | null;
  minRam?: number | null;
  servers: AuthorServer[];
  socials: AuthorSocial[];
  theme?: AuthorTheme | null;
}

/** Отдельный профиль Mono (не игровой аккаунт — лежит поверх аккаунтов). */
export interface MonoProfile {
  username: string;
  uuid: string;
  access_token: string;
}

/** Сборка, загруженная на бэкенд Mono (и разложенная на storage-сервер). */
export interface MonoPackPublic {
  id: string;
  file: string;
  name: string;
  description: string;
  url: string;
  size: number;
  sha1: string;
  sha512: string;
}

/** Запись в каталоге сборок Mono (GET /packs, GET /packs/mine). */
export interface PackCatalog {
  id: string;
  name: string;
  description: string;
  author_user_id: string | null;
  author_name: string | null;
  icon_url: string | null;
  min_ram_mb: number | null;
  boosty_blog: string | null;
  meta: Record<string, unknown> | null;
  version: string | null;
  url: string;
  size: number;
  versions_count: number;
  likes: number;
  dislikes: number;
  rating: number;
  created_at: string;
}

/** Публичная версия сборки Mono (GET /packs/{id}/versions). */
export interface PackVersionPublic {
  id: string;
  version: string;
  changelog: string;
  file: string;
  url: string;
  size: number;
  sha1: string;
  sha512: string;
  created_at: string;
}

/** Запись новостей Mono (глобальная или сборки). */
export interface PackNewsPublic {
  id: string;
  pack_id: string | null;
  kind: string;
  title: string;
  body: string;
  created_at: string;
}

/** Деталь сборки Mono (GET /packs/{id}, PUT /packs/{id}). */
export interface PackDetail {
  id: string;
  name: string;
  description: string;
  author_user_id: string | null;
  author_name: string | null;
  icon_url: string | null;
  min_ram_mb: number | null;
  boosty_blog: string | null;
  meta: Record<string, unknown> | null;
  url: string;
  size: number;
  likes: number;
  dislikes: number;
  created_at: string;
  versions: PackVersionPublic[];
  news: PackNewsPublic[];
  my_rating: number | null;
}

/** Частичное обновление описания сборки (PUT /packs/{id}, COALESCE). */
export interface UpdatePackRequest {
  name?: string;
  description?: string;
  min_ram_mb?: number | null;
  boosty_blog?: string | null;
  icon_url?: string | null;
  meta?: Record<string, unknown> | null;
}

/** Доступное обновление установленного из Modrinth файла. */
export interface ModUpdate {
  fileName: string;
  /** Папка игры, где лежит файл. */
  folder: string;
  newVersion: ModrinthVersion;
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
  /** Название тарифа активной подписки (null — неизвестно/не подписан). */
  tier: string | null;
  /** Тарифы, требуемые сборке (пусто — подходит любой). */
  requiredTiers: string[];
}

/** Токены входа Boosty, захваченные окном входа (для автопродления). */
export interface BoostyAuth {
  accessToken: string;
  refreshToken: string;
  deviceId: string;
  tokenExpiresAt: number;
}

/** Версия Minecraft для выбора при создании своей сборки. */
export interface McVersionInfo {
  id: string;
  /** "release" | "snapshot" */
  kind: string;
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
  loader_version: string | null;
  pack_name: string | null;
  session: UserSession | null;
  mrpack_url: string;
  active_version: string | null;
  active_source_tag: string | null;
  installed_versions: string[];
  discord_rp_enabled: boolean;
  /** Показывать ли плашку предупреждения о кастомных модах. */
  warn_custom_mods: boolean;
  /** Суммарное время игры в этой сборке (секунды). */
  playtime_seconds: number;
  /** Суммарное время игры во всех сборках (секунды). */
  total_playtime_seconds: number;
  /** Сколько сборок когда-либо запускалось. */
  played_packs: number;
  /** Файлы активной версии, скачанные не с доверенных CDN (кастомные моды). */
  custom_mods: CustomFile[];
}

/** Файл сборки, скачанный не с Modrinth/CurseForge CDN. */
export interface CustomFile {
  path: string;
  url: string;
  sha256?: string;
  safe?: boolean;
  scan_result?: string;
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

export interface InstalledVersion {
  version_id: string;
  name: string;
  source_tag: string | null;
  total_seconds: number;
}

export interface VersionsInfo {
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
  /** Помечен как фатальный маркер краша (подсветка в консоли). */
  fatal?: boolean;
}

/** Фаза 1 device code flow (Microsoft и Ely.by) — код и страница подтверждения. */
export interface MsDeviceCodeInfo {
  user_code: string;
  verification_uri: string;
  device_code: string;
  interval: number;
  expires_in: number;
  qr_svg: string;
}

/** Результат поиска на CurseForge (моды/ресурспаки/шейдеры/сборки). */
export interface CurseSearchHit {
  projectId: number;
  name: string;
  /** Полное HTML описание из проекта CurseForge (если есть в API). */
  description: string | null;
  summary: string;
  author: string;
  downloadCount: number;
  fileExt: string;
  iconUrl?: string;
}

/** Полное описание проекта CurseForge (деталка сборки: описание/скриншоты). */
export interface CurseProjectDetail {
  projectId: number;
  name: string;
  slug: string;
  summary: string;
  /// Полное описание из проекта CurseForge (если есть).
  description?: string,
  /** Описание из файла проекта CurseForge (если project endpoint вернул null). */
  curseFileDescription?: string,
  isShortDescription?: boolean;
  author: string;
  downloadCount: number;
  iconUrl?: string;
  screenshots: string[];
  categories: string[];
  websiteUrl: string;
}

/** Категория класса проектов CurseForge (для фильтра поиска). */
export interface CurseCategory {
  id: number;
  name: string;
}

/** Файл сборки CurseForge (для выбора версии). */
export interface CursePackFile {
  fileId: number;
  fileName: string;
  displayName: string;
  gameVersion: string;
  fileDate: string;
}

/** Файл CurseForge, готовый к установке (ответ curseforge_latest_file). */
export interface CurseFile {
  fileId: number;
  projectId: number;
  fileName: string;
  downloadUrl: string;
  sha1: string;
  gameVersion: string;
  /** Required-зависимости файла (для автодокачки). */
  dependencies?: Array<{ modId?: number; relationType: number }>;
}

/** Результат установки файла CurseForge. */
export interface CurseInstallResult {
  name: string;
  depsInstalled: number;
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
  /** unix-секунды последнего изменения файла. */
  modified: number;
  modrinthUrl?: string | null;
  /** слаг проекта Modrinth (из .mono-modrinth.json), если файл установлен вручную с Modrinth. */
  modrinthProjectId?: string | null;
  /** id версии Modrinth (из .mono-modrinth.json) — для показа версии в списке. */
  modrinthVersionId?: string | null;
  /** ID проекта CurseForge (для меты/иконки), если файл установлен вручную с CurseForge. */
  curseforgeProjectId?: number | null;
  /** Название проекта CurseForge (из трекера) — показывается без API-запроса. */
  curseforgeTitle?: string | null;
  /** URL логотипа проекта CurseForge (из трекера). */
  curseforgeIcon?: string | null;
}

export interface GameFileIcon {
  name: string;
  data: string | null;
}

/** Сервер из servers.dat игрока (camelCase из Rust). */
export interface SavedServer {
  name: string;
  address: string;
}

/** Ответ list_screenshots_command. */
export interface ScreenshotList {
  installed: boolean;
  screenshots: ScreenshotInfo[];
}

/** Один скриншот: путь + время создания/изменения (epoch, сек). */
export interface ScreenshotInfo {
  path: string;
  modified: number;
}

/** Один файл-дубликат. */
export interface DuplicateFile {
  path: string;
  folder: string;
  name: string;
}

/** Группа одинаковых по содержимому файлов; size_bytes — размер одного. */
export interface DuplicateGroup {
  files: DuplicateFile[];
  size_bytes: number;
}

/** Ответ analyze_duplicates_command. */
export interface DuplicatesResult {
  groups: DuplicateGroup[];
  wasted_bytes: number;
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
  /** Никнеймы игроков (players.sample) — может быть пусто, если сервер их не шлёт. */
  players: string[];
  latencyMs: number | null;
}

/** Сохранённый аккаунт (несколько аккаунтов, accounts.json). */
export interface AccountEntry {
  id: string;
  username: string;
  uuid: string;
  access_token: string;
  user_type: string;
}

/** Список аккаунтов + активный (ответ list_accounts_command). */
export interface Accounts {
  active: string | null;
  list: AccountEntry[];
}

export interface GameExited {
  success: boolean;
  code: number;
}

/** Подозреваемый мод из анализа краш-репорта. */
export interface SuspectedMod {
  name: string;
  file: string;
  package: string;
}

/** Результат анализа краш-артефактов (analyze_crash_command / событие crash-analyzed). */
export interface CrashAnalysis {
  hasCause: boolean;
  file: string;
  kind: string;
  exception: string;
  description: string;
  javaHint: number | null;
  suspected: SuspectedMod[];
}

// ==== Комментарии ====

export interface MonoUserPublic {
  id: string;
  username: string;
  displayName: string | null;
}

export interface CommentPublic {
  id: string;
  packId: string;
  userId: string;
  user: MonoUserPublic;
  parentId: string | null;
  body: string;
  likes: number;
  dislikes: number;
  myRating: number | null;
  createdAt: string;
  updatedAt: string;
}

export interface CommentWithReplies extends CommentPublic {
  replies: CommentWithReplies[];
}

// ==== Профили ====

export interface ProfilePublic {
  user: MonoUserPublic;
  bio: string;
  avatarUrl: string | null;
  packsCount: number;
  commentsCount: number;
  joinedAt: string;
}

export interface UserPackSummary {
  id: string;
  name: string;
  description: string;
  iconUrl: string | null;
  version: string | null;
  likes: number;
  dislikes: number;
  versionsCount: number;
  createdAt: string;
}

export interface UserCommentSummary {
  id: string;
  packId: string;
  packName: string;
  body: string;
  createdAt: string;
}

export interface ProfileDetail {
  profile: ProfilePublic;
  packs: UserPackSummary[];
  comments: UserCommentSummary[];
}

// ==== Сканер модов ====

export interface ScanResult {
  id: string;
  fileName: string;
  sha256: string;
  safe: boolean;
  scanResult: string;
  dangerousClasses: string | null;
  cached: boolean;
}

// ==== Соавторы ====

export interface CollaboratorPublic {
  id: string;
  user: MonoUserPublic;
  permEditMeta: boolean;
  permManageVersions: boolean;
  permManageNews: boolean;
}

// ==== Админ ====

export interface AdminUser {
  id: string;
  username: string;
  displayName: string | null;
  email: string | null;
  emailConfirmed: boolean;
  role: string;
  banned: boolean;
  banReason: string | null;
  createdAt: string;
}

export interface AdminPack {
  id: string;
  name: string;
  description: string;
  authorUserId: string | null;
  authorName: string | null;
  likes: number;
  dislikes: number;
  versionsCount: number;
  createdAt: string;
}

export interface AdminComment {
  id: string;
  packId: string;
  packName: string;
  authorName: string;
  body: string;
  createdAt: string;
}

export interface AdminCreateUser {
  username: string;
  password: string;
  email: string | null;
  role: string | null;
}