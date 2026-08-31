import { execSync } from "node:child_process";
import { existsSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, "..", "..", "..");
const GIT_FIXTURE_DIR = join(repoRoot, "tests", "_fixtures", "_git");
const BIN_EXT = process.platform === "win32" ? ".exe" : "";
const VIZE_RELEASE_BIN = join(repoRoot, "target", "release", `vize${BIN_EXT}`);
const VIZE_CI_BIN = join(repoRoot, "target", "ci", `vize${BIN_EXT}`);
const VIZE_DEBUG_BIN = join(repoRoot, "target", "debug", `vize${BIN_EXT}`);

export const VIZE_BIN =
  process.env.VIZE_BIN ??
  [VIZE_CI_BIN, VIZE_RELEASE_BIN, VIZE_DEBUG_BIN].find((candidate) => existsSync(candidate)) ??
  VIZE_RELEASE_BIN;

export const REAL_WORLD_TYPECHECK_FIXTURES: RealWorldTypecheckFixture[] = [
  {
    name: "voicevox",
    cwd: join(GIT_FIXTURE_DIR, "voicevox"),
    patterns: ["src/**/*.vue"],
    tsconfig: "tsconfig.json",
    timeoutMs: 300_000,
  },
  {
    name: "elk",
    cwd: join(GIT_FIXTURE_DIR, "elk"),
    patterns: ["app/**/*.vue"],
    tsconfig: "tsconfig.json",
    timeoutMs: 300_000,
  },
  {
    name: "misskey",
    cwd: join(GIT_FIXTURE_DIR, "misskey", "packages", "frontend"),
    patterns: ["src/**/*.vue"],
    tsconfig: "tsconfig.json",
    timeoutMs: 300_000,
  },
  {
    name: "vue-vben-admin",
    cwd: join(GIT_FIXTURE_DIR, "vue-vben-admin"),
    patterns: ["playground/src/**/*.vue", "apps/**/*.vue", "packages/**/*.vue"],
    timeoutMs: 300_000,
  },
  {
    name: "hoppscotch",
    cwd: join(GIT_FIXTURE_DIR, "hoppscotch"),
    patterns: ["packages/**/*.vue"],
    timeoutMs: 300_000,
  },
  {
    name: "element-plus",
    cwd: join(GIT_FIXTURE_DIR, "element-plus"),
    patterns: ["packages/**/*.vue", "docs/**/*.vue", "ssr-testing/**/*.vue"],
    tsconfig: "tsconfig.json",
    timeoutMs: 300_000,
  },
];

interface RealWorldTypecheckFixture {
  name: string;
  cwd: string;
  patterns: string[];
  tsconfig?: string;
  timeoutMs: number;
}

interface RealWorldTypecheckResult {
  name: string;
  status: "ok" | "skipped" | "crashed" | "timed-out";
  ms: number;
  fileCount: number;
  errorCount: number;
  reason?: string;
}

export function runVizeRealWorldTypecheck(
  fixture: RealWorldTypecheckFixture,
): RealWorldTypecheckResult {
  if (!existsSync(VIZE_BIN)) {
    return skippedFixture(fixture.name, "vize CLI not found");
  }

  if (!existsSync(fixture.cwd)) {
    return skippedFixture(fixture.name, "fixture not found");
  }

  const patterns = fixture.patterns.map(shellQuote).join(" ");
  const tsconfig = fixture.tsconfig ? ` --tsconfig ${shellQuote(fixture.tsconfig)}` : "";
  const cmd = `${shellQuote(VIZE_BIN)} check ${patterns} --format json --quiet${tsconfig}`;
  const start = performance.now();
  let stdout = "";

  try {
    stdout = execSync(cmd, {
      cwd: fixture.cwd,
      encoding: "utf8",
      maxBuffer: 100 * 1024 * 1024,
      timeout: fixture.timeoutMs,
    });
  } catch (error: unknown) {
    const commandError = error as {
      status?: number;
      stdout?: { toString(): string };
      stderr?: { toString(): string };
      signal?: string;
    };
    const ms = performance.now() - start;

    if (commandError.status === 1 && commandError.stdout) {
      stdout = commandError.stdout.toString();
    } else {
      return {
        name: fixture.name,
        status: commandError.signal === "SIGTERM" ? "timed-out" : "crashed",
        ms,
        fileCount: countVueFiles(fixture.cwd),
        errorCount: 0,
        reason: commandError.stderr?.toString().trim().split("\n").slice(-1)[0],
      };
    }
  }

  const ms = performance.now() - start;
  const parsed = JSON.parse(stdout) as { fileCount?: number; errorCount?: number };
  return {
    name: fixture.name,
    status: "ok",
    ms,
    fileCount: parsed.fileCount ?? countVueFiles(fixture.cwd),
    errorCount: parsed.errorCount ?? 0,
  };
}

function skippedFixture(name: string, reason: string): RealWorldTypecheckResult {
  return { name, status: "skipped", ms: 0, fileCount: 0, errorCount: 0, reason };
}

function shellQuote(value: string): string {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

function countVueFiles(dir: string): number {
  if (!existsSync(dir)) return 0;

  let count = 0;
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      count += countVueFiles(join(dir, entry.name));
    } else if (entry.name.endsWith(".vue")) {
      count += 1;
    }
  }
  return count;
}
