import type { DiffStats, InspectorPayload } from "./types";

export const INSPECTOR_HASH_KEY = "inspector";
const REPOSITORY_URL = "https://github.com/ubugeeei-prod/vize";

export function encodeInspectorPayload(payload: InspectorPayload): string {
  return encodeURIComponent(JSON.stringify(payload));
}

export function decodeInspectorPayload(value: string): InspectorPayload | null {
  try {
    const decoded = decodeURIComponent(value);
    const payload = JSON.parse(decoded) as Partial<InspectorPayload>;
    if (payload.version !== 1 || !Array.isArray(payload.files) || payload.files.length === 0) {
      return null;
    }
    return payload as InspectorPayload;
  } catch {
    return null;
  }
}

export function readInspectorPayloadFromUrl(href = window.location.href): InspectorPayload | null {
  const url = new URL(href);
  const hash = url.hash.replace(/^#/, "");
  if (!hash) return null;

  const params = new URLSearchParams(hash);
  const value = params.get(INSPECTOR_HASH_KEY) ?? params.get("payload") ?? hash;
  return decodeInspectorPayload(value);
}

export function createInspectorUrl(payload: InspectorPayload, href = window.location.href): string {
  const url = new URL(href);
  url.searchParams.set("tab", "inspector");
  url.hash = `${INSPECTOR_HASH_KEY}=${encodeInspectorPayload(payload)}`;
  return url.toString();
}

export function createPullRequestBody({
  permalink,
  payload,
  stats,
}: {
  permalink: string;
  payload: InspectorPayload;
  stats: DiffStats;
}): string {
  const files = payload.files.map((file) => `- \`${file.path}\``).join("\n");
  const target = payload.target === "ssr" ? "SSR" : payload.target === "vapor" ? "Vapor" : "DOM";

  return [
    "## Compiler inspector",
    "",
    `Playground permalink: ${permalink}`,
    "",
    "### Scope",
    "",
    `- Target: ${target}`,
    `- Files: ${payload.files.length}`,
    `- Output delta: +${stats.additions} / -${stats.removals}`,
    "- Includes: Vue output, Vize output, Virtual TS, VIR, and cross-file graph",
    "",
    "### Repro files",
    "",
    files,
    "",
    "### Notes",
    "",
    "Generated from the Vize playground compiler inspector. Please add the minimized fixture or full snapshot that explains the parity change.",
  ].join("\n");
}

export function createPullRequestUrl({
  permalink,
  payload,
  stats,
}: {
  permalink: string;
  payload: InspectorPayload;
  stats: DiffStats;
}): string {
  const title = "fix(compiler): align Vue compiler output";
  const body = createPullRequestBody({ permalink, payload, stats });
  const params = new URLSearchParams({
    quick_pull: "1",
    title,
    body,
  });

  return `${REPOSITORY_URL}/compare/main...compiler-inspector-repro?${params.toString()}`;
}
