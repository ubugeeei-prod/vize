import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  binaryRelativePath,
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../../../differential/build-receipt.mjs";

export const repoRoot = fileURLToPath(new URL("../../../../", import.meta.url));
export const corpusRoot = path.join(
  repoRoot,
  "tests/_fixtures/differential/typecheck/cli-jsconfig-discovery",
);
export const sha256 = (bytes: Uint8Array | string): string =>
  createHash("sha256").update(bytes).digest("hex");

export type ProcessRow = {
  command: string;
  argv: string[];
  cwd: string;
  status: number | null;
  signal: string | null;
  error: string | null;
  stdout: string;
  stderr: string;
  stdoutBase64: string;
  stderrBase64: string;
};

export class JsconfigOracle {
  readonly root: string;
  readonly binary: string;
  readonly native: string;
  readonly rows: unknown[] = [];
  private readonly output: string;
  private readonly provenance: unknown;

  constructor(id: string) {
    const directory = path.join(repoRoot, "target/differential/typecheck/cli-jsconfig-discovery");
    fs.mkdirSync(directory, { recursive: true });
    this.root = fs.realpathSync(fs.mkdtempSync(path.join(directory, `${id}-`)));
    this.output = path.join(this.root, "observations.json");
    this.binary = path.join(repoRoot, binaryRelativePath());
    const buildBytes = fs.readFileSync(`${this.binary}.differential-build.json`);
    const build = JSON.parse(buildBytes.toString("utf8"));
    const identity = expectedBuildIdentity(repoRoot);
    validateBuildReceipt(build, identity);
    const sdkName = `@typescript/typescript-${process.platform}-${process.arch}`;
    const sdk = fs.realpathSync(path.join(repoRoot, "node_modules", sdkName));
    const manifestBytes = fs.readFileSync(path.join(sdk, "package.json"));
    const manifest = JSON.parse(manifestBytes.toString("utf8"));
    assert.equal(manifest.name, sdkName);
    assert.equal(manifest.version, "7.0.2");
    this.native = fs.realpathSync(
      path.join(sdk, "lib", process.platform === "win32" ? "tsc.exe" : "tsc"),
    );
    const corpusBytes = fs.readFileSync(path.join(corpusRoot, "corpus.json"));
    const corpus = JSON.parse(corpusBytes.toString("utf8"));
    const sourceStatus = this.git(["status", "--porcelain=v1"]);
    assert.equal(sourceStatus, "", "source-built observations require a clean source checkout");
    for (const input of [...corpus.inputFiles, ...corpus.nativeSource]) {
      const bytes = fs.readFileSync(path.join(corpusRoot, input.carrier));
      assert.equal(bytes.length, input.bytes);
      assert.equal(sha256(bytes), input.sha256, input.carrier);
    }
    this.provenance = {
      source: identity,
      sourceBuild: build,
      buildReceiptBase64: buildBytes.toString("base64"),
      buildReceiptSha256: sha256(buildBytes),
      native: {
        path: this.native,
        sha256: sha256(fs.readFileSync(this.native)),
        manifestPath: path.join(sdk, "package.json"),
        manifest,
        manifestBase64: manifestBytes.toString("base64"),
        manifestSha256: sha256(manifestBytes),
      },
      node: { executable: process.execPath, version: process.version },
      lock: {
        path: "pnpm-lock.yaml",
        sha256: sha256(fs.readFileSync(path.join(repoRoot, "pnpm-lock.yaml"))),
      },
      corpus,
      corpusSha256: sha256(corpusBytes),
      sourceStatus,
      sourceTree: this.git(["rev-parse", "HEAD^{tree}"]),
    };
    this.save();
    const version = this.capture("native-version", this.native, ["--version"]);
    assert.equal(version.status, 0, version.stderr);
    assert.equal(version.stdout, "Version 7.0.2\n");
    assert.equal(version.stderr, "");
    const cliVersion = this.capture("source-cli-version", this.binary, ["--version"]);
    assert.equal(cliVersion.status, 0, cliVersion.stderr);
    assert.equal(cliVersion.stdout, `${identity.cliVersion}\n`);
    assert.equal(cliVersion.stderr, "");
  }

  carrier(name: string): string {
    return fs.readFileSync(path.join(corpusRoot, name), "utf8");
  }

  write(name: string, source: string): void {
    const file = path.resolve(this.root, name);
    assert(file.startsWith(`${this.root}${path.sep}`));
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, source);
  }

  check(label: string, args: string[] = [], cwd = this.root): ProcessRow {
    return this.capture(
      label,
      this.binary,
      ["check", ...args, "--format", "json", "--quiet", "--corsa-path", this.native],
      cwd,
    );
  }

  stock(label: string, config: string, args: string[] = [], cwd = this.root): ProcessRow {
    return this.capture(
      label,
      this.native,
      ["--noEmit", "--pretty", "false", "--project", config, ...args],
      cwd,
    );
  }

  finish(error: unknown = null): void {
    this.rows.push({ kind: "finished", error: error === null ? null : String(error) });
    this.save();
  }

  private capture(label: string, command: string, argv: string[], cwd = this.root): ProcessRow {
    const beforeInputs = this.inputs();
    const result = spawnSync(command, argv, {
      cwd,
      env: { ...process.env, LANG: "C", LC_ALL: "C", NO_COLOR: "1" },
      timeout: 120_000,
      maxBuffer: 16 * 1024 * 1024,
    });
    const row: ProcessRow = {
      command,
      argv,
      cwd,
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      stdout: result.stdout?.toString("utf8") ?? "",
      stderr: result.stderr?.toString("utf8") ?? "",
      stdoutBase64: result.stdout?.toString("base64") ?? "",
      stderrBase64: result.stderr?.toString("base64") ?? "",
    };
    const afterInputs = this.inputs();
    this.rows.push({ label, beforeInputs, afterInputs, process: row });
    this.save();
    assert.deepEqual(afterInputs, beforeInputs, `${label}: command changed authored inputs`);
    assert.equal(row.signal, null);
    assert.equal(row.error, null, row.error ?? "");
    return row;
  }

  private inputs(): Record<string, unknown> {
    const files: Record<string, unknown> = {};
    const visit = (directory: string) => {
      for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
        const file = path.join(directory, entry.name);
        if (file === this.output || entry.name === "node_modules" || entry.name === ".vize")
          continue;
        if (entry.isDirectory()) visit(file);
        else if (entry.isFile()) {
          const bytes = fs.readFileSync(file);
          files[path.relative(this.root, file).replaceAll("\\", "/")] = {
            bytes: bytes.length,
            sha256: sha256(bytes),
            source: bytes.toString("utf8"),
            base64: bytes.toString("base64"),
          };
        }
      }
    };
    visit(this.root);
    return files;
  }

  private git(argv: string[]): string {
    const result = spawnSync("git", argv, { cwd: repoRoot, encoding: "utf8" });
    assert.equal(result.status, 0, result.stderr);
    return result.stdout.trim();
  }

  private save(): void {
    fs.writeFileSync(
      this.output,
      `${JSON.stringify({ provenance: this.provenance, rows: this.rows }, null, 2)}\n`,
    );
  }
}
