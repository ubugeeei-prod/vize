import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { runTools } from "../../../../npm/builder/vite/src/vite-plus/runner.ts";
import type { VizeTaskConfig } from "../../../../npm/builder/vite/src/vite-plus/types.ts";
import type { ResolvedVizeConfig } from "../../../../npm/builder/vite/src/types.ts";
import { resolveConfigExport } from "../../../../npm/builder/vite/src/config.ts";
import {
  binaryRelativePath,
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../../../differential/build-receipt.mjs";
import { resolveTsgoBinary } from "../../../_helpers/realworld-typecheck.ts";
import { originalInputs, repoRoot, sha256 } from "./fixture.ts";
import { sourceTaskConfig } from "./factory.ts";
import { sourceConfigProvider } from "./provider.ts";

export type Report = {
  files: Array<{ file: string; diagnostics: string[]; virtualTs?: string }>;
  programs: Array<{
    root: string;
    tsconfig: string;
    compilerOptions: Record<string, unknown>;
    files: string[];
  }>;
  errorCount: number;
  warningCount: number;
  fileCount: number;
};
type ProcessRow = {
  command: string;
  argv: string[];
  cwd: string;
  status: number | null;
  signal: string | null;
  error: string | null;
  stdout: string;
  stderr: string;
  stdoutBytes: number[];
  stderrBytes: number[];
};
export type Receipt = ProcessRow & { report: Report | null };

export class ProjectOracle {
  readonly root: string;
  readonly binary: string;
  readonly corsa: string;
  readonly metadata: VizeTaskConfig;
  readonly rows: unknown[] = [];
  private readonly output: string;
  private readonly provenance: unknown;

  private constructor(id: string, metadata: VizeTaskConfig, root: string) {
    this.binary = path.join(repoRoot, binaryRelativePath());
    this.corsa = resolveTsgoBinary();
    this.metadata = metadata;
    this.output = path.join(
      repoRoot,
      "target/differential/typechecker/vite-plus-relative-tsconfig",
      `${id}.json`,
    );
    fs.mkdirSync(path.dirname(this.output), { recursive: true });
    const identity = expectedBuildIdentity(repoRoot);
    const build = JSON.parse(fs.readFileSync(`${this.binary}.differential-build.json`, "utf8"));
    validateBuildReceipt(build, identity);
    this.provenance = {
      sourceBuild: build,
      native: { path: this.corsa, sha256: sha256(fs.readFileSync(this.corsa)) },
      node: { executable: process.execPath, version: process.version },
      lockSha256: sha256(fs.readFileSync(path.join(repoRoot, "pnpm-lock.yaml"))),
      sourceModules: Object.fromEntries(
        [
          "npm/builder/vite/src/vite-plus.ts",
          "npm/builder/vite/src/vite-plus/config.ts",
          "npm/builder/vite/src/vite-plus/config-paths.ts",
          "npm/builder/vite/src/vite-plus/runner.ts",
        ].map((file) => [file, sha256(fs.readFileSync(path.join(repoRoot, file)))]),
      ),
      providers: Object.fromEntries(
        ["vue/package.json", "vite-plus/package.json"].map((specifier) => {
          const resolved = fs.realpathSync(createRequire(import.meta.url).resolve(specifier));
          const bytes = fs.readFileSync(resolved);
          return [
            specifier,
            {
              path: resolved,
              sha256: sha256(bytes),
              ...(specifier.endsWith("package.json")
                ? { manifest: JSON.parse(bytes.toString("utf8")) }
                : {}),
            },
          ];
        }),
      ),
      original: originalInputs().manifest,
      configProvider: sourceConfigProvider(),
      dispatcher:
        "Source runTools/runNative selects argv; execute callback launches the default-source Rust CLI directly instead of the published Node launcher.",
    };
    this.root = root;
  }

  static async create(id: string): Promise<ProjectOracle> {
    const original = originalInputs();
    const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), `vize-8017-${id}-`)));
    for (const [name, content] of Object.entries(original.inputs)) {
      const file = path.join(root, name);
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, content);
    }
    fs.mkdirSync(path.join(root, "node_modules"));
    for (const name of ["vue", "vite-plus"]) {
      const manifest = createRequire(import.meta.url).resolve(`${name}/package.json`);
      fs.symlinkSync(
        fs.realpathSync(path.dirname(manifest)),
        path.join(root, "node_modules", name),
        process.platform === "win32" ? "junction" : "dir",
      );
    }
    const before = process.cwd();
    process.chdir(root);
    try {
      const { metadata, ...provider } = await sourceTaskConfig(original.inputs["vite.config.mts"]);
      const oracle = new ProjectOracle(id, metadata, root);
      oracle.rows.push({
        ...provider,
        originalDefineConfig: original.inputs["vite.config.mts"],
        cwd: root,
      });
      oracle.save();
      return oracle;
    } catch (error) {
      fs.rmSync(root, { recursive: true, force: true });
      throw error;
    } finally {
      process.chdir(before);
    }
  }

  write(name: string, content: string): void {
    const file = path.resolve(this.root, name);
    assert(file.startsWith(`${this.root}${path.sep}`));
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, content);
  }

  direct(label: string, args: string[], json = true): Receipt {
    const argv = [
      "check",
      ...args,
      ...(json ? ["--format", "json"] : []),
      "--quiet",
      "--corsa-path",
      this.corsa,
    ];
    return this.capture(label, argv);
  }

  async task(
    label: string,
    args: string[] = [],
    config?: Partial<ResolvedVizeConfig>,
  ): Promise<Receipt> {
    const before = process.cwd();
    const metadata = config === undefined ? this.metadata : { ...this.metadata, config };
    let receipt: Receipt | undefined;
    let temporaryConfig = "";
    const taskArgs = [...args, "--format", "json", "--quiet", "--corsa-path", this.corsa];
    process.chdir(this.root);
    try {
      const originalMetadata = await resolveConfigExport(metadata.config ?? {}, {
        mode: "production",
        command: "check",
      });
      const status = await runTools(
        "typecheck",
        taskArgs,
        metadata,
        "unused-vp",
        this.binary,
        async (command, argv) => {
          assert.equal(command, process.execPath);
          assert.equal(argv[0], this.binary);
          assert.equal(argv[1], "check");
          assert.equal(argv[2], "--config");
          temporaryConfig = argv[3];
          const raw = fs.readFileSync(temporaryConfig);
          this.rows.push({
            label,
            temporaryConfig,
            configBytes: Array.from(raw),
            config: JSON.parse(raw.toString("utf8")),
          });
          receipt = this.capture(label, argv.slice(1));
          return receipt.status ?? 1;
        },
      );
      assert(receipt, "an enabled typecheck task must launch exactly one native check");
      assert.equal(status, receipt.status);
      assert.equal(
        fs.existsSync(temporaryConfig),
        false,
        "temporary config must be removed after a failed check too",
      );
      assert.deepEqual(
        await resolveConfigExport(metadata.config ?? {}, { mode: "production", command: "check" }),
        originalMetadata,
      );
      return receipt;
    } finally {
      process.chdir(before);
      this.save();
    }
  }

  finish(): void {
    this.save();
    fs.rmSync(this.root, { recursive: true, force: true });
  }

  async disabledFrontend(): Promise<{ status: number; launches: number }> {
    const input = originalInputs().inputs["vite.config.mts"].replace(
      'typecheck: { tsconfig: "tsconfig.app.json" }',
      "typecheck: false",
    );
    const before = process.cwd();
    let status: number | null = null;
    let launches = 0;
    process.chdir(this.root);
    try {
      const { metadata } = await sourceTaskConfig(input);
      assert.equal(metadata.options.check, false);
      status = await runTools("typecheck", [], metadata, "unused-vp", this.binary, async () => {
        launches += 1;
        throw new Error("Disabled source task attempted native execution");
      });
      return { status, launches };
    } finally {
      this.rows.push({
        kind: "source-disabled-no-launch",
        input,
        cwd: this.root,
        status,
        launches,
      });
      process.chdir(before);
      this.save();
    }
  }

  private capture(label: string, argv: string[]): Receipt {
    const result = spawnSync(this.binary, argv, {
      cwd: this.root,
      env: { ...process.env, LANG: "C", LC_ALL: "C" },
      timeout: 120_000,
      maxBuffer: 64 * 1024 * 1024,
    });
    const row: ProcessRow = {
      command: this.binary,
      argv,
      cwd: this.root,
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      stdout: result.stdout?.toString("utf8") ?? "",
      stderr: result.stderr?.toString("utf8") ?? "",
      stdoutBytes: Array.from(result.stdout ?? []),
      stderrBytes: Array.from(result.stderr ?? []),
    };
    this.rows.push({ label, inputs: this.inputs(), process: row });
    this.save(); // Preserve complete raw streams and inputs before parsing or assertions.
    assert.equal(row.error, null, row.error ?? "");
    assert.equal(row.signal, null);
    return { ...row, report: row.stdout.trim() ? (JSON.parse(row.stdout) as Report) : null };
  }

  private inputs(): Record<string, { source: string; sha256: string }> {
    const files: Record<string, { source: string; sha256: string }> = {};
    const visit = (directory: string) => {
      for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
        if (entry.name === "node_modules" || entry.name === ".vize") continue;
        const file = path.join(directory, entry.name);
        if (entry.isDirectory()) visit(file);
        else if (entry.isFile()) {
          const source = fs.readFileSync(file, "utf8");
          files[path.relative(this.root, file).replaceAll("\\", "/")] = {
            source,
            sha256: sha256(source),
          };
        }
      }
    };
    visit(this.root);
    return files;
  }

  private save(): void {
    fs.writeFileSync(
      this.output,
      `${JSON.stringify({ provenance: this.provenance, rows: this.rows }, null, 2)}\n`,
    );
  }
}
