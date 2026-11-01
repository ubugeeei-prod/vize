import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { sha256 } from "./value.ts";
import { boundedBytes } from "../release/quota/http.ts";
export type Sink = (kind: string, value: unknown) => void;
export const discard: Sink = () => {};
export function requestUrl(input: Parameters<typeof fetch>[0]): string {
  if (typeof input === "string") return input;
  return input instanceof URL ? input.href : input.url;
}
export function observedFetch(sink: Sink, fetcher: typeof fetch = fetch): typeof fetch {
  return async (input, init) => {
    const response = await fetcher(input, init);
    sink("http-headers", {
      url: requestUrl(input),
      status: response.status,
      date: response.headers.get("date"),
      location: response.headers.get("location"),
    });
    try {
      const hadBody = response.body !== null;
      const raw = hadBody ? await boundedBytes(response, 2 * 1024 * 1024) : Buffer.alloc(0);
      sink("http-body", {
        url: requestUrl(input),
        status: response.status,
        bytes: raw.length,
        sha256: sha256(raw),
        rawBase64: raw.toString("base64"),
      });
      const rebuilt = new Response(hadBody ? new Uint8Array(raw) : null, {
        status: response.status,
        statusText: response.statusText,
        headers: new Headers(response.headers),
      });
      for (const key of ["url", "redirected", "type"] as const)
        Object.defineProperty(rebuilt, key, { value: response[key] });
      return rebuilt;
    } catch (error) {
      sink("http-body-refusal", { url: requestUrl(input), error: String(error) });
      throw error;
    }
  };
}
function sanitize(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sanitize);
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value).map(([k, v]) => [
        /^(?:authorization|github_token|gh_token|node_auth_token|npm_token)$/iu.test(k) ? k : k,
        /^(?:authorization|github_token|gh_token|node_auth_token|npm_token)$/iu.test(k)
          ? "REDACTED"
          : sanitize(v),
      ]),
    );
  if (typeof value === "string" && /^https?:\/\//u.test(value)) {
    const u = new URL(value);
    if (u.search) return { origin: u.origin, path: u.pathname, querySha256: sha256(u.search) };
  }
  return value;
}
export function evidenceSink(root: string): Sink {
  let sequence = 0;
  return (kind, value) => {
    const raw = JSON.stringify(value);
    writeFileSync(
      join(
        root,
        `evidence-${String(++sequence).padStart(4, "0")}-${kind.replace(/[^a-z0-9-]/giu, "-")}.json`,
      ),
      JSON.stringify({ kind, logicalValueSha256: sha256(raw), value: sanitize(value) }, null, 2) +
        "\n",
      { flag: "wx" },
    );
  };
}
