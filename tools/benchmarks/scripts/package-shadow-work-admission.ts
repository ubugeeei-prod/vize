/** Exact controlled state/work attribution; no native checker or timing acceptance. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const modulePath = "crates/vize_canon/src/batch/virtual_project/workspace_aliases.rs";
const testsPath = modulePath.slice(0, -3) + "/dirty_propagation_tests";
const inputPath =
  "tests/_fixtures/differential/typechecker/cold-package-shadow-routes-11265/input.json";
const observerFiles = [
  testsPath + ".rs",
  testsPath + "/fixture.rs",
  testsPath + "/fixture/capture.rs",
  inputPath,
];
const states = [
  "cold",
  "warm-noop",
  "warm-rebind",
  "owner-removed",
  "owner-repair",
  "all-removed",
  "all-repair",
];
const runtimeTests = [
  "canonical_self_shadow_keeps_generated_bytes_and_authored_diagnostic_positions",
  "relocated_typescript_shadow_keeps_authored_bytes_and_rejects_the_generated_map",
  "same_line_import_growth_maps_across_unicode_and_crlf_without_losing_the_file",
].map((name) => "batch::virtual_project::package_shadow_runtime::tests::" + name);
const captureTest =
  "batch::virtual_project::workspace_aliases::dirty_propagation_tests::fixture::capture::complete_prepared_graph_observations";
const hash = (body: string | Buffer) => createHash("sha256").update(body).digest("hex");
const fileHash = (file: string) => hash(fs.readFileSync(file));

export function insertOnce(source: string, before: string, after: string): string {
  assert.equal(
    source.split(before).length,
    2,
    "observer insertion must have one exact source anchor",
  );
  return source.replace(before, after);
}

export function baselineObserver(source: string): {
  observed: string;
  edits: Array<[string, string]>;
} {
  const edits: Array<[string, string]> = [
    ["mod install_roots;", "#[cfg(test)]\nmod dirty_propagation_tests;\nmod install_roots;"],
    [
      "        let mut pending = keys.into_iter().collect::<Vec<_>>();",
      "        #[cfg(test)]\n        dirty_propagation_tests::record_call(!self.package_shadows_initialized);\n        let mut pending = keys.into_iter().collect::<Vec<_>>();",
    ],
    [
      "        while let Some(key) = pending.pop() {",
      "        while let Some(key) = pending.pop() {\n            #[cfg(test)]\n            dirty_propagation_tests::record_pop();",
    ],
  ];
  assert.equal(source.includes("mod dirty_propagation_tests;"), false);
  assert.equal(source.includes("dirty_propagation_tests::record_"), false);
  let observed = source;
  for (const [before, after] of edits) observed = insertOnce(observed, before, after);
  let inverse = observed;
  for (const [before, after] of [...edits].reverse()) inverse = insertOnce(inverse, after, before);
  assert.equal(
    inverse,
    source,
    "ENTIRE original production source must survive the test-only overlay",
  );
  return { observed, edits };
}

export function verifyPair(before: Buffer, after: Buffer, state: string, root: string): void {
  assert.equal(
    after.equals(before),
    true,
    "whole controlled state bytes must match without normalization; inspect both retained packets",
  );
  const packet = JSON.parse(after.toString("utf8"));
  assert.deepEqual(
    Object.keys(packet).sort(),
    [
      "fixtureRoot",
      "nativeCheckerExecuted",
      "reporterProjectReproduced",
      "schema",
      "snapshot",
      "state",
      "version",
      "virtualRoot",
    ].sort(),
  );
  assert.equal(packet.schema, "vize.controlled-package-shadow-state");
  assert.equal(packet.version, 1);
  assert.equal(packet.state, state);
  assert.equal(packet.fixtureRoot, root);
  const cacheRelative = path.relative(path.join(path.dirname(root), "cache"), packet.virtualRoot);
  assert.equal(
    cacheRelative === "" || cacheRelative.startsWith("..") || path.isAbsolute(cacheRelative),
    false,
    "the observed virtual root must remain inside the fresh owned cache",
  );
  assert.equal(packet.nativeCheckerExecuted, false);
  assert.equal(packet.reporterProjectReproduced, false);
  assert.deepEqual(
    Object.keys(packet.snapshot).sort(),
    [
      "routes",
      "files",
      "originals",
      "shadows",
      "manifests",
      "fileOwners",
      "manifestOwners",
      "sourceIndex",
      "sourceOwners",
      "sourcePaths",
      "rootOwners",
      "importerOwners",
      "manifestRoutes",
      "shadowSources",
      "shadowScopes",
      "artifacts",
      "reverse",
      "aliases",
    ].sort(),
  );
  for (const field of Object.values(packet.snapshot)) assert.equal(typeof field, "string");
}

type Options = {
  sourceRoot: string;
  out: string;
  baseSha: string;
  headExecutable: string;
  compilerEnvironment: NodeJS.ProcessEnv;
  buildArgs: string[];
};

export function qualifyPackageShadowWork(options: Options): void {
  const { sourceRoot, baseSha, headExecutable, compilerEnvironment, buildArgs } = options;
  assert.match(baseSha, /^[a-f0-9]{40}$/u);
  const out = path.resolve(options.out);
  assert.equal(fs.existsSync(out), false);
  fs.mkdirSync(out, { recursive: true });
  const save = (name: string, value: unknown) =>
    fs.writeFileSync(path.join(out, name), JSON.stringify(value, null, 2) + "\n");
  const git = (cwd: string, ...argv: string[]) => {
    const result = spawnSync("git", argv, { cwd, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 });
    assert.equal(result.status, 0, result.stderr);
    return result.stdout;
  };
  const head = git(sourceRoot, "rev-parse", "HEAD").trim();
  const headTree = git(sourceRoot, "rev-parse", "HEAD^{tree}").trim();
  const associatedHead = process.env.PACKAGE_ROUTE_ASSOCIATED_HEAD;
  assert.match(associatedHead ?? "", /^[a-f0-9]{40}$/u);
  const parents = git(sourceRoot, "cat-file", "-p", "HEAD")
    .split("\n")
    .filter((line) => line.startsWith("parent "))
    .map((line) => line.slice(7));
  if (process.env.GITHUB_EVENT_NAME === "pull_request") {
    assert.deepEqual(parents, [baseSha, associatedHead]);
  } else {
    assert.equal(head, associatedHead);
  }
  assert.equal(git(sourceRoot, "status", "--porcelain", "--untracked-files=all"), "");
  const fetched = spawnSync(
    "git",
    ["fetch", "--no-tags", "--depth=1", "https://github.com/ubugeeei-prod/vize.git", baseSha],
    { cwd: sourceRoot, encoding: "utf8", timeout: 120_000 },
  );
  fs.writeFileSync(path.join(out, "fetch.stdout.txt"), fetched.stdout ?? "");
  fs.writeFileSync(path.join(out, "fetch.stderr.txt"), fetched.stderr ?? "");
  assert.equal(fetched.status, 0);
  const original = git(sourceRoot, "show", baseSha + ":" + modulePath);
  const current = fs.readFileSync(path.join(sourceRoot, modulePath), "utf8");
  const coldGuard = "if !self.package_shadows_initialized {";
  if (original.includes(coldGuard) || !current.includes(coldGuard)) {
    save("selection.json", {
      head,
      headTree,
      associatedHead,
      executionParents: parents,
      baseSha,
      selected: false,
      reason: "not the introduction of cold propagation deferral",
      nativeCheckerExecuted: false,
    });
    return;
  }
  const changed = git(
    sourceRoot,
    "diff",
    "--name-only",
    baseSha,
    head,
    "--",
    "crates/vize_canon/src",
  )
    .trim()
    .split("\n");
  assert.deepEqual(
    changed.sort(),
    [modulePath, ...observerFiles.filter((file) => file.startsWith("crates/"))].sort(),
  );
  for (const file of ["Cargo.toml", "Cargo.lock"]) {
    assert.equal(
      git(sourceRoot, "show", baseSha + ":" + file),
      fs.readFileSync(path.join(sourceRoot, file), "utf8"),
      file,
    );
  }
  const baseRoot = path.join(out, "base-source");
  git(sourceRoot, "worktree", "add", "--detach", baseRoot, baseSha);
  assert.equal(git(baseRoot, "status", "--porcelain", "--untracked-files=all"), "");
  const baseTree = git(baseRoot, "rev-parse", "HEAD^{tree}").trim();
  const overlay = baselineObserver(original);
  fs.writeFileSync(path.join(out, "base-original-module.rs"), original);
  fs.writeFileSync(path.join(out, "base-observed-module.rs"), overlay.observed);
  fs.writeFileSync(path.join(baseRoot, modulePath), overlay.observed);
  const copied = observerFiles.map((file) => {
    const body = fs.readFileSync(path.join(sourceRoot, file));
    const destination = path.join(baseRoot, file);
    assert.equal(fs.existsSync(destination), false, "baseline observer destinations must be new");
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, body);
    return { path: file, bytes: body.length, sha256: hash(body) };
  });
  save("source-and-overlay.json", {
    head,
    headTree,
    associatedHead,
    executionParents: parents,
    baseSha,
    baseTree,
    originalModuleSha256: hash(original),
    observedModuleSha256: hash(overlay.observed),
    edits: overlay.edits,
    copied,
    nativeCheckerExecuted: false,
    overlayKind:
      "test module/input plus cfg(test) call/pop observation; production body whole inverse verified",
    baselinePushCountsAvailable: false,
  });
  const target = path.join(out, "base-cargo-target");
  assert.equal(fs.existsSync(target), false);
  const environment = { ...compilerEnvironment, CARGO_TARGET_DIR: target };
  const build = spawnSync("cargo", buildArgs, {
    cwd: baseRoot,
    env: environment,
    encoding: "utf8",
    timeout: 1_200_000,
    maxBuffer: 128 * 1024 * 1024,
    killSignal: "SIGKILL",
  });
  fs.writeFileSync(path.join(out, "base-cargo.stdout.jsonl"), build.stdout ?? "");
  fs.writeFileSync(path.join(out, "base-cargo.stderr.txt"), build.stderr ?? "");
  save("base-build-command.json", {
    command: "cargo",
    argv: buildArgs,
    cwd: baseRoot,
    target,
    status: build.status,
    signal: build.signal,
    error: build.error?.message ?? null,
    freshTarget: true,
    timingComparison: false,
  });
  assert.equal(build.error, undefined);
  assert.equal(build.signal, null);
  assert.equal(build.status, 0, "inspect complete retained base build output");
  const artifacts = (build.stdout ?? "").split("\n").flatMap((line) => {
    if (!line.startsWith("{")) return [];
    const row = JSON.parse(line);
    return row.reason === "compiler-artifact" &&
      row.target?.name === "vize_canon" &&
      row.profile?.test &&
      row.executable
      ? [row]
      : [];
  });
  assert.equal(artifacts.length, 1);
  const artifact = artifacts[0];
  assert.equal(artifact.fresh, false);
  assert.equal(artifact.features.includes("native"), true);
  const baseExecutable = fs.realpathSync(artifact.executable);
  assert.equal(path.relative(target, baseExecutable).startsWith(".."), false);
  const images = {
    base: { executable: baseExecutable, sha256: fileHash(baseExecutable) },
    head: { executable: headExecutable, sha256: fileHash(headExecutable) },
  };
  save("images.json", { images, artifact, timingComparison: false, nativeCheckerExecuted: false });
  const runtime = path.join(out, "runtime");
  const root = path.join(runtime, "fixture"),
    cache = path.join(runtime, "cache");
  const commands = [];
  for (const role of ["base", "head"] as const) {
    fs.rmSync(runtime, { recursive: true, force: true });
    fs.mkdirSync(root, { recursive: true });
    fs.mkdirSync(cache);
    assert.equal(fs.realpathSync(root), root);
    assert.deepEqual(fs.readdirSync(cache), []);
    const capture = path.join(out, role);
    const env = {
      ...compilerEnvironment,
      XDG_CACHE_HOME: cache,
      VIZE_PACKAGE_ROUTE_GRAPH_ROOT: root,
      VIZE_PACKAGE_ROUTE_GRAPH_CAPTURE: capture,
    };
    for (const [index, name] of [captureTest, ...runtimeTests].entries()) {
      const argv = ["--exact", name, "--nocapture", "--test-threads", "1"];
      const result = spawnSync(images[role].executable, argv, {
        cwd: role === "base" ? baseRoot : sourceRoot,
        env,
        encoding: "utf8",
        timeout: 120_000,
        maxBuffer: 64 * 1024 * 1024,
      });
      fs.writeFileSync(path.join(out, `${role}-${index}.stdout.txt`), result.stdout ?? "");
      fs.writeFileSync(path.join(out, `${role}-${index}.stderr.txt`), result.stderr ?? "");
      commands.push({
        role,
        executable: images[role].executable,
        argv,
        cwd: role === "base" ? baseRoot : sourceRoot,
        environment: {
          XDG_CACHE_HOME: cache,
          VIZE_PACKAGE_ROUTE_GRAPH_ROOT: root,
          VIZE_PACKAGE_ROUTE_GRAPH_CAPTURE: capture,
        },
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
      });
      save("commands.json", commands);
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      assert.equal(result.status, 0);
      assert.match(result.stdout ?? "", /test result: ok\. 1 passed; 0 failed; 0 ignored;/u);
    }
    assert.equal(fileHash(images[role].executable), images[role].sha256);
    const input = JSON.parse(fs.readFileSync(path.join(sourceRoot, inputPath), "utf8"));
    for (const file of input.files)
      assert.deepEqual(
        fs.readFileSync(path.join(root, file.path)),
        Buffer.from(file.source),
        "phase input bytes must remain unchanged",
      );
    const links = [
      "node_modules/.pnpm/@fixture+kit@1/node_modules/@fixture/kit",
      "node_modules/@fixture/kit",
    ].map((file) => {
      const link = path.join(root, file);
      assert.equal(fs.lstatSync(link).isSymbolicLink(), true);
      return { path: file, target: fs.readlinkSync(link), realpath: fs.realpathSync(link) };
    });
    save(`${role}-runtime-inputs.json`, {
      root,
      cacheStartedEmpty: true,
      fixture: fileHash(path.join(sourceRoot, inputPath)),
      links,
      files: input.files.map((file: { path: string; source: string }) => ({
        path: file.path,
        bytes: Buffer.byteLength(file.source),
        sha256: hash(file.source),
      })),
    });
  }
  const packets = states.map((state) => {
    const before = fs.readFileSync(path.join(out, "base", state + ".state.json"));
    const after = fs.readFileSync(path.join(out, "head", state + ".state.json"));
    verifyPair(before, after, state, root);
    return { state, bytes: after.length, sha256: hash(after), wholeEqual: true };
  });
  const beforeWork = JSON.parse(fs.readFileSync(path.join(out, "base/work.json"), "utf8"));
  const afterWork = JSON.parse(fs.readFileSync(path.join(out, "head/work.json"), "utf8"));
  assert.equal(beforeWork.registrationWork.coldCalls > 0, true);
  assert.equal(beforeWork.registrationWork.popped > 0, true);
  assert.equal(afterWork.registrationWork.coldCalls, beforeWork.registrationWork.coldCalls);
  assert.equal(afterWork.registrationWork.popped, 0);
  assert.equal(afterWork.registrationWork.pushed, 0);
  assert.equal(git(sourceRoot, "rev-parse", "HEAD").trim(), head);
  assert.equal(git(sourceRoot, "status", "--porcelain", "--untracked-files=all"), "");
  assert.equal(fs.readFileSync(path.join(baseRoot, modulePath), "utf8"), overlay.observed);
  for (const file of copied) assert.equal(fileHash(path.join(baseRoot, file.path)), file.sha256);
  save("acceptance.json", {
    source: { head, headTree, associatedHead, executionParents: parents, baseSha, baseTree },
    images,
    packets,
    registration: {
      base: beforeWork.registrationWork,
      head: afterWork.registrationWork,
      baselinePushCountsAvailable: false,
    },
    commands: commands.length,
    exactRuntimeGuardsPerRole: runtimeTests,
    wholeControlledStateEqual: true,
    nativeCheckerExecuted: false,
    reporterProjectReproduced: false,
    wallTimeOrRssImprovementEstablished: false,
  });
}
