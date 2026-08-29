import { ref, computed, watch, nextTick } from "vue";
import type { Ref } from "vue";
import type { McVersionInfo } from "~/lib/types";
import { createLocalPack, localLoaderVersions, minecraftVersions, isTauri } from "~/lib/bridge";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

export interface UseCreatePackDeps {
  notify: (text: string, type?: string) => void;
  t: (key: string, params?: Record<string, unknown>) => string;
  loadPacks: () => Promise<void>;
  openPackTab: (id: string) => Promise<void>;
}

export function useCreatePack(deps: UseCreatePackDeps) {
  const { notify, t, loadPacks, openPackTab } = deps;

  const createPackOpen = ref(false);
  const CREATE_LOADERS = ["vanilla", "fabric", "quilt", "forge", "neoforge"] as const;
  type LoaderKey = (typeof CREATE_LOADERS)[number];
  const createPackName = ref("");
  const createPackMc = ref("1.21.4");
  const createPackLoader = ref<LoaderKey>("fabric");
  const createPackBusy = ref(false);
  /** "" = последняя версия загрузчика ("Latest"). */
  const createPackLoaderVersion = ref("");
  const createPackLoaderVersions = ref<string[]>([]);
  const createPackLoaderLvOpen = ref(false);
  const createPackLvBox = ref<HTMLElement | null>(null);
  let createPackLvClose: ((e: MouseEvent) => void) | null = null;
  const createPackVersions = ref<McVersionInfo[]>([]);
  const createPackVersionOpen = ref(false);
  const createPackVersionQuery = ref("");
  const createPackVersionBox = ref<HTMLElement | null>(null);
  let createPackVersionClose: ((e: MouseEvent) => void) | null = null;
  const createPackIcon = ref<string | null>(null);
  const createPackBanner = ref<string | null>(null);

  /** При открытии модалки создания — грузим список версий Minecraft и сбрасываем выбор файлов. */
  watch(createPackOpen, async (open) => {
    if (!open) return;
    createPackIcon.value = null;
    createPackBanner.value = null;
    createPackLoaderVersion.value = "";
    if (createPackVersions.value.length) return;
    try {
      createPackVersions.value = await minecraftVersions();
      if (createPackVersions.value.length) {
        const cur = createPackMc.value;
        const has = (id: string) => createPackVersions.value.some((v) => v.id === id);
        createPackMc.value = has(cur) ? cur : has("1.21.4") ? "1.21.4" : createPackVersions.value[0].id;
      }
    } catch (e) {
      notify(t("mods.createErr", { e }));
    }
  });

  /** При смене загрузчика/версии — заново грузим доступные версии загрузчика. */
  watch([createPackLoader, createPackMc], async ([loader, mc]) => {
    createPackLoaderVersion.value = "";
    createPackLoaderVersions.value = [];
    if (createPackOpen.value && loader !== "vanilla") {
      try {
        createPackLoaderVersions.value = await localLoaderVersions(loader, mc.trim());
      } catch (e) {
        createPackLoaderVersions.value = [];
        notify(t("mods.createErr", { e }));
      }
    }
  });

  /** Закрываем выпадающий список версий загрузчика при клике вне его. */
  watch(createPackLoaderLvOpen, (open) => {
    if (createPackLvClose) {
      document.removeEventListener("mousedown", createPackLvClose);
      createPackLvClose = null;
    }
    if (open) {
      createPackLvClose = (e) => {
        if (!createPackLvBox.value?.contains(e.target as Node)) createPackLoaderLvOpen.value = false;
      };
      document.addEventListener("mousedown", createPackLvClose);
    }
  });

  function chooseCreateLoaderVersion(v: string) {
    createPackLoaderVersion.value = v;
    createPackLoaderLvOpen.value = false;
  }

  /** Отфильтрованные по запросу подгруппы версий для выпадающего списка. */
  const filteredCreateReleases = computed(() =>
    createPackVersions.value
      .filter((v) => v.kind !== "snapshot" && v.id.toLowerCase().includes(createPackVersionQuery.value.toLowerCase()))
  );
  const filteredCreateSnapshots = computed(() =>
    createPackVersions.value
      .filter((v) => v.kind === "snapshot" && v.id.toLowerCase().includes(createPackVersionQuery.value.toLowerCase()))
  );
  const createVersionGroups = computed(() => [
    { label: t("mods.createReleases"), items: filteredCreateReleases.value },
    { label: t("mods.createSnapshots"), items: filteredCreateSnapshots.value },
  ]);

  function chooseCreateVersion(id: string) {
    createPackMc.value = id;
    createPackVersionOpen.value = false;
    createPackVersionQuery.value = "";
  }

  /** Закрываем выпадающий список при клике вне его. */
  watch(createPackVersionOpen, (open) => {
    if (createPackVersionClose) {
      document.removeEventListener("mousedown", createPackVersionClose);
      createPackVersionClose = null;
    }
    if (open) {
      createPackVersionClose = (e) => {
        if (!createPackVersionBox.value?.contains(e.target as Node)) {
          createPackVersionOpen.value = false;
          createPackVersionQuery.value = "";
        }
      };
      document.addEventListener("mousedown", createPackVersionClose);
    }
  });

  async function pickCreateFile(target: "icon" | "banner") {
    const path = await pickImage();
    if (!path) return;
    if (target === "icon") createPackIcon.value = path;
    else createPackBanner.value = path;
  }

  async function pickImage(): Promise<string | null> {
    if (!isTauri()) {
      notify(t("skin.tauriOnly"), "info");
      return null;
    }
    try {
      const p = await openDialog({
        multiple: false,
        filters: [
          { name: "Изображение", extensions: ["png", "jpg", "jpeg", "webp"] },
        ],
      });
      return typeof p === "string" ? p : null;
    } catch {
      notify(t("skin.readFail"), "error");
      return null;
    }
  }

  async function createPack() {
    if (createPackBusy.value) return;
    const name = createPackName.value.trim();
    if (!name) {
      notify(t("mods.createName"), "info");
      return;
    }
    createPackBusy.value = true;
    try {
      const pack = await createLocalPack(
        name,
        createPackMc.value.trim(),
        createPackLoader.value,
        createPackIcon.value,
        createPackBanner.value,
        createPackLoaderVersion.value || null
      );
      notify(t("mods.packCreated", { name: pack.name }), "success");
      createPackOpen.value = false;
      createPackName.value = "";
      createPackIcon.value = null;
      createPackBanner.value = null;
      await loadPacks();
      await nextTick();
      openPackTab(pack.id);
    } catch (e) {
      notify(t("mods.createErr", { e }));
    } finally {
      createPackBusy.value = false;
    }
  }

  return {
    createPackOpen,
    CREATE_LOADERS,
    createPackName,
    createPackMc,
    createPackLoader,
    createPackBusy,
    createPackLoaderVersion,
    createPackLoaderVersions,
    createPackLoaderLvOpen,
    createPackLvBox,
    createPackVersions,
    createPackVersionOpen,
    createPackVersionQuery,
    createPackVersionBox,
    createPackIcon,
    createPackBanner,
    chooseCreateLoaderVersion,
    filteredCreateReleases,
    filteredCreateSnapshots,
    createVersionGroups,
    chooseCreateVersion,
    pickCreateFile,
    createPack,
  };
}
