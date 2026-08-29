import type { Ref } from 'vue';
import type { CrashAnalysis, PackDescriptor } from '~/lib/types';
import { isTauri, openExternal } from '~/lib/bridge';

export interface UseCrashAnalysisDeps {
  crashAnalysis: Ref<CrashAnalysis | null>;
  activePack: Ref<PackDescriptor | null>;
  ram: Ref<number>;
  t: (key: string, params?: Record<string, unknown>) => string;
  notify: (text: string, type?: string) => void;
}

/** Формирует заголовок и сообщение для отображения краш-анализа. */
function crashView(a: CrashAnalysis, ram: number, t: (key: string, params?: Record<string, unknown>) => string) {
  const k = `crash.kind.${a.kind}`;
  if (a.kind === "oom") {
    return { title: t(`${k}.title`), msg: t(`${k}.msg`, { ram }) };
  }
  if (a.kind === "javaVersion") {
    return {
      title: t(`${k}.title`),
      msg: a.javaHint ? t(`${k}.msg`, { java: a.javaHint }) : t(`${k}.msgAuto`),
    };
  }
  return { title: t(`${k}.title`), msg: t(`${k}.msg`) };
}

/** Копирует текстовое резюме краш-анализа в буфер. */
async function copyCrash(a: CrashAnalysis, ram: number, t: (key: string, params?: Record<string, unknown>) => string) {
  const v = crashView(a, ram, t);
  const lines = [
    v.title,
    v.msg,
    "",
    a.exception ? `Exception: ${a.exception}` : "",
    a.description ? `Description: ${a.description}` : "",
    a.suspected.length
      ? `Suspected: ${a.suspected.map((m) => `${m.name} (${m.file})`).join(", ")}`
      : "",
    `File: ${a.file}`,
  ].filter(Boolean);
  try {
    await navigator.clipboard.writeText(lines.join("\n"));
  } catch {
    const ta = document.createElement("textarea");
    ta.value = lines.join("\n");
    document.body.appendChild(ta);
    ta.select();
    document.execCommand("copy");
    ta.remove();
  }
}

/** Открывает GitHub Issues сборки с предзаполненным краш-анализом. */
function openIssue(
  pack: PackDescriptor | null,
  a: CrashAnalysis,
  ram: number,
  t: (key: string, params?: Record<string, unknown>) => string,
) {
  const rest = (pack?.url || "").replace(/^https?:\/\/github\.com\//, "");
  const [owner, repo] = rest.split("/");
  if (!owner || !repo || owner === "USER" || repo === "REPO") return;
  const v = crashView(a, ram, t);
  const body = [
    v.title,
    v.msg,
    "",
    a.exception ? `Exception: ${a.exception}` : "",
    a.description ? `Description: ${a.description}` : "",
    a.suspected.length
      ? `Suspected: ${a.suspected.map((m) => `${m.name} (${m.file})`).join(", ")}`
      : "",
    `File: ${a.file}`,
  ]
    .filter(Boolean)
    .join("\n");
  const url = `https://github.com/${owner}/${repo}/issues/new?title=${encodeURIComponent(t("reportPack.title", { name: pack?.name ?? "?" }))}&body=${encodeURIComponent(body)}`;
  if (isTauri()) openExternal(url).catch(() => window.open(url, "_blank"));
  else window.open(url, "_blank");
}

export function useCrashAnalysis(deps: UseCrashAnalysisDeps) {
  const { crashAnalysis, activePack, ram, t, notify } = deps;

  /** Формирует заголовок и сообщение для отображения краш-анализа. */
  function crashViewLocal(a: CrashAnalysis) {
    return crashView(a, ram.value, t);
  }

  /** Копирует текстовое резюме краш-анализа в буфер. */
  async function copyCrashAnalysis() {
    const a = crashAnalysis.value;
    if (!a) return;
    await copyCrash(a, ram.value, t);
  }

  /** Открывает GitHub Issues сборки с предзаполненным краш-анализом. */
  function openCrashIssue() {
    const a = crashAnalysis.value;
    if (!a) return;
    openIssue(activePack.value, a, ram.value, t);
  }

  return { crashView: crashViewLocal, copyCrashAnalysis, openCrashIssue };
}
