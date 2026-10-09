import { createHash } from "node:crypto";

type Download = { sha256: string; sha512: string; bytes: number };
type Redirect = NonNullable<RequestInit["redirect"]>;

function requireValue(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}
function object(value: unknown): Record<string, unknown> {
  requireValue(
    value && typeof value === "object" && !Array.isArray(value),
    "registry object required",
  );
  return value as Record<string, unknown>;
}

/** Bounded unauthenticated reads; streamed archive digests are not signature claims. */
export function publicReader(fetchOverride?: typeof globalThis.fetch) {
  const fetchImpl = fetchOverride ?? globalThis.fetch;
  const request = async <T>(
    url: string,
    read: (response: Response) => Promise<T>,
    redirect: Redirect = "error",
  ): Promise<T> => {
    const controller = new AbortController();
    let timer: ReturnType<typeof setTimeout> | undefined;
    const timeout = new Promise<never>((_, reject) => {
      timer = setTimeout(() => {
        controller.abort();
        reject(new Error(`public request timed out after 30000ms: ${url}`));
      }, 30_000);
    });
    try {
      return await Promise.race([
        timeout,
        (async () => {
          const response = await fetchImpl(url, {
            signal: controller.signal,
            credentials: "omit",
            redirect,
            headers: { Accept: "application/json", "User-Agent": "vize-public-release-acceptance" },
          });
          requireValue(response.ok, `public HTTP ${response.status}: ${url}`);
          return read(response);
        })(),
      ]);
    } finally {
      clearTimeout(timer);
    }
  };
  const json = (url: string) => request(url, async (response) => object(await response.json()));
  const download = (url: string, redirect: Redirect = "error") =>
    request<Download>(
      url,
      async (response) => {
        requireValue(response.body, "public archive body missing");
        const reader = response.body.getReader();
        const hashes = [createHash("sha256"), createHash("sha512")];
        let bytes = 0;
        try {
          for (;;) {
            const part = await reader.read();
            if (part.done) break;
            bytes += part.value.byteLength;
            requireValue(bytes <= 512 * 1024 * 1024, "public archive exceeds 512MiB bound");
            for (const hash of hashes) hash.update(part.value);
          }
        } finally {
          await reader.cancel();
          reader.releaseLock();
        }
        requireValue(bytes > 0, "empty public archive");
        return { sha256: hashes[0].digest("hex"), sha512: hashes[1].digest("base64"), bytes };
      },
      redirect,
    );

  const text = (url: string, redirect: Redirect = "error") =>
    request(
      url,
      async (response) => {
        const content = await response.text();
        requireValue(content.length <= 4096, "public checksum text exceeds 4096-character bound");
        return content;
      },
      redirect,
    );
  return { json, download, text };
}
