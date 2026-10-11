import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { lstat, mkdir, readFile, realpath, writeFile } from "node:fs/promises";
import path from "node:path";

export const quickTunnelPin = {
  version: "2026.10.0",
  releaseId: 403974563,
  assetId: 613255386,
  bytes: 40129756,
  sha256: "d33ff2d14475178d2012c2c56beba87389ac5ded27649519f198a7d3134a99db",
  source: "18cdfe0a6fc7b72a0702d255a1f984e776ce0498",
  url: "https://github.com/cloudflare/cloudflared/releases/download/2026.10.0/cloudflared-linux-amd64",
} as const;

const allowedEnvironment = ["HOME", "PATH", "TMPDIR", "LANG", "LC_ALL", "TZ"] as const;

/** Explicit allowlist preserves HOME and never forwards account, trust, debug or proxy settings. */
export function quickTunnelEnvironment(parent: NodeJS.ProcessEnv): NodeJS.ProcessEnv {
  assert.ok(typeof parent.HOME === "string" && parent.HOME.length > 0, "Original HOME required");
  return Object.fromEntries(
    allowedEnvironment
      .filter((key) => typeof parent[key] === "string")
      .map((key) => [key, parent[key]]),
  );
}

export function quickTunnelArguments(config: string, origin: string): string[] {
  let url: URL;
  try {
    url = new URL(origin);
  } catch {
    throw new Error("Owned static loopback origin required");
  }
  assert.ok(
    url.protocol === "http:" &&
      url.hostname === "127.0.0.1" &&
      url.port !== "" &&
      origin === url.origin &&
      path.isAbsolute(config),
    "Owned static loopback origin and absolute config required",
  );
  return [
    "tunnel",
    "--config",
    config,
    "--no-autoupdate",
    "--metrics",
    "127.0.0.1:0",
    "--url",
    origin,
  ];
}

/** Caller owns the containing task directory; no default config is opened by this helper. */
export async function writeQuickTunnelConfig(taskDirectory: string): Promise<string> {
  assert.ok(path.isAbsolute(taskDirectory), "Absolute owned config directory required");
  await mkdir(taskDirectory, { mode: 0o700 });
  const config = path.join(taskDirectory, "config.yml");
  await writeFile(config, "{}\n", { flag: "wx", mode: 0o600 });
  return config;
}

export async function assertQuickTunnelConfig(
  taskDirectory: string,
  config: string,
): Promise<void> {
  assert.ok(path.isAbsolute(taskDirectory) && path.isAbsolute(config));
  const rootInfo = await lstat(taskDirectory);
  const fileInfo = await lstat(config);
  assert.ok(rootInfo.isDirectory() && !rootInfo.isSymbolicLink());
  assert.ok(fileInfo.isFile() && !fileInfo.isSymbolicLink());
  assert.equal(rootInfo.mode & 0o777, 0o700);
  assert.equal(fileInfo.mode & 0o777, 0o600);
  const root = await realpath(taskDirectory);
  assert.ok((await realpath(config)) === path.join(root, "config.yml"));
  assert.ok((await readFile(config)).equals(Buffer.from("{}\n")), "Owned empty YAML changed");
}

/** Read only a separately approved downloaded asset; this module never acquires an executable. */
export async function assertQuickTunnelBinary(
  assetDirectory: string,
  binary: string,
): Promise<void> {
  const info = await lstat(binary);
  assert.ok(info.isFile() && !info.isSymbolicLink());
  const root = await realpath(assetDirectory);
  assert.ok((await realpath(binary)) === path.join(root, "cloudflared-linux-amd64"));
  assert.equal(info.size, quickTunnelPin.bytes, "Pinned executable length mismatch");
  const bytes = await readFile(binary);
  assert.equal(bytes.length, quickTunnelPin.bytes);
  assert.ok(
    createHash("sha256").update(bytes).digest("hex") === quickTunnelPin.sha256,
    "Pinned executable digest mismatch",
  );
}

export function quickTunnelHostname(value: string): string {
  let url: URL;
  try {
    url = new URL(value);
  } catch {
    throw new Error("Anonymous HTTPS Quick Tunnel hostname required");
  }
  assert.ok(
    value === url.origin &&
      url.protocol === "https:" &&
      url.port === "" &&
      url.hostname !== "api.trycloudflare.com" &&
      /^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.trycloudflare\.com$/.test(url.hostname),
    "Anonymous HTTPS Quick Tunnel hostname required",
  );
  return url.origin;
}

/** Tagged LogTable rows only; all other provisioning output stays volatile and unretained. */
export class QuickTunnelAnnouncement {
  #announced = false;
  #hostname: string | undefined;

  line(value: string): string | undefined {
    assert.ok(Buffer.byteLength(value) <= 16_384, "Quick Tunnel log row too large");
    if (value.includes("Your quick Tunnel has been created! Visit it at")) {
      assert.equal(this.#announced, false, "Repeated Quick Tunnel provisioning refused");
      this.#announced = true;
    }
    if (!this.#announced) return;
    const row = /\|\s*(https:\/\/[^\s|]+)\s*\|\s*$/.exec(value);
    if (!row) return;
    const hostname = quickTunnelHostname(row[1]);
    assert.ok(!this.#hostname || hostname === this.#hostname, "Quick Tunnel hostname changed");
    return (this.#hostname = hostname);
  }
}
