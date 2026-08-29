import { computed, ref, watch } from "vue";
import type { Ref } from "vue";
import type { ServerStatus, PackDescriptor } from "~/lib/types";
import { pingServer } from "~/lib/bridge";
import { useI18n } from "./useI18n";

/** Разбирает адрес "host" или "host:port" из servers.dat. */
function splitServerAddress(address: string): { ip: string; port: number | null } {
  const idx = address.lastIndexOf(":");
  if (idx > 0 && /^\d+$/.test(address.slice(idx + 1))) {
    return { ip: address.slice(0, idx), port: Number(address.slice(idx + 1)) };
  }
  return { ip: address, port: null };
}

type ServerGroup = {
  key: "mine";
  title: string;
  servers: { name: string; ip: string; port: number | null; desc: string | null }[];
  emptyText: string;
};

export type { ServerGroup };

export interface UseServersDeps {
  activePack: Ref<PackDescriptor | null>;
  myServers: Ref<any[]>;
  myServersInstalled: Ref<boolean>;
  tab: Ref<string>;
  playSubTab: Ref<string>;
  packId: Ref<string | null>;
  loadMyServers: (id: string) => Promise<void>;
  notify: (text: string, type?: string) => void;
}

export function useServers(deps: UseServersDeps) {
  const { t } = useI18n();

  /** Группы серверов: свои (servers.dat). */
  const serverGroups = computed<ServerGroup[]>(() => {
    const mine = deps.myServers.value.map((s) => {
      const { ip, port } = splitServerAddress(s.address);
      return { name: s.name, ip, port, desc: null };
    });
    return [
      {
        key: "mine",
        title: t("servers.myTitle"),
        servers: mine,
        emptyText: deps.myServersInstalled.value ? t("servers.myEmpty") : t("servers.noInstall"),
      },
    ];
  });

  async function copyServerIp(srv: { ip: string; port: number | null }) {
    const text = `${srv.ip}${srv.port ? `:${srv.port}` : ""}`;
    try {
      await navigator.clipboard.writeText(text);
      deps.notify(t("servers.copied", { ip: text }), "success");
    } catch {
      deps.notify(`${t("servers.copyFail")}: ${text}`, "error");
    }
  }

  /** Статусы серверов активной сборки: key "host:port" → результат пинга. */
  const serverStatuses = ref<Record<string, ServerStatus>>({});
  const serverPinging = ref<Record<string, boolean>>({});

  function serverKey(srv: { ip: string; port: number | null }): string {
    return `${srv.ip}:${srv.port ?? 25565}`;
  }

  function stopServerPingTimer() {
    if (serverPingTimer) {
      clearInterval(serverPingTimer);
      serverPingTimer = null;
    }
  }

  async function pingOneServer(srv: { ip: string; port: number | null }) {
    const key = serverKey(srv);
    if (serverPinging.value[key]) return;
    serverPinging.value[key] = true;
    try {
      serverStatuses.value[key] = await pingServer(srv.ip, srv.port ?? null);
    } catch {
      serverStatuses.value[key] = { online: false, version: null, motd: null, playersOnline: null, playersMax: null, players: [], latencyMs: null };
    } finally {
      serverPinging.value[key] = false;
    }
  }

  function serverPlayersOf(srv: { ip: string; port: number | null }): string[] {
    return serverStatuses.value[serverKey(srv)]?.players ?? [];
  }

  function pingActiveServers() {
    serverGroups.value.forEach((g) => g.servers.forEach((srv) => void pingOneServer(srv)));
  }

  function serverStateOf(srv: { ip: string; port: number | null }): ServerState {
    const key = serverKey(srv);
    if (serverPinging.value[key]) return "checking";
    const st = serverStatuses.value[key];
    if (!st) return "unknown";
    return st.online ? "online" : "offline";
  }

  function serverStatusText(srv: { ip: string; port: number | null }): string {
    const key = serverKey(srv);
    const st = serverStatuses.value[key];
    switch (serverStateOf(srv)) {
      case "checking":
        return t("servers.checking");
      case "unknown":
        return t("servers.unknown");
      case "offline":
        return t("servers.offline");
      default: {
        const parts = [t("servers.online")];
        if (st?.playersOnline != null) parts.push(`${st.playersOnline}/${st.playersMax ?? "?"}`);
        if (st?.version) parts.push(st.version);
        if (st?.latencyMs != null) parts.push(`${st.latencyMs}мс`);
        return parts.join(" · ");
      }
    }
  }

  // Таймер пинга серверов живёт только пока экран «Серверы» реально активен
  // (вкладка play + серверный сабтаб + выбранная сборка). Иначе — гасим, чтобы
  // не пинговать в фоне при переключении сборок/вкладок.
  let serverPingTimer: ReturnType<typeof setInterval> | null = null;
  watch(
    () => [deps.tab.value, deps.playSubTab.value, deps.packId.value],
    () => {
      if (deps.tab.value === "play" && deps.playSubTab.value === "servers" && deps.activePack.value) {
        deps.loadMyServers(deps.activePack.value.id);
        pingActiveServers();
        stopServerPingTimer();
        serverPingTimer = setInterval(pingActiveServers, 45000);
      } else {
        stopServerPingTimer();
      }
    },
    { immediate: true },
  );

  return {
    serverGroups,
    copyServerIp,
    serverStatuses,
    serverPinging,
    serverKey,
    stopServerPingTimer,
    pingOneServer,
    serverPlayersOf,
    pingActiveServers,
    serverStateOf,
    serverStatusText,
  };
}

export type ServerState = "online" | "offline" | "checking" | "unknown";
