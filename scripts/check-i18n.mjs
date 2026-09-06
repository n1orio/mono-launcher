// Проверка синхронности локалей. Запуск: node scripts/check-i18n.mjs
// Падает (exit 1) если:
//   1. в каком-то locales/*.json не хватает ключей из базового locales/ru.json;
//   2. в коде есть статический t("literal"), которого нет в ru.json (опечатка).
// Только предупреждает (exit 0): лишние ключи, неполные plural-группы.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const localesDir = path.join(root, "locales");
const files = fs.readdirSync(localesDir).filter((f) => f.endsWith(".json"));

const dicts = {};
for (const f of files) {
  dicts[f.replace(/\.json$/, "")] = JSON.parse(fs.readFileSync(path.join(localesDir, f), "utf8"));
}
const base = dicts.ru;
if (!base) {
  console.error("FATAL: locales/ru.json not found");
  process.exit(1);
}
const baseKeys = new Set(Object.keys(base).filter((k) => k !== "__meta__"));

let errors = 0;
const warn = (m) => console.log(`WARN  ${m}`);
const fail = (m) => { console.error(`FAIL  ${m}`); errors++; };

// 1. Полнота локалей против ru.
for (const [code, dict] of Object.entries(dicts)) {
  // t() умеет только плоские ключи — вложенные объекты (кроме __meta__) молча ломают перевод.
  for (const [k, v] of Object.entries(dict)) {
    if (k !== "__meta__" && v !== null && typeof v === "object") {
      fail(`[${code}] nested object (use flat dotted keys): ${k}`);
    }
  }
  if (code === "ru") continue;
  const keys = new Set(Object.keys(dict).filter((k) => k !== "__meta__"));
  const missing = [...baseKeys].filter((k) => !keys.has(k));
  const extra = [...keys].filter((k) => !baseKeys.has(k));
  for (const k of missing) fail(`[${code}] missing key: ${k}`);
  for (const k of extra) warn(`[${code}] extra key (not in ru): ${k}`);
  // Неполные plural-группы: есть .one, но нет .other.
  for (const k of [...baseKeys].filter((x) => x.endsWith(".one"))) {
    const stem = k.slice(0, -4);
    if (!keys.has(`${stem}.other`)) warn(`[${code}] plural group without .other: ${stem}.*`);
  }
}

// 2. Статические t("...") в коде должны существовать в ru.json.
const SRC_DIRS = ["components", "composables", "lib", "pages"];
const SKIP_DIRS = new Set(["node_modules", ".nuxt", ".output", "dist"]);
const used = new Set();
function scan(dir) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) {
      if (!SKIP_DIRS.has(e.name)) scan(p);
    } else if (/\.(vue|ts|js)$/.test(e.name)) {
      const src = fs.readFileSync(p, "utf8");
      // t("key") / t('key') — только строковые литералы без конкатенации.
      for (const m of src.matchAll(/\bt\(\s*(["'])((?:(?!\1).)+)\1\s*[,)]/g)) {
        used.add(m[2]);
      }
    }
  }
}
for (const d of SRC_DIRS) {
  const p = path.join(root, d);
  if (fs.existsSync(p)) scan(p);
}
for (const k of [...used].sort()) {
  if (!baseKeys.has(k)) fail(`unknown i18n key in code: t("${k}")`);
}

console.log(`check-i18n: ${Object.keys(dicts).length} locales, ${baseKeys.size} base keys, ${used.size} static t() usages, ${errors} error(s)`);
process.exit(errors ? 1 : 0);
