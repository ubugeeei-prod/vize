/** Compile the same public guard snippet against normal, old-control and new libraries. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {
  checkExactJson,
  digest,
  isMain,
  list,
  parseExact,
  record,
  render,
  string,
  writeExact,
} from "./profile-evidence-json.ts";

type Artifact = ReturnType<typeof record>;
function invoke(command: string[], stdout: string, stderr: string): number {
  const result = spawnSync(command[0]!, command.slice(1), {
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  fs.writeFileSync(stdout, result.stdout ?? "");
  fs.writeFileSync(stderr, result.stderr ?? "");
  assert(result.error == null, result.error?.message);
  assert(result.status != null, "process ended with " + result.signal);
  return result.status;
}
function run(command: string[]): void {
  const result = spawnSync(command[0]!, command.slice(1), { stdio: "inherit" });
  assert(result.error == null, result.error?.message);
  assert.equal(result.status, 0);
}
function libraryArtifact(log: string): { artifact: Artifact; library: string } {
  const matches: Artifact[] = [];
  for (const line of fs.readFileSync(log, "utf8").split("\n")) {
    let row: Artifact;
    try {
      row = record(parseExact(line));
    } catch {
      continue;
    }
    const target = row.target == null ? null : record(row.target);
    if (
      row.reason === "compiler-artifact" &&
      target?.name === "vize_l0" &&
      render(target.kind) === '["lib"]'
    )
      matches.push(row);
  }
  assert.equal(matches.length, 1, log);
  const artifact = matches[0]!;
  const libraries = list(artifact.filenames)
    .map(string)
    .filter((name) => name.endsWith(".rlib"));
  assert.equal(libraries.length, 1);
  assert(fs.statSync(libraries[0]!).isFile());
  return { artifact, library: libraries[0]! };
}
function compileProbe(
  role: string,
  artifact: Artifact,
  library: string,
  snippet: string,
  evidence: string,
) {
  const directories = new Set(
    list(artifact.filenames)
      .map(string)
      .map(path.dirname)
      .filter((name) => path.basename(name) === "deps"),
  );
  assert.equal(directories.size, 1);
  const dependencies = [...directories][0]!;
  assert(fs.statSync(dependencies).isDirectory());
  const destination = path.join(evidence, "send-" + role);
  fs.mkdirSync(destination);
  const stdout = path.join(evidence, "send-" + role + ".stdout");
  const stderr = path.join(evidence, "send-" + role + ".jsonl");
  const code = invoke(
    [
      "rustc",
      "+1.99.1",
      "--edition=2024",
      "--crate-name",
      "guard_send_probe",
      "--emit=metadata",
      "--error-format=json",
      "--out-dir",
      destination,
      "--extern",
      "vize_l0=" + library,
      "-L",
      "dependency=" + dependencies,
      snippet,
    ],
    stdout,
    stderr,
  );
  const diagnostics: { code: string; message: string }[] = [];
  for (const line of fs.readFileSync(stderr, "utf8").split("\n")) {
    let row: Artifact;
    try {
      row = record(parseExact(line));
    } catch {
      continue;
    }
    if (row.level === "error" && row.code != null)
      diagnostics.push({ code: string(record(row.code).code), message: string(row.message) });
  }
  return {
    returnCode: code,
    diagnostics,
    rlib: library,
    rlibSha256: digest(library),
    dependencyDirectory: dependencies,
    artifact,
  };
}
function env(name: string): string {
  const value = process.env[name];
  assert(value != null);
  return value;
}
export function main(): void {
  checkExactJson();
  const root = process.cwd(),
    evidence = path.join(root, "evidence");
  const headResult = spawnSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" });
  assert.equal(headResult.status, 0);
  const head = headResult.stdout.trim();
  const normal = libraryArtifact(path.join(evidence, "normal-law.log"));
  const instrumented = libraryArtifact(path.join(evidence, "instrumented-law.log"));
  assert(!list(normal.artifact.features).includes("micro-profiling"));
  assert(list(instrumented.artifact.features).includes("micro-profiling"));
  assert.notEqual(normal.library, instrumented.library);
  for (const { artifact } of [normal, instrumented]) {
    assert.equal(artifact.manifest_path, path.join(root, "davinci/vize_l0/Cargo.toml"));
    assert.equal(record(artifact.target).src_path, path.join(root, "davinci/vize_l0/src/lib.rs"));
  }
  const control = path.join(
    env("RUNNER_TEMP"),
    "old-guard-" + env("GITHUB_RUN_ID") + "-" + env("GITHUB_RUN_ATTEMPT"),
  );
  assert(!fs.existsSync(control));
  run(["git", "worktree", "add", "--detach", control, head]);
  const relative = "davinci/vize_l0/src/profiler/core.rs",
    core = path.join(control, relative);
  const original = fs.readFileSync(core, "utf8");
  const field =
    '    #[cfg(feature = "micro-profiling")]\n    _same_thread: std::marker::PhantomData<std::rc::Rc<()>>,\n';
  const initializer =
    '            #[cfg(feature = "micro-profiling")]\n            _same_thread: std::marker::PhantomData,\n';
  assert.equal(original.split(field).length - 1, 1);
  assert.equal(original.split(initializer).length - 1, 1);
  const previous = original.replace(field, "").replace(initializer, "");
  fs.writeFileSync(core, previous);
  const diff = spawnSync("git", ["-C", control, "diff", "--", relative], { encoding: "utf8" });
  assert.equal(diff.status, 0);
  fs.writeFileSync(path.join(evidence, "old-guard-mutation.patch"), diff.stdout);
  const oldLog = path.join(evidence, "old-guard-build.jsonl");
  const oldCode = invoke(
    [
      "cargo",
      "+1.99.1",
      "build",
      "--locked",
      "--profile",
      "ci",
      "-p",
      "vize_l0",
      "--lib",
      "--no-default-features",
      "--features",
      list(instrumented.artifact.features).map(string).join(","),
      "--manifest-path",
      path.join(control, "Cargo.toml"),
      "--target-dir",
      path.join(root, "target/guard-old-control"),
      "--message-format",
      "json",
    ],
    oldLog,
    path.join(evidence, "old-guard-build.stderr"),
  );
  assert.equal(oldCode, 0);
  const old = libraryArtifact(oldLog);
  assert.deepEqual(old.artifact.features, instrumented.artifact.features);
  assert.equal(old.artifact.manifest_path, path.join(control, "davinci/vize_l0/Cargo.toml"));
  const snippet = path.join(evidence, "guard-send-snippet.rs");
  fs.writeFileSync(
    snippet,
    "use vize_l0::profiler::ProfileGuard;\nfn require_send<T: Send>() {}\nfn main() { require_send::<ProfileGuard>(); }\n",
  );
  const roles = {
    normal: compileProbe("normal", normal.artifact, normal.library, snippet, evidence),
    "old-profiling-control": compileProbe("old", old.artifact, old.library, snippet, evidence),
    instrumented: compileProbe(
      "instrumented",
      instrumented.artifact,
      instrumented.library,
      snippet,
      evidence,
    ),
  };
  const hash = (text: string) => createHash("sha256").update(text).digest("hex");
  const receipt = {
    source: head,
    snippetSha256: digest(snippet),
    originalCoreSha256: hash(original),
    mutatedCoreSha256: hash(previous),
    mutation:
      "Remove only the added zero-sized field and initializer; keep profiling features equal",
    roles,
  };
  writeExact(path.join(evidence, "guard-send-receipt.json"), receipt);
  for (const role of [roles.normal, roles["old-profiling-control"]]) {
    assert.equal(role.returnCode, 0);
    assert.deepEqual(role.diagnostics, []);
  }
  assert.notEqual(roles.instrumented.returnCode, 0);
  assert.deepEqual(
    roles.instrumented.diagnostics.map((row) => row.code),
    ["E0277"],
  );
  assert(roles.instrumented.diagnostics[0]!.message.includes("Rc"));
  assert(roles.instrumented.diagnostics[0]!.message.includes("cannot be sent"));
  console.log("guard-send-mutation=" + render(receipt));
}
if (isMain(import.meta.url)) main();
