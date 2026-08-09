export interface PackDescriptor {
  id: string;
  name: string;
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