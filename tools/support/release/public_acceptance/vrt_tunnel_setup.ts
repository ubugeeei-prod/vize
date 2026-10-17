import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  appendFile,
  chmod,
  lstat,
  mkdtemp,
  open,
  readFile,
  realpath,
  rm,
  writeFile,
} from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { publicInputs } from "./inputs.ts";
import { readRawBlob } from "./plan.ts";
import { assertQuickTunnelBinary, quickTunnelPin } from "./vrt_tunnel.ts";
import {
  assertQuickTunnelDownloadLocation,
  assertQuickTunnelMetadata,
  assertTunnelReleaseCustody,
  quickTunnelMetadata,
} from "./vrt_tunnel_metadata.ts";

const serviceImplementation = "9c86772fe1e5de18aa8b658a92ab067969567c6a";
const digest = (body: Buffer) => createHash("sha256").update(body).digest("hex");

/** Full-history checkout must contain the delivered implementation before any acquisition. */
export function requireReleasedService(root: string, head: string) {
  assert.match(head, /^[0-9a-f]{40}$/u);
  execFileSync(
    "git",
    [
      "--no-replace-objects",
      "-C",
      root,
      "merge-base",
      "--is-ancestor",
      serviceImplementation,
      head,
    ],
    { stdio: "pipe", timeout: 30_000 },
  );
  return ["src/cli/serve.ts", "src/cli/index.ts", "package.json"].map((relative) => {
    const file = `npm/builder/vite-musea/${relative}`,
      bytes = readRawBlob(root, head, file);
    return { file, bytes: bytes.length, sha256: digest(bytes) };
  });
}

/** New private directory only; never reuse a personal config or put executable bytes in retained output. */
export async function createTunnelAssetDirectory(temporary: string, retained: string) {
  assert.ok(path.isAbsolute(temporary) && path.isAbsolute(retained));
  const root = await realpath(temporary),
    output = await realpath(retained);
  assert.ok(
    root !== output && !root.startsWith(output + path.sep),
    "private temporary root required",
  );
  assert.ok((await lstat(root)).isDirectory());
  const directory = await mkdtemp(path.join(root, "vize-public-tunnel-"));
  try {
    await chmod(directory, 0o700);
    assert.equal(await realpath(directory), directory);
    assert.equal((await lstat(directory)).mode & 0o777, 0o700);
    assert.ok(!/[\r\n]/u.test(directory));
    return directory;
  } catch (error) {
    await rm(directory, { recursive: true, force: true });
    throw error;
  }
}

async function metadata(url: string) {
  const response = await fetch(url, {
    credentials: "omit",
    redirect: "error",
    signal: AbortSignal.timeout(30_000),
    headers: {
      Accept: "application/vnd.github+json",
      "User-Agent": "vize-public-release-acceptance",
      "X-GitHub-Api-Version": "2022-11-28",
    },
  });
  assert.equal(response.status, 200, "official pin metadata HTTP200 required");
  assert.ok(response.body, "pin metadata body required");
  const reader = response.body.getReader(),
    parts: Uint8Array[] = [];
  let bytes = 0;
  try {
    for (;;) {
      const part = await reader.read();
      if (part.done) break;
      bytes += part.value.byteLength;
      assert.ok(bytes <= 1024 * 1024, "pin metadata exceeds 1MiB");
      parts.push(part.value);
    }
  } finally {
    await reader.cancel();
    reader.releaseLock();
  }
  const body = Buffer.concat(parts);
  return { body, value: JSON.parse(body.toString("utf8")) as unknown };
}

/** Stream the pinned asset with one manual official-host redirect and a whole byte/hash barrier. */
async function acquireBinary(directory: string) {
  const target = path.join(directory, "cloudflared-linux-amd64"),
    pending = target + ".partial";
  let response: Response | undefined;
  const controller = new AbortController(),
    timer = setTimeout(() => controller.abort(), 60_000);
  try {
    let url = assertQuickTunnelDownloadLocation(quickTunnelPin.url, true);
    for (let attempt = 0; attempt < 2; attempt++) {
      response = await fetch(url, {
        credentials: "omit",
        redirect: "manual",
        signal: controller.signal,
        headers: {
          Accept: "application/octet-stream",
          "User-Agent": "vize-public-release-acceptance",
        },
      });
      if (response.status === 200) break;
      assert.equal(attempt, 0, "at most one asset redirect");
      assert.equal(response.status, 302, "official asset redirect must be302");
      const location = response.headers.get("location");
      assert.ok(location, "asset redirect location missing");
      const next = assertQuickTunnelDownloadLocation(location, false);
      await response.body?.cancel();
      url = next;
    }
    assert.equal(response?.status, 200, "pinned asset HTTP200 required");
    assert.ok(response?.body, "pinned binary body required");
    const length = response.headers.get("content-length");
    if (length !== null)
      assert.equal(length, String(quickTunnelPin.bytes), "pinned binary content length");
    const file = await open(pending, "wx", 0o600);
    let reader: ReadableStreamDefaultReader<Uint8Array> | undefined;
    let bytes = 0;
    const hash = createHash("sha256");
    try {
      reader = response.body.getReader();
      for (;;) {
        const part = await reader.read();
        if (part.done) break;
        bytes += part.value.byteLength;
        assert.ok(bytes <= quickTunnelPin.bytes, "binary exceeds exact pin length");
        hash.update(part.value);
        let offset = 0;
        while (offset < part.value.byteLength) {
          const written = await file.write(part.value, offset, part.value.byteLength - offset);
          assert.ok(written.bytesWritten > 0, "binary write must progress");
          offset += written.bytesWritten;
        }
      }
      assert.equal(bytes, quickTunnelPin.bytes, "binary exact pin length");
      assert.equal(hash.digest("hex"), quickTunnelPin.sha256, "binary whole SHA256 pin");
      await file.sync();
    } finally {
      try {
        await reader?.cancel();
      } finally {
        try {
          reader?.releaseLock();
        } finally {
          await file.close();
        }
      }
    }
    // Copy to the final exclusive file while it is still nonexecutable, then verify disk bytes again.
    const body = await readFile(pending);
    assert.equal(body.length, quickTunnelPin.bytes);
    assert.equal(digest(body), quickTunnelPin.sha256);
    await writeFile(target, body, { flag: "wx", mode: 0o600 });
    await rm(pending);
    await assertQuickTunnelBinary(directory, target);
    await chmod(target, 0o700);
    assert.equal((await lstat(target)).mode & 0o777, 0o700);
    return target;
  } finally {
    clearTimeout(timer);
    if (response?.body && !response.body.locked) await response.body.cancel();
  }
}

/** Retain the refusal phase and safe assertion label, never raw HTTP URLs, values or child output. */
export function tunnelSetupRefusal(error: unknown, phase: string) {
  const reason =
    error instanceof assert.AssertionError && !error.generatedMessage
      ? error.message.split("\n", 1)[0]
      : error instanceof assert.AssertionError
        ? `assertion ${error.operator}`
        : error instanceof Error
          ? error.name
          : "unknown failure";
  return new Error("Pinned tunnel setup refused; no executable directory is available.", {
    cause: { phase, reason },
  });
}

async function main() {
  let directory: string | undefined;
  let phase = "runtime identity";
  try {
    assert.equal(process.platform, "linux");
    assert.equal(process.arch, "x64");
    assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
    assert.equal(process.env.GITHUB_REF, "refs/heads/main");
    assert.equal(process.env.VRT_TUNNEL_DIRECTORY, undefined, "no borrowed executable directory");
    const source = publicInputs(process.env),
      root = await realpath(process.cwd());
    const output = await realpath(process.env.PUBLIC_OUTPUT!),
      temporary = process.env.RUNNER_TEMP!;
    phase = "prepared publication records";
    const preparation = JSON.parse(
      await readFile(path.join(output, "preparation.json"), "utf8"),
    ) as unknown;
    const publication = JSON.parse(
      await readFile(path.join(output, "publication.json"), "utf8"),
    ) as unknown;
    phase = "authenticated release custody";
    assertTunnelReleaseCustody(source, preparation, publication);
    phase = "released service ancestry";
    const publishedSources = requireReleasedService(root, source.head);
    phase = "official pinned metadata";
    const entries = await Promise.all(
      Object.entries(quickTunnelMetadata).map(
        async ([key, url]) => [key, await metadata(url)] as const,
      ),
    );
    const values = Object.fromEntries(entries.map(([key, entry]) => [key, entry.value])) as Record<
      keyof typeof quickTunnelMetadata,
      unknown
    >;
    const release = assertQuickTunnelMetadata(values);
    phase = "private executable directory";
    directory = await createTunnelAssetDirectory(temporary, output);
    phase = "pinned executable acquisition";
    await acquireBinary(directory);
    phase = "acquisition receipt";
    const receipt = {
      source,
      serviceImplementation,
      publishedSources,
      pin: quickTunnelPin,
      release,
      observedAt: new Date().toISOString(),
      directory,
      metadata: entries.map(([key, entry]) => ({
        key,
        bytes: entry.body.length,
        sha256: digest(entry.body),
      })),
      scope:
        "Official metadata and whole owned executable bytes verified. No executable/help/tunnel/product execution or reproducible-build claim.",
    };
    for (const [key, entry] of entries)
      await writeFile(path.join(output, `tunnel-${key}.json`), entry.body, { flag: "wx" });
    await writeFile(
      path.join(output, "tunnel-acquisition.json"),
      JSON.stringify(receipt, null, 2) + "\n",
      { flag: "wx" },
    );
    await appendFile(process.env.GITHUB_ENV!, `VRT_TUNNEL_DIRECTORY=${directory}\n`);
    console.log("Pinned static-gallery tunnel executable verified; no process launched.");
  } catch (error) {
    if (directory) await Promise.allSettled([rm(directory, { recursive: true, force: true })]);
    throw tunnelSetupRefusal(error, phase);
  }
}

if (process.argv[1] && pathToFileURL(path.resolve(process.argv[1])).href === import.meta.url)
  await main();
