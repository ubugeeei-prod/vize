import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash, X509Certificate } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { repository, sha256 } from "./hosted-vrt-build.fixtures.ts";

/** Exercise the built bin entry, keeping its printed bearer exclusively in owned memory. */
export async function proveCompiledSession(
  host: { origin: string; certificatePath: string; spki: string },
  galleryUrl: string,
  root: string,
  output: string,
) {
  const certificate = await readFile(host.certificatePath);
  const publicKey = new X509Certificate(certificate).publicKey.export({
    type: "spki",
    format: "der",
  });
  assert.equal(createHash("sha256").update(publicKey).digest("base64"), host.spki);
  const entry = path.join(repository, "npm/builder/vite-musea/dist/cli/index.mjs");
  const compiledEntrySha256 = sha256(await readFile(entry));
  const child = spawn(
    process.execPath,
    [
      entry,
      "serve",
      "--gallery-url",
      galleryUrl,
      "--output",
      path.join(root, "compiled-cli-vrt"),
      "--config",
      path.join(root, "absent.vite.config.ts"),
    ],
    {
      cwd: root,
      env: { ...process.env, NODE_EXTRA_CA_CERTS: host.certificatePath },
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  child.stderr.resume();
  let stdout = "";
  let token = "";
  const exited = new Promise<{ code: number | null; signal: string | null }>((resolve) =>
    child.once("exit", (code, signal) => resolve({ code, signal })),
  );
  const receipt: Record<string, unknown> = {
    compiledEntrySha256,
    certificateSha256: sha256(certificate),
    command: "musea-vrt serve",
    galleryUrl,
  };
  let timer: ReturnType<typeof setTimeout> | undefined;
  try {
    const endpoint = await new Promise<string>((resolve, reject) => {
      timer = setTimeout(() => reject(new Error("Compiled CLI did not become ready")), 30000);
      child.once("error", reject);
      child.once("exit", () => reject(new Error("Compiled CLI exited before listening")));
      child.stdout.on("data", (chunk: Buffer) => {
        stdout += chunk.toString();
        const address = stdout.match(/VRT endpoint: (http:\/\/127\.0\.0\.1:\d+)/)?.[1];
        const bearer = stdout.match(/Session token: ([A-Za-z0-9_-]{43})/)?.[1];
        if (address && bearer) {
          token = bearer;
          resolve(address);
        }
      });
    });
    clearTimeout(timer);
    const actual = await fetch(`${endpoint}/session`, {
      headers: { Origin: host.origin, Authorization: `Bearer ${token}` },
      signal: AbortSignal.timeout(30000),
    });
    const body = Buffer.from(await actual.arrayBuffer());
    assert.equal(actual.status, 200);
    assert.deepEqual(JSON.parse(body.toString()), { version: 1, galleryUrl });
    await writeFile(path.join(output, "compiled-cli-session.json"), body);
    const refusal = await fetch(`${endpoint}/session`, {
      headers: { Origin: host.origin },
      signal: AbortSignal.timeout(30000),
    });
    assert.equal(refusal.status, 401);
    receipt.endpoint = endpoint;
    receipt.session = { status: actual.status, bytes: body.length, sha256: sha256(body) };
    receipt.missingToken = refusal.status;
  } finally {
    clearTimeout(timer);
    if (child.exitCode === null && child.signalCode === null) child.kill("SIGTERM");
    let stopTimer: ReturnType<typeof setTimeout> | undefined;
    try {
      const exit = await Promise.race([
        exited,
        new Promise<never>((_, reject) => {
          stopTimer = setTimeout(() => {
            child.kill("SIGKILL");
            reject(new Error("Compiled CLI did not finish shutdown"));
          }, 15000);
        }),
      ]);
      receipt.exit = exit;
      assert.deepEqual(exit, { code: 0, signal: null });
    } finally {
      clearTimeout(stopTimer);
      stdout = "";
    }
    const serialized = JSON.stringify(receipt, null, 2);
    assert.ok(token && !serialized.includes(token));
    await writeFile(path.join(output, "compiled-cli-lifecycle.json"), serialized);
    token = "";
  }
  return receipt;
}
