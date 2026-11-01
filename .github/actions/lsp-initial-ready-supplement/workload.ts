import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { object, hash, type Obj } from "./fixtures.ts";
import { memory } from "./resources.ts";
import { errorText, collectCaptures, type Session } from "./native-queries.ts";
export function parseArgs(args: readonly string[]): {
  mode: "memory";
  repo: string;
  output: string;
  source: string;
  attempt: number;
} {
  assert.equal(args.length, 10);
  const values: Record<string, string> = {};
  for (let i = 0; i < args.length; i += 2) {
    const key = args[i]!;
    assert.ok(["--mode", "--repo", "--output", "--source", "--attempt"].includes(key));
    assert.equal(values[key], undefined);
    values[key] = args[i + 1]!;
  }
  assert.ok(["memory"].includes(values["--mode"]!));
  assert.match(values["--source"]!, /^[a-f0-9]{40}$/);
  assert.ok([1, 2, 3, 4].includes(Number(values["--attempt"])));
  assert.ok(path.isAbsolute(values["--repo"]!) && path.isAbsolute(values["--output"]!));
  return {
    mode: "memory",
    repo: values["--repo"]!,
    output: values["--output"]!,
    source: values["--source"]!,
    attempt: Number(values["--attempt"]),
  };
}
async function modules(
  repo: string,
): Promise<{ Session: new () => Session; linkVue: (root: string) => void }> {
  const session = object(
    await import(pathToFileURL(path.join(repo, "tests/tooling/support/lsp/session.ts")).href),
  );
  const types = object(
    await import(pathToFileURL(path.join(repo, "tests/_helpers/realworld-typecheck.ts")).href),
  );
  assert.equal(typeof session.LspSession, "function");
  assert.equal(typeof types.symlinkVueTypes, "function");
  return {
    Session: session.LspSession as new () => Session,
    linkVue: types.symlinkVueTypes as (root: string) => void,
  };
}
const save = (file: string, value: unknown): void =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx", mode: 0o600 });
async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2));
  assert.equal(fs.realpathSync(args.repo), fs.realpathSync(process.cwd()));
  assert.equal(
    execFileSync("git", ["rev-parse", "HEAD"], { cwd: args.repo, encoding: "utf8" }).trim(),
    args.source,
  );
  assert.equal(process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  assert.equal(process.env.VIZE_LSP_BUILD_RECIPE, "cargo build --release -p vize");
  assert.equal(process.env.VIZE_TRACE_EDITOR_PREPARATION, "1");
  assert.equal(process.env.VIZE_PERF_BUDGET_SCALE, undefined);
  const captureRoot = path.join(args.repo, "target/differential/lsp-sessions");
  assert.ok(!fs.existsSync(captureRoot));
  const binary = process.env.VIZE_LSP_BIN;
  assert.ok(binary);
  const receipt = object(JSON.parse(fs.readFileSync(`${binary}.differential-build.json`, "utf8")));
  assert.equal(receipt.sourceRevision, args.source);
  const image = String(receipt.binarySha256);
  assert.equal(hash(fs.readFileSync(binary)), image);
  const api = await modules(args.repo);
  const rows: Obj[] = [];
  const errors: string[] = [];
  const outcome: Obj = {
    schemaVersion: 1,
    workloadVersion: 2,
    source: args.source,
    mode: args.mode,
    attempt: args.attempt,
    complete: false,
    errors,
    rows,
    rawCaptures: [],
    scope: {
      firstArmRerun: false,
      osCacheCold: "UNKNOWN",
      initialCompletionProjectionAuthority: "UNAVAILABLE",
      droppedRetry: "UNKNOWN",
      nativeCpuEntered: "UNKNOWN",
    },
  };
  try {
    rows.push(await memory(args, api));
  } catch (error) {
    errors.push(errorText(error));
  } finally {
    for (const row of rows) for (const error of object(row).errors as string[]) errors.push(error);
    try {
      outcome.rawCaptures = collectCaptures(args, image);
      assert.equal((outcome.rawCaptures as Obj[]).length, 1);
    } catch (error) {
      errors.push(errorText(error));
    }
    outcome.complete = rows.length === 1 && errors.length === 0;
    save(path.join(args.output, "outcome.json"), outcome);
  }
  assert.equal(
    outcome.complete,
    true,
    "all memory cohort/raw/source/resource laws required; failures are retained",
  );
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
