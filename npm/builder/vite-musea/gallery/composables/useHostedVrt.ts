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

export function useHostedVrt() {
  const connected = ref(false);
  const connectionError = ref("");
  const reports = ref<{ json: string; html: string } | null>(null);
  let endpoint = "";
  let token = "";
  let objectUrls: string[] = [];

  function revokeImages() {
    for (const url of objectUrls) URL.revokeObjectURL(url);
    objectUrls = [];
  }

  async function request(route: string, init: RequestInit = {}) {
    const headers = new Headers(init.headers);
    headers.set("Authorization", `Bearer ${token}`);
    const options: LoopbackRequest = {
      ...init,
      mode: "cors",
      credentials: "omit",
      redirect: "error",
      cache: "no-store",
      targetAddressSpace: "loopback",
      headers,
    };
    const response = await fetch(`${endpoint}${route}`, options);
    if (!response.ok) throw new Error(`Local VRT session: HTTP ${response.status}`);
    return response;
  }

  async function connect(address: string, secret: string) {
    connected.value = false;
    connectionError.value = "";
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
      let state: PermissionState = "prompt";
      try {
        state = (await navigator.permissions.query({ name: "loopback-network" as PermissionName }))
          .state;
      } catch {
        /* Unsupported granular query: the explicit fetch remains authoritative. */
      }
      if (state === "denied")
        throw new Error(
          "Loopback access is blocked. Allow it in this site's settings, or use the CLI below.",
        );
      endpoint = url.origin;
      token = secret;
      const session = (await (await request("/session")).json()) as {
        version: number;
        galleryUrl: string;
      };
      const galleryUrl = new URL(joinBasePath("/"), window.location.href).href;
      if (session.version !== 1 || session.galleryUrl !== galleryUrl)
        throw new Error(
          "This session belongs to a different gallery. Start it with this gallery URL.",
        );
      connected.value = true;
    } catch (error) {
      endpoint = "";
      token = "";
      connectionError.value =
        error instanceof TypeError
          ? "Connection blocked. Allow loopback access in site settings and check the session endpoint, or use the CLI below."
          : error instanceof Error
            ? error.message
            : String(error);
    }
  }

  async function run(artPath: string, update: boolean) {
    if (!connected.value) throw new Error("Connect a local VRT session first.");
    const result = (await (
      await request("/capture", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ artPath, update }),
      })
    ).json()) as Capture;
    if (!result.success) throw new Error("Capture did not complete.");
    revokeImages();
    for (const item of result.results) {
      const images: Record<string, string> = {};
      for (const [kind, route] of Object.entries(item.images)) {
        if (!/^\/artifacts\/[a-f0-9-]{36}$/.test(route))
          throw new Error("Invalid artifact registration");
        const url = URL.createObjectURL(await (await request(route)).blob());
        objectUrls.push(url);
        images[kind] = url;
      }
      item.images = images;
    }
    reports.value = result.reports;
    return result;
  }

  async function download(kind: "html" | "json") {
    const route = reports.value?.[kind];
    if (!route || !/^\/artifacts\/[a-f0-9-]{36}$/.test(route)) return;
    const url = URL.createObjectURL(await (await request(route)).blob());
    const link = document.createElement("a");
    link.href = url;
    link.download = `musea-vrt-report.${kind}`;
    link.click();
    URL.revokeObjectURL(url);
  }

  onUnmounted(() => {
    revokeImages();
    endpoint = "";
    token = "";
  });
  return { connected, connectionError, reports, connect, run, download };
}
