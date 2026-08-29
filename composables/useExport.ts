import { computed, ref, watch } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import {
  isTauri,
  openExternal,
  exportPack as exportPackFn,
  exportSourceList,
  exportAuthorPack,
  uploadPack,
} from "~/lib/bridge";
import type {
  AppStatus,
  AuthorPackConfig,
  AuthorServer,
  AuthorSocial,
  AuthorTheme,
  ExportSourceItem,
  MonoProfile,
  PackDescriptor,
} from "~/lib/types";
import { themeFromAccent, normalizeHex } from "~/lib/misc";

type TranslateFn = (key: string, params?: Record<string, unknown>) => string;

export interface UseExportDeps {
  packId: ReturnType<typeof ref<string | null>>;
  activePack: ReturnType<typeof ref<PackDescriptor | null>>;
  packs: ReturnType<typeof ref<PackDescriptor[]>>;
  status: ReturnType<typeof ref<AppStatus | null>>;
  monoProfile: ReturnType<typeof ref<MonoProfile | null>>;
  notify: (text: string, type?: string) => void;
  t: TranslateFn;
  importAuthorPack: (
    filePath: string,
    name: string,
    description: string,
    version: string,
    changelog: string,
    opts: {
      minRamMb: number | null;
      boostyBlog: string | null;
      meta: Record<string, unknown>;
      iconUrl: string | null;
    },
  ) => Promise<boolean>;
}

export function useExport(deps: UseExportDeps) {
  const {
    packId,
    activePack,
    packs: _packs,
    status: _status,
    monoProfile,
    notify,
    t,
    importAuthorPack,
  } = deps;

  /* ================= Общее состояние экспорта ================= */

  const exportOpen = ref(false);
  const exportFormat = ref<"mrpack" | "curseforge" | "author">("mrpack");
  const exportLoading = ref(false);
  const exportItems = ref<ExportSourceItem[]>([]);
  const exportSelected = ref(new Set<string>());
  const exportExpanded = ref(new Set<string>());
  const exportVersionNum = ref("1.0.0");
  const exportName = ref("");
  const exportBusy = ref(false);
  const exportUpload = ref(false);
  const exportAllChecked = computed(
    () =>
      exportItems.value.length > 0 &&
      exportItems.value.every((it) => exportSelected.value.has(it.path)),
  );

  /** Дети узла дерева экспорта (по префиксу пути). */
  function exportChildrenOf(parent: string): ExportSourceItem[] {
    const pref = parent ? `${parent}/` : "";
    return exportItems.value
      .filter(
        (it) =>
          it.path.startsWith(pref) &&
          it.path.slice(pref.length).indexOf("/") === -1,
      )
      .sort(
        (a, b) => Number(b.isDir) - Number(a.isDir) || a.path.localeCompare(b.path),
      );
  }

  /** Все потомки узла (включая сам узел). */
  function exportDescendantsOf(path: string): string[] {
    const pref = path ? `${path}/` : "";
    return exportItems.value
      .filter((it) => it.path === path || it.path.startsWith(pref))
      .map((it) => it.path);
  }

  /** Видимые строки дерева (preorder, с учётом развёрнутых папок). */
  const exportVisibleRows = computed(() => {
    const rows: { it: ExportSourceItem; depth: number }[] = [];
    const walk = (parent: string, depth: number) => {
      for (const it of exportChildrenOf(parent)) {
        rows.push({ it, depth });
        if (it.isDir && exportExpanded.value.has(it.path))
          walk(it.path, depth + 1);
      }
    };
    walk("", 0);
    return rows;
  });

  /** Множество выбранных потомков узла (для неопределённого состояния чекбокса папки). */
  function exportSelectedCount(path: string): {
    selected: number;
    total: number;
  } {
    const kids = exportChildrenOf(path);
    if (!kids.length) {
      return exportSelected.value.has(path)
        ? { selected: 1, total: 1 }
        : { selected: 0, total: 1 };
    }
    let selected = 0;
    let total = 0;
    for (const k of kids) {
      const [s, t] = k.isDir
        ? [
            exportSelectedCount(k.path).selected,
            exportSelectedCount(k.path).total,
          ]
        : [Number(exportSelected.value.has(k.path)), 1];
      selected += s;
      total += t;
    }
    return { selected, total };
  }

  /** Открывает диалог выбора папок/файлов, версии и имени перед экспортом сборки. */
  async function openExport(format: "mrpack" | "curseforge") {
    if (exportBusy.value || !packId.value || !isTauri()) return;
    exportFormat.value = format;
    exportOpen.value = true;
    exportName.value = activePack?.value?.name || "pack";
    exportExpanded.value = new Set();
    await loadExportList();
  }

  async function loadExportList() {
    if (!packId.value) return;
    exportLoading.value = true;
    try {
      const items = await exportSourceList(packId.value, "");
      exportItems.value = items;
      const sel = new Set<string>();
      for (const it of items) if (it.defaultIncluded) sel.add(it.path);
      exportSelected.value = sel;
    } catch (e) {
      notify(t("pack.exportListErr", { e }));
    } finally {
      exportLoading.value = false;
    }
  }

  /** Развернуть/свернуть папку в дереве. */
  function toggleExportExpand(path: string) {
    const ex = new Set(exportExpanded.value);
    if (ex.has(path)) ex.delete(path);
    else ex.add(path);
    exportExpanded.value = ex;
  }

  function toggleExport(path: string) {
    const it = exportItems.value.find((x) => x.path === path);
    if (!it) return;
    const sel = new Set(exportSelected.value);
    if (it.isDir) {
      const all = exportDescendantsOf(path);
      if (all.every((p) => sel.has(p))) for (const p of all) sel.delete(p);
      else for (const p of all) sel.add(p);
    } else {
      if (sel.has(path)) sel.delete(path);
      else sel.add(path);
    }
    exportSelected.value = sel;
  }

  function toggleExportAll() {
    if (exportAllChecked.value) exportSelected.value = new Set();
    else
      exportSelected.value = new Set(exportItems.value.map((it) => it.path));
  }

  /** Подтверждает выбор, показывает диалог сохранения и запускает экспорт. */
  async function doExport() {
    if (exportBusy.value || !packId.value || !isTauri()) return;
    const format = exportFormat.value === "author" ? "mrpack" : exportFormat.value;
    const include = [...exportSelected.value];
    const name =
      exportName.value.trim() || activePack?.value?.name || "pack";
    const ext = format === "mrpack" ? "mrpack" : "zip";
    const dest = await save({
      defaultPath: `${name}.${ext}`,
      filters:
        format === "mrpack"
          ? [{ name: "MRPack", extensions: ["mrpack"] }]
          : [{ name: "ZIP", extensions: ["zip"] }],
    });
    if (!dest) return;
    exportBusy.value = true;
    try {
      await exportPackFn(
        packId.value,
        "",
        format,
        dest,
        include,
        name,
        exportVersionNum.value.trim() || "1.0.0",
      );
      if (exportUpload.value && monoProfile.value) {
        try {
          const pack = await uploadPack(
            monoProfile.value.access_token,
            dest,
            name,
            "",
          );
          notify(t("pack.uploadDone"), "success");
          void openExternal(pack.url);
        } catch (e) {
          notify(t("pack.uploadErr", { e }), "error");
        }
      } else {
        notify(t("pack.exportDone"), "success");
      }
      exportOpen.value = false;
    } catch (e) {
      notify(t("pack.exportErr", { e }));
    } finally {
      exportBusy.value = false;
      exportUpload.value = false;
    }
  }

  /* ================= Экспорт «авторской» сборки ================= */

  const authorName = ref("");
  const authorAuthor = ref("");
  const authorDesc = ref("");
  const authorBoosty = ref("");
  const authorIcon = ref("");
  const authorBanner = ref("");
  const authorMinRam = ref(false);
  const authorMinRamMb = ref<number | null>(null);
  const authorServers = ref<AuthorServer[]>([
    { name: "", ip: "", port: null, desc: "" },
  ]);
  const authorSocials = ref<AuthorSocial[]>([
    { name: "", url: "", color: "" },
  ]);
  const authorTheme = ref<AuthorTheme>({});
  const authorAccent = ref("");
  const AUTHOR_MAX_SERVERS = 5;
  const AUTHOR_MAX_SOCIALS = 4;

  const authorThemeFields: Array<{ key: keyof AuthorTheme; cap: string }> = [
    { key: "accent", cap: "pack.exportThemeAccent" },
    { key: "accentStrong", cap: "pack.exportThemeAccentStrong" },
    { key: "accentHover", cap: "pack.exportThemeAccentHover" },
    { key: "accentDeep", cap: "pack.exportThemeAccentDeep" },
    { key: "bg", cap: "pack.exportThemeBg" },
    { key: "panel", cap: "pack.exportThemePanel" },
    { key: "input", cap: "pack.exportThemeInput" },
    { key: "border", cap: "pack.exportThemeBorder" },
    { key: "tx", cap: "pack.exportThemeTx" },
    { key: "txStrong", cap: "pack.exportThemeTxStrong" },
    { key: "txMuted", cap: "pack.exportThemeTxMuted" },
  ];

  function openAuthorExport() {
    if (exportBusy.value || !packId.value || !isTauri()) return;
    authorImportMode.value = false;
    exportFormat.value = "author";
    authorName.value = activePack?.value?.name || "pack";
    authorAuthor.value = activePack?.value?.author || "";
    authorDesc.value = "";
    authorBoosty.value = activePack?.value?.boostyBlog || "";
    const mr = activePack?.value?.minRam ?? null;
    authorMinRam.value = !!mr;
    authorMinRamMb.value = mr ? Math.round(mr / 1024) : null;
    authorServers.value = [{ name: "", ip: "", port: null, desc: "" }];
    authorSocials.value = [{ name: "", url: "", color: "" }];
    authorTheme.value = {};
    authorAccent.value = "";
    exportOpen.value = true;
    exportExpanded.value = new Set();
    void loadExportList();
  }

  function addAuthorServer() {
    if (authorServers.value.length < AUTHOR_MAX_SERVERS)
      authorServers.value.push({ name: "", ip: "", port: null, desc: "" });
  }

  function removeAuthorServer(i: number) {
    if (authorServers.value.length > 1) authorServers.value.splice(i, 1);
    else authorServers.value = [{ name: "", ip: "", port: null, desc: "" }];
  }

  function addAuthorSocial() {
    if (authorSocials.value.length < AUTHOR_MAX_SOCIALS)
      authorSocials.value.push({ name: "", url: "", color: "" });
  }

  function removeAuthorSocial(i: number) {
    if (authorSocials.value.length > 1) authorSocials.value.splice(i, 1);
    else authorSocials.value = [{ name: "", url: "", color: "" }];
  }

  /** По одному введённому цвету автозаполняет акцентную тему. */
  function applyAuthorAccent() {
    const th = themeFromAccent(authorAccent.value);
    if (th) authorTheme.value = th;
  }

  /** Событие нативного color-picker: ставит валидный hex и применяет тему. */
  function applyAuthorAccentColor(ev: Event) {
    const v = (ev.target as HTMLInputElement).value;
    if (!v) return;
    authorAccent.value = v;
    applyAuthorAccent();
  }

  function themePreview(hex?: string | null): string {
    return normalizeHex(hex ?? "") ?? "#000";
  }

  function authorConfig(): AuthorPackConfig {
    return {
      name: authorName.value.trim() || activePack?.value?.name || "pack",
      author: authorAuthor.value.trim(),
      description: authorDesc.value.trim() ? authorDesc.value.trim() : null,
      boostyBlog: authorBoosty.value.trim()
        ? authorBoosty.value.trim()
        : null,
      minRam: authorMinRam.value ? (authorMinRamMb.value ?? null) : null,
      servers: authorServers.value
        .filter((s) => s.name.trim() || s.ip.trim())
        .map((s) => ({
          name: s.name.trim(),
          ip: s.ip.trim(),
          port: s.port ?? null,
          desc: s.desc?.trim() ? s.desc.trim() : null,
        })),
      socials: authorSocials.value
        .filter((s) => s.name.trim() && s.url.trim())
        .map((s) => ({
          name: s.name.trim(),
          url: s.url.trim(),
          color: s.color?.trim() ? s.color.trim() : null,
        })),
      theme: authorThemeFields.some(
        (f) => (authorTheme.value[f.key] ?? "").trim(),
      )
        ? (Object.fromEntries(
            authorThemeFields
              .filter((f) => (authorTheme.value[f.key] ?? "").trim())
              .map((f) => [f.key, authorTheme.value[f.key]!.trim()]),
          ) as AuthorTheme)
        : null,
    };
  }

  async function doAuthorExport() {
    if (exportBusy.value || !packId.value || !isTauri()) return;
    const include = [...exportSelected.value];
    if (include.length === 0) {
      notify(t("pack.exportEmpty"), "info");
      return;
    }
    const cfg = authorConfig();
    const dest = await save({
      defaultPath: `${cfg.name.replace(/[^a-zа-яё0-9-]+/gi, "-").toLowerCase() || "pack"}.zip`,
      filters: [{ name: "ZIP", extensions: ["zip"] }],
    });
    if (!dest) return;
    exportBusy.value = true;
    try {
      await exportAuthorPack(packId.value, "", dest, include, cfg);
      notify(t("pack.exportAuthorDone"), "success");
      exportOpen.value = false;
    } catch (e) {
      notify(t("pack.exportAuthorErr", { e }));
    } finally {
      exportBusy.value = false;
    }
  }

  /* ================= Импорт авторской сборки ================= */

  const authorImportFile = ref("");
  const authorImportVersion = ref("");
  const authorImportChangelog = ref("");
  const authorImportMode = ref(false);

  async function pickAuthorImportFile() {
    const { open: openDialog } = await import("@tauri-apps/plugin-dialog");
    const picked = await openDialog({
      filters: [{ name: "Modrinth Pack", extensions: ["mrpack", "zip"] }],
    });
    if (typeof picked === "string") {
      authorImportFile.value = picked;
      const base =
        picked.split("/").pop()?.replace(/\.(mrpack|zip)$/i, "") ?? "";
      authorName.value = base;
      authorAuthor.value = "";
      authorDesc.value = "";
      authorBoosty.value = "";
      authorIcon.value = "";
      authorBanner.value = "";
      authorMinRam.value = false;
      authorMinRamMb.value = null;
      authorServers.value = [{ name: "", ip: "", port: null, desc: "" }];
      authorSocials.value = [{ name: "", url: "", color: "" }];
      authorTheme.value = {};
      authorAccent.value = "";
      authorImportVersion.value = "";
      authorImportChangelog.value = "";
      authorImportMode.value = true;
      exportFormat.value = "author";
      exportOpen.value = true;
    }
  }

  async function doAuthorImport() {
    if (!authorImportFile.value || exportBusy.value) return;
    const cfg = authorConfig();
    const meta: Record<string, unknown> = {
      theme: cfg.theme,
      servers: cfg.servers,
      socials: cfg.socials,
    };
    if (authorBanner.value.trim()) meta.banner = authorBanner.value.trim();
    const ok = await importAuthorPack(
      authorImportFile.value,
      cfg.name,
      cfg.description ?? "",
      authorImportVersion.value,
      authorImportChangelog.value,
      {
        minRamMb: cfg.minRam ?? null,
        boostyBlog: cfg.boostyBlog ?? null,
        meta,
        iconUrl: authorIcon.value.trim() ? authorIcon.value.trim() : null,
      },
    );
    if (ok) exportOpen.value = false;
  }

  return {
    /* --- export dialog state --- */
    exportOpen,
    exportFormat,
    exportLoading,
    exportItems,
    exportSelected,
    exportExpanded,
    exportVersionNum,
    exportName,
    exportBusy,
    exportUpload,
    exportAllChecked,
    exportVisibleRows,

    /* --- tree helpers --- */
    exportChildrenOf,
    exportDescendantsOf,
    exportSelectedCount,
    toggleExportExpand,
    toggleExport,
    toggleExportAll,

    /* --- export actions --- */
    openExport,
    loadExportList,
    doExport,

    /* --- author export form state --- */
    authorName,
    authorAuthor,
    authorDesc,
    authorBoosty,
    authorIcon,
    authorBanner,
    authorMinRam,
    authorMinRamMb,
    authorServers,
    authorSocials,
    authorTheme,
    authorAccent,
    authorThemeFields,
    AUTHOR_MAX_SERVERS,
    AUTHOR_MAX_SOCIALS,

    /* --- author export helpers --- */
    openAuthorExport,
    authorConfig,
    doAuthorExport,
    addAuthorServer,
    removeAuthorServer,
    addAuthorSocial,
    removeAuthorSocial,
    applyAuthorAccent,
    applyAuthorAccentColor,
    themePreview,

    /* --- author import state --- */
    authorImportFile,
    authorImportVersion,
    authorImportChangelog,
    authorImportMode,

    /* --- author import actions --- */
    pickAuthorImportFile,
    doAuthorImport,
  };
}
