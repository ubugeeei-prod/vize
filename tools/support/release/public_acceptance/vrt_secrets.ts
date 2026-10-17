import assert from "node:assert/strict";
import { fstatSync, fsyncSync, writeSync } from "node:fs";
import { lstat, readFile, realpath, writeFile } from "node:fs/promises";
import path from "node:path";
import { inventory, type Evidence } from "./vrt_artifacts.ts";
import { sha256 } from "./vrt_fixtures.ts";
import { scanRetained } from "./vrt_retained_scan.ts";

const receiptName = "vrt-log-barrier.json";
const schema = "vize-public-vrt-log-barrier-v1";

const diagnosticPhases = [
  "observer-entry",
  "observer-cleanup",
  "installed-native-custody",
  "sealed-delivered-inputs",
  "public-chromium",
  "configured-public-api",
  "mandatory-public-hosted-cli-and-audits",
  "mandatory-normally-trusted-public-https",
  "final-installed-custody",
  "trusted-config-task",
  "trusted-static-host",
  "trusted-tunnel",
  "trusted-session",
  "trusted-browser-context",
  "trusted-cdp",
  "trusted-observation",
  "trusted-cleanup-cdp",
  "trusted-cleanup-context",
  "trusted-cleanup-session",
  "trusted-cleanup-tunnel",
  "trusted-cleanup-host",
  "trusted-cleanup-config",
  "hosted-audit-installation",
  "hosted-audit-build",
  "hosted-audit-vendor-bytes",
  "hosted-audit-previews",
  "hosted-audit-run-all",
  "hosted-audit-positive-results",
  "hosted-audit-individual",
  "hosted-audit-browser-version",
  "hosted-audit-vendor-requests",
  "hosted-audit-vendor-refusal",
  "closure-barrier",
  "closure-trusted-surfaces",
  "closure-published-children",
  "seal-owned-writes",
  "seal-log-identity",
  "seal-stdio-drain",
  "seal-retained-scan",
  "seal-private-scan",
  "seal-recheck",
  "seal-receipt",
  "seal-complete",
  "promotion-inputs",
  "promotion-whole-files",
  "promotion-retained-fallback",
  "promotion-cleanup",
] as const;
export type VrtPhase = (typeof diagnosticPhases)[number];
export type VrtDisposition = "entered" | "passed" | "failed";
type Diagnostic = (phase: VrtPhase, disposition: VrtDisposition) => void;

/** Fixed vocabulary only: never accept an error, path, URL, credential or derived fingerprint. */
export function vrtDiagnosticRecord(phase: VrtPhase, disposition: VrtDisposition): string {
  if (!diagnosticPhases.includes(phase) || !["entered", "passed", "failed"].includes(disposition))
    throw new Error("Invalid VRT diagnostic vocabulary");
  return JSON.stringify({ schema: "vize-public-vrt-diagnostic-v1", phase, disposition }) + "\n";
}

/** The reviewed shell opens fd3 before redirecting all private stdout/stderr. */
export function writeVrtDiagnostic(phase: VrtPhase, disposition: VrtDisposition): void {
  writeSync(3, vrtDiagnosticRecord(phase, disposition));
}

/** Campaign credentials stay private through failure evidence, cleanup and the closed-writer scan. */
export class VrtSecrets {
  readonly #diagnostic?: Diagnostic;
  constructor(diagnostic?: Diagnostic) {
    this.#diagnostic = diagnostic;
  }
  #tokens: Buffer[] = [];
  #writes = new Set<Promise<void>>();
  #writeFailures = 0;
  #closed = false;
  #phase: VrtPhase = "observer-entry";

  stage(phase: VrtPhase): void {
    if (this.report(phase, "entered")) this.#phase = phase;
  }

  failed(phase = this.#phase): void {
    this.report(phase, "failed");
  }

  report(phase: VrtPhase, disposition: VrtDisposition): boolean {
    try {
      vrtDiagnosticRecord(phase, disposition);
      this.#diagnostic?.(phase, disposition);
      return true;
    } catch {
      // Diagnostic channel failure refuses acceptance but never interrupts owned cleanup or scans.
      process.exitCode ||= 1;
      return false;
    }
  }

  remember(token: string): void {
    assert.equal(this.#closed, false, "Credential scope already closed");
    assert.ok(/^[A-Za-z0-9_-]{43}$/.test(token), "Invalid session credential format");
    const decoded = Buffer.from(token, "base64url");
    assert.equal(decoded.length, 32);
    assert.ok(decoded.toString("base64url") === token, "Noncanonical session credential");
    const candidate = Buffer.from(token);
    if (this.#tokens.some((value) => value.equals(candidate))) candidate.fill(0);
    else this.#tokens.push(candidate);
    decoded.fill(0);
  }

  #unsafe(bytes: Uint8Array | string): boolean {
    return this.#tokens.some((token) => Buffer.from(bytes).includes(token));
  }

  #check(bytes: Uint8Array | string): void {
    assert.ok(!this.#unsafe(bytes), "Credential-bearing retained data refused");
  }

  /** Use before any new evidence write; no credential or fingerprint is returned. */
  assertSafe(bytes: Uint8Array | string): void {
    assert.equal(this.#closed, false, "Retained writer already closed");
    this.#check(bytes);
  }

  /** Original evidence bytes are forwarded unchanged; only writer custody is added. */
  evidence(original: Evidence): Evidence {
    const save = (name: string, bytes: Uint8Array | string): Promise<void> => {
      this.assertSafe(name);
      this.assertSafe(bytes);
      const writing = original.save(name, bytes);
      this.#writes.add(writing);
      void writing.then(
        () => this.#writes.delete(writing),
        () => {
          this.#writeFailures++;
          this.#writes.delete(writing);
        },
      );
      return writing;
    };
    return {
      output: original.output,
      records: original.records,
      save,
      json: (name, value) => save(name, JSON.stringify(value, null, 2) + "\n"),
    };
  }

  #text(text: string): string {
    let safe = text;
    for (const token of this.#tokens) safe = safe.replaceAll(token.toString(), "[redacted]");
    return safe;
  }

  sanitizeError(error: unknown, depth = 0): unknown {
    if (this.#tokens.length === 0) return error;
    if (depth > 4) return new Error("Additional nested failure omitted");
    const message = this.#text(error instanceof Error ? error.message : String(error));
    const safe =
      error instanceof AggregateError
        ? new AggregateError(
            (error.errors as unknown[]).map((item) => this.sanitizeError(item, depth + 1)),
            message,
          )
        : new Error(message);
    if (error instanceof Error) {
      safe.name = this.#text(error.name);
      safe.stack = error.stack ? this.#text(error.stack) : undefined;
      if (error.cause !== undefined) safe.cause = this.sanitizeError(error.cause, depth + 1);
    }
    return safe;
  }

  async closeBarrier(
    trustedSurfaces: () => void | Promise<void>,
    publishedChildren: () => void | Promise<void>,
  ): Promise<void> {
    this.stage("closure-barrier");
    const checks = [
      ["closure-trusted-surfaces", trustedSurfaces],
      ["closure-published-children", publishedChildren],
    ] as const;
    const results = await Promise.allSettled(
      checks.map(([phase, verify]) =>
        Promise.resolve().then(async () => {
          try {
            await verify();
            this.report(phase, "passed");
          } catch (error) {
            this.failed(phase);
            throw error;
          }
        }),
      ),
    );
    assert.ok(
      results.every((result) => result.status === "fulfilled"),
      "Owned VRT closure barrier refused",
    );
  }

  /** No timers, child streams or evidence writes may remain owned when this starts. */
  async seal(log: string, retainedRoot?: string): Promise<void> {
    this.stage("seal-owned-writes");
    assert.equal(this.#closed, false);
    this.#closed = true;
    await Promise.allSettled(this.#writes);
    if (this.#writeFailures > 0) {
      console.error("VRT retained writer failure; acceptance refused");
      process.exitCode ||= 1;
    }
    this.stage("seal-log-identity");
    const output = await realpath(path.dirname(log));
    assert.equal(path.basename(log), "vrt.log");
    const info = await lstat(log, { bigint: true });
    assert.equal(info.isSymbolicLink(), false);
    assert.equal(info.isFile(), true);
    this.stage("seal-stdio-drain");
    for (const [stream, descriptor] of [
      [process.stdout, 1],
      [process.stderr, 2],
    ] as const) {
      const fd = fstatSync(descriptor, { bigint: true });
      assert.ok(fd.isFile(), "VRT log requires owned regular-file stdout/stderr");
      assert.equal(fd.dev, info.dev);
      assert.equal(fd.ino, info.ino);
      await new Promise<void>((resolve, reject) =>
        stream.write("", (error) => (error ? reject(error) : resolve())),
      );
      fsyncSync(descriptor);
    }
    const retainedOutput = retainedRoot ? await realpath(retainedRoot) : undefined;
    if (retainedOutput) {
      this.stage("seal-retained-scan");
      assert.ok(output !== retainedOutput && !output.startsWith(retainedOutput + path.sep));
      assert.ok(!retainedOutput.startsWith(output + path.sep));
    }
    if (retainedOutput) {
      const quarantined = await scanRetained(output, retainedOutput, (bytes) =>
        this.#unsafe(bytes),
      );
      assert.equal(quarantined, false, "Unsafe retained artifact quarantined; acceptance refused");
    }
    this.stage("seal-private-scan");
    const retainedFiles = retainedOutput ? await inventory(retainedOutput) : undefined;
    const before = await inventory(output);
    assert.equal(
      before.some((item) => item.path === receiptName),
      false,
    );
    for (const item of before) {
      this.#check(item.path);
      this.#check(await readFile(path.join(output, item.path)));
    }
    if (retainedOutput && retainedFiles) {
      for (const item of retainedFiles) {
        this.#check(item.path);
        this.#check(await readFile(path.join(retainedOutput, item.path)));
      }
      assert.deepEqual(await inventory(retainedOutput), retainedFiles);
    }
    this.stage("seal-recheck");
    assert.deepEqual(await inventory(output), before, "Retained bytes changed during scan");
    const body =
      JSON.stringify(
        {
          schema,
          stdoutAndStderrRegularFileDrained: true,
          allOwnedWritersClosed: true,
          allRetainedArtifactsScanned: true,
          credentialCount: this.#tokens.length,
          retainedWriterFailures: this.#writeFailures,
          files: before,
          retainedOutput,
          retainedFiles,
        },
        null,
        2,
      ) + "\n";
    this.stage("seal-receipt");
    this.#check(body);
    await writeFile(path.join(output, receiptName), body, { flag: "wx" });
    this.report("seal-complete", "passed");
  }

  erase(): void {
    for (const token of this.#tokens) token.fill(0);
    this.#tokens.length = 0;
    this.#closed = true;
  }
}

/** Executed only after observer exit; never emits an unsealed or changed log. */
export async function verifiedVrtFiles(log: string, retainedRoot?: string) {
  assert.equal(path.basename(log), "vrt.log");
  const output = await realpath(path.dirname(log));
  const receiptPath = path.join(output, receiptName);
  const receiptInfo = await lstat(receiptPath);
  assert.equal(receiptInfo.isSymbolicLink(), false);
  assert.equal(receiptInfo.isFile(), true);
  const receiptBytes = await readFile(receiptPath);
  const receipt = JSON.parse(receiptBytes.toString()) as {
    schema: string;
    stdoutAndStderrRegularFileDrained: boolean;
    allOwnedWritersClosed: boolean;
    allRetainedArtifactsScanned: boolean;
    credentialCount: number;
    retainedWriterFailures: number;
    files: Awaited<ReturnType<typeof inventory>>;
    retainedOutput?: string;
    retainedFiles?: Awaited<ReturnType<typeof inventory>>;
  };
  assert.equal(receipt.schema, schema);
  assert.equal(receipt.stdoutAndStderrRegularFileDrained, true);
  assert.equal(receipt.allOwnedWritersClosed, true);
  assert.equal(receipt.allRetainedArtifactsScanned, true);
  assert.ok(Number.isSafeInteger(receipt.credentialCount) && receipt.credentialCount >= 0);
  assert.ok(
    Number.isSafeInteger(receipt.retainedWriterFailures) && receipt.retainedWriterFailures >= 0,
  );
  if (retainedRoot) {
    assert.equal(await realpath(retainedRoot), receipt.retainedOutput);
    assert.ok(
      JSON.stringify(await inventory(retainedRoot)) === JSON.stringify(receipt.retainedFiles),
      "Original native/browser retained bytes changed before promotion",
    );
  } else assert.equal(receipt.retainedOutput, undefined);
  const files = await inventory(output);
  const barrier = files.find((item) => item.path === receiptName);
  assert.ok(
    barrier &&
      barrier.sha256 === sha256(receiptBytes) &&
      barrier.size === String(receiptBytes.length),
  );
  const actual = files.filter((item) => item.path !== receiptName);
  assert.ok(
    JSON.stringify(actual) === JSON.stringify(receipt.files),
    "Closed retained evidence changed after observer exit",
  );
  const recorded = actual.find((item) => item.path === "vrt.log");
  assert.ok(recorded);
  const bytes = await readFile(log);
  assert.equal(sha256(bytes), recorded.sha256);
  assert.equal(String(bytes.length), recorded.size);
  return { log: bytes, files };
}

export async function verifiedVrtLog(log: string, retainedRoot?: string): Promise<Buffer> {
  return (await verifiedVrtFiles(log, retainedRoot)).log;
}
