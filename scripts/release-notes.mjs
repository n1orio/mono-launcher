// Формирует тело GitHub-релиза из секции CHANGELOG.md для версии.
// Используется в CI (update-manifest джоба) и локально:
//   node scripts/release-notes.mjs "<ver>" ["<первая строка тела>"]
import fs from "node:fs";

const ver = (process.argv[2] ?? "").trim();
const intro = (process.argv[3] ?? "").trim();
if (!ver) {
  console.error("usage: node scripts/release-notes.mjs <ver> [intro]");
  process.exit(2);
}

const md = fs.readFileSync(new URL("../CHANGELOG.md", import.meta.url), "utf8");
const header = `## [${ver}]`;
const lineIdx = md.indexOf(header);
if (lineIdx === -1) {
  console.error(`Секция [${ver}] не найдена в CHANGELOG.md`);
  process.exit(1);
}
// конец строки заголовка — начало тела секции
const headerEnd = md.indexOf("\n", lineIdx);
const rest = headerEnd === -1 ? "" : md.slice(headerEnd + 1);
// тело заканчивается перед следующим заголовком `## [` (или в конце файла)
const nextIdx = rest.indexOf("\n## [");
const body = (nextIdx === -1 ? rest : rest.slice(0, nextIdx)).trim();

process.stdout.write(
  [intro, "", `## Изменения в ${ver}`, "", body].filter(Boolean).join("\n")
);