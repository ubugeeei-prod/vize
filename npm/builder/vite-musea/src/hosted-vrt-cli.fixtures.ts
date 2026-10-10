import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash, X509Certificate } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
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
  let stderr = Buffer.alloc(0);
  child.stderr.on("data", (chunk: Buffer) => {
    stderr = Buffer.concat([stderr, chunk]).subarray(0, 4096);
  });
  let stdout = "";
  let token = "";
  const exited = new Promise<{ code: number | null; signal: string | null }>((resolve) =>
    child.once("close", (code, signal) => resolve({ code, signal })),
  );
  const receipt: Record<string, unknown> = {
    compiledEntrySha256,
    certificateSha256: sha256(certificate),
    command: "musea-vrt serve",
    galleryUrl,
  };
  let timer: ReturnType<typeof setTimeout> | undefined;
  let failure: unknown;
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
    assert.ok(
      stdout
        .split("\n")
        .includes(
          "  This session token is a live credential; do not share it or retain it in captured logs.",
        ),
      "Compiled CLI must explain the printed session token credential",
    );
    receipt.tokenPrivacyNotice = true;
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
  } catch (error) {
    failure = error;
    receipt.failure = String(error);
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
    } catch (error) {
      receipt.cleanupError = String(error);
      failure ??= error;
    } finally {
      clearTimeout(stopTimer);
      stdout = "";
    }
    receipt.stderr = stderr
      .toString()
      .replace(/Session token: [A-Za-z0-9_-]{43}/g, "Session token: [redacted]");
    if (token) receipt.stderr = String(receipt.stderr).replaceAll(token, "[redacted]");
    try {
      const serialized = JSON.stringify(receipt, null, 2);
      assert.ok(!token || !serialized.includes(token));
      await writeFile(path.join(output, "compiled-cli-lifecycle.json"), serialized);
    } catch (error) {
      failure ??= error;
    } finally {
      token = "";
      stderr = Buffer.alloc(0);
    }
  }
  if (failure !== undefined) throw failure;
  return receipt;
}

/** A real early CLI exit must retain its cause and bounded, credential-free stderr. */
export async function proveCompiledStartupRefusal(
  host: { origin: string; certificatePath: string; spki: string },
  root: string,
  output: string,
) {
  const directory = path.join(output, "compiled-cli-refusal");
  await mkdir(directory, { recursive: true });
  const galleryUrl = `${host.origin.replace("https:", "http:")}/built/gallery/`;
  await assert.rejects(
    proveCompiledSession(host, galleryUrl, root, directory),
    /Compiled CLI exited before listening/,
  );
  const receipt = JSON.parse(
    await readFile(path.join(directory, "compiled-cli-lifecycle.json"), "utf8"),
  );
  assert.deepEqual(receipt.exit, { code: 1, signal: null });
  assert.equal(receipt.failure, "Error: Compiled CLI exited before listening");
  assert.match(receipt.stderr, /\n  Error: Error: serve requires an HTTPS gallery/);
  assert.equal(receipt.stderr.includes("Session token:"), false);
  const commands = [];
  for (const [args, message] of [
    [[], "serve requires --gallery-url"],
    [["--gallery-url", `${host.origin}/missing-gallery/`], "Hosted gallery manifest: HTTP 404"],
  ] as const) {
    const entry = path.join(repository, "npm/builder/vite-musea/dist/cli/index.mjs");
    const child = spawn(
      process.execPath,
      [entry, "serve", ...args, "--config", path.join(root, "absent.vite.config.ts")],
      {
        cwd: root,
        env: { ...process.env, NODE_EXTRA_CA_CERTS: host.certificatePath },
        stdio: ["ignore", "pipe", "pipe"],
      },
    );
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk: Buffer) => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", (chunk: Buffer) => {
      stderr = (stderr + chunk.toString()).slice(0, 4096);
    });
    let timer: ReturnType<typeof setTimeout> | undefined;
    try {
      const exit = await new Promise<{ code: number | null; signal: string | null }>(
        (resolve, reject) => {
          timer = setTimeout(() => {
            child.kill("SIGKILL");
            reject(new Error("Compiled CLI refusal did not exit"));
          }, 30000);
          child.once("error", reject);
          child.once("close", (code, signal) => resolve({ code, signal }));
        },
      );
      assert.deepEqual(exit, { code: 1, signal: null });
      assert.ok(stderr.includes(`\n  Error: Error: ${message}`), stderr);
      assert.equal(stdout.includes("Session token:"), false);
      commands.push({ args, exit, stderr });
    } finally {
      clearTimeout(timer);
      if (child.exitCode === null && child.signalCode === null) child.kill("SIGKILL");
      stdout = "";
    }
  }
  const evidence = { earlyExit: receipt, commands };
  await writeFile(path.join(directory, "startup-refusals.json"), JSON.stringify(evidence, null, 2));
  return evidence;
}
