import type { Ref } from "vue";
import { openExternal } from "~/lib/bridge";
import type { PackDescriptor } from "~/lib/types";
import { useI18n } from "./useI18n";

export interface UseDevHelpersDeps {
  activePack: Ref<PackDescriptor | null>;
  notify: (text: string, type?: string) => void;
}

const EXAMPLE_PACK_REPO = "https://github.com/n1orio/mono-pack-example";

const examplePackJson = `{
  "name": "Example Pack",
  "id": "example-pack",
  "version": "1.0.0",
  "description": "Минимальная сборка-пример"
}`;

const deepLinkExample =
  "https://n1orio.github.io/mono-launcher/?url=" +
  encodeURIComponent("https://github.com/n1orio/mono-pack-example") +
  "&name=" +
  encodeURIComponent("Example Pack");

const SITE_SHARE_URL = "http://2.27.200.74";

const ICON_TAG =
  "M1 7.775V2.75C1 1.784 1.784 1 2.75 1h5.025c.464 0 .91.184 1.238.513l6.25 6.25a1.75 1.75 0 0 1 0 2.474l-5.026 5.026a1.75 1.75 0 0 1-2.474 0l-6.25-6.25A1.752 1.752 0 0 1 1 7.775Zm1.5 0c0 .066.026.13.073.177l6.25 6.25a.25.25 0 0 0 .354 0l5.025-5.025a.25.25 0 0 0 0-.354l-6.25-6.25a.25.25 0 0 0-.177-.073H2.75a.25.25 0 0 0-.25.25ZM6 5a1 1 0 1 0 0 2 1 1 0 0 0 0-2Z";
const ICON_PACKAGE =
  "M8.878.392a1.75 1.75 0 0 0-1.756 0l-6.065 3.685A1.75 1.75 0 0 0 .25 5.607v4.786c0 .649.353 1.247.925 1.562l6.065 3.653a1.75 1.75 0 0 0 1.72 0l6.065-3.653a1.75 1.75 0 0 0 .925-1.562V5.607a1.75 1.75 0 0 0-.807-1.53ZM5.5 2.8h5l.972.972H4.528ZM3.747 2.2h2.109l-.972.972H2.775Zm.903 3.547 3.35 2.034 3.35-2.034.14 6.994H4.51Zm-1.564.913-.972.972-.43.005L2 4.814Zm10.828-.972.143 6.988-.43-.005-.972-.972L11.25 5.841Z";
const ICON_PAINT =
  "M.75 7.5a.75.75 0 0 1 .75-.75h13a.75.75 0 0 1 .75.75v6a1.75 1.75 0 0 1-1.75 1.75H2.5A1.75 1.75 0 0 1 .75 13.5ZM2.5 8.25v5.25h11V8.25ZM4 5.25a.75.75 0 0 1-.75-.75V2.75A1.75 1.75 0 0 1 5 1h2.5A1.75 1.75 0 0 1 9.25 2.75L11 4.5h.75V7H6.25a.75.75 0 0 1-.53-.22L4.22 5.28a.75.75 0 0 1-.22-.53Z";
const ICON_SUN =
  "M8 5.5a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5ZM2.5 8a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 0 1.5h-1.5A.75.75 0 0 1 2.5 8Zm7.75 0a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 0 1.5h-1.5A.75.75 0 0 1 10.25 8Zm-5.53 2.72a.75.75 0 0 1 0 1.061l-1.06 1.061a.75.75 0 0 1-1.061-1.061l1.06-1.061a.75.75 0 0 1 1.061 0Zm5.657 0a.75.75 0 0 1 1.061 0l1.06.53a.75.75 0 1 1-.53 1.404l-1.06-.53a.75.75 0 0 1-.53-.531.75.75 0 0 1 0-.53.75.75 0 0 1 .53-.53Zm-5.657 4.803a.75.75 0 0 1 0-1.061l1.06-1.061a.75.75 0 0 1 1.061 1.061l-1.06 1.061a.75.75 0 0 1-1.061 0Zm5.657 0a.75.75 0 0 1 1.061-1.061l1.06 1.061a.75.75 0 0 1-1.06 1.061Z";
const ICON_FOLDER =
  "M.513 1.513A1.75 1.75 0 0 1 1.75 1h3.5c.55 0 1.07.26 1.4.7l.9 1.2a.25.25 0 0 0 .2.1H13a1 1 0 0 1 1 1v.5H2.75a.75.75 0 0 0 0 1.5h11.978a1 1 0 0 1 .994 1.117L15 13.25A1.75 1.75 0 0 1 13.25 15H1.75A1.75 1.75 0 0 1 0 13.25V2.75c0-.464.184-.91.513-1.237Z";
const ICON_TERMINAL =
  "M0 2.75C0 1.784.784 1 1.75 1h12.5c.966 0 1.75.784 1.75 1.75v10.5A1.75 1.75 0 0 1 14.25 15H1.75A1.75 1.75 0 0 1 0 13.25Zm1.75-.25a.25.25 0 0 0-.25.25v10.5c0 .138.112.25.25.25h12.5a.25.25 0 0 0 .25-.25V2.75a.25.25 0 0 0-.25-.25ZM7.25 8a.75.75 0 0 1-.22.53l-2.25 2.25a.75.75 0 0 1-1.06-1.06L5.44 8 3.72 6.28a.75.75 0 1 1 1.06-1.06l2.25 2.25c.141.14.22.331.22.53Zm1.5 1.5a.75.75 0 0 1 0-1.5h3a.75.75 0 0 1 0 1.5Z";
const ICON_IMAGE =
  "M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-1 1v.878A2.25 2.25 0 1 1 2 13.378V2.5ZM5.5 1a1.5 1.5 0 1 0 0 3 1.5 1.5 0 0 0 0-3Zm5.912.5a.75.75 0 0 1 .232 1.136l-3.75 4.5a.75.75 0 0 1-1.136.029L4.22 4.441a.75.75 0 0 0-1.014.023L.22 7.341A.75.75 0 0 1-.252 6.22l3.47-3.47a2.25 2.25 0 0 1 3.043-.07l1.714 1.53 3.15-3.781a.75.75 0 0 1 1.087-.071Z";
const ICON_DUP =
  "M5 1h7.75A2.25 2.25 0 0 1 15 3.25v7.75a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2V6.75a.75.75 0 0 1 .75-.75H9a2 2 0 0 1-2-2V3.25A2.25 2.25 0 0 1 5 1Zm3.25 5H7V3.25a.25.25 0 0 1 .5 0V4.5h1.5a.5.5 0 0 1 0 1h-.25a.75.75 0 0 0 0 1.5ZM2.5 4.5h.25v3h4V9H2.5A.5.5 0 0 1 2 8.5v-3.5a.5.5 0 0 1 .5-.5Z";
const ICON_SERVER =
  "M3 1.5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-9a2 2 0 0 0-2-2ZM1.5 4.5H14.5v1.5H1.5ZM1.5 8H14.5v1.25H1.5Zm0 3.25H7v1.5H1.5A.5.5 0 0 1 1 12.25v-1ZM8.5 12.75v-1.5h6v1.5A.5.5 0 0 1 14.5 13h-5a1 1 0 0 1-1-.25ZM2 5.75a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Zm3 0a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0ZM2 9.25a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Zm3 0a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Z";
const ICON_GEAR =
  "M8 0a1.5 1.5 0 0 1 1.5 1.5v.364a4.98 4.98 0 0 1 1.424.845l.319-.19a1.5 1.5 0 0 1 1.5 2.598l-.322.19a4.97 4.97 0 0 1 0 1.784l.322.19a1.5 1.5 0 0 1-1.5 2.598l-.319-.19a4.98 4.98 0 0 1-1.424.845V13a1.5 1.5 0 0 1-3 0v-.364a4.98 4.98 0 0 1-1.424-.845l-.319.19a1.5 1.5 0 0 1-1.5-2.598l.322-.19a4.97 4.97 0 0 1 0-1.784l-.322-.19a1.5 1.5 0 0 1 1.5-2.598l.319.19A4.98 4.98 0 0 1 6.5 1.864V1.5A1.5 1.5 0 0 1 8 0Zm0 4a2 2 0 1 0 0 4 2 2 0 0 0 0-4Z";

export function useDevHelpers(deps: UseDevHelpersDeps) {
  const { activePack, notify } = deps;
  const { t } = useI18n();

  async function openExamplePack() {
    try {
      await openExternal(EXAMPLE_PACK_REPO);
    } catch {
      notify(t("dev.errOpen", { url: EXAMPLE_PACK_REPO }), "error");
    }
  }

  /** URL репозитория GitHub активной сборки (из mrpack-ссылки). */
  const activePackRepo = computed(() => {
    const url = activePack.value?.url?.replace(/^https?:\/\/github\.com\//, "") ?? "";
    const [owner, repo] = url.split("/");
    if (!owner || !repo) return "";
    return `https://github.com/${owner}/${repo}`;
  });

  async function openExampleInLauncher() {
    try {
      await openExternal(deepLinkExample);
    } catch {
      notify(t("dev.errOpen", { url: deepLinkExample }), "error");
    }
  }

  /** Строит универсальную ссылку-приглашение для любой сборки
   *  (blog — ник блога Boosty издателя, чтобы сборка пришла платной). */
  function inviteLinkFor(pack: { name: string; url: string; boostyBlog?: string | null }): string {
    let link =
      "https://n1orio.github.io/mono-launcher/?url=" + encodeURIComponent(pack.url);
    if (pack.name) link += "&name=" + encodeURIComponent(pack.name);
    if (pack.boostyBlog) link += "&blog=" + encodeURIComponent(pack.boostyBlog);
    return link;
  }

  async function copyInviteLink() {
    const pack = activePack.value;
    if (!pack) return;
    const link = inviteLinkFor(pack);
    try {
      await navigator.clipboard.writeText(link);
      notify(t("dev.copyInviteDone"), "success");
    } catch {
      notify(`${t("dev.copyInviteFail")}: ${link}`, "error");
    }
  }

  /** Ссылка шаринга сборки (сайт /mono?url=&name=&blog=). */
  function packDeepLink(p: PackDescriptor | null | undefined): string | null {
    if (!p?.url) return null;
    const params = new URLSearchParams({ url: p.url, name: p.name });
    if (p.boostyBlog) params.set("blog", p.boostyBlog);
    return `${SITE_SHARE_URL}/mono?${params.toString()}`;
  }

  async function copyPackDeepLink(p: PackDescriptor | null | undefined) {
    const link = packDeepLink(p);
    if (!p || !link) {
      notify(t("pack.linkLocal"), "error");
      return;
    }
    try {
      await navigator.clipboard.writeText(link);
      notify(t("pack.linkCopied"), "success");
    } catch {
      notify(t("pack.linkCopyFail"), "error");
    }
  }

  return {
    EXAMPLE_PACK_REPO,
    examplePackJson,
    deepLinkExample,
    SITE_SHARE_URL,
    ICON_TAG,
    ICON_PACKAGE,
    ICON_PAINT,
    ICON_SUN,
    ICON_FOLDER,
    ICON_TERMINAL,
    ICON_IMAGE,
    ICON_DUP,
    ICON_SERVER,
    ICON_GEAR,
    activePackRepo,
    openExamplePack,
    openExampleInLauncher,
    inviteLinkFor,
    copyInviteLink,
    packDeepLink,
    copyPackDeepLink,
  };
}
