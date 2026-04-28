import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { compileScript, compileTemplate, parse as parseSfc } from "@vue/compiler-sfc";
import { CORSA_BIN, VIZE_BIN, type AppConfig } from "./apps.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

interface CommandResult {
  status: number | null;
  stdout: string;
  stderr: string;
}

interface VizeCheckJson {
  errorCount: number;
  fileCount: number;
  files: Array<{
    diagnostics: string[];
    file: string;
  }>;
}

export function hasToolchainParityBinaries(): boolean {
  return fs.existsSync(VIZE_BIN) && fs.existsSync(CORSA_BIN) && resolveBin("vue-tsc") != null;
}

export function assertOfficialCompilerAccepts(app: AppConfig): void {
  const check = requireCheckConfig(app);
  const files = collectVueFiles(check.cwd, check.patterns);

  assert.ok(files.length > 0, `${app.name} should have Vue fixtures for compiler parity`);

  for (const file of files) {
    const source = fs.readFileSync(file, "utf8");
    const filename = path.relative(check.cwd, file).split(path.sep).join("/");
    const parsed = parseSfc(source, { filename });

    assert.deepEqual(
      parsed.errors.map(formatCompilerError),
      [],
      `${app.name}:${filename} should parse with @vue/compiler-sfc`,
    );

    const descriptor = parsed.descriptor;
    const script =
      descriptor.script || descriptor.scriptSetup
        ? compileScript(descriptor, { id: filename })
        : undefined;

    if (descriptor.template) {
      const template = compileTemplate({
        compilerOptions: {
          bindingMetadata: script?.bindings,
          mode: "module",
        },
        filename,
        id: filename,
        source: descriptor.template.content,
      });

      assert.deepEqual(
        template.errors.map(formatCompilerError),
        [],
        `${app.name}:${filename} template should compile with @vue/compiler-sfc`,
      );
    }
  }
}

export function assertVueTscDiagnosticSurface(
  app: AppConfig,
  options: { expectErrors: boolean },
): void {
  const check = requireCheckConfig(app);
  const vize = runVizeCheck(check.cwd, check.patterns);
  const vueTsc = runVueTsc(check.cwd);

  if (options.expectErrors) {
    assert.equal(vize.status, 1, vize.stderr);
    assert.notEqual(vueTsc.status, 0, vueTsc.stderr || vueTsc.stdout);
  } else {
    assert.equal(vize.status, 0, vize.stderr);
    assert.equal(vueTsc.status, 0, vueTsc.stderr || vueTsc.stdout);
  }

  const vizeJson = parseVizeCheckJson(vize.stdout);
  const vizeFiles = collectVizeDiagnosticCodes(vizeJson, check.cwd);
  const vueTscFiles = collectVueTscDiagnosticCodes(vueTsc.stdout + vueTsc.stderr, check.cwd);

  assert.deepEqual(
    [...vizeFiles.keys()].sort(),
    [...vueTscFiles.keys()].sort(),
    `${app.name} should report diagnostics for the same Vue files as vue-tsc`,
  );

  for (const [file, vizeCodes] of vizeFiles) {
    const vueCodes = vueTscFiles.get(file) ?? new Set<string>();
    const sharedCodes = [...vizeCodes].filter((code) => vueCodes.has(code));
    assert.ok(
      sharedCodes.length > 0,
      `${app.name}:${file} should share at least one TypeScript diagnostic code with vue-tsc`,
    );
  }
}

function runVizeCheck(cwd: string, patterns: string[]): CommandResult {
  return runCommand(
    VIZE_BIN,
    ["check", ...patterns, "--format", "json", "--quiet", "--corsa-path", CORSA_BIN],
    cwd,
  );
}

function runVueTsc(cwd: string): CommandResult {
  const vueTsc = resolveBin("vue-tsc");
  assert.ok(vueTsc, "vue-tsc binary should exist");
  return runCommand(vueTsc, ["--noEmit", "--pretty", "false", "-p", "tsconfig.json"], cwd);
}

function runCommand(command: string, args: string[], cwd: string): CommandResult {
  const result = spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    env: {
      ...process.env,
      LANG: "C",
      LC_ALL: "C",
    },
    maxBuffer: 128 * 1024 * 1024,
    timeout: 120_000,
  });

  if (result.error != null) {
    throw result.error;
  }

  return {
    status: result.status,
    stderr: result.stderr,
    stdout: result.stdout,
  };
}

function parseVizeCheckJson(stdout: string): VizeCheckJson {
  assert.ok(stdout.trim().length > 0, "vize check should print JSON");
  return JSON.parse(stdout) as VizeCheckJson;
}

function collectVizeDiagnosticCodes(result: VizeCheckJson, cwd: string): Map<string, Set<string>> {
  const files = new Map<string, Set<string>>();
  for (const file of result.files) {
    const codes = new Set<string>();
    for (const diagnostic of file.diagnostics) {
      for (const match of diagnostic.matchAll(/\[(TS\d+)\]/g)) {
        codes.add(match[1]!);
      }
    }
    if (codes.size > 0) {
      files.set(normalizeFile(file.file, cwd), codes);
    }
  }
  return files;
}

function collectVueTscDiagnosticCodes(output: string, cwd: string): Map<string, Set<string>> {
  const files = new Map<string, Set<string>>();
  for (const line of output.split(/\r?\n/)) {
    const match = /^(.*?\.vue)\(\d+,\d+\): error (TS\d+):/.exec(line);
    if (!match) {
      continue;
    }
    const file = normalizeFile(match[1]!, cwd);
    let codes = files.get(file);
    if (codes == null) {
      codes = new Set<string>();
      files.set(file, codes);
    }
    codes.add(match[2]!);
  }
  return files;
}

function collectVueFiles(cwd: string, patterns: string[]): string[] {
  const files = new Set<string>();
  for (const pattern of patterns) {
    const root = patternRoot(cwd, pattern);
    if (!fs.existsSync(root)) {
      continue;
    }
    visitVueFiles(root, files);
  }
  return [...files].sort();
}

function visitVueFiles(dir: string, files: Set<string>): void {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      visitVueFiles(fullPath, files);
    } else if (entry.isFile() && entry.name.endsWith(".vue")) {
      files.add(fullPath);
    }
  }
}

function patternRoot(cwd: string, pattern: string): string {
  const globIndex = pattern.search(/[*{[]/);
  if (globIndex === -1) {
    return path.resolve(cwd, path.dirname(pattern));
  }
  const prefix = pattern.slice(0, globIndex);
  const root = prefix.endsWith("/") ? prefix : path.dirname(prefix);
  return path.resolve(cwd, root === "." ? "" : root);
}

function normalizeFile(file: string, cwd: string): string {
  const absolute = path.isAbsolute(file) ? file : path.resolve(cwd, file);
  return path.relative(cwd, absolute).split(path.sep).join("/");
}

function requireCheckConfig(app: AppConfig): NonNullable<AppConfig["check"]> {
  assert.ok(app.check, `${app.name} should define check fixtures`);
  return app.check;
}

function resolveBin(name: string): string | null {
  const testsRoot = path.resolve(__dirname, "..");
  const repoRoot = path.resolve(testsRoot, "..");
  const binNames = process.platform === "win32" ? [`${name}.cmd`, name] : [name];
  const candidates = binNames.flatMap((binName) => [
    path.join(testsRoot, "node_modules", ".bin", binName),
    path.join(repoRoot, "node_modules", ".bin", binName),
  ]);
  return candidates.find((candidate) => fs.existsSync(candidate)) ?? null;
}

function formatCompilerError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
