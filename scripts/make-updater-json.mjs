#!/usr/bin/env node
// Генерирует latest.json для tauri-plugin-updater из ассетов релиза GitHub.
// Запуск (в CI): TAG=launcher-v0.1.4 node scripts/make-updater-json.mjs <папка> <выход.json>
// В папке должны лежать бандлы релиза и их .sig-файлы (gh release download).
import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const dir = process.argv[2];
const out = process.argv[3] ?? "latest.json";
const tag = process.env.TAG ?? "";
const repo = "n1orio/mono-launcher";
if (!dir || !tag) {
  console.error("Usage: TAG=launcher-vX.Y.Z node scripts/make-updater-json.mjs <dir> [out.json]");
  process.exit(1);
}

const version = tag.replace(/^launcher-v/, "");

/** Секция `## [<ver>]` из markdown-файла ченджлога (как в release-notes.mjs). */
function sectionFromMarkdown(md, ver) {
  const header = `## [${ver}]`;
  const i = md.indexOf(header);
  if (i === -1) return null;
  const headerEnd = md.indexOf("\n", i);
  const rest = headerEnd === -1 ? "" : md.slice(headerEnd + 1);
  const nextIdx = rest.indexOf("\n## [");
  const body = (nextIdx === -1 ? rest : rest.slice(0, nextIdx)).trim();
  return body || null;
}

// Локализованные ноты новостей: `notes_localized[locale]`. Источник — файлы
// CHANGELOG.<locale>.md в репозитории (en = основной CHANGELOG.md). Если файла
// или секции для версии нет — язык просто не попадёт в манифест, и лаунчер
// использует английский фолбэк.
const LOCALES = ["en", "ru", "uk", "de", "be"];
const rootDir = fileURLToPath(new URL("../", import.meta.url));
const notesLocalized = {};
for (const loc of LOCALES) {
  const file = loc === "en" ? "CHANGELOG.md" : `CHANGELOG.${loc}.md`;
  let md = "";
  try {
    md = readFileSync(join(rootDir, file), "utf8");
  } catch {
    continue; // файла нет — пропускаем локализацию
  }
  const sec = sectionFromMarkdown(md, version);
  if (sec) notesLocalized[loc] = sec;
}

const files = readdirSync(dir);

// target key -> regex имени бандла
const PATTERNS = {
  "windows-x86_64": /x64-setup\.exe$/,
  "windows-aarch64": /arm64-setup\.exe$/,
  "windows-i686": /x86-setup\.exe$/,
  "linux-x86_64": /_amd64\.AppImage$/,
  "linux-arm64": /_aarch64\.AppImage$/,
  "darwin-aarch64": /\.app\.tar\.gz$/,
};

function platformEntry(key, regex) {
  const name = files.find((f) => regex.test(f));
  if (!name) return null;
  const sigPath = join(dir, `${name}.sig`);
  if (!existsSync(sigPath)) {
    console.warn(`Нет подписи для ${name} — пропускаю ${key}`);
    return null;
  }
  const url = `https://github.com/${repo}/releases/download/${encodeURIComponent(tag)}/${encodeURIComponent(name)}`;
  return {
    url,
    signature: readFileSync(sigPath, "utf8").trim(),
  };
}

const platforms = {};
for (const [key, regex] of Object.entries(PATTERNS)) {
  const entry = platformEntry(key, regex);
  if (entry) platforms[key] = entry;
}

if (Object.keys(platforms).length === 0) {
  console.error("Не нашёл ни одного бандла с подписью в папке:", files);
  process.exit(1);
}

const latest = {
  version,
  notes: process.env.BODY ?? "",
  notes_localized: notesLocalized,
  pub_date: new Date().toISOString(),
  platforms,
};

writeFileSync(out, JSON.stringify(latest, null, 2));
console.log(`latest.json: ${Object.keys(platforms).join(", ")} -> ${out}`);