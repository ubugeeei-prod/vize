import { randomBytes } from "node:crypto";
import { createServer, type IncomingMessage, type Server, type ServerResponse } from "node:http";
import type { AddressInfo } from "node:net";

import type { VizeNuxtInspectorLintPlanRequest } from "../compiler-options.ts";
import type { VizeNuxtLintDevtoolsOptions } from "./options.ts";
import { renderNuxtLintInspectorHtml } from "./inspector-view.ts";

const MAX_URL_BYTES = 8 * 1024;
const MAX_FILES = 128;
const MAX_FILE_BYTES = 4 * 1024;
const MAX_RESPONSE_BYTES = 4 * 1024 * 1024;

type Awaitable<T> = T | Promise<T>;
export type NuxtLintPlanProvider = (
  request: VizeNuxtInspectorLintPlanRequest,
) => Awaitable<unknown>;

interface NuxtDevtoolsTab {
  name: string;
  title: string;
  icon: string;
  requireAuth: boolean;
  view:
    | { type: "iframe"; src: string; persistent: boolean }
    | {
        type: "launch";
        description: string;
        actions: Array<{ label: string; pending: boolean; handle: () => Promise<void> }>;
      };
}

export interface NuxtLintDevtoolsNuxt {
  hook(name: string, callback: (...args: unknown[]) => unknown): void;
  callHook?(name: string, ...args: unknown[]): Awaitable<unknown>;
}

export interface ResolvedNuxtLintDevtoolsOptions {
  enabled: boolean | "lazy";
  port: number | undefined;
}

export interface NuxtLintDevtoolsController {
  close(): Promise<void>;
  start(): Promise<void>;
  tab(): NuxtDevtoolsTab;
  url(): string | undefined;
}

export function resolveNuxtLintDevtoolsOptions(
  options: VizeNuxtLintDevtoolsOptions | undefined,
): ResolvedNuxtLintDevtoolsOptions {
  const enabled = options?.enabled ?? "lazy";
  const port = options?.port;
  if (port !== undefined && (!Number.isInteger(port) || port < 1 || port > 65_535)) {
    throw new RangeError("Nuxt lint inspector port must be an integer from 1 to 65535");
  }
  return { enabled, port };
}

export async function setupNuxtLintDevtools(
  options: VizeNuxtLintDevtoolsOptions | undefined,
  nuxt: NuxtLintDevtoolsNuxt,
  provider: NuxtLintPlanProvider,
): Promise<NuxtLintDevtoolsController | undefined> {
  const resolved = resolveNuxtLintDevtoolsOptions(options);
  if (resolved.enabled === false) return undefined;

  const controller = createController(resolved.port, provider, nuxt);
  nuxt.hook("devtools:customTabs", (...args) => {
    const tabs = args[0] as NuxtDevtoolsTab[];
    tabs.push(controller.tab());
  });
  nuxt.hook("close", () => controller.close());
  if (resolved.enabled === true) {
    try {
      await controller.start();
    } catch (error) {
      console.warn(
        `[vize] Nuxt lint inspector failed to start: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
  }
  return controller;
}

function createController(
  requestedPort: number | undefined,
  provider: NuxtLintPlanProvider,
  nuxt: NuxtLintDevtoolsNuxt,
): NuxtLintDevtoolsController {
  const token = randomBytes(18).toString("base64url");
  const nonce = randomBytes(18).toString("base64url");
  const server = createServer((request, response) => {
    void handleRequest(request, response, provider, token, nonce, server);
  });
  let closed = false;
  let startPromise: Promise<void> | undefined;
  let viewerUrl: string | undefined;

  const start = async (): Promise<void> => {
    if (closed) throw new Error("Nuxt lint inspector is closed");
    if (viewerUrl) return;
    startPromise ||= listen(server, requestedPort).then(async (port) => {
      viewerUrl = `http://127.0.0.1:${port}/${token}/`;
      await nuxt.callHook?.("devtools:customTabs:refresh");
    });
    try {
      await startPromise;
    } catch (error) {
      startPromise = undefined;
      throw error;
    }
  };

  return {
    async close() {
      closed = true;
      try {
        await startPromise;
      } catch {
        // A failed listener has nothing left to close.
      }
      if (!server.listening) return;
      await new Promise<void>((resolve, reject) => {
        server.close((error) => (error ? reject(error) : resolve()));
        server.closeAllConnections();
      });
      viewerUrl = undefined;
      startPromise = undefined;
    },
    start,
    tab() {
      return {
        name: "vize-nuxt-lint-plan",
        title: "Vize Nuxt Lint Plan",
        icon: "carbon:rule",
        requireAuth: true,
        view: viewerUrl
          ? { type: "iframe", src: viewerUrl, persistent: true }
          : {
              type: "launch",
              description: "Inspect the effective Vize lint rules for any Nuxt source file.",
              actions: [{ label: "Launch", pending: startPromise !== undefined, handle: start }],
            },
      };
    },
    url: () => viewerUrl,
  };
}

function listen(server: Server, port: number | undefined): Promise<number> {
  return new Promise((resolve, reject) => {
    const onError = (error: Error) => {
      server.off("listening", onListening);
      reject(error);
    };
    const onListening = () => {
      server.off("error", onError);
      resolve((server.address() as AddressInfo).port);
    };
    server.once("error", onError);
    server.once("listening", onListening);
    server.listen({ host: "127.0.0.1", port: port ?? 0, exclusive: true });
  });
}

async function handleRequest(
  request: IncomingMessage,
  response: ServerResponse,
  provider: NuxtLintPlanProvider,
  token: string,
  nonce: string,
  server: Server,
): Promise<void> {
  if (Buffer.byteLength(request.url ?? "") > MAX_URL_BYTES) {
    sendJson(response, 414, { error: "request_uri_too_long" });
    return;
  }
  const port = (server.address() as AddressInfo | null)?.port;
  if (!port || ![`127.0.0.1:${port}`, `localhost:${port}`].includes(request.headers.host ?? "")) {
    sendJson(response, 421, { error: "misdirected_request" });
    return;
  }
  if (request.method !== "GET" && request.method !== "HEAD") {
    response.setHeader("allow", "GET, HEAD");
    sendJson(response, 405, { error: "method_not_allowed" });
    return;
  }

  let url: URL;
  try {
    url = new URL(request.url ?? "", "http://localhost");
  } catch {
    sendJson(response, 400, { error: "invalid_url" }, request.method === "HEAD");
    return;
  }
  const rootPath = `/${token}/`;
  if (url.pathname === rootPath && url.search === "") {
    sendHtml(response, renderNuxtLintInspectorHtml(nonce), nonce, request.method === "HEAD");
    return;
  }
  if (url.pathname !== `${rootPath}api`) {
    sendJson(response, 404, { error: "not_found" }, request.method === "HEAD");
    return;
  }

  const parsed = parseApiRequest(url);
  if ("error" in parsed) {
    sendJson(response, parsed.status, { error: parsed.error }, request.method === "HEAD");
    return;
  }
  try {
    sendJson(response, 200, await provider(parsed.request), request.method === "HEAD");
  } catch {
    sendJson(response, 500, { error: "inspector_lint_plan_failed" }, request.method === "HEAD");
  }
}

function parseApiRequest(
  url: URL,
): { request: VizeNuxtInspectorLintPlanRequest } | { status: number; error: string } {
  for (const key of url.searchParams.keys()) {
    if (key !== "file" && key !== "fresh") return { status: 400, error: "invalid_query" };
  }
  const fresh = url.searchParams.getAll("fresh");
  if (fresh.length > 1 || (fresh[0] !== undefined && fresh[0] !== "1")) {
    return { status: 400, error: "invalid_fresh" };
  }
  const requested = url.searchParams.getAll("file");
  if (requested.length > MAX_FILES) return { status: 413, error: "too_many_files" };
  const files = [...new Set(requested)];
  if (files.some((file) => !isSafeFile(file))) return { status: 400, error: "invalid_file" };
  return { request: { files, fresh: fresh[0] === "1" } };
}

function isSafeFile(file: string): boolean {
  return (
    file.length > 0 &&
    Buffer.byteLength(file) <= MAX_FILE_BYTES &&
    !file.includes("\0") &&
    !file.includes("\\") &&
    !file.startsWith("/") &&
    !/^[A-Za-z]:/u.test(file) &&
    !file.split("/").some((part) => part.length === 0 || part === "..")
  );
}

function setCommonHeaders(
  response: ServerResponse,
  resourcePolicy: "cross-origin" | "same-origin",
): void {
  response.setHeader("cache-control", "no-store");
  response.setHeader("cross-origin-resource-policy", resourcePolicy);
  response.setHeader("referrer-policy", "no-referrer");
  response.setHeader("x-content-type-options", "nosniff");
}

function sendHtml(response: ServerResponse, body: string, nonce: string, headOnly: boolean): void {
  response.statusCode = 200;
  setCommonHeaders(response, "cross-origin");
  response.setHeader(
    "content-security-policy",
    `default-src 'none'; script-src 'nonce-${nonce}'; style-src 'nonce-${nonce}'; connect-src 'self'; frame-ancestors http://localhost:* http://127.0.0.1:*`,
  );
  response.setHeader("content-type", "text/html; charset=utf-8");
  response.setHeader("content-length", Buffer.byteLength(body));
  response.end(headOnly ? undefined : body);
}

function sendJson(
  response: ServerResponse,
  status: number,
  payload: unknown,
  headOnly = false,
): void {
  let body = JSON.stringify(payload);
  if (body === undefined) {
    status = 500;
    body = JSON.stringify({ error: "inspector_lint_plan_failed" });
  }
  if (Buffer.byteLength(body) > MAX_RESPONSE_BYTES) {
    status = 413;
    body = JSON.stringify({ error: "inspector_response_too_large" });
  }
  response.statusCode = status;
  setCommonHeaders(response, "same-origin");
  response.setHeader("content-security-policy", "default-src 'none'");
  response.setHeader("content-type", "application/json; charset=utf-8");
  response.setHeader("content-length", Buffer.byteLength(body));
  response.end(headOnly ? undefined : body);
}
