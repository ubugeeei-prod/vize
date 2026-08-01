import type { IncomingMessage } from "node:http";
import path from "node:path";
import type { ViteDevServer } from "vite";

import type {
  VizeInspectorLintPlanProvider,
  VizeInspectorLintPlanRequest,
} from "../inspector-types.ts";
import type { VizePluginState } from "./state.ts";

export const VIZE_INSPECTOR_LINT_PLAN_ENDPOINT = "/__vize/inspector/lint-plan";

const MAX_REQUEST_URL_BYTES = 8 * 1024;
const MAX_FILE_COUNT = 128;
const MAX_FILE_BYTES = 4 * 1024;
const MAX_RESPONSE_BYTES = 4 * 1024 * 1024;
const ALLOWED_QUERY_KEYS = new Set(["file", "fresh"]);

interface InspectorResponse {
  statusCode: number;
  setHeader(name: string, value: number | string): void;
  end(chunk?: string): void;
}

export type InspectorLintPlanParseResult =
  | { request: VizeInspectorLintPlanRequest }
  | { statusCode: number; error: string }
  | null;

export function installInspectorLintPlanMiddleware(
  devServer: ViteDevServer,
  state: Pick<VizePluginState, "clientViteBase" | "logger" | "mergedOptions">,
): void {
  const provider = state.mergedOptions.inspector?.lintPlan;
  if (!provider) {
    return;
  }

  devServer.middlewares.use((req, res, next) => {
    if (!isInspectorLintPlanRequest(req.url, state.clientViteBase)) {
      next();
      return;
    }

    void handleInspectorLintPlanRequest(req, res, provider, state.logger, state.clientViteBase);
  });
}

export function isInspectorLintPlanRequest(reqUrl: string | undefined, base = "/"): boolean {
  if (!reqUrl) {
    return false;
  }

  try {
    return new URL(reqUrl, "http://localhost").pathname === resolveInspectorEndpoint(base);
  } catch {
    return false;
  }
}

export function parseInspectorLintPlanRequest(
  reqUrl: string | undefined,
  base = "/",
): InspectorLintPlanParseResult {
  if (!reqUrl || !isInspectorLintPlanRequest(reqUrl, base)) {
    return null;
  }
  if (Buffer.byteLength(reqUrl) > MAX_REQUEST_URL_BYTES) {
    return { statusCode: 414, error: "request_uri_too_long" };
  }

  const url = new URL(reqUrl, "http://localhost");
  for (const key of url.searchParams.keys()) {
    if (!ALLOWED_QUERY_KEYS.has(key)) {
      return { statusCode: 400, error: "invalid_query" };
    }
  }

  const freshValues = url.searchParams.getAll("fresh");
  if (freshValues.length > 1 || (freshValues[0] !== undefined && freshValues[0] !== "1")) {
    return { statusCode: 400, error: "invalid_fresh" };
  }

  const rawFiles = url.searchParams.getAll("file");
  if (rawFiles.length > MAX_FILE_COUNT) {
    return { statusCode: 413, error: "too_many_files" };
  }

  const files: string[] = [];
  const seen = new Set<string>();
  for (const file of rawFiles) {
    if (!isSafeInspectorFile(file)) {
      return { statusCode: 400, error: "invalid_file" };
    }
    if (!seen.has(file)) {
      seen.add(file);
      files.push(file);
    }
  }

  return { request: { files, fresh: freshValues[0] === "1" } };
}

export async function handleInspectorLintPlanRequest(
  req: Pick<IncomingMessage, "method" | "url">,
  res: InspectorResponse,
  provider: VizeInspectorLintPlanProvider,
  logger: Pick<Console, "error">,
  base = "/",
): Promise<void> {
  if (req.method !== "GET" && req.method !== "HEAD") {
    res.setHeader("allow", "GET, HEAD");
    sendJson(res, 405, { error: "method_not_allowed" });
    return;
  }

  const parsed = parseInspectorLintPlanRequest(req.url, base);
  if (!parsed) {
    sendJson(res, 404, { error: "not_found" });
    return;
  }
  if ("error" in parsed) {
    sendJson(res, parsed.statusCode, { error: parsed.error }, req.method === "HEAD");
    return;
  }

  try {
    const payload = await provider(parsed.request);
    sendJson(res, 200, payload, req.method === "HEAD");
  } catch (error) {
    logger.error("Failed to build inspector lint plan:", error);
    sendJson(res, 500, { error: "inspector_lint_plan_failed" }, req.method === "HEAD");
  }
}

function resolveInspectorEndpoint(base: string): string {
  const pathname = new URL(base, "http://localhost").pathname.replace(/\/+$/, "");
  return `${pathname}${VIZE_INSPECTOR_LINT_PLAN_ENDPOINT}`;
}

function isSafeInspectorFile(file: string): boolean {
  if (
    file.length === 0 ||
    Buffer.byteLength(file) > MAX_FILE_BYTES ||
    file.includes("\0") ||
    file.includes("\\") ||
    path.posix.isAbsolute(file)
  ) {
    return false;
  }
  return !file.split("/").some((segment) => segment === ".." || segment.length === 0);
}

function sendJson(
  res: InspectorResponse,
  statusCode: number,
  payload: unknown,
  headOnly = false,
): void {
  let body = JSON.stringify(payload);
  if (body === undefined) {
    throw new TypeError("Inspector payload is not JSON serializable");
  }
  if (Buffer.byteLength(body) > MAX_RESPONSE_BYTES) {
    statusCode = 413;
    body = JSON.stringify({ error: "inspector_response_too_large" });
  }

  res.statusCode = statusCode;
  res.setHeader("cache-control", "no-store");
  res.setHeader("content-type", "application/json; charset=utf-8");
  res.setHeader("content-length", Buffer.byteLength(body));
  res.setHeader("x-content-type-options", "nosniff");
  res.end(headOnly ? undefined : body);
}
