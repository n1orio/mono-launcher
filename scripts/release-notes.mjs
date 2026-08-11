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
const esc = ver.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
const re = new RegExp(`## \\[${esc}\\][^\\n]*\\n([\\s\\S]*?)(?=^## \\[|\\z)`, "m");
const m = md.match(re);
if (!m) {
  console.error(`Секция [${ver}] не найдена в CHANGELOG.md`);
  process.exit(1);
}

process.stdout.write(
  [intro, "", `## Изменения в ${ver}`, "", m[1].trim()].filter(Boolean).join("\n")
);