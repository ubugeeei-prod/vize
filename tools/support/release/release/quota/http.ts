import { createHash } from "node:crypto";
export async function boundedBytes(response: Response, maximum: number): Promise<Buffer> {
  if (!response.body) throw new Error("bounded HTTP body required");
  const reader = response.body.getReader(),
    chunks: Uint8Array[] = [];
  let length = 0;
  try {
    for (;;) {
      const part = await reader.read();
      if (part.done) break;
      length += part.value.byteLength;
      if (length > maximum) throw new Error("HTTP body exceeds proof byte limit");
      chunks.push(part.value);
    }
  } finally {
    await reader.cancel().catch(() => {});
    reader.releaseLock();
  }
  return Buffer.concat(chunks);
}
export async function githubSnapshot(path: string, fetchImpl: typeof fetch = globalThis.fetch) {
  const response = await fetchImpl("https://api.github.com/repos/ubugeeei-prod/vize/" + path, {
    redirect: "error",
    signal: AbortSignal.timeout(15_000),
    headers: {
      Accept: "application/vnd.github+json",
      ...(process.env.GITHUB_TOKEN ? { Authorization: "Bearer " + process.env.GITHUB_TOKEN } : {}),
    },
  });
  if (response.status !== 200) throw new Error("original Release API HTTP " + response.status);
  const bytes = await boundedBytes(response, 2 * 1024 * 1024);
  return {
    value: JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes)) as unknown,
    url: "https://api.github.com/repos/ubugeeei-prod/vize/" + path,
    status: response.status,
    date: response.headers.get("date"),
    bytes: bytes.length,
    sha256: createHash("sha256").update(bytes).digest("hex"),
    rawBase64: bytes.toString("base64"),
  };
}

export async function githubJson(
  path: string,
  fetchImpl: typeof fetch = globalThis.fetch,
): Promise<unknown> {
  return (await githubSnapshot(path, fetchImpl)).value;
}
