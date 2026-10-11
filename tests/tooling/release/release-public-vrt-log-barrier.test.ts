import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdir, mkdtemp, open, readFile, realpath, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { fileURLToPath } from "node:url";
import { inventory } from "../../../tools/support/release/public_acceptance/vrt_artifacts.ts";
import { sha256 } from "../../../tools/support/release/public_acceptance/vrt_fixtures.ts";
import {
  VrtSecrets,
  verifiedVrtLog,
} from "../../../tools/support/release/public_acceptance/vrt_secrets.ts";

const fixture = fileURLToPath(
  new URL("../fixtures/release/vrt-log-barrier/observer.fixture.ts", import.meta.url),
);

async function owned(t: TestContext) {
  const root = await mkdtemp(path.join(os.tmpdir(), "vize-public-vrt-log-control-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  return root;
}

async function run(root: string, mode: string, pipe = false, retained?: string) {
  const log = path.join(root, "vrt.log");
  const file = await open(log, "wx", 0o600);
  let code: number | null = null;
  let signal: NodeJS.Signals | null = null;
  try {
    const child = spawn(process.execPath, [fixture, root, mode], {
      stdio: pipe ? ["ignore", "pipe", "pipe"] : ["ignore", file.fd, file.fd],
      env: retained ? { ...process.env, VRT_RETAINED_OUTPUT: retained } : undefined,
    });
    child.stdout?.resume();
    child.stderr?.resume();
    await new Promise<void>((resolve, reject) => {
      const timer = setTimeout(() => {
        child.kill("SIGKILL");
        reject(new Error("Bounded log control did not close"));
      }, 5_000);
      child.once("error", (error) => {
        clearTimeout(timer);
        reject(error);
      });
      child.once("close", (status, reason) => {
        clearTimeout(timer);
        code = status;
        signal = reason;
        resolve();
      });
    });
  } finally {
    await file.close();
  }
  assert.equal(signal, null);
  return { log, code };
}

// Helper/file custody only: no product, compiler, browser, server, tunnel or live session is used.
test("owned regular-file stdout/stderr drain before the complete scan and post-exit publication", async (t) => {
  const root = await owned(t);
  const { log, code } = await run(root, "plain");
  assert.equal(code, 0);
  const receipt = JSON.parse(await readFile(path.join(root, "vrt-log-barrier.json"), "utf8"));
  assert.equal(receipt.credentialCount, 2);
  assert.equal(receipt.retainedWriterFailures, 0);
  assert.equal(receipt.stdoutAndStderrRegularFileDrained, true);
  assert.equal(receipt.allRetainedArtifactsScanned, true);
  assert.deepEqual(await verifiedVrtLog(log), await readFile(log));
  assert.deepEqual(
    await readFile(path.join(root, "evidence/body.bin")),
    Buffer.from([0, 255, 128, 10]),
  );
  assert.equal(
    await readFile(path.join(root, "evidence/cleanup-end.bin"), "utf8"),
    "cleanup completed",
  );
  await assert.rejects(readFile(path.join(root, "evidence/late.bin")), { code: "ENOENT" });
  const redacted = await readFile(path.join(root, "evidence/redacted.json"), "utf8");
  assert.ok(redacted.includes("[redacted]"));
  for (const byte of [2, 3])
    assert.equal(redacted.includes(Buffer.alloc(32, byte).toString("base64url")), false);
});

test("the pending real filesystem write is included before the closed-writer barrier", async (t) => {
  const root = await owned(t);
  const { log, code } = await run(root, "pending");
  assert.equal(code, 0);
  assert.deepEqual(
    await readFile(path.join(root, "evidence/pending.bin")),
    Buffer.alloc(1024 * 1024, 11),
  );
  assert.deepEqual(await verifiedVrtLog(log), await readFile(log));
});

test("natural acquisition and cleanup refusal preserve primary exit and all later evidence", async (t) => {
  for (const mode of ["failure", "cleanup-failure", "writer-failure"]) {
    const root = await owned(t);
    const { log, code } = await run(root, mode);
    assert.equal(code, 7, mode);
    const failure = JSON.parse(await readFile(path.join(root, "evidence/failure.json"), "utf8"));
    assert.ok(failure.error.includes(mode === "writer-failure" ? "EEXIST" : "missing-acquisition"));
    assert.deepEqual(await verifiedVrtLog(log), await readFile(log));
    assert.equal(
      await readFile(path.join(root, "evidence/cleanup-end.bin"), "utf8"),
      "cleanup completed",
    );
    const observations = JSON.parse(
      await readFile(path.join(root, "evidence/observations.json"), "utf8"),
    );
    assert.equal(observations.success, false);
    if (mode === "cleanup-failure") {
      assert.ok(
        observations.records.some(
          (item: { operation?: string }) => item.operation === "natural-cleanup-refusal",
        ),
      );
      assert.equal(failure.error.includes("already-removed"), false);
    }
    if (mode === "writer-failure") {
      const receipt = JSON.parse(await readFile(path.join(root, "vrt-log-barrier.json"), "utf8"));
      assert.equal(receipt.retainedWriterFailures, 1);
    }
  }
});

test("ALL-file scan refusal emits no success barrier and the verifier withholds the log", async (t) => {
  const root = await owned(t);
  const { log, code } = await run(root, "scan-refusal");
  assert.equal(code, 1);
  await assert.rejects(readFile(path.join(root, "vrt-log-barrier.json")), { code: "ENOENT" });
  await assert.rejects(verifiedVrtLog(log));
});

test("pipes cannot claim the regular-file drain contract", async (t) => {
  const root = await owned(t);
  const { log, code } = await run(root, "plain", true);
  assert.equal(code, 1);
  await assert.rejects(verifiedVrtLog(log));
});

test("post-barrier suffix, extra file, missing file and modified old bytes refuse publication", async (t) => {
  for (const change of ["suffix", "extra", "missing", "body"]) {
    const root = await owned(t);
    const { log, code } = await run(root, change === "suffix" ? "changed-after-barrier" : "plain");
    assert.equal(code, 0);
    if (change === "extra") await writeFile(path.join(root, "extra.bin"), "extra");
    if (change === "missing") await rm(path.join(root, "evidence/body.bin"));
    if (change === "body") await writeFile(path.join(root, "evidence/body.bin"), "changed");
    await assert.rejects(verifiedVrtLog(log), /changed after observer exit/);
  }
});

test("credential format errors and evidence guards never expose supplied control bytes", () => {
  const scope = new VrtSecrets();
  const token = Buffer.alloc(32, 9).toString("base64url");
  scope.remember(token);
  assert.throws(() => scope.assertSafe(token), /Credential-bearing retained data refused/);
  assert.throws(() => scope.remember("!".repeat(43)), /Invalid session credential format/);
  const safe = scope.sanitizeError(new Error(`private ${token}`));
  assert.equal(String(safe).includes(token), false);
  scope.erase();
  assert.throws(() => scope.remember(token), /already closed/);
});

const publisher = fileURLToPath(
  new URL("../../../tools/support/release/public_acceptance/vrt_retained.ts", import.meta.url),
);

async function staged(t: TestContext) {
  const root = await realpath(await owned(t));
  const privateRoot = await mkdtemp(path.join(root, "vize-public-vrt."));
  const retained = path.join(root, "public-release");
  await mkdir(retained);
  await writeFile(path.join(retained, "native.json"), "original native bytes\n");
  await writeFile(path.join(retained, "musea.json"), "original browser bytes\n");
  return { privateRoot, retained };
}

async function publish(log: string, retained: string) {
  const child = spawn(process.execPath, [publisher, log, retained], {
    stdio: ["ignore", "pipe", "pipe"],
  });
  const stdout: Buffer[] = [],
    stderr: Buffer[] = [];
  child.stdout.on("data", (chunk: Buffer) => stdout.push(chunk));
  child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));
  const code = await new Promise<number | null>((resolve, reject) => {
    const timer = setTimeout(() => {
      child.kill("SIGKILL");
      reject(new Error("Bounded promotion control did not close"));
    }, 5_000);
    child.once("error", (error) => {
      clearTimeout(timer);
      reject(error);
    });
    child.once("close", (status) => {
      clearTimeout(timer);
      resolve(status);
    });
  });
  return { code, stdout: Buffer.concat(stdout), stderr: Buffer.concat(stderr) };
}

async function uploadedSafe(retained: string) {
  for (const file of await inventory(retained)) {
    const body = await readFile(path.join(retained, file.path));
    for (const byte of [2, 3]) {
      const control = Buffer.alloc(32, byte).toString("base64url");
      assert.equal(file.path.includes(control), false);
      assert.equal(body.includes(Buffer.from(control)), false);
    }
  }
}

async function originalObservations(retained: string) {
  assert.equal(
    await readFile(path.join(retained, "native.json"), "utf8"),
    "original native bytes\n",
  );
  assert.equal(
    await readFile(path.join(retained, "musea.json"), "utf8"),
    "original browser bytes\n",
  );
}

test("only verified whole private files are promoted with explicit relative mapping", async (t) => {
  const { privateRoot, retained } = await staged(t);
  const { log, code } = await run(privateRoot, "plain", false, retained);
  assert.equal(code, 0);
  const originalLog = await readFile(log);
  const originalBody = await readFile(path.join(privateRoot, "evidence/body.bin"));
  const originalFiles = new Map(
    await Promise.all(
      (await inventory(privateRoot)).map(
        async (file) => [file.path, await readFile(path.join(privateRoot, file.path))] as const,
      ),
    ),
  );
  const result = await publish(log, retained);
  assert.equal(result.code, 0);
  assert.deepEqual(result.stdout, originalLog);
  assert.deepEqual(await readFile(path.join(retained, "vrt.log")), originalLog);
  assert.deepEqual(await readFile(path.join(retained, "evidence/body.bin")), originalBody);
  await originalObservations(retained);
  await uploadedSafe(retained);
  const mapping = JSON.parse(await readFile(path.join(retained, "vrt-promotion.json"), "utf8"));
  assert.equal(mapping.privateRoot, privateRoot);
  assert.equal(mapping.retainedRoot, retained);
  assert.equal(mapping.absolutePathsInOriginalEvidencePreserved, true);
  assert.ok(
    mapping.files.every(
      (file: { privateRelative: string; retainedRelative: string }) =>
        file.privateRelative === file.retainedRelative,
    ),
  );
  assert.deepEqual(
    mapping.files.map((file: { privateRelative: string }) => file.privateRelative).sort(),
    [...originalFiles.keys()].sort(),
  );
  for (const file of mapping.files) {
    const original = originalFiles.get(file.privateRelative);
    assert.ok(original);
    assert.equal(file.sha256, sha256(original));
    assert.equal(file.bytes, String(original.length));
    assert.deepEqual(await readFile(path.join(retained, file.retainedRelative)), original);
  }
  await assert.rejects(readFile(log), { code: "ENOENT" });
});

test("scan refusal never uploads private control bytes and retains fixed refusal only", async (t) => {
  const { privateRoot, retained } = await staged(t);
  const { log, code } = await run(privateRoot, "scan-refusal", false, retained);
  assert.equal(code, 1);
  assert.equal(
    await readFile(path.join(privateRoot, "injected-control.bin"), "utf8"),
    Buffer.alloc(32, 2).toString("base64url"),
  );
  const result = await publish(log, retained);
  assert.equal(result.code, 1);
  assert.equal(result.stdout.length, 0);
  await originalObservations(retained);
  await uploadedSafe(retained);
  assert.deepEqual(JSON.parse(await readFile(path.join(retained, "vrt-refused.json"), "utf8")), {
    schema: "vize-public-vrt-retention-refused-v1",
    success: false,
  });
  await assert.rejects(readFile(path.join(retained, "injected-control.bin")), { code: "ENOENT" });
  await assert.rejects(readFile(path.join(retained, "vrt.log")), { code: "ENOENT" });
  await assert.rejects(readFile(log), { code: "ENOENT" });
});

test("natural terminal EEXIST still finalizes the complete scope and preserves failing observer exit", async (t) => {
  const { privateRoot, retained } = await staged(t);
  const { log, code } = await run(privateRoot, "terminal-write-failure", false, retained);
  assert.equal(code, 9);
  const barrier = JSON.parse(
    await readFile(path.join(privateRoot, "vrt-log-barrier.json"), "utf8"),
  );
  assert.equal(barrier.credentialCount, 2);
  assert.equal(barrier.allRetainedArtifactsScanned, true);
  assert.ok((await readFile(log, "utf8")).includes("EEXIST"));
  assert.equal((await publish(log, retained)).code, 0);
  await originalObservations(retained);
  await uploadedSafe(retained);
});

test("natural destination collision refuses promotion without partial private files or overwrites", async (t) => {
  const { privateRoot, retained } = await staged(t);
  await writeFile(path.join(retained, "evidence"), "pre-existing unrelated evidence\n");
  const { log, code } = await run(privateRoot, "plain", false, retained);
  assert.equal(code, 0);
  const result = await publish(log, retained);
  assert.equal(result.code, 1);
  assert.equal(result.stdout.length, 0);
  assert.equal(
    await readFile(path.join(retained, "evidence"), "utf8"),
    "pre-existing unrelated evidence\n",
  );
  await originalObservations(retained);
  await uploadedSafe(retained);
  await assert.rejects(readFile(path.join(retained, "vrt.log")), { code: "ENOENT" });
  await assert.rejects(readFile(path.join(retained, "vrt-log-barrier.json")), { code: "ENOENT" });
  await assert.rejects(readFile(log), { code: "ENOENT" });
});
