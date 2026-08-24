import type { ViteDevServer } from "vite";
import fs, { type Stats } from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import path from "node:path";
import { buildInspectorGraph, normalizeViteDevMiddlewareUrl } from "@vizejs/native";
import { glob } from "tinyglobby";

import type { VizePluginState } from "./state.ts";
import { installInspectorLintPlanMiddleware } from "./inspector-lint-plan.ts";

export const VIZE_INSPECTOR_GRAPH_ENDPOINT = "/__vize/inspector/graph";

const INSPECTOR_SCRIPT_PATTERNS = ["**/*.{js,jsx,ts,tsx}"];
const INSPECTOR_EXTRA_IGNORE_PATTERNS = ["**/*.d.ts"];
const INSPECTOR_FILE_EXTENSION_RE = /\.(?:vue|[jt]sx?)$/;

interface InspectorSourceFile {
  path: string;
  source: string;
}

export interface VizeInspectorGraphPayload {
  schema: "vize.inspector.graph";
  version: 1;
  root: string;
  fileCount: number;
  graph: ReturnType<typeof buildInspectorGraph>;
}

export function installDevMiddleware(
  devServer: ViteDevServer,
  state: Pick<
    VizePluginState,
    "clientViteBase" | "ignorePatterns" | "logger" | "mergedOptions" | "root" | "scanPatterns"
  >,
): void {
  installVirtualAssetMiddleware(devServer, state);
  installInspectorGraphMiddleware(devServer, state);
  installInspectorLintPlanMiddleware(devServer, state);
}

export function installVirtualAssetMiddleware(
  devServer: ViteDevServer,
  state: Pick<VizePluginState, "logger">,
): void {
  devServer.middlewares.use((req, _res, next) => {
    const rewrite = req.url ? normalizeViteDevMiddlewareUrl(req.url) : null;
    if (rewrite && fs.existsSync(rewrite.fsPath) && fs.statSync(rewrite.fsPath).isFile()) {
      state.logger.log(`middleware: rewriting ${req.url} -> ${rewrite.cleanedUrl}`);
      req.url = rewrite.cleanedUrl;
    }
    next();
  });
}

export function installInspectorGraphMiddleware(
  devServer: ViteDevServer,
  state: Pick<VizePluginState, "ignorePatterns" | "logger" | "root" | "scanPatterns">,
): void {
  devServer.middlewares.use((req, res, next) => {
    if (!isInspectorGraphRequest(req.url)) {
      next();
      return;
    }

    void handleInspectorGraphRequest(req, res, state);
  });
}

export async function createInspectorGraphPayload(
  state: Pick<VizePluginState, "ignorePatterns" | "root" | "scanPatterns">,
): Promise<VizeInspectorGraphPayload> {
  const files = await collectInspectorSourceFiles(state);
  return {
    schema: "vize.inspector.graph",
    version: 1,
    root: state.root,
    fileCount: files.length,
    graph: buildInspectorGraph(files),
  };
}

export async function collectInspectorSourceFiles(
  state: Pick<VizePluginState, "ignorePatterns" | "root" | "scanPatterns">,
): Promise<InspectorSourceFile[]> {
  const paths = await glob(resolveInspectorScanPatterns(state.scanPatterns), {
    cwd: state.root,
    absolute: true,
    followSymbolicLinks: false,
    ignore: [...state.ignorePatterns, ...INSPECTOR_EXTRA_IGNORE_PATTERNS],
  });

  const files: InspectorSourceFile[] = [];
  for (const filePath of [...new Set(paths)].sort()) {
    if (!INSPECTOR_FILE_EXTENSION_RE.test(filePath)) {
      continue;
    }

    const stat = lstatFile(filePath);
    if (!stat || stat.isSymbolicLink() || !stat.isFile()) {
      continue;
    }

    let realPath: string;
    try {
      realPath = fs.realpathSync.native(filePath);
    } catch {
      continue;
    }
    if (!isResolvedPathInside(state.root, realPath)) {
      continue;
    }

    files.push({
      path: normalizeInspectorPath(state.root, filePath),
      source: fs.readFileSync(filePath, "utf-8"),
    });
  }

  return files;
}

export function isInspectorGraphRequest(reqUrl: string | undefined): boolean {
  if (!reqUrl) {
    return false;
  }

  try {
    return new URL(reqUrl, "http://localhost").pathname === VIZE_INSPECTOR_GRAPH_ENDPOINT;
  } catch {
    return false;
  }
}

function resolveInspectorScanPatterns(scanPatterns: string[] | null): string[] {
  const vuePatterns = scanPatterns && scanPatterns.length > 0 ? scanPatterns : ["**/*.vue"];
  return [...vuePatterns, ...INSPECTOR_SCRIPT_PATTERNS];
}

function normalizeInspectorPath(root: string, filePath: string): string {
  return path.relative(root, filePath).split(path.sep).join("/");
}

function lstatFile(filePath: string): Stats | null {
  try {
    return fs.lstatSync(filePath);
  } catch {
    return null;
  }
}

function isResolvedPathInside(parentDir: string, candidatePath: string): boolean {
  const parent = path.resolve(parentDir);
  const candidate = path.resolve(candidatePath);
  const relative = path.relative(parent, candidate);
  return relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative));
}

async function handleInspectorGraphRequest(
  req: IncomingMessage,
  res: ServerResponse,
  state: Pick<VizePluginState, "ignorePatterns" | "logger" | "root" | "scanPatterns">,
): Promise<void> {
  if (req.method !== "GET" && req.method !== "HEAD") {
    sendJson(res, 405, { error: "method_not_allowed" });
    return;
  }

  try {
    const payload = await createInspectorGraphPayload(state);
    sendJson(res, 200, payload, req.method === "HEAD");
  } catch (error) {
    state.logger.error("Failed to build inspector graph:", error);
    sendJson(res, 500, { error: "inspector_graph_failed", message: formatUnknownError(error) });
  }
}

function sendJson(
  res: ServerResponse,
  statusCode: number,
  payload: unknown,
  headOnly = false,
): void {
  const body = JSON.stringify(payload);
  res.statusCode = statusCode;
  res.setHeader("content-type", "application/json; charset=utf-8");
  res.setHeader("content-length", Buffer.byteLength(body));
  res.end(headOnly ? undefined : body);
}

function formatUnknownError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
