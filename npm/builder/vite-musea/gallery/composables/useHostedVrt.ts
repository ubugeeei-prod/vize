import { onUnmounted, ref } from "vue";
import { joinBasePath } from "../staticApi";
import type { VrtResult, VrtSummary } from "../components/vrtResults";

type LoopbackRequest = RequestInit & { targetAddressSpace: "loopback" };
interface Capture {
  artifacts?: never;
  success: boolean;
  results: Array<VrtResult & { images: Record<string, string> }>;
  summary: VrtSummary;
  reports: { json: string; html: string };
}
interface Connection {
  endpoint: string;
  token: string;
  generation: number;
  controller: AbortController;
}

/** Accept only a bounded JSON error from the authenticated service. */
async function responseError(response: Response): Promise<string> {
  let message = `Local VRT session: HTTP ${response.status}`;
  if (/^application\/json(?:;|$)/i.test(response.headers.get("Content-Type") ?? "")) {
    const reader = response.body?.getReader();
    if (reader) {
      try {
        const decoder = new TextDecoder();
        let bytes = 0;
        let text = "";
        while (true) {
          const { value, done } = await reader.read();
          if (done) break;
          bytes += value.length;
          if (bytes > 4096) {
            await reader.cancel();
            throw new Error("Error response exceeds the bounded message size");
          }
          text += decoder.decode(value, { stream: true });
        }
        const data: unknown = JSON.parse(text + decoder.decode());
        if (
          data &&
          typeof data === "object" &&
          typeof (data as { error?: unknown }).error === "string"
        ) {
          const error = (data as { error: string }).error.trim().slice(0, 1024);
          if (error) message = error;
        }
      } catch {
        /* An unreadable error body keeps the bounded HTTP fallback. */
      } finally {
        reader.releaseLock();
      }
    }
  }
  return response.status === 401
    ? `${message}. Reconnect with the endpoint and token printed by musea-vrt serve.`
    : message;
}

export function useHostedVrt() {
  const connected = ref(false);
  const connectionError = ref("");
  const reports = ref<{ json: string; html: string } | null>(null);
  let active: Connection | undefined;
  const pending = new Set<Connection>();
  let generation = 0;
  let alive = true;
  let objectUrls: string[] = [];

  function owns(connection: Connection) {
    return alive && connection.generation === generation && !connection.controller.signal.aborted;
  }
  function assertOwner(connection: Connection) {
    if (!owns(connection)) throw new Error("The VRT connection changed.");
  }
  function revokeImages() {
    for (const url of objectUrls) URL.revokeObjectURL(url);
    objectUrls = [];
  }

  async function request(connection: Connection, route: string, init: RequestInit = {}) {
    assertOwner(connection);
    const headers = new Headers(init.headers);
    headers.set("Authorization", `Bearer ${connection.token}`);
    const options: LoopbackRequest = {
      ...init,
      mode: "cors",
      credentials: "omit",
      redirect: "error",
      cache: "no-store",
      targetAddressSpace: "loopback",
      signal: connection.controller.signal,
      headers,
    };
    const response = await fetch(`${connection.endpoint}${route}`, options);
    assertOwner(connection);
    if (!response.ok) {
      const message = await responseError(response);
      assertOwner(connection);
      if (response.status === 401) {
        connected.value = false;
        active = undefined;
        reports.value = null;
        connectionError.value = message;
        revokeImages();
        connection.controller.abort();
      }
      throw new Error(message);
    }
    return response;
  }

  async function connect(address: string, secret: string) {
    if (!alive) return;
    const owner = ++generation;
    active?.controller.abort();
    active = undefined;
    connected.value = false;
    connectionError.value = "";
    reports.value = null;
    revokeImages();
    let connection: Connection | undefined;
    try {
      if (!window.isSecureContext)
        throw new Error("Open this gallery over HTTPS, or use the CLI below.");
      const url = new URL(address);
      if (
        url.protocol !== "http:" ||
        url.hostname !== "127.0.0.1" ||
        !url.port ||
        url.username ||
        url.password ||
        url.search ||
        url.hash ||
        url.pathname !== "/"
      )
        throw new Error("Enter the 127.0.0.1 endpoint printed by musea-vrt serve.");
      if (!/^[A-Za-z0-9_-]{43}$/.test(secret)) throw new Error("Enter the current session token.");
      connection = {
        endpoint: url.origin,
        token: secret,
        generation: owner,
        controller: new AbortController(),
      };
      pending.add(connection);
      let state: PermissionState = "prompt";
      try {
        state = (await navigator.permissions.query({ name: "loopback-network" as PermissionName }))
          .state;
      } catch {
        /* Unsupported granular query: the explicit fetch remains authoritative. */
      }
      assertOwner(connection);
      if (state === "denied")
        throw new Error(
          "Loopback access is blocked. Allow it in this site's settings, or use the CLI below.",
        );
      const session = (await (await request(connection, "/session")).json()) as {
        version: number;
        galleryUrl: string;
      };
      assertOwner(connection);
      const galleryUrl = new URL(joinBasePath("/"), window.location.href).href;
      if (session.version !== 1 || session.galleryUrl !== galleryUrl)
        throw new Error(
          "This session belongs to a different gallery. Start it with this gallery URL.",
        );
      active = connection;
      connected.value = true;
    } catch (error) {
      if (alive && generation === owner) {
        active = undefined;
        connectionError.value =
          error instanceof TypeError
            ? "Connection blocked. Allow loopback access in site settings and check the session endpoint, or use the CLI below."
            : error instanceof Error
              ? error.message
              : String(error);
      }
    } finally {
      if (connection) pending.delete(connection);
    }
  }

  async function run(artPath: string, update: boolean) {
    const connection = active;
    if (!connection || !connected.value) throw new Error("Connect a local VRT session first.");
    const created: string[] = [];
    try {
      const result = (await (
        await request(connection, "/capture", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ artPath, update }),
        })
      ).json()) as Capture;
      assertOwner(connection);
      if (!result.success) throw new Error("Capture did not complete.");
      for (const item of result.results) {
        const images: Record<string, string> = {};
        for (const [kind, route] of Object.entries(item.images)) {
          if (!/^\/artifacts\/[a-f0-9-]{36}$/.test(route))
            throw new Error("Invalid artifact registration");
          const blob = await (await request(connection, route)).blob();
          assertOwner(connection);
          const url = URL.createObjectURL(blob);
          created.push(url);
          images[kind] = url;
        }
        item.images = images;
      }
      assertOwner(connection);
      revokeImages();
      objectUrls = created.splice(0);
      reports.value = result.reports;
      return result;
    } catch (error) {
      for (const url of created) URL.revokeObjectURL(url);
      throw error;
    }
  }

  async function download(kind: "html" | "json") {
    const connection = active;
    const route = reports.value?.[kind];
    if (!connection || !route || !/^\/artifacts\/[a-f0-9-]{36}$/.test(route)) return;
    try {
      const blob = await (await request(connection, route)).blob();
      assertOwner(connection);
      const url = URL.createObjectURL(blob);
      try {
        const link = document.createElement("a");
        link.href = url;
        link.download = `musea-vrt-report.${kind}`;
        link.click();
      } finally {
        URL.revokeObjectURL(url);
      }
    } catch (error) {
      if (owns(connection))
        connectionError.value = error instanceof Error ? error.message : String(error);
    }
  }

  onUnmounted(() => {
    alive = false;
    generation++;
    for (const connection of pending) connection.controller.abort();
    pending.clear();
    active?.controller.abort();
    active = undefined;
    revokeImages();
  });
  return { connected, connectionError, reports, connect, run, download };
}
